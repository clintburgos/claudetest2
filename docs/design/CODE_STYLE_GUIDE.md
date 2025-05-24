# Code Style & Best Practices Guide

## Overview
This document establishes coding standards and best practices for the creature simulation project, ensuring consistency, maintainability, and performance across the codebase.

## Rust Style Guidelines

### Formatting
- **Use rustfmt** with default settings
- **Line length**: 100 characters (configured in rustfmt.toml)
- **Indentation**: 4 spaces (Rust default)

```toml
# rustfmt.toml
max_width = 100
use_small_heuristics = "Default"
imports_granularity = "Crate"
group_imports = "StdExternalCrate"
```

### Naming Conventions

```rust
// Modules: snake_case
mod creature_system;
mod world_generation;

// Types: PascalCase
struct CreatureData { }
enum BiomeType { }
trait Drawable { }

// Functions and methods: snake_case
fn calculate_distance() { }
fn update_creature_needs() { }

// Constants: SCREAMING_SNAKE_CASE
const MAX_CREATURES: usize = 10_000;
const HUNGER_DECAY_RATE: f32 = 0.1;

// Variables: snake_case
let creature_count = 42;
let mut needs_update = true;

// Bevy Components: PascalCase (like structs)
#[derive(Component)]
struct Position(Vec2);

// Bevy Systems: snake_case with descriptive names
fn move_hungry_creatures() { }
fn animate_idle_creatures() { }
```

### Import Organization

```rust
// Standard library
use std::collections::HashMap;
use std::sync::Arc;

// External crates
use bevy::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};

// Internal modules
use crate::creature::{Creature, CreatureBundle};
use crate::world::BiomeType;
```

## Bevy-Specific Best Practices

### Component Design

```rust
// GOOD: Small, focused components
#[derive(Component, Debug, Clone, Copy)]
struct Position(Vec2);

#[derive(Component, Debug, Clone, Copy)]
struct Velocity(Vec2);

#[derive(Component, Debug)]
struct Health {
    current: f32,
    max: f32,
}

// BAD: Monolithic component
#[derive(Component)]
struct CreatureData {
    position: Vec2,
    velocity: Vec2,
    health: f32,
    max_health: f32,
    hunger: f32,
    // ... many more fields
}
```

### Bundle Organization

```rust
#[derive(Bundle)]
struct CreatureBundle {
    // Core components
    creature: Creature,
    position: Position,
    velocity: Velocity,
    
    // Visual components
    #[bundle]
    sprite: SpriteBundle,
    
    // Behavior components
    needs: Needs,
    genetics: Genetics,
    
    // Metadata
    name: Name,
}

impl CreatureBundle {
    pub fn new(position: Vec2, dna: DNA) -> Self {
        Self {
            creature: Creature::default(),
            position: Position(position),
            velocity: Velocity(Vec2::ZERO),
            sprite: SpriteBundle {
                transform: Transform::from_translation(position.extend(0.0)),
                ..default()
            },
            needs: Needs::new(),
            genetics: Genetics::from_dna(dna),
            name: Name::new(generate_name()),
        }
    }
}
```

### System Organization

```rust
// Group related systems in plugins
pub struct CreaturePlugin;

impl Plugin for CreaturePlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<CreatureSettings>()
            
            // Events
            .add_event::<CreatureSpawnEvent>()
            .add_event::<CreatureDeathEvent>()
            
            // Systems - use SystemSets for organization
            .add_systems(
                Update,
                (
                    // Input systems
                    handle_creature_selection,
                    
                    // Simulation systems
                    (
                        update_creature_needs,
                        make_decisions,
                        execute_actions,
                    )
                    .chain()
                    .in_set(SimulationSet::Creatures),
                    
                    // Cleanup systems
                    despawn_dead_creatures,
                )
            );
    }
}
```
### Query Best Practices

```rust
// GOOD: Specific queries with filters
fn update_hungry_creatures(
    mut query: Query<
        (&mut Needs, &Position, &mut Velocity),
        (With<Creature>, Without<Sleeping>)
    >,
) {
    for (mut needs, pos, mut vel) in &mut query {
        // Process only awake creatures
    }
}

// GOOD: Use change detection
fn respond_to_health_changes(
    query: Query<(&Health, &Name), Changed<Health>>,
) {
    for (health, name) in &query {
        info!("{} health changed to {}", name.0, health.current);
    }
}

// BAD: Overly broad queries
fn update_everything(
    mut query: Query<&mut Transform>,
) {
    // This queries ALL entities with Transform!
}
```

## Error Handling

### Use Result Types

```rust
// GOOD: Explicit error handling
pub fn load_world_data(path: &Path) -> Result<WorldData, WorldError> {
    let data = std::fs::read_to_string(path)
        .map_err(|e| WorldError::Io(e))?;
    
    let world = serde_json::from_str(&data)
        .map_err(|e| WorldError::Parse(e))?;
    
    Ok(world)
}

// Define custom error types
#[derive(Debug, thiserror::Error)]
pub enum WorldError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Parse error: {0}")]
    Parse(#[from] serde_json::Error),
    
    #[error("Invalid world size: {0}")]
    InvalidSize(String),
}
```

### Graceful Degradation

```rust
// Handle missing resources gracefully
fn update_with_settings(
    settings: Option<Res<GameSettings>>,
    mut query: Query<&mut Creature>,
) {
    let settings = settings.as_deref().unwrap_or(&GameSettings::default());
    
    for mut creature in &mut query {
        creature.update(settings);
    }
}
```

## Documentation Standards

### Module Documentation

```rust
//! # Creature System
//! 
//! This module handles all creature-related functionality including:
//! - Spawning and despawning
//! - Need management
//! - Decision making
//! - Movement and interactions
//! 
//! ## Example
//! ```
//! use creature_system::spawn_creature;
//! 
//! let creature = spawn_creature(&mut commands, position, dna);
//! ```

use bevy::prelude::*;
```

### Function Documentation

```rust
/// Spawns a new creature at the specified position.
/// 
/// # Arguments
/// * `commands` - Bevy commands for entity spawning
/// * `position` - World position for the creature
/// * `dna` - Genetic information for trait expression
/// 
/// # Returns
/// The Entity ID of the spawned creature
/// 
/// # Example
/// ```
/// let creature = spawn_creature(&mut commands, Vec2::new(100.0, 100.0), DNA::random());
/// ```
pub fn spawn_creature(
    commands: &mut Commands,
    position: Vec2,
    dna: DNA,
) -> Entity {
    // Implementation
}
```
### Inline Comments

```rust
// GOOD: Explain why, not what
fn calculate_trait_expression(gene: f32) -> f32 {
    // Sigmoid function ensures traits have diminishing returns
    // This prevents extreme values from dominating
    1.0 / (1.0 + (-4.0 * (gene - 0.5)).exp())
}

// BAD: Obvious comments
fn add_numbers(a: f32, b: f32) -> f32 {
    // Add a and b together
    a + b
}
```

## Testing Practices

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_need_decay() {
        let mut needs = Needs {
            hunger: 100.0,
            thirst: 100.0,
            ..default()
        };
        
        needs.update(1.0); // 1 second elapsed
        
        assert!(needs.hunger < 100.0);
        assert!(needs.hunger > 90.0);
    }
    
    #[test]
    fn test_creature_spawning() {
        // Use Bevy's test app
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        
        let creature_id = app.world.spawn(CreatureBundle::new(
            Vec2::ZERO,
            DNA::random(),
        )).id();
        
        assert!(app.world.get::<Creature>(creature_id).is_some());
    }
}
```

### Integration Tests

```rust
// tests/creature_simulation.rs
use bevy::prelude::*;
use creature_sim::*;

#[test]
fn test_full_creature_lifecycle() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        CreaturePlugin,
    ));
    
    // Spawn parent creatures
    let parent1 = spawn_creature(&mut app.world, Vec2::new(0.0, 0.0));
    let parent2 = spawn_creature(&mut app.world, Vec2::new(10.0, 0.0));
    
    // Run simulation
    for _ in 0..100 {
        app.update();
    }
    
    // Check for offspring
    let creature_count = app.world.query::<&Creature>().iter(&app.world).count();
    assert!(creature_count >= 2);
}
```

## Performance Best Practices

### Profile Annotations

```rust
#[tracing::instrument(skip(query))]
fn expensive_pathfinding(
    mut query: Query<(&Position, &Target, &mut Path)>,
) {
    for (pos, target, mut path) in &mut query {
        // Expensive operation - will show up in profiler
        *path = calculate_path(pos.0, target.0);
    }
}
```

### Optimization Patterns

```rust
// GOOD: Early returns
fn process_creature(creature: &Creature) -> Option<Action> {
    if creature.is_sleeping() {
        return None;
    }
    
    if creature.health <= 0.0 {
        return Some(Action::Die);
    }
    
    // Complex logic only for active creatures
    Some(decide_action(creature))
}

// GOOD: Pre-allocate collections
fn gather_nearby_food(position: Vec2, spatial_index: &SpatialIndex) -> Vec<Entity> {
    let mut food = Vec::with_capacity(32); // Assume ~32 food items nearby
    
    for entity in spatial_index.query_radius(position, SEARCH_RADIUS) {
        if is_food(entity) {
            food.push(entity);
        }
    }
    
    food
}
```
## Project-Specific Conventions

### Resource Management

```rust
// Resources should have sensible defaults
#[derive(Resource)]
pub struct SimulationSettings {
    pub creature_spawn_rate: f32,
    pub max_creatures: usize,
    pub time_scale: f32,
}

impl Default for SimulationSettings {
    fn default() -> Self {
        Self {
            creature_spawn_rate: 0.1,
            max_creatures: 1000,
            time_scale: 1.0,
        }
    }
}
```

### Event Patterns

```rust
// Events should be small and focused
#[derive(Event)]
pub struct CreatureSpawnEvent {
    pub position: Vec2,
    pub dna: DNA,
}

#[derive(Event)]
pub struct CreatureDeathEvent {
    pub entity: Entity,
    pub cause: DeathCause,
}

// Event handlers should be idempotent
fn handle_creature_death(
    mut commands: Commands,
    mut events: EventReader<CreatureDeathEvent>,
    query: Query<Entity, With<Creature>>,
) {
    for event in events.read() {
        // Check entity still exists
        if query.get(event.entity).is_ok() {
            commands.entity(event.entity).despawn_recursive();
        }
    }
}
```

### Constants Organization

```rust
// Group related constants
pub mod creature_constants {
    pub const MAX_HEALTH: f32 = 100.0;
    pub const MAX_ENERGY: f32 = 100.0;
    pub const HUNGER_DECAY_RATE: f32 = 0.1;
    pub const THIRST_DECAY_RATE: f32 = 0.15;
}

pub mod world_constants {
    pub const CHUNK_SIZE: i32 = 32;
    pub const TILE_SIZE: f32 = 64.0;
    pub const MAX_ELEVATION: f32 = 100.0;
}
```

## Safety and Correctness

### Unsafe Code

```rust
// AVOID unsafe code unless absolutely necessary
// When used, document thoroughly

/// SAFETY: This function assumes the index is within bounds.
/// The caller must ensure index < buffer.len()
unsafe fn get_unchecked_mut(buffer: &mut [f32], index: usize) -> &mut f32 {
    buffer.get_unchecked_mut(index)
}

// Prefer safe alternatives
fn get_mut(buffer: &mut [f32], index: usize) -> Option<&mut f32> {
    buffer.get_mut(index)
}
```

### Assertions and Invariants

```rust
// Use debug_assert! for expensive checks
fn normalize_vector(mut vec: Vec2) -> Vec2 {
    debug_assert!(vec.length() > 0.0, "Cannot normalize zero vector");
    
    vec.normalize()
}

// Document invariants
/// Position component. Always within world bounds.
#[derive(Component)]
struct Position(Vec2);

impl Position {
    pub fn new(pos: Vec2) -> Self {
        assert!(is_within_world_bounds(pos), "Position out of bounds");
        Self(pos)
    }
}
```

## Git Commit Conventions

### Commit Message Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types
- `feat`: New feature
- `fix`: Bug fix
- `perf`: Performance improvement
- `refactor`: Code refactoring
- `test`: Adding tests
- `docs`: Documentation changes
- `style`: Code style changes
- `chore`: Maintenance tasks

### Examples

```bash
feat(creature): Add genetic crossover system

Implement DNA crossover with configurable mutation rates.
Creatures can now reproduce and pass traits to offspring.

Closes #42

---

perf(rendering): Implement creature instancing

Reduce draw calls from O(n) to O(1) by using instanced
rendering for all creatures of the same type.

Benchmark: 1000 creatures now render at 120 FPS (was 45 FPS)

---

fix(ai): Prevent creatures from getting stuck in corners

Add wall detection to pathfinding algorithm. Creatures
now properly navigate around obstacles.

Fixes #13
```

## Code Review Checklist

- [ ] Code follows style guidelines
- [ ] Functions have appropriate documentation
- [ ] Complex logic includes explanatory comments
- [ ] Error cases are handled properly
- [ ] Performance-critical sections are optimized
- [ ] Tests cover new functionality
- [ ] No clippy warnings
- [ ] Components are small and focused
- [ ] Systems use appropriate queries and filters

---
*Last Updated: 2024-12-XX*