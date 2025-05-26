use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use creature_simulation::core::{
    determinism::DeterminismPlugin, error_boundary::ErrorBoundaryPlugin,
    performance_monitor::PerformanceMonitorPlugin, simulation_control::SimulationControlPlugin,
};
use creature_simulation::plugins::{
    CameraPlugin, CreatureSimulationPlugin, DebugPlugin, RenderingPlugin, SelectionPlugin,
    UiEguiPlugin,
};

fn main() {
    App::new()
        // Core Bevy plugins
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Creature Simulation".to_string(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        // Third-party plugins
        .add_plugins(EguiPlugin)
        // Core systems (must be added first)
        .add_plugins((
            ErrorBoundaryPlugin,
            PerformanceMonitorPlugin,
            SimulationControlPlugin,
            DeterminismPlugin,
        ))
        // Our custom plugins
        .add_plugins((
            CreatureSimulationPlugin, // Includes Simulation, Spatial, and Spawn plugins
            CameraPlugin,
            RenderingPlugin,
            SelectionPlugin,
            UiEguiPlugin, // Using egui version
            DebugPlugin,
        ))
        .run();
}
