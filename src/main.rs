use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use creature_simulation::core::{
    determinism::DeterminismPlugin, error_boundary::ErrorBoundaryPlugin,
    memory_profiler::MemoryProfilerPlugin, performance_monitor::PerformanceMonitorPlugin, 
    simulation_control::SimulationControlPlugin,
};
use creature_simulation::plugins::{
    CameraPlugin, CreatureSimulationPlugin, DebugPlugin, DebugConsolePlugin, RenderingPlugin, 
    SelectionPlugin, UiEguiPlugin, VisualProfilerPlugin,
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
            MemoryProfilerPlugin,
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
            DebugConsolePlugin,
            VisualProfilerPlugin, // Performance overlay (F9 to toggle)
        ))
        .run();
}
