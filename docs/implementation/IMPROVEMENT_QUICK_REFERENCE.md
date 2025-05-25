# Improvement Quick Reference Guide

This guide provides practical code examples, patterns, and quick wins for implementing the improvements outlined in the improvement plan.

## Table of Contents

1. [Entity Versioning Patterns](#entity-versioning-patterns)
2. [Decision System Patterns](#decision-system-patterns)
3. [Spatial Optimization Patterns](#spatial-optimization-patterns)
4. [Parallel Processing Patterns](#parallel-processing-patterns)
5. [Memory Pool Patterns](#memory-pool-patterns)
6. [Common Pitfalls](#common-pitfalls)
7. [Performance Checklist](#performance-checklist)
8. [Quick Wins](#quick-wins)

## Entity Versioning Patterns

### Before: No Version Tracking
```rust
// Problem: Can't detect stale entity references
pub struct Movement {
    target: Option<Entity>,
}

impl Movement {
    fn update(&mut self, world: &World) {
        if let Some(target) = self.target {
            // May crash if entity was despawned!
            let pos = world.get::<Position>(target).unwrap();
        }
    }
}
```

### After: Versioned Entities
```rust
// Solution: Version tracking prevents stale references
#[derive(Component)]
pub struct Version<T> {
    generation: u32,
    data: T,
}

pub struct Movement {
    target: Option<(Entity, u32)>, // Store generation
}

impl Movement {
    fn update(&mut self, world: &World) {
        if let Some((entity, gen)) = self.target {
            // Safe check with generation
            if let Some(Version { generation, data }) = world.get::<Version<Position>>(entity) {
                if *generation == gen {
                    // Use position safely
                } else {
                    // Handle stale reference
                    self.target = None;
                }
            }
        }
    }
}
```

## Decision System Patterns

### Before: Tightly Coupled Decision Making
```rust
// Problem: Decision logic mixed with state access
fn decide_action(creature: &Creature, world: &World) -> Action {
    let hunger = creature.needs.hunger;
    let mut best_food = None;
    
    // Direct world queries in decision logic
    for entity in world.query::<Food>() {
        let distance = calculate_distance(creature.pos, entity.pos);
        // Complex logic mixed with queries...
    }
    
    // Hard to test, profile, or optimize
    Action::Move { target: best_food }
}
```

### After: Decoupled Decision System
```rust
// Solution: Pure decision functions with clear inputs
#[derive(Clone)]
pub struct DecisionContext {
    pub needs: NeedState,
    pub position: Vec2,
    pub nearby_food: Vec<FoodInfo>,
    pub nearby_creatures: Vec<CreatureInfo>,
}

// Pure function - easy to test and optimize
pub fn decide_action(ctx: &DecisionContext) -> Decision {
    let food_utilities = ctx.nearby_food.iter()
        .map(|food| calculate_food_utility(ctx.needs.hunger, food))
        .collect::<Vec<_>>();
    
    Decision::Eat { 
        target: find_best_option(&food_utilities) 
    }
}

// Separate system for gathering context
pub fn gather_decision_context(
    creature: Entity,
    world: &World,
    spatial: &SpatialGrid,
) -> DecisionContext {
    let pos = world.get::<Position>(creature).unwrap();
    DecisionContext {
        position: pos.0,
        needs: world.get::<Needs>(creature).unwrap().clone(),
        nearby_food: spatial.query_radius(pos.0, PERCEPTION_RADIUS)
            .filter_map(|e| world.get::<Food>(e))
            .map(|f| f.into())
            .collect(),
        nearby_creatures: vec![], // Similar pattern
    }
}
```

## Spatial Optimization Patterns

### Before: Naive Spatial Queries
```rust
// Problem: O(n²) complexity for finding nearby entities
fn find_nearby_creatures(pos: Vec2, world: &World) -> Vec<Entity> {
    let mut nearby = Vec::new();
    
    // Checks EVERY entity!
    for (entity, other_pos) in world.query::<&Position>() {
        if distance(pos, other_pos.0) < INTERACTION_RANGE {
            nearby.push(entity);
        }
    }
    
    nearby
}
```

### After: Spatial Hash Grid
```rust
// Solution: O(1) average case spatial queries
pub struct SpatialHashGrid {
    cells: HashMap<(i32, i32), Vec<Entity>>,
    cell_size: f32,
}

impl SpatialHashGrid {
    pub fn query_radius(&self, center: Vec2, radius: f32) -> impl Iterator<Item = Entity> + '_ {
        let min_cell = self.world_to_cell(center - Vec2::splat(radius));
        let max_cell = self.world_to_cell(center + Vec2::splat(radius));
        
        (min_cell.0..=max_cell.0)
            .flat_map(move |x| {
                (min_cell.1..=max_cell.1)
                    .filter_map(move |y| self.cells.get(&(x, y)))
                    .flatten()
                    .copied()
            })
    }
    
    // Incremental updates instead of full rebuilds
    pub fn update_entity(&mut self, entity: Entity, old_pos: Vec2, new_pos: Vec2) {
        let old_cell = self.world_to_cell(old_pos);
        let new_cell = self.world_to_cell(new_pos);
        
        if old_cell != new_cell {
            // Remove from old cell
            if let Some(entities) = self.cells.get_mut(&old_cell) {
                entities.retain(|&e| e != entity);
            }
            
            // Add to new cell
            self.cells.entry(new_cell)
                .or_insert_with(Vec::new)
                .push(entity);
        }
    }
}
```

## Parallel Processing Patterns

### Before: Sequential System Updates
```rust
// Problem: Only uses one CPU core
pub fn update_all_creatures(world: &mut World) {
    let creatures: Vec<_> = world.query::<&Creature>().collect();
    
    for creature in creatures {
        update_movement(creature, world);
        update_needs(creature, world);
        update_decisions(creature, world);
    }
}
```

### After: Parallel System with Bevy
```rust
// Solution: Utilize all CPU cores efficiently
use bevy::prelude::*;
use bevy::ecs::query::QueryEntityError;

// Define parallel-safe systems
fn update_movement_system(
    mut query: Query<(&mut Position, &Velocity), With<Creature>>,
    time: Res<Time>,
) {
    // Bevy automatically parallelizes this!
    query.par_iter_mut().for_each_mut(|(mut pos, vel)| {
        pos.0 += vel.0 * time.delta_seconds();
    });
}

fn update_needs_system(
    mut query: Query<&mut Needs, With<Creature>>,
    time: Res<Time>,
) {
    query.par_iter_mut().for_each_mut(|mut needs| {
        needs.hunger += HUNGER_RATE * time.delta_seconds();
        needs.thirst += THIRST_RATE * time.delta_seconds();
    });
}

// Configure parallel execution
app.add_systems(Update, (
    update_movement_system,
    update_needs_system,
).chain()); // Or use .distributive_run_if() for conditional parallel execution
```

## Memory Pool Patterns

### Before: Frequent Allocations
```rust
// Problem: Allocates new Vec every frame
fn find_path(start: Vec2, end: Vec2) -> Vec<Vec2> {
    let mut path = Vec::new(); // Allocation!
    let mut open_set = Vec::new(); // Another allocation!
    let mut closed_set = HashSet::new(); // And another!
    
    // A* algorithm...
    
    path
}
```

### After: Object Pooling
```rust
// Solution: Reuse allocations
use std::cell::RefCell;

thread_local! {
    static PATH_POOL: RefCell<Vec<Vec<Vec2>>> = RefCell::new(Vec::new());
}

pub struct PooledPath {
    path: Vec<Vec2>,
}

impl PooledPath {
    pub fn acquire() -> Self {
        PATH_POOL.with(|pool| {
            let mut pool = pool.borrow_mut();
            let mut path = pool.pop().unwrap_or_else(Vec::new);
            path.clear();
            PooledPath { path }
        })
    }
}

impl Drop for PooledPath {
    fn drop(&mut self) {
        if self.path.capacity() < 1024 { // Don't pool huge allocations
            PATH_POOL.with(|pool| {
                pool.borrow_mut().push(std::mem::take(&mut self.path));
            });
        }
    }
}

// Usage
fn find_path(start: Vec2, end: Vec2) -> PooledPath {
    let mut path = PooledPath::acquire();
    // Use path.path as normal Vec<Vec2>
    path
}
```

## Common Pitfalls

### 1. Over-Engineering Systems
```rust
// ❌ Bad: Too many abstractions
trait System {
    type Input;
    type Output;
    fn process(&self, input: Self::Input) -> Self::Output;
}

trait SystemManager {
    fn register_system<S: System>(&mut self, system: S);
}

// ✅ Good: Simple and direct
fn update_creatures(world: &mut World) {
    // Direct, simple logic
}
```

### 2. Premature Optimization
```rust
// ❌ Bad: Optimizing without profiling
#[inline(always)]
fn calculate_distance_simd(a: Vec2, b: Vec2) -> f32 {
    // Complex SIMD code for simple operation
}

// ✅ Good: Profile first, optimize hot paths
fn calculate_distance(a: Vec2, b: Vec2) -> f32 {
    (a - b).length()
}
```

### 3. Ignoring Bevy's Built-in Features
```rust
// ❌ Bad: Custom threading
use std::thread;
fn parallel_update(creatures: Vec<Creature>) {
    let handles: Vec<_> = creatures.chunks(100)
        .map(|chunk| {
            thread::spawn(move || {
                // Process chunk
            })
        })
        .collect();
}

// ✅ Good: Use Bevy's parallel queries
query.par_iter_mut().for_each_mut(|creature| {
    // Bevy handles threading
});
```

### 4. Inefficient Component Access
```rust
// ❌ Bad: Multiple lookups
fn update(entity: Entity, world: &World) {
    let pos = world.get::<Position>(entity).unwrap();
    let vel = world.get::<Velocity>(entity).unwrap();
    let health = world.get::<Health>(entity).unwrap();
}

// ✅ Good: Single query
fn update(mut query: Query<(&Position, &Velocity, &Health)>) {
    for (pos, vel, health) in &query {
        // All components in one lookup
    }
}
```

## Performance Checklist

Before implementing any optimization, verify:

- [ ] **Profile First**: Use `cargo flamegraph` or `tracy` to identify actual bottlenecks
- [ ] **Measure Impact**: Benchmark before and after changes
- [ ] **Check Bevy Docs**: Ensure you're not duplicating built-in functionality
- [ ] **Consider Complexity**: Will this make the code significantly harder to maintain?
- [ ] **Test Thoroughly**: Optimizations often introduce subtle bugs

### Quick Profiling Setup
```bash
# Install profiling tools
cargo install flamegraph
cargo install cargo-criterion

# Create benchmark
# benches/spatial.rs
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_spatial_query(c: &mut Criterion) {
    c.bench_function("spatial_radius_query", |b| {
        let grid = setup_spatial_grid();
        b.iter(|| {
            grid.query_radius(Vec2::ZERO, 100.0).count()
        });
    });
}

criterion_group!(benches, bench_spatial_query);
criterion_main!(benches);
```

## Quick Wins

### 1. Enable Bevy's Optimization Features
```toml
# Cargo.toml
[dependencies]
bevy = { version = "0.12", features = ["trace", "dynamic_linking"] }

[profile.dev]
opt-level = 1 # Enable basic optimizations in debug mode

[profile.dev.package."*"]
opt-level = 3 # Optimize dependencies in debug mode
```

### 2. Use Type Aliases for Complex Queries
```rust
// Makes code cleaner and potentially faster (compiler hints)
type CreatureQuery<'w, 's> = Query<'w, 's, (
    &'static Position,
    &'static mut Velocity,
    &'static Needs,
), With<Creature>>;

fn movement_system(mut creatures: CreatureQuery) {
    // Clear, efficient code
}
```

### 3. Batch Entity Spawning
```rust
// ❌ Slow: Individual spawns
for i in 0..1000 {
    commands.spawn(CreatureBundle::new(i));
}

// ✅ Fast: Batched spawning
commands.spawn_batch((0..1000).map(|i| CreatureBundle::new(i)));
```

### 4. Use Changed/Added Filters
```rust
// Only process entities that actually changed
fn update_spatial_index(
    mut spatial: ResMut<SpatialGrid>,
    moved: Query<(Entity, &Position), Changed<Position>>,
) {
    for (entity, pos) in &moved {
        spatial.update_entity(entity, pos.0);
    }
}
```

### 5. Prefer Resources Over Singleton Entities
```rust
// ❌ Inefficient: Singleton entity
fn get_world_settings(query: Query<&WorldSettings>) -> &WorldSettings {
    query.single()
}

// ✅ Efficient: Resource
fn get_world_settings(settings: Res<WorldSettings>) -> &WorldSettings {
    &settings
}
```

## Implementation Priority Matrix

| Improvement | Effort | Impact | Do First? |
|------------|---------|---------|-----------|
| Spatial optimization | Low | High | ✅ |
| Parallel queries | Low | High | ✅ |
| Decision caching | Medium | High | ✅ |
| Memory pooling | Medium | Medium | ⚠️ |
| SIMD math | High | Low | ❌ |
| Custom allocators | High | Low | ❌ |

## Final Tips

1. **Start Small**: Implement one improvement at a time
2. **Measure Everything**: Use benchmarks to validate improvements
3. **Keep It Simple**: Prefer clarity over clever optimizations
4. **Document Changes**: Explain why optimizations were made
5. **Review Regularly**: Performance characteristics change as code evolves

Remember: The best optimization is often better algorithms, not micro-optimizations!