//! Integration tests for Phase 1 improvements
//!
//! Tests the entity versioning, decoupled decision system, and spatial optimization
//! working together in realistic scenarios.

use bevy::prelude::*;
use creature_simulation::core::{EntityVersions, SpatialHashGrid, Version, VersionedEntity};
use creature_simulation::simulation::ResourceType;
use creature_simulation::systems::decision_v2::{
    decision_functions, Decision, DecisionContext, NeedState, ResourceInfo,
};

#[test]
fn test_versioned_entity_lifecycle() {
    let versions = EntityVersions::new();

    // Test entity allocation
    let e1 = versions.allocate();
    assert_eq!(e1.generation, 0);

    // Test validity check
    assert!(versions.is_valid(e1));

    // Test deallocation and recycling
    assert!(versions.deallocate(e1));
    assert!(!versions.is_valid(e1)); // Old reference should be invalid

    // Allocate again - should get same ID with new generation
    let e2 = versions.allocate();
    assert_eq!(e2.id, e1.id);
    assert_eq!(e2.generation, 1);

    // Old reference should still be invalid
    assert!(!versions.is_valid(e1));
    assert!(versions.is_valid(e2));
}

#[test]
fn test_version_component_auto_increment() {
    let mut component = Version::new(100.0f32);
    assert_eq!(component.generation, 0);
    assert_eq!(*component, 100.0);

    // Direct access doesn't increment
    let _ = component.data;
    assert_eq!(component.generation, 0);

    // Mutable access increments
    *component = 200.0;
    assert_eq!(component.generation, 1);
    assert_eq!(*component, 200.0);
}

#[test]
fn test_decision_system_integration() {
    // Create a hungry creature near food
    let context = DecisionContext {
        entity: VersionedEntity::new(1, 0),
        position: Vec2::new(100.0, 100.0),
        velocity: Vec2::ZERO,
        state: creature_simulation::simulation::CreatureState::Idle,
        needs: NeedState {
            hunger: 0.9, // Very hungry
            thirst: 0.3,
            energy: 0.7,
            social: 0.2,
        },
        health: 1.0,
        energy: 0.7,
        nearby_resources: vec![
            ResourceInfo {
                entity: VersionedEntity::new(2, 0),
                position: Vec2::new(110.0, 100.0),
                resource_type: ResourceType::Food,
                amount: 50.0,
                distance: 10.0,
            },
            ResourceInfo {
                entity: VersionedEntity::new(3, 0),
                position: Vec2::new(150.0, 100.0),
                resource_type: ResourceType::Water,
                amount: 100.0,
                distance: 50.0,
            },
        ],
        nearby_creatures: vec![],
        nearby_threats: vec![],
        time_since_last_decision: 1.0,
    };

    let decision = decision_functions::make_decision(&context);

    // Should decide to consume the nearby food
    match decision {
        Decision::Consume {
            resource,
            resource_type,
        } => {
            assert_eq!(resource.id, 2);
            assert_eq!(resource_type, ResourceType::Food);
        },
        _ => panic!("Expected creature to decide to eat nearby food"),
    }
}

#[test]
fn test_spatial_grid_performance() {
    let grid = SpatialHashGrid::new(10.0);

    // Insert many entities in a pattern
    for i in 0..100 {
        for j in 0..100 {
            let entity = Entity::from_raw(i * 100 + j);
            let position = Vec2::new(i as f32 * 5.0, j as f32 * 5.0);
            grid.update_entity(entity, position);
        }
    }

    // Test query performance
    let start = std::time::Instant::now();
    let results = grid.query_radius(Vec2::new(250.0, 250.0), 50.0);
    let query_time = start.elapsed();

    // Should be very fast even with 10,000 entities
    assert!(query_time.as_micros() < 1000); // Less than 1ms
    assert!(!results.is_empty());

    // Check metrics
    let metrics = grid.metrics();
    assert_eq!(metrics.entity_count, 10000);
    assert!(metrics.cells_checked > 0);
    assert!(metrics.entities_checked > 0);
}

#[test]
fn test_spatial_cache_effectiveness() {
    let grid = SpatialHashGrid::new(10.0);

    // Insert some entities
    for i in 0..10 {
        let entity = Entity::from_raw(i);
        grid.update_entity(entity, Vec2::new(i as f32 * 10.0, 0.0));
    }

    // First query - should miss cache
    let _ = grid.query_radius(Vec2::new(50.0, 0.0), 20.0);
    let metrics = grid.metrics();
    assert_eq!(metrics.cache_hits, 0);
    assert_eq!(metrics.cache_misses, 1);

    // Same query - should hit cache
    let _ = grid.query_radius(Vec2::new(50.0, 0.0), 20.0);
    let metrics = grid.metrics();
    assert_eq!(metrics.cache_hits, 1);
    assert_eq!(metrics.cache_misses, 1);

    // Move an entity that's in a different cell to ensure cache isn't invalidated
    // Entity 5 is at (50, 0), moving to (55, 0) might still be in same/adjacent cells
    // Let's move an entity that's far away to avoid cache invalidation
    grid.update_entity(Entity::from_raw(9), Vec2::new(95.0, 0.0));

    // Query again - cache should still be valid since we moved a distant entity
    let _ = grid.query_radius(Vec2::new(50.0, 0.0), 20.0);
    let metrics = grid.metrics();
    assert_eq!(metrics.cache_hits, 2); // Should get another cache hit
    assert_eq!(metrics.cache_misses, 1); // Still only one miss
}

#[test]
fn test_decision_caching() {
    use creature_simulation::systems::decision_v2::DecisionCache;

    let cache = DecisionCache::new(100);

    let context = DecisionContext {
        entity: VersionedEntity::new(1, 0),
        position: Vec2::new(100.0, 100.0),
        velocity: Vec2::ZERO,
        state: creature_simulation::simulation::CreatureState::Idle,
        needs: NeedState {
            hunger: 0.5,
            thirst: 0.5,
            energy: 0.5,
            social: 0.5,
        },
        health: 1.0,
        energy: 0.5,
        nearby_resources: vec![],
        nearby_creatures: vec![],
        nearby_threats: vec![],
        time_since_last_decision: 1.0,
    };

    // Cache a decision
    let decision = Decision::Idle;
    cache.insert(&context, decision.clone(), 0.0);

    // Should retrieve from cache within timeout
    assert_eq!(cache.get(&context, 0.5), Some(Decision::Idle));

    // Should expire after timeout
    assert_eq!(cache.get(&context, 1.5), None);
}

#[test]
fn test_batch_spatial_updates() {
    let grid = SpatialHashGrid::new(10.0);

    // Prepare batch updates
    let updates: Vec<_> = (0..1000)
        .map(|i| {
            let entity = Entity::from_raw(i);
            let position = Vec2::new((i as f32 * 7.0) % 500.0, (i as f32 * 11.0) % 500.0);
            (entity, position)
        })
        .collect();

    // Batch update should be efficient
    let start = std::time::Instant::now();
    grid.update_entities_batch(&updates);
    let update_time = start.elapsed();

    assert!(update_time.as_millis() < 10); // Should be very fast

    // Verify all entities are queryable
    let all_entities = grid.query_radius(Vec2::new(250.0, 250.0), 1000.0);
    assert_eq!(all_entities.len(), 1000);
}

#[test]
fn test_decision_with_threats() {
    use creature_simulation::systems::decision_v2::ThreatInfo;

    let context = DecisionContext {
        entity: VersionedEntity::new(1, 0),
        position: Vec2::new(100.0, 100.0),
        velocity: Vec2::ZERO,
        state: creature_simulation::simulation::CreatureState::Idle,
        needs: NeedState {
            hunger: 0.8, // Hungry but...
            thirst: 0.3,
            energy: 0.7,
            social: 0.2,
        },
        health: 1.0,
        energy: 0.7,
        nearby_resources: vec![ResourceInfo {
            entity: VersionedEntity::new(2, 0),
            position: Vec2::new(110.0, 100.0),
            resource_type: ResourceType::Food,
            amount: 50.0,
            distance: 10.0,
        }],
        nearby_creatures: vec![],
        nearby_threats: vec![ThreatInfo {
            position: Vec2::new(105.0, 100.0),
            threat_level: 0.9, // High threat nearby!
            distance: 5.0,
        }],
        time_since_last_decision: 1.0,
    };

    let decision = decision_functions::make_decision(&context);

    // Should flee despite being hungry
    match decision {
        Decision::Flee { direction } => {
            // Should flee away from threat (negative x direction)
            assert!(direction.x < 0.0);
        },
        _ => panic!("Expected creature to flee from nearby threat"),
    }
}

// Performance regression test
#[test]
fn test_performance_regression() {
    let grid = SpatialHashGrid::new(10.0);
    let versions = EntityVersions::new();

    // Create a realistic scenario with many entities
    let mut entities = Vec::new();
    for i in 0..5000 {
        let versioned = versions.allocate();
        let entity = Entity::from_raw(versioned.id);
        entities.push((entity, versioned));

        let position = Vec2::new((i as f32 * 13.0) % 1000.0, (i as f32 * 17.0) % 1000.0);
        grid.update_entity(entity, position);
    }

    // Measure query performance with some repeated queries
    let mut total_query_time = std::time::Duration::ZERO;
    let query_count = 100;
    let unique_locations = 20; // Use fewer unique locations to ensure cache hits

    for i in 0..query_count {
        // Use modulo to repeat some queries
        let location_index = i % unique_locations;
        let center = Vec2::new(
            (location_index as f32 * 50.0) % 1000.0,
            (location_index as f32 * 50.0) % 1000.0,
        );

        let start = std::time::Instant::now();
        let _ = grid.query_radius(center, 50.0);
        total_query_time += start.elapsed();
    }

    let avg_query_time = total_query_time / query_count;

    // Performance assertions
    assert!(avg_query_time.as_micros() < 500); // Average query < 0.5ms

    // Check cache effectiveness
    let metrics = grid.metrics();
    assert!(
        metrics.cache_hit_rate > 0.0,
        "Cache hit rate was {}, expected > 0.0",
        metrics.cache_hit_rate
    ); // Should have some cache hits
}
