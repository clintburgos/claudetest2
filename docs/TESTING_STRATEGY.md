# Testing Strategy

## Overview
This document outlines the test-driven development approach and testing strategies for the creature simulation project.

## Testing Philosophy

### Core Principles
1. **Test First**: Write tests before implementation
2. **Fast Feedback**: Unit tests must run in milliseconds
3. **Isolated**: Tests should not depend on each other
4. **Descriptive**: Test names describe behavior, not implementation
5. **Comprehensive**: Test edge cases and error conditions

### Red-Green-Refactor Cycle
1. **Red**: Write a failing test for desired behavior
2. **Green**: Write minimal code to pass the test
3. **Refactor**: Improve code while keeping tests green

## Test Categories

### 1. Unit Tests
Fast, isolated tests for individual components.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creature_hunger_increases_over_time() {
        let mut creature = Creature::new();
        let initial_hunger = creature.needs.hunger;
        
        creature.update(1.0); // 1 second
        
        assert!(creature.needs.hunger > initial_hunger);
    }

    #[test]
    fn creature_dies_when_hunger_reaches_zero() {
        let mut creature = Creature::new();
        creature.needs.hunger = 0.0;
        
        creature.update(0.1);
        
        assert!(!creature.is_alive());
    }
}
```

### 2. Property-Based Tests
For systems with complex invariants, especially genetics.

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn genetic_inheritance_preserves_gene_count(
        parent1 in arbitrary_dna(),
        parent2 in arbitrary_dna()
    ) {
        let child = reproduce(&parent1, &parent2);
        
        prop_assert_eq!(
            child.gene_count(),
            parent1.gene_count()
        );
    }

    #[test]
    fn mutations_stay_within_valid_range(
        gene_value in 0.0f32..=1.0,
        mutation_rate in 0.0f32..=0.1
    ) {
        let mutated = mutate_gene(gene_value, mutation_rate);
        
        prop_assert!(mutated >= 0.0 && mutated <= 1.0);
    }
}
```

### 3. Integration Tests
Test interactions between systems.

```rust
#[test]
fn creatures_can_find_and_consume_food() {
    let mut world = World::new(10, 10);
    let creature_id = world.spawn_creature(Position(5, 5));
    world.spawn_food(Position(6, 5));
    
    // Run simulation until creature finds food
    for _ in 0..10 {
        world.update(0.1);
    }
    
    let creature = world.get_creature(creature_id);
    assert!(creature.needs.hunger < 100.0);
    assert_eq!(world.count_food(), 0);
}
```

### 4. Behavior Tests
Verify emergent behaviors.

```rust
#[test]
fn social_creatures_form_groups() {
    let mut world = TestWorld::with_social_creatures(20);
    
    // Run for "1 day" at high speed
    world.run_for_duration(Duration::days(1));
    
    let groups = world.identify_groups();
    assert!(groups.len() < 10); // Should form groups
    assert!(groups.iter().any(|g| g.size() > 2));
}
```

### 5. Performance Benchmarks
Ensure performance requirements are met.

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_creature_decisions(c: &mut Criterion) {
    let creature = create_test_creature();
    let world_state = create_test_world_state();
    
    c.bench_function("creature decision making", |b| {
        b.iter(|| {
            creature.make_decision(black_box(&world_state))
        })
    });
}

fn bench_thousand_creatures(c: &mut Criterion) {
    let mut world = World::with_creatures(1000);
    
    c.bench_function("update 1000 creatures", |b| {
        b.iter(|| world.update(black_box(0.016))) // 60 FPS
    });
}
```

## Testing Tools

### Required Dependencies
```toml
[dev-dependencies]
# Property testing
proptest = "1.4"
proptest-derive = "0.4"

# Benchmarking
criterion = { version = "0.5", features = ["html_reports"] }

# Test utilities
rstest = "0.18"  # Parameterized tests
fake = "2.9"     # Generate test data
approx = "0.5"   # Float comparisons

# Mocking (if needed)
mockall = "0.12"
```

### Test Organization
```
tests/
├── unit/
│   ├── creatures/
│   ├── genetics/
│   ├── world/
│   └── social/
├── integration/
│   ├── reproduction_test.rs
│   ├── ecosystem_test.rs
│   └── time_scaling_test.rs
├── property/
│   ├── genetics_props.rs
│   └── world_props.rs
└── benchmarks/
    ├── creature_bench.rs
    └── world_bench.rs
```

## Test Patterns for Key Systems

### ECS Testing
```rust
#[test]
fn ecs_component_storage_and_retrieval() {
    let mut world = World::new();
    let entity = world.create_entity();
    
    world.add_component(entity, Position { x: 5.0, y: 10.0 });
    world.add_component(entity, Health { current: 100.0, max: 100.0 });
    
    assert_eq!(
        world.get_component::<Position>(entity).unwrap().x,
        5.0
    );
}

#[test]
fn ecs_system_processes_matching_entities() {
    let mut world = World::new();
    let mut movement_system = MovementSystem::new();
    
    // Create entities with different component combinations
    let mover = world.create_entity();
    world.add_component(mover, Position::new(0.0, 0.0));
    world.add_component(mover, Velocity::new(1.0, 0.0));
    
    let static_entity = world.create_entity();
    world.add_component(static_entity, Position::new(10.0, 10.0));
    // No velocity component
    
    movement_system.update(&mut world, 1.0);
    
    // Only the mover should have moved
    assert_eq!(world.get_component::<Position>(mover).unwrap().x, 1.0);
    assert_eq!(world.get_component::<Position>(static_entity).unwrap().x, 10.0);
}
```

### Genetics Testing
```rust
#[test]
fn gene_expression_follows_dominance_rules() {
    let dominant_gene = Gene { value: 1.0, dominance: 0.9 };
    let recessive_gene = Gene { value: 0.0, dominance: 0.1 };
    
    let expressed = express_genes(&dominant_gene, &recessive_gene);
    
    // Should strongly favor dominant gene
    assert!(expressed > 0.8);
}

#[rstest]
#[case(0.0, 0.001, 0.0, 0.01)]  // Min mutation
#[case(0.5, 0.01, 0.4, 0.6)]     // Mid-range
#[case(1.0, 0.001, 0.99, 1.0)]   // Max value
fn gene_mutation_within_bounds(
    #[case] original: f32,
    #[case] rate: f32,
    #[case] min_expected: f32,
    #[case] max_expected: f32,
) {
    let mutated = mutate_gene_value(original, rate);
    assert!(mutated >= min_expected && mutated <= max_expected);
}
```

### Social System Testing
```rust
#[test]
fn trust_decreases_when_lie_detected() {
    let mut receiver = TestCreature::new();
    let sender_id = CreatureId(42);
    
    // Establish initial trust
    receiver.social_memory.set_trust(sender_id, 0.8);
    
    // Receive a lie that gets detected
    let lie = Message {
        sender: sender_id,
        concept: Concept::Location(Resource::Food, Direction::North),
        honesty: 0.0,  // Complete lie
        urgency: 0.5,
    };
    
    receiver.receive_message(lie);
    receiver.investigate_claim(); // Finds no food north
    
    assert!(receiver.social_memory.get_trust(sender_id) < 0.8);
}
```

### Time Scaling Testing
```rust
#[test]
fn time_scaling_maintains_consistency() {
    let mut world = World::new();
    let mut time_controller = TimeController::new();
    
    // Add some creatures
    for _ in 0..10 {
        world.spawn_creature_random();
    }
    
    // Record state at normal speed
    time_controller.set_scale(1.0);
    world.update(1.0);
    let gen1_births = world.get_birth_count();
    
    // Reset and run at high speed
    world.reset();
    time_controller.set_scale(1000.0);
    world.update(0.001); // Same logical time
    let gen2_births = world.get_birth_count();
    
    // Results should be statistically similar
    assert!((gen1_births as f32 - gen2_births as f32).abs() < 2.0);
}
```

## Test Data Builders

### Creature Builder
```rust
#[cfg(test)]
pub struct CreatureBuilder {
    needs: Needs,
    genes: Vec<Gene>,
    position: Position,
}

impl CreatureBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn hungry(mut self) -> Self {
        self.needs.hunger = 20.0;
        self
    }
    
    pub fn with_gene(mut self, gene_type: GeneType, value: f32) -> Self {
        self.genes.push(Gene {
            gene_type,
            value: value.clamp(0.0, 1.0),
            dominance: 0.5,
        });
        self
    }
    
    pub fn at_position(mut self, x: f32, y: f32) -> Self {
        self.position = Position { x, y };
        self
    }
    
    pub fn build(self) -> Creature {
        Creature {
            needs: self.needs,
            dna: DNA::from_genes(self.genes),
            position: self.position,
            ..Default::default()
        }
    }
}
```

## Testing Emergent Behaviors

### Statistical Tests
```rust
#[test]
fn population_exhibits_natural_selection() {
    let mut world = World::new();
    
    // Create initial population with varied metabolism genes
    for _ in 0..100 {
        world.spawn_creature_with_random_genes();
    }
    
    // Run for many generations
    let generations = 50;
    for _ in 0..generations {
        world.run_generation();
    }
    
    // Analyze population genetics
    let final_metabolism_stats = world.calculate_gene_statistics(GeneType::Metabolism);
    
    // In food-scarce environment, expect lower metabolism to be selected
    assert!(final_metabolism_stats.mean < 0.5);
    assert!(final_metabolism_stats.std_dev < 0.2); // Population converging
}
```

## Continuous Integration Tests

### GitHub Actions Workflow
```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - uses: dtolnay/rust-toolchain@stable
    
    - name: Run tests
      run: cargo test --all-features
      
    - name: Run benchmarks
      run: cargo bench --no-run
      
    - name: Check test coverage
      run: |
        cargo install cargo-tarpaulin
        cargo tarpaulin --out Xml
        
    - name: Property tests (extended)
      run: cargo test --release -- --proptest-cases 1000
```

## Test-Driven Implementation Guide

### Example: Implementing Creature Hunger

1. **Write the test first**:
```rust
#[test]
fn creature_gets_hungry_over_time() {
    let mut creature = Creature::new();
    assert_eq!(creature.hunger(), 50.0); // Start at mid-hunger
    
    creature.update(10.0); // 10 seconds pass
    
    assert!(creature.hunger() > 50.0);
    assert!(creature.hunger() <= 100.0);
}
```

2. **Run test (RED)**: Test fails - Creature doesn't exist yet

3. **Implement minimum to pass**:
```rust
pub struct Creature {
    hunger: f32,
}

impl Creature {
    pub fn new() -> Self {
        Self { hunger: 50.0 }
    }
    
    pub fn hunger(&self) -> f32 {
        self.hunger
    }
    
    pub fn update(&mut self, dt: f32) {
        self.hunger += dt * 0.1; // Increase by 1 per 10 seconds
        self.hunger = self.hunger.min(100.0);
    }
}
```

4. **Run test (GREEN)**: Test passes!

5. **Refactor**: Extract constants, improve names
```rust
const INITIAL_HUNGER: f32 = 50.0;
const HUNGER_RATE: f32 = 0.1;
const MAX_HUNGER: f32 = 100.0;

impl Creature {
    pub fn update(&mut self, dt: f32) {
        self.hunger = (self.hunger + dt * HUNGER_RATE).min(MAX_HUNGER);
    }
}
```

6. **Add edge case tests**:
```rust
#[test]
fn hunger_cannot_exceed_maximum() {
    let mut creature = Creature::new();
    creature.hunger = 99.0;
    
    creature.update(100.0); // Long time
    
    assert_eq!(creature.hunger(), 100.0);
}
```

---
*Last Updated: 2024-01-XX*
