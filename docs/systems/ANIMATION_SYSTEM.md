# Animation System Architecture

## Overview

The animation system provides expressive, performant character animations for thousands of creatures simultaneously. It uses a combination of sprite-based animations, procedural effects, and expression systems to bring creatures to life.

## Animation Types

### Sprite Animation

Core animation using sprite sheets:

```rust
pub struct SpriteAnimation {
    pub id: AnimationId,
    pub sprite_sheet: Handle<SpriteSheet>,
    pub frames: Vec<SpriteFrame>,
    pub duration: f32,
    pub looping: LoopMode,
}

pub struct SpriteFrame {
    pub index: u32,              // Index in sprite sheet
    pub duration: f32,           // Frame duration
    pub offset: Vec2,            // Position offset
    pub events: Vec<FrameEvent>, // Sound, particles, etc.
}

pub enum LoopMode {
    Once,
    Loop,
    PingPong,
    ClampForever,
}

pub struct SpriteSheet {
    pub texture: Handle<Image>,
    pub layout: TextureAtlasLayout,
    pub animations: HashMap<String, AnimationRange>,
}

pub struct AnimationRange {
    pub start_frame: u32,
    pub end_frame: u32,
    pub default_fps: f32,
}
```

### Procedural Animation

Dynamic animations generated at runtime:

```rust
pub struct ProceduralAnimation {
    pub animation_type: ProceduralType,
    pub parameters: AnimationParams,
    pub modifiers: Vec<AnimationModifier>,
}

pub enum ProceduralType {
    Bounce {
        height: f32,
        frequency: f32,
    },
    Sway {
        amplitude: f32,
        speed: f32,
        offset: f32,
    },
    Shake {
        intensity: f32,
        decay: f32,
    },
    Squash {
        amount: f32,
        duration: f32,
    },
}

pub struct AnimationModifier {
    pub target: ModifierTarget,
    pub curve: AnimationCurve,
    pub blend_mode: BlendMode,
}

pub enum ModifierTarget {
    Position,
    Rotation,
    Scale,
    Color,
}
```

### Expression System

Facial expressions and emotional displays:

```rust
pub struct ExpressionSystem {
    pub base_expression: Expression,
    pub emotion_overlays: Vec<EmotionOverlay>,
    pub eye_controller: EyeController,
    pub mouth_controller: MouthController,
}

pub enum Expression {
    Neutral,
    Happy,
    Sad,
    Angry,
    Fearful,
    Surprised,
    Disgusted,
    Confused,
}

pub struct EmotionOverlay {
    pub emotion: Emotion,
    pub intensity: f32,
    pub fade_time: f32,
}

pub struct EyeController {
    pub blink_timer: f32,
    pub blink_duration: f32,
    pub look_target: Option<Vec3>,
    pub pupil_dilation: f32,
}

pub struct MouthController {
    pub mouth_state: MouthState,
    pub talk_animation: Option<TalkAnimation>,
}
```

## Animation State Machine

```rust
pub struct AnimationStateMachine {
    pub states: HashMap<StateId, AnimationState>,
    pub transitions: Vec<StateTransition>,
    pub current_state: StateId,
    pub transition_progress: Option<TransitionProgress>,
    pub parameters: AnimationParameters,
}

pub struct AnimationState {
    pub id: StateId,
    pub animation: AnimationClip,
    pub blend_tree: Option<BlendTree>,
    pub on_enter: Vec<StateEvent>,
    pub on_exit: Vec<StateEvent>,
}

pub struct StateTransition {
    pub from: StateId,
    pub to: StateId,
    pub conditions: Vec<TransitionCondition>,
    pub duration: f32,
    pub curve: TransitionCurve,
}

pub enum TransitionCondition {
    Parameter {
        name: String,
        comparison: Comparison,
        value: ParameterValue,
    },
    Trigger {
        name: String,
    },
    TimeElapsed {
        duration: f32,
    },
}

pub struct AnimationParameters {
    pub floats: HashMap<String, f32>,
    pub bools: HashMap<String, bool>,
    pub triggers: HashSet<String>,
}
```

### State Machine Example

```rust
// Define creature animation states
pub fn create_creature_state_machine() -> AnimationStateMachine {
    let mut sm = AnimationStateMachine::new();
    
    // Add states
    sm.add_state("idle", idle_animation());
    sm.add_state("walk", walk_animation());
    sm.add_state("run", run_animation());
    sm.add_state("eat", eat_animation());
    sm.add_state("sleep", sleep_animation());
    sm.add_state("attack", attack_animation());
    sm.add_state("die", death_animation());
    
    // Add transitions
    sm.add_transition("idle", "walk", vec![
        TransitionCondition::Parameter {
            name: "speed".into(),
            comparison: Comparison::Greater,
            value: ParameterValue::Float(0.1),
        }
    ], 0.2);
    
    sm.add_transition("walk", "run", vec![
        TransitionCondition::Parameter {
            name: "speed".into(),
            comparison: Comparison::Greater,
            value: ParameterValue::Float(0.7),
        }
    ], 0.1);
    
    sm.add_transition("any", "die", vec![
        TransitionCondition::Trigger {
            name: "death".into(),
        }
    ], 0.0);
    
    sm
}
```

## Blend Trees

For smooth animation blending:

```rust
pub struct BlendTree {
    pub root: BlendNode,
}

pub enum BlendNode {
    Clip(AnimationClip),
    
    Blend1D {
        parameter: String,
        children: Vec<(f32, Box<BlendNode>)>,
    },
    
    Blend2D {
        x_parameter: String,
        y_parameter: String,
        children: Vec<(Vec2, Box<BlendNode>)>,
    },
    
    Additive {
        base: Box<BlendNode>,
        additive: Box<BlendNode>,
        weight: f32,
    },
    
    Override {
        base: Box<BlendNode>,
        override_: Box<BlendNode>,
        mask: BoneMask,
    },
}

pub struct BoneMask {
    pub bones: HashSet<BoneId>,
    pub blend_in_time: f32,
}
```

## Animation Playback

```rust
pub struct AnimationPlayer {
    pub current_animations: Vec<ActiveAnimation>,
    pub blend_stack: BlendStack,
    pub time_scale: f32,
    pub root_motion: Option<RootMotion>,
}

pub struct ActiveAnimation {
    pub animation: Animation,
    pub time: f32,
    pub weight: f32,
    pub layer: u8,
    pub blend_mode: BlendMode,
}

pub enum BlendMode {
    Replace,
    Additive,
    Override(BoneMask),
}

impl AnimationPlayer {
    pub fn update(&mut self, dt: f32) {
        let scaled_dt = dt * self.time_scale;
        
        // Update all active animations
        for anim in &mut self.current_animations {
            anim.time += scaled_dt;
            
            // Handle looping
            match anim.animation.loop_mode {
                LoopMode::Once => {
                    if anim.time > anim.animation.duration {
                        anim.weight = 0.0; // Mark for removal
                    }
                }
                LoopMode::Loop => {
                    anim.time = anim.time % anim.animation.duration;
                }
                LoopMode::PingPong => {
                    let cycle = anim.time / anim.animation.duration;
                    if cycle as u32 % 2 == 1 {
                        anim.time = anim.animation.duration - (anim.time % anim.animation.duration);
                    } else {
                        anim.time = anim.time % anim.animation.duration;
                    }
                }
                LoopMode::ClampForever => {
                    anim.time = anim.time.min(anim.animation.duration);
                }
            }
        }
        
        // Remove finished animations
        self.current_animations.retain(|a| a.weight > 0.0);
        
        // Update blend stack
        self.blend_stack.update(dt);
    }
    
    pub fn sample(&self) -> AnimationPose {
        let mut pose = AnimationPose::default();
        
        // Sample each animation and blend
        for anim in &self.current_animations {
            let sample = anim.animation.sample(anim.time);
            pose.blend(sample, anim.weight, anim.blend_mode);
        }
        
        pose
    }
}
```

## LOD Integration

Animation quality scales with LOD:

```rust
pub struct AnimationLODSettings {
    pub lod_0: AnimationQuality {
        sprite_fps: 60,
        enable_blend_trees: true,
        enable_procedural: true,
        enable_expressions: true,
        enable_particles: true,
        sub_frame_interpolation: true,
    },
    
    pub lod_1: AnimationQuality {
        sprite_fps: 30,
        enable_blend_trees: true,
        enable_procedural: true,
        enable_expressions: true,
        enable_particles: true,
        sub_frame_interpolation: false,
    },
    
    pub lod_2: AnimationQuality {
        sprite_fps: 15,
        enable_blend_trees: false,
        enable_procedural: true,
        enable_expressions: false,
        enable_particles: false,
        sub_frame_interpolation: false,
    },
    
    pub lod_3: AnimationQuality {
        sprite_fps: 10,
        enable_blend_trees: false,
        enable_procedural: false,
        enable_expressions: false,
        enable_particles: false,
        sub_frame_interpolation: false,
    },
    
    pub lod_4_5: AnimationQuality {
        sprite_fps: 0, // Static sprite only
        enable_blend_trees: false,
        enable_procedural: false,
        enable_expressions: false,
        enable_particles: false,
        sub_frame_interpolation: false,
    },
}
```

## Particle Effects

Particles enhance animations:

```rust
pub struct AnimationParticles {
    pub emitters: Vec<ParticleEmitter>,
    pub triggers: Vec<ParticleTrigger>,
}

pub struct ParticleEmitter {
    pub id: EmitterId,
    pub attach_point: AttachPoint,
    pub particle_type: ParticleType,
    pub emission_rate: f32,
    pub lifetime: Range<f32>,
    pub velocity: Range<Vec3>,
    pub color_gradient: ColorGradient,
}

pub enum AttachPoint {
    Bone(BoneId),
    Sprite(SpriteAnchor),
    WorldPosition(Vec3),
}

pub enum ParticleTrigger {
    OnFrame {
        animation: AnimationId,
        frame: u32,
        emitter: EmitterId,
        burst_count: u32,
    },
    OnEvent {
        event: AnimationEvent,
        emitter: EmitterId,
        duration: f32,
    },
    WhileState {
        state: StateId,
        emitter: EmitterId,
    },
}
```

## Performance Optimization

### Animation Batching

```rust
pub struct AnimationBatcher {
    pub sprite_batches: HashMap<AnimationId, SpriteBatch>,
    pub instance_data: Vec<InstanceData>,
}

pub struct SpriteBatch {
    pub animation: AnimationId,
    pub instances: Vec<SpriteInstance>,
    pub texture_array: Handle<Image>,
}

pub struct SpriteInstance {
    pub entity: Entity,
    pub position: Vec3,
    pub rotation: f32,
    pub scale: Vec2,
    pub color: Color,
    pub frame: u32,
    pub flip_x: bool,
}

impl AnimationBatcher {
    pub fn batch_sprites(&mut self, visible_entities: &[Entity]) {
        self.sprite_batches.clear();
        
        for &entity in visible_entities {
            let anim_data = self.get_animation_data(entity);
            let batch = self.sprite_batches
                .entry(anim_data.animation_id)
                .or_insert_with(|| SpriteBatch::new(anim_data.animation_id));
                
            batch.instances.push(SpriteInstance {
                entity,
                position: anim_data.position,
                rotation: anim_data.rotation,
                scale: anim_data.scale,
                color: anim_data.color,
                frame: anim_data.current_frame,
                flip_x: anim_data.facing_left,
            });
        }
    }
}
```

### Animation Caching

```rust
pub struct AnimationCache {
    pub sampled_poses: LruCache<CacheKey, AnimationPose>,
    pub blend_results: LruCache<BlendKey, AnimationPose>,
    pub expression_cache: HashMap<(Expression, f32), ExpressionData>,
}

#[derive(Hash, Eq, PartialEq)]
pub struct CacheKey {
    animation_id: AnimationId,
    time: OrderedFloat<f32>,
    lod_level: u8,
}

impl AnimationCache {
    pub fn get_or_sample(&mut self, 
                        animation: &Animation, 
                        time: f32, 
                        lod: u8) -> AnimationPose {
        let key = CacheKey {
            animation_id: animation.id,
            time: OrderedFloat(quantize_time(time, lod)),
            lod_level: lod,
        };
        
        if let Some(pose) = self.sampled_poses.get(&key) {
            return pose.clone();
        }
        
        let pose = animation.sample(time);
        self.sampled_poses.put(key, pose.clone());
        pose
    }
}

fn quantize_time(time: f32, lod: u8) -> f32 {
    let quantization = match lod {
        0 => 1.0 / 60.0,  // Full precision
        1 => 1.0 / 30.0,  // Half precision
        2 => 1.0 / 15.0,  // Quarter precision
        _ => 1.0 / 10.0,  // Low precision
    };
    
    (time / quantization).round() * quantization
}
```

## Animation Events

```rust
pub enum AnimationEvent {
    // Sound events
    PlaySound {
        sound_id: SoundId,
        volume: f32,
        pitch_variance: f32,
    },
    
    // Visual effects
    SpawnParticles {
        emitter: EmitterId,
        count: u32,
    },
    ScreenShake {
        intensity: f32,
        duration: f32,
    },
    
    // Gameplay events
    DealDamage {
        amount: f32,
        radius: f32,
    },
    ApplyForce {
        direction: Vec3,
        magnitude: f32,
    },
    
    // State changes
    SetParameter {
        name: String,
        value: ParameterValue,
    },
    TriggerTransition {
        target_state: StateId,
    },
}

pub struct AnimationEventQueue {
    pub events: VecDeque<TimedEvent>,
}

pub struct TimedEvent {
    pub event: AnimationEvent,
    pub entity: Entity,
    pub timestamp: f32,
}
```

## Debug Visualization

```rust
pub struct AnimationDebugger {
    pub show_state_machine: bool,
    pub show_blend_weights: bool,
    pub show_bone_names: bool,
    pub show_animation_events: bool,
    pub highlight_transitions: bool,
}

impl AnimationDebugger {
    pub fn draw_debug_info(&self, entity: Entity, anim_data: &AnimationData) {
        if self.show_state_machine {
            self.draw_state_diagram(anim_data.state_machine);
        }
        
        if self.show_blend_weights {
            for (i, anim) in anim_data.active_animations.iter().enumerate() {
                draw_text(
                    &format!("{}: {:.2}", anim.animation.name, anim.weight),
                    Vec2::new(10.0, 30.0 + i as f32 * 20.0),
                    Color::WHITE,
                );
            }
        }
        
        if self.show_animation_events {
            for event in &anim_data.pending_events {
                self.draw_event_marker(event);
            }
        }
    }
}
```

## Integration Example

```rust
pub fn update_creature_animation(
    mut query: Query<(
        &mut AnimationPlayer,
        &mut AnimationStateMachine,
        &Velocity,
        &Health,
        &CreatureState,
        &LODLevel,
    )>,
    time: Res<Time>,
) {
    for (mut player, mut state_machine, velocity, health, creature_state, lod) in query.iter_mut() {
        // Update animation parameters
        state_machine.parameters.set_float("speed", velocity.0.length());
        state_machine.parameters.set_float("health", health.current / health.max);
        
        // Handle state triggers
        if health.current <= 0.0 {
            state_machine.parameters.set_trigger("death");
        }
        
        match creature_state {
            CreatureState::Eating => state_machine.parameters.set_trigger("eat"),
            CreatureState::Sleeping => state_machine.parameters.set_trigger("sleep"),
            CreatureState::Fighting => state_machine.parameters.set_trigger("attack"),
            _ => {}
        }
        
        // Update state machine
        state_machine.update(time.delta_seconds());
        
        // Apply LOD settings
        let quality = AnimationLODSettings::for_level(*lod);
        player.time_scale = quality.sprite_fps as f32 / 60.0;
        
        // Update animation playback
        player.update(time.delta_seconds());
    }
}
```

This animation system provides rich, expressive animations while maintaining performance across thousands of creatures through LOD integration, batching, and caching.