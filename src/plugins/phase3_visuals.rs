//! Phase 3: Creature Visual Systems Plugin
//! Integrates animation state machine, expressions, patterns, and attachments

use bevy::prelude::*;
use crate::{
    systems::{
        animation::{AnimationStateMachine, AnimationPlaybackState, AnimationLayers, AnimationLayer, AnimationMask, update_animation_states},
        expression::{EmotionMapper, ExpressionController, DetailedExpressionOverlay, sync_ai_state_to_emotions, update_expression_overlays},
        attachments::{CreatureAttachmentPoints, AttachedItem, update_attachment_transforms},
    },
    rendering::{
        patterns::{GeneticPattern, PatternRenderingConfig, PatternMaterial, update_genetic_patterns, apply_pattern_overlays},
        atlas::{AtlasManager, AtlasUVMapping, update_atlas_indices},
    },
    components::*,
};
// Use the AnimationType from animation module
use crate::systems::animation::AnimationType;
use crate::rendering::atlas::CreatureSpecies;

/// Main plugin for Phase 3 creature visual systems
pub struct Phase3VisualsPlugin;

impl Plugin for Phase3VisualsPlugin {
    fn build(&self, app: &mut App) {
        // Add resources
        app.insert_resource(AnimationStateMachine::default())
            .insert_resource(EmotionMapper::default())
            .insert_resource(PatternRenderingConfig::default())
            .insert_resource(AtlasManager::default())
            .insert_resource(Phase3Config::default());
        
        // Add material types
        app.add_plugins(MaterialPlugin::<PatternMaterial>::default());
        
        // Register types for reflection/serialization
        app.register_type::<AnimationPlaybackState>()
            .register_type::<ExpressionController>()
            .register_type::<GeneticPattern>()
            .register_type::<CreatureAttachmentPoints>()
            .register_type::<AttachedItem>()
            .register_type::<AnimationLayers>()
            .register_type::<DetailedExpressionOverlay>()
            .register_type::<AtlasUVMapping>();
        
        // Add startup systems
        app.add_systems(Startup, (
            setup_phase3_resources,
            load_creature_atlases,
        ).chain());
        
        // Add update systems in proper order
        app.add_systems(Update, (
            // Animation systems
            update_animation_states,
            update_atlas_indices,
            
            // Expression systems
            sync_ai_state_to_emotions,
            update_expression_overlays,
            
            // Pattern systems
            update_genetic_patterns,
            apply_pattern_overlays,
            
            // Attachment systems
            update_attachment_transforms,
            
            // Integration systems
            apply_visual_enhancements,
            update_creature_visuals,
        ).chain());
        
        // Add debug systems
        #[cfg(debug_assertions)]
        app.add_systems(Update, (
            debug_animation_states,
            debug_expression_states,
            debug_attachment_points,
        ));
    }
}

/// Configuration for Phase 3 visual systems
#[derive(Resource)]
pub struct Phase3Config {
    pub animations_enabled: bool,
    pub expressions_enabled: bool,
    pub patterns_enabled: bool,
    pub attachments_enabled: bool,
    pub debug_visualizations: bool,
}

impl Default for Phase3Config {
    fn default() -> Self {
        Self {
            animations_enabled: true,
            expressions_enabled: true,
            patterns_enabled: true,
            attachments_enabled: true,
            debug_visualizations: false,
        }
    }
}

/// Setup Phase 3 resources and default configurations
fn setup_phase3_resources(
    mut commands: Commands,
    mut atlas_manager: ResMut<AtlasManager>,
) {
    info!("Initializing Phase 3: Creature Visual Systems");
    
    // Set up creature atlas layouts
    for species in [CreatureSpecies::Herbivore, CreatureSpecies::Carnivore, CreatureSpecies::Omnivore] {
        let layout = atlas_manager.organize_creature_atlas(species);
        atlas_manager.creature_atlases.insert(species, layout);
    }
    
    info!("Phase 3 resources initialized");
}

/// Load creature sprite atlases
fn load_creature_atlases(
    asset_server: Res<AssetServer>,
    mut atlas_manager: ResMut<AtlasManager>,
) {
    info!("Loading creature atlases for Phase 3");
    
    // Load atlas textures
    let atlases_to_load: Vec<_> = atlas_manager.creature_atlases
        .iter()
        .map(|(species, layout)| (species.clone(), layout.texture_path.clone()))
        .collect();
    
    for (species, texture_path) in atlases_to_load {
        let handle: Handle<Image> = asset_server.load(&texture_path);
        atlas_manager.loaded_atlases.insert(texture_path.clone(), handle);
        info!("Loading atlas for {:?}: {}", species, texture_path);
    }
}

/// Apply visual enhancements to creatures
fn apply_visual_enhancements(
    config: Res<Phase3Config>,
    mut commands: Commands,
    creatures_without_enhancements: Query<
        (Entity, &Genetics, &CreatureType),
        (
            With<Creature>,
            Without<AnimationPlaybackState>,
            Without<ExpressionController>,
            Without<GeneticPattern>,
            Without<CreatureAttachmentPoints>,
        ),
    >,
) {
    for (entity, genetics, creature_type) in creatures_without_enhancements.iter() {
        let mut entity_commands = commands.entity(entity);
        
        // Add animation state
        if config.animations_enabled {
            entity_commands.insert((
                AnimationPlaybackState::default(),
                AnimationLayers::default(),
            ));
        }
        
        // Add expression controller
        if config.expressions_enabled {
            entity_commands.insert(ExpressionController::default());
        }
        
        // Add genetic pattern
        if config.patterns_enabled {
            let pattern = GeneticPattern::from_genetics(genetics);
            entity_commands.insert(pattern);
        }
        
        // Add attachment points
        if config.attachments_enabled {
            entity_commands.insert(CreatureAttachmentPoints::default());
        }
        
        // Add UV mapping component
        let atlas_size = Vec2::new(2048.0, 1024.0); // Standard atlas size
        let sprite_size = Vec2::new(48.0, 48.0);
        entity_commands.insert(AtlasUVMapping::new(
            atlas_size.x,
            atlas_size.y,
            sprite_size,
        ));
        
        info!("Applied Phase 3 enhancements to creature {:?}", entity);
    }
}

/// Main system to update all creature visuals
fn update_creature_visuals(
    config: Res<Phase3Config>,
    time: Res<Time>,
    mut query: Query<(
        &mut CartoonSprite,
        &AnimationPlaybackState,
        &ExpressionController,
        Option<&GeneticPattern>,
        Option<&CreatureAttachmentPoints>,
    )>,
) {
    for (mut sprite, animation, expression, pattern, attachments) in query.iter_mut() {
        // Update animation state
        if config.animations_enabled {
            sprite.base_animation = animation_type_to_state(animation.animation_type);
        }
        
        // Update expression overlay
        if config.expressions_enabled {
            // Expression overlay is already updated by update_expression_overlays system
        }
        
        // Apply pattern modifications
        if let Some(pattern) = pattern {
            if config.patterns_enabled {
                // Pattern is applied via shader/material
            }
        }
        
        // Attachment points are handled by update_attachment_transforms
    }
}

fn animation_type_to_state(anim_type: AnimationType) -> AnimationState {
    match anim_type {
        AnimationType::Idle => AnimationState::Idle,
        AnimationType::Walk => AnimationState::Walk,
        AnimationType::Run => AnimationState::Run,
        AnimationType::Eat => AnimationState::Eat,
        AnimationType::Sleep => AnimationState::Sleep,
        AnimationType::Talk => AnimationState::Talk,
        AnimationType::Attack => AnimationState::Attack,
        AnimationType::Death => AnimationState::Death,
        _ => AnimationState::Idle,
    }
}

// Debug visualization systems

#[cfg(debug_assertions)]
fn debug_animation_states(
    config: Res<Phase3Config>,
    query: Query<(&AnimationPlaybackState, &Transform)>,
    mut gizmos: Gizmos,
) {
    if !config.debug_visualizations {
        return;
    }
    
    for (animation, transform) in query.iter() {
        // Draw animation state above creature
        let text_pos = transform.translation + Vec3::new(0.0, 40.0, 0.0);
        let state_text = format!("{:?}", animation.animation_type);
        
        // Draw a circle at position (gizmos don't support text directly)
        gizmos.circle_2d(
            Vec2::new(text_pos.x, text_pos.y),
            5.0,
            Color::YELLOW,
        );
    }
}

#[cfg(debug_assertions)]
fn debug_expression_states(
    config: Res<Phase3Config>,
    query: Query<(&ExpressionController, &Transform)>,
    mut gizmos: Gizmos,
) {
    if !config.debug_visualizations {
        return;
    }
    
    for (expression, transform) in query.iter() {
        // Draw emotion state
        let emotion_color = match expression.current_emotion {
            EmotionType::Happy => Color::GREEN,
            EmotionType::Sad => Color::BLUE,
            EmotionType::Angry => Color::RED,
            EmotionType::Frightened => Color::PURPLE,
            _ => Color::WHITE,
        };
        
        gizmos.circle_2d(
            Vec2::new(transform.translation.x, transform.translation.y + 30.0),
            3.0,
            emotion_color,
        );
    }
}

#[cfg(debug_assertions)]
fn debug_attachment_points(
    config: Res<Phase3Config>,
    query: Query<(&CreatureAttachmentPoints, &Transform)>,
    mut gizmos: Gizmos,
) {
    if !config.debug_visualizations {
        return;
    }
    
    for (points, transform) in query.iter() {
        // Draw attachment points
        let world_pos = Vec2::new(transform.translation.x, transform.translation.y);
        
        // Head
        let head_pos = world_pos + points.head.base_position;
        gizmos.circle_2d(head_pos, 2.0, Color::CYAN);
        
        // Hands
        let left_hand = world_pos + points.left_hand.base_position;
        let right_hand = world_pos + points.right_hand.base_position;
        gizmos.circle_2d(left_hand, 2.0, Color::LIME_GREEN);
        gizmos.circle_2d(right_hand, 2.0, Color::LIME_GREEN);
        
        // Back
        let back_pos = world_pos + points.back.base_position;
        gizmos.circle_2d(back_pos, 2.0, Color::ORANGE);
    }
}

/// Helper function to spawn a test creature with all Phase 3 features
pub fn spawn_phase3_test_creature(
    commands: &mut Commands,
    position: Vec3,
    creature_type: CreatureType,
    genetics: Genetics,
    texture: Handle<Image>,
    atlas_layout: Handle<TextureAtlasLayout>,
) -> Entity {
    let species = match creature_type {
        CreatureType::Herbivore => CreatureSpecies::Herbivore,
        CreatureType::Carnivore => CreatureSpecies::Carnivore,
        CreatureType::Omnivore => CreatureSpecies::Omnivore,
    };
    
    // Create all Phase 3 components
    let pattern = GeneticPattern::from_genetics(&genetics);
    let expression_controller = ExpressionController::default();
    let attachment_points = CreatureAttachmentPoints::default();
    let animation_state = AnimationPlaybackState::default();
    let animation_layers = AnimationLayers::default();
    
    // Create cartoon sprite with genetic variations
    let mut cartoon_sprite = CartoonSprite::default();
    cartoon_sprite.body_modifiers.size_scale = 0.7 + genetics.size * 0.6;
    cartoon_sprite.body_modifiers.pattern_type = if genetics.pattern > 0.7 {
        PatternType::Stripes
    } else if genetics.pattern > 0.4 {
        PatternType::Spots
    } else {
        PatternType::None
    };
    
    commands.spawn((
        // Basic components
        Creature,
        creature_type,
        genetics,
        Position(position.truncate()),
        Velocity(Vec2::ZERO),
        
        // Sprite components
        SpriteBundle {
            texture,
            transform: Transform::from_translation(position),
            ..default()
        },
        TextureAtlas {
            layout: atlas_layout,
            index: 0,
        },
        cartoon_sprite,
        AnimatedSprite::new(vec![0, 1], 0.3, true),
        
        // Phase 3 components
        animation_state,
        animation_layers,
        expression_controller,
        pattern,
        attachment_points,
        AtlasUVMapping::new(2048.0, 1024.0, Vec2::new(48.0, 48.0)),
    )).id()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_phase3_config_default() {
        let config = Phase3Config::default();
        assert!(config.animations_enabled);
        assert!(config.expressions_enabled);
        assert!(config.patterns_enabled);
        assert!(config.attachments_enabled);
        assert!(!config.debug_visualizations);
    }
    
    #[test]
    fn test_creature_species_conversion() {
        assert_eq!(
            match CreatureType::Herbivore {
                CreatureType::Herbivore => CreatureSpecies::Herbivore,
                CreatureType::Carnivore => CreatureSpecies::Carnivore,
                CreatureType::Omnivore => CreatureSpecies::Omnivore,
            },
            CreatureSpecies::Herbivore
        );
    }
}