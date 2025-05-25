//! Simple example demonstrating Phase 1 improvements
//!
//! This example shows the three key improvements:
//! - Entity versioning system
//! - Decoupled decision system
//! - Optimized spatial queries
//!
//! Run with: cargo run --example phase1_demo_simple

use creature_simulation::{
    core::{Entity as CoreEntity, EntityVersions, SpatialHashGrid, Version, VersionedEntity},
    simulation::{Creature, CreatureState, Resource, ResourceType},
    systems::decision_v2::{
        decision_functions, DecisionCache, DecisionContext, NeedState, ResourceInfo,
    },
    Vec2,
};

fn main() {
    println!("=== Phase 1 Improvements Demo ===\n");

    // Demo 1: Entity Versioning
    demo_entity_versioning();

    // Demo 2: Decoupled Decision System
    demo_decision_system();

    // Demo 3: Spatial Optimization
    demo_spatial_optimization();
}

fn demo_entity_versioning() {
    println!("1. Entity Versioning Demo");
    println!("-------------------------");

    let versions = EntityVersions::new();

    // Allocate some entities
    let e1 = versions.allocate();
    let e2 = versions.allocate();

    println!(
        "Allocated entity 1: id={}, generation={}",
        e1.id, e1.generation
    );
    println!(
        "Allocated entity 2: id={}, generation={}",
        e2.id, e2.generation
    );

    // Verify they're valid
    println!("Entity 1 valid: {}", versions.is_valid(e1));
    println!("Entity 2 valid: {}", versions.is_valid(e2));

    // Deallocate entity 1
    versions.deallocate(e1);
    println!("\nDeallocated entity 1");
    println!("Entity 1 valid: {}", versions.is_valid(e1));

    // Allocate again - should reuse ID with new generation
    let e3 = versions.allocate();
    println!(
        "\nAllocated entity 3: id={}, generation={}",
        e3.id, e3.generation
    );
    println!(
        "Old entity 1 reference still invalid: {}",
        versions.is_valid(e1)
    );
    println!("New entity 3 valid: {}", versions.is_valid(e3));

    println!("\n✓ Entity versioning prevents stale references!\n");
}

fn demo_decision_system() {
    println!("2. Decoupled Decision System Demo");
    println!("---------------------------------");

    // Create a decision context for a hungry creature
    let context = DecisionContext {
        entity: VersionedEntity::new(1, 0),
        position: Vec2::new(100.0, 100.0),
        velocity: Vec2::ZERO,
        state: CreatureState::Idle,
        needs: NeedState {
            hunger: 0.8, // Very hungry!
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
                position: Vec2::new(90.0, 100.0),
                resource_type: ResourceType::Water,
                amount: 100.0,
                distance: 10.0,
            },
        ],
        nearby_creatures: vec![],
        nearby_threats: vec![],
        time_since_last_decision: 1.0,
    };

    // Make decision using pure function
    let decision = decision_functions::make_decision(&context);
    println!("Creature state: {:?}", context.state);
    println!("Hunger level: {:.1}", context.needs.hunger);
    println!(
        "Nearby resources: {} food, {} water",
        context
            .nearby_resources
            .iter()
            .filter(|r| r.resource_type == ResourceType::Food)
            .count(),
        context
            .nearby_resources
            .iter()
            .filter(|r| r.resource_type == ResourceType::Water)
            .count()
    );
    println!("Decision made: {:?}", decision);

    // Demo caching
    let cache = DecisionCache::new(100);
    cache.insert(&context, decision.clone(), 0.0);

    if let Some(cached) = cache.get(&context, 0.5) {
        println!("\n✓ Decision retrieved from cache!");
    }

    println!("\n✓ Decision system uses pure functions and caching!\n");
}

fn demo_spatial_optimization() {
    println!("3. Spatial Optimization Demo");
    println!("----------------------------");

    let grid = SpatialHashGrid::new(10.0);

    // Insert many entities
    let entity_count = 1000;
    println!("Inserting {} entities...", entity_count);

    let start = std::time::Instant::now();
    for i in 0..entity_count {
        let entity = bevy::ecs::entity::Entity::from_raw(i);
        let position = Vec2::new((i as f32 * 13.0) % 500.0, (i as f32 * 17.0) % 500.0);
        grid.update_entity(entity, position);
    }
    let insert_time = start.elapsed();

    println!("Insert time: {:.2}ms", insert_time.as_secs_f64() * 1000.0);

    // Perform queries
    let query_count = 100;
    println!("\nPerforming {} spatial queries...", query_count);

    let start = std::time::Instant::now();
    let mut total_found = 0;
    for i in 0..query_count {
        let center = Vec2::new((i as f32 * 31.0) % 500.0, (i as f32 * 37.0) % 500.0);
        let results = grid.query_radius(center, 50.0);
        total_found += results.len();
    }
    let query_time = start.elapsed();

    println!("Query time: {:.2}ms", query_time.as_secs_f64() * 1000.0);
    println!(
        "Average query time: {:.3}ms",
        query_time.as_secs_f64() * 1000.0 / query_count as f64
    );
    println!("Total entities found: {}", total_found);

    // Show metrics
    let metrics = grid.metrics();
    println!("\nSpatial Grid Metrics:");
    println!("- Cache hit rate: {:.1}%", metrics.cache_hit_rate * 100.0);
    println!("- Entities tracked: {}", metrics.entity_count);
    println!("- Active cells: {}", metrics.cell_count);
    println!("- Total queries: {}", metrics.total_queries);

    println!("\n✓ Spatial system uses caching and incremental updates!\n");
}
