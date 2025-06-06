//! Bevy ECS Components for the creature simulation.
//!
//! This module defines all components used in the ECS architecture.
//! Components are grouped into bundles for convenient entity spawning.

use bevy::prelude::*;

// Re-export all components
pub use ai::*;
pub use creature::*;
pub use health::*;
pub use movement::*;
pub use needs::{NeedType, Needs};
pub use rendering::*;
pub use resource::*;

mod ai;
mod creature;
mod health;
mod movement;
mod needs;
mod rendering;
mod resource;

/// Bundle for spawning a complete creature entity
#[derive(Bundle)]
pub struct CreatureBundle {
    // Core components
    pub creature: Creature,
    pub creature_type: CreatureType,
    pub position: Position,
    pub velocity: Velocity,

    // State components
    pub health: Health,
    pub needs: Needs,
    pub state: CreatureState,
    pub age: Age,
    pub size: Size,
    pub genetics: Genetics,

    // Movement components
    pub max_speed: MaxSpeed,

    // AI components
    pub decision_timer: DecisionTimer,
    pub current_target: CurrentTarget,
}

/// Bundle for spawning a resource entity
#[derive(Bundle)]
pub struct ResourceBundle {
    // Core components
    pub resource: ResourceMarker,
    pub position: Position,
    pub resource_type: ResourceTypeComponent,

    // State components
    pub amount: ResourceAmount,
}

impl CreatureBundle {
    /// Creates a new creature bundle at the specified position
    pub fn new(position: Vec2, size: f32) -> Self {
        Self {
            creature: Creature,
            creature_type: CreatureType::default(),
            position: Position(position),
            velocity: Velocity(Vec2::ZERO),
            health: Health::new(100.0),
            needs: Needs::default(),
            state: CreatureState::Idle,
            age: Age(0.0),
            size: Size(size),
            genetics: Genetics::default(),
            max_speed: MaxSpeed::default(),
            decision_timer: DecisionTimer::default(),
            current_target: CurrentTarget::None,
        }
    }
}

impl ResourceBundle {
    /// Creates a new resource bundle
    pub fn new(position: Vec2, resource_type: ResourceType, amount: f32) -> Self {
        Self {
            resource: ResourceMarker,
            position: Position(position),
            resource_type: ResourceTypeComponent(resource_type),
            amount: ResourceAmount::new(amount),
        }
    }
}
