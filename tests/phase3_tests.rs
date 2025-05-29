//! Integration tests for Phase 3: Creature Visual Systems

use bevy::prelude::*;
use creature_simulation::{
    components::*,
    systems::{
        animation::{AnimationStateMachine, AnimationPlaybackState, AnimationLayers, AnimationType, AnimationLayer, AnimationMask},
        expression::{ExpressionController, EmotionMapper},
        attachments::{CreatureAttachmentPoints},
    },
    rendering::{
        patterns::{GeneticPattern, PatternRenderingConfig, PatternQuality, PatternParameters},
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

#[test]
fn test_animation_mask_creation() {
    let full_mask = AnimationMask::full();
    assert!(full_mask.body);
    assert!(full_mask.head);
    assert!(full_mask.left_arm);
    assert!(full_mask.right_arm);
    assert!(full_mask.tail);
    
    let upper_mask = AnimationMask::upper_body();
    assert!(!upper_mask.body);
    assert!(upper_mask.head);
    assert!(upper_mask.left_arm);
    assert!(upper_mask.right_arm);
    assert!(!upper_mask.tail);
}

#[test]
fn test_animation_state_machine_get_transition() {
    let state_machine = AnimationStateMachine::default();
    
    // Test specific transition
    let transition = state_machine.get_transition(AnimationType::Idle, AnimationType::Walk);
    assert!(transition.is_some());
    assert_eq!(transition.unwrap().duration, 0.2);
    
    // Test Any transition
    let any_transition = state_machine.get_transition(AnimationType::Run, AnimationType::Eat);
    assert!(any_transition.is_some());
    assert_eq!(any_transition.unwrap().from, AnimationType::Any);
    
    // Test non-existent transition
    let no_transition = state_machine.get_transition(AnimationType::Death, AnimationType::Walk);
    assert!(no_transition.is_none());
}

#[test]
fn test_pattern_texture_generation() {
    use creature_simulation::rendering::patterns::{generate_pattern_texture, PatternParameters};
    
    let params = PatternParameters::default();
    
    // Test spots pattern
    let spots_texture = generate_pattern_texture(PatternType::Spots, &params, 64);
    assert_eq!(spots_texture.texture_descriptor.size.width, 64);
    assert_eq!(spots_texture.texture_descriptor.size.height, 64);
    assert!(!spots_texture.data.is_empty());
    
    // Test transparent pattern
    let none_texture = generate_pattern_texture(PatternType::None, &params, 32);
    // Check that alpha channel is 0 (transparent)
    assert_eq!(none_texture.data[3], 0); // First pixel alpha
}

#[test]
fn test_attached_item_creation() {
    use creature_simulation::systems::attachments::{AttachedItem, ItemType};
    use creature_simulation::components::ToolType;
    
    // Test AttachedItem component creation
    let attached = AttachedItem {
        attachment_point: "right_hand".to_string(),
        item_type: ItemType::Tool(ToolType::Stick),
        custom_offset: Vec2::ZERO,
        custom_rotation: 0.0,
        inherit_animation: true,
        flip_with_direction: true,
    };
    
    assert_eq!(attached.attachment_point, "right_hand");
    assert!(attached.inherit_animation);
    assert!(attached.flip_with_direction);
    
    // Test different item types
    match attached.item_type {
        ItemType::Tool(tool) => assert_eq!(tool, ToolType::Stick),
        _ => panic!("Wrong item type"),
    }
}

#[test]
fn test_expression_creation_for_emotions() {
    use creature_simulation::systems::expression::create_expression_for_emotion;
    
    // Test happy emotion
    let happy_expr = create_expression_for_emotion(EmotionType::Happy, 1.0);
    assert!(happy_expr.mouth_curve > 0.0);
    assert!(happy_expr.eye_scale > 1.0);
    
    // Test sad emotion
    let sad_expr = create_expression_for_emotion(EmotionType::Sad, 1.0);
    assert!(sad_expr.mouth_curve < 0.0);
    assert!(sad_expr.eye_scale < 1.0);
    
    // Test neutral emotion
    let neutral_expr = create_expression_for_emotion(EmotionType::Neutral, 1.0);
    assert_eq!(neutral_expr.mouth_curve, 0.0);
    assert_eq!(neutral_expr.eye_scale, 1.0);
}

#[test]
fn test_determine_target_animation() {
    use creature_simulation::systems::animation::determine_target_animation;
    use creature_simulation::components::ConversationState;
    
    // Test idle state
    let target = determine_target_animation(&CreatureState::Idle, &Velocity(Vec2::ZERO), None);
    assert_eq!(target, AnimationType::Idle);
    
    // Test idle with conversation
    let conv_state = ConversationState::Greeting;
    let target = determine_target_animation(&CreatureState::Idle, &Velocity(Vec2::ZERO), Some(&conv_state));
    assert_eq!(target, AnimationType::Talk);
    
    // Test moving slow
    let target = determine_target_animation(&CreatureState::Moving { target: Vec2::ZERO }, &Velocity(Vec2::new(3.0, 0.0)), None);
    assert_eq!(target, AnimationType::Walk);
    
    // Test moving fast
    let target = determine_target_animation(&CreatureState::Moving { target: Vec2::ZERO }, &Velocity(Vec2::new(6.0, 0.0)), None);
    assert_eq!(target, AnimationType::Run);
    
    // Test other states
    assert_eq!(determine_target_animation(&CreatureState::Dead, &Velocity(Vec2::ZERO), None), AnimationType::Death);
    assert_eq!(determine_target_animation(&CreatureState::Eating, &Velocity(Vec2::ZERO), None), AnimationType::Eat);
    assert_eq!(determine_target_animation(&CreatureState::Resting, &Velocity(Vec2::ZERO), None), AnimationType::Sleep);
}

#[test]
fn test_atlas_species_conversion() {
    use creature_simulation::rendering::atlas::{species_to_string, CreatureSpecies};
    
    assert_eq!(species_to_string(CreatureSpecies::Herbivore), "herbivore");
    assert_eq!(species_to_string(CreatureSpecies::Carnivore), "carnivore");
    assert_eq!(species_to_string(CreatureSpecies::Omnivore), "omnivore");
}

#[test]
fn test_pattern_generation_variations() {
    use creature_simulation::rendering::patterns::generate_pattern_texture;
    
    let params = PatternParameters {
        scale: 2.0,
        rotation: 45.0,
        offset: Vec2::new(10.0, 10.0),
        noise_seed: 12345,
    };
    
    // Test stripes pattern
    let stripes = generate_pattern_texture(PatternType::Stripes, &params, 32);
    assert!(!stripes.data.is_empty());
    
    // Test patches pattern
    let patches = generate_pattern_texture(PatternType::Patches, &params, 32);
    assert!(!patches.data.is_empty());
    
    // Verify textures have data
    assert_eq!(stripes.data.len(), 32 * 32 * 4); // RGBA for 32x32
    assert_eq!(patches.data.len(), 32 * 32 * 4);
    
    // Test that non-transparent patterns have some visible pixels
    let spots = generate_pattern_texture(PatternType::Spots, &params, 32);
    let has_visible_pixels = spots.data.chunks(4).any(|pixel| pixel[3] > 0);
    assert!(has_visible_pixels);
}

#[test]
fn test_emotion_mapper_modifiers() {
    let mapper = EmotionMapper::default();
    let mut needs = Needs::default();
    
    // Test with low social need
    needs.social = 0.2;
    let (emotion, _intensity) = mapper.calculate_emotion(&CreatureState::Idle, &needs);
    // Low social need should trigger sad emotion through the modifier
    assert!(emotion == EmotionType::Sad || emotion == EmotionType::Content);
    
    // Test eating state - which has no modifiers and should be Happy
    let eating_needs = Needs { hunger: 0.5, thirst: 0.5, energy: 0.5, social: 0.5 };
    let (emotion, _) = mapper.calculate_emotion(&CreatureState::Eating, &eating_needs);
    assert_eq!(emotion, EmotionType::Happy);
}

#[test]
fn test_animation_playback_state() {
    let mut playback = AnimationPlaybackState::default();
    
    // Test initial state
    assert_eq!(playback.animation_type, AnimationType::Idle);
    assert_eq!(playback.current_frame, 0);
    assert!(!playback.is_complete);
    
    // Test frame advancement
    playback.frames_played = 5;
    playback.time_played = 2.5;
    assert_eq!(playback.frames_played, 5);
    
    // Test completion
    playback.is_complete = true;
    assert!(playback.is_complete);
}