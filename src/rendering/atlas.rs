//! Sprite atlas organization and UV mapping for creature animations

use bevy::prelude::*;
use std::collections::HashMap;
use crate::components::{AnimationState, PatternType};

/// Atlas layout information for creature sprites
#[derive(Clone)]
pub struct AtlasLayout {
    pub texture_path: String,
    pub grid_size: (u32, u32),
    pub sprite_size: Vec2,
    pub animations: HashMap<AnimationType, AnimationRange>,
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

#[derive(Clone)]
pub struct AnimationRange {
    pub start_frame: usize,
    pub frame_count: usize,
    pub fps: f32,
    pub loop_mode: AnimationLoopMode,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AnimationLoopMode {
    Once,
    Loop,
    PingPong,
}

#[derive(Clone)]
pub struct VariationInfo {
    pub name: String,
    pub row_offset: u32,
    pub genetic_trait: Option<GeneticTrait>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum GeneticTrait {
    Size(f32),
    Pattern(PatternType),
    Color(Color),
}

/// UV coordinate mapping for sprite atlas
#[derive(Component, Clone, Reflect)]
#[reflect(Component)]
pub struct AtlasUVMapping {
    pub frame_width: f32,
    pub frame_height: f32,
    pub expression_width: f32,
    pub expression_height: f32,
    pub atlas_size: Vec2,
}

impl AtlasUVMapping {
    pub fn new(atlas_width: f32, atlas_height: f32, sprite_size: Vec2) -> Self {
        Self {
            frame_width: sprite_size.x / atlas_width,
            frame_height: sprite_size.y / atlas_height,
            expression_width: (sprite_size.x * 2.0) / atlas_width, // Expressions are 2x size
            expression_height: (sprite_size.y * 2.0) / atlas_height,
            atlas_size: Vec2::new(atlas_width, atlas_height),
        }
    }
    
    pub fn get_animation_uv(&self, animation: AnimationType, frame: usize, variation: usize) -> Rect {
        let (col, row_offset) = self.get_frame_position(animation, frame);
        let row = variation * 3 + row_offset; // 3 rows per variation
        
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
#[derive(Resource)]
pub struct AtlasManager {
    pub creature_atlases: HashMap<CreatureSpecies, AtlasLayout>,
    pub terrain_atlases: HashMap<BiomeType, AtlasLayout>,
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
    
    pub fn select_variation_for_genetics(
        &self,
        species: CreatureSpecies,
        genetics: &crate::components::Genetics,
    ) -> u32 {
        if let Some(layout) = self.creature_atlases.get(&species) {
            for variation in &layout.variations {
                if let Some(trait_match) = &variation.genetic_trait {
                    match trait_match {
                        GeneticTrait::Size(target_size) => {
                            let size_diff = (genetics.size - target_size).abs();
                            if size_diff < 0.1 {
                                return variation.row_offset;
                            }
                        }
                        GeneticTrait::Pattern(pattern) => {
                            let current_pattern = genetics_to_pattern(genetics.pattern);
                            if current_pattern == *pattern {
                                return variation.row_offset;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        
        0 // Default variation
    }
}

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

fn create_standard_variations() -> Vec<VariationInfo> {
    vec![
        VariationInfo {
            name: "Normal".to_string(),
            row_offset: 0,
            genetic_trait: None,
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

pub fn species_to_string(species: CreatureSpecies) -> &'static str {
    match species {
        CreatureSpecies::Herbivore => "herbivore",
        CreatureSpecies::Carnivore => "carnivore",
        CreatureSpecies::Omnivore => "omnivore",
    }
}

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
pub fn update_atlas_indices(
    mut query: Query<(
        &crate::components::CartoonSprite,
        &mut TextureAtlas,
        &AtlasUVMapping,
        Option<&crate::components::AnimatedSprite>,
    ), Changed<crate::components::CartoonSprite>>,
) {
    for (cartoon_sprite, mut atlas, uv_mapping, animated_sprite) in query.iter_mut() {
        // Get current frame from animated sprite or default to 0
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