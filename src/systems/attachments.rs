//! Attachment point system for tools and accessories on creatures
//!
//! This system allows creatures to have items attached to specific body parts (hands, head, back, etc.)
//! that move and animate synchronously with the creature's animations. It supports:
//! 
//! - Dynamic attachment points that follow creature animations
//! - Interpolated movement between animation keyframes
//! - Automatic flipping when creatures change direction
//! - Depth ordering for proper visual layering
//! - Different attachment behaviors for tools vs decorations
//!
//! # Architecture
//!
//! The system uses a parent-child entity relationship where attached items are children of the creature
//! entity. Each attachment point has predefined animation offsets that are interpolated based on the
//! current animation frame, creating smooth, natural movement.
//!
//! # Example
//!
//! ```rust
//! // Spawn a stick tool in the creature's right hand
//! let stick_entity = spawn_attached_item(
//!     &mut commands,
//!     creature_entity,
//!     ItemType::Tool(ToolType::Stick),
//!     "right_hand",
//!     stick_texture,
//! );
//! ```

use bevy::prelude::*;
use std::collections::HashMap;
use crate::components::{
    AnimationState, CartoonSprite, AnimatedSprite, AccessoryType, ToolType, DecorationType,
};

/// Attachment point definition for creature body parts
/// 
/// Each attachment point represents a location on the creature's body where items can be attached.
/// The position and rotation of attached items are automatically updated based on the creature's
/// current animation state.
#[derive(Clone, Debug, Reflect)]
pub struct AttachmentPoint {
    /// Unique identifier for this attachment point (e.g., "head", "left_hand")
    pub name: String,
    
    /// Base position relative to the creature's sprite center in pixels
    /// This is the default position when no animation is playing
    pub base_position: Vec2,
    
    /// Pivot point for rotation, relative to the attachment point's position
    /// Items rotate around this point during animations
    pub rotation_pivot: Vec2,
    
    /// Z-order adjustment for rendering depth
    /// Positive values render in front, negative values render behind the creature
    pub depth_offset: f32,
    
    /// Scale multiplier for attached items at this point
    /// Allows items to be sized appropriately for different body parts
    pub scale_factor: f32,
    
    /// Animation-specific position offsets and rotations
    /// Maps animation types to keyframe data for smooth interpolation
    #[reflect(ignore)]
    pub animation_offsets: HashMap<AnimationType, Vec<AnimationOffset>>,
}

/// Animation-specific offset for attachment points
///
/// Defines how an attachment point should move during a specific frame of an animation.
/// Multiple AnimationOffsets create keyframes that are interpolated for smooth movement.
#[derive(Clone, Debug)]
pub struct AnimationOffset {
    /// The animation frame number this offset applies to
    pub frame: usize,
    
    /// Additional position offset from the base position (in pixels)
    pub position_offset: Vec2,
    
    /// Additional rotation in degrees (positive = clockwise)
    pub rotation: f32,
    
    /// Scale multiplier for this frame (1.0 = no change)
    pub scale: f32,
}

/// Component containing all attachment points for a creature
///
/// This component defines the standard attachment points available on a creature.
/// Each creature type may have slightly different positions, but all use the same
/// attachment point names for consistency.
#[derive(Component, Clone, Reflect)]
#[reflect(Component)]
pub struct CreatureAttachmentPoints {
    /// Head attachment for hats, horns, or thought bubbles
    pub head: AttachmentPoint,
    
    /// Left hand for tools or held items
    pub left_hand: AttachmentPoint,
    
    /// Right hand for primary tools or weapons
    pub right_hand: AttachmentPoint,
    
    /// Back attachment for bags, wings, or capes
    pub back: AttachmentPoint,
    
    /// Waist attachment for belts or hanging items
    pub waist: AttachmentPoint,
    
    /// Optional tail tip for decorations (not all creatures have tails)
    pub tail_tip: Option<AttachmentPoint>,
}

impl Default for CreatureAttachmentPoints {
    fn default() -> Self {
        Self {
            head: AttachmentPoint {
                name: "head".to_string(),
                base_position: Vec2::new(0.0, 20.0),
                rotation_pivot: Vec2::new(0.0, 0.0),
                depth_offset: 0.1,
                scale_factor: 1.0,
                animation_offsets: create_head_animation_offsets(),
            },
            left_hand: AttachmentPoint {
                name: "left_hand".to_string(),
                base_position: Vec2::new(-12.0, 5.0),
                rotation_pivot: Vec2::new(-2.0, 0.0),
                depth_offset: 0.2,
                scale_factor: 0.8,
                animation_offsets: create_left_hand_animation_offsets(),
            },
            right_hand: AttachmentPoint {
                name: "right_hand".to_string(),
                base_position: Vec2::new(12.0, 5.0),
                rotation_pivot: Vec2::new(2.0, 0.0),
                depth_offset: -0.1,
                scale_factor: 0.8,
                animation_offsets: create_right_hand_animation_offsets(),
            },
            back: AttachmentPoint {
                name: "back".to_string(),
                base_position: Vec2::new(0.0, 10.0),
                rotation_pivot: Vec2::new(0.0, 5.0),
                depth_offset: -0.2,
                scale_factor: 1.0,
                animation_offsets: create_back_animation_offsets(),
            },
            waist: AttachmentPoint {
                name: "waist".to_string(),
                base_position: Vec2::new(0.0, 0.0),
                rotation_pivot: Vec2::new(0.0, 0.0),
                depth_offset: 0.05,
                scale_factor: 0.9,
                animation_offsets: HashMap::new(),
            },
            tail_tip: Some(AttachmentPoint {
                name: "tail_tip".to_string(),
                base_position: Vec2::new(-15.0, -5.0),
                rotation_pivot: Vec2::new(-5.0, 0.0),
                depth_offset: -0.15,
                scale_factor: 0.7,
                animation_offsets: create_tail_animation_offsets(),
            }),
        }
    }
}

/// Component for attached items on creatures
///
/// Represents an item that is attached to a creature at a specific attachment point.
/// The item will follow the creature's movements and animations based on its settings.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct AttachedItem {
    /// Name of the attachment point this item is connected to
    pub attachment_point: String,
    
    /// Type of item (tool, decoration, or custom)
    pub item_type: ItemType,
    
    /// Additional offset from the attachment point's position (in pixels)
    pub custom_offset: Vec2,
    
    /// Additional rotation from the attachment point's rotation (in degrees)
    pub custom_rotation: f32,
    
    /// Whether this item should animate with the creature's movements
    pub inherit_animation: bool,
    
    /// Whether this item should flip horizontally when the creature changes direction
    pub flip_with_direction: bool,
}

#[derive(Clone, Debug, PartialEq, Reflect)]
pub enum ItemType {
    Tool(ToolType),
    Decoration(DecorationType),
    Custom(String),
}

/// Animation type enum (matching the animation system)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AnimationType {
    Idle,
    Walk,
    Run,
    Eat,
    Sleep,
    Talk,
    Attack,
    Death,
    UseItem,
}

impl From<AnimationState> for AnimationType {
    fn from(state: AnimationState) -> Self {
        match state {
            AnimationState::Idle => AnimationType::Idle,
            AnimationState::Walk => AnimationType::Walk,
            AnimationState::Run => AnimationType::Run,
            AnimationState::Eat => AnimationType::Eat,
            AnimationState::Sleep => AnimationType::Sleep,
            AnimationState::Talk => AnimationType::Talk,
            AnimationState::Attack => AnimationType::Attack,
            AnimationState::Death => AnimationType::Death,
            AnimationState::Special(_) => AnimationType::Idle,
        }
    }
}

/// System to update attachment transforms based on parent animation
///
/// This system runs every frame to update the position, rotation, and scale of all attached items
/// based on their parent creature's current animation state. It handles:
/// - Interpolating between animation keyframes for smooth movement
/// - Applying creature scale and direction flipping
/// - Maintaining proper depth ordering
///
/// # Performance Note
/// This system uses string-based attachment point lookup which could be optimized
/// with an enum-based approach for better performance with many attachments.
pub fn update_attachment_transforms(
    mut attachments: Query<(&AttachedItem, &mut Transform, &Parent)>,
    creatures: Query<(
        &CreatureAttachmentPoints,
        &CartoonSprite,
        &AnimatedSprite,
        &Transform,
        &GlobalTransform,
    ), Without<AttachedItem>>,
) {
    for (attached, mut attachment_transform, parent) in attachments.iter_mut() {
        if let Ok((points, cartoon_sprite, animated_sprite, creature_transform, creature_global)) = creatures.get(parent.get()) {
            // Find the attachment point by name
            // Falls back to waist for unknown attachment points or missing tail
            let point = match attached.attachment_point.as_str() {
                "head" => &points.head,
                "left_hand" => &points.left_hand,
                "right_hand" => &points.right_hand,
                "back" => &points.back,
                "waist" => &points.waist,
                "tail_tip" => points.tail_tip.as_ref().unwrap_or(&points.waist),
                _ => continue, // Skip unknown attachment points
            };
            
            // Calculate base transform
            let mut position = point.base_position + attached.custom_offset;
            let mut rotation = attached.custom_rotation;
            let mut scale = point.scale_factor;
            
            // Apply animation offsets if enabled
            if attached.inherit_animation {
                let anim_type = AnimationType::from(cartoon_sprite.base_animation);
                if let Some(anim_offsets) = point.animation_offsets.get(&anim_type) {
                    // Find the appropriate offset for current frame
                    let current_frame = animated_sprite.current_frame;
                    
                    // Linear interpolation between keyframes creates smooth movement
                    // This prevents jerky transitions between animation frames
                    if let Some(offset) = interpolate_animation_offset(anim_offsets, current_frame) {
                        position += offset.position_offset;
                        rotation += offset.rotation;
                        scale *= offset.scale;
                    }
                }
            }
            
            // Apply creature scale
            position *= cartoon_sprite.body_modifiers.size_scale;
            
            // Check if creature is facing left (negative X scale)
            let facing_left = creature_transform.scale.x < 0.0;
            if facing_left && attached.flip_with_direction {
                position.x = -position.x;
                rotation = -rotation;
            }
            
            // Apply to transform
            attachment_transform.translation = Vec3::new(
                position.x,
                position.y,
                point.depth_offset,
            );
            attachment_transform.rotation = Quat::from_rotation_z(rotation.to_radians());
            attachment_transform.scale = Vec3::splat(scale);
        }
    }
}

/// System to spawn attached items on creatures
///
/// Creates a new attached item entity as a child of the specified creature.
/// The item will automatically follow the creature and animate based on its attachment settings.
///
/// # Arguments
/// * `commands` - Bevy command buffer for entity spawning
/// * `creature_entity` - The creature entity to attach the item to
/// * `item_type` - Type of item being attached (tool, decoration, etc.)
/// * `attachment_point` - Name of the attachment point (e.g., "right_hand")
/// * `texture` - Handle to the item's sprite texture
///
/// # Returns
/// The entity ID of the newly spawned attached item
pub fn spawn_attached_item(
    commands: &mut Commands,
    creature_entity: Entity,
    item_type: ItemType,
    attachment_point: &str,
    texture: Handle<Image>,
) -> Entity {
    let attached_item = AttachedItem {
        attachment_point: attachment_point.to_string(),
        item_type,
        custom_offset: Vec2::ZERO,
        custom_rotation: 0.0,
        inherit_animation: true,
        flip_with_direction: true,
    };
    
    let item_entity = commands.spawn((
        SpriteBundle {
            texture,
            transform: Transform::default(),
            ..default()
        },
        attached_item,
    )).id();
    
    // Make it a child of the creature
    commands.entity(creature_entity).add_child(item_entity);
    
    item_entity
}

/// Interpolates between animation keyframes to find the offset for a specific frame
///
/// This function performs linear interpolation between defined keyframes to create
/// smooth movement. If the current frame is between two keyframes, it calculates
/// the interpolated position, rotation, and scale.
///
/// # Algorithm
/// 1. Find the keyframes before and after the current frame
/// 2. Calculate the interpolation progress (0.0 to 1.0)
/// 3. Linearly interpolate all properties
///
/// # Returns
/// - Some(AnimationOffset) with interpolated values
/// - None if no keyframes are defined
fn interpolate_animation_offset(
    offsets: &[AnimationOffset],
    current_frame: usize,
) -> Option<AnimationOffset> {
    if offsets.is_empty() {
        return None;
    }
    
    // Find surrounding keyframes for interpolation
    let mut prev_offset: Option<&AnimationOffset> = None;
    let mut next_offset: Option<&AnimationOffset> = None;
    
    for offset in offsets {
        if offset.frame <= current_frame {
            prev_offset = Some(offset);
        }
        if offset.frame >= current_frame && next_offset.is_none() {
            next_offset = Some(offset);
            break;
        }
    }
    
    match (prev_offset, next_offset) {
        (Some(prev), Some(next)) if prev.frame != next.frame => {
            // Interpolate between keyframes using linear interpolation
            // Calculate normalized progress between the two keyframes (0.0 to 1.0)
            let frame_diff = next.frame - prev.frame;
            let progress = (current_frame - prev.frame) as f32 / frame_diff as f32;
            
            // Create interpolated offset with smooth transitions
            Some(AnimationOffset {
                frame: current_frame,
                position_offset: prev.position_offset.lerp(next.position_offset, progress),
                rotation: prev.rotation + (next.rotation - prev.rotation) * progress,
                scale: prev.scale + (next.scale - prev.scale) * progress,
            })
        }
        (Some(offset), _) | (_, Some(offset)) => Some(offset.clone()),
        _ => None,
    }
}

// Animation offset creation functions
// These functions define the keyframes for each attachment point during different animations.
// The offsets create natural, cartoon-like movements that enhance the creature's personality.

/// Creates animation offsets for head attachments
///
/// Defines how items attached to the head (hats, horns, etc.) should move during animations.
/// Head movements are subtle but important for conveying emotion and energy.
fn create_head_animation_offsets() -> HashMap<AnimationType, Vec<AnimationOffset>> {
    let mut offsets = HashMap::new();
    
    // Head bob during walking - creates a natural bouncing motion
    // The head moves up/down by 2 pixels and tilts slightly left/right
    offsets.insert(AnimationType::Walk, vec![
        AnimationOffset { frame: 0, position_offset: Vec2::new(0.0, 0.0), rotation: 0.0, scale: 1.0 },
        AnimationOffset { frame: 2, position_offset: Vec2::new(0.0, 2.0), rotation: 5.0, scale: 1.0 },
        AnimationOffset { frame: 4, position_offset: Vec2::new(0.0, 0.0), rotation: 0.0, scale: 1.0 },
        AnimationOffset { frame: 6, position_offset: Vec2::new(0.0, 2.0), rotation: -5.0, scale: 1.0 },
    ]);
    
    // Head tilt during eating
    offsets.insert(AnimationType::Eat, vec![
        AnimationOffset { frame: 0, position_offset: Vec2::new(0.0, 0.0), rotation: 0.0, scale: 1.0 },
        AnimationOffset { frame: 2, position_offset: Vec2::new(2.0, -3.0), rotation: 15.0, scale: 1.0 },
        AnimationOffset { frame: 4, position_offset: Vec2::new(2.0, -3.0), rotation: 15.0, scale: 1.0 },
    ]);
    
    offsets
}

/// Creates animation offsets for left hand attachments
///
/// Defines movement for items held in the left hand. These animations are designed to
/// create believable tool use and natural arm swinging during movement.
fn create_left_hand_animation_offsets() -> HashMap<AnimationType, Vec<AnimationOffset>> {
    let mut offsets = HashMap::new();
    
    // Walking animation - hand swings
    offsets.insert(AnimationType::Walk, vec![
        AnimationOffset { frame: 0, position_offset: Vec2::new(0.0, 0.0), rotation: 0.0, scale: 1.0 },
        AnimationOffset { frame: 2, position_offset: Vec2::new(2.0, -1.0), rotation: 15.0, scale: 1.0 },
        AnimationOffset { frame: 4, position_offset: Vec2::new(0.0, 0.0), rotation: 0.0, scale: 1.0 },
        AnimationOffset { frame: 6, position_offset: Vec2::new(-2.0, -1.0), rotation: -15.0, scale: 1.0 },
    ]);
    
    // Eating animation - hand to mouth
    offsets.insert(AnimationType::Eat, vec![
        AnimationOffset { frame: 0, position_offset: Vec2::new(0.0, 0.0), rotation: 0.0, scale: 1.0 },
        AnimationOffset { frame: 1, position_offset: Vec2::new(5.0, 8.0), rotation: 45.0, scale: 1.0 },
        AnimationOffset { frame: 3, position_offset: Vec2::new(8.0, 15.0), rotation: 70.0, scale: 1.0 },
        AnimationOffset { frame: 5, position_offset: Vec2::new(5.0, 8.0), rotation: 45.0, scale: 1.0 },
    ]);
    
    // Tool use animation - simulates swinging or using a tool
    // The hand raises up and forward, with slight scale changes to show effort
    offsets.insert(AnimationType::UseItem, vec![
        AnimationOffset { frame: 0, position_offset: Vec2::new(0.0, 0.0), rotation: 0.0, scale: 1.0 },
        AnimationOffset { frame: 1, position_offset: Vec2::new(3.0, 5.0), rotation: -30.0, scale: 1.1 },
        AnimationOffset { frame: 2, position_offset: Vec2::new(5.0, 10.0), rotation: -60.0, scale: 1.2 },
        AnimationOffset { frame: 3, position_offset: Vec2::new(4.0, 12.0), rotation: -75.0, scale: 1.15 },
        AnimationOffset { frame: 4, position_offset: Vec2::new(2.0, 8.0), rotation: -45.0, scale: 1.05 },
    ]);
    
    offsets
}

/// Creates animation offsets for right hand attachments
///
/// The right hand is typically the primary hand for tools and weapons.
/// Animations here are more dramatic than the left hand for attack moves.
fn create_right_hand_animation_offsets() -> HashMap<AnimationType, Vec<AnimationOffset>> {
    let mut offsets = HashMap::new();
    
    // Mirror of left hand for walking
    offsets.insert(AnimationType::Walk, vec![
        AnimationOffset { frame: 0, position_offset: Vec2::new(0.0, 0.0), rotation: 0.0, scale: 1.0 },
        AnimationOffset { frame: 2, position_offset: Vec2::new(-2.0, -1.0), rotation: -15.0, scale: 1.0 },
        AnimationOffset { frame: 4, position_offset: Vec2::new(0.0, 0.0), rotation: 0.0, scale: 1.0 },
        AnimationOffset { frame: 6, position_offset: Vec2::new(2.0, -1.0), rotation: 15.0, scale: 1.0 },
    ]);
    
    // Attack animation - weapon swing
    offsets.insert(AnimationType::Attack, vec![
        AnimationOffset { frame: 0, position_offset: Vec2::new(0.0, 0.0), rotation: 0.0, scale: 1.0 },
        AnimationOffset { frame: 1, position_offset: Vec2::new(-5.0, 10.0), rotation: -45.0, scale: 1.2 },
        AnimationOffset { frame: 2, position_offset: Vec2::new(-8.0, 15.0), rotation: -90.0, scale: 1.3 },
        AnimationOffset { frame: 3, position_offset: Vec2::new(10.0, 5.0), rotation: 45.0, scale: 1.4 },
        AnimationOffset { frame: 4, position_offset: Vec2::new(5.0, 0.0), rotation: 20.0, scale: 1.1 },
    ]);
    
    offsets
}

/// Creates animation offsets for back attachments
///
/// Back items (bags, capes) have subtle movements that follow the body's motion
/// without being too distracting.
fn create_back_animation_offsets() -> HashMap<AnimationType, Vec<AnimationOffset>> {
    let mut offsets = HashMap::new();
    
    // Subtle movement during walk
    offsets.insert(AnimationType::Walk, vec![
        AnimationOffset { frame: 0, position_offset: Vec2::new(0.0, 0.0), rotation: 0.0, scale: 1.0 },
        AnimationOffset { frame: 4, position_offset: Vec2::new(0.0, 1.0), rotation: 2.0, scale: 1.0 },
    ]);
    
    // Running causes more movement
    offsets.insert(AnimationType::Run, vec![
        AnimationOffset { frame: 0, position_offset: Vec2::new(0.0, 0.0), rotation: 0.0, scale: 1.0 },
        AnimationOffset { frame: 2, position_offset: Vec2::new(-1.0, 2.0), rotation: -5.0, scale: 1.05 },
        AnimationOffset { frame: 4, position_offset: Vec2::new(1.0, 2.0), rotation: 5.0, scale: 1.05 },
    ]);
    
    offsets
}

/// Creates animation offsets for tail tip attachments
///
/// Tail animations add personality and emotion to creatures. The tail wags when
/// idle and swings naturally during movement.
fn create_tail_animation_offsets() -> HashMap<AnimationType, Vec<AnimationOffset>> {
    let mut offsets = HashMap::new();
    
    // Tail wag during idle
    offsets.insert(AnimationType::Idle, vec![
        AnimationOffset { frame: 0, position_offset: Vec2::new(0.0, 0.0), rotation: 0.0, scale: 1.0 },
        AnimationOffset { frame: 1, position_offset: Vec2::new(2.0, 0.0), rotation: 10.0, scale: 1.0 },
        AnimationOffset { frame: 2, position_offset: Vec2::new(0.0, 0.0), rotation: 0.0, scale: 1.0 },
        AnimationOffset { frame: 3, position_offset: Vec2::new(-2.0, 0.0), rotation: -10.0, scale: 1.0 },
    ]);
    
    // Tail movement during walk
    offsets.insert(AnimationType::Walk, vec![
        AnimationOffset { frame: 0, position_offset: Vec2::new(0.0, 0.0), rotation: 0.0, scale: 1.0 },
        AnimationOffset { frame: 2, position_offset: Vec2::new(3.0, 1.0), rotation: 15.0, scale: 1.0 },
        AnimationOffset { frame: 4, position_offset: Vec2::new(0.0, 0.0), rotation: 0.0, scale: 1.0 },
        AnimationOffset { frame: 6, position_offset: Vec2::new(-3.0, 1.0), rotation: -15.0, scale: 1.0 },
    ]);
    
    offsets
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_attachment_point_creation() {
        let points = CreatureAttachmentPoints::default();
        
        assert_eq!(points.head.name, "head");
        assert_eq!(points.head.base_position, Vec2::new(0.0, 20.0));
        assert_eq!(points.left_hand.name, "left_hand");
        assert_eq!(points.right_hand.name, "right_hand");
    }
    
    #[test]
    fn test_animation_offset_interpolation() {
        let offsets = vec![
            AnimationOffset { frame: 0, position_offset: Vec2::new(0.0, 0.0), rotation: 0.0, scale: 1.0 },
            AnimationOffset { frame: 4, position_offset: Vec2::new(4.0, 4.0), rotation: 40.0, scale: 2.0 },
        ];
        
        // Test exact keyframe
        let result = interpolate_animation_offset(&offsets, 0);
        assert!(result.is_some());
        let offset = result.unwrap();
        assert_eq!(offset.position_offset, Vec2::new(0.0, 0.0));
        
        // Test interpolation at frame 2
        let result = interpolate_animation_offset(&offsets, 2);
        assert!(result.is_some());
        let offset = result.unwrap();
        assert_eq!(offset.position_offset, Vec2::new(2.0, 2.0));
        assert_eq!(offset.rotation, 20.0);
        assert_eq!(offset.scale, 1.5);
    }
    
    #[test]
    fn test_animation_type_conversion() {
        assert_eq!(AnimationType::from(AnimationState::Idle), AnimationType::Idle);
        assert_eq!(AnimationType::from(AnimationState::Walk), AnimationType::Walk);
        assert_eq!(AnimationType::from(AnimationState::Eat), AnimationType::Eat);
    }
}