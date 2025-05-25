# Bevy ECS Migration Plan

## Overview
This document outlines the migration from our custom ECS-like architecture to full Bevy ECS integration.

## Migration Steps

### Phase 1: Components (High Priority)
1. **Convert data structs to Bevy Components**
   - [x] Creature → CreatureBundle with components
   - [ ] Resource → ResourceBundle with components
   - [ ] Position, Velocity as separate components
   - [ ] Health, Needs as components
   - [ ] CreatureState as component

2. **Component Design**
   ```rust
   // Example structure:
   #[derive(Component)]
   struct Position(Vec2);
   
   #[derive(Component)]
   struct Velocity(Vec2);
   
   #[derive(Component)]
   struct Health { current: f32, max: f32 }
   
   #[derive(Component)]
   struct Needs { hunger: f32, thirst: f32, energy: f32 }
   
   #[derive(Component)]
   struct CreatureTag; // Marker component
   ```

### Phase 2: Resources (High Priority)
1. **Convert World to Bevy Resources**
   - [ ] SpatialGrid → Resource
   - [ ] EventBus → Use Bevy Events
   - [ ] GameTime → Use Bevy Time
   - [ ] SimulationSettings → Resource

### Phase 3: Systems (High Priority)
1. **Convert update functions to Bevy Systems**
   - [ ] DecisionSystem
   - [ ] MovementSystem
   - [ ] NeedsSystem
   - [ ] ResourceSpawnerSystem
   - [ ] DeathSystem

2. **System Ordering**
   ```rust
   app.add_systems(Update, (
       decision_system,
       movement_system,
       needs_system,
       death_system,
   ).chain());
   ```

### Phase 4: Queries (Medium Priority)
1. **Implement proper spatial queries**
   - [ ] nearby_creatures using spatial grid
   - [ ] nearby_threats using spatial grid
   - [ ] nearby_resources using spatial grid

### Phase 5: Events
1. **Replace GameEvent with Bevy Events**
   - [ ] EventWriter<CreatureSpawned>
   - [ ] EventWriter<CreatureDied>
   - [ ] EventWriter<ResourceConsumed>

## Key Architectural Changes

### Before (Custom ECS)
```rust
struct World {
    entities: EntityManager,
    creatures: HashMap<Entity, Creature>,
    resources: HashMap<Entity, Resource>,
}
```

### After (Bevy ECS)
```rust
// Components stored in Bevy's World
// Accessed via Queries
fn my_system(
    query: Query<(&Position, &Velocity), With<CreatureTag>>
) { }
```

## Benefits
1. **Parallel Systems** - Bevy automatically parallelizes compatible systems
2. **Better Performance** - Archetypal ECS storage
3. **Integration** - Direct path to rendering, UI, input
4. **Tools** - Bevy Inspector, diagnostics, etc.

## Risks & Mitigations
1. **Test Breakage** - Will need to rewrite many tests
   - Mitigation: Create test helpers for Bevy world setup
2. **API Changes** - Public API will change significantly
   - Mitigation: Document changes clearly
3. **Performance Regression** - Initial version might be slower
   - Mitigation: Profile and optimize after migration

## Success Criteria
- [ ] All existing tests pass (after adaptation)
- [ ] Simulation runs at 60+ FPS with 1000 creatures
- [ ] Clean separation of concerns via Systems
- [ ] Ready for Week 7-8 UI/rendering features