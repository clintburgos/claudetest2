//! Test example for Phase 1 Week 7-8 UI implementation

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use creature_simulation::{components::*, plugins::*};

fn main() {
    App::new()
        // Core Bevy plugins
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Creature Simulation - UI Test".to_string(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        // Third-party plugins
        .add_plugins(EguiPlugin)
        // Our custom plugins
        .add_plugins((
            CreatureSimulationPlugin,
            CameraPlugin,
            RenderingPlugin,
            SelectionPlugin,
            UiEguiPlugin,
            DebugPlugin,
        ))
        // Enable diagnostics
        .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        // Add test spawning system
        .add_systems(Startup, spawn_test_entities)
        .run();
}

fn spawn_test_entities(mut commands: Commands) {
    // Spawn some test creatures in different states
    let positions = [
        Vec2::new(-100.0, -100.0),
        Vec2::new(100.0, -100.0),
        Vec2::new(-100.0, 100.0),
        Vec2::new(100.0, 100.0),
        Vec2::new(0.0, 0.0),
    ];

    let types = [
        CreatureType::Herbivore,
        CreatureType::Carnivore,
        CreatureType::Omnivore,
        CreatureType::Herbivore,
        CreatureType::Carnivore,
    ];

    for (i, (pos, creature_type)) in positions.iter().zip(types.iter()).enumerate() {
        let mut bundle = CreatureBundle::new(*pos, 1.0 + (i as f32) * 0.2);
        bundle.creature_type = *creature_type;

        // Vary their states
        bundle.state = match i {
            0 => CreatureState::Eating,
            1 => CreatureState::Drinking,
            2 => CreatureState::Resting,
            3 => CreatureState::Moving {
                target: Vec2::new(200.0, 200.0),
            },
            _ => CreatureState::Idle,
        };

        // Vary their needs
        bundle.needs.hunger = (i as f32) * 0.2;
        bundle.needs.thirst = (i as f32) * 0.15;
        bundle.needs.energy = (i as f32) * 0.1;

        // One creature with low health
        if i == 2 {
            bundle.health.current = 30.0;
        }

        commands.spawn(bundle);
    }

    // Spawn some resources
    let resource_positions = [
        (Vec2::new(-50.0, 0.0), ResourceType::Food),
        (Vec2::new(50.0, 0.0), ResourceType::Water),
        (Vec2::new(0.0, -50.0), ResourceType::Food),
        (Vec2::new(0.0, 50.0), ResourceType::Water),
    ];

    for (pos, res_type) in resource_positions {
        commands.spawn(ResourceBundle::new(pos, res_type, 50.0));
    }

    println!("Spawned test entities!");
    println!("Controls:");
    println!("- WASD/Arrows: Move camera");
    println!("- Q/E or Mouse Wheel: Zoom");
    println!("- Middle/Right Mouse: Pan camera");
    println!("- Left Click: Select creature");
    println!("- ESC: Deselect/Stop following");
    println!("- Space: Pause/Unpause");
    println!("- Number keys 1-5: Change speed");
}
