# Testing Guide

## Table of Contents
1. [Testing Strategy](#testing-strategy)
2. [Test-Driven Development](#test-driven-development)
3. [Testing Best Practices](#testing-best-practices)

---

## Testing Strategy

### Overview

Our testing strategy ensures the simulation remains stable, performant, and fun as it grows in complexity. We use a multi-layered approach covering unit tests, integration tests, performance benchmarks, and gameplay validation.

### Testing Pyramid

```
         /\
        /  \  Gameplay Tests (5%)
       /    \  - Player experience validation
      /      \  - Emergent behavior verification
     /--------\
    /          \ Integration Tests (20%)
   /            \ - System interactions
  /              \ - Save/load cycles
 /                \ - Multi-creature scenarios
/------------------\
/                    \ Unit Tests (75%)
/                      \ - Component logic
                        \ - Pure functions
                         \ - Edge cases
```

### Test Categories

#### 1. Unit Tests
Fast, isolated tests for individual components and functions.

```rust
#[cfg(test)]
mod creature_tests {
    use super::*;
    
    #[test]
    fn hunger_increases_over_time() {
        let mut creature = Creature::new(Species::Herbivore);
        let initial_hunger = creature.hunger;
        
        creature.update_needs(10.0); // 10 seconds
        
        assert!(creature.hunger > initial_hunger);
        assert!(creature.hunger <= 100.0); // Capped at max
    }
    
    #[test]
    fn creature_dies_when_health_reaches_zero() {
        let mut creature = Creature::new(Species::Carnivore);
        creature.health = 0.0;
        
        assert!(!creature.is_alive());
    }
    
    #[test]
    fn personality_affects_behavior_selection() {
        let mut bold_creature = Creature::new(Species::Omnivore);
        bold_creature.personality.courage = 1.0;
        
        let mut timid_creature = Creature::new(Species::Omnivore);
        timid_creature.personality.courage = 0.0;
        
        let threat = ThreatLevel::High;
        
        let bold_response = bold_creature.choose_threat_response(threat);
        let timid_response = timid_creature.choose_threat_response(threat);
        
        assert_eq!(bold_response, ThreatResponse::Fight);
        assert_eq!(timid_response, ThreatResponse::Flee);
    }
}
```

#### 2. Integration Tests
Tests that verify multiple systems work together correctly.

```rust
// tests/world_integration.rs
#[test]
fn creatures_find_food_when_hungry() {
    let mut app = App::new();
    app.add_plugins(TestPlugin);
    
    // Spawn hungry creature
    let creature_id = app.world.spawn(CreatureBundle {
        creature: Creature {
            hunger: 80.0,
            ..Default::default()
        },
        position: Position(Vec3::new(0.0, 0.0, 0.0)),
        ..Default::default()
    }).id();
    
    // Spawn food nearby
    app.world.spawn(FoodBundle {
        food: Food { nutrition: 50.0 },
        position: Position(Vec3::new(10.0, 0.0, 0.0)),
        ..Default::default()
    });
    
    // Run simulation for a while
    for _ in 0..100 {
        app.update();
    }
    
    // Verify creature moved toward food and ate
    let creature = app.world.get::<Creature>(creature_id).unwrap();
    assert!(creature.hunger < 80.0, "Creature should have eaten");
}

#[test]
fn social_creatures_form_groups() {
    let mut app = App::new();
    app.add_plugins(TestPlugin);
    
    // Spawn multiple social creatures
    let creature_ids: Vec<_> = (0..5)
        .map(|i| {
            app.world.spawn(CreatureBundle {
                creature: Creature {
                    personality: Personality {
                        sociability: 0.9,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                position: Position(Vec3::new(i as f32 * 5.0, 0.0, 0.0)),
                ..Default::default()
            }).id()
        })
        .collect();
    
    // Run simulation
    for _ in 0..1000 {
        app.update();
    }
    
    // Check if creatures grouped together
    let positions: Vec<Vec3> = creature_ids.iter()
        .map(|&id| app.world.get::<Position>(id).unwrap().0)
        .collect();
    
    let center = positions.iter().sum::<Vec3>() / positions.len() as f32;
    let avg_distance = positions.iter()
        .map(|pos| pos.distance(center))
        .sum::<f32>() / positions.len() as f32;
    
    assert!(avg_distance < 20.0, "Social creatures should cluster together");
}
```

#### 3. Performance Tests
Benchmarks to ensure the simulation maintains target performance.

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_spatial_queries(c: &mut Criterion) {
    let mut group = c.benchmark_group("spatial_queries");
    
    for creature_count in [100, 500, 1000, 5000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(creature_count),
            creature_count,
            |b, &count| {
                let world = create_world_with_creatures(count);
                let query_pos = Vec3::new(500.0, 0.0, 500.0);
                
                b.iter(|| {
                    black_box(world.query_creatures_in_radius(query_pos, 100.0))
                });
            },
        );
    }
    
    group.finish();
}

fn bench_pathfinding(c: &mut Criterion) {
    let world = create_test_world();
    
    c.bench_function("pathfinding_medium_distance", |b| {
        let start = Vec3::new(0.0, 0.0, 0.0);
        let end = Vec3::new(100.0, 0.0, 100.0);
        
        b.iter(|| {
            black_box(world.find_path(start, end))
        });
    });
}

criterion_group!(benches, bench_spatial_queries, bench_pathfinding);
criterion_main!(benches);
```

#### 4. Property-Based Tests
Tests that verify invariants hold for random inputs.

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn genetics_remain_valid_after_mutation(
        gene_value in 0.0f32..1.0,
        mutation_strength in 0.0f32..0.5,
    ) {
        let mut gene = Gene { value: gene_value };
        gene.mutate(mutation_strength);
        
        prop_assert!(gene.value >= 0.0);
        prop_assert!(gene.value <= 1.0);
    }
    
    #[test]
    fn creature_needs_stay_bounded(
        initial_hunger in 0.0f32..100.0,
        time_delta in 0.0f32..3600.0,
    ) {
        let mut creature = Creature {
            hunger: initial_hunger,
            ..Default::default()
        };
        
        creature.update_needs(time_delta);
        
        prop_assert!(creature.hunger >= 0.0);
        prop_assert!(creature.hunger <= 100.0);
    }
}
```

### Testing Infrastructure

#### Test Utilities
```rust
// tests/common/mod.rs
pub struct TestWorld {
    app: App,
}

impl TestWorld {
    pub fn new() -> Self {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
           .add_plugin(TestSimulationPlugin);
        
        Self { app }
    }
    
    pub fn spawn_creature_at(&mut self, position: Vec3) -> Entity {
        self.app.world.spawn(CreatureBundle {
            position: Position(position),
            ..Default::default()
        }).id()
    }
    
    pub fn tick(&mut self, times: usize) {
        for _ in 0..times {
            self.app.update();
        }
    }
    
    pub fn get_creature(&self, entity: Entity) -> Option<&Creature> {
        self.app.world.get::<Creature>(entity)
    }
}

// Test data builders
pub struct CreatureBuilder {
    creature: Creature,
}

impl CreatureBuilder {
    pub fn new() -> Self {
        Self {
            creature: Creature::default(),
        }
    }
    
    pub fn with_hunger(mut self, hunger: f32) -> Self {
        self.creature.hunger = hunger;
        self
    }
    
    pub fn with_personality(mut self, personality: Personality) -> Self {
        self.creature.personality = personality;
        self
    }
    
    pub fn build(self) -> Creature {
        self.creature
    }
}
```

---

## Test-Driven Development

### TDD Workflow

1. **Write a failing test** for the desired behavior
2. **Implement** the minimum code to pass
3. **Refactor** while keeping tests green
4. **Repeat**

### Example: Implementing Creature Bonding

#### Step 1: Write Failing Test
```rust
#[test]
fn creatures_form_bonds_through_positive_interactions() {
    let mut creature1 = Creature::new(Species::Herbivore);
    let mut creature2 = Creature::new(Species::Herbivore);
    
    // No bond initially
    assert_eq!(creature1.get_relationship(&creature2.id), None);
    
    // Positive interaction
    creature1.interact_with(&mut creature2, InteractionType::Grooming);
    
    // Bond should form
    let relationship = creature1.get_relationship(&creature2.id).unwrap();
    assert!(relationship.bond_strength > 0.0);
}
```

#### Step 2: Implement Minimum Code
```rust
impl Creature {
    pub fn interact_with(&mut self, other: &mut Creature, interaction: InteractionType) {
        match interaction {
            InteractionType::Grooming => {
                // Create or strengthen bond
                let relationship = self.relationships
                    .entry(other.id)
                    .or_insert(Relationship::default());
                
                relationship.bond_strength += 0.1;
            }
            _ => {}
        }
    }
    
    pub fn get_relationship(&self, other_id: &EntityId) -> Option<&Relationship> {
        self.relationships.get(other_id)
    }
}
```

#### Step 3: Refactor
```rust
// Extract constants
const GROOMING_BOND_INCREASE: f32 = 0.1;
const MAX_BOND_STRENGTH: f32 = 1.0;

impl Creature {
    pub fn interact_with(&mut self, other: &mut Creature, interaction: InteractionType) {
        let bond_change = match interaction {
            InteractionType::Grooming => GROOMING_BOND_INCREASE,
            InteractionType::Feeding => FEEDING_BOND_INCREASE,
            InteractionType::Playing => PLAYING_BOND_INCREASE,
            InteractionType::Fighting => -FIGHTING_BOND_DECREASE,
            _ => 0.0,
        };
        
        self.update_relationship(other.id, bond_change);
    }
    
    fn update_relationship(&mut self, other_id: EntityId, bond_change: f32) {
        let relationship = self.relationships
            .entry(other_id)
            .or_insert(Relationship::default());
        
        relationship.bond_strength = (relationship.bond_strength + bond_change)
            .clamp(0.0, MAX_BOND_STRENGTH);
        
        relationship.last_interaction = current_time();
    }
}
```

#### Step 4: Add More Tests
```rust
#[test]
fn bond_strength_is_capped() {
    let mut creature1 = Creature::new(Species::Herbivore);
    let mut creature2 = Creature::new(Species::Herbivore);
    
    // Many positive interactions
    for _ in 0..20 {
        creature1.interact_with(&mut creature2, InteractionType::Grooming);
    }
    
    let relationship = creature1.get_relationship(&creature2.id).unwrap();
    assert_eq!(relationship.bond_strength, MAX_BOND_STRENGTH);
}

#[test]
fn fighting_reduces_bond_strength() {
    let mut creature1 = Creature::new(Species::Carnivore);
    let mut creature2 = Creature::new(Species::Carnivore);
    
    // Form initial bond
    creature1.interact_with(&mut creature2, InteractionType::Playing);
    let initial_bond = creature1.get_relationship(&creature2.id)
        .unwrap()
        .bond_strength;
    
    // Fight
    creature1.interact_with(&mut creature2, InteractionType::Fighting);
    
    let final_bond = creature1.get_relationship(&creature2.id)
        .unwrap()
        .bond_strength;
    
    assert!(final_bond < initial_bond);
}
```

---

## Testing Best Practices

### Writing Good Tests

#### 1. Test Names Should Describe Behavior
```rust
// Good
#[test]
fn creature_seeks_water_when_thirst_exceeds_threshold() { }

// Bad
#[test]
fn test_thirst() { }
```

#### 2. Arrange-Act-Assert Pattern
```rust
#[test]
fn hungry_creature_prioritizes_food_over_social_interaction() {
    // Arrange
    let mut creature = CreatureBuilder::new()
        .with_hunger(90.0)
        .with_social_need(80.0)
        .build();
    
    let food_source = FoodSource::new(Vec3::new(10.0, 0.0, 0.0));
    let friend = Creature::new(Species::Herbivore);
    
    // Act
    let decision = creature.make_decision(&[
        Option::Eat(food_source),
        Option::Socialize(friend),
    ]);
    
    // Assert
    assert_matches!(decision, Decision::Eat(_));
}
```

#### 3. Test One Thing at a Time
```rust
// Good - focused tests
#[test]
fn creature_speed_affected_by_injury() {
    let mut creature = Creature::new(Species::Herbivore);
    let base_speed = creature.calculate_speed();
    
    creature.apply_injury(Injury::LegWound);
    
    assert!(creature.calculate_speed() < base_speed);
}

#[test]
fn creature_speed_affected_by_age() {
    let mut young = Creature::new(Species::Herbivore);
    let mut old = Creature::new(Species::Herbivore);
    old.age = old.species.lifespan() * 0.9;
    
    assert!(young.calculate_speed() > old.calculate_speed());
}

// Bad - testing multiple things
#[test]
fn creature_movement() {
    // Tests speed, injuries, age, terrain, weather all in one...
}
```

### Test Data Management

#### Test Fixtures
```rust
mod fixtures {
    pub fn basic_world() -> World {
        World {
            size: IVec2::new(100, 100),
            biomes: vec![Biome::Forest; 10000],
            ..Default::default()
        }
    }
    
    pub fn creature_family() -> (Creature, Creature, Vec<Creature>) {
        let parent1 = Creature::new(Species::Herbivore);
        let parent2 = Creature::new(Species::Herbivore);
        let children = vec![
            Creature::with_parents(&parent1, &parent2),
            Creature::with_parents(&parent1, &parent2),
        ];
        
        (parent1, parent2, children)
    }
}
```

#### Parameterized Tests
```rust
#[test_case(Species::Herbivore, 8.0)]
#[test_case(Species::Carnivore, 12.0)]
#[test_case(Species::Omnivore, 10.0)]
fn species_have_correct_base_speed(species: Species, expected_speed: f32) {
    let creature = Creature::new(species);
    assert_eq!(creature.base_speed(), expected_speed);
}
```

### Performance Test Guidelines

#### 1. Establish Baselines
```rust
// Track performance over time
#[bench]
fn bench_world_update_baseline(b: &mut Bencher) {
    let world = create_world_with_creatures(1000);
    
    b.iter(|| {
        world.update(0.016); // One frame at 60 FPS
    });
}
```

#### 2. Test Scaling
```rust
#[bench]
fn bench_creature_scaling(b: &mut Bencher) {
    let counts = vec![100, 500, 1000, 2000, 5000];
    
    for count in counts {
        b.iter_with_setup(
            || create_world_with_creatures(count),
            |world| world.update(0.016),
        );
    }
}
```

### Debugging Test Failures

#### 1. Add Debug Output
```rust
#[test]
fn complex_behavior_test() {
    let creature = setup_creature();
    
    // Add debug print before assertion
    dbg!(&creature.current_state);
    dbg!(&creature.decision_factors);
    
    assert_eq!(creature.current_behavior, Behavior::ExpectedBehavior);
}
```

#### 2. Use Snapshot Testing
```rust
#[test]
fn creature_state_snapshot() {
    let creature = create_complex_creature();
    
    insta::assert_yaml_snapshot!(creature);
}
```

#### 3. Visual Test Debugging
```rust
#[test]
#[ignore] // Run with --ignored flag when needed
fn visual_pathfinding_test() {
    let world = create_test_world();
    let path = world.find_path(start, end);
    
    // Output path visualization
    visualize_path(&world, &path, "test_output/pathfinding.png");
}
```

### Continuous Integration

```yaml
# .github/workflows/test.yml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
      - name: Run tests
        run: cargo test --all-features
      - name: Run benchmarks
        run: cargo bench --no-run
      - name: Check test coverage
        run: cargo tarpaulin --out Xml
      - name: Upload coverage
        uses: codecov/codecov-action@v1
```