//! Movement and position components

use bevy::prelude::*;

/// World position of an entity
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Position(pub Vec2);

/// Velocity of an entity
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Velocity(pub Vec2);

/// Maximum movement speed
#[derive(Component, Debug)]
pub struct MaxSpeed(pub f32);

impl Default for MaxSpeed {
    fn default() -> Self {
        Self(50.0)
    }
}

/// Steering force for smooth movement
#[derive(Component, Debug, Default)]
pub struct SteeringForce(pub Vec2);