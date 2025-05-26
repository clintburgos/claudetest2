//! Bevy plugins for the creature simulation.
//!
//! This module provides modular plugins that can be added to a Bevy app
//! to enable different aspects of the simulation.

use bevy::prelude::*;

pub use camera::{CameraPlugin, CameraState};
pub use debug::DebugPlugin;
pub use debug_console::DebugConsolePlugin;
pub use rendering::RenderingPlugin;
pub use selection::SelectionPlugin;
pub use simulation::SimulationPlugin;
pub use spatial::{SpatialGrid, SpatialPlugin};
pub use spawn::SpawnPlugin;
pub use ui::UiPlugin;
pub use ui_egui::{UiEguiPlugin, UiState};
pub use visual_profiler::VisualProfilerPlugin;

mod camera;
mod debug;
mod debug_console;
mod rendering;
mod selection;
mod simulation;
mod spatial;
mod spawn;
mod ui;
mod ui_egui;
mod visual_profiler;

/// Main plugin that includes all creature simulation functionality
pub struct CreatureSimulationPlugin;

impl Plugin for CreatureSimulationPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add sub-plugins
            .add_plugins((
                SpatialPlugin,
                SimulationPlugin,
                SpawnPlugin,
            ))
            // Add global resources
            .init_resource::<SimulationSettings>()
            // Add global events
            .add_event::<CreatureSpawnedEvent>()
            .add_event::<CreatureDiedEvent>()
            .add_event::<ResourceConsumedEvent>()
            .add_event::<ResourceDepletedEvent>();
    }
}

/// Global simulation settings
#[derive(Resource, Debug, Clone, PartialEq)]
pub struct SimulationSettings {
    pub paused: bool,
    pub speed_multiplier: f32,
    pub world_bounds: Option<(Vec2, Vec2)>,
}

impl Default for SimulationSettings {
    fn default() -> Self {
        Self {
            paused: false,
            speed_multiplier: 1.0,
            world_bounds: Some((Vec2::new(-500.0, -500.0), Vec2::new(500.0, 500.0))),
        }
    }
}

/// Event emitted when a creature is spawned
#[derive(Event, Debug)]
pub struct CreatureSpawnedEvent {
    pub entity: Entity,
    pub position: Vec2,
}

/// Event emitted when a creature dies
#[derive(Event, Debug)]
pub struct CreatureDiedEvent {
    pub entity: Entity,
    pub cause: DeathCause,
}

/// Event emitted when a resource is consumed
#[derive(Event, Debug)]
pub struct ResourceConsumedEvent {
    pub creature: Entity,
    pub resource: Entity,
    pub amount: f32,
}

/// Event emitted when a resource is depleted
#[derive(Event, Debug)]
pub struct ResourceDepletedEvent {
    pub entity: Entity,
}

/// Cause of creature death
#[derive(Debug, Clone, Copy)]
pub enum DeathCause {
    Starvation,
    Dehydration,
    Exhaustion,
    OldAge,
}
