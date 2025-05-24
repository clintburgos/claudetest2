# Notification and Filtering System

## Overview

The notification system provides intelligent, context-aware notifications to keep observers informed of important events without overwhelming them. It includes filtering, prioritization, aggregation, and customization features.

## Core Architecture

### Notification Manager

```rust
pub struct NotificationManager {
    // Active notifications
    active_notifications: VecDeque<Notification>,
    notification_queue: BinaryHeap<PrioritizedNotification>,
    
    // Filtering and rules
    filter_rules: Vec<NotificationFilter>,
    aggregation_rules: Vec<AggregationRule>,
    suppression_rules: Vec<SuppressionRule>,
    
    // User preferences
    user_preferences: NotificationPreferences,
    importance_thresholds: HashMap<NotificationCategory, f32>,
    
    // History and analytics
    notification_history: RingBuffer<NotificationRecord>,
    statistics: NotificationStatistics,
    
    // Display settings
    max_visible: usize,
    display_duration: HashMap<NotificationPriority, Duration>,
}

pub struct Notification {
    id: NotificationId,
    category: NotificationCategory,
    priority: NotificationPriority,
    
    // Content
    title: String,
    message: String,
    icon: Option<IconType>,
    color: Color,
    
    // Metadata
    timestamp: Instant,
    source: NotificationSource,
    related_entities: Vec<Entity>,
    location: Option<Vec3>,
    
    // Interaction
    actions: Vec<NotificationAction>,
    dismissible: bool,
    click_action: Option<ClickAction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum NotificationPriority {
    Critical,   // Always show, never auto-dismiss
    High,       // Show immediately
    Normal,     // Standard notifications  
    Low,        // Show when space available
    Info,       // Passive information
}
```

### Notification Categories

```rust
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum NotificationCategory {
    // Creature events
    CreatureLifecycle,
    CreatureHealth,
    CreatureAchievement,
    
    // Population events
    PopulationMilestone,
    PopulationCrisis,
    SpeciesEvent,
    
    // Environmental
    WeatherChange,
    ResourceDepletion,
    EnvironmentalHazard,
    
    // Social dynamics
    GroupFormation,
    ConflictAlert,
    CulturalDevelopment,
    
    // Evolution
    MutationDiscovered,
    EvolutionaryMilestone,
    ExtinctionWarning,
    
    // System
    PerformanceWarning,
    SaveGameReminder,
    TutorialHint,
}

impl NotificationCategory {
    pub fn default_priority(&self) -> NotificationPriority {
        match self {
            NotificationCategory::PopulationCrisis => NotificationPriority::Critical,
            NotificationCategory::ExtinctionWarning => NotificationPriority::Critical,
            NotificationCategory::EnvironmentalHazard => NotificationPriority::High,
            NotificationCategory::CreatureAchievement => NotificationPriority::Normal,
            NotificationCategory::WeatherChange => NotificationPriority::Low,
            _ => NotificationPriority::Normal,
        }
    }
    
    pub fn default_icon(&self) -> IconType {
        match self {
            NotificationCategory::CreatureLifecycle => IconType::Creature,
            NotificationCategory::CreatureHealth => IconType::Health,
            NotificationCategory::PopulationMilestone => IconType::Population,
            NotificationCategory::WeatherChange => IconType::Weather,
            NotificationCategory::ResourceDepletion => IconType::Resource,
            NotificationCategory::GroupFormation => IconType::Group,
            NotificationCategory::MutationDiscovered => IconType::DNA,
            _ => IconType::Info,
        }
    }
}
```

## Filtering System

### Filter Rules

```rust
pub struct NotificationFilter {
    id: FilterId,
    name: String,
    enabled: bool,
    conditions: Vec<FilterCondition>,
    action: FilterAction,
}

pub enum FilterCondition {
    // Category-based
    Category(NotificationCategory),
    CategoryExclude(NotificationCategory),
    
    // Priority-based
    PriorityAbove(NotificationPriority),
    PriorityBelow(NotificationPriority),
    
    // Entity-based
    RelatedToSelected,
    RelatedToFollowed,
    InViewport,
    WithinDistance { center: Vec3, radius: f32 },
    
    // Content-based
    ContainsKeyword(String),
    MatchesPattern(Regex),
    
    // Frequency-based
    RateLimitExceeded { max_per_minute: u32 },
    
    // Time-based
    DuringTimeScale { min: f32, max: f32 },
    GameTimeBetween { start: f32, end: f32 },
    
    // Custom
    Custom(Box<dyn Fn(&Notification) -> bool>),
}

pub enum FilterAction {
    Allow,
    Block,
    Downgrade(NotificationPriority),
    Redirect(NotificationChannel),
    Tag(String),
}

impl NotificationFilter {
    pub fn apply(&self, notification: &mut Notification) -> FilterResult {
        if !self.enabled {
            return FilterResult::Pass;
        }
        
        let all_match = self.conditions.iter().all(|condition| {
            condition.matches(notification)
        });
        
        if all_match {
            match &self.action {
                FilterAction::Allow => FilterResult::Pass,
                FilterAction::Block => FilterResult::Block,
                FilterAction::Downgrade(priority) => {
                    notification.priority = *priority;
                    FilterResult::Modified
                }
                FilterAction::Redirect(channel) => {
                    FilterResult::Redirect(*channel)
                }
                FilterAction::Tag(tag) => {
                    notification.tags.push(tag.clone());
                    FilterResult::Modified
                }
            }
        } else {
            FilterResult::Pass
        }
    }
}
```

### Aggregation Rules

```rust
pub struct AggregationRule {
    id: RuleId,
    name: String,
    trigger: AggregationTrigger,
    aggregator: Box<dyn NotificationAggregator>,
}

pub enum AggregationTrigger {
    Count { threshold: u32, time_window: Duration },
    Rate { per_second: f32 },
    Pattern { pattern: AggregationPattern },
}

pub trait NotificationAggregator: Send + Sync {
    fn can_aggregate(&self, a: &Notification, b: &Notification) -> bool;
    fn aggregate(&self, notifications: Vec<Notification>) -> Notification;
}

pub struct CategoryAggregator {
    category: NotificationCategory,
    time_window: Duration,
}

impl NotificationAggregator for CategoryAggregator {
    fn can_aggregate(&self, a: &Notification, b: &Notification) -> bool {
        a.category == self.category && 
        b.category == self.category &&
        b.timestamp.duration_since(a.timestamp) < self.time_window
    }
    
    fn aggregate(&self, notifications: Vec<Notification>) -> Notification {
        let count = notifications.len();
        let first = &notifications[0];
        
        Notification {
            id: NotificationId::new(),
            category: self.category,
            priority: notifications.iter()
                .map(|n| n.priority)
                .max()
                .unwrap_or(NotificationPriority::Normal),
            title: format!("{} {} events", count, self.category.name()),
            message: self.build_summary(&notifications),
            icon: Some(self.category.default_icon()),
            color: first.color,
            timestamp: Instant::now(),
            source: NotificationSource::Aggregated,
            related_entities: notifications.iter()
                .flat_map(|n| &n.related_entities)
                .cloned()
                .collect(),
            location: None,
            actions: vec![
                NotificationAction::ExpandGroup(notifications),
            ],
            dismissible: true,
            click_action: None,
        }
    }
}
```

### Suppression Rules

```rust
pub struct SuppressionRule {
    id: RuleId,
    condition: SuppressionCondition,
    duration: SuppressionDuration,
}

pub enum SuppressionCondition {
    AfterDismissal { category: NotificationCategory },
    DuringHighActivity { threshold: u32 },
    WhilePaused,
    DuringFastForward { min_speed: f32 },
    AfterRepeated { count: u32, time_window: Duration },
}

pub enum SuppressionDuration {
    Forever,
    Duration(Duration),
    UntilCondition(Box<dyn Fn() -> bool>),
    Count(u32),
}
```

## User Preferences

```rust
pub struct NotificationPreferences {
    // Global settings
    enabled: bool,
    max_simultaneous: usize,
    position: ScreenPosition,
    
    // Category preferences
    category_settings: HashMap<NotificationCategory, CategoryPreference>,
    
    // Display preferences
    animation_style: AnimationStyle,
    sound_enabled: bool,
    persistence: PersistenceSettings,
    
    // Advanced settings
    aggregation_enabled: bool,
    smart_filtering: bool,
    importance_learning: bool,
}

pub struct CategoryPreference {
    enabled: bool,
    priority_override: Option<NotificationPriority>,
    sound_override: Option<SoundId>,
    color_override: Option<Color>,
    auto_focus: bool,
    log_to_history: bool,
}

pub struct PersistenceSettings {
    save_history: bool,
    history_size: usize,
    persist_filters: bool,
    persist_dismissed: bool,
}
```

## Smart Filtering

### Importance Learning

```rust
pub struct ImportanceLearner {
    interaction_history: HashMap<NotificationCategory, InteractionStats>,
    feature_weights: HashMap<FeatureType, f32>,
    model: SimpleNeuralNetwork,
}

pub struct InteractionStats {
    shown_count: u32,
    clicked_count: u32,
    dismissed_count: u32,
    average_view_time: Duration,
    action_taken_count: u32,
}

impl ImportanceLearner {
    pub fn calculate_importance(&self, notification: &Notification) -> f32 {
        let features = self.extract_features(notification);
        let base_importance = self.model.predict(&features);
        
        // Adjust based on interaction history
        if let Some(stats) = self.interaction_history.get(&notification.category) {
            let interaction_rate = stats.clicked_count as f32 / stats.shown_count.max(1) as f32;
            base_importance * (0.5 + interaction_rate)
        } else {
            base_importance
        }
    }
    
    fn extract_features(&self, notification: &Notification) -> FeatureVector {
        FeatureVector {
            category_importance: self.category_importance(notification.category),
            entity_relevance: self.calculate_entity_relevance(&notification.related_entities),
            temporal_relevance: self.calculate_temporal_relevance(notification.timestamp),
            spatial_relevance: self.calculate_spatial_relevance(notification.location),
            content_score: self.analyze_content(&notification.message),
        }
    }
}
```

### Context-Aware Filtering

```rust
pub struct ContextFilter {
    current_focus: Option<Entity>,
    viewport: AABB,
    time_scale: f32,
    active_tools: HashSet<ToolType>,
    recent_interactions: RingBuffer<Interaction>,
}

impl ContextFilter {
    pub fn adjust_notification(&self, notification: &mut Notification) {
        // Boost notifications related to current focus
        if let Some(focus) = self.current_focus {
            if notification.related_entities.contains(&focus) {
                notification.priority = notification.priority.upgrade();
            }
        }
        
        // Reduce priority during fast-forward
        if self.time_scale > 10.0 {
            match notification.category {
                NotificationCategory::WeatherChange |
                NotificationCategory::CreatureLifecycle => {
                    notification.priority = notification.priority.downgrade();
                }
                _ => {}
            }
        }
        
        // Boost notifications in viewport
        if let Some(location) = notification.location {
            if self.viewport.contains(location) {
                notification.priority = notification.priority.upgrade();
            }
        }
    }
}
```

## Display System

### Notification Rendering

```rust
pub struct NotificationRenderer {
    layout: NotificationLayout,
    animations: HashMap<NotificationId, Animation>,
    render_queue: Vec<NotificationRenderData>,
}

pub enum NotificationLayout {
    Stack { spacing: f32, direction: StackDirection },
    Grid { columns: u32, spacing: Vec2 },
    Timeline { height: f32 },
    Floating { positions: HashMap<NotificationId, Vec2> },
}

pub struct NotificationRenderData {
    notification: Notification,
    position: Vec2,
    size: Vec2,
    opacity: f32,
    scale: f32,
    lifetime_progress: f32,
}

impl NotificationRenderer {
    pub fn render(&mut self, ui: &mut Ui, notifications: &[Notification], dt: f32) {
        // Update animations
        for (id, animation) in &mut self.animations {
            animation.update(dt);
        }
        
        // Calculate layout
        self.render_queue.clear();
        match &self.layout {
            NotificationLayout::Stack { spacing, direction } => {
                self.layout_stack(notifications, *spacing, *direction);
            }
            // ... other layouts
        }
        
        // Render each notification
        for render_data in &self.render_queue {
            self.render_notification(ui, render_data);
        }
    }
    
    fn render_notification(&self, ui: &mut Ui, data: &NotificationRenderData) {
        let mut rect = Rect::from_xy_size(data.position, data.size);
        
        // Apply animation transforms
        rect = rect.scale_from_center(data.scale);
        
        // Background
        let bg_color = data.notification.color.with_alpha(0.9 * data.opacity);
        ui.painter().rect_filled(rect, 4.0, bg_color);
        
        // Icon
        if let Some(icon) = &data.notification.icon {
            let icon_rect = Rect::from_xy_size(
                rect.min + vec2(8.0, 8.0),
                vec2(24.0, 24.0)
            );
            ui.painter().image(icon.texture_id(), icon_rect, Color::WHITE);
        }
        
        // Text
        let text_pos = rect.min + vec2(40.0, 8.0);
        ui.painter().text(
            text_pos,
            Align::LEFT,
            &data.notification.title,
            FontId::proportional(14.0),
            Color::WHITE.with_alpha(data.opacity)
        );
        
        // Progress bar for lifetime
        let progress_rect = Rect::from_min_size(
            rect.min + vec2(0.0, rect.height() - 2.0),
            vec2(rect.width() * data.lifetime_progress, 2.0)
        );
        ui.painter().rect_filled(progress_rect, 0.0, Color::WHITE.with_alpha(0.5));
        
        // Actions
        if !data.notification.actions.is_empty() {
            self.render_actions(ui, &data.notification.actions, rect);
        }
    }
}
```

### Animation System

```rust
pub enum NotificationAnimation {
    SlideIn { from: SlideDirection, duration: f32 },
    FadeIn { duration: f32 },
    PopIn { overshoot: f32, duration: f32 },
    Attention { shake_amount: f32, duration: f32 },
}

pub struct AnimationState {
    animation_type: NotificationAnimation,
    progress: f32,
    phase: AnimationPhase,
}

pub enum AnimationPhase {
    Entering,
    Idle,
    Exiting,
}

impl AnimationState {
    pub fn get_transform(&self) -> Transform2D {
        match (&self.animation_type, self.phase) {
            (NotificationAnimation::SlideIn { from, .. }, AnimationPhase::Entering) => {
                let offset = match from {
                    SlideDirection::Right => vec2(300.0 * (1.0 - self.progress), 0.0),
                    SlideDirection::Top => vec2(0.0, -100.0 * (1.0 - self.progress)),
                    // ... other directions
                };
                Transform2D::from_translation(offset)
            }
            (NotificationAnimation::PopIn { overshoot, .. }, AnimationPhase::Entering) => {
                let scale = 1.0 + overshoot * (1.0 - self.progress).sin();
                Transform2D::from_scale(vec2(scale, scale))
            }
            // ... other animations
        }
    }
}
```

## Notification Actions

```rust
pub enum NotificationAction {
    Dismiss,
    Focus(Entity),
    JumpToLocation(Vec3),
    OpenPanel(PanelType),
    ExecuteCommand(Command),
    ExpandGroup(Vec<Notification>),
    Custom(Box<dyn Fn() + Send + Sync>),
}

pub struct ActionButton {
    label: String,
    icon: Option<IconType>,
    action: NotificationAction,
    style: ButtonStyle,
}

impl NotificationManager {
    pub fn handle_action(&mut self, notification_id: NotificationId, action: NotificationAction) {
        match action {
            NotificationAction::Dismiss => {
                self.dismiss_notification(notification_id);
            }
            NotificationAction::Focus(entity) => {
                self.events.send(UIEvent::FocusEntity(entity));
            }
            NotificationAction::JumpToLocation(pos) => {
                self.events.send(UIEvent::CameraJumpTo(pos));
            }
            NotificationAction::ExpandGroup(notifications) => {
                self.expand_aggregated_group(notifications);
            }
            // ... other actions
        }
        
        // Track interaction for learning
        self.record_interaction(notification_id, InteractionType::ActionTaken);
    }
}
```

## History and Search

```rust
pub struct NotificationHistory {
    entries: RingBuffer<NotificationRecord>,
    index: SearchIndex,
    filters: HistoryFilters,
}

pub struct NotificationRecord {
    notification: Notification,
    received_at: Instant,
    dismissed_at: Option<Instant>,
    interactions: Vec<InteractionRecord>,
    outcome: Option<NotificationOutcome>,
}

pub struct SearchIndex {
    text_index: HashMap<String, Vec<NotificationId>>,
    category_index: HashMap<NotificationCategory, Vec<NotificationId>>,
    entity_index: HashMap<Entity, Vec<NotificationId>>,
    time_index: BTreeMap<Instant, NotificationId>,
}

impl NotificationHistory {
    pub fn search(&self, query: &SearchQuery) -> Vec<NotificationRecord> {
        let mut results = HashSet::new();
        
        // Text search
        if let Some(text) = &query.text {
            for word in text.split_whitespace() {
                if let Some(ids) = self.index.text_index.get(word) {
                    results.extend(ids);
                }
            }
        }
        
        // Category filter
        if let Some(categories) = &query.categories {
            for category in categories {
                if let Some(ids) = self.index.category_index.get(category) {
                    results.extend(ids);
                }
            }
        }
        
        // Time range
        if let Some(range) = &query.time_range {
            let ids: Vec<_> = self.index.time_index
                .range(range.start..range.end)
                .map(|(_, id)| id)
                .collect();
            results.extend(ids);
        }
        
        // Fetch and return records
        results.into_iter()
            .filter_map(|id| self.get_record(id))
            .collect()
    }
}
```

## Integration Example

```rust
pub fn setup_notification_system(world: &mut World) {
    let mut notification_manager = NotificationManager::new();
    
    // Setup default filters
    notification_manager.add_filter(NotificationFilter {
        id: FilterId::new(),
        name: "Suppress during fast-forward".into(),
        enabled: true,
        conditions: vec![
            FilterCondition::DuringTimeScale { min: 100.0, max: f32::MAX },
            FilterCondition::PriorityBelow(NotificationPriority::High),
        ],
        action: FilterAction::Block,
    });
    
    // Setup aggregation rules
    notification_manager.add_aggregation_rule(AggregationRule {
        id: RuleId::new(),
        name: "Aggregate deaths".into(),
        trigger: AggregationTrigger::Count { 
            threshold: 5, 
            time_window: Duration::from_secs(10) 
        },
        aggregator: Box::new(CategoryAggregator {
            category: NotificationCategory::CreatureLifecycle,
            time_window: Duration::from_secs(10),
        }),
    });
    
    // Setup importance learning
    notification_manager.enable_importance_learning();
    
    world.insert_resource(notification_manager);
}

// Handle incoming events
pub fn process_creature_events(
    mut events: EventReader<CreatureEvent>,
    mut notifications: ResMut<NotificationManager>,
) {
    for event in events.iter() {
        match event {
            CreatureEvent::Died { entity, cause, .. } => {
                if notifications.should_notify_death(entity, cause) {
                    notifications.push(Notification {
                        category: NotificationCategory::CreatureLifecycle,
                        priority: cause.severity().into(),
                        title: format!("Creature Death: {}", cause.describe()),
                        message: format!("A creature has died from {}", cause),
                        icon: Some(IconType::Death),
                        color: Color::RED,
                        related_entities: vec![*entity],
                        actions: vec![
                            NotificationAction::Focus(*entity),
                            NotificationAction::Dismiss,
                        ],
                        // ...
                    });
                }
            }
            // ... other events
        }
    }
}
```

This notification system provides intelligent, context-aware notifications that keep users informed without overwhelming them.