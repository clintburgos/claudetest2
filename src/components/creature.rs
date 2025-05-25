//! Creature-specific components

use bevy::prelude::*;

/// Marker component for creature entities
#[derive(Component, Debug, Default)]
pub struct Creature;

/// Age of the creature in seconds
#[derive(Component, Debug, Default)]
pub struct Age(pub f32);

/// Size of the creature (affects metabolism and speed)
#[derive(Component, Debug)]
pub struct Size(pub f32);

impl Default for Size {
    fn default() -> Self {
        Self(1.0)
    }
}

/// Current state of the creature
#[derive(Component, Debug, Clone, PartialEq)]
pub enum CreatureState {
    Idle,
    Moving { target: Vec2 },
    Eating,
    Drinking,
    Resting,
    Dead,
}

impl Default for CreatureState {
    fn default() -> Self {
        Self::Idle
    }
}

/// Tracks how long creature has been in current state
#[derive(Component, Debug, Default)]
pub struct StateDuration(pub f32);

/// Creature type for predator/prey dynamics
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub enum CreatureType {
    Herbivore,
    Carnivore,
    Omnivore,
}

impl Default for CreatureType {
    fn default() -> Self {
        Self::Herbivore
    }
}
