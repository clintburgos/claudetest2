//! Sprite atlas organization and UV mapping for creature animations
//!
//! This module manages the organization of sprite sheets (atlases) for efficient rendering
//! of creature animations and expressions. It provides:
//!
//! - Automatic UV coordinate calculation for sprite frames
//! - Animation frame mapping and timing information
//! - Genetic variation selection based on creature traits
//! - Expression overlay coordination
//!
//! # Atlas Organization
//!
//! Creature atlases are organized in a grid layout:
//! - Rows 0-2: Basic animations (idle, walk, eat, etc.)
//! - Rows 3-4: Special animations (emotions)
//! - Rows 5-6: Expression overlays (2x size)
//! - Multiple variations stack vertically (3 rows per variation)
//!
//! # Performance Considerations
//!
//! Using texture atlases reduces draw calls by batching multiple sprites
//! into a single texture. UV coordinates are calculated once and cached.

use bevy::prelude::*;
use std::collections::HashMap;
use crate::components::{AnimationState, PatternType};

/// Atlas layout information for creature sprites
///
/// Defines the structure and organization of a sprite atlas texture,
/// including animation frames, variations, and grid layout.
#[derive(Clone)]
pub struct AtlasLayout {
    /// Path to the atlas texture file
    pub texture_path: String,
    
    /// Grid dimensions (columns, rows) of the atlas
    pub grid_size: (u32, u32),
    
    /// Size of individual sprite frames in pixels
    pub sprite_size: Vec2,
    
    /// Animation definitions with frame ranges and timing
    pub animations: HashMap<AnimationType, AnimationRange>,
    
    /// Genetic variations available in this atlas
    pub variations: Vec<VariationInfo>,
}

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
    Special(SpecialAnimationType),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SpecialAnimationType {
    Happy,
    Sad,
    Angry,
    Curious,
}

/// Defines the frame range and playback settings for an animation
///
/// Each animation has a specific range of frames in the atlas and
/// timing information for smooth playback.
#[derive(Clone)]
pub struct AnimationRange {
    /// First frame index in the atlas grid
    pub start_frame: usize,
    
    /// Total number of frames in this animation
    pub frame_count: usize,
    
    /// Playback speed in frames per second
    pub fps: f32,
    
    /// How the animation should repeat
    pub loop_mode: AnimationLoopMode,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AnimationLoopMode {
    Once,
    Loop,
    PingPong,
}

/// Information about a visual variation of a creature
///
/// Variations allow the same creature species to have different
/// appearances based on genetic traits.
#[derive(Clone)]
pub struct VariationInfo {
    /// Human-readable name for this variation
    pub name: String,
    
    /// Row offset in the atlas for this variation (multiplied by 3 for actual row)
    pub row_offset: u32,
    
    /// Optional genetic trait that triggers this variation
    pub genetic_trait: Option<GeneticTrait>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum GeneticTrait {
    Size(f32),
    Pattern(PatternType),
    Color(Color),
}

/// UV coordinate mapping for sprite atlas
///
/// This component stores pre-calculated UV coordinate information
/// for efficient texture sampling during rendering.
#[derive(Component, Clone, Reflect)]
#[reflect(Component)]
pub struct AtlasUVMapping {
    /// Normalized width of a single animation frame (0.0-1.0)
    pub frame_width: f32,
    
    /// Normalized height of a single animation frame (0.0-1.0)
    pub frame_height: f32,
    
    /// Normalized width of expression overlays (typically 2x frame_width)
    pub expression_width: f32,
    
    /// Normalized height of expression overlays (typically 2x frame_height)
    pub expression_height: f32,
    
    /// Total size of the atlas texture in pixels
    pub atlas_size: Vec2,
}

impl AtlasUVMapping {
    /// Creates a new UV mapping for the given atlas dimensions
    ///
    /// # Arguments
    /// * `atlas_width` - Total width of the atlas texture in pixels
    /// * `atlas_height` - Total height of the atlas texture in pixels
    /// * `sprite_size` - Size of individual sprite frames in pixels
    ///
    /// # Note
    /// Expression overlays are automatically calculated as 2x the sprite size
    /// to allow for more detailed facial features.
    pub fn new(atlas_width: f32, atlas_height: f32, sprite_size: Vec2) -> Self {
        Self {
            frame_width: sprite_size.x / atlas_width,
            frame_height: sprite_size.y / atlas_height,
            expression_width: (sprite_size.x * 2.0) / atlas_width, // Expressions are 2x size for detail
            expression_height: (sprite_size.y * 2.0) / atlas_height,
            atlas_size: Vec2::new(atlas_width, atlas_height),
        }
    }
    
    /// Calculates UV coordinates for a specific animation frame
    ///
    /// # Arguments
    /// * `animation` - The type of animation being played
    /// * `frame` - Current frame index within the animation
    /// * `variation` - Genetic variation index (0 for default)
    ///
    /// # Returns
    /// A `Rect` containing normalized UV coordinates (0.0-1.0) for the texture
    ///
    /// # Atlas Layout
    /// Each variation occupies 3 rows in the atlas, allowing for all animation
    /// types to be stored vertically. Frames are arranged horizontally.
    pub fn get_animation_uv(&self, animation: AnimationType, frame: usize, variation: usize) -> Rect {
        let (col, row_offset) = self.get_frame_position(animation, frame);
        let row = variation * 3 + row_offset; // 3 rows per variation for all animations
        
        Rect {
            min: Vec2::new(
                col as f32 * self.frame_width,
                row as f32 * self.frame_height,
            ),
            max: Vec2::new(
                (col + 1) as f32 * self.frame_width,
                (row + 1) as f32 * self.frame_height,
            ),
        }
    }
    
    pub fn get_expression_uv(&self, expression: ExpressionType) -> Rect {
        let (col, row) = self.get_expression_position(expression);
        let base_row = 5; // Expression sheet starts at row 5
        
        Rect {
            min: Vec2::new(
                col as f32 * self.expression_width,
                (base_row + row) as f32 * self.expression_height,
            ),
            max: Vec2::new(
                (col + 1) as f32 * self.expression_width,
                (base_row + row + 1) as f32 * self.expression_height,
            ),
        }
    }
    
    /// Maps animation types and frames to grid positions in the atlas
    ///
    /// # Atlas Grid Layout:
    /// ```text
    /// Row 0: [Idle(0-3)] [Walk(4-11)]
    /// Row 1: [Run(0-5)] [Eat(2-7)] [Sleep(8-11)]
    /// Row 2: [Talk(0-7)] [Attack(4-9)] [Death(10-15)]
    /// Row 3: [Special animations]
    /// ```
    ///
    /// # Returns
    /// Tuple of (column, row_offset) in the atlas grid
    fn get_frame_position(&self, animation: AnimationType, frame: usize) -> (usize, usize) {
        match animation {
            AnimationType::Idle => (frame % 4, 0),
            AnimationType::Walk => ((frame % 8) + 4, 0),
            AnimationType::Run => (frame % 6, 1),
            AnimationType::Eat => ((frame % 6) + 2, 1),
            AnimationType::Sleep => ((frame % 4) + 8, 1),
            AnimationType::Talk => (frame % 8, 2),
            AnimationType::Attack => ((frame % 6) + 4, 2),
            AnimationType::Death => ((frame % 6) + 10, 2),
            AnimationType::Special(special) => self.get_special_position(special, frame),
        }
    }
    
    fn get_special_position(&self, special: SpecialAnimationType, frame: usize) -> (usize, usize) {
        match special {
            SpecialAnimationType::Happy => ((frame % 4) + 0, 3),
            SpecialAnimationType::Sad => ((frame % 4) + 4, 3),
            SpecialAnimationType::Angry => ((frame % 4) + 8, 3),
            SpecialAnimationType::Curious => ((frame % 4) + 12, 3),
        }
    }
    
    fn get_expression_position(&self, expression: ExpressionType) -> (usize, usize) {
        match expression {
            ExpressionType::Neutral => (0, 0),
            ExpressionType::Happy => (1, 0),
            ExpressionType::Sad => (2, 0),
            ExpressionType::Angry => (3, 0),
            ExpressionType::Scared => (4, 0),
            ExpressionType::Curious => (5, 0),
            ExpressionType::Tired => (6, 0),
            ExpressionType::Hungry => (7, 0),
            ExpressionType::Excited => (0, 1),
            ExpressionType::Content => (1, 1),
            ExpressionType::Disgusted => (2, 1),
            ExpressionType::Surprised => (3, 1),
            ExpressionType::Confused => (4, 1),
            ExpressionType::Sleeping => (5, 1),
            ExpressionType::Sick => (6, 1),
            ExpressionType::Love => (7, 1),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExpressionType {
    Neutral,
    Happy,
    Sad,
    Angry,
    Scared,
    Curious,
    Tired,
    Hungry,
    Excited,
    Content,
    Disgusted,
    Surprised,
    Confused,
    Sleeping,
    Sick,
    Love,
}

/// Resource for managing sprite atlases
///
/// Central manager for all texture atlases in the game. Handles loading,
/// organization, and selection of appropriate sprites based on creature
/// types and genetic traits.
#[derive(Resource)]
pub struct AtlasManager {
    /// Atlas layouts for each creature species
    pub creature_atlases: HashMap<CreatureSpecies, AtlasLayout>,
    
    /// Atlas layouts for terrain tiles by biome
    pub terrain_atlases: HashMap<BiomeType, AtlasLayout>,
    
    /// Cached texture handles to avoid reloading
    pub loaded_atlases: HashMap<String, Handle<Image>>,
}

impl Default for AtlasManager {
    fn default() -> Self {
        Self {
            creature_atlases: HashMap::new(),
            terrain_atlases: HashMap::new(),
            loaded_atlases: HashMap::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CreatureSpecies {
    Herbivore,
    Carnivore,
    Omnivore,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BiomeType {
    Forest,
    Desert,
    Grassland,
    Tundra,
    Ocean,
}

impl AtlasManager {
    /// Creates and organizes a standard creature atlas layout
    ///
    /// Sets up the standard animation set and variations for a creature species.
    /// All creatures share the same basic animation structure for consistency.
    ///
    /// # Standard Atlas Specifications:
    /// - Grid: 8x8 tiles minimum
    /// - Sprite size: 48x48 pixels
    /// - 5 genetic variations
    /// - 8 animation types with varying frame counts
    pub fn organize_creature_atlas(&mut self, species: CreatureSpecies) -> AtlasLayout {
        let animations = create_standard_animation_set();
        let variations = create_standard_variations();
        
        AtlasLayout {
            texture_path: format!("sprites/creatures/{}/atlas.png", species_to_string(species)),
            grid_size: (8, 8),
            sprite_size: Vec2::new(48.0, 48.0),
            animations,
            variations,
        }
    }
    
    /// Selects the appropriate visual variation based on creature genetics
    ///
    /// Matches genetic traits to available variations in the atlas. This allows
    /// creatures with different genetics to have visually distinct appearances.
    ///
    /// # Matching Algorithm:
    /// 1. Size variations: Match if within 0.1 of target size
    /// 2. Pattern variations: Direct pattern type matching
    /// 3. Color variations: Not yet implemented
    ///
    /// # Returns
    /// Row offset for the selected variation, or 0 for default appearance
    pub fn select_variation_for_genetics(
        &self,
        species: CreatureSpecies,
        genetics: &crate::components::Genetics,
    ) -> u32 {
        if let Some(layout) = self.creature_atlases.get(&species) {
            // Check each variation for genetic trait matches
            for variation in &layout.variations {
                if let Some(trait_match) = &variation.genetic_trait {
                    match trait_match {
                        GeneticTrait::Size(target_size) => {
                            // Size tolerance of 0.1 allows for genetic variation
                            let size_diff = (genetics.size - target_size).abs();
                            if size_diff < 0.1 {
                                return variation.row_offset;
                            }
                        }
                        GeneticTrait::Pattern(pattern) => {
                            // Convert genetic value to pattern type
                            let current_pattern = genetics_to_pattern(genetics.pattern);
                            if current_pattern == *pattern {
                                return variation.row_offset;
                            }
                        }
                        _ => {} // Color variations not yet implemented
                    }
                }
            }
        }
        
        0 // Default variation when no genetic match found
    }
}

/// Creates the standard animation set shared by all creatures
///
/// Defines frame counts, timing, and loop modes for each animation type.
/// These values are carefully tuned for smooth, cartoon-like movement.
///
/// # Animation Design Principles:
/// - Idle: Subtle breathing motion (4 frames)
/// - Walk: Full gait cycle (8 frames)
/// - Run: Faster cycle with fewer frames (6 frames)
/// - Action animations: Play once then return to idle
fn create_standard_animation_set() -> HashMap<AnimationType, AnimationRange> {
    let mut animations = HashMap::new();
    
    animations.insert(AnimationType::Idle, AnimationRange {
        start_frame: 0,
        frame_count: 4,
        fps: 4.0,
        loop_mode: AnimationLoopMode::Loop,
    });
    
    animations.insert(AnimationType::Walk, AnimationRange {
        start_frame: 4,
        frame_count: 8,
        fps: 12.0,
        loop_mode: AnimationLoopMode::Loop,
    });
    
    animations.insert(AnimationType::Run, AnimationRange {
        start_frame: 12,
        frame_count: 6,
        fps: 18.0,
        loop_mode: AnimationLoopMode::Loop,
    });
    
    animations.insert(AnimationType::Eat, AnimationRange {
        start_frame: 18,
        frame_count: 6,
        fps: 8.0,
        loop_mode: AnimationLoopMode::Once,
    });
    
    animations.insert(AnimationType::Sleep, AnimationRange {
        start_frame: 24,
        frame_count: 4,
        fps: 2.0,
        loop_mode: AnimationLoopMode::Loop,
    });
    
    animations.insert(AnimationType::Talk, AnimationRange {
        start_frame: 28,
        frame_count: 8,
        fps: 10.0,
        loop_mode: AnimationLoopMode::Loop,
    });
    
    animations.insert(AnimationType::Attack, AnimationRange {
        start_frame: 36,
        frame_count: 6,
        fps: 15.0,
        loop_mode: AnimationLoopMode::Once,
    });
    
    animations.insert(AnimationType::Death, AnimationRange {
        start_frame: 42,
        frame_count: 8,
        fps: 10.0,
        loop_mode: AnimationLoopMode::Once,
    });
    
    animations
}

/// Creates the standard genetic variations available for creatures
///
/// Each variation represents a distinct visual appearance triggered by
/// specific genetic traits. This allows for visual diversity while
/// maintaining a consistent art style.
///
/// # Current Variations:
/// - Normal: Default appearance for average genetics
/// - Large/Small: Size-based variations
/// - Spotted/Striped: Pattern-based variations
fn create_standard_variations() -> Vec<VariationInfo> {
    vec![
        VariationInfo {
            name: "Normal".to_string(),
            row_offset: 0,
            genetic_trait: None, // Default for creatures without special traits
        },
        VariationInfo {
            name: "Large".to_string(),
            row_offset: 1,
            genetic_trait: Some(GeneticTrait::Size(1.3)),
        },
        VariationInfo {
            name: "Small".to_string(),
            row_offset: 2,
            genetic_trait: Some(GeneticTrait::Size(0.7)),
        },
        VariationInfo {
            name: "Spotted".to_string(),
            row_offset: 3,
            genetic_trait: Some(GeneticTrait::Pattern(PatternType::Spots)),
        },
        VariationInfo {
            name: "Striped".to_string(),
            row_offset: 4,
            genetic_trait: Some(GeneticTrait::Pattern(PatternType::Stripes)),
        },
    ]
}

/// Converts creature species enum to string for file paths
///
/// Used for constructing atlas texture paths and other resource identifiers.
pub fn species_to_string(species: CreatureSpecies) -> &'static str {
    match species {
        CreatureSpecies::Herbivore => "herbivore",
        CreatureSpecies::Carnivore => "carnivore",
        CreatureSpecies::Omnivore => "omnivore",
    }
}

/// Converts a genetic pattern value to a discrete pattern type
///
/// Maps the continuous genetic value (0.0-1.0) to distinct visual patterns.
/// Higher values indicate more complex patterns.
///
/// # Thresholds:
/// - 0.0-0.2: No pattern (solid color)
/// - 0.2-0.4: Patches (large irregular spots)
/// - 0.4-0.7: Spots (regular circular patterns)
/// - 0.7-1.0: Stripes (linear patterns)
fn genetics_to_pattern(pattern_value: f32) -> PatternType {
    if pattern_value > 0.7 {
        PatternType::Stripes
    } else if pattern_value > 0.4 {
        PatternType::Spots
    } else if pattern_value > 0.2 {
        PatternType::Patches
    } else {
        PatternType::None
    }
}

/// System to update texture atlas indices based on animation state
///
/// This system runs whenever a creature's animation state changes, updating
/// the texture atlas index to display the correct sprite frame.
///
/// # Performance Note
/// Only runs on changed CartoonSprite components to minimize unnecessary
/// calculations. The UV mapping is pre-calculated for efficiency.
pub fn update_atlas_indices(
    mut query: Query<(
        &crate::components::CartoonSprite,
        &mut TextureAtlas,
        &AtlasUVMapping,
        Option<&crate::components::AnimatedSprite>,
    ), Changed<crate::components::CartoonSprite>>,
) {
    for (cartoon_sprite, mut atlas, uv_mapping, animated_sprite) in query.iter_mut() {
        // Get current frame from animated sprite or default to first frame
        let current_frame = animated_sprite
            .map(|a| a.current_frame)
            .unwrap_or(0);
        
        // Convert AnimationState to AnimationType
        let anim_type = animation_state_to_type(cartoon_sprite.base_animation);
        
        // Calculate UV coordinates
        let uv_rect = uv_mapping.get_animation_uv(anim_type, current_frame, 0);
        
        // Update atlas index based on UV position
        // This is a simplified calculation - in practice would need proper index mapping
        let col = (uv_rect.min.x / uv_mapping.frame_width) as usize;
        let row = (uv_rect.min.y / uv_mapping.frame_height) as usize;
        atlas.index = row * 8 + col; // Assuming 8 columns
    }
}

/// Converts component AnimationState to atlas AnimationType
///
/// This conversion is necessary because the atlas system uses its own
/// animation type enum to avoid circular dependencies with the components module.
fn animation_state_to_type(state: AnimationState) -> AnimationType {
    match state {
        AnimationState::Idle => AnimationType::Idle,
        AnimationState::Walk => AnimationType::Walk,
        AnimationState::Run => AnimationType::Run,
        AnimationState::Eat => AnimationType::Eat,
        AnimationState::Sleep => AnimationType::Sleep,
        AnimationState::Talk => AnimationType::Talk,
        AnimationState::Attack => AnimationType::Attack,
        AnimationState::Death => AnimationType::Death,
        AnimationState::Special(special) => match special {
            crate::components::SpecialAnimation::Happy => AnimationType::Special(SpecialAnimationType::Happy),
            crate::components::SpecialAnimation::Sad => AnimationType::Special(SpecialAnimationType::Sad),
            crate::components::SpecialAnimation::Angry => AnimationType::Special(SpecialAnimationType::Angry),
            crate::components::SpecialAnimation::Curious => AnimationType::Special(SpecialAnimationType::Curious),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_uv_mapping_calculation() {
        let uv_mapping = AtlasUVMapping::new(384.0, 384.0, Vec2::new(48.0, 48.0));
        
        // Test frame dimensions
        assert_eq!(uv_mapping.frame_width, 48.0 / 384.0);
        assert_eq!(uv_mapping.frame_height, 48.0 / 384.0);
        
        // Test animation UV calculation
        let uv = uv_mapping.get_animation_uv(AnimationType::Idle, 0, 0);
        assert_eq!(uv.min, Vec2::new(0.0, 0.0));
        assert_eq!(uv.max.x, 48.0 / 384.0);
    }
    
    #[test]
    fn test_variation_selection() {
        let mut manager = AtlasManager::default();
        let layout = manager.organize_creature_atlas(CreatureSpecies::Herbivore);
        manager.creature_atlases.insert(CreatureSpecies::Herbivore, layout);
        
        let mut genetics = crate::components::Genetics::default();
        genetics.size = 1.3;
        
        let variation = manager.select_variation_for_genetics(CreatureSpecies::Herbivore, &genetics);
        assert_eq!(variation, 1); // Large variation
    }
    
    #[test]
    fn test_animation_range_creation() {
        let animations = create_standard_animation_set();
        
        assert!(animations.contains_key(&AnimationType::Idle));
        assert!(animations.contains_key(&AnimationType::Walk));
        
        let idle = &animations[&AnimationType::Idle];
        assert_eq!(idle.frame_count, 4);
        assert_eq!(idle.loop_mode, AnimationLoopMode::Loop);
    }
    
    #[test]
    fn test_genetics_to_pattern() {
        assert_eq!(genetics_to_pattern(0.1), PatternType::None);
        assert_eq!(genetics_to_pattern(0.3), PatternType::Patches);
        assert_eq!(genetics_to_pattern(0.5), PatternType::Spots);
        assert_eq!(genetics_to_pattern(0.8), PatternType::Stripes);
    }
    
    #[test]
    fn test_expression_type_position() {
        let uv_mapping = AtlasUVMapping::new(1024.0, 1024.0, Vec2::new(64.0, 64.0));
        
        // Test different expression positions
        let happy_uv = uv_mapping.get_expression_uv(ExpressionType::Happy);
        let sad_uv = uv_mapping.get_expression_uv(ExpressionType::Sad);
        
        // Different expressions should have different UV coordinates
        assert_ne!(happy_uv.min.x, sad_uv.min.x);
    }
}