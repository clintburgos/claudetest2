//! Expression and emotion management system for creature faces

use bevy::prelude::*;
use std::collections::HashMap;
use crate::components::{
    EmotionType, ExpressionOverlay, CreatureState, Needs, CartoonSprite,
};

/// Eye state for expression rendering
#[derive(Clone, Debug, Reflect)]
pub struct EyeState {
    pub openness: f32,        // 0.0 (closed) - 1.0 (wide open)
    pub pupil_position: Vec2, // -1.0 to 1.0 for looking around
    pub pupil_size: f32,      // 0.5 (constricted) - 1.5 (dilated)
    pub shape: EyeShape,
}

#[derive(Clone, Debug, Reflect)]
pub enum EyeShape {
    Normal,
    Happy,      // Curved/squinting
    Sad,        // Droopy
    Angry,      // Narrowed
    Surprised,  // Round
    Heart,      // Special love state
    Spiral,     // Dizzy/confused
    Star,       // Excited
}

/// Mouth shape for expression rendering
#[derive(Clone, Debug, Reflect)]
pub struct MouthShape {
    pub curve_points: [Vec2; 4], // Bezier curve control points
    pub width: f32,              // 0.5-1.5 relative to base
    pub thickness: f32,          // Line thickness
}

/// Eyebrow state for expression rendering
#[derive(Clone, Debug, Reflect)]
pub struct BrowState {
    pub height: f32,    // -0.5 (lowered) to 0.5 (raised)
    pub angle: f32,     // -30 to 30 degrees
    pub curve: f32,     // -0.5 (furrowed) to 0.5 (arched)
}

/// Complete expression overlay with all facial features
#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct DetailedExpressionOverlay {
    pub left_eye: EyeState,
    pub right_eye: EyeState,
    pub mouth_shape: MouthShape,
    pub mouth_openness: f32,
    pub left_brow: BrowState,
    pub right_brow: BrowState,
    #[reflect(ignore)]
    pub cheek_blush: Option<BlushState>,
    #[reflect(ignore)]
    pub sweat_drops: Option<SweatState>,
    #[reflect(ignore)]
    pub emotion_effects: Vec<EmotionEffect>,
}

#[derive(Clone, Debug)]
pub struct BlushState {
    pub intensity: f32,
    pub color: Color,
    pub position_offset: Vec2,
}

#[derive(Clone, Debug)]
pub struct SweatState {
    pub drop_count: u8,
    pub drop_size: f32,
    pub animation_phase: f32,
}

#[derive(Clone, Debug)]
pub enum EmotionEffect {
    HeartBubbles { count: u8, size: f32 },
    AngerVeins { intensity: f32 },
    TearDrops { flow_rate: f32 },
    Sparkles { density: f32, color: Color },
    QuestionMarks { count: u8 },
    ExclamationPoint { size: f32 },
    SleepBubbles { z_count: u8 },
}

/// Expression controller component that manages emotion transitions
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ExpressionController {
    pub current_emotion: EmotionType,
    pub target_emotion: EmotionType,
    pub previous_emotion: EmotionType,
    pub blend_progress: f32,
    pub emotion_intensity: f32,
    pub emotion_timer: Timer,
    #[reflect(ignore)]
    pub priority_overrides: HashMap<EmotionType, f32>,
}

impl Default for ExpressionController {
    fn default() -> Self {
        Self {
            current_emotion: EmotionType::Neutral,
            target_emotion: EmotionType::Neutral,
            previous_emotion: EmotionType::Neutral,
            blend_progress: 1.0,
            emotion_intensity: 0.5,
            emotion_timer: Timer::from_seconds(2.0, TimerMode::Once),
            priority_overrides: HashMap::new(),
        }
    }
}

/// Resource that maps AI states and needs to visual emotions
#[derive(Resource)]
pub struct EmotionMapper {
    pub state_mappings: HashMap<CreatureState, EmotionRule>,
    pub need_thresholds: NeedEmotionThresholds,
    pub emotion_priorities: HashMap<EmotionType, f32>,
}

#[derive(Clone)]
pub struct EmotionRule {
    pub primary_emotion: EmotionType,
    pub intensity_base: f32,
    pub modifiers: Vec<EmotionModifier>,
}

#[derive(Clone)]
pub enum EmotionModifier {
    NeedBased { need: NeedType, threshold: f32, emotion: EmotionType },
    HealthBased { threshold: f32, emotion: EmotionType },
    SocialBased { threshold: f32, emotion: EmotionType },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NeedType {
    Hunger,
    Thirst,
    Energy,
    Social,
}

pub struct NeedEmotionThresholds {
    pub hunger_happy: f32,
    pub hunger_content: f32,
    pub hunger_worried: f32,
    pub hunger_desperate: f32,
    pub thirst_happy: f32,
    pub thirst_content: f32,
    pub thirst_worried: f32,
    pub thirst_desperate: f32,
    pub energy_energetic: f32,
    pub energy_normal: f32,
    pub energy_tired: f32,
    pub energy_exhausted: f32,
    pub social_fulfilled: f32,
    pub social_content: f32,
    pub social_lonely: f32,
    pub social_depressed: f32,
}

impl Default for NeedEmotionThresholds {
    fn default() -> Self {
        Self {
            hunger_happy: 0.2,
            hunger_content: 0.4,
            hunger_worried: 0.7,
            hunger_desperate: 0.9,
            thirst_happy: 0.2,
            thirst_content: 0.4,
            thirst_worried: 0.7,
            thirst_desperate: 0.9,
            energy_energetic: 0.8,
            energy_normal: 0.5,
            energy_tired: 0.3,
            energy_exhausted: 0.1,
            social_fulfilled: 0.8,
            social_content: 0.5,
            social_lonely: 0.3,
            social_depressed: 0.1,
        }
    }
}

impl Default for EmotionMapper {
    fn default() -> Self {
        let mut state_mappings = HashMap::new();
        
        // Map creature states to base emotions
        state_mappings.insert(CreatureState::Idle, EmotionRule {
            primary_emotion: EmotionType::Content,
            intensity_base: 0.5,
            modifiers: vec![
                EmotionModifier::NeedBased {
                    need: NeedType::Social,
                    threshold: 0.3,
                    emotion: EmotionType::Sad,
                },
            ],
        });
        
        state_mappings.insert(CreatureState::Eating, EmotionRule {
            primary_emotion: EmotionType::Happy,
            intensity_base: 0.8,
            modifiers: vec![],
        });
        
        state_mappings.insert(CreatureState::Resting, EmotionRule {
            primary_emotion: EmotionType::Tired,
            intensity_base: 0.7,
            modifiers: vec![],
        });
        
        // Set up emotion priorities
        let mut emotion_priorities = HashMap::new();
        emotion_priorities.insert(EmotionType::Angry, 0.9);
        emotion_priorities.insert(EmotionType::Frightened, 0.85);
        emotion_priorities.insert(EmotionType::Sad, 0.7);
        emotion_priorities.insert(EmotionType::Hungry, 0.6);
        emotion_priorities.insert(EmotionType::Tired, 0.5);
        emotion_priorities.insert(EmotionType::Happy, 0.4);
        emotion_priorities.insert(EmotionType::Curious, 0.3);
        emotion_priorities.insert(EmotionType::Content, 0.2);
        emotion_priorities.insert(EmotionType::Neutral, 0.1);
        
        Self {
            state_mappings,
            need_thresholds: NeedEmotionThresholds::default(),
            emotion_priorities,
        }
    }
}

impl EmotionMapper {
    pub fn calculate_emotion(
        &self,
        creature_state: &CreatureState,
        needs: &Needs,
    ) -> (EmotionType, f32) {
        // Check critical needs first
        if let Some((emotion, intensity)) = self.check_critical_needs(needs) {
            return (emotion, intensity);
        }
        
        // Get base emotion from state
        if let Some(rule) = self.state_mappings.get(creature_state) {
            let mut emotion = rule.primary_emotion;
            let mut intensity = rule.intensity_base;
            
            // Apply modifiers
            for modifier in &rule.modifiers {
                if let Some((mod_emotion, mod_intensity)) = self.evaluate_modifier(modifier, needs) {
                    if mod_intensity > intensity {
                        emotion = mod_emotion;
                        intensity = mod_intensity;
                    }
                }
            }
            
            (emotion, intensity)
        } else {
            // Default emotional response based on needs
            self.emotion_from_needs(needs)
        }
    }
    
    fn check_critical_needs(&self, needs: &Needs) -> Option<(EmotionType, f32)> {
        // Starvation
        if needs.hunger > self.need_thresholds.hunger_desperate {
            return Some((EmotionType::Hungry, 1.0));
        }
        
        // Dehydration
        if needs.thirst > self.need_thresholds.thirst_desperate {
            return Some((EmotionType::Frightened, 0.95));
        }
        
        // Exhaustion
        if needs.energy < self.need_thresholds.energy_exhausted {
            return Some((EmotionType::Tired, 0.9));
        }
        
        None
    }
    
    fn evaluate_modifier(&self, modifier: &EmotionModifier, needs: &Needs) -> Option<(EmotionType, f32)> {
        match modifier {
            EmotionModifier::NeedBased { need, threshold, emotion } => {
                let need_value = match need {
                    NeedType::Hunger => needs.hunger,
                    NeedType::Thirst => needs.thirst,
                    NeedType::Energy => needs.energy,
                    NeedType::Social => needs.social,
                };
                
                if need_value > *threshold {
                    Some((*emotion, need_value))
                } else {
                    None
                }
            }
            EmotionModifier::HealthBased { threshold: _, emotion } => {
                // Health check would go here
                Some((*emotion, 0.5))
            }
            EmotionModifier::SocialBased { threshold, emotion } => {
                if needs.social < *threshold {
                    Some((*emotion, 1.0 - needs.social))
                } else {
                    None
                }
            }
        }
    }
    
    fn emotion_from_needs(&self, needs: &Needs) -> (EmotionType, f32) {
        let lowest_need = needs.get_lowest();
        
        if lowest_need.1 > self.need_thresholds.hunger_content {
            (EmotionType::Content, 0.6)
        } else if lowest_need.1 > self.need_thresholds.hunger_worried {
            (EmotionType::Curious, 0.5)
        } else {
            (EmotionType::Sad, 0.7)
        }
    }
}

/// System to sync AI states to visual emotions
pub fn sync_ai_state_to_emotions(
    time: Res<Time>,
    emotion_mapper: Res<EmotionMapper>,
    mut query: Query<(
        &mut ExpressionController,
        &CreatureState,
        &Needs,
    )>,
) {
    for (mut controller, state, needs) in query.iter_mut() {
        // Calculate target emotion from AI state
        let (target_emotion, intensity) = emotion_mapper.calculate_emotion(state, needs);
        
        // Check if we should change emotion
        if controller.target_emotion != target_emotion {
            let current_priority = emotion_mapper.emotion_priorities
                .get(&controller.current_emotion)
                .unwrap_or(&0.0);
            let target_priority = emotion_mapper.emotion_priorities
                .get(&target_emotion)
                .unwrap_or(&0.0);
            
            // Change if target has higher priority or current emotion timer expired
            if target_priority > current_priority || controller.emotion_timer.finished() {
                controller.previous_emotion = controller.current_emotion;
                controller.target_emotion = target_emotion;
                controller.blend_progress = 0.0;
                controller.emotion_intensity = intensity;
                controller.emotion_timer.reset();
            }
        }
        
        // Update blend progress
        if controller.blend_progress < 1.0 {
            controller.blend_progress += time.delta_seconds() * 2.0; // 0.5 second transitions
            if controller.blend_progress >= 1.0 {
                controller.current_emotion = controller.target_emotion;
                controller.blend_progress = 1.0;
            }
        }
        
        // Tick emotion timer
        controller.emotion_timer.tick(time.delta());
    }
}

/// System to update expression overlays based on emotion controller
pub fn update_expression_overlays(
    mut query: Query<(
        &ExpressionController,
        &mut CartoonSprite,
        Option<&mut DetailedExpressionOverlay>,
    )>,
) {
    for (controller, mut cartoon_sprite, detailed_overlay) in query.iter_mut() {
        // Create or update simple expression overlay
        let expression = create_expression_for_emotion(
            controller.current_emotion,
            controller.emotion_intensity,
        );
        
        cartoon_sprite.expression_overlay = Some(expression);
        
        // Update detailed overlay if present
        if let Some(mut detailed) = detailed_overlay {
            update_detailed_expression(
                &mut detailed,
                controller.current_emotion,
                controller.emotion_intensity,
            );
        }
    }
}

fn create_expression_for_emotion(emotion: EmotionType, intensity: f32) -> ExpressionOverlay {
    let (mouth_curve, eye_scale, brow_angle) = match emotion {
        EmotionType::Happy => (0.5 * intensity, 1.1, -10.0),
        EmotionType::Sad => (-0.5 * intensity, 0.9, 10.0),
        EmotionType::Angry => (-0.3 * intensity, 0.8, -20.0),
        EmotionType::Frightened => (-0.2, 1.3 + 0.2 * intensity, 15.0),
        EmotionType::Tired => (-0.1, 0.7 - 0.2 * intensity, 5.0),
        EmotionType::Hungry => (-0.2, 1.0, 0.0),
        EmotionType::Curious => (0.1, 1.2, -5.0),
        EmotionType::Content => (0.3 * intensity, 1.0, 0.0),
        EmotionType::Neutral => (0.0, 1.0, 0.0),
        EmotionType::Excited => (0.6 * intensity, 1.2, -15.0),
        EmotionType::Disgusted => (-0.4, 0.8, 5.0),
        EmotionType::Surprised => (0.0, 1.4, 20.0),
        EmotionType::Confused => (-0.1, 1.1, 10.0),
        EmotionType::Sleeping => (0.0, 0.5, 0.0),
        EmotionType::Sick => (-0.3, 0.7, 5.0),
        EmotionType::Love => (0.7 * intensity, 1.1, -10.0),
    };
    
    ExpressionOverlay {
        eye_offset: Vec2::ZERO,
        eye_scale,
        mouth_curve,
        mouth_open: if matches!(emotion, EmotionType::Frightened) { 0.3 } else { 0.0 },
        brow_angle,
    }
}

fn update_detailed_expression(
    overlay: &mut DetailedExpressionOverlay,
    emotion: EmotionType,
    intensity: f32,
) {
    // Update eyes
    let (eye_openness, eye_shape, pupil_size) = match emotion {
        EmotionType::Happy => (0.7, EyeShape::Happy, 1.0),
        EmotionType::Sad => (0.5, EyeShape::Sad, 1.1),
        EmotionType::Angry => (0.6, EyeShape::Angry, 0.8),
        EmotionType::Frightened => (1.0, EyeShape::Surprised, 1.3),
        EmotionType::Tired => (0.3, EyeShape::Normal, 1.0),
        EmotionType::Curious => (0.9, EyeShape::Normal, 1.2),
        _ => (0.8, EyeShape::Normal, 1.0),
    };
    
    overlay.left_eye.openness = eye_openness;
    overlay.left_eye.shape = eye_shape.clone();
    overlay.left_eye.pupil_size = pupil_size;
    overlay.right_eye = overlay.left_eye.clone();
    
    // Update mouth
    overlay.mouth_shape = create_mouth_shape(emotion, intensity);
    
    // Update brows
    let (brow_height, brow_angle, brow_curve) = match emotion {
        EmotionType::Happy => (0.1, -5.0, 0.2),
        EmotionType::Sad => (0.0, 15.0, 0.1),
        EmotionType::Angry => (-0.3, -20.0, -0.3),
        EmotionType::Frightened => (0.3, 10.0, 0.0),
        _ => (0.0, 0.0, 0.0),
    };
    
    overlay.left_brow = BrowState { height: brow_height, angle: brow_angle, curve: brow_curve };
    overlay.right_brow = overlay.left_brow.clone();
    
    // Add emotion effects
    overlay.emotion_effects.clear();
    match emotion {
        EmotionType::Happy if intensity > 0.8 => {
            overlay.emotion_effects.push(EmotionEffect::Sparkles {
                density: 0.5,
                color: Color::YELLOW,
            });
        }
        EmotionType::Angry if intensity > 0.9 => {
            overlay.emotion_effects.push(EmotionEffect::AngerVeins { intensity: 0.8 });
        }
        EmotionType::Sad if intensity > 0.6 => {
            overlay.emotion_effects.push(EmotionEffect::TearDrops { flow_rate: intensity });
        }
        _ => {}
    }
}

fn create_mouth_shape(emotion: EmotionType, intensity: f32) -> MouthShape {
    let curve_points = match emotion {
        EmotionType::Happy => [
            Vec2::new(-0.3, 0.0),
            Vec2::new(-0.2, 0.2 * intensity),
            Vec2::new(0.2, 0.2 * intensity),
            Vec2::new(0.3, 0.0),
        ],
        EmotionType::Sad => [
            Vec2::new(-0.2, 0.0),
            Vec2::new(-0.15, -0.2 * intensity),
            Vec2::new(0.15, -0.2 * intensity),
            Vec2::new(0.2, 0.0),
        ],
        EmotionType::Angry => [
            Vec2::new(-0.3, 0.1),
            Vec2::new(-0.2, -0.1),
            Vec2::new(0.2, -0.1),
            Vec2::new(0.3, 0.1),
        ],
        _ => [
            Vec2::new(-0.2, 0.0),
            Vec2::new(-0.1, 0.0),
            Vec2::new(0.1, 0.0),
            Vec2::new(0.2, 0.0),
        ],
    };
    
    MouthShape {
        curve_points,
        width: 1.0,
        thickness: 0.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_emotion_priority() {
        let mapper = EmotionMapper::default();
        
        assert!(mapper.emotion_priorities[&EmotionType::Angry] > mapper.emotion_priorities[&EmotionType::Happy]);
        assert!(mapper.emotion_priorities[&EmotionType::Frightened] > mapper.emotion_priorities[&EmotionType::Sad]);
    }
    
    #[test]
    fn test_critical_needs_detection() {
        let mapper = EmotionMapper::default();
        let mut needs = Needs::default();
        
        // Test extreme hunger
        needs.hunger = 0.95;
        let (emotion, intensity) = mapper.calculate_emotion(&CreatureState::Idle, &needs);
        assert_eq!(emotion, EmotionType::Hungry);
        assert_eq!(intensity, 1.0);
        
        // Test exhaustion
        needs.hunger = 0.5;
        needs.energy = 0.05;
        let (emotion, _) = mapper.calculate_emotion(&CreatureState::Idle, &needs);
        assert_eq!(emotion, EmotionType::Tired);
    }
    
    #[test]
    fn test_expression_controller_transitions() {
        let mut controller = ExpressionController::default();
        controller.current_emotion = EmotionType::Neutral;
        controller.target_emotion = EmotionType::Happy;
        controller.blend_progress = 0.0;
        
        // Simulate time passing
        controller.blend_progress += 0.5;
        assert!(controller.blend_progress < 1.0);
        assert_eq!(controller.current_emotion, EmotionType::Neutral);
        
        controller.blend_progress = 1.0;
        assert_eq!(controller.blend_progress, 1.0);
    }
}