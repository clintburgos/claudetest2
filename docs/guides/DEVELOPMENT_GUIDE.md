# Development Guide

## Table of Contents
1. [Code Style Guide](#code-style-guide)
2. [Best Practices](#best-practices)
3. [Development Workflow](#development-workflow)
4. [Quick Reference](#quick-reference)

---

## Code Style Guide

### Rust Code Style

We follow the standard Rust style guide with some project-specific conventions to ensure consistency and readability.

#### Naming Conventions

```rust
// Modules: snake_case
mod creature_system;

// Types: PascalCase
struct CreatureState { }
enum DecisionType { }
trait Behavioral { }

// Functions and methods: snake_case
fn calculate_utility() { }

// Constants: SCREAMING_SNAKE_CASE
const MAX_CREATURES: usize = 10_000;

// Variables: snake_case
let creature_position = Vec3::new(0.0, 0.0, 0.0);

// Enum variants: PascalCase
enum GoalType {
    SatisfyHunger,
    FindShelter,
}
```

#### Code Organization

```rust
// Standard import order
use std::collections::HashMap;         // 1. Standard library
use std::sync::Arc;

use bevy::prelude::*;                  // 2. External crates
use serde::{Deserialize, Serialize};

use crate::creature::Creature;         // 3. Internal crates
use crate::world::Terrain;

use super::goals::Goal;                // 4. Parent modules

// Module organization
pub struct MySystem {
    // Public fields first
    pub name: String,
    
    // Private fields after
    internal_state: State,
}

impl MySystem {
    // Constructors first
    pub fn new() -> Self { }
    
    // Public methods
    pub fn update(&mut self) { }
    
    // Private methods
    fn internal_helper(&self) { }
}
```

#### Documentation

```rust
/// Brief description of what this does.
/// 
/// Longer explanation with details about behavior,
/// edge cases, and important notes.
/// 
/// # Arguments
/// 
/// * `creature` - The creature making the decision
/// * `context` - Environmental context
/// 
/// # Returns
/// 
/// Returns `Some(Decision)` if a decision was made, `None` otherwise.
/// 
/// # Examples
/// 
/// ```
/// let decision = make_decision(&creature, &context);
/// ```
pub fn make_decision(creature: &Creature, context: &Context) -> Option<Decision> {
    // Implementation
}
```

#### Error Handling

```rust
// Use Result for operations that can fail
pub fn load_creature(id: EntityId) -> Result<Creature, LoadError> {
    // Prefer ? operator for propagation
    let data = load_data(id)?;
    let creature = deserialize_creature(data)?;
    Ok(creature)
}

// Use Option for values that might not exist
pub fn find_nearest_food(position: Vec3) -> Option<FoodSource> {
    // Return None instead of panic
}

// Custom error types
#[derive(Debug, thiserror::Error)]
pub enum CreatureError {
    #[error("Creature not found: {0}")]
    NotFound(EntityId),
    
    #[error("Invalid state transition: {0:?} to {1:?}")]
    InvalidTransition(State, State),
}
```

#### Performance-Critical Code

```rust
// Mark hot paths
#[inline(always)]
fn distance_squared(a: Vec3, b: Vec3) -> f32 {
    (a - b).length_squared()
}

// Avoid allocations in loops
let mut reusable_buffer = Vec::with_capacity(100);
for creature in creatures {
    reusable_buffer.clear();
    // Use buffer...
}

// Prefer iterators over indexing
creatures.iter_mut()
    .filter(|c| c.is_alive())
    .for_each(|c| c.update(delta));
```

---

## Best Practices

### Architecture Principles

#### 1. Data-Oriented Design
```rust
// Good: Separate hot and cold data
pub struct CreatureHotData {
    pub position: Vec3,
    pub velocity: Vec3,
    pub health: f32,
}

pub struct CreatureColdData {
    pub name: String,
    pub biography: String,
    pub relationships: HashMap<EntityId, Relationship>,
}

// Bad: Everything in one struct
pub struct Creature {
    pub position: Vec3,
    pub name: String,  // Cold data mixed with hot
    pub velocity: Vec3,
    pub biography: String,
}
```

#### 2. Entity Component System (ECS)
```rust
// Good: Small, focused components
#[derive(Component)]
struct Position(Vec3);

#[derive(Component)]
struct Velocity(Vec3);

#[derive(Component)]
struct Health(f32);

// Good: Systems that operate on component queries
fn movement_system(
    mut query: Query<(&mut Position, &Velocity), With<Creature>>,
    time: Res<Time>,
) {
    for (mut pos, vel) in query.iter_mut() {
        pos.0 += vel.0 * time.delta_seconds();
    }
}
```

#### 3. Composition Over Inheritance
```rust
// Good: Trait composition
trait Movable {
    fn move_to(&mut self, position: Vec3);
}

trait Mortal {
    fn take_damage(&mut self, amount: f32);
    fn is_alive(&self) -> bool;
}

struct Creature;
impl Movable for Creature { }
impl Mortal for Creature { }

// Bad: Deep inheritance hierarchies (not even possible in Rust!)
```

### Performance Guidelines

#### 1. Memory Efficiency
```rust
// Use appropriate integer sizes
pub struct CreatureStats {
    health: u8,       // 0-100, no need for u32
    hunger: u8,       // 0-100
    age_days: u16,    // Up to 65k days is plenty
}

// Pack related booleans into bitflags
bitflags! {
    pub struct CreatureFlags: u32 {
        const ALIVE = 0b00000001;
        const MOVING = 0b00000010;
        const HUNGRY = 0b00000100;
        const MATING = 0b00001000;
    }
}
```

#### 2. Avoiding Common Pitfalls
```rust
// Bad: Cloning large structures
let creatures_copy = creatures.clone(); // Expensive!

// Good: Use references or indices
let creature_ref = &creatures[index];

// Bad: String allocations in hot loops
for i in 0..1000 {
    let name = format!("Creature {}", i); // Allocates!
}

// Good: Pre-allocate or use static strings
let mut name_buffer = String::with_capacity(20);
for i in 0..1000 {
    name_buffer.clear();
    write!(&mut name_buffer, "Creature {}", i).unwrap();
}
```

#### 3. Parallelization
```rust
use rayon::prelude::*;

// Good: Parallel iteration for independent operations
creatures.par_iter_mut()
    .filter(|c| c.needs_update())
    .for_each(|c| c.update_needs());

// Good: Batch operations
const BATCH_SIZE: usize = 64;
creatures.par_chunks_mut(BATCH_SIZE)
    .for_each(|batch| {
        for creature in batch {
            creature.update();
        }
    });
```

### Code Quality

#### 1. Testing
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_creature_hunger() {
        let mut creature = Creature::new();
        assert_eq!(creature.hunger, 50.0);
        
        creature.update_needs(10.0);
        assert!(creature.hunger > 50.0);
    }
    
    #[test]
    fn test_spatial_query_performance() {
        let mut world = setup_world_with_creatures(1000);
        let start = Instant::now();
        
        let nearby = world.query_radius(Vec3::ZERO, 100.0);
        
        let elapsed = start.elapsed();
        assert!(elapsed < Duration::from_millis(1));
    }
}
```

#### 2. Debug Helpers
```rust
// Conditional compilation for debug features
#[cfg(debug_assertions)]
fn validate_state(&self) {
    assert!(self.health >= 0.0 && self.health <= 100.0);
    assert!(self.position.is_finite());
}

#[cfg(not(debug_assertions))]
fn validate_state(&self) {} // No-op in release

// Debug visualization
impl fmt::Debug for Creature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Creature")
            .field("id", &self.id)
            .field("pos", &format_args!("({:.1}, {:.1})", self.position.x, self.position.y))
            .field("health", &self.health)
            .finish()
    }
}
```

---

## Development Workflow

### Project Setup

#### Initial Setup
```bash
# Clone repository
git clone https://github.com/yourusername/creature-simulation.git
cd creature-simulation

# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install development tools
cargo install cargo-watch cargo-flamegraph

# Build and run
cargo run --release
```

#### IDE Configuration

**VS Code** (Recommended)
```json
// .vscode/settings.json
{
    "rust-analyzer.cargo.features": ["debug-ui"],
    "rust-analyzer.checkOnSave.command": "clippy",
    "editor.formatOnSave": true
}
```

Required extensions:
- rust-analyzer
- CodeLLDB (for debugging)
- Even Better TOML

### Git Workflow

#### Branch Strategy
```bash
main           # Stable, release-ready code
├── develop    # Integration branch
    ├── feature/creature-emotions
    ├── feature/world-generation
    └── fix/pathfinding-performance
```

#### Commit Convention
```bash
# Format: <type>(<scope>): <subject>

feat(creatures): add emotional state system
fix(pathfinding): resolve A* performance issue
docs(readme): update installation instructions
perf(simulation): optimize spatial queries
refactor(ai): extract decision making into module
test(creatures): add reproduction tests
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `perf`: Performance improvement
- `refactor`: Code change that neither fixes a bug nor adds a feature
- `test`: Adding missing tests
- `chore`: Changes to build process or auxiliary tools

#### Pull Request Process

1. Create feature branch from `develop`
2. Make changes following code style
3. Add/update tests
4. Run checks locally:
   ```bash
   cargo fmt
   cargo clippy
   cargo test
   cargo bench (if performance related)
   ```
5. Create PR with description:
   - What changed
   - Why it changed
   - How to test
   - Performance impact (if any)

### Development Commands

```bash
# Watch for changes and rebuild
cargo watch -x run

# Run with debug UI
cargo run --features debug-ui

# Run specific tests
cargo test creature_tests

# Check code style
cargo fmt -- --check

# Lint code
cargo clippy -- -D warnings

# Generate documentation
cargo doc --open

# Profile performance
cargo flamegraph
```

### Testing Strategy

#### Unit Tests
Located next to implementation:
```rust
// src/creature/mod.rs
impl Creature {
    pub fn take_damage(&mut self, amount: f32) {
        self.health = (self.health - amount).max(0.0);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn damage_cannot_go_negative() {
        let mut creature = Creature::new();
        creature.take_damage(200.0);
        assert_eq!(creature.health, 0.0);
    }
}
```

#### Integration Tests
Located in `tests/` directory:
```rust
// tests/simulation_test.rs
#[test]
fn creatures_reproduce_when_conditions_met() {
    let mut world = setup_test_world();
    world.spawn_creatures(2);
    
    // Run simulation
    for _ in 0..1000 {
        world.update(0.1);
    }
    
    assert!(world.creature_count() > 2);
}
```

#### Performance Tests
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_spatial_query(c: &mut Criterion) {
    let world = create_world_with_creatures(1000);
    
    c.bench_function("spatial_query_100m", |b| {
        b.iter(|| {
            black_box(world.query_radius(Vec3::ZERO, 100.0))
        })
    });
}

criterion_group!(benches, bench_spatial_query);
criterion_main!(benches);
```

### Debugging

#### Debug Prints
```rust
// Use debug! macro for development
log::debug!("Creature {} decided to {}", creature.id, decision);

// Conditional debug code
#[cfg(feature = "debug-ui")]
{
    draw_debug_info(&creature);
}
```

#### Visual Debugging
```rust
// Draw debug overlays
if self.debug_settings.show_paths {
    for creature in &creatures {
        draw_path(&creature.planned_path);
    }
}

if self.debug_settings.show_spatial_grid {
    draw_spatial_grid(&self.spatial_index);
}
```

### Performance Profiling

#### CPU Profiling
```bash
# Using perf (Linux)
cargo build --release
perf record --call-graph=dwarf ./target/release/game
perf report

# Using Instruments (macOS)
cargo instruments -t "Time Profiler"

# Using flamegraph
cargo flamegraph
```

#### Memory Profiling
```bash
# Using Valgrind (Linux)
valgrind --tool=massif ./target/release/game
ms_print massif.out.*

# Using heaptrack
heaptrack ./target/release/game
heaptrack_gui heaptrack.game.*
```

### Release Process

1. Update version in `Cargo.toml`
2. Update CHANGELOG.md
3. Run full test suite:
   ```bash
   cargo test --all-features
   cargo test --release
   ```
4. Build release binaries:
   ```bash
   cargo build --release
   ```
5. Tag release:
   ```bash
   git tag -a v1.0.0 -m "Release version 1.0.0"
   git push origin v1.0.0
   ```

---

## Quick Reference

### Common Patterns

```rust
// Option handling
let value = optional_value.unwrap_or_default();
let value = optional_value.unwrap_or(42);
if let Some(value) = optional_value { }

// Result handling  
let value = result?; // Propagate error
let value = result.unwrap_or_else(|e| {
    log::error!("Error: {}", e);
    default_value
});

// Iterator chains
let hungry_creatures: Vec<_> = creatures.iter()
    .filter(|c| c.hunger > 80.0)
    .map(|c| c.id)
    .collect();

// Pattern matching
match creature.state {
    State::Idle => creature.find_activity(),
    State::Moving { target } => creature.move_toward(target),
    State::Eating { food } => creature.consume(food),
}
```

### Performance Checklist

- [ ] Components under 64 bytes
- [ ] No allocations in hot loops
- [ ] Spatial queries cached
- [ ] Using parallel iterators where safe
- [ ] LOD system implemented
- [ ] Debug code compiled out in release
- [ ] String operations minimized
- [ ] Collections pre-allocated

### Code Review Checklist

- [ ] Follows naming conventions
- [ ] Has appropriate documentation
- [ ] Includes tests
- [ ] No compiler warnings
- [ ] Clippy passes
- [ ] Performance impact considered
- [ ] Error handling is robust
- [ ] No panics in library code

### Common Commands

```bash
# Format code
cargo fmt

# Lint
cargo clippy

# Test
cargo test
cargo test --release

# Benchmark
cargo bench

# Document
cargo doc --open

# Clean
cargo clean

# Update dependencies
cargo update
```