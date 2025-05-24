# Event System Catalog and Architecture

## Overview

The event system provides decoupled communication between game systems using a priority-based, buffered event bus. Events are processed in batches with well-defined ordering to ensure consistency.

## Event Architecture

### Core Event System

```rust
pub struct EventBus {
    // Event queues by priority
    immediate_queue: VecDeque<Event>,
    high_priority_queue: BinaryHeap<PrioritizedEvent>,
    normal_queue: VecDeque<Event>,
    low_priority_queue: VecDeque<Event>,
    
    // Deferred events
    deferred_queue: Vec<DeferredEvent>,
    
    // Event handlers
    handlers: HashMap<TypeId, Vec<EventHandler>>,
    
    // Performance tracking
    event_counts: HashMap<TypeId, u32>,
    processing_times: HashMap<TypeId, Duration>,
}

pub struct PrioritizedEvent {
    pub event: Event,
    pub priority: f32,
    pub timestamp: Instant,
}

pub struct DeferredEvent {
    pub event: Event,
    pub execute_at: Instant,
}

pub enum EventPriority {
    Immediate,    // Process before frame update
    High,         // Process first in frame
    Normal,       // Process in order received
    Low,          // Process after other events
    Deferred(Duration), // Process after delay
}
```

### Event Handler Registration

```rust
pub trait EventHandler: Send + Sync {
    fn handle(&mut self, event: &Event, world: &mut World);
    fn event_type(&self) -> TypeId;
    fn priority(&self) -> i32; // Handler priority within event type
}

impl EventBus {
    pub fn register_handler<E: Event, H: EventHandler>(&mut self, handler: H) {
        let type_id = TypeId::of::<E>();
        self.handlers.entry(type_id)
            .or_default()
            .push(Box::new(handler));
            
        // Sort handlers by priority
        self.handlers.get_mut(&type_id).unwrap()
            .sort_by_key(|h| -h.priority());
    }
}
```

## Event Catalog

### Creature Events

```rust
#[derive(Debug, Clone, Event)]
pub enum CreatureEvent {
    // Lifecycle
    Spawned {
        entity: Entity,
        position: Vec3,
        genetics: Genetics,
        parent_ids: Option<(Entity, Entity)>,
    },
    
    Died {
        entity: Entity,
        position: Vec3,
        cause: DeathCause,
        age: f32,
        legacy: CreatureLegacy,
    },
    
    // State changes
    StateChanged {
        entity: Entity,
        from: CreatureState,
        to: CreatureState,
        reason: StateChangeReason,
    },
    
    // Health
    HealthChanged {
        entity: Entity,
        old_health: f32,
        new_health: f32,
        cause: HealthChangeCause,
    },
    
    NeedCritical {
        entity: Entity,
        need: NeedType,
        value: f32,
        time_until_death: f32,
    },
    
    // Reproduction
    PregnancyStarted {
        mother: Entity,
        father: Entity,
        conception_time: f32,
    },
    
    BirthOccurred {
        mother: Entity,
        offspring: Vec<Entity>,
        birth_location: Vec3,
    },
}

#[derive(Debug, Clone)]
pub enum DeathCause {
    Starvation,
    Dehydration,
    Disease(DiseaseType),
    Predation(Entity),
    OldAge,
    Environmental(EnvironmentalHazard),
    Combat(Entity),
}

impl EventPriority for CreatureEvent {
    fn priority(&self) -> EventPriority {
        match self {
            CreatureEvent::Died { .. } => EventPriority::Immediate,
            CreatureEvent::NeedCritical { .. } => EventPriority::High,
            CreatureEvent::HealthChanged { .. } => EventPriority::Normal,
            _ => EventPriority::Normal,
        }
    }
}
```

### Movement Events

```rust
#[derive(Debug, Clone, Event)]
pub enum MovementEvent {
    // Position changes
    Moved {
        entity: Entity,
        from: Vec3,
        to: Vec3,
        distance: f32,
    },
    
    TeleportedToArea {
        entity: Entity,
        from_area: AreaId,
        to_area: AreaId,
    },
    
    // Pathfinding
    PathRequested {
        entity: Entity,
        from: Vec3,
        to: Vec3,
        priority: PathPriority,
    },
    
    PathFound {
        entity: Entity,
        path: Path,
        cost: f32,
    },
    
    PathBlocked {
        entity: Entity,
        blocker: BlockerType,
        position: Vec3,
    },
    
    // Group movement
    GroupMovementStarted {
        leader: Entity,
        followers: Vec<Entity>,
        destination: Vec3,
    },
    
    StoppedFollowing {
        follower: Entity,
        leader: Entity,
        reason: UnfollowReason,
    },
}

impl EventPriority for MovementEvent {
    fn priority(&self) -> EventPriority {
        match self {
            MovementEvent::PathBlocked { .. } => EventPriority::High,
            MovementEvent::PathRequested { priority, .. } => {
                match priority {
                    PathPriority::Emergency => EventPriority::Immediate,
                    PathPriority::High => EventPriority::High,
                    _ => EventPriority::Normal,
                }
            }
            _ => EventPriority::Low,
        }
    }
}
```

### Social Events

```rust
#[derive(Debug, Clone, Event)]
pub enum SocialEvent {
    // Relationships
    RelationshipFormed {
        creature_a: Entity,
        creature_b: Entity,
        relationship_type: RelationshipType,
        initial_strength: f32,
    },
    
    RelationshipChanged {
        creature_a: Entity,
        creature_b: Entity,
        old_strength: f32,
        new_strength: f32,
        cause: RelationshipChangeCause,
    },
    
    RelationshipBroken {
        creature_a: Entity,
        creature_b: Entity,
        reason: BreakupReason,
    },
    
    // Groups
    GroupFormed {
        id: GroupId,
        leader: Entity,
        initial_members: Vec<Entity>,
        group_type: GroupType,
    },
    
    JoinedGroup {
        creature: Entity,
        group: GroupId,
        role: GroupRole,
    },
    
    LeftGroup {
        creature: Entity,
        group: GroupId,
        reason: LeaveReason,
    },
    
    GroupDisbanded {
        group: GroupId,
        reason: DisbandReason,
        former_members: Vec<Entity>,
    },
    
    LeadershipChanged {
        group: GroupId,
        old_leader: Entity,
        new_leader: Entity,
        succession_type: SuccessionType,
    },
    
    // Conflicts
    ConflictStarted {
        aggressor: Entity,
        target: Entity,
        conflict_type: ConflictType,
    },
    
    ConflictResolved {
        winner: Option<Entity>,
        loser: Option<Entity>,
        resolution: ConflictResolution,
    },
}

impl EventPriority for SocialEvent {
    fn priority(&self) -> EventPriority {
        match self {
            SocialEvent::ConflictStarted { .. } => EventPriority::High,
            SocialEvent::GroupDisbanded { .. } => EventPriority::High,
            _ => EventPriority::Normal,
        }
    }
}
```

### Conversation Events

```rust
#[derive(Debug, Clone, Event)]
pub enum ConversationEvent {
    // Conversation lifecycle
    Started {
        id: ConversationId,
        participants: Vec<Entity>,
        topic: Topic,
        location: Vec3,
    },
    
    Ended {
        id: ConversationId,
        duration: f32,
        outcome: ConversationOutcome,
        relationship_changes: Vec<(Entity, Entity, f32)>,
    },
    
    // Participation
    JoinedConversation {
        creature: Entity,
        conversation: ConversationId,
        role: ParticipantRole,
    },
    
    LeftConversation {
        creature: Entity,
        conversation: ConversationId,
        reason: LeaveReason,
    },
    
    // Content
    StatementMade {
        speaker: Entity,
        conversation: ConversationId,
        statement_type: StatementType,
        emotion: Emotion,
    },
    
    KnowledgeShared {
        sharer: Entity,
        recipients: Vec<Entity>,
        knowledge: Knowledge,
    },
    
    TradeProposed {
        proposer: Entity,
        recipient: Entity,
        offered: Vec<ResourceStack>,
        requested: Vec<ResourceStack>,
    },
    
    TradeCompleted {
        trader_a: Entity,
        trader_b: Entity,
        exchanged_items: Vec<(Entity, Vec<ResourceStack>)>,
    },
}
```

### Resource Events

```rust
#[derive(Debug, Clone, Event)]
pub enum ResourceEvent {
    // Resource lifecycle
    Spawned {
        resource_type: ResourceType,
        position: Vec3,
        amount: u32,
    },
    
    Depleted {
        resource_type: ResourceType,
        position: Vec3,
        last_gatherer: Option<Entity>,
    },
    
    Regenerated {
        resource_type: ResourceType,
        position: Vec3,
        amount: u32,
    },
    
    // Gathering
    Gathered {
        gatherer: Entity,
        resource_type: ResourceType,
        amount: u32,
        position: Vec3,
    },
    
    // Storage
    Stored {
        creature: Entity,
        resource_type: ResourceType,
        amount: u32,
        storage_location: StorageType,
    },
    
    Consumed {
        consumer: Entity,
        resource_type: ResourceType,
        amount: u32,
        purpose: ConsumptionPurpose,
    },
    
    // Sharing
    Shared {
        giver: Entity,
        receiver: Entity,
        resource_type: ResourceType,
        amount: u32,
        reason: SharingReason,
    },
}
```

### World Events

```rust
#[derive(Debug, Clone, Event)]
pub enum WorldEvent {
    // Time
    DayNightChanged {
        is_day: bool,
        sun_angle: f32,
        light_level: f32,
    },
    
    SeasonChanged {
        old_season: Season,
        new_season: Season,
        transition_progress: f32,
    },
    
    // Weather
    WeatherChanged {
        old_weather: Weather,
        new_weather: Weather,
        severity: f32,
    },
    
    DisasterStarted {
        disaster_type: DisasterType,
        affected_area: AABB,
        severity: f32,
        duration: f32,
    },
    
    DisasterEnded {
        disaster_type: DisasterType,
        casualties: u32,
        damage: DisasterDamage,
    },
    
    // Environment
    TerrainModified {
        position: Vec3,
        old_terrain: TerrainType,
        new_terrain: TerrainType,
        cause: TerrainChangeCause,
    },
    
    BiomeTransition {
        position: Vec3,
        from_biome: BiomeType,
        to_biome: BiomeType,
        blend_factor: f32,
    },
}

impl EventPriority for WorldEvent {
    fn priority(&self) -> EventPriority {
        match self {
            WorldEvent::DisasterStarted { .. } => EventPriority::Immediate,
            WorldEvent::WeatherChanged { .. } => EventPriority::High,
            _ => EventPriority::Normal,
        }
    }
}
```

### UI Events

```rust
#[derive(Debug, Clone, Event)]
pub enum UIEvent {
    // Selection
    CreatureSelected {
        entity: Entity,
        selection_mode: SelectionMode,
        modifier_keys: ModifierKeys,
    },
    
    CreatureDeselected {
        entity: Entity,
    },
    
    AreaSelected {
        bounds: AABB,
        entities_in_area: Vec<Entity>,
    },
    
    // Camera
    CameraFocusRequested {
        target: CameraTarget,
        transition_time: f32,
    },
    
    FollowModeChanged {
        entity: Option<Entity>,
        follow_mode: FollowMode,
    },
    
    // User notifications
    NotificationRequested {
        message: String,
        severity: NotificationSeverity,
        duration: f32,
        icon: Option<IconType>,
    },
    
    // Data visualization
    ViewModeChanged {
        old_mode: ViewMode,
        new_mode: ViewMode,
    },
    
    DataOverlayToggled {
        overlay_type: DataOverlay,
        enabled: bool,
    },
}
```

## Event Processing

### Event Dispatch

```rust
impl EventBus {
    pub fn dispatch<E: Event>(&mut self, event: E) {
        let priority = event.priority();
        let boxed_event = Box::new(event) as Box<dyn Any + Send + Sync>;
        
        match priority {
            EventPriority::Immediate => {
                self.immediate_queue.push_back(Event(boxed_event));
            }
            EventPriority::High => {
                self.high_priority_queue.push(PrioritizedEvent {
                    event: Event(boxed_event),
                    priority: 1.0,
                    timestamp: Instant::now(),
                });
            }
            EventPriority::Normal => {
                self.normal_queue.push_back(Event(boxed_event));
            }
            EventPriority::Low => {
                self.low_priority_queue.push_back(Event(boxed_event));
            }
            EventPriority::Deferred(delay) => {
                self.deferred_queue.push(DeferredEvent {
                    event: Event(boxed_event),
                    execute_at: Instant::now() + delay,
                });
            }
        }
    }
    
    pub fn process_events(&mut self, world: &mut World) {
        // Process immediate events first
        while let Some(event) = self.immediate_queue.pop_front() {
            self.process_single_event(event, world);
        }
        
        // Process high priority events
        while let Some(prioritized) = self.high_priority_queue.pop() {
            self.process_single_event(prioritized.event, world);
        }
        
        // Process normal events
        let normal_count = self.normal_queue.len().min(100); // Batch limit
        for _ in 0..normal_count {
            if let Some(event) = self.normal_queue.pop_front() {
                self.process_single_event(event, world);
            }
        }
        
        // Process low priority events if time permits
        let low_count = self.low_priority_queue.len().min(50);
        for _ in 0..low_count {
            if let Some(event) = self.low_priority_queue.pop_front() {
                self.process_single_event(event, world);
            }
        }
        
        // Check deferred events
        let now = Instant::now();
        self.deferred_queue.retain(|deferred| {
            if deferred.execute_at <= now {
                self.normal_queue.push_back(deferred.event.clone());
                false
            } else {
                true
            }
        });
    }
}
```

### Event Batching

```rust
pub struct EventBatcher {
    pub batch_size: usize,
    pub batch_timeout: Duration,
    pub pending_batches: HashMap<TypeId, EventBatch>,
}

pub struct EventBatch {
    pub events: Vec<Event>,
    pub created_at: Instant,
}

impl EventBatcher {
    pub fn add_event<E: Event>(&mut self, event: E) {
        let type_id = TypeId::of::<E>();
        let batch = self.pending_batches.entry(type_id)
            .or_insert_with(|| EventBatch {
                events: Vec::new(),
                created_at: Instant::now(),
            });
            
        batch.events.push(Box::new(event).into());
        
        if batch.events.len() >= self.batch_size {
            self.flush_batch(type_id);
        }
    }
    
    pub fn update(&mut self) {
        let now = Instant::now();
        let to_flush: Vec<_> = self.pending_batches
            .iter()
            .filter(|(_, batch)| now - batch.created_at > self.batch_timeout)
            .map(|(type_id, _)| *type_id)
            .collect();
            
        for type_id in to_flush {
            self.flush_batch(type_id);
        }
    }
}
```

## Event Filtering

```rust
pub struct EventFilter {
    pub entity_filter: Option<EntityFilter>,
    pub spatial_filter: Option<SpatialFilter>,
    pub type_filter: HashSet<TypeId>,
    pub priority_threshold: Option<EventPriority>,
}

pub enum EntityFilter {
    Single(Entity),
    Multiple(HashSet<Entity>),
    WithComponent(TypeId),
    Custom(Box<dyn Fn(&Entity) -> bool>),
}

pub enum SpatialFilter {
    InRadius { center: Vec3, radius: f32 },
    InBounds(AABB),
    InBiome(BiomeType),
}

impl EventBus {
    pub fn subscribe_filtered<E: Event>(&mut self, 
                                      filter: EventFilter, 
                                      handler: impl Fn(&E) + 'static) {
        // Store filtered subscription
        self.filtered_handlers.push(FilteredHandler {
            event_type: TypeId::of::<E>(),
            filter,
            handler: Box::new(move |event| {
                if let Some(e) = event.downcast_ref::<E>() {
                    handler(e);
                }
            }),
        });
    }
}
```

## Performance Monitoring

```rust
pub struct EventMetrics {
    pub events_per_second: f32,
    pub average_processing_time: Duration,
    pub queue_depths: HashMap<EventPriority, usize>,
    pub hot_events: Vec<(TypeId, u32)>,
    pub slow_handlers: Vec<(String, Duration)>,
}

impl EventBus {
    pub fn get_metrics(&self) -> EventMetrics {
        let mut hot_events: Vec<_> = self.event_counts
            .iter()
            .map(|(type_id, count)| (*type_id, *count))
            .collect();
        hot_events.sort_by_key(|(_, count)| Reverse(*count));
        hot_events.truncate(10);
        
        EventMetrics {
            events_per_second: self.calculate_event_rate(),
            average_processing_time: self.calculate_avg_processing_time(),
            queue_depths: self.get_queue_depths(),
            hot_events,
            slow_handlers: self.get_slow_handlers(),
        }
    }
}
```

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_priority_ordering() {
        let mut bus = EventBus::new();
        
        // Dispatch events in reverse priority order
        bus.dispatch(TestEvent::Low);
        bus.dispatch(TestEvent::Normal);
        bus.dispatch(TestEvent::High);
        bus.dispatch(TestEvent::Immediate);
        
        // Process and verify order
        let processed = bus.process_all_events_test();
        assert_eq!(processed[0].priority(), EventPriority::Immediate);
        assert_eq!(processed[1].priority(), EventPriority::High);
        assert_eq!(processed[2].priority(), EventPriority::Normal);
        assert_eq!(processed[3].priority(), EventPriority::Low);
    }
    
    #[test]
    fn test_event_batching() {
        let mut batcher = EventBatcher::new(5, Duration::from_millis(100));
        
        // Add events
        for i in 0..4 {
            batcher.add_event(CreatureEvent::Moved { 
                entity: Entity::from_raw(i),
                from: Vec3::ZERO,
                to: Vec3::ONE,
                distance: 1.0,
            });
        }
        
        // Should not flush yet
        assert_eq!(batcher.pending_batches.len(), 1);
        
        // Add one more to trigger flush
        batcher.add_event(CreatureEvent::Moved { 
            entity: Entity::from_raw(5),
            from: Vec3::ZERO,
            to: Vec3::ONE,
            distance: 1.0,
        });
        
        // Should have flushed
        assert_eq!(batcher.pending_batches.len(), 0);
    }
}
```

This event system provides a robust, priority-based communication mechanism between all game systems while maintaining performance through batching and filtering.