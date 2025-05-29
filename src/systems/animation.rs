//! Animation state machine and blending system for creature visuals

use bevy::prelude::*;
use std::collections::HashMap;
use crate::components::{
    AnimationState, CartoonSprite, EmotionType, Velocity, CreatureState,
    AnimatedSprite, BodyModifiers, PatternType,
};

/// Animation type enumeration with all possible animations
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
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
    Shiver,
    Bounce,
    Any, // Special case for transitions from any state
}

/// Animation transition rules and priorities
#[derive(Clone, Debug)]
pub struct AnimationTransition {
    pub from: AnimationType,
    pub to: AnimationType,
    pub duration: f32,
    pub blend_curve: AnimationCurve,
    pub priority: u8,
    pub conditions: Vec<TransitionCondition>,
}

#[derive(Clone, Debug)]
pub enum TransitionCondition {
    Immediate,
    OnAnimationComplete,
    AfterFrames(usize),
    WithMinDuration(f32),
    VelocityThreshold(f32),
    StateRequired(CreatureState),
}

#[derive(Clone, Debug)]
pub enum AnimationCurve {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Custom(Vec<(f32, f32)>),
}

/// Current animation playback state
#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct AnimationPlaybackState {
    pub animation_type: AnimationType,
    pub current_frame: usize,
    pub time_played: f32,
    pub frames_played: usize,
    pub is_complete: bool,
    pub transition_progress: Option<f32>,
}

impl Default for AnimationPlaybackState {
    fn default() -> Self {
        Self {
            animation_type: AnimationType::Idle,
            current_frame: 0,
            time_played: 0.0,
            frames_played: 0,
            is_complete: false,
            transition_progress: None,
        }
    }
}

/// Resource containing the animation state machine configuration
#[derive(Resource)]
pub struct AnimationStateMachine {
    pub transitions: HashMap<(AnimationType, AnimationType), AnimationTransition>,
    pub interrupt_priorities: HashMap<AnimationType, u8>,
}

impl Default for AnimationStateMachine {
    fn default() -> Self {
        let mut transitions = HashMap::new();
        
        // Idle transitions
        transitions.insert((AnimationType::Idle, AnimationType::Walk), AnimationTransition {
            from: AnimationType::Idle,
            to: AnimationType::Walk,
            duration: 0.2,
            blend_curve: AnimationCurve::EaseIn,
            priority: 5,
            conditions: vec![TransitionCondition::Immediate],
        });
        
        transitions.insert((AnimationType::Idle, AnimationType::Run), AnimationTransition {
            from: AnimationType::Idle,
            to: AnimationType::Run,
            duration: 0.15,
            blend_curve: AnimationCurve::EaseIn,
            priority: 6,
            conditions: vec![TransitionCondition::VelocityThreshold(5.0)],
        });
        
        // Walk transitions
        transitions.insert((AnimationType::Walk, AnimationType::Run), AnimationTransition {
            from: AnimationType::Walk,
            to: AnimationType::Run,
            duration: 0.25,
            blend_curve: AnimationCurve::Linear,
            priority: 6,
            conditions: vec![
                TransitionCondition::AfterFrames(2),
                TransitionCondition::VelocityThreshold(5.0),
            ],
        });
        
        transitions.insert((AnimationType::Walk, AnimationType::Idle), AnimationTransition {
            from: AnimationType::Walk,
            to: AnimationType::Idle,
            duration: 0.3,
            blend_curve: AnimationCurve::EaseOut,
            priority: 4,
            conditions: vec![TransitionCondition::AfterFrames(4)],
        });
        
        // Action transitions
        transitions.insert((AnimationType::Any, AnimationType::Eat), AnimationTransition {
            from: AnimationType::Any,
            to: AnimationType::Eat,
            duration: 0.2,
            blend_curve: AnimationCurve::EaseInOut,
            priority: 8,
            conditions: vec![TransitionCondition::StateRequired(CreatureState::Eating)],
        });
        
        transitions.insert((AnimationType::Any, AnimationType::Attack), AnimationTransition {
            from: AnimationType::Any,
            to: AnimationType::Attack,
            duration: 0.1,
            blend_curve: AnimationCurve::EaseIn,
            priority: 10,
            conditions: vec![TransitionCondition::Immediate],
        });
        
        transitions.insert((AnimationType::Any, AnimationType::Death), AnimationTransition {
            from: AnimationType::Any,
            to: AnimationType::Death,
            duration: 0.0,
            blend_curve: AnimationCurve::Linear,
            priority: 11,
            conditions: vec![TransitionCondition::Immediate],
        });
        
        // Sleep transitions
        transitions.insert((AnimationType::Idle, AnimationType::Sleep), AnimationTransition {
            from: AnimationType::Idle,
            to: AnimationType::Sleep,
            duration: 1.0,
            blend_curve: AnimationCurve::EaseOut,
            priority: 3,
            conditions: vec![
                TransitionCondition::OnAnimationComplete,
                TransitionCondition::StateRequired(CreatureState::Resting),
            ],
        });
        
        let interrupt_priorities = HashMap::from([
            (AnimationType::Idle, 1),
            (AnimationType::Walk, 2),
            (AnimationType::Run, 3),
            (AnimationType::Talk, 4),
            (AnimationType::Sleep, 2),
            (AnimationType::Eat, 5),
            (AnimationType::Attack, 10),
            (AnimationType::Death, 11),
            (AnimationType::UseItem, 6),
            (AnimationType::Shiver, 1),
            (AnimationType::Bounce, 1),
        ]);
        
        Self {
            transitions,
            interrupt_priorities,
        }
    }
}

impl AnimationStateMachine {
    pub fn can_transition(
        &self,
        current: &AnimationPlaybackState,
        target: AnimationType,
        creature_state: &CreatureState,
        velocity: f32,
    ) -> bool {
        let key = (current.animation_type, target);
        let any_key = (AnimationType::Any, target);
        
        if let Some(transition) = self.transitions.get(&key)
            .or_else(|| self.transitions.get(&any_key)) {
            
            // Check priority
            let current_priority = self.interrupt_priorities.get(&current.animation_type).unwrap_or(&0);
            if transition.priority <= *current_priority && !current.is_complete {
                return false;
            }
            
            // Check conditions
            for condition in &transition.conditions {
                match condition {
                    TransitionCondition::Immediate => continue,
                    TransitionCondition::OnAnimationComplete => {
                        if !current.is_complete {
                            return false;
                        }
                    }
                    TransitionCondition::AfterFrames(frames) => {
                        if current.frames_played < *frames {
                            return false;
                        }
                    }
                    TransitionCondition::WithMinDuration(duration) => {
                        if current.time_played < *duration {
                            return false;
                        }
                    }
                    TransitionCondition::VelocityThreshold(threshold) => {
                        if velocity < *threshold {
                            return false;
                        }
                    }
                    TransitionCondition::StateRequired(required_state) => {
                        if creature_state != required_state {
                            return false;
                        }
                    }
                }
            }
            
            true
        } else {
            false
        }
    }
    
    pub fn get_transition(&self, from: AnimationType, to: AnimationType) -> Option<&AnimationTransition> {
        self.transitions.get(&(from, to))
            .or_else(|| self.transitions.get(&(AnimationType::Any, to)))
    }
}

/// Multi-layer animation blending system
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct AnimationLayers {
    pub base_layer: AnimationLayer,
    pub overlay_layer: Option<AnimationLayer>,
    pub additive_layers: Vec<AnimationLayer>,
    pub blend_weights: HashMap<AnimationType, f32>,
}

impl Default for AnimationLayers {
    fn default() -> Self {
        Self {
            base_layer: AnimationLayer {
                animation: AnimationType::Idle,
                weight: 1.0,
                speed_multiplier: 1.0,
                time_offset: 0.0,
                mask: AnimationMask::full(),
            },
            overlay_layer: None,
            additive_layers: Vec::new(),
            blend_weights: HashMap::new(),
        }
    }
}

#[derive(Clone, Debug, Reflect)]
pub struct AnimationLayer {
    pub animation: AnimationType,
    pub weight: f32,
    pub speed_multiplier: f32,
    pub time_offset: f32,
    pub mask: AnimationMask,
}

#[derive(Clone, Debug, Reflect)]
pub struct AnimationMask {
    pub body: bool,
    pub head: bool,
    pub left_arm: bool,
    pub right_arm: bool,
    pub tail: bool,
}

impl AnimationMask {
    pub fn full() -> Self {
        Self {
            body: true,
            head: true,
            left_arm: true,
            right_arm: true,
            tail: true,
        }
    }
    
    pub fn upper_body() -> Self {
        Self {
            body: false,
            head: true,
            left_arm: true,
            right_arm: true,
            tail: false,
        }
    }
}

/// System to update animation states based on creature behavior
pub fn update_animation_states(
    time: Res<Time>,
    state_machine: Res<AnimationStateMachine>,
    mut query: Query<(
        &mut AnimationPlaybackState,
        &mut AnimationLayers,
        &mut CartoonSprite,
        &mut AnimatedSprite,
        &CreatureState,
        &Velocity,
        Option<&crate::components::ConversationState>,
    )>,
) {
    for (
        mut playback,
        mut layers,
        mut cartoon_sprite,
        mut animated_sprite,
        creature_state,
        velocity,
        conversation,
    ) in query.iter_mut() {
        // Update playback timer
        playback.time_played += time.delta_seconds();
        
        // Determine target animation
        let target_animation = determine_target_animation(creature_state, velocity, conversation);
        
        // Check if we can transition
        if playback.animation_type != target_animation {
            let can_transition = state_machine.can_transition(
                &playback,
                target_animation,
                creature_state,
                velocity.0.length(),
            );
            
            if can_transition {
                // Start transition
                if let Some(transition) = state_machine.get_transition(playback.animation_type, target_animation) {
                    start_animation_transition(
                        &mut playback,
                        &mut layers,
                        &mut cartoon_sprite,
                        &mut animated_sprite,
                        target_animation,
                        transition,
                    );
                }
            }
        }
        
        // Update current animation
        update_animation_playback(&mut playback, &mut animated_sprite, &time);
        
        // Update layer blending
        update_animation_layers(&mut layers, creature_state, velocity, &time);
    }
}

pub fn determine_target_animation(
    state: &CreatureState,
    velocity: &Velocity,
    conversation: Option<&crate::components::ConversationState>,
) -> AnimationType {
    match state {
        CreatureState::Dead => AnimationType::Death,
        CreatureState::Idle => {
            if conversation.is_some() {
                AnimationType::Talk
            } else {
                AnimationType::Idle
            }
        },
        CreatureState::Moving { .. } => {
            if velocity.0.length() > 5.0 {
                AnimationType::Run
            } else {
                AnimationType::Walk
            }
        },
        CreatureState::Eating => AnimationType::Eat,
        CreatureState::Drinking => AnimationType::Eat, // Reuse eat animation
        CreatureState::Resting => AnimationType::Sleep,
    }
}

fn start_animation_transition(
    playback: &mut AnimationPlaybackState,
    layers: &mut AnimationLayers,
    cartoon_sprite: &mut CartoonSprite,
    animated_sprite: &mut AnimatedSprite,
    target: AnimationType,
    transition: &AnimationTransition,
) {
    // Update playback state
    playback.animation_type = target;
    playback.current_frame = 0;
    playback.time_played = 0.0;
    playback.frames_played = 0;
    playback.is_complete = false;
    playback.transition_progress = Some(0.0);
    
    // Update cartoon sprite
    cartoon_sprite.base_animation = animation_type_to_state(target);
    
    // Update animated sprite frames
    let (start_frame, frame_count) = get_animation_frame_range(target);
    animated_sprite.frames = (start_frame..start_frame + frame_count).collect();
    animated_sprite.current_frame = 0;
    
    // Set up base layer
    layers.base_layer.animation = target;
    layers.base_layer.weight = 1.0;
}

fn update_animation_playback(
    playback: &mut AnimationPlaybackState,
    animated_sprite: &mut AnimatedSprite,
    time: &Time,
) {
    // Update frame counter
    if animated_sprite.timer.finished() {
        playback.frames_played += 1;
        
        // Check if animation is complete
        if !animated_sprite.looping && animated_sprite.current_frame >= animated_sprite.frames.len() - 1 {
            playback.is_complete = true;
        }
    }
    
    // Update transition progress
    if let Some(ref mut progress) = playback.transition_progress {
        *progress += time.delta_seconds() * 2.0; // 0.5 second transitions
        if *progress >= 1.0 {
            playback.transition_progress = None;
        }
    }
}

fn update_animation_layers(
    layers: &mut AnimationLayers,
    state: &CreatureState,
    velocity: &Velocity,
    time: &Time,
) {
    // Clear additive layers
    layers.additive_layers.clear();
    
    // Add procedural animations based on state
    if matches!(state, CreatureState::Idle) && velocity.0.length() < 0.1 {
        // Add subtle idle breathing
        layers.additive_layers.push(AnimationLayer {
            animation: AnimationType::Bounce,
            weight: 0.1,
            speed_multiplier: 0.5,
            time_offset: 0.0,
            mask: AnimationMask::full(),
        });
    }
}

fn animation_type_to_state(anim: AnimationType) -> AnimationState {
    match anim {
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

fn get_animation_frame_range(animation: AnimationType) -> (usize, usize) {
    match animation {
        AnimationType::Idle => (0, 4),
        AnimationType::Walk => (4, 8),
        AnimationType::Run => (12, 6),
        AnimationType::Eat => (18, 6),
        AnimationType::Sleep => (24, 4),
        AnimationType::Talk => (28, 8),
        AnimationType::Attack => (36, 6),
        AnimationType::Death => (42, 8),
        AnimationType::UseItem => (50, 4),
        AnimationType::Shiver => (54, 4),
        AnimationType::Bounce => (58, 4),
        AnimationType::Any => (0, 1), // Shouldn't be used directly
    }
}

/// Evaluate animation curve at time t (0.0 to 1.0)
pub fn evaluate_curve(curve: &AnimationCurve, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    match curve {
        AnimationCurve::Linear => t,
        AnimationCurve::EaseIn => t * t,
        AnimationCurve::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
        AnimationCurve::EaseInOut => {
            if t < 0.5 {
                2.0 * t * t
            } else {
                1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
            }
        }
        AnimationCurve::Custom(points) => {
            // Simple linear interpolation between control points
            if points.is_empty() {
                return t;
            }
            
            // Find surrounding points
            let mut prev = (0.0, 0.0);
            let mut next = (1.0, 1.0);
            
            for point in points {
                if point.0 <= t && point.0 > prev.0 {
                    prev = *point;
                }
                if point.0 >= t && point.0 < next.0 {
                    next = *point;
                }
            }
            
            // Linear interpolation
            if next.0 - prev.0 > 0.0 {
                let local_t = (t - prev.0) / (next.0 - prev.0);
                prev.1 + (next.1 - prev.1) * local_t
            } else {
                prev.1
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_animation_state_machine_creation() {
        let state_machine = AnimationStateMachine::default();
        
        // Check that key transitions exist
        assert!(state_machine.transitions.contains_key(&(AnimationType::Idle, AnimationType::Walk)));
        assert!(state_machine.transitions.contains_key(&(AnimationType::Walk, AnimationType::Run)));
        assert!(state_machine.transitions.contains_key(&(AnimationType::Any, AnimationType::Death)));
        
        // Check priorities
        assert_eq!(state_machine.interrupt_priorities[&AnimationType::Death], 11);
        assert_eq!(state_machine.interrupt_priorities[&AnimationType::Attack], 10);
    }
    
    #[test]
    fn test_animation_curve_evaluation() {
        assert_eq!(evaluate_curve(&AnimationCurve::Linear, 0.5), 0.5);
        assert_eq!(evaluate_curve(&AnimationCurve::EaseIn, 0.5), 0.25);
        assert_eq!(evaluate_curve(&AnimationCurve::EaseOut, 0.5), 0.75);
        
        // Test clamping
        assert_eq!(evaluate_curve(&AnimationCurve::Linear, -0.5), 0.0);
        assert_eq!(evaluate_curve(&AnimationCurve::Linear, 1.5), 1.0);
    }
    
    #[test]
    fn test_transition_conditions() {
        let state_machine = AnimationStateMachine::default();
        let mut current = AnimationPlaybackState::default();
        
        // Test immediate transition
        let can_transition = state_machine.can_transition(
            &current,
            AnimationType::Walk,
            &CreatureState::Moving { target: Vec2::ZERO },
            0.0,
        );
        assert!(can_transition);
        
        // Test velocity threshold
        let can_transition = state_machine.can_transition(
            &current,
            AnimationType::Run,
            &CreatureState::Moving { target: Vec2::ZERO },
            3.0, // Below threshold
        );
        assert!(!can_transition);
        
        let can_transition = state_machine.can_transition(
            &current,
            AnimationType::Run,
            &CreatureState::Moving { target: Vec2::ZERO },
            6.0, // Above threshold
        );
        assert!(can_transition);
    }
}