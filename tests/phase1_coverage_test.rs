use bevy::prelude::*;
use creature_simulation::core::{spatial_v2::*, versioned_entity::*};
use creature_simulation::simulation::CreatureState;
use creature_simulation::systems::decision_v2::*;

#[test]
fn test_versioned_entity_integration() {
    // Test that versioned entities work with Bevy's Entity type
    let versions = EntityVersions::new();

    // Allocate some entities
    let v1 = versions.allocate();
    let v2 = versions.allocate();

    // Convert to Bevy entities
    let e1 = Entity::from_raw(v1.id);
    let e2 = Entity::from_raw(v2.id);

    // Test conversion back
    assert_eq!(e1.to_versioned(&versions), Some(v1));
    assert_eq!(e2.to_versioned(&versions), Some(v2));

    // Test invalid entity
    let invalid = Entity::from_raw(999);
    assert_eq!(invalid.to_versioned(&versions), None);
}

#[test]
fn test_spatial_grid_with_versioned_entities() {
    let grid = SpatialHashGrid::new(10.0);
    let versions = EntityVersions::new();

    // Create versioned entities
    let v1 = versions.allocate();
    let v2 = versions.allocate();

    // Use them with spatial grid
    let e1 = Entity::from_raw(v1.id);
    let e2 = Entity::from_raw(v2.id);

    grid.update_entity(e1, Vec2::new(0.0, 0.0));
    grid.update_entity(e2, Vec2::new(5.0, 5.0));

    // Query should find both
    let results = grid.query_radius(Vec2::new(2.5, 2.5), 5.0);
    assert_eq!(results.len(), 2);
}

#[test]
fn test_decision_context_with_real_data() {
    use creature_simulation::simulation::{CreatureState, ResourceType};

    // Create a realistic decision context
    let context = DecisionContext {
        entity: VersionedEntity::new(1, 0),
        position: Vec2::new(100.0, 100.0),
        velocity: Vec2::new(1.0, 0.0),
        state: CreatureState::Moving {
            target: Vec2::new(110.0, 100.0),
        },
        needs: NeedState {
            hunger: 0.7,
            thirst: 0.3,
            energy: 0.6,
            social: 0.4,
        },
        health: 0.8,
        energy: 0.6,
        nearby_resources: vec![
            ResourceInfo {
                entity: VersionedEntity::new(2, 0),
                position: Vec2::new(120.0, 100.0),
                resource_type: ResourceType::Food,
                amount: 30.0,
                distance: 20.0,
            },
            ResourceInfo {
                entity: VersionedEntity::new(3, 0),
                position: Vec2::new(90.0, 90.0),
                resource_type: ResourceType::Water,
                amount: 50.0,
                distance: 14.14,
            },
        ],
        nearby_creatures: vec![CreatureInfo {
            entity: VersionedEntity::new(4, 0),
            position: Vec2::new(105.0, 105.0),
            relationship: Relationship::Friendly,
            distance: 7.07,
        }],
        nearby_threats: vec![ThreatInfo {
            position: Vec2::new(150.0, 100.0),
            threat_level: 0.3,
            distance: 50.0,
        }],
        time_since_last_decision: 0.5,
    };

    // Make decision
    let decision = decision_functions::make_decision(&context);

    // Debug the decision
    println!("Decision: {:?}", decision);

    // With hunger at 0.7 and food 20 units away, should decide to consume it
    // if within decision radius, otherwise move toward it
    match decision {
        Decision::Consume { resource_type, .. } => {
            assert_eq!(
                resource_type,
                creature_simulation::simulation::ResourceType::Food
            );
        },
        Decision::Move { urgency, .. } => {
            // Moving to search for food with hunger urgency
            assert!(urgency >= 0.7);
        },
        _ => panic!("Expected consume or move decision, got {:?}", decision),
    }
}

#[test]
fn test_decision_cache_performance() {
    let cache = DecisionCache::new(100);
    let base_context = DecisionContext {
        entity: VersionedEntity::new(1, 0),
        position: Vec2::new(0.0, 0.0),
        velocity: Vec2::ZERO,
        state: CreatureState::Idle,
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

    // Insert many decisions
    for i in 0..100 {
        let mut context = base_context.clone();
        context.position.x = i as f32;
        let decision = Decision::Move {
            target: Vec2::new(i as f32 * 2.0, 0.0),
            urgency: 0.5,
        };
        cache.insert(&context, decision, 0.0);
    }

    // All should be cached
    for i in 0..100 {
        let mut context = base_context.clone();
        context.position.x = i as f32;
        assert!(cache.get(&context, 0.5).is_some());
    }

    // Old entries should expire
    for i in 0..100 {
        let mut context = base_context.clone();
        context.position.x = i as f32;
        assert!(cache.get(&context, 1.5).is_none());
    }
}

#[test]
fn test_spatial_grid_stress() {
    let grid = SpatialHashGrid::new(50.0);

    // Insert 1000 entities
    for i in 0..1000 {
        let entity = Entity::from_raw(i);
        let x = (i % 100) as f32 * 10.0;
        let y = (i / 100) as f32 * 10.0;
        grid.update_entity(entity, Vec2::new(x, y));
    }

    // Perform many queries - query where entities actually are
    for i in 0..100 {
        let center = Vec2::new((i % 10) as f32 * 100.0, (i / 10) as f32 * 100.0);
        let results = grid.query_radius(center, 50.0);
        // Should find some entities in most queries
        if i < 10 {
            assert!(results.len() > 0);
        }
    }

    // Check cache effectiveness
    let metrics = grid.metrics();
    // With 100 different queries, cache hit rate won't be very high
    println!("Cache hit rate: {}", metrics.cache_hit_rate);
    assert!(metrics.total_queries >= 100);
}
