use creature_simulation::simulation::{Creature, ResourceType};
use creature_simulation::systems::Simulation;
use creature_simulation::Vec2;

#[test]
fn debug_resource_spawning_and_consumption() {
    tracing_subscriber::fmt::init();

    let mut sim = Simulation::with_bounds(500.0, 500.0);

    // Add one hungry creature in center
    let entity = sim.world.entities.create();
    let mut creature = Creature::new(entity, Vec2::new(250.0, 250.0));
    creature.needs.hunger = 0.8; // Very hungry
    creature.needs.thirst = 0.3; // Not too thirsty
    sim.world.creatures.insert(entity, creature);
    sim.world.spatial_grid.insert(entity, Vec2::new(250.0, 250.0));

    println!(
        "Initial creature state: {:?}",
        sim.world.creatures[&entity].state
    );
    println!(
        "Initial needs - Hunger: {}, Thirst: {}",
        sim.world.creatures[&entity].needs.hunger, sim.world.creatures[&entity].needs.thirst
    );

    // Run for a few seconds and monitor
    for i in 0..300 {
        // 5 seconds
        sim.update(1.0 / 60.0);

        if i % 60 == 0 || i < 10 {
            // Every second or first 10 frames
            let creature = &sim.world.creatures[&entity];
            let food_count = sim
                .world
                .resources
                .values()
                .filter(|r| r.resource_type == ResourceType::Food)
                .count();
            let water_count = sim
                .world
                .resources
                .values()
                .filter(|r| r.resource_type == ResourceType::Water)
                .count();

            println!(
                "Frame {}: State={:?}, Pos={:?}, Hunger={:.2}, Resources: {} food, {} water",
                i,
                creature.state,
                creature.position,
                creature.needs.hunger,
                food_count,
                water_count
            );

            // Check if any food is near the creature
            if food_count > 0 {
                let nearest_food =
                    sim.world.find_nearest_resource(creature.position, ResourceType::Food, None);
                if let Some((food_entity, distance)) = nearest_food {
                    let food = &sim.world.resources[&food_entity];
                    println!(
                        "  Nearest food at {:?}, distance: {:.1}, amount: {:.1}",
                        food.position, distance, food.amount
                    );
                }
            }
        }

        // Stop if creature starts eating
        if matches!(
            sim.world.creatures[&entity].state,
            creature_simulation::simulation::CreatureState::Eating
        ) {
            println!("Creature started eating at frame {}!", i);

            // Continue for a bit to see consumption
            for j in 0..60 {
                sim.update(1.0 / 60.0);
                if j % 10 == 0 {
                    let creature = &sim.world.creatures[&entity];
                    println!(
                        "  Eating frame {}: Hunger={:.2}, State={:?}",
                        j, creature.needs.hunger, creature.state
                    );
                }
            }
            break;
        }
    }

    let final_creature = &sim.world.creatures[&entity];
    println!("\nFinal state: {:?}", final_creature.state);
    println!(
        "Final needs - Hunger: {:.2}, Thirst: {:.2}",
        final_creature.needs.hunger, final_creature.needs.thirst
    );
}
