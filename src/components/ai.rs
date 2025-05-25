//! AI and decision-making components

use bevy::prelude::*;

/// Timer for making decisions
#[derive(Component, Debug)]
pub struct DecisionTimer {
    pub timer: Timer,
    pub last_decision_time: f32,
}

impl Default for DecisionTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.5, TimerMode::Repeating),
            last_decision_time: 0.0,
        }
    }
}

/// Current target or goal
#[derive(Component, Debug, Clone)]
pub enum CurrentTarget {
    None,
    Position(Vec2),
    Entity(Entity),
}

impl Default for CurrentTarget {
    fn default() -> Self {
        Self::None
    }
}

/// Cached decision for performance
#[derive(Component, Debug, Clone)]
pub struct CachedDecision {
    pub decision: Decision,
    pub hash: u64,
}

/// Possible decisions a creature can make
#[derive(Debug, Clone, PartialEq)]
pub enum Decision {
    Idle,
    Move {
        target: Vec2,
        urgency: f32,
    },
    Consume {
        resource: Entity,
        resource_type: crate::simulation::ResourceType,
    },
    Rest {
        duration: f32,
    },
    Flee {
        from: Vec2,
        urgency: f32,
    },
    Socialize {
        with: Entity,
    },
}
