# System Interaction Matrix

This document defines how all systems in the creature simulation interact with each other, including data flow, event triggers, and dependencies.

## System Overview

### Core Systems
1. **Creature System** - Entity management, component storage
2. **Decision System** - AI behavior and action selection
3. **Movement System** - Pathfinding and position updates
4. **Social System** - Relationships and group dynamics
5. **Conversation System** - Communication between creatures
6. **Resource System** - Food, water, material management
7. **Health System** - Needs, disease, death
8. **Genetics System** - Traits, reproduction, evolution
9. **World System** - Terrain, weather, day/night
10. **Rendering System** - Visual representation
11. **UI System** - User interface and controls
12. **Event System** - Message passing between systems

## Interaction Matrix

| From ↓ To → | Creature | Decision | Movement | Social | Conversation | Resource | Health | Genetics | World | Rendering | UI | Event |
|-------------|----------|----------|----------|---------|--------------|----------|---------|----------|--------|-----------|-----|--------|
| **Creature** | - | Provides state | Updates position | Tracks relationships | - | Owns inventory | Tracks health | Stores genome | Exists in | Provides visuals | Selected by | Emits events |
| **Decision** | Queries state | - | Requests path | Considers social | Initiates talk | Seeks resources | Monitors needs | Uses traits | Scans environment | - | - | Triggers actions |
| **Movement** | Updates position | - | - | Affects proximity | - | - | Consumes energy | Speed from traits | Navigates terrain | Updates position | - | Movement events |
| **Social** | Queries relations | Influences decisions | Groups move together | - | Affects tone | Shares resources | Spreads disease | Inherited behaviors | Territory claims | Group indicators | Shows relationships | Social events |
| **Conversation** | Between creatures | - | - | Builds relationships | - | Trade/share | Shares knowledge | - | - | Speech bubbles | - | Chat events |
| **Resource** | Carried by | Influences decisions | - | Shared in groups | Traded | - | Provides needs | - | Exists in world | Shows availability | Resource overlay | Depletion events |
| **Health** | Component of | Affects decisions | Limits movement | - | - | Consumes resources | - | Genetic resistance | Affected by weather | Health bars | Health warnings | Health events |
| **Genetics** | Component of | Trait bonuses | Movement speed | Social tendencies | Language evolution | Metabolism | Disease resistance | - | Adaptation | Appearance | Genetics view | Mutation events |
| **World** | Contains | Provides context | Terrain costs | Meeting places | - | Spawns resources | Weather effects | Environmental pressure | - | Terrain rendering | World view | Weather events |
| **Rendering** | Draws | - | Shows paths | Shows groups | Shows bubbles | Shows items | Shows states | Visual traits | Draws world | - | - | - |
| **UI** | Selects | Shows thinking | Shows destination | Shows network | Shows chats | Shows inventory | Shows needs | Shows genome | Camera control | - | - | User events |
| **Event** | Notifies | Notifies | Notifies | Notifies | Notifies | Notifies | Notifies | Notifies | Notifies | - | Notifies | - |

## Detailed System Interactions

### Creature → Other Systems

The Creature System is the central entity manager:

```rust
// Creature provides components to other systems
pub struct CreatureComponents {
    pub transform: Transform,
    pub health: Health,
    pub needs: Needs,
    pub genetics: Genetics,
    pub memory: Memory,
    pub social: SocialData,
    pub inventory: Inventory,
}
```

**Data Flow:**
- Provides entity IDs and component access to all systems
- Maintains spatial index for efficient queries
- Handles entity lifecycle (spawn/death)

### Decision → Movement

Decision System requests movement from Movement System:

```rust
pub struct MovementRequest {
    pub entity: Entity,
    pub destination: Vec3,
    pub priority: MovementPriority,
    pub reason: MovementReason,
}

pub enum MovementReason {
    SeekingFood,
    Fleeing,
    Socializing,
    Exploring,
    Following,
}
```

### Social → Conversation

Social relationships influence conversation dynamics:

```rust
pub struct ConversationContext {
    pub relationship: RelationshipType,
    pub trust_level: f32,
    pub shared_experiences: Vec<MemoryId>,
    pub group_membership: Option<GroupId>,
}
```

### Health → Resource

Health system consumes resources:

```rust
pub struct ResourceConsumption {
    pub creature: Entity,
    pub resource: ResourceType,
    pub amount: f32,
    pub need: NeedType,
}

// Resource system validates and applies consumption
pub fn consume_resource(consumption: ResourceConsumption) -> Result<()> {
    // Check availability
    // Reduce creature inventory
    // Update need satisfaction
    // Emit consumption event
}
```

### Movement → World

Movement system queries world for navigation:

```rust
pub struct TerrainQuery {
    pub from: Vec3,
    pub to: Vec3,
    pub creature_type: CreatureType,
}

pub struct TerrainResponse {
    pub traversable: bool,
    pub movement_cost: f32,
    pub hazards: Vec<EnvironmentalHazard>,
}
```

### Genetics → Multiple Systems

Genetics influences many systems:

```rust
pub struct GeneticTraits {
    // Decision System
    pub aggression: f32,
    pub curiosity: f32,
    pub sociability: f32,
    
    // Movement System
    pub speed_multiplier: f32,
    pub stamina: f32,
    
    // Health System
    pub metabolism_rate: f32,
    pub disease_resistance: f32,
    pub lifespan_modifier: f32,
    
    // Social System
    pub group_cohesion: f32,
    pub leadership: f32,
    
    // Conversation System
    pub language_learning: f32,
    pub memory_capacity: f32,
}
```

### Event System Flow

Central event bus for loose coupling:

```rust
pub enum GameEvent {
    // Creature Events
    CreatureSpawned { entity: Entity, position: Vec3 },
    CreatureDied { entity: Entity, cause: DeathCause },
    
    // Movement Events
    CreatureMoved { entity: Entity, from: Vec3, to: Vec3 },
    PathBlocked { entity: Entity, reason: BlockedReason },
    
    // Social Events
    RelationshipFormed { a: Entity, b: Entity, type: RelationshipType },
    GroupFormed { leader: Entity, members: Vec<Entity> },
    
    // Resource Events
    ResourceDepleted { position: Vec3, type: ResourceType },
    ResourceDiscovered { position: Vec3, type: ResourceType },
    
    // Health Events
    CreatureHungry { entity: Entity, severity: f32 },
    DiseaseSpread { from: Entity, to: Entity, disease: DiseaseType },
    
    // World Events
    WeatherChanged { old: Weather, new: Weather },
    SeasonChanged { season: Season },
}
```

## System Dependencies

### Initialization Order

Systems must initialize in dependency order:

1. **World System** - Terrain and environment
2. **Resource System** - Place resources in world
3. **Creature System** - Entity management
4. **Genetics System** - Initialize genomes
5. **Health System** - Set initial health
6. **Social System** - Initialize relationships
7. **Movement System** - Prepare pathfinding
8. **Decision System** - AI initialization
9. **Conversation System** - Communication setup
10. **Event System** - Connect all systems
11. **Rendering System** - Visual setup
12. **UI System** - Interface initialization

### Update Order

Systems update in specific order each frame:

```rust
pub struct SystemSchedule {
    // Input Phase
    ui_input: SystemStage,           // Process user input
    
    // Simulation Phase
    world_update: SystemStage,       // Weather, time, seasons
    decision_making: SystemStage,    // AI decisions
    movement_update: SystemStage,    // Apply movement
    resource_update: SystemStage,    // Resource consumption/regeneration
    health_update: SystemStage,      // Update needs, check death
    social_update: SystemStage,      // Update relationships
    conversation_update: SystemStage, // Process conversations
    
    // Post-process Phase
    event_processing: SystemStage,   // Handle events
    spatial_update: SystemStage,     // Update spatial indices
    
    // Render Phase
    render_prepare: SystemStage,     // Prepare render data
    ui_update: SystemStage,         // Update UI state
}
```

## Cross-System Queries

Common queries that span multiple systems:

### "Find nearby food"
1. **Decision System** triggers query
2. **Spatial System** finds nearby entities
3. **Resource System** filters for food
4. **Movement System** checks accessibility
5. Returns sorted list by distance/preference

### "Spread disease"
1. **Health System** detects infection
2. **Spatial System** finds nearby creatures
3. **Social System** weights by relationship
4. **Genetics System** checks resistance
5. **Health System** applies infection

### "Form group"
1. **Social System** initiates grouping
2. **Spatial System** finds candidates
3. **Genetics System** checks compatibility
4. **Decision System** gets agreement
5. **Social System** creates group

## Performance Considerations

### System Communication

- **Events**: Asynchronous, buffered, batched per frame
- **Direct Queries**: Synchronous, cached where possible
- **Shared State**: Minimal, through ECS components only

### Data Locality

Systems are organized to minimize cache misses:

```rust
// Good: Systems access their own data
struct MovementSystem {
    paths: HashMap<Entity, Path>,
    requests: Vec<MovementRequest>,
}

// Bad: Systems reaching into others
struct BadSystem {
    other_system_data: Arc<Mutex<OtherData>>,
}
```

### Batching

Systems batch operations for efficiency:

```rust
pub struct BatchedUpdates {
    position_updates: Vec<(Entity, Vec3)>,
    health_updates: Vec<(Entity, HealthDelta)>,
    relationship_updates: Vec<(Entity, Entity, RelationshipDelta)>,
}
```

## Testing System Interactions

### Integration Tests

```rust
#[test]
fn test_creature_death_cascade() {
    // Creature dies
    // -> Health system emits death event
    // -> Social system removes from groups
    // -> Conversation system ends conversations  
    // -> Resource system drops inventory
    // -> Movement system clears paths
    // -> Rendering system removes visuals
}

#[test]
fn test_resource_depletion_flow() {
    // Resource depleted
    // -> Resource system emits event
    // -> Decision system updates goals
    // -> Movement system replans paths
    // -> Social system may share info
}
```

This matrix ensures all systems interact correctly and efficiently, with clear data flow and minimal coupling.