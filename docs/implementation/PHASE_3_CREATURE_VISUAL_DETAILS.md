# Phase 3: Creature Visual Systems - Detailed Specifications

This document provides the missing implementation details for Phase 3 of the cartoon isometric graphics system, focusing on creature animations, expressions, and genetic variations.

## Table of Contents
1. [Sprite Atlas Organization](#sprite-atlas-organization)
2. [Animation State Machine](#animation-state-machine)
3. [Genetic Pattern Rendering](#genetic-pattern-rendering)
4. [Expression Overlay Rendering](#expression-overlay-rendering)
5. [Tool/Accessory Attachment Points](#toolaccessory-attachment-points)
6. [Animation Blending System](#animation-blending-system)

## Sprite Atlas Organization

### Creature Sprite Atlas Layout

Each creature species uses a standardized 2048x1024 texture atlas with the following organization:

```
Atlas Layout (2048x1024):
┌─────────────────────────────────────────────────────────┐
│ Row 0: Base Variation - Normal Size                      │
│ Row 1: Large Variation (+30% size)                       │
│ Row 2: Small Variation (-30% size)                       │
│ Row 3: Spotted Pattern Overlay                           │
│ Row 4: Striped Pattern Overlay                           │
│ Row 5: Expressions Sheet                                 │
│ Row 6: Accessories/Tools Overlay                         │
│ Row 7: Special Effects (glow, shadows)                   │
└─────────────────────────────────────────────────────────┘

Each Row Layout (for animation rows 0-4):
┌────┬────┬────┬────┬────┬────┬────┬────┬────┬────┬────┬────┬────┬────┬────┬────┐
│Idle│Idle│Idle│Idle│Walk│Walk│Walk│Walk│Walk│Walk│Walk│Walk│Run │Run │Run │Run │ 48x48
│ 0  │ 1  │ 2  │ 3  │ 0  │ 1  │ 2  │ 3  │ 4  │ 5  │ 6  │ 7  │ 0  │ 1  │ 2  │ 3  │ each
├────┼────┼────┼────┼────┼────┼────┼────┼────┼────┼────┼────┼────┼────┼────┼────┤
│Run │Run │Eat │Eat │Eat │Eat │Eat │Eat │Sleep│Sleep│Sleep│Sleep│Talk│Talk│Talk│Talk│
│ 4  │ 5  │ 0  │ 1  │ 2  │ 3  │ 4  │ 5  │ 0  │ 1  │ 2  │ 3  │ 0  │ 1  │ 2  │ 3  │
├────┼────┼────┼────┼────┼────┼────┼────┼────┼────┼────┼────┼────┼────┼────┼────┤
│Talk│Talk│Talk│Talk│Atk │Atk │Atk │Atk │Atk │Atk │Death│Death│Death│Death│Death│Death│
│ 4  │ 5  │ 6  │ 7  │ 0  │ 1  │ 2  │ 3  │ 4  │ 5  │ 0  │ 1  │ 2  │ 3  │ 4  │ 5  │
└────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┘
```

### Expression Sheet Layout (Row 5)

```
Expression Row (specialized 96x96 sprites for detailed faces):
┌──────────┬──────────┬──────────┬──────────┬──────────┬──────────┬──────────┬──────────┐
│ Neutral  │  Happy   │   Sad    │  Angry   │ Scared   │ Curious  │  Tired   │  Hungry  │
│ Default  │ Smiling  │  Frown   │ Furrowed │Wide Eyes │Raised Brow│Half Closed│ Drooling │
├──────────┼──────────┼──────────┼──────────┼──────────┼──────────┼──────────┼──────────┤
│ Excited  │ Content  │Disgusted │Surprised │ Confused │ Sleeping │  Sick    │  Love    │
│Sparkle Eye│Soft Smile│Tongue Out│   :O     │    ?     │   ZZZ    │Green Tint│Heart Eyes│
└──────────┴──────────┴──────────┴──────────┴──────────┴──────────┴──────────┴──────────┘
```

### UV Coordinate Mapping

```rust
pub struct AtlasUVMapping {
    pub frame_width: f32,  // 48.0 / 2048.0 = 0.0234375
    pub frame_height: f32, // 48.0 / 1024.0 = 0.046875
    pub expression_width: f32,  // 96.0 / 2048.0 = 0.046875
    pub expression_height: f32, // 96.0 / 1024.0 = 0.09375
}

impl AtlasUVMapping {
    pub fn get_animation_uv(&self, animation: AnimationType, frame: usize, variation: usize) -> Rect {
        let (col, row_offset) = match animation {
            AnimationType::Idle => (frame % 4, 0),
            AnimationType::Walk => ((frame % 8) + 4, 0),
            AnimationType::Run => (frame % 6, 1),
            AnimationType::Eat => ((frame % 6) + 2, 1),
            AnimationType::Sleep => ((frame % 4) + 8, 1),
            AnimationType::Talk => (frame % 8, 2),
            AnimationType::Attack => ((frame % 6) + 4, 2),
            AnimationType::Death => ((frame % 6) + 10, 2),
            _ => (0, 0),
        };
        
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
    
    pub fn get_expression_uv(&self, expression: EmotionType) -> Rect {
        let (col, row) = match expression {
            EmotionType::Neutral => (0, 0),
            EmotionType::Happy => (1, 0),
            EmotionType::Sad => (2, 0),
            EmotionType::Angry => (3, 0),
            EmotionType::Scared => (4, 0),
            EmotionType::Curious => (5, 0),
            EmotionType::Tired => (6, 0),
            EmotionType::Hungry => (7, 0),
            EmotionType::Excited => (0, 1),
            EmotionType::Content => (1, 1),
            EmotionType::Disgusted => (2, 1),
            EmotionType::Surprised => (3, 1),
            EmotionType::Confused => (4, 1),
            EmotionType::Sleeping => (5, 1),
            EmotionType::Sick => (6, 1),
            EmotionType::Love => (7, 1),
        };
        
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
}
```

## Animation State Machine

### Transition Rules and Priorities

```rust
#[derive(Clone, Debug)]
pub struct AnimationTransition {
    pub from: AnimationType,
    pub to: AnimationType,
    pub duration: f32,
    pub blend_curve: AnimationCurve,
    pub priority: u8, // Higher priority can interrupt lower
    pub conditions: Vec<TransitionCondition>,
}

#[derive(Clone, Debug)]
pub enum TransitionCondition {
    Immediate,                    // Can transition immediately
    OnAnimationComplete,          // Wait for current animation to finish
    AfterFrames(usize),          // Wait for N frames
    WithMinDuration(f32),        // Current animation must play for at least X seconds
    VelocityThreshold(f32),     // Speed must exceed threshold
    StateRequired(CreatureState), // Specific AI state required
}

#[derive(Clone, Debug)]
pub enum AnimationCurve {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Custom(Vec<(f32, f32)>), // Control points for bezier curve
}

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
            priority: 10, // Highest priority
            conditions: vec![TransitionCondition::Immediate],
        });
        
        transitions.insert((AnimationType::Any, AnimationType::Death), AnimationTransition {
            from: AnimationType::Any,
            to: AnimationType::Death,
            duration: 0.0,
            blend_curve: AnimationCurve::Linear,
            priority: 11, // Absolute highest
            conditions: vec![TransitionCondition::Immediate],
        });
        
        // Sleep transitions (special handling)
        transitions.insert((AnimationType::Idle, AnimationType::Sleep), AnimationTransition {
            from: AnimationType::Idle,
            to: AnimationType::Sleep,
            duration: 1.0, // Slow transition to sleep
            blend_curve: AnimationCurve::EaseOut,
            priority: 3,
            conditions: vec![
                TransitionCondition::OnAnimationComplete,
                TransitionCondition::StateRequired(CreatureState::Sleeping),
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
        current: &AnimationState,
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
}
```

## Genetic Pattern Rendering

### Pattern System Implementation

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PatternType {
    None,
    Spots { density: f32, size: f32 },
    Stripes { width: f32, angle: f32 },
    Patches { scale: f32, irregularity: f32 },
    Gradient { start_color: Color, end_color: Color },
}

#[derive(Component)]
pub struct GeneticPattern {
    pub pattern_type: PatternType,
    pub primary_color: Color,
    pub secondary_color: Color,
    pub blend_mode: PatternBlendMode,
    pub intensity: f32, // 0.0-1.0
}

#[derive(Clone, Debug)]
pub enum PatternBlendMode {
    Multiply,
    Overlay,
    Replace,
    Add,
}

impl GeneticPattern {
    pub fn from_genetics(genetics: &Genetics) -> Self {
        let pattern_type = match genetics.pattern_gene {
            0..=20 => PatternType::None,
            21..=40 => PatternType::Spots {
                density: genetics.pattern_density * 0.5,
                size: genetics.pattern_scale * 10.0,
            },
            41..=60 => PatternType::Stripes {
                width: genetics.pattern_scale * 5.0,
                angle: genetics.pattern_angle,
            },
            61..=80 => PatternType::Patches {
                scale: genetics.pattern_scale * 20.0,
                irregularity: genetics.pattern_chaos,
            },
            _ => PatternType::Gradient {
                start_color: genetics.primary_color,
                end_color: genetics.secondary_color,
            },
        };
        
        Self {
            pattern_type,
            primary_color: genetics.primary_color,
            secondary_color: genetics.secondary_color,
            blend_mode: PatternBlendMode::Overlay,
            intensity: genetics.pattern_intensity,
        }
    }
}

/// Shader for rendering genetic patterns
pub const PATTERN_SHADER: &str = r#"
#import bevy_sprite::mesh2d_vertex_output

@group(1) @binding(0)
var pattern_texture: texture_2d<f32>;
@group(1) @binding(1)
var pattern_sampler: sampler;

struct PatternMaterial {
    pattern_type: u32,
    primary_color: vec4<f32>,
    secondary_color: vec4<f32>,
    pattern_params: vec4<f32>, // x: density/width, y: size/angle, z: intensity, w: time
}

@group(1) @binding(2)
var<uniform> material: PatternMaterial;

@fragment
fn fragment(in: MeshVertexOutput) -> @location(0) vec4<f32> {
    let base_color = textureSample(pattern_texture, pattern_sampler, in.uv);
    var pattern_color = base_color;
    
    switch material.pattern_type {
        // Spots pattern
        case 1u: {
            let density = material.pattern_params.x;
            let size = material.pattern_params.y;
            
            // Create spot pattern using noise
            let spot_coord = in.uv * density;
            let spot_noise = noise_2d(spot_coord);
            
            if spot_noise > (1.0 - size) {
                pattern_color = mix(base_color, material.secondary_color, material.pattern_params.z);
            }
        }
        
        // Stripes pattern
        case 2u: {
            let width = material.pattern_params.x;
            let angle = material.pattern_params.y;
            
            // Rotate UV coordinates
            let cos_a = cos(angle);
            let sin_a = sin(angle);
            let rotated_uv = vec2<f32>(
                in.uv.x * cos_a - in.uv.y * sin_a,
                in.uv.x * sin_a + in.uv.y * cos_a
            );
            
            // Create stripe pattern
            let stripe = step(0.5, fract(rotated_uv.x * width));
            pattern_color = mix(base_color, material.secondary_color, stripe * material.pattern_params.z);
        }
        
        // Patches pattern
        case 3u: {
            let scale = material.pattern_params.x;
            let irregularity = material.pattern_params.y;
            
            // Voronoi-based patches
            let cell_coord = in.uv * scale;
            let voronoi = voronoi_2d(cell_coord, irregularity);
            
            pattern_color = mix(
                material.primary_color,
                material.secondary_color,
                voronoi * material.pattern_params.z
            );
        }
        
        // Gradient pattern
        case 4u: {
            let gradient = in.uv.y; // Vertical gradient
            pattern_color = mix(
                material.primary_color,
                material.secondary_color,
                gradient * material.pattern_params.z
            );
        }
        
        default: {}
    }
    
    return pattern_color;
}

// Simple 2D noise function
fn noise_2d(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    
    let a = hash_2d(i);
    let b = hash_2d(i + vec2<f32>(1.0, 0.0));
    let c = hash_2d(i + vec2<f32>(0.0, 1.0));
    let d = hash_2d(i + vec2<f32>(1.0, 1.0));
    
    let u = f * f * (3.0 - 2.0 * f);
    
    return mix(a, b, u.x) + (c - a) * u.y * (1.0 - u.x) + (d - b) * u.x * u.y;
}

fn hash_2d(p: vec2<f32>) -> f32 {
    let h = dot(p, vec2<f32>(127.1, 311.7));
    return fract(sin(h) * 43758.5453123);
}
"#;
```

## Expression Overlay Rendering

### Detailed Expression Implementation

```rust
#[derive(Clone, Debug)]
pub struct ExpressionOverlay {
    // Eye components
    pub left_eye: EyeState,
    pub right_eye: EyeState,
    
    // Mouth components
    pub mouth_shape: MouthShape,
    pub mouth_openness: f32, // 0.0-1.0
    
    // Eyebrow components
    pub left_brow: BrowState,
    pub right_brow: BrowState,
    
    // Additional features
    pub cheek_blush: Option<BlushState>,
    pub sweat_drops: Option<SweatState>,
    pub emotion_effects: Vec<EmotionEffect>,
}

#[derive(Clone, Debug)]
pub struct EyeState {
    pub openness: f32,        // 0.0 (closed) - 1.0 (wide open)
    pub pupil_position: Vec2, // -1.0 to 1.0 for looking around
    pub pupil_size: f32,      // 0.5 (constricted) - 1.5 (dilated)
    pub shape: EyeShape,
}

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
pub struct MouthShape {
    pub curve_points: [Vec2; 4], // Bezier curve control points
    pub width: f32,              // 0.5-1.5 relative to base
    pub thickness: f32,          // Line thickness
}

#[derive(Clone, Debug)]
pub struct BrowState {
    pub height: f32,    // -0.5 (lowered) to 0.5 (raised)
    pub angle: f32,     // -30 to 30 degrees
    pub curve: f32,     // -0.5 (furrowed) to 0.5 (arched)
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
    TearDrops { side: EyeSide, flow_rate: f32 },
    Sparkles { density: f32, color: Color },
    QuestionMarks { count: u8 },
    ExclamationPoint { size: f32 },
    SleepBubbles { z_count: u8 },
}

impl ExpressionOverlay {
    pub fn from_emotion(emotion: EmotionType, intensity: f32) -> Self {
        match emotion {
            EmotionType::Happy => Self {
                left_eye: EyeState {
                    openness: 0.7,
                    pupil_position: Vec2::ZERO,
                    pupil_size: 1.0,
                    shape: EyeShape::Happy,
                },
                right_eye: EyeState {
                    openness: 0.7,
                    pupil_position: Vec2::ZERO,
                    pupil_size: 1.0,
                    shape: EyeShape::Happy,
                },
                mouth_shape: MouthShape {
                    curve_points: [
                        Vec2::new(-0.3, 0.0),
                        Vec2::new(-0.2, 0.2),
                        Vec2::new(0.2, 0.2),
                        Vec2::new(0.3, 0.0),
                    ],
                    width: 1.0,
                    thickness: 0.1,
                },
                mouth_openness: 0.3 * intensity,
                left_brow: BrowState {
                    height: 0.1,
                    angle: -5.0,
                    curve: 0.2,
                },
                right_brow: BrowState {
                    height: 0.1,
                    angle: 5.0,
                    curve: 0.2,
                },
                cheek_blush: Some(BlushState {
                    intensity: 0.3 * intensity,
                    color: Color::rgba(1.0, 0.7, 0.7, 0.5),
                    position_offset: Vec2::new(0.0, -0.1),
                }),
                sweat_drops: None,
                emotion_effects: if intensity > 0.8 {
                    vec![EmotionEffect::Sparkles {
                        density: 0.5,
                        color: Color::YELLOW,
                    }]
                } else {
                    vec![]
                },
            },
            
            EmotionType::Angry => Self {
                left_eye: EyeState {
                    openness: 0.6,
                    pupil_position: Vec2::ZERO,
                    pupil_size: 0.8,
                    shape: EyeShape::Angry,
                },
                right_eye: EyeState {
                    openness: 0.6,
                    pupil_position: Vec2::ZERO,
                    pupil_size: 0.8,
                    shape: EyeShape::Angry,
                },
                mouth_shape: MouthShape {
                    curve_points: [
                        Vec2::new(-0.3, 0.1),
                        Vec2::new(-0.2, -0.1),
                        Vec2::new(0.2, -0.1),
                        Vec2::new(0.3, 0.1),
                    ],
                    width: 0.8,
                    thickness: 0.15,
                },
                mouth_openness: 0.1,
                left_brow: BrowState {
                    height: -0.3,
                    angle: 20.0,
                    curve: -0.3,
                },
                right_brow: BrowState {
                    height: -0.3,
                    angle: -20.0,
                    curve: -0.3,
                },
                cheek_blush: None,
                sweat_drops: if intensity > 0.7 {
                    Some(SweatState {
                        drop_count: 2,
                        drop_size: 0.8,
                        animation_phase: 0.0,
                    })
                } else {
                    None
                },
                emotion_effects: if intensity > 0.9 {
                    vec![EmotionEffect::AngerVeins { intensity: 0.8 }]
                } else {
                    vec![]
                },
            },
            
            EmotionType::Sad => Self {
                left_eye: EyeState {
                    openness: 0.5,
                    pupil_position: Vec2::new(0.0, -0.2),
                    pupil_size: 1.1,
                    shape: EyeShape::Sad,
                },
                right_eye: EyeState {
                    openness: 0.5,
                    pupil_position: Vec2::new(0.0, -0.2),
                    pupil_size: 1.1,
                    shape: EyeShape::Sad,
                },
                mouth_shape: MouthShape {
                    curve_points: [
                        Vec2::new(-0.2, 0.0),
                        Vec2::new(-0.15, -0.2),
                        Vec2::new(0.15, -0.2),
                        Vec2::new(0.2, 0.0),
                    ],
                    width: 0.9,
                    thickness: 0.08,
                },
                mouth_openness: 0.0,
                left_brow: BrowState {
                    height: 0.0,
                    angle: -15.0,
                    curve: 0.1,
                },
                right_brow: BrowState {
                    height: 0.0,
                    angle: 15.0,
                    curve: 0.1,
                },
                cheek_blush: None,
                sweat_drops: None,
                emotion_effects: if intensity > 0.6 {
                    vec![
                        EmotionEffect::TearDrops {
                            side: EyeSide::Both,
                            flow_rate: intensity,
                        }
                    ]
                } else {
                    vec![]
                },
            },
            
            // ... Continue for other emotions
            _ => Self::default(),
        }
    }
    
    pub fn blend(from: &Self, to: &Self, t: f32) -> Self {
        Self {
            left_eye: blend_eye_state(&from.left_eye, &to.left_eye, t),
            right_eye: blend_eye_state(&from.right_eye, &to.right_eye, t),
            mouth_shape: blend_mouth_shape(&from.mouth_shape, &to.mouth_shape, t),
            mouth_openness: lerp(from.mouth_openness, to.mouth_openness, t),
            left_brow: blend_brow_state(&from.left_brow, &to.left_brow, t),
            right_brow: blend_brow_state(&from.right_brow, &to.right_brow, t),
            cheek_blush: blend_optional_blush(&from.cheek_blush, &to.cheek_blush, t),
            sweat_drops: blend_optional_sweat(&from.sweat_drops, &to.sweat_drops, t),
            emotion_effects: if t < 0.5 {
                from.emotion_effects.clone()
            } else {
                to.emotion_effects.clone()
            },
        }
    }
}

fn blend_eye_state(from: &EyeState, to: &EyeState, t: f32) -> EyeState {
    EyeState {
        openness: lerp(from.openness, to.openness, t),
        pupil_position: from.pupil_position.lerp(to.pupil_position, t),
        pupil_size: lerp(from.pupil_size, to.pupil_size, t),
        shape: if t < 0.5 { from.shape.clone() } else { to.shape.clone() },
    }
}

fn blend_mouth_shape(from: &MouthShape, to: &MouthShape, t: f32) -> MouthShape {
    MouthShape {
        curve_points: [
            from.curve_points[0].lerp(to.curve_points[0], t),
            from.curve_points[1].lerp(to.curve_points[1], t),
            from.curve_points[2].lerp(to.curve_points[2], t),
            from.curve_points[3].lerp(to.curve_points[3], t),
        ],
        width: lerp(from.width, to.width, t),
        thickness: lerp(from.thickness, to.thickness, t),
    }
}
```

## Tool/Accessory Attachment Points

### Bone and Anchor System

```rust
#[derive(Clone, Debug)]
pub struct CreatureAttachmentPoints {
    pub head: AttachmentPoint,
    pub left_hand: AttachmentPoint,
    pub right_hand: AttachmentPoint,
    pub back: AttachmentPoint,
    pub waist: AttachmentPoint,
    pub tail_tip: Option<AttachmentPoint>,
}

#[derive(Clone, Debug)]
pub struct AttachmentPoint {
    pub name: String,
    pub base_position: Vec2,      // Relative to sprite center
    pub rotation_pivot: Vec2,     // Pivot point for rotation
    pub depth_offset: f32,        // Z-order adjustment
    pub scale_factor: f32,        // Size adjustment for attached items
    pub animation_offsets: HashMap<AnimationType, Vec<AnimationOffset>>,
}

#[derive(Clone, Debug)]
pub struct AnimationOffset {
    pub frame: usize,
    pub position_offset: Vec2,
    pub rotation: f32,
    pub scale: f32,
}

impl Default for CreatureAttachmentPoints {
    fn default() -> Self {
        Self {
            head: AttachmentPoint {
                name: "head".to_string(),
                base_position: Vec2::new(0.0, 20.0), // 20 pixels above center
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
                depth_offset: -0.1, // Behind creature
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
                animation_offsets: HashMap::new(), // Minimal animation
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

fn create_left_hand_animation_offsets() -> HashMap<AnimationType, Vec<AnimationOffset>> {
    let mut offsets = HashMap::new();
    
    // Walking animation - hand swings
    offsets.insert(AnimationType::Walk, vec![
        AnimationOffset {
            frame: 0,
            position_offset: Vec2::new(0.0, 0.0),
            rotation: 0.0,
            scale: 1.0,
        },
        AnimationOffset {
            frame: 2,
            position_offset: Vec2::new(2.0, -1.0),
            rotation: 15.0,
            scale: 1.0,
        },
        AnimationOffset {
            frame: 4,
            position_offset: Vec2::new(0.0, 0.0),
            rotation: 0.0,
            scale: 1.0,
        },
        AnimationOffset {
            frame: 6,
            position_offset: Vec2::new(-2.0, -1.0),
            rotation: -15.0,
            scale: 1.0,
        },
    ]);
    
    // Eating animation - hand to mouth
    offsets.insert(AnimationType::Eat, vec![
        AnimationOffset {
            frame: 0,
            position_offset: Vec2::new(0.0, 0.0),
            rotation: 0.0,
            scale: 1.0,
        },
        AnimationOffset {
            frame: 1,
            position_offset: Vec2::new(5.0, 8.0),
            rotation: 45.0,
            scale: 1.0,
        },
        AnimationOffset {
            frame: 3,
            position_offset: Vec2::new(8.0, 15.0),
            rotation: 70.0,
            scale: 1.0,
        },
        AnimationOffset {
            frame: 5,
            position_offset: Vec2::new(5.0, 8.0),
            rotation: 45.0,
            scale: 1.0,
        },
    ]);
    
    // Tool use animation
    offsets.insert(AnimationType::UseItem, vec![
        AnimationOffset {
            frame: 0,
            position_offset: Vec2::new(0.0, 0.0),
            rotation: 0.0,
            scale: 1.0,
        },
        AnimationOffset {
            frame: 1,
            position_offset: Vec2::new(3.0, 5.0),
            rotation: -30.0,
            scale: 1.1,
        },
        AnimationOffset {
            frame: 2,
            position_offset: Vec2::new(5.0, 10.0),
            rotation: -60.0,
            scale: 1.2,
        },
        AnimationOffset {
            frame: 3,
            position_offset: Vec2::new(4.0, 12.0),
            rotation: -75.0,
            scale: 1.15,
        },
        AnimationOffset {
            frame: 4,
            position_offset: Vec2::new(2.0, 8.0),
            rotation: -45.0,
            scale: 1.05,
        },
    ]);
    
    offsets
}

#[derive(Component)]
pub struct AttachedItem {
    pub attachment_point: String,
    pub item_type: ItemType,
    pub custom_offset: Vec2,
    pub custom_rotation: f32,
    pub inherit_animation: bool,
}

pub fn update_attachment_transforms(
    mut attachments: Query<(&AttachedItem, &mut Transform, &Parent)>,
    creatures: Query<(&CreatureAttachmentPoints, &CartoonSprite, &Transform), Without<AttachedItem>>,
) {
    for (attached, mut attachment_transform, parent) in attachments.iter_mut() {
        if let Ok((points, sprite, creature_transform)) = creatures.get(parent.get()) {
            // Find the attachment point
            let point = match attached.attachment_point.as_str() {
                "head" => &points.head,
                "left_hand" => &points.left_hand,
                "right_hand" => &points.right_hand,
                "back" => &points.back,
                "waist" => &points.waist,
                "tail_tip" => points.tail_tip.as_ref().unwrap_or(&points.waist),
                _ => continue,
            };
            
            // Calculate base transform
            let mut position = point.base_position + attached.custom_offset;
            let mut rotation = attached.custom_rotation;
            let mut scale = point.scale_factor;
            
            // Apply animation offsets if enabled
            if attached.inherit_animation {
                if let Some(anim_offsets) = point.animation_offsets.get(&sprite.animation_state.current) {
                    // Find the appropriate offset for current frame
                    let current_frame = sprite.current_frame;
                    
                    // Linear interpolation between keyframes
                    if let Some(offset) = interpolate_animation_offset(anim_offsets, current_frame, sprite.frame_progress) {
                        position += offset.position_offset;
                        rotation += offset.rotation;
                        scale *= offset.scale;
                    }
                }
            }
            
            // Apply to transform
            attachment_transform.translation = creature_transform.translation + position.extend(point.depth_offset);
            attachment_transform.rotation = Quat::from_rotation_z(rotation.to_radians());
            attachment_transform.scale = Vec3::splat(scale);
        }
    }
}

fn interpolate_animation_offset(
    offsets: &[AnimationOffset],
    current_frame: usize,
    frame_progress: f32,
) -> Option<AnimationOffset> {
    // Find surrounding keyframes
    let mut prev_offset = None;
    let mut next_offset = None;
    
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
            // Interpolate between keyframes
            let frame_diff = next.frame - prev.frame;
            let progress = (current_frame - prev.frame) as f32 + frame_progress;
            let t = progress / frame_diff as f32;
            
            Some(AnimationOffset {
                frame: current_frame,
                position_offset: prev.position_offset.lerp(next.position_offset, t),
                rotation: lerp(prev.rotation, next.rotation, t),
                scale: lerp(prev.scale, next.scale, t),
            })
        }
        (Some(offset), _) | (_, Some(offset)) => Some(offset.clone()),
        _ => None,
    }
}
```

## Animation Blending System

### Multi-Layer Animation Blending

```rust
#[derive(Component)]
pub struct AnimationLayers {
    pub base_layer: AnimationLayer,
    pub overlay_layer: Option<AnimationLayer>,
    pub additive_layers: Vec<AnimationLayer>,
    pub blend_tree: AnimationBlendTree,
}

#[derive(Clone, Debug)]
pub struct AnimationLayer {
    pub animation: AnimationType,
    pub weight: f32,           // 0.0-1.0
    pub speed_multiplier: f32,
    pub time_offset: f32,
    pub mask: AnimationMask,
}

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
pub struct AnimationBlendTree {
    pub nodes: Vec<BlendNode>,
    pub connections: Vec<BlendConnection>,
}

#[derive(Clone, Debug)]
pub enum BlendNode {
    Animation(AnimationType),
    Blend2D { x_param: String, y_param: String },
    AdditiveBlend { base: Box<BlendNode>, additive: Box<BlendNode> },
    LayeredBlend { layers: Vec<(Box<BlendNode>, f32)> },
    StateMachine { states: HashMap<String, Box<BlendNode>> },
}

#[derive(Clone, Debug)]
pub struct BlendConnection {
    pub from: usize,
    pub to: usize,
    pub weight: f32,
}

pub fn blend_animations(
    base_frame: &AnimationFrame,
    overlay_frame: Option<&AnimationFrame>,
    blend_params: &BlendParameters,
) -> AnimationFrame {
    let mut result = base_frame.clone();
    
    if let Some(overlay) = overlay_frame {
        // Blend positions
        for (bone_name, base_transform) in &result.bone_transforms {
            if let Some(overlay_transform) = overlay.bone_transforms.get(bone_name) {
                let mask_weight = get_mask_weight(bone_name, &blend_params.mask);
                let effective_weight = blend_params.weight * mask_weight;
                
                let blended_transform = BoneTransform {
                    position: base_transform.position.lerp(
                        overlay_transform.position,
                        effective_weight
                    ),
                    rotation: base_transform.rotation.slerp(
                        overlay_transform.rotation,
                        effective_weight
                    ),
                    scale: base_transform.scale.lerp(
                        overlay_transform.scale,
                        effective_weight
                    ),
                };
                
                result.bone_transforms.insert(bone_name.clone(), blended_transform);
            }
        }
    }
    
    result
}

/// Blend between walk and run based on velocity
pub fn velocity_based_blend(
    walk_animation: &Animation,
    run_animation: &Animation,
    velocity: f32,
    walk_speed: f32,
    run_speed: f32,
) -> AnimationFrame {
    let blend_factor = ((velocity - walk_speed) / (run_speed - walk_speed)).clamp(0.0, 1.0);
    
    let walk_frame = walk_animation.sample(velocity / walk_speed);
    let run_frame = run_animation.sample(velocity / run_speed);
    
    blend_animation_frames(&walk_frame, &run_frame, blend_factor)
}

/// Smooth transition system
pub struct AnimationTransitionState {
    pub from_animation: AnimationType,
    pub to_animation: AnimationType,
    pub transition_time: f32,
    pub transition_duration: f32,
    pub easing_curve: EasingFunction,
}

impl AnimationTransitionState {
    pub fn update(&mut self, delta_time: f32) -> f32 {
        self.transition_time += delta_time;
        let progress = (self.transition_time / self.transition_duration).clamp(0.0, 1.0);
        
        match self.easing_curve {
            EasingFunction::Linear => progress,
            EasingFunction::EaseIn => progress * progress,
            EasingFunction::EaseOut => 1.0 - (1.0 - progress) * (1.0 - progress),
            EasingFunction::EaseInOut => {
                if progress < 0.5 {
                    2.0 * progress * progress
                } else {
                    1.0 - (-2.0 * progress + 2.0).powi(2) / 2.0
                }
            }
            EasingFunction::Custom(curve) => {
                evaluate_bezier_curve(&curve, progress)
            }
        }
    }
}

/// System to handle complex animation scenarios
pub fn update_animation_blending(
    mut creatures: Query<(
        &mut AnimationLayers,
        &mut CartoonSprite,
        &Velocity,
        &CreatureState,
        &ActionState,
    )>,
    time: Res<Time>,
) {
    for (mut layers, mut sprite, velocity, creature_state, action_state) in creatures.iter_mut() {
        // Update base locomotion layer
        layers.base_layer = match velocity.length() {
            v if v < 0.1 => AnimationLayer {
                animation: AnimationType::Idle,
                weight: 1.0,
                speed_multiplier: 1.0,
                time_offset: 0.0,
                mask: AnimationMask::full(),
            },
            v if v < 5.0 => AnimationLayer {
                animation: AnimationType::Walk,
                weight: 1.0,
                speed_multiplier: v / 3.0, // Adjust walk speed based on velocity
                time_offset: 0.0,
                mask: AnimationMask::full(),
            },
            _ => AnimationLayer {
                animation: AnimationType::Run,
                weight: 1.0,
                speed_multiplier: velocity.length() / 8.0,
                time_offset: 0.0,
                mask: AnimationMask::full(),
            },
        };
        
        // Add overlay for actions
        layers.overlay_layer = match action_state {
            ActionState::Eating => Some(AnimationLayer {
                animation: AnimationType::Eat,
                weight: 0.8,
                speed_multiplier: 1.0,
                time_offset: 0.0,
                mask: AnimationMask::upper_body(), // Only affect upper body
            }),
            ActionState::Talking => Some(AnimationLayer {
                animation: AnimationType::Talk,
                weight: 0.6,
                speed_multiplier: 1.2,
                time_offset: 0.0,
                mask: AnimationMask {
                    body: false,
                    head: true,
                    left_arm: true,
                    right_arm: true,
                    tail: false,
                },
            }),
            _ => None,
        };
        
        // Apply procedural animations
        if creature_state.is_cold {
            layers.additive_layers.push(AnimationLayer {
                animation: AnimationType::Shiver,
                weight: 0.3,
                speed_multiplier: 2.0,
                time_offset: time.elapsed_seconds() * 10.0, // Offset for variation
                mask: AnimationMask::full(),
            });
        }
        
        if creature_state.is_excited {
            layers.additive_layers.push(AnimationLayer {
                animation: AnimationType::Bounce,
                weight: 0.4,
                speed_multiplier: 1.5,
                time_offset: 0.0,
                mask: AnimationMask {
                    body: true,
                    head: false,
                    left_arm: false,
                    right_arm: false,
                    tail: true,
                },
            });
        }
    }
}
```

## Integration Example

Here's how all these systems work together:

```rust
pub fn setup_creature_visuals(
    mut commands: Commands,
    genetics: &Genetics,
    species: Species,
) -> Entity {
    let attachment_points = CreatureAttachmentPoints::default();
    let pattern = GeneticPattern::from_genetics(genetics);
    let expression_controller = ExpressionController::new();
    
    commands.spawn((
        CartoonCreature {
            base_animation: AnimationState::new(AnimationType::Idle),
            expression_overlay: None,
            body_modifiers: BodyModifiers {
                size_scale: genetics.size_modifier,
                color_tint: genetics.base_color,
                limb_proportions: genetics.limb_ratios,
            },
            accessory_slots: vec![],
        },
        CartoonSprite {
            atlas_path: format!("creatures/{}/atlas.png", species.to_string()),
            current_frame: 0,
            frame_progress: 0.0,
            animation_speed: 1.0,
            variation_row: select_variation_for_genetics(genetics),
            expression_overlay: ExpressionOverlay::default(),
        },
        expression_controller,
        pattern,
        attachment_points,
        AnimationLayers {
            base_layer: AnimationLayer {
                animation: AnimationType::Idle,
                weight: 1.0,
                speed_multiplier: 1.0,
                time_offset: 0.0,
                mask: AnimationMask::full(),
            },
            overlay_layer: None,
            additive_layers: vec![],
            blend_tree: AnimationBlendTree::default(),
        },
    ))
}
```

This comprehensive documentation provides all the missing details for Phase 3 implementation, including exact sprite layouts, animation transition rules, pattern rendering systems, expression details, attachment points, and the animation blending system.