use bevy::prelude::*;

/// Marker component for creature sprites
#[derive(Component)]
pub struct CreatureSprite;

/// Marker component for resource sprites
#[derive(Component)]
pub struct ResourceSprite;

/// Component that links a Bevy entity to a simulation entity
#[derive(Component)]
pub struct SimulationEntity {
    pub id: crate::core::entity::Entity,
}

/// Component for isometric sprite rendering
#[derive(Component)]
pub struct IsometricSprite {
    pub z_offset: f32,
    pub sort_offset: f32,
}

impl Default for IsometricSprite {
    fn default() -> Self {
        Self {
            z_offset: 0.0,
            sort_offset: 0.0,
        }
    }
}

/// Component for animated sprites
#[derive(Component)]
pub struct AnimatedSprite {
    pub frames: Vec<usize>,
    pub current_frame: usize,
    pub timer: Timer,
    pub looping: bool,
}

impl AnimatedSprite {
    pub fn new(frames: Vec<usize>, frame_time: f32, looping: bool) -> Self {
        Self {
            frames,
            current_frame: 0,
            timer: Timer::from_seconds(frame_time, TimerMode::Repeating),
            looping,
        }
    }
}

/// Component for health bar display
#[derive(Component)]
pub struct HealthBar {
    pub width: f32,
    pub height: f32,
    pub offset: Vec2,
}

impl Default for HealthBar {
    fn default() -> Self {
        Self {
            width: 30.0,
            height: 4.0,
            offset: Vec2::new(0.0, 20.0),
        }
    }
}

/// Component for debug visualization
#[derive(Component)]
pub struct DebugVisualization {
    pub show_id: bool,
    pub show_state: bool,
    pub show_needs: bool,
}

impl Default for DebugVisualization {
    fn default() -> Self {
        Self {
            show_id: false,
            show_state: false,
            show_needs: false,
        }
    }
}