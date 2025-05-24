# Phase 1 Class Diagrams

## Core System Class Diagram

```mermaid
classDiagram
    class Entity {
        -u32 id
        +new(id: u32) Entity
    }
    
    class EntityManager {
        -u32 next_id
        -HashSet~Entity~ active_entities
        -Vec~u32~ recycled_ids
        +create() Entity
        +destroy(entity: Entity)
        +is_alive(entity: Entity) bool
    }
    
    class World {
        +EntityManager entities
        +HashMap~Entity, Creature~ creatures
        +HashMap~Entity, Resource~ resources  
        +SpatialGrid spatial_grid
        +EventBus events
        +GameTime time
        +ErrorBoundary error_boundary
    }
    
    class GameTime {
        +f64 total_seconds
        +f32 delta_seconds
        +u64 frame_count
    }
    
    World --> EntityManager
    World --> GameTime
    EntityManager --> Entity
```

## Creature System Class Diagram

```mermaid
classDiagram
    class Creature {
        +Entity id
        +Vec2 position
        +Vec2 velocity
        +Health health
        +Needs needs
        +CreatureState state
        +f32 age
        +f32 size
    }
    
    class Health {
        +f32 current
        +f32 max
        +damage(amount: f32)
        +heal(amount: f32)
        +percentage() f32
    }
    
    class Needs {
        +f32 hunger
        +f32 thirst
        +f32 energy
        +get_most_critical() NeedType
        +is_critical() bool
    }
    
    class CreatureState {
        <<enumeration>>
        Idle
        Moving(Vec2 target)
        Eating
        Drinking
        Resting
        Dead
    }
    
    Creature --> Health
    Creature --> Needs
    Creature --> CreatureState
```

## Resource System Class Diagram

```mermaid
classDiagram
    class Resource {
        +Entity id
        +Vec2 position
        +ResourceType resource_type
        +f32 amount
        +f32 max_amount
        +consume(amount: f32) f32
        +regenerate(amount: f32)
        +is_depleted() bool
    }
    
    class ResourceType {
        <<enumeration>>
        Food
        Water
        +regeneration_rate() f32
        +consumption_rate() f32
    }
    
    Resource --> ResourceType
```

## System Architecture Class Diagram

```mermaid
classDiagram
    class System {
        <<interface>>
        +update(world: &mut World, dt: f32)
        +name() &str
    }
    
    class MovementSystem {
        -Vec~PathRequest~ pathfinding_requests
        +update(world: &mut World, dt: f32)
        +name() &str
        -update_movement(creature: &mut Creature, target: Vec2, dt: f32)
    }
    
    class NeedsSystem {
        -f32 metabolism_rate
        -f32 thirst_rate
        -f32 rest_recovery_rate
        +update(world: &mut World, dt: f32)
        +name() &str
    }
    
    class DecisionSystem {
        -f32 decision_interval
        -f32 time_since_last_decision
        +update(world: &mut World, dt: f32)
        +name() &str
        -make_decision(creature: &Creature, world: &World) Decision
        -apply_decision(creature: &mut Creature, decision: Decision)
    }
    
    class ResourceSystem {
        -f32 regeneration_timer
        -f32 regeneration_interval
        +update(world: &mut World, dt: f32)
        +name() &str
    }
    
    System <|.. MovementSystem
    System <|.. NeedsSystem
    System <|.. DecisionSystem
    System <|.. ResourceSystem
```

## Spatial System Class Diagram

```mermaid
classDiagram
    class SpatialGrid {
        -f32 cell_size
        -HashMap~GridCoord, Vec~Entity~~ cells
        -HashMap~Entity, GridCoord~ entity_positions
        +new(cell_size: f32) SpatialGrid
        +insert(entity: Entity, position: Vec2)
        +remove(entity: Entity)
        +update(entity: Entity, position: Vec2)
        +query_radius(center: Vec2, radius: f32) Vec~Entity~
        +query_rect(min: Vec2, max: Vec2) Vec~Entity~
        -world_to_grid(pos: Vec2) GridCoord
    }
    
    class GridCoord {
        +i32 x
        +i32 y
        +neighbors() Vec~GridCoord~
        +distance_to(other: GridCoord) f32
    }
    
    SpatialGrid --> GridCoord
```

## Event System Class Diagram

```mermaid
classDiagram
    class EventBus {
        -VecDeque~GameEvent~ events
        -HashMap~TypeId, Vec~EventHandler~~ handlers
        +emit(event: GameEvent)
        +subscribe(handler: EventHandler)
        +process(world: &mut World)
    }
    
    class EventHandler {
        <<interface>>
        +handle(event: &GameEvent, world: &mut World)
    }
    
    class GameEvent {
        <<enumeration>>
        CreatureSpawned(Entity, Vec2)
        CreatureDied(Entity, DeathCause)
        ResourceDepleted(Entity)
        ResourceReplenished(Entity)
        CreatureStateChanged(Entity, CreatureState, CreatureState)
    }
    
    class DeathCause {
        <<enumeration>>
        Starvation
        Dehydration
        OldAge
    }
    
    EventBus --> EventHandler
    EventBus --> GameEvent
    GameEvent --> DeathCause
```

## Error System Class Diagram

```mermaid
classDiagram
    class ErrorBoundary {
        -HashMap~ErrorType, RecoveryStrategy~ recovery_strategies
        -RingBuffer~ErrorEvent~ error_log
        -CorruptionDetector corruption_detector
        +check_and_recover(world: &mut World)
        +register_strategy(error_type: ErrorType, strategy: RecoveryStrategy)
    }
    
    class ErrorType {
        <<enumeration>>
        CreatureStuck
        InvalidPosition
        ResourceNegative
        PathfindingFailed
    }
    
    class RecoveryStrategy {
        <<interface>>
        +can_recover(error: &SimulationError) bool
        +recover(world: &mut World, error: &SimulationError) Result
    }
    
    class CreatureStuckRecovery {
        +can_recover(error: &SimulationError) bool
        +recover(world: &mut World, error: &SimulationError) Result
    }
    
    ErrorBoundary --> ErrorType
    ErrorBoundary --> RecoveryStrategy
    RecoveryStrategy <|.. CreatureStuckRecovery
```

## UI System Class Diagram

```mermaid
classDiagram
    class UISystem {
        -Option~Entity~ selected_creature
        -bool show_stats
        -Vec2 time_controls_pos
        -Vec2 stats_panel_pos
        +update(world: &World, input: &InputState, camera: &CameraController)
        +render(world: &World, time: &TimeSystem) UIRenderData
        -find_creature_at(pos: Vec2, world: &World) Option~Entity~
    }
    
    class UIElement {
        <<enumeration>>
        Text(String, Vec2)
        Button(String, Vec2, Vec2, UIAction)
        Panel(Vec2, Vec2, Vec~UIElement~)
        ProgressBar(Vec2, Vec2, f32)
    }
    
    class UIAction {
        <<enumeration>>
        TogglePause
        SetTimeScale(f32)
        SelectCreature(Entity)
        ToggleStats
    }
    
    class UIRenderData {
        +Vec~UIElement~ elements
    }
    
    UISystem --> UIElement
    UISystem --> UIRenderData
    UIElement --> UIAction
```

## Camera System Class Diagram

```mermaid
classDiagram
    class CameraController {
        +Vec2 position
        +f32 zoom
        +Vec2 viewport_size
        +f32 movement_speed
        +f32 zoom_speed
        +update(input: &InputState, dt: f32)
        +world_to_screen(world_pos: Vec2) Vec2
        +screen_to_world(screen_pos: Vec2) Vec2
        +get_visible_bounds() AABB
    }
    
    class InputState {
        +HashMap~KeyCode, bool~ keys_pressed
        +Vec2 mouse_position
        +bool mouse_clicked
        +f32 scroll_delta
        +is_key_pressed(key: KeyCode) bool
        +key_just_pressed(key: KeyCode) bool
    }
    
    CameraController --> InputState
```

## Main Game Loop Class Diagram

```mermaid
classDiagram
    class Game {
        -World world
        -Vec~System~ systems
        -TimeSystem time_system
        -CameraController camera
        -UISystem ui
        -Renderer renderer
        -ProfilerSystem profiler
        +new() Game
        +update(real_dt: f32, input: &InputState)
        +render()
        -spawn_initial_entities()
    }
    
    class TimeSystem {
        -f64 accumulated_time
        -f64 game_time
        -f32 time_scale
        -bool paused
        +update(real_dt: f32) f32
        +set_time_scale(scale: f32)
        +toggle_pause()
    }
    
    class ProfilerSystem {
        -FrameProfiler frame_profiler
        -HashMap~SystemId, SystemProfiler~ system_profilers
        -bool show_overlay
        +begin_frame()
        +end_frame()
        +scope(name: &str) ProfileScope
        +get_metrics() PerformanceMetrics
    }
    
    Game --> World
    Game --> TimeSystem
    Game --> ProfilerSystem
    Game --> CameraController
    Game --> UISystem
```

## Sequence Diagram: Creature Decision and Movement

```mermaid
sequenceDiagram
    participant Game
    participant DecisionSystem
    participant Creature
    participant SpatialGrid
    participant MovementSystem
    participant World
    
    Game->>DecisionSystem: update(world, dt)
    DecisionSystem->>Creature: check needs
    Creature-->>DecisionSystem: hunger > 0.5
    DecisionSystem->>SpatialGrid: query_radius(position, 50.0)
    SpatialGrid-->>DecisionSystem: nearby entities
    DecisionSystem->>World: find food resources
    World-->>DecisionSystem: nearest food position
    DecisionSystem->>Creature: set state Moving(target)
    
    Game->>MovementSystem: update(world, dt)
    MovementSystem->>Creature: get state
    Creature-->>MovementSystem: Moving(target)
    MovementSystem->>Creature: calculate velocity
    MovementSystem->>Creature: update position
    MovementSystem->>SpatialGrid: update(entity, new_position)
```

These diagrams provide a clear visual representation of the Phase 1 architecture, showing the relationships between classes and the flow of data through the system.