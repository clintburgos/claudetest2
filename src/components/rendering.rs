use bevy::prelude::*;

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

/// Component for entity height in isometric rendering
#[derive(Component)]
pub struct IsometricHeight(pub f32);

impl Default for IsometricHeight {
    fn default() -> Self {
        Self(0.0)
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
#[derive(Component, Default)]
pub struct DebugVisualization {
    pub show_id: bool,
    pub show_state: bool,
    pub show_needs: bool,
}

/// Cartoon sprite component for enhanced visuals
#[derive(Component)]
pub struct CartoonSprite {
    pub base_animation: AnimationState,
    pub expression_overlay: Option<ExpressionOverlay>,
    pub body_modifiers: BodyModifiers,
    pub accessory_slots: Vec<Accessory>,
    pub shadow_offset: Vec2,
    pub sprite_anchor: Vec2,
}

impl Default for CartoonSprite {
    fn default() -> Self {
        Self {
            base_animation: AnimationState::Idle,
            expression_overlay: None,
            body_modifiers: BodyModifiers::default(),
            accessory_slots: Vec::new(),
            shadow_offset: Vec2::new(0.0, -8.0),
            sprite_anchor: Vec2::new(24.0, 48.0), // Bottom center for 48x48 sprite
        }
    }
}

/// Animation states for creatures
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationState {
    Idle,
    Walk,
    Run,
    Eat,
    Sleep,
    Talk,
    Attack,
    Death,
    Special(SpecialAnimation),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpecialAnimation {
    Happy,
    Sad,
    Angry,
    Curious,
}

/// Expression overlay for facial features
#[derive(Clone, Debug)]
pub struct ExpressionOverlay {
    pub eye_offset: Vec2,
    pub eye_scale: f32,
    pub mouth_curve: f32,
    pub mouth_open: f32,
    pub brow_angle: f32,
}

/// Body modifiers based on genetics
#[derive(Component, Clone, Debug)]
pub struct BodyModifiers {
    pub size_scale: f32,
    pub color_tint: Color,
    pub pattern_type: PatternType,
    pub ear_size: f32,
    pub tail_length: f32,
}

impl Default for BodyModifiers {
    fn default() -> Self {
        Self {
            size_scale: 1.0,
            color_tint: Color::WHITE,
            pattern_type: PatternType::None,
            ear_size: 1.0,
            tail_length: 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PatternType {
    None,
    Spots,
    Stripes,
    Patches,
}

/// Accessory/tool attachment
#[derive(Clone, Debug)]
pub struct Accessory {
    pub accessory_type: AccessoryType,
    pub position_offset: Vec2,
    pub rotation: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AccessoryType {
    Tool(ToolType),
    Decoration(DecorationType),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ToolType {
    Stick,
    Stone,
    Spear,
    Basket,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DecorationType {
    Flower,
    Feather,
    Shell,
    Leaf,
}

/// Emotion types for expression system
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EmotionType {
    Neutral,
    Happy,
    Sad,
    Angry,
    Frightened,
    Curious,
    Tired,
    Hungry,
    Content,
}
