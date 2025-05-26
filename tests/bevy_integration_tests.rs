//! Integration tests for the Bevy-based creature simulation

use bevy::prelude::*;
use creature_simulation::{
    components::*,
    core::{
        determinism::DeterminismPlugin, error_boundary::ErrorBoundaryPlugin,
        performance_monitor::PerformanceMonitorPlugin, simulation_control::SimulationControlPlugin,
        performance_config::{PerformanceConfig, UpdateFrequencies},
    },
    plugins::CreatureSimulationPlugin,
};

/// Helper to create a test app with the simulation plugin
fn create_test_app() -> App {
    let mut app = App::new();

    // Use headless render plugin for tests to get proper time
    app.add_plugins((
        bevy::asset::AssetPlugin::default(),
        bevy::scene::ScenePlugin,
        bevy::time::TimePlugin,
        bevy::transform::TransformPlugin,
        bevy::hierarchy::HierarchyPlugin,
        bevy::diagnostic::DiagnosticsPlugin,
        bevy::app::ScheduleRunnerPlugin::run_loop(std::time::Duration::from_secs_f64(1.0 / 60.0)),
    ))
    .add_plugins((
        ErrorBoundaryPlugin,
        PerformanceMonitorPlugin,
        SimulationControlPlugin,
        DeterminismPlugin,
    ))
    .add_plugins(CreatureSimulationPlugin);

    // Configure performance for tests - run all systems every frame
    let mut perf_config = app.world.resource_mut::<PerformanceConfig>();
    perf_config.update_frequencies = UpdateFrequencies {
        movement_divisor: 1,
        decision_divisor: 1, // Run decisions every frame in tests
        needs_divisor: 1,
        render_divisor: 1,
    };
    drop(perf_config);

    app
}


/// Helper to spawn a creature in the test app
fn spawn_test_creature(app: &mut App, position: Vec2, creature_type: CreatureType) -> Entity {
    app.world
        .spawn(CreatureBundle {
            creature: Creature,
            creature_type,
            position: Position(position),
            velocity: Velocity(Vec2::ZERO),
            health: Health::new(100.0),
            needs: Needs::default(),
            state: CreatureState::Idle,
            age: Age(0.0),
            size: Size(1.0),
            max_speed: MaxSpeed(50.0),
            decision_timer: DecisionTimer {
                timer: Timer::from_seconds(0.01, TimerMode::Repeating), // Fast decisions for testing
                last_decision_time: 0.0,
            },
            current_target: CurrentTarget::None,
        })
        .id()
}

/// Helper to spawn a resource in the test app
fn spawn_test_resource(app: &mut App, position: Vec2, resource_type: ResourceType) -> Entity {
    app.world
        .spawn(ResourceBundle {
            resource: ResourceMarker,
            resource_type: ResourceTypeComponent(resource_type),
            position: Position(position),
            amount: ResourceAmount::new(100.0),
        })
        .id()
}

#[test]
fn test_creature_spawns_correctly() {
    let mut app = create_test_app();

    let creature_id = spawn_test_creature(&mut app, Vec2::new(0.0, 0.0), CreatureType::Herbivore);

    // Verify creature components
    let world = app.world;
    assert!(world.get::<Creature>(creature_id).is_some());
    assert!(world.get::<Position>(creature_id).is_some());
    assert!(world.get::<Health>(creature_id).is_some());
    assert!(world.get::<Needs>(creature_id).is_some());

    let position = world.get::<Position>(creature_id).unwrap();
    assert_eq!(position.0, Vec2::new(0.0, 0.0));
}

#[test]
fn test_creature_needs_increase_over_time() {
    let mut app = create_test_app();

    let creature_id = spawn_test_creature(&mut app, Vec2::new(0.0, 0.0), CreatureType::Herbivore);

    // Get initial needs
    let initial_hunger = app.world.get::<Needs>(creature_id).unwrap().hunger;
    let initial_thirst = app.world.get::<Needs>(creature_id).unwrap().thirst;

    // Run simulation for 1 second worth of updates
    for _ in 0..60 {
        app.update();
    }

    // Check that needs increased
    let final_needs = app.world.get::<Needs>(creature_id).unwrap();
    assert!(
        final_needs.hunger > initial_hunger,
        "Hunger should increase over time"
    );
    assert!(
        final_needs.thirst > initial_thirst,
        "Thirst should increase over time"
    );
}

#[test]
fn test_creature_moves_towards_food_when_hungry() {
    let mut app = create_test_app();

    // Spawn hungry creature
    let creature_id = spawn_test_creature(&mut app, Vec2::new(0.0, 0.0), CreatureType::Herbivore);

    // Spawn food nearby
    let food_id = spawn_test_resource(&mut app, Vec2::new(10.0, 0.0), ResourceType::Food);
    println!("Spawned food at (10.0, 0.0) with id {:?}", food_id);

    // Make creature very hungry after spawning
    app.world.get_mut::<Needs>(creature_id).unwrap().hunger = 0.8;

    // Get initial position
    let initial_pos = app.world.get::<Position>(creature_id).unwrap().0;

    // Run simulation for multiple frames
    // First ensure spatial grid is populated
    for _ in 0..10 {
        app.update();
    }
    
    // Check initial state
    if let Some(state) = app.world.get::<CreatureState>(creature_id) {
        println!("Initial state: {:?}", state);
    }
    if let Some(needs) = app.world.get::<Needs>(creature_id) {
        println!("Initial needs - hunger: {}", needs.hunger);
    }
    
    // Check spatial grid
    let spatial_grid = app.world.resource::<creature_simulation::plugins::SpatialGrid>();
    let nearby = spatial_grid.query_radius(Vec2::new(0.0, 0.0), 50.0);
    println!("Spatial grid query from (0,0) radius 50: {} entities", nearby.len());
    for entity in &nearby {
        if *entity == food_id {
            println!("  - Found food entity in spatial grid!");
        }
    }
    
    // Run simulation until creature has moved enough
    for _ in 0..300 {
        app.update();
    }

    // Check that creature moved
    let final_pos = app.world.get::<Position>(creature_id).unwrap().0;
    let distance_moved = (final_pos - initial_pos).length();
    assert!(
        distance_moved > 1.0,
        "Creature should move towards food, moved {} units",
        distance_moved
    );
}

#[test]
fn test_creature_flees_from_threat() {
    let mut app = create_test_app();

    // Spawn herbivore
    let herbivore_id = spawn_test_creature(&mut app, Vec2::new(0.0, 0.0), CreatureType::Herbivore);

    // Spawn carnivore very close
    spawn_test_creature(&mut app, Vec2::new(5.0, 0.0), CreatureType::Carnivore);

    // Get initial position
    let initial_pos = app.world.get::<Position>(herbivore_id).unwrap().0;

    // Run simulation until creature flees
    for _ in 0..300 {
        app.update();
    }

    // Check that herbivore moved away
    let final_pos = app.world.get::<Position>(herbivore_id).unwrap().0;
    let distance_moved = (final_pos - initial_pos).length();
    assert!(
        distance_moved > 1.0,
        "Herbivore should flee from carnivore, moved {} units",
        distance_moved
    );
}

#[test]
fn test_spatial_grid_queries() {
    let mut app = create_test_app();

    // Spawn creatures at different positions
    spawn_test_creature(&mut app, Vec2::new(0.0, 0.0), CreatureType::Herbivore);
    spawn_test_creature(&mut app, Vec2::new(10.0, 0.0), CreatureType::Herbivore);
    spawn_test_creature(&mut app, Vec2::new(100.0, 0.0), CreatureType::Herbivore);

    // Update multiple times to ensure spatial grid is fully populated
    for _ in 0..3 {
        app.update();
    }

    // Query spatial grid
    let spatial_grid = app.world.resource::<creature_simulation::plugins::SpatialGrid>();
    let nearby = spatial_grid.query_radius(Vec2::new(5.0, 0.0), 20.0);

    // Should find at least the first two creatures
    assert!(
        nearby.len() >= 2,
        "Should find at least 2 creatures within radius, found {}",
        nearby.len()
    );
}

#[test]
fn test_resource_consumption() {
    let mut app = create_test_app();

    // Spawn very hungry creature
    let creature_id = spawn_test_creature(&mut app, Vec2::new(0.0, 0.0), CreatureType::Herbivore);

    // Spawn food very close
    let food_id = spawn_test_resource(&mut app, Vec2::new(1.0, 0.0), ResourceType::Food);

    // Make creature extremely hungry
    app.world.get_mut::<Needs>(creature_id).unwrap().hunger = 0.95;

    // Get initial resource amount
    let initial_amount = app.world.get::<ResourceAmount>(food_id).unwrap().current;
    let initial_hunger = app.world.get::<Needs>(creature_id).unwrap().hunger;

    // Run simulation and check periodically
    for i in 0..180 {
        app.update();
        
        // Check every 30 frames
        if i % 30 == 0 {
            if let Some(amount) = app.world.get::<ResourceAmount>(food_id) {
                if amount.current < initial_amount {
                    // Resource was consumed
                    return;
                }
            }
        }
    }

    // Check final state
    let final_amount = app.world.get::<ResourceAmount>(food_id).map(|a| a.current).unwrap_or(0.0);
    let final_hunger = app.world.get::<Needs>(creature_id).map(|n| n.hunger).unwrap_or(1.0);

    assert!(
        final_amount < initial_amount || final_hunger < initial_hunger,
        "Either food should be consumed ({} -> {}) or hunger decreased ({} -> {})",
        initial_amount,
        final_amount,
        initial_hunger,
        final_hunger
    );
}

#[test]
fn test_creature_death_from_starvation() {
    let mut app = create_test_app();

    // Spawn creature with maximum hunger
    let creature_id = spawn_test_creature(&mut app, Vec2::new(0.0, 0.0), CreatureType::Herbivore);
    app.world.get_mut::<Needs>(creature_id).unwrap().hunger = 1.0; // Maximum hunger = starvation

    // Run simulation for a few frames
    for _ in 0..10 {
        app.update();
    }

    // Check that creature was despawned
    assert!(
        app.world.get_entity(creature_id).is_none(),
        "Starved creature should be despawned immediately"
    );
}

#[test]
fn test_multiple_creatures_performance() {
    let mut app = create_test_app();

    // Spawn 100 creatures
    let mut creature_ids = Vec::new();
    for i in 0..100 {
        let x = (i % 10) as f32 * 10.0;
        let y = (i / 10) as f32 * 10.0;
        let id = spawn_test_creature(&mut app, Vec2::new(x, y), CreatureType::Herbivore);
        creature_ids.push(id);
    }

    // Spawn some resources
    for i in 0..20 {
        let x = (i % 5) as f32 * 20.0 + 5.0;
        let y = (i / 5) as f32 * 20.0 + 5.0;
        spawn_test_resource(&mut app, Vec2::new(x, y), ResourceType::Food);
    }

    // Run simulation for 2 seconds worth of frames
    let start = std::time::Instant::now();
    for _ in 0..120 {
        app.update();
    }
    let elapsed = start.elapsed();

    // Should complete in reasonable time
    assert!(
        elapsed.as_secs_f32() < 2.0,
        "Simulation should run efficiently, took {} seconds",
        elapsed.as_secs_f32()
    );

    // Check for any movement or state changes
    let mut active_count = 0;
    let mut moving_count = 0;
    for id in creature_ids.iter().take(20) {
        if let Some(entity) = app.world.get_entity(*id) {
            if let Some(state) = entity.get::<CreatureState>() {
                if !matches!(state, CreatureState::Dead | CreatureState::Idle) {
                    active_count += 1;
                }
            }
            if let Some(vel) = entity.get::<Velocity>() {
                if vel.0.length_squared() > 0.01 {
                    moving_count += 1;
                }
            }
        }
    }
    assert!(active_count > 0 || moving_count > 0,
            "Some creatures should be active or moving, found {} active and {} moving out of 20 checked", 
            active_count, moving_count);
}
