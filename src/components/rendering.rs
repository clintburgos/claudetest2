use bevy::prelude::*;

/// Component for isometric sprite rendering
/// 
/// Controls sprite positioning and sorting in isometric view
#[derive(Component)]
pub struct IsometricSprite {
    /// Additional Z-axis offset for rendering order (in world units)
    /// Positive values move sprite forward (rendered on top)
    pub z_offset: f32,
    /// Fine-tuning for sort order within same depth layer
    /// Used to resolve sorting conflicts between overlapping sprites
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
/// 
/// Represents vertical position in world space (Y-axis)
/// Units: 1.0 = one tile height (32 pixels in screen space)
/// Used for flying creatures, jumping, or multi-level terrain
#[derive(Component)]
pub struct IsometricHeight(pub f32);

impl Default for IsometricHeight {
    fn default() -> Self {
        Self(0.0)
    }
}

/// Component for animated sprites
/// 
/// Manages frame-based animation playback for texture atlas sprites.
/// The animation system updates the TextureAtlas index based on this component.
#[derive(Component)]
pub struct AnimatedSprite {
    /// List of texture atlas indices that make up this animation
    pub frames: Vec<usize>,
    /// Current frame being displayed (index into frames vector)
    pub current_frame: usize,
    /// Timer controlling frame advancement speed
    pub timer: Timer,
    /// Whether animation should restart after reaching last frame
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
/// 
/// Configures the visual health indicator above creatures
#[derive(Component)]
pub struct HealthBar {
    /// Width of health bar in pixels
    pub width: f32,
    /// Height of health bar in pixels
    pub height: f32,
    /// Position offset from entity center (in screen space pixels)
    /// X: horizontal offset (positive = right)
    /// Y: vertical offset (positive = up)
    pub offset: Vec2,
}

impl Default for HealthBar {
    fn default() -> Self {
        Self {
            width: 30.0,   // Standard width for visibility
            height: 4.0,   // Thin bar to not obscure sprite
            offset: Vec2::new(0.0, 20.0), // Positioned above 48px sprite
        }
    }
}

/// Component for debug visualization
/// 
/// Controls which debug overlays are shown for this entity.
/// Used with F1-F4 debug hotkeys to display development information.
#[derive(Component, Default)]
pub struct DebugVisualization {
    /// Display entity ID number above sprite
    pub show_id: bool,
    /// Display current AI state (Idle, Moving, Eating, etc.)
    pub show_state: bool,
    /// Display need values (hunger, thirst, energy, social)
    pub show_needs: bool,
}

/// Cartoon sprite component for enhanced visuals
/// 
/// Main component for the cartoon rendering system. Manages all visual
/// aspects of a creature including animations, expressions, and customization.
#[derive(Component)]
pub struct CartoonSprite {
    /// Current animation state (walk, run, eat, etc.)
    pub base_animation: AnimationState,
    /// Optional facial expression overlay for emotions
    pub expression_overlay: Option<ExpressionOverlay>,
    /// Genetic modifications to appearance (size, color, patterns)
    pub body_modifiers: BodyModifiers,
    /// Tools and decorations attached to the creature
    pub accessory_slots: Vec<Accessory>,
    /// Shadow position offset from sprite base (in pixels)
    pub shadow_offset: Vec2,
    /// Anchor point for sprite positioning (in pixels from top-left)
    /// Used to align sprite's "feet" with world position
    pub sprite_anchor: Vec2,
}

impl Default for CartoonSprite {
    fn default() -> Self {
        Self {
            base_animation: AnimationState::Idle,
            expression_overlay: None,
            body_modifiers: BodyModifiers::default(),
            accessory_slots: Vec::new(),
            shadow_offset: Vec2::new(0.0, -8.0), // Shadow 8px below sprite
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

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
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

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
pub enum AccessoryType {
    Tool(ToolType),
    Decoration(DecorationType),
}

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
pub enum ToolType {
    Stick,
    Stone,
    Spear,
    Basket,
}

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
pub enum DecorationType {
    Flower,
    Feather,
    Shell,
    Leaf,
}

/// Emotion types for expression system
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
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
    Excited,
    Disgusted,
    Surprised,
    Confused,
    Sleeping,
    Sick,
    Love,
}
