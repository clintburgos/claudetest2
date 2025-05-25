//! Example of running the creature simulation with Bevy ECS

use bevy::app::AppExit;
use bevy::prelude::*;
use creature_simulation::{components::*, plugins::*, simulation::ResourceType};

fn main() {
    App::new()
        // Bevy default plugins (excluding render for headless operation)
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: None,
                    ..default()
                })
                .build()
                .disable::<bevy::render::RenderPlugin>()
                .disable::<bevy::winit::WinitPlugin>()
                .disable::<bevy::a11y::AccessibilityPlugin>()
        )
        // Our simulation plugin
        .add_plugins(CreatureSimulationPlugin)
        // Setup system
        .add_systems(Startup, setup)
        // Example system to stop after 5 seconds
        .add_systems(Update, check_stop_condition)
        .run();
}

fn setup(mut commands: Commands) {
    // Spawn some creatures
    for i in 0..10 {
        let x = (i as f32) * 100.0 - 450.0;
        commands.spawn(CreatureBundle::new(Vec2::new(x, 0.0), 1.0));
    }

    // Spawn some resources
    for i in 0..20 {
        let x = (i as f32 % 5.0) * 200.0 - 400.0;
        let y = (i as f32 / 5.0).floor() * 200.0 - 400.0;

        let resource_type = if i % 2 == 0 { ResourceType::Food } else { ResourceType::Water };

        commands.spawn(ResourceBundle::new(Vec2::new(x, y), resource_type, 100.0));
    }

    info!("Simulation started with 10 creatures and 20 resources");
}

fn check_stop_condition(time: Res<Time>, mut exit: EventWriter<AppExit>) {
    if time.elapsed_seconds() > 5.0 {
        info!("Simulation complete after 5 seconds");
        exit.send(AppExit);
    }
}
