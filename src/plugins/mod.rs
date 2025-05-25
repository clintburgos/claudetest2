//! Bevy plugins for the creature simulation.
//!
//! This module provides modular plugins that can be added to a Bevy app
//! to enable different aspects of the simulation.

use bevy::prelude::*;

pub use simulation::SimulationPlugin;
pub use spatial::{SpatialPlugin, SpatialGrid};
pub use debug::DebugPlugin;

mod simulation;
mod spatial;
mod debug;

/// Main plugin that includes all creature simulation functionality
pub struct CreatureSimulationPlugin;

impl Plugin for CreatureSimulationPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add sub-plugins
            .add_plugins((
                SimulationPlugin,
                SpatialPlugin,
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
    pub time_scale: f32,
    pub world_bounds: Option<(Vec2, Vec2)>,
}

impl Default for SimulationSettings {
    fn default() -> Self {
        Self {
            paused: false,
            time_scale: 1.0,
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