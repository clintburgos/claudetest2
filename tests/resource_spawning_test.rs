use creature_simulation::systems::Simulation;
use creature_simulation::simulation::{Creature, ResourceType};
use glam::Vec2;
use log::debug;

#[test]
fn test_resource_spawning_maintains_density() {
    env_logger::init();
    
    let mut sim = Simulation::with_bounds(1000.0, 1000.0);
    
    // Initially should have no resources
    assert_eq!(sim.world.resource_count(), 0);
    
    // Update for a few seconds to let resources spawn
    for i in 0..180 { // 3 seconds at 60 FPS
        sim.update(1.0 / 60.0);
        
        if i % 60 == 0 {
            let food_count = sim.world.resources.values()
                .filter(|r| r.resource_type == ResourceType::Food)
                .count();
            let water_count = sim.world.resources.values()
                .filter(|r| r.resource_type == ResourceType::Water)
                .count();
            debug!("Time {}s - Food: {}, Water: {}", i / 60, food_count, water_count);
        }
    }
    
    // Count resources by type
    let food_count = sim.world.resources.values()
        .filter(|r| r.resource_type == ResourceType::Food)
        .count();
    let water_count = sim.world.resources.values()
        .filter(|r| r.resource_type == ResourceType::Water)
        .count();
    
    // Based on config:
    // - TARGET_FOOD_DENSITY = 0.5 per 100x100 area
    // - TARGET_WATER_DENSITY = 0.3 per 100x100 area
    // - World is 1000x1000 = 100 grid cells
    // Expected: ~50 food, ~30 water
    
    println!("Final counts - Food: {}, Water: {}", food_count, water_count);
    
    assert!(food_count >= 40 && food_count <= 60, "Food count {} not in expected range 40-60", food_count);
    assert!(water_count >= 20 && water_count <= 40, "Water count {} not in expected range 20-40", water_count);
}

#[test] 
fn test_resource_spawning_with_consumption() {
    let mut sim = Simulation::with_bounds(500.0, 500.0);
    
    // Add some hungry creatures
    for i in 0..10 {
        let entity = sim.world.entities.create();
        let mut creature = Creature::new(entity, Vec2::new(i as f32 * 50.0, 250.0));
        creature.needs.hunger = 0.7; // Hungry
        creature.needs.thirst = 0.7; // Thirsty
        sim.world.creatures.insert(entity, creature);
        sim.world.spatial_grid.insert(entity, Vec2::new(i as f32 * 50.0, 250.0));
    }
    
    // Let simulation run for 10 seconds
    for _ in 0..600 {
        sim.update(1.0 / 60.0);
    }
    
    // Should still have resources despite consumption
    let resource_count = sim.world.resource_count();
    println!("Resources after consumption: {}", resource_count);
    assert!(resource_count > 0, "Resources should respawn to maintain density");
    
    // Check that some creatures have satisfied their needs
    let satisfied_creatures = sim.world.creatures.values()
        .filter(|c| c.needs.hunger < 0.5 || c.needs.thirst < 0.5)
        .count();
    println!("Creatures with satisfied needs: {}/{}", satisfied_creatures, sim.world.creature_count());
    assert!(satisfied_creatures > 0, "At least some creatures should have found resources");
}

#[test]
fn test_resource_spacing() {
    let mut sim = Simulation::with_bounds(200.0, 200.0);
    
    // Update to spawn some resources
    for _ in 0..120 { // 2 seconds
        sim.update(1.0 / 60.0);
    }
    
    // Check that resources maintain minimum spacing
    let positions: Vec<Vec2> = sim.world.resources.values()
        .map(|r| r.position)
        .collect();
    
    let mut min_distance = f32::MAX;
    for i in 0..positions.len() {
        for j in i+1..positions.len() {
            let distance = (positions[i] - positions[j]).length();
            min_distance = min_distance.min(distance);
        }
    }
    
    println!("Minimum distance between resources: {}", min_distance);
    
    // Should maintain at least some spacing (not exact MIN_RESOURCE_SPACING due to spawning algorithm)
    if positions.len() > 1 {
        assert!(min_distance > 10.0, "Resources too close together: {}", min_distance);
    }
}