use creature_simulation::{Vec2, core::*, simulation::*, systems::*};

/// Helper to create a simulation with bounds
fn create_test_simulation(width: f32, height: f32) -> Simulation {
    Simulation::with_bounds(width, height)
}

/// Helper to spawn a creature at a position
fn spawn_creature(sim: &mut Simulation, position: Vec2) -> Entity {
    let entity = sim.world.entities.create();
    let creature = Creature::new(entity, position);
    sim.world.creatures.insert(entity, creature);
    sim.world.spatial_grid.insert(entity, position);
    entity
}

/// Helper to spawn a resource at a position
fn spawn_resource(sim: &mut Simulation, position: Vec2, resource_type: ResourceType) -> Entity {
    let entity = sim.world.entities.create();
    let resource = Resource::new(entity, position, resource_type);
    sim.world.resources.insert(entity, resource);
    sim.world.spatial_grid.insert(entity, position);
    entity
}

#[test]
fn test_creature_finds_and_consumes_food() {
    let mut sim = create_test_simulation(100.0, 100.0);
    
    // Create hungry creature
    let creature_entity = spawn_creature(&mut sim, Vec2::new(50.0, 50.0));
    sim.world.creatures.get_mut(&creature_entity).unwrap().needs.hunger = 0.8;
    
    // Create food nearby
    let food_entity = spawn_resource(&mut sim, Vec2::new(55.0, 50.0), ResourceType::Food);
    
    // Run simulation for 3 seconds
    for _ in 0..180 {
        sim.update(1.0 / 60.0);
    }
    
    // Verify creature consumed food
    let creature = &sim.world.creatures[&creature_entity];
    assert!(creature.needs.hunger < 0.8, "Creature should have reduced hunger");
    
    let resource = &sim.world.resources[&food_entity];
    assert!(resource.amount < resource.max_amount, "Food should have been consumed");
}

#[test]
fn test_creature_finds_and_consumes_water() {
    let mut sim = create_test_simulation(100.0, 100.0);
    
    // Create thirsty creature
    let creature_entity = spawn_creature(&mut sim, Vec2::new(50.0, 50.0));
    sim.world.creatures.get_mut(&creature_entity).unwrap().needs.thirst = 0.8;
    
    // Create water nearby
    let water_entity = spawn_resource(&mut sim, Vec2::new(52.0, 50.0), ResourceType::Water);
    
    // Run simulation for 3 seconds
    for _ in 0..180 {
        sim.update(1.0 / 60.0);
    }
    
    // Verify creature consumed water
    let creature = &sim.world.creatures[&creature_entity];
    assert!(creature.needs.thirst < 0.8, "Creature should have reduced thirst");
    
    let resource = &sim.world.resources[&water_entity];
    assert!(resource.amount < resource.max_amount, "Water should have been consumed");
}

#[test]
fn test_creature_prioritizes_most_urgent_need() {
    let mut sim = create_test_simulation(100.0, 100.0);
    
    // Create creature with higher thirst than hunger
    let creature_entity = spawn_creature(&mut sim, Vec2::new(50.0, 50.0));
    {
        let creature = sim.world.creatures.get_mut(&creature_entity).unwrap();
        creature.needs.hunger = 0.5;
        creature.needs.thirst = 0.8;
    }
    
    // Create both resources equidistant
    spawn_resource(&mut sim, Vec2::new(45.0, 50.0), ResourceType::Food);
    spawn_resource(&mut sim, Vec2::new(55.0, 50.0), ResourceType::Water);
    
    // Run simulation briefly
    for _ in 0..30 {
        sim.update(1.0 / 60.0);
    }
    
    // Creature should be moving towards water (higher priority)
    let creature = &sim.world.creatures[&creature_entity];
    if let CreatureState::Moving { target } = creature.state {
        assert!(target.x > 50.0, "Creature should move towards water (right)");
    }
}

#[test]
fn test_creature_dies_from_starvation() {
    let mut sim = create_test_simulation(100.0, 100.0);
    
    // Create starving creature
    let creature_entity = spawn_creature(&mut sim, Vec2::new(50.0, 50.0));
    {
        let creature = sim.world.creatures.get_mut(&creature_entity).unwrap();
        creature.needs.hunger = 1.0; // Already starving
        creature.health.current = 10.0; // Low health
    }
    
    // Track death events
    let mut death_event_received = false;
    let mut death_cause = None;
    
    // Run simulation until creature dies
    for _ in 0..60 {
        sim.update(1.0 / 60.0);
        
        // Check for death events
        sim.world.events.process(|event| {
            if let GameEvent::CreatureDied { entity, cause } = event {
                if entity == &creature_entity {
                    death_event_received = true;
                    death_cause = Some(*cause);
                }
            }
        });
        
        if death_event_received {
            break;
        }
    }
    
    assert!(death_event_received, "Death event should be emitted");
    assert_eq!(death_cause, Some(DeathCause::Starvation), "Should die from starvation");
    
    let creature = &sim.world.creatures[&creature_entity];
    assert!(!creature.is_alive(), "Creature should be dead");
}

#[test]
fn test_creature_dies_from_dehydration() {
    let mut sim = create_test_simulation(100.0, 100.0);
    
    // Create dehydrated creature
    let creature_entity = spawn_creature(&mut sim, Vec2::new(50.0, 50.0));
    {
        let creature = sim.world.creatures.get_mut(&creature_entity).unwrap();
        creature.needs.thirst = 1.0; // Already dehydrated
        creature.health.current = 10.0; // Low health
    }
    
    // Track death events
    let mut death_cause = None;
    
    // Run simulation until creature dies
    for _ in 0..60 {
        sim.update(1.0 / 60.0);
        
        sim.world.events.process(|event| {
            if let GameEvent::CreatureDied { entity, cause } = event {
                if entity == &creature_entity {
                    death_cause = Some(*cause);
                }
            }
        });
        
        if death_cause.is_some() {
            break;
        }
    }
    
    assert_eq!(death_cause, Some(DeathCause::Dehydration), "Should die from dehydration");
}

#[test]
fn test_exhausted_creature_rests() {
    let mut sim = create_test_simulation(100.0, 100.0);
    
    // Create exhausted creature
    let creature_entity = spawn_creature(&mut sim, Vec2::new(50.0, 50.0));
    sim.world.creatures.get_mut(&creature_entity).unwrap().needs.energy = 0.1;
    
    // Run simulation
    for _ in 0..60 {
        sim.update(1.0 / 60.0);
    }
    
    // Creature should have rested and recovered energy
    let creature = &sim.world.creatures[&creature_entity];
    assert!(creature.needs.energy > 0.1, "Creature should have recovered energy");
}

#[test]
fn test_resource_depletion_event() {
    let mut sim = create_test_simulation(100.0, 100.0);
    
    // Create hungry creature
    let creature_entity = spawn_creature(&mut sim, Vec2::new(50.0, 50.0));
    sim.world.creatures.get_mut(&creature_entity).unwrap().needs.hunger = 0.8;
    
    // Create small food resource
    let food_entity = spawn_resource(&mut sim, Vec2::new(51.0, 50.0), ResourceType::Food);
    sim.world.resources.get_mut(&food_entity).unwrap().amount = 5.0; // Very little food
    
    let mut depletion_event_received = false;
    
    // Run simulation until resource depletes
    for _ in 0..300 {
        sim.update(1.0 / 60.0);
        
        sim.world.events.process(|event| {
            if let GameEvent::ResourceDepleted { entity } = event {
                if entity == &food_entity {
                    depletion_event_received = true;
                }
            }
        });
        
        if depletion_event_received {
            break;
        }
    }
    
    assert!(depletion_event_received, "Resource depletion event should be emitted");
    assert!(sim.world.resources[&food_entity].is_depleted(), "Resource should be depleted");
}

#[test]
fn test_resource_consumption_event() {
    let mut sim = create_test_simulation(100.0, 100.0);
    
    // Create hungry creature near food
    let creature_entity = spawn_creature(&mut sim, Vec2::new(50.0, 50.0));
    {
        let creature = sim.world.creatures.get_mut(&creature_entity).unwrap();
        creature.needs.hunger = 0.8;
        creature.state = CreatureState::Eating; // Already eating
    }
    
    // Create food at same position
    let food_entity = spawn_resource(&mut sim, Vec2::new(50.0, 50.0), ResourceType::Food);
    
    let mut consumption_event_received = false;
    let mut total_consumed = 0.0;
    
    // Run one update cycle
    sim.update(1.0 / 60.0);
    
    sim.world.events.process(|event| {
        if let GameEvent::ResourceConsumed { creature, resource, amount } = event {
            if creature == &creature_entity && resource == &food_entity {
                consumption_event_received = true;
                total_consumed += amount;
            }
        }
    });
    
    assert!(consumption_event_received, "Consumption event should be emitted");
    assert!(total_consumed > 0.0, "Some food should have been consumed");
}

#[test]
fn test_multiple_creatures_share_resources() {
    let mut sim = create_test_simulation(100.0, 100.0);
    
    // Create two hungry creatures
    let creature1 = spawn_creature(&mut sim, Vec2::new(48.0, 50.0));
    let creature2 = spawn_creature(&mut sim, Vec2::new(52.0, 50.0));
    
    sim.world.creatures.get_mut(&creature1).unwrap().needs.hunger = 0.8;
    sim.world.creatures.get_mut(&creature2).unwrap().needs.hunger = 0.8;
    
    // Create food between them
    spawn_resource(&mut sim, Vec2::new(50.0, 50.0), ResourceType::Food);
    
    // Run simulation
    for _ in 0..300 {
        sim.update(1.0 / 60.0);
    }
    
    // Both creatures should have reduced hunger
    let c1 = &sim.world.creatures[&creature1];
    let c2 = &sim.world.creatures[&creature2];
    
    assert!(c1.needs.hunger < 0.8 || c2.needs.hunger < 0.8, 
            "At least one creature should have eaten");
}

#[test]
fn test_creature_stops_eating_when_satisfied() {
    let mut sim = create_test_simulation(100.0, 100.0);
    
    // Create slightly hungry creature
    let creature_entity = spawn_creature(&mut sim, Vec2::new(50.0, 50.0));
    sim.world.creatures.get_mut(&creature_entity).unwrap().needs.hunger = 0.15;
    
    // Create food nearby
    spawn_resource(&mut sim, Vec2::new(52.0, 50.0), ResourceType::Food);
    
    // Run simulation
    for _ in 0..180 {
        sim.update(1.0 / 60.0);
    }
    
    // Creature should have stopped eating and returned to idle
    let creature = &sim.world.creatures[&creature_entity];
    assert!(creature.needs.hunger <= 0.1, "Hunger should be satisfied");
    assert!(matches!(creature.state, CreatureState::Idle | CreatureState::Moving { .. }), 
            "Creature should not still be eating");
}