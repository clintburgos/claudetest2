use creature_simulation::{Vec2, core::*, simulation::*, systems::*};

#[test]
fn test_simple_eating() {
    let mut sim = Simulation::with_bounds(100.0, 100.0);
    
    // Create hungry creature at exact food position
    let creature_entity = sim.world.entities.create();
    let mut creature = Creature::new(creature_entity, Vec2::new(50.0, 50.0));
    creature.needs.hunger = 0.8;
    creature.state = CreatureState::Eating; // Force eating state
    sim.world.creatures.insert(creature_entity, creature);
    sim.world.spatial_grid.insert(creature_entity, Vec2::new(50.0, 50.0));
    
    // Create food at same position
    let food_entity = sim.world.entities.create();
    let food = Resource::new(food_entity, Vec2::new(50.0, 50.0), ResourceType::Food);
    sim.world.resources.insert(food_entity, food);
    sim.world.spatial_grid.insert(food_entity, Vec2::new(50.0, 50.0));
    
    let initial_hunger = sim.world.creatures[&creature_entity].needs.hunger;
    let initial_food = sim.world.resources[&food_entity].amount;
    
    // Run one update
    sim.update(1.0 / 60.0);
    
    let final_hunger = sim.world.creatures[&creature_entity].needs.hunger;
    let final_food = sim.world.resources[&food_entity].amount;
    
    println!("Initial hunger: {}, Final hunger: {}", initial_hunger, final_hunger);
    println!("Initial food: {}, Final food: {}", initial_food, final_food);
    
    assert!(final_hunger < initial_hunger, "Hunger should decrease");
    assert!(final_food < initial_food, "Food should be consumed");
}