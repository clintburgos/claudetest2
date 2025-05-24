# Comprehensive Testing Strategy

## Overview

This document outlines the testing strategy for the creature simulation project, covering unit tests, integration tests, performance benchmarks, and simulation validation.

## Testing Philosophy

- **Test-Driven Development (TDD)** for core systems
- **Property-based testing** for emergent behaviors
- **Performance regression testing** for maintaining 60+ FPS with 5000 creatures
- **Simulation validation** for biological plausibility

## Test Categories

### 1. Unit Tests

#### Component Tests

```rust
#[cfg(test)]
mod creature_component_tests {
    use super::*;

    #[test]
    fn test_health_component_damage() {
        let mut health = Health::new(100.0);
        health.apply_damage(30.0);
        
        assert_eq!(health.current, 70.0);
        assert_eq!(health.max, 100.0);
        assert!(!health.is_dead());
    }
    
    #[test]
    fn test_needs_update() {
        let mut needs = Needs::default();
        let initial_hunger = needs.hunger;
        
        needs.update(1.0); // 1 second
        
        assert!(needs.hunger > initial_hunger);
        assert_eq!(needs.get_most_pressing(), NeedType::Food);
    }
    
    #[test]
    fn test_genetics_crossover() {
        let parent1 = Genetics::random();
        let parent2 = Genetics::random();
        
        let offspring = Genetics::crossover(&parent1, &parent2);
        
        // Verify offspring has traits from both parents
        for (trait_name, trait_value) in &offspring.traits {
            let p1_value = parent1.traits.get(trait_name).unwrap();
            let p2_value = parent2.traits.get(trait_name).unwrap();
            
            // Trait should be between parent values (with small mutation chance)
            assert!(trait_value >= &(p1_value.min(p2_value) - 0.1));
            assert!(trait_value <= &(p1_value.max(p2_value) + 0.1));
        }
    }
}
```

#### System Logic Tests

```rust
#[cfg(test)]
mod decision_system_tests {
    use super::*;

    #[test]
    fn test_decision_priority() {
        let mut decision_system = DecisionSystem::new();
        let creature = create_test_creature();
        
        // Set critical hunger
        creature.needs.hunger = 0.9;
        creature.needs.thirst = 0.3;
        
        let decision = decision_system.evaluate(&creature, &test_world());
        
        assert_eq!(decision.action_type, ActionType::SeekFood);
        assert!(decision.priority > 0.8);
    }
    
    #[test]
    fn test_decision_interruption() {
        let mut decision_system = DecisionSystem::new();
        let mut creature = create_test_creature();
        
        // Start with food seeking
        creature.current_action = Some(Action::SeekFood { target: None });
        
        // Introduce predator threat
        let threat = ThreatInfo::Predator { 
            position: Vec3::new(10.0, 0.0, 10.0),
            threat_level: 0.9 
        };
        
        let decision = decision_system.evaluate_with_threat(&creature, threat);
        
        assert_eq!(decision.action_type, ActionType::Flee);
        assert!(decision.interrupts_current);
    }
}
```

### 2. Integration Tests

#### System Interaction Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_movement_spatial_index_integration() {
        let mut world = World::new();
        let mut spatial_index = SpatialIndex::new(20.0);
        
        // Spawn creature
        let creature = world.spawn()
            .insert(Position(Vec3::new(0.0, 0.0, 0.0)))
            .insert(Velocity(Vec3::new(1.0, 0.0, 0.0)))
            .id();
            
        // Update position via movement system
        movement_system.update(&mut world, 1.0);
        
        // Verify spatial index updated
        let nearby = spatial_index.query_range(Vec3::new(1.0, 0.0, 0.0), 5.0);
        assert!(nearby.contains(&creature));
    }
    
    #[test]
    fn test_conversation_relationship_integration() {
        let mut world = create_test_world();
        
        let creature_a = spawn_creature(&mut world, Vec3::ZERO);
        let creature_b = spawn_creature(&mut world, Vec3::new(2.0, 0.0, 0.0));
        
        // Start conversation
        let conversation = Conversation::new(vec![creature_a, creature_b], Topic::FoodLocation);
        conversation_system.register(conversation);
        
        // Run conversation to completion
        for _ in 0..10 {
            conversation_system.update(&mut world, 0.1);
        }
        
        // Check relationship improved
        let social_a = world.get::<SocialComponent>(creature_a).unwrap();
        let relationship = social_a.get_relationship(creature_b).unwrap();
        
        assert!(relationship.strength > 0.0);
        assert!(relationship.interaction_count > 0);
    }
}
```

#### Event Flow Tests

```rust
#[test]
fn test_death_event_cascade() {
    let mut world = World::new();
    let mut event_bus = EventBus::new();
    
    // Setup creature with group membership
    let creature = spawn_creature(&mut world, Vec3::ZERO);
    let group = create_group(vec![creature, /* others */]);
    
    // Trigger death
    event_bus.send(CreatureEvent::Died {
        entity: creature,
        cause: DeathCause::Starvation,
        position: Vec3::ZERO,
    });
    
    // Process events
    event_bus.process_events(&mut world);
    
    // Verify cascading effects
    assert!(!world.get::<Creature>(creature).is_some()); // Entity removed
    assert!(!group.members.contains(&creature)); // Removed from group
    
    // Check for follow-up events
    let events = event_bus.drain_processed();
    assert!(events.iter().any(|e| matches!(e, GroupEvent::MemberLost { .. })));
    assert!(events.iter().any(|e| matches!(e, ResourceEvent::Dropped { .. })));
}
```

### 3. Performance Tests

#### Benchmark Tests

```rust
#[bench]
fn bench_spatial_query_1000_creatures(b: &mut Bencher) {
    let mut spatial_index = create_populated_spatial_index(1000);
    
    b.iter(|| {
        let query = RangeQuery {
            center: Vec3::new(500.0, 0.0, 500.0),
            radius: 50.0,
            filter: None,
            max_results: None,
        };
        
        test::black_box(spatial_index.query_range(&query));
    });
}

#[bench]
fn bench_decision_system_5000_creatures(b: &mut Bencher) {
    let world = create_world_with_creatures(5000);
    let mut decision_system = DecisionSystem::new();
    
    b.iter(|| {
        for (entity, creature) in world.query::<&Creature>().iter() {
            test::black_box(decision_system.evaluate(creature, &world));
        }
    });
}

#[bench]
fn bench_pathfinding_complex_terrain(b: &mut Bencher) {
    let terrain = create_complex_terrain();
    let mut pathfinder = Pathfinder::new();
    
    b.iter(|| {
        let path = pathfinder.find_path(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(100.0, 0.0, 100.0),
            &terrain
        );
        test::black_box(path);
    });
}
```

#### Memory Usage Tests

```rust
#[test]
fn test_memory_usage_5000_creatures() {
    let initial_memory = get_current_memory_usage();
    
    let world = create_world_with_creatures(5000);
    
    let memory_after_spawn = get_current_memory_usage();
    let memory_per_creature = (memory_after_spawn - initial_memory) / 5000;
    
    // Verify we're within budget (200KB per creature)
    assert!(memory_per_creature <= 200 * 1024, 
        "Memory per creature: {} bytes", memory_per_creature);
}

#[test]
fn test_cache_memory_limits() {
    let mut cache_manager = CacheManager::new(100 * 1024 * 1024); // 100MB limit
    
    // Fill caches
    for i in 0..10000 {
        cache_manager.pathfinding_cache.insert(
            PathKey::new(i),
            generate_random_path()
        );
    }
    
    // Verify memory limit respected
    assert!(cache_manager.total_memory_usage() <= 100 * 1024 * 1024);
    
    // Verify eviction working
    assert!(cache_manager.pathfinding_cache.len() < 10000);
}
```

### 4. Simulation Tests

#### Emergent Behavior Tests

```rust
#[test]
fn test_group_formation_emerges() {
    let mut simulation = Simulation::new(SimulationConfig {
        creature_count: 100,
        world_size: 1000.0,
        enable_groups: true,
        ..Default::default()
    });
    
    // Run for simulated hour
    simulation.run_for(Duration::from_secs(3600));
    
    let groups = simulation.get_groups();
    
    // Verify groups formed
    assert!(!groups.is_empty(), "No groups formed");
    
    // Verify group properties
    for group in groups {
        assert!(group.members.len() >= 3, "Group too small");
        assert!(group.cohesion > 0.3, "Group cohesion too low");
    }
}

#[test]
fn test_population_dynamics() {
    let mut simulation = Simulation::new(SimulationConfig {
        creature_count: 50,
        resource_abundance: 1.5, // Plenty of resources
        ..Default::default()
    });
    
    let initial_population = simulation.creature_count();
    
    // Run for simulated day
    simulation.run_for(Duration::from_secs(86400));
    
    let final_population = simulation.creature_count();
    
    // Population should grow with abundant resources
    assert!(final_population > initial_population, 
        "Population didn't grow: {} -> {}", initial_population, final_population);
        
    // But not explode unrealistically
    assert!(final_population < initial_population * 2,
        "Population grew too fast: {} -> {}", initial_population, final_population);
}
```

#### Property-Based Tests

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_genetics_bounds(
        traits in prop::collection::hash_map(
            any::<String>(),
            0.0f32..=1.0f32,
            1..20
        )
    ) {
        let genetics = Genetics { traits };
        
        // All traits should remain bounded after mutation
        let mutated = genetics.mutate(0.1);
        
        for (_, value) in &mutated.traits {
            assert!(*value >= 0.0);
            assert!(*value <= 1.0);
        }
    }
    
    #[test]
    fn test_resource_conservation(
        initial_resources in 100u32..10000u32,
        creature_count in 10usize..100usize,
        consumption_rate in 0.1f32..2.0f32,
    ) {
        let mut world = create_world_with_resources(initial_resources);
        spawn_creatures(&mut world, creature_count);
        
        // Run consumption
        resource_system.update(&mut world, 1.0);
        
        let remaining = count_total_resources(&world);
        let consumed = initial_resources - remaining;
        
        // Consumed should not exceed possible consumption
        let max_consumption = creature_count as f32 * consumption_rate;
        assert!(consumed as f32 <= max_consumption);
    }
}
```

### 5. Validation Tests

#### Biological Plausibility Tests

```rust
#[test]
fn test_realistic_lifespans() {
    let mut simulation = Simulation::new_deterministic(42);
    
    // Track creature lifespans
    let lifespans = simulation.run_and_track_lifespans(Duration::from_secs(86400 * 30));
    
    let avg_lifespan = lifespans.iter().sum::<f32>() / lifespans.len() as f32;
    
    // Verify realistic lifespan distribution
    assert!(avg_lifespan > 86400.0 * 5.0, "Average lifespan too short");
    assert!(avg_lifespan < 86400.0 * 60.0, "Average lifespan too long");
    
    // Verify some die young, some die old
    let young_deaths = lifespans.iter().filter(|&&l| l < 86400.0 * 2.0).count();
    let old_deaths = lifespans.iter().filter(|&&l| l > 86400.0 * 30.0).count();
    
    assert!(young_deaths > 0, "No young deaths");
    assert!(old_deaths > 0, "No old deaths");
}

#[test] 
fn test_energy_conservation() {
    let mut world = create_controlled_environment();
    
    let total_energy_before = calculate_total_system_energy(&world);
    
    // Run for extended period
    for _ in 0..1000 {
        update_all_systems(&mut world, 0.1);
    }
    
    let total_energy_after = calculate_total_system_energy(&world);
    
    // Allow for some loss due to metabolism, but not massive changes
    let energy_ratio = total_energy_after / total_energy_before;
    assert!(energy_ratio > 0.7, "Too much energy lost");
    assert!(energy_ratio < 1.3, "Energy created from nothing");
}
```

### 6. Stress Tests

```rust
#[test]
#[ignore] // Run with --ignored for stress tests
fn stress_test_10000_creatures() {
    let mut world = World::new();
    
    // Spawn 10,000 creatures
    for i in 0..10000 {
        spawn_creature(&mut world, random_position());
    }
    
    // Run for 1000 frames
    let start = Instant::now();
    for _ in 0..1000 {
        update_all_systems(&mut world, 0.016); // 60 FPS
    }
    let duration = start.elapsed();
    
    let avg_frame_time = duration.as_secs_f32() / 1000.0;
    let fps = 1.0 / avg_frame_time;
    
    println!("10,000 creatures: {:.1} FPS", fps);
    assert!(fps >= 30.0, "Performance below minimum with 10k creatures");
}

#[test]
fn test_memory_pressure_handling() {
    let mut cache_manager = CacheManager::new(50 * 1024 * 1024); // 50MB
    
    // Fill caches beyond limit
    for i in 0..100000 {
        cache_manager.insert_large_item(i, generate_large_data());
        
        // Should handle pressure gracefully
        assert!(cache_manager.total_memory_usage() <= 60 * 1024 * 1024); // Some overhead allowed
    }
    
    // Should still function after pressure
    let test_key = 99999;
    cache_manager.insert_small_item(test_key, 42);
    assert_eq!(cache_manager.get(&test_key), Some(&42));
}
```

## Test Infrastructure

### Test Utilities

```rust
pub mod test_utils {
    pub fn create_test_world() -> World {
        let mut world = World::new();
        world.insert_resource(Time::default());
        world.insert_resource(SpatialIndex::new(20.0));
        world
    }
    
    pub fn spawn_test_creature(world: &mut World, pos: Vec3) -> Entity {
        world.spawn()
            .insert_bundle(CreatureBundle::default())
            .insert(Position(pos))
            .id()
    }
    
    pub fn create_deterministic_rng(seed: u64) -> StdRng {
        StdRng::seed_from_u64(seed)
    }
}
```

### Continuous Integration

```yaml
# .github/workflows/test.yml
name: Test Suite

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      
      - name: Run unit tests
        run: cargo test --lib
        
      - name: Run integration tests  
        run: cargo test --test '*'
        
      - name: Run benchmarks
        run: cargo bench --no-run
        
      - name: Check performance regression
        run: cargo bench -- --save-baseline new
        
      - name: Memory leak check
        run: cargo test --features memory-profiling
```

### Performance Monitoring

```rust
pub struct PerformanceMonitor {
    frame_times: RingBuffer<Duration>,
    system_times: HashMap<SystemId, RingBuffer<Duration>>,
    memory_snapshots: RingBuffer<MemorySnapshot>,
}

impl PerformanceMonitor {
    pub fn assert_performance_targets(&self) {
        let avg_frame_time = self.average_frame_time();
        assert!(avg_frame_time < Duration::from_millis(16), 
            "Frame time exceeds 60 FPS target: {:?}", avg_frame_time);
            
        for (system, times) in &self.system_times {
            let avg = times.average();
            let budget = SYSTEM_TIME_BUDGETS[system];
            assert!(avg < budget,
                "System {:?} exceeds time budget: {:?} > {:?}", 
                system, avg, budget);
        }
    }
}
```

## Test Coverage Goals

- **Unit Test Coverage**: 80% minimum
- **Integration Test Coverage**: 60% minimum  
- **Performance Benchmarks**: All critical paths
- **Simulation Validation**: Key emergent behaviors

## Running Tests

```bash
# Run all tests
cargo test

# Run specific test category
cargo test unit_
cargo test integration_
cargo test simulation_

# Run benchmarks
cargo bench

# Run with coverage
cargo tarpaulin --out Html

# Stress tests
cargo test --ignored --release
```

This comprehensive testing strategy ensures the simulation remains performant, correct, and biologically plausible as it evolves.