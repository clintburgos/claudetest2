//! Integration tests for Phase 3: Creature Visual Systems

use bevy::prelude::*;
use creature_simulation::{
    components::*,
    plugins::Phase3VisualsPlugin,
    systems::{
        animation::{AnimationStateMachine, AnimationPlaybackState, AnimationLayers, AnimationType, AnimationLayer, AnimationMask},
        expression::{ExpressionController, EmotionMapper},
        attachments::{CreatureAttachmentPoints},
    },
    rendering::{
        patterns::{GeneticPattern, PatternRenderingConfig, PatternQuality},
        atlas::{AtlasManager, AtlasUVMapping},
    },
};

#[test]
fn test_phase3_plugin_loads() {
    // Test that resources can be created without full app context
    let state_machine = AnimationStateMachine::default();
    let emotion_mapper = EmotionMapper::default();
    let pattern_config = PatternRenderingConfig::default();
    let atlas_manager = AtlasManager::default();
    
    // Check they initialize correctly
    assert!(!state_machine.transitions.is_empty());
    assert!(!emotion_mapper.emotion_priorities.is_empty());
    assert_eq!(pattern_config.pattern_quality, PatternQuality::Medium);
    assert!(atlas_manager.creature_atlases.is_empty());
}

#[test]
fn test_animation_state_machine() {
    let state_machine = AnimationStateMachine::default();
    let mut current = AnimationPlaybackState::default();
    
    // Test idle to walk transition
    let can_transition = state_machine.can_transition(
        &current,
        AnimationType::Walk,
        &CreatureState::Moving { target: Vec2::ZERO },
        0.0,
    );
    assert!(can_transition);
    
    // Test priority system - death should always be allowed
    current.animation_type = AnimationType::Attack;
    let can_transition = state_machine.can_transition(
        &current,
        AnimationType::Death,
        &CreatureState::Dead,
        0.0,
    );
    assert!(can_transition);
}

#[test]
fn test_expression_controller() {
    let mut controller = ExpressionController::default();
    assert_eq!(controller.current_emotion, EmotionType::Neutral);
    
    // Test emotion transition
    controller.target_emotion = EmotionType::Happy;
    controller.blend_progress = 0.5;
    assert_eq!(controller.current_emotion, EmotionType::Neutral); // Still blending
    
    controller.blend_progress = 1.0;
    controller.current_emotion = controller.target_emotion;
    assert_eq!(controller.current_emotion, EmotionType::Happy);
}

#[test]
fn test_genetic_pattern_generation() {
    let mut genetics = Genetics::default();
    
    // Test stripe pattern
    genetics.pattern = 0.8;
    let pattern = GeneticPattern::from_genetics(&genetics);
    assert_eq!(pattern.pattern_type, PatternType::Stripes);
    
    // Test spot pattern
    genetics.pattern = 0.5;
    let pattern = GeneticPattern::from_genetics(&genetics);
    assert_eq!(pattern.pattern_type, PatternType::Spots);
    
    // Test pattern intensity
    assert!(pattern.intensity > 0.3);
    assert!(pattern.intensity < 0.8);
}

#[test]
fn test_attachment_points() {
    let points = CreatureAttachmentPoints::default();
    
    // Test default attachment points exist
    assert_eq!(points.head.name, "head");
    assert_eq!(points.left_hand.name, "left_hand");
    assert_eq!(points.right_hand.name, "right_hand");
    assert_eq!(points.back.name, "back");
    assert_eq!(points.waist.name, "waist");
    
    // Test attachment point positions
    assert_eq!(points.head.base_position, Vec2::new(0.0, 20.0));
    assert_eq!(points.left_hand.base_position, Vec2::new(-12.0, 5.0));
}

#[test]
fn test_emotion_mapper() {
    let mapper = EmotionMapper::default();
    let mut needs = Needs::default();
    
    // Test happy when eating
    let (emotion, intensity) = mapper.calculate_emotion(&CreatureState::Eating, &needs);
    assert_eq!(emotion, EmotionType::Happy);
    assert!(intensity > 0.5);
    
    // Test tired when resting
    let (emotion, _) = mapper.calculate_emotion(&CreatureState::Resting, &needs);
    assert_eq!(emotion, EmotionType::Tired);
    
    // Test critical hunger
    needs.hunger = 0.95;
    let (emotion, intensity) = mapper.calculate_emotion(&CreatureState::Idle, &needs);
    assert_eq!(emotion, EmotionType::Hungry);
    assert_eq!(intensity, 1.0);
}

#[test]
fn test_atlas_uv_mapping() {
    let uv_mapping = AtlasUVMapping::new(2048.0, 1024.0, Vec2::new(48.0, 48.0));
    
    // Test UV calculation for idle animation
    use creature_simulation::rendering::atlas::{AnimationType as AtlasAnimationType, ExpressionType};
    let uv = uv_mapping.get_animation_uv(AtlasAnimationType::Idle, 0, 0);
    assert_eq!(uv.min, Vec2::new(0.0, 0.0));
    assert!(uv.max.x > 0.0);
    assert!(uv.max.y > 0.0);
    
    // Test expression UV
    let expr_uv = uv_mapping.get_expression_uv(ExpressionType::Happy);
    assert!(expr_uv.min.x > 0.0); // Should be in column 1
}

#[test]
fn test_animation_layers() {
    let mut layers = AnimationLayers::default();
    
    // Test default base layer
    assert_eq!(layers.base_layer.animation, AnimationType::Idle);
    assert_eq!(layers.base_layer.weight, 1.0);
    
    // Test adding overlay
    layers.overlay_layer = Some(AnimationLayer {
        animation: AnimationType::Talk,
        weight: 0.8,
        speed_multiplier: 1.0,
        time_offset: 0.0,
        mask: AnimationMask::upper_body(),
    });
    
    assert!(layers.overlay_layer.is_some());
    assert_eq!(layers.overlay_layer.as_ref().unwrap().animation, AnimationType::Talk);
}

#[test]
fn test_pattern_quality_settings() {
    let config = PatternRenderingConfig::default();
    assert!(config.patterns_enabled);
    assert_eq!(config.pattern_quality, PatternQuality::Medium);
    assert_eq!(config.max_pattern_complexity, 100);
}

// Test complete creature visual setup
#[test]
fn test_complete_visual_creature() {
    // Test that all components can be created
    let cartoon_sprite = CartoonSprite::default();
    let animation_state = AnimationPlaybackState::default();
    let animation_layers = AnimationLayers::default();
    let expression_controller = ExpressionController::default();
    let attachment_points = CreatureAttachmentPoints::default();
    
    // Test default values
    assert_eq!(cartoon_sprite.base_animation, AnimationState::Idle);
    assert_eq!(animation_state.animation_type, AnimationType::Idle);
    assert_eq!(animation_layers.base_layer.animation, AnimationType::Idle);
    assert_eq!(expression_controller.current_emotion, EmotionType::Neutral);
    assert!(attachment_points.head.base_position.y > 0.0);
}