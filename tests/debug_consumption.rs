use creature_simulation::simulation::{Creature, CreatureState, ResourceType};
use creature_simulation::systems::Simulation;
use creature_simulation::Vec2;

#[test]
fn debug_resource_consumption() {
    // Initialize logging to see debug output
    let _ = tracing_subscriber::fmt().with_max_level(tracing::Level::DEBUG).try_init();

    let mut sim = Simulation::with_bounds(200.0, 200.0);

    // Add one very hungry creature in center
    let entity = sim.world.entities.create();
    let mut creature = Creature::new(entity, Vec2::new(100.0, 100.0));
    creature.needs.hunger = 0.9; // Very hungry
    creature.needs.thirst = 0.1; // Not thirsty
    println!(
        "Created creature at {:?} with hunger {}",
        creature.position, creature.needs.hunger
    );
    sim.world.creatures.insert(entity, creature);
    sim.world.spatial_grid.insert(entity, Vec2::new(100.0, 100.0));

    // Add food resource nearby
    let food_entity = sim.world.entities.create();
    let food = creature_simulation::simulation::Resource::new(
        food_entity,
        Vec2::new(110.0, 100.0),
        ResourceType::Food,
    );
    println!("Created food at {:?}", food.position);
    sim.world.resources.insert(food_entity, food);
    sim.world.spatial_grid.insert(food_entity, Vec2::new(110.0, 100.0));

    // Run simulation for a short time
    for i in 0..120 {
        // 2 seconds to give more time
        sim.update(1.0 / 60.0);

        // Check creature state every 5 frames for more detail
        if i % 5 == 0 {
            if let Some(creature) = sim.world.creatures.get(&entity) {
                println!(
                    "Frame {}: Creature at {:?}, state: {:?}, hunger: {:.2}",
                    i, creature.position, creature.state, creature.needs.hunger
                );

                // Check if creature is near food
                if let Some(food) = sim.world.resources.get(&food_entity) {
                    let distance = (creature.position - food.position).length();
                    println!(
                        "  Distance to food: {:.2}, food amount: {:.2}",
                        distance, food.amount
                    );

                    // Check if creature should be able to interact
                    if distance <= 2.0 {
                        println!("  >>> WITHIN INTERACTION DISTANCE! <<<");
                    }
                } else {
                    println!("  Food was consumed!");
                }
            }
        }
    }

    // Check final state
    let creature = sim.world.creatures.get(&entity).unwrap();
    println!("\nFinal state: {:?}", creature.state);
    println!("Final hunger: {}", creature.needs.hunger);
    println!("Final position: {:?}", creature.position);

    // Check if food was consumed
    let food_remaining = sim.world.resources.get(&food_entity).is_some();
    println!("Food still exists: {}", food_remaining);

    assert!(
        creature.needs.hunger < 0.9,
        "Creature should have eaten and reduced hunger"
    );
}
