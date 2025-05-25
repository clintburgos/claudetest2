use creature_simulation::{Vec2, core::*, simulation::*, systems::*};

#[test]
fn test_basic_simulation_flow() {
    // Initialize logging
    let _ = env_logger::builder().is_test(true).try_init();
    
    let mut sim = Simulation::with_bounds(100.0, 100.0);
    
    // Create hungry creature
    let creature_entity = sim.world.entities.create();
    let mut creature = Creature::new(creature_entity, Vec2::new(50.0, 50.0));
    creature.needs.hunger = 0.8;
    println!("Initial creature hunger: {}", creature.needs.hunger);
    sim.world.creatures.insert(creature_entity, creature);
    sim.world.spatial_grid.insert(creature_entity, Vec2::new(50.0, 50.0));
    
    // Create food very close
    let food_entity = sim.world.entities.create();
    let food = Resource::new(food_entity, Vec2::new(51.0, 50.0), ResourceType::Food);
    println!("Food position: {:?}, amount: {}", food.position, food.amount);
    sim.world.resources.insert(food_entity, food);
    sim.world.spatial_grid.insert(food_entity, Vec2::new(51.0, 50.0));
    
    // Check initial most urgent need
    let creature = &sim.world.creatures[&creature_entity];
    println!("Most urgent need: {:?}", creature.needs.most_urgent());
    println!("Hunger urgency: {}", creature.needs.get_urgency(creature_simulation::simulation::needs::NeedType::Hunger));
    
    // Run one update
    println!("\n=== Update 1 ===");
    sim.update(1.0 / 60.0);
    
    let creature = &sim.world.creatures[&creature_entity];
    println!("After update 1 - State: {:?}, Position: {:?}, Hunger: {}", 
             creature.state, creature.position, creature.needs.hunger);
    
    // Run more updates
    for i in 2..=10 {
        println!("\n=== Update {} ===", i);
        sim.update(1.0 / 60.0);
        
        let creature = &sim.world.creatures[&creature_entity];
        let food = &sim.world.resources[&food_entity];
        println!("Creature - State: {:?}, Position: {:?}, Hunger: {}", 
                 creature.state, creature.position, creature.needs.hunger);
        println!("Food amount: {}", food.amount);
        
        if creature.needs.hunger < 0.8 {
            println!("SUCCESS: Creature hunger reduced!");
            break;
        }
    }
    
    let final_creature = &sim.world.creatures[&creature_entity];
    assert!(final_creature.needs.hunger < 0.8, "Creature should have eaten");
}