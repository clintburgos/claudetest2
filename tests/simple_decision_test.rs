//! Simple test to verify decision system works with enough updates

use bevy::prelude::*;
use creature_simulation::{
    components::*,
    core::{
        determinism::DeterminismPlugin, error_boundary::ErrorBoundaryPlugin,
        performance_monitor::PerformanceMonitorPlugin, simulation_control::SimulationControlPlugin,
    },
    plugins::*,
};

#[test]
fn test_decision_system_with_many_updates() {
    let mut app = App::new();

    // Add plugins
    app.add_plugins(MinimalPlugins)
        .add_plugins((
            ErrorBoundaryPlugin,
            PerformanceMonitorPlugin,
            SimulationControlPlugin,
            DeterminismPlugin,
        ))
        .add_plugins(CreatureSimulationPlugin);

    // Make sure simulation is not paused
    {
        let mut settings = app.world.resource_mut::<SimulationSettings>();
        settings.paused = false;
    }

    // Spawn a hungry creature with a fast decision timer
    let creature_id = app
        .world
        .spawn(CreatureBundle {
            creature: Creature,
            creature_type: CreatureType::Herbivore,
            position: Position(Vec2::new(0.0, 0.0)),
            velocity: Velocity(Vec2::ZERO),
            health: Health::new(100.0),
            needs: Needs {
                hunger: 0.9, // Very hungry
                thirst: 0.0,
                energy: 1.0,
                social: 0.0,
            },
            state: CreatureState::Idle,
            age: Age(0.0),
            size: Size(1.0),
            genetics: Genetics::default(),
            max_speed: MaxSpeed(50.0),
            decision_timer: DecisionTimer {
                timer: Timer::from_seconds(0.01, TimerMode::Repeating), // Very fast timer (10ms)
                last_decision_time: 0.0,
            },
            current_target: CurrentTarget::None,
        })
        .id();

    // Spawn food nearby
    app.world.spawn(ResourceBundle {
        resource: ResourceMarker,
        position: Position(Vec2::new(5.0, 0.0)),
        resource_type: ResourceTypeComponent(ResourceType::Food),
        amount: ResourceAmount::new(100.0),
    });

    // Also spawn the spatial grid entity if it doesn't exist
    app.world.insert_resource(creature_simulation::plugins::SpatialGrid::new(50.0));

    let initial_state = app.world.get::<CreatureState>(creature_id).unwrap().clone();
    let initial_pos = app.world.get::<Position>(creature_id).unwrap().0;

    println!(
        "Initial state: {:?}, position: {:?}",
        initial_state, initial_pos
    );

    // Run MANY updates to accumulate enough real time
    let mut state_changed = false;
    for i in 0..1000 {
        app.update();

        if let Some(entity) = app.world.get_entity(creature_id) {
            let state = entity.get::<CreatureState>().unwrap();
            let timer = entity.get::<DecisionTimer>().unwrap();

            // Print progress every 100 frames
            if i % 100 == 0 {
                let time = app.world.resource::<Time>();
                println!(
                    "Frame {}: elapsed={:.6}s, timer={:.6}s, state={:?}",
                    i,
                    time.elapsed_seconds(),
                    timer.timer.elapsed_secs(),
                    state
                );
            }

            // Check if state changed
            if !matches!(state, CreatureState::Idle) {
                println!("SUCCESS: State changed to {:?} at frame {}", state, i);
                state_changed = true;
                break;
            }
        } else {
            panic!("Creature despawned!");
        }

        // Add a small delay to accumulate real time
        std::thread::sleep(std::time::Duration::from_micros(100));
    }

    assert!(
        state_changed,
        "Creature should have made a decision after 1000 updates!"
    );
}

#[test]
fn test_timer_ticking() {
    let mut app = App::new();

    app.add_plugins(MinimalPlugins);

    // Simple timer test
    let mut timer = Timer::from_seconds(0.01, TimerMode::Repeating);
    let mut triggered = false;

    for i in 0..1000 {
        app.update();

        let time = app.world.resource::<Time>();
        timer.tick(time.delta());

        if timer.just_finished() {
            println!(
                "Timer triggered at frame {} (elapsed: {:.6}s)",
                i,
                time.elapsed_seconds()
            );
            triggered = true;
            break;
        }

        // Add small delay
        std::thread::sleep(std::time::Duration::from_micros(100));
    }

    assert!(triggered, "Timer should have triggered!");
}
