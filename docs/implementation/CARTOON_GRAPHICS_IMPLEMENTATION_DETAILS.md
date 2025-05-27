# Cartoon Graphics Implementation Details

This document provides the remaining implementation details for the cartoon graphics system, covering expression blending, save/load integration, audio synchronization, and other gaps identified in the design review.

## Creature Expression Blending System

### Expression State Machine

```rust
/// Expression blending with priority and timing
#[derive(Component)]
pub struct ExpressionController {
    pub current_expression: EmotionType,
    pub target_expression: EmotionType,
    pub blend_progress: f32,
    pub blend_duration: f32,
    pub expression_priority: HashMap<EmotionType, f32>,
    pub expression_timers: HashMap<EmotionType, Timer>,
}

impl ExpressionController {
    pub fn new() -> Self {
        let mut priority = HashMap::new();
        // Higher priority emotions override lower ones
        priority.insert(EmotionType::Angry, 0.9);
        priority.insert(EmotionType::Frightened, 0.85);
        priority.insert(EmotionType::Sad, 0.7);
        priority.insert(EmotionType::Hungry, 0.6);
        priority.insert(EmotionType::Tired, 0.5);
        priority.insert(EmotionType::Happy, 0.4);
        priority.insert(EmotionType::Curious, 0.3);
        priority.insert(EmotionType::Content, 0.2);
        priority.insert(EmotionType::Neutral, 0.1);
        
        Self {
            current_expression: EmotionType::Neutral,
            target_expression: EmotionType::Neutral,
            blend_progress: 1.0,
            blend_duration: 0.3, // Default 300ms transition
            expression_priority: priority,
            expression_timers: HashMap::new(),
        }
    }
    
    /// Request expression change with priority check
    pub fn request_expression(
        &mut self,
        new_expression: EmotionType,
        duration: Option<f32>,
        force: bool,
    ) {
        let new_priority = self.expression_priority[&new_expression];
        let current_priority = self.expression_priority[&self.current_expression];
        
        // Only change if higher priority or forced
        if force || new_priority > current_priority || self.blend_progress >= 1.0 {
            self.target_expression = new_expression;
            self.blend_progress = 0.0;
            self.blend_duration = duration.unwrap_or(0.3);
            
            // Set expression duration timer
            if let Some(duration) = duration {
                self.expression_timers.insert(
                    new_expression,
                    Timer::from_seconds(duration, TimerMode::Once),
                );
            }
        }
    }
    
    /// Update expression blending
    pub fn update(&mut self, delta: f32) {
        // Update blend progress
        if self.blend_progress < 1.0 {
            self.blend_progress = (self.blend_progress + delta / self.blend_duration).min(1.0);
            
            if self.blend_progress >= 1.0 {
                self.current_expression = self.target_expression;
            }
        }
        
        // Update expression timers
        let mut expired_expressions = Vec::new();
        for (emotion, timer) in self.expression_timers.iter_mut() {
            timer.tick(Duration::from_secs_f32(delta));
            if timer.finished() {
                expired_expressions.push(*emotion);
            }
        }
        
        // Revert expired expressions to neutral
        for emotion in expired_expressions {
            self.expression_timers.remove(&emotion);
            if self.current_expression == emotion {
                self.request_expression(EmotionType::Neutral, None, false);
            }
        }
    }
    
    /// Get blended expression values for rendering
    pub fn get_blend_values(&self) -> (EmotionType, EmotionType, f32) {
        (self.current_expression, self.target_expression, self.blend_progress)
    }
}

/// Expression rendering with smooth transitions
pub fn render_expression_overlay(
    expression: &ExpressionController,
    sprite: &mut CartoonSprite,
) {
    let (from, to, blend) = expression.get_blend_values();
    
    // Blend facial features
    let eye_state = blend_eye_states(
        get_eye_state_for_emotion(from),
        get_eye_state_for_emotion(to),
        blend,
    );
    
    let mouth_state = blend_mouth_states(
        get_mouth_state_for_emotion(from),
        get_mouth_state_for_emotion(to),
        blend,
    );
    
    // Apply to sprite overlay
    sprite.expression_overlay = ExpressionOverlay {
        eye_offset: eye_state.offset,
        eye_scale: eye_state.scale,
        mouth_curve: mouth_state.curve_amount,
        mouth_open: mouth_state.open_amount,
        brow_angle: lerp(
            get_brow_angle(from),
            get_brow_angle(to),
            blend,
        ),
    };
}
```

### Expression Conflict Resolution

```rust
/// Handle multiple simultaneous emotion triggers
pub struct EmotionPriorityQueue {
    emotions: BinaryHeap<PrioritizedEmotion>,
    cooldowns: HashMap<EmotionType, Timer>,
}

#[derive(Eq, PartialEq)]
struct PrioritizedEmotion {
    emotion: EmotionType,
    priority: u32,
    timestamp: Instant,
}

impl Ord for PrioritizedEmotion {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.cmp(&other.priority)
            .then_with(|| self.timestamp.cmp(&other.timestamp))
    }
}

impl EmotionPriorityQueue {
    pub fn push_emotion(&mut self, emotion: EmotionType, context: &EmotionContext) {
        // Check cooldown
        if let Some(cooldown) = self.cooldowns.get(&emotion) {
            if !cooldown.finished() {
                return; // Still in cooldown
            }
        }
        
        // Calculate dynamic priority based on context
        let base_priority = get_base_priority(emotion);
        let context_modifier = calculate_context_modifier(emotion, context);
        let final_priority = base_priority + context_modifier;
        
        self.emotions.push(PrioritizedEmotion {
            emotion,
            priority: final_priority,
            timestamp: Instant::now(),
        });
        
        // Set cooldown to prevent emotion spam
        self.cooldowns.insert(
            emotion,
            Timer::from_seconds(get_emotion_cooldown(emotion), TimerMode::Once),
        );
    }
    
    pub fn get_current_emotion(&mut self) -> Option<EmotionType> {
        // Clean up expired emotions
        while let Some(emotion) = self.emotions.peek() {
            let age = emotion.timestamp.elapsed();
            if age > Duration::from_secs(5) {
                self.emotions.pop();
            } else {
                break;
            }
        }
        
        self.emotions.peek().map(|e| e.emotion)
    }
}
```

## Save/Load Visual State Integration

```rust
/// Visual state data to persist in saves
#[derive(Serialize, Deserialize)]
pub struct PersistentVisualState {
    // Animation state
    pub current_animation: AnimationType,
    pub animation_frame: f32,
    pub animation_speed: f32,
    
    // Expression state
    pub current_expression: EmotionType,
    pub expression_intensity: f32,
    
    // Particle emitters
    pub active_particles: Vec<ParticleEmitterState>,
    
    // Camera state (optional, for resuming view)
    pub camera_state: Option<CameraState>,
    
    // Time of day for consistent lighting
    pub time_of_day: f32,
    
    // Weather state
    pub weather: WeatherType,
    pub weather_intensity: f32,
}

#[derive(Serialize, Deserialize)]
pub struct ParticleEmitterState {
    pub effect_type: ParticleEffectType,
    pub position: Vec3,
    pub lifetime_remaining: f32,
    pub particles_spawned: u32,
}

#[derive(Serialize, Deserialize)]
pub struct CameraState {
    pub position: Vec3,
    pub zoom: f32,
    pub rotation: f32,
}

/// System to collect visual state for saving
pub fn collect_visual_state(
    creatures: Query<(&CartoonSprite, &ExpressionController, &Transform)>,
    particles: Query<(&ParticleEmitter, &Transform)>,
    camera: Query<&IsometricCamera>,
    time: Res<TimeOfDay>,
    weather: Res<WeatherSystem>,
) -> HashMap<Entity, PersistentVisualState> {
    let mut visual_states = HashMap::new();
    
    // Collect creature visual states
    for (entity, (sprite, expression, transform)) in creatures.iter() {
        let particle_states = particles
            .iter()
            .filter(|(emitter, _)| emitter.parent_entity == Some(entity))
            .map(|(emitter, transform)| ParticleEmitterState {
                effect_type: emitter.effect_type.clone(),
                position: transform.translation,
                lifetime_remaining: emitter.lifetime_remaining,
                particles_spawned: emitter.particles_spawned,
            })
            .collect();
        
        visual_states.insert(entity, PersistentVisualState {
            current_animation: sprite.animation_state.current,
            animation_frame: sprite.current_frame as f32 + sprite.frame_progress,
            animation_speed: sprite.animation_speed,
            current_expression: expression.current_expression,
            expression_intensity: expression.blend_progress,
            active_particles: particle_states,
            camera_state: None, // Set separately
            time_of_day: time.current,
            weather: weather.current_weather,
            weather_intensity: weather.intensity,
        });
    }
    
    // Add camera state to first entry (or separate field)
    if let Ok(camera) = camera.get_single() {
        if let Some(state) = visual_states.values_mut().next() {
            state.camera_state = Some(CameraState {
                position: camera.position,
                zoom: camera.zoom,
                rotation: 0.0, // If camera rotation is supported
            });
        }
    }
    
    visual_states
}

/// System to restore visual state after loading
pub fn restore_visual_state(
    mut creatures: Query<(&mut CartoonSprite, &mut ExpressionController)>,
    mut commands: Commands,
    visual_states: HashMap<Entity, PersistentVisualState>,
    mut camera: Query<&mut IsometricCamera>,
    mut time: ResMut<TimeOfDay>,
    mut weather: ResMut<WeatherSystem>,
) {
    for (entity, state) in visual_states {
        // Restore creature animations and expressions
        if let Ok((mut sprite, mut expression)) = creatures.get_mut(entity) {
            sprite.animation_state.current = state.current_animation;
            sprite.current_frame = state.animation_frame as usize;
            sprite.frame_progress = state.animation_frame.fract();
            sprite.animation_speed = state.animation_speed;
            
            expression.current_expression = state.current_expression;
            expression.blend_progress = state.expression_intensity;
            
            // Restore particle emitters
            for particle_state in state.active_particles {
                commands.spawn(ParticleEmitter {
                    effect_type: particle_state.effect_type,
                    lifetime_remaining: particle_state.lifetime_remaining,
                    particles_spawned: particle_state.particles_spawned,
                    parent_entity: Some(entity),
                    ..default()
                })
                .insert(Transform::from_translation(particle_state.position));
            }
        }
        
        // Restore camera (once)
        if let Some(cam_state) = state.camera_state {
            if let Ok(mut camera) = camera.get_single_mut() {
                camera.position = cam_state.position;
                camera.zoom = cam_state.zoom;
            }
        }
        
        // Restore time and weather (once)
        time.current = state.time_of_day;
        weather.current_weather = state.weather;
        weather.intensity = state.weather_intensity;
    }
}
```

## Audio Synchronization System

```rust
/// Audio cue definitions for animations
#[derive(Debug, Clone)]
pub struct AnimationAudioCue {
    pub frame: usize,
    pub sound: SoundEffect,
    pub volume: f32,
    pub pitch_variation: f32,
}

/// Sound effect types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SoundEffect {
    Footstep { surface: SurfaceType },
    Eat { food_type: FoodType },
    Drink,
    Sleep { phase: SleepPhase },
    Vocalization { emotion: EmotionType },
    Action { action: ActionType },
}

/// Animation audio configuration
pub struct AnimationAudioConfig {
    pub cues: HashMap<AnimationType, Vec<AnimationAudioCue>>,
}

impl Default for AnimationAudioConfig {
    fn default() -> Self {
        let mut cues = HashMap::new();
        
        // Walking animation - footsteps on frames 2 and 6
        cues.insert(AnimationType::Walk, vec![
            AnimationAudioCue {
                frame: 2,
                sound: SoundEffect::Footstep { surface: SurfaceType::Grass },
                volume: 0.3,
                pitch_variation: 0.1,
            },
            AnimationAudioCue {
                frame: 6,
                sound: SoundEffect::Footstep { surface: SurfaceType::Grass },
                volume: 0.3,
                pitch_variation: 0.1,
            },
        ]);
        
        // Running animation - faster footsteps
        cues.insert(AnimationType::Run, vec![
            AnimationAudioCue {
                frame: 1,
                sound: SoundEffect::Footstep { surface: SurfaceType::Grass },
                volume: 0.5,
                pitch_variation: 0.15,
            },
            AnimationAudioCue {
                frame: 3,
                sound: SoundEffect::Footstep { surface: SurfaceType::Grass },
                volume: 0.5,
                pitch_variation: 0.15,
            },
            AnimationAudioCue {
                frame: 5,
                sound: SoundEffect::Footstep { surface: SurfaceType::Grass },
                volume: 0.5,
                pitch_variation: 0.15,
            },
        ]);
        
        // Eating animation - chewing sounds
        cues.insert(AnimationType::Eat, vec![
            AnimationAudioCue {
                frame: 2,
                sound: SoundEffect::Eat { food_type: FoodType::Generic },
                volume: 0.4,
                pitch_variation: 0.2,
            },
            AnimationAudioCue {
                frame: 5,
                sound: SoundEffect::Eat { food_type: FoodType::Generic },
                volume: 0.3,
                pitch_variation: 0.2,
            },
        ]);
        
        // Sleeping animation - snoring
        cues.insert(AnimationType::Sleep, vec![
            AnimationAudioCue {
                frame: 0,
                sound: SoundEffect::Sleep { phase: SleepPhase::Inhale },
                volume: 0.2,
                pitch_variation: 0.05,
            },
            AnimationAudioCue {
                frame: 4,
                sound: SoundEffect::Sleep { phase: SleepPhase::Exhale },
                volume: 0.2,
                pitch_variation: 0.05,
            },
        ]);
        
        Self { cues }
    }
}

/// Audio synchronization system
pub fn sync_animation_audio(
    mut creatures: Query<(
        &CartoonSprite,
        &Transform,
        &mut AnimationAudioState,
        Option<&TerrainInfo>,
    )>,
    audio_config: Res<AnimationAudioConfig>,
    audio: Res<Audio>,
    camera: Query<&Transform, With<Camera>>,
) {
    let camera_pos = camera.single().translation;
    
    for (sprite, transform, mut audio_state, terrain_info) in creatures.iter_mut() {
        let animation = &sprite.animation_state.current;
        let current_frame = sprite.current_frame;
        
        // Check if we've entered a new frame
        if current_frame != audio_state.last_frame {
            audio_state.last_frame = current_frame;
            
            // Look for audio cues on this frame
            if let Some(cues) = audio_config.cues.get(animation) {
                for cue in cues {
                    if cue.frame == current_frame && !audio_state.played_cues.contains(&cue.frame) {
                        // Calculate volume based on distance
                        let distance = camera_pos.distance(transform.translation);
                        let distance_attenuation = (1.0 - (distance / 500.0)).max(0.0);
                        let final_volume = cue.volume * distance_attenuation;
                        
                        if final_volume > 0.01 {
                            // Adjust sound based on context
                            let mut sound = cue.sound.clone();
                            if let SoundEffect::Footstep { ref mut surface } = sound {
                                // Update surface type based on terrain
                                if let Some(terrain) = terrain_info {
                                    *surface = terrain.surface_type;
                                }
                            }
                            
                            // Play the sound
                            play_sound_effect(
                                &audio,
                                sound,
                                final_volume,
                                cue.pitch_variation,
                                transform.translation,
                            );
                        }
                        
                        audio_state.played_cues.insert(cue.frame);
                    }
                }
            }
        }
        
        // Reset played cues when animation loops
        if current_frame < audio_state.last_frame {
            audio_state.played_cues.clear();
        }
    }
}

/// Helper to play sound effects with variations
fn play_sound_effect(
    audio: &Audio,
    effect: SoundEffect,
    volume: f32,
    pitch_variation: f32,
    position: Vec3,
) {
    let (sound_path, base_pitch) = match effect {
        SoundEffect::Footstep { surface } => {
            let path = match surface {
                SurfaceType::Grass => "audio/footstep_grass.ogg",
                SurfaceType::Sand => "audio/footstep_sand.ogg",
                SurfaceType::Stone => "audio/footstep_stone.ogg",
                SurfaceType::Water => "audio/footstep_water.ogg",
                _ => "audio/footstep_generic.ogg",
            };
            (path, 1.0)
        }
        SoundEffect::Eat { food_type } => {
            let path = match food_type {
                FoodType::Berry => "audio/eat_soft.ogg",
                FoodType::Meat => "audio/eat_meat.ogg",
                FoodType::Plant => "audio/eat_crunch.ogg",
                _ => "audio/eat_generic.ogg",
            };
            (path, 1.0)
        }
        SoundEffect::Drink => ("audio/drink.ogg", 1.0),
        SoundEffect::Sleep { phase } => {
            match phase {
                SleepPhase::Inhale => ("audio/snore_in.ogg", 0.9),
                SleepPhase::Exhale => ("audio/snore_out.ogg", 0.9),
            }
        }
        SoundEffect::Vocalization { emotion } => {
            let path = match emotion {
                EmotionType::Happy => "audio/voice_happy.ogg",
                EmotionType::Sad => "audio/voice_sad.ogg",
                EmotionType::Angry => "audio/voice_angry.ogg",
                _ => "audio/voice_neutral.ogg",
            };
            (path, 1.2)
        }
        SoundEffect::Action { action } => {
            // Map action types to sounds
            ("audio/action_generic.ogg", 1.0)
        }
    };
    
    // Apply pitch variation
    let pitch = base_pitch + (rand::random::<f32>() - 0.5) * pitch_variation;
    
    // Load and play the sound
    if let Ok(sound) = audio.load(sound_path) {
        audio.play(sound)
            .with_volume(volume)
            .with_speed(pitch)
            .with_spatial(position);
    }
}
```

## Sprite Atlas Organization Strategy

```rust
/// Sprite atlas organization for efficient loading and rendering
pub struct AtlasOrganizer {
    pub creature_atlases: HashMap<Species, AtlasLayout>,
    pub terrain_atlases: HashMap<BiomeType, AtlasLayout>,
    pub effect_atlases: HashMap<EffectCategory, AtlasLayout>,
}

#[derive(Clone)]
pub struct AtlasLayout {
    pub texture_path: String,
    pub grid_size: (u32, u32),
    pub sprite_size: Vec2,
    pub animations: HashMap<AnimationType, AnimationRange>,
    pub variations: Vec<VariationInfo>,
}

#[derive(Clone)]
pub struct AnimationRange {
    pub start_frame: usize,
    pub frame_count: usize,
    pub fps: f32,
    pub loop_mode: AnimationLoopMode,
}

#[derive(Clone)]
pub struct VariationInfo {
    pub name: String,
    pub row_offset: u32,  // Which row in the atlas
    pub genetic_trait: Option<GeneticTrait>,
}

impl AtlasOrganizer {
    pub fn organize_creature_atlas(species: Species) -> AtlasLayout {
        // Standard layout for all creatures
        let animations = HashMap::from([
            (AnimationType::Idle, AnimationRange { 
                start_frame: 0, frame_count: 4, fps: 4.0, 
                loop_mode: AnimationLoopMode::Loop 
            }),
            (AnimationType::Walk, AnimationRange { 
                start_frame: 4, frame_count: 8, fps: 12.0, 
                loop_mode: AnimationLoopMode::Loop 
            }),
            (AnimationType::Run, AnimationRange { 
                start_frame: 12, frame_count: 6, fps: 18.0, 
                loop_mode: AnimationLoopMode::Loop 
            }),
            (AnimationType::Eat, AnimationRange { 
                start_frame: 18, frame_count: 6, fps: 8.0, 
                loop_mode: AnimationLoopMode::Once 
            }),
            (AnimationType::Sleep, AnimationRange { 
                start_frame: 24, frame_count: 4, fps: 2.0, 
                loop_mode: AnimationLoopMode::Loop 
            }),
            (AnimationType::Talk, AnimationRange { 
                start_frame: 28, frame_count: 8, fps: 10.0, 
                loop_mode: AnimationLoopMode::Loop 
            }),
            (AnimationType::Attack, AnimationRange { 
                start_frame: 36, frame_count: 6, fps: 15.0, 
                loop_mode: AnimationLoopMode::Once 
            }),
            (AnimationType::Death, AnimationRange { 
                start_frame: 42, frame_count: 8, fps: 10.0, 
                loop_mode: AnimationLoopMode::Once 
            }),
        ]);
        
        // Genetic variations (different rows)
        let variations = vec![
            VariationInfo { 
                name: "Normal".to_string(), 
                row_offset: 0, 
                genetic_trait: None 
            },
            VariationInfo { 
                name: "Large".to_string(), 
                row_offset: 1, 
                genetic_trait: Some(GeneticTrait::Size(1.3)) 
            },
            VariationInfo { 
                name: "Small".to_string(), 
                row_offset: 2, 
                genetic_trait: Some(GeneticTrait::Size(0.7)) 
            },
            VariationInfo { 
                name: "Spotted".to_string(), 
                row_offset: 3, 
                genetic_trait: Some(GeneticTrait::Pattern(PatternType::Spots)) 
            },
            VariationInfo { 
                name: "Striped".to_string(), 
                row_offset: 4, 
                genetic_trait: Some(GeneticTrait::Pattern(PatternType::Stripes)) 
            },
        ];
        
        AtlasLayout {
            texture_path: format!("sprites/creatures/{}/atlas.png", species.to_string()),
            grid_size: (8, 8), // 8x8 grid of sprites
            sprite_size: Vec2::new(48.0, 48.0),
            animations,
            variations,
        }
    }
    
    pub fn select_variation_for_genetics(
        layout: &AtlasLayout,
        genetics: &Genetics,
    ) -> u32 {
        // Match genetic traits to variations
        for variation in &layout.variations {
            if let Some(trait_match) = &variation.genetic_trait {
                match trait_match {
                    GeneticTrait::Size(target_size) => {
                        let size_diff = (genetics.size_modifier - target_size).abs();
                        if size_diff < 0.1 {
                            return variation.row_offset;
                        }
                    }
                    GeneticTrait::Pattern(pattern) => {
                        if genetics.pattern_type == *pattern {
                            return variation.row_offset;
                        }
                    }
                    _ => {}
                }
            }
        }
        
        0 // Default variation
    }
}

/// Runtime atlas generation for modded content
pub struct DynamicAtlasBuilder {
    packer: AtlasPacker,
    texture_data: Vec<u8>,
    sprite_mappings: HashMap<String, Rect>,
}

impl DynamicAtlasBuilder {
    pub fn new(size: u32) -> Self {
        Self {
            packer: AtlasPacker::new(size, size),
            texture_data: vec![0; (size * size * 4) as usize],
            sprite_mappings: HashMap::new(),
        }
    }
    
    pub fn add_sprite(&mut self, name: String, image_data: &[u8], width: u32, height: u32) -> Result<(), String> {
        // Pack the sprite
        let rect = self.packer.pack(width, height)
            .ok_or("Atlas full, cannot pack sprite")?;
        
        // Copy image data to atlas
        self.copy_to_atlas(image_data, &rect);
        
        // Store mapping
        self.sprite_mappings.insert(name, rect);
        
        Ok(())
    }
    
    fn copy_to_atlas(&mut self, src: &[u8], rect: &Rect) {
        let atlas_width = self.packer.width;
        
        for y in 0..rect.height {
            for x in 0..rect.width {
                let src_idx = ((y * rect.width + x) * 4) as usize;
                let dst_idx = (((rect.y + y) * atlas_width + (rect.x + x)) * 4) as usize;
                
                // Copy RGBA
                self.texture_data[dst_idx..dst_idx + 4]
                    .copy_from_slice(&src[src_idx..src_idx + 4]);
            }
        }
    }
    
    pub fn build(self) -> (Vec<u8>, HashMap<String, Rect>) {
        (self.texture_data, self.sprite_mappings)
    }
}
```

## Performance Quality Settings

```rust
/// Detailed quality settings for performance scaling
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CartoonQualitySettings {
    // Rendering quality
    pub sprite_resolution_scale: f32,     // 0.5 - 2.0
    pub animation_framerate_scale: f32,   // 0.25 - 1.0
    pub particle_density: f32,            // 0.0 - 1.0
    pub shadow_quality: ShadowQuality,
    pub texture_filtering: TextureFiltering,
    
    // LOD settings
    pub lod_bias: f32,                    // -1.0 (favor quality) to 1.0 (favor performance)
    pub max_visible_creatures: Option<usize>,
    pub cull_distance_multiplier: f32,
    
    // Effects
    pub enable_weather_effects: bool,
    pub enable_particle_effects: bool,
    pub enable_expression_blending: bool,
    pub enable_dynamic_shadows: bool,
    pub enable_water_animation: bool,
    
    // UI
    pub ui_animation_speed: f32,
    pub speech_bubble_quality: SpeechBubbleQuality,
    pub enable_ui_blur: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ShadowQuality {
    None,
    Simple,    // Blob shadows
    Detailed,  // Sprite-based shadows
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TextureFiltering {
    Nearest,   // Pixelated
    Linear,    // Smooth
    Trilinear, // Smooth with mipmaps
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SpeechBubbleQuality {
    Simple,    // Rectangle with tail
    Fancy,     // 9-slice with animations
}

impl CartoonQualitySettings {
    pub fn preset_ultra() -> Self {
        Self {
            sprite_resolution_scale: 2.0,
            animation_framerate_scale: 1.0,
            particle_density: 1.0,
            shadow_quality: ShadowQuality::Detailed,
            texture_filtering: TextureFiltering::Trilinear,
            lod_bias: -0.5,
            max_visible_creatures: None,
            cull_distance_multiplier: 2.0,
            enable_weather_effects: true,
            enable_particle_effects: true,
            enable_expression_blending: true,
            enable_dynamic_shadows: true,
            enable_water_animation: true,
            ui_animation_speed: 1.0,
            speech_bubble_quality: SpeechBubbleQuality::Fancy,
            enable_ui_blur: true,
        }
    }
    
    pub fn preset_high() -> Self {
        Self {
            sprite_resolution_scale: 1.0,
            animation_framerate_scale: 1.0,
            particle_density: 0.8,
            shadow_quality: ShadowQuality::Detailed,
            texture_filtering: TextureFiltering::Linear,
            lod_bias: 0.0,
            max_visible_creatures: Some(1000),
            cull_distance_multiplier: 1.5,
            enable_weather_effects: true,
            enable_particle_effects: true,
            enable_expression_blending: true,
            enable_dynamic_shadows: true,
            enable_water_animation: true,
            ui_animation_speed: 1.0,
            speech_bubble_quality: SpeechBubbleQuality::Fancy,
            enable_ui_blur: true,
        }
    }
    
    pub fn preset_medium() -> Self {
        Self {
            sprite_resolution_scale: 1.0,
            animation_framerate_scale: 0.75,
            particle_density: 0.5,
            shadow_quality: ShadowQuality::Simple,
            texture_filtering: TextureFiltering::Linear,
            lod_bias: 0.25,
            max_visible_creatures: Some(500),
            cull_distance_multiplier: 1.0,
            enable_weather_effects: true,
            enable_particle_effects: true,
            enable_expression_blending: false,
            enable_dynamic_shadows: false,
            enable_water_animation: true,
            ui_animation_speed: 1.0,
            speech_bubble_quality: SpeechBubbleQuality::Simple,
            enable_ui_blur: false,
        }
    }
    
    pub fn preset_low() -> Self {
        Self {
            sprite_resolution_scale: 0.75,
            animation_framerate_scale: 0.5,
            particle_density: 0.25,
            shadow_quality: ShadowQuality::Simple,
            texture_filtering: TextureFiltering::Nearest,
            lod_bias: 0.5,
            max_visible_creatures: Some(200),
            cull_distance_multiplier: 0.75,
            enable_weather_effects: false,
            enable_particle_effects: true,
            enable_expression_blending: false,
            enable_dynamic_shadows: false,
            enable_water_animation: false,
            ui_animation_speed: 0.5,
            speech_bubble_quality: SpeechBubbleQuality::Simple,
            enable_ui_blur: false,
        }
    }
    
    pub fn preset_minimum() -> Self {
        Self {
            sprite_resolution_scale: 0.5,
            animation_framerate_scale: 0.25,
            particle_density: 0.0,
            shadow_quality: ShadowQuality::None,
            texture_filtering: TextureFiltering::Nearest,
            lod_bias: 1.0,
            max_visible_creatures: Some(100),
            cull_distance_multiplier: 0.5,
            enable_weather_effects: false,
            enable_particle_effects: false,
            enable_expression_blending: false,
            enable_dynamic_shadows: false,
            enable_water_animation: false,
            ui_animation_speed: 0.0,
            speech_bubble_quality: SpeechBubbleQuality::Simple,
            enable_ui_blur: false,
        }
    }
    
    /// Dynamically adjust quality based on performance
    pub fn auto_adjust(&mut self, current_fps: f32, target_fps: f32) {
        let performance_ratio = current_fps / target_fps;
        
        if performance_ratio < 0.8 {
            // Decrease quality
            self.particle_density = (self.particle_density - 0.1).max(0.0);
            self.animation_framerate_scale = (self.animation_framerate_scale - 0.1).max(0.25);
            self.lod_bias = (self.lod_bias + 0.1).min(1.0);
            
            if performance_ratio < 0.6 {
                self.enable_expression_blending = false;
                self.enable_weather_effects = false;
                self.shadow_quality = ShadowQuality::Simple;
            }
            
            if performance_ratio < 0.4 {
                self.enable_particle_effects = false;
                self.shadow_quality = ShadowQuality::None;
                self.sprite_resolution_scale = (self.sprite_resolution_scale - 0.25).max(0.5);
            }
        } else if performance_ratio > 1.2 {
            // Increase quality gradually
            if self.particle_density < 1.0 {
                self.particle_density = (self.particle_density + 0.05).min(1.0);
            } else if self.animation_framerate_scale < 1.0 {
                self.animation_framerate_scale = (self.animation_framerate_scale + 0.05).min(1.0);
            } else if self.lod_bias > -0.5 {
                self.lod_bias = (self.lod_bias - 0.05).max(-0.5);
            }
        }
    }
}
```

## Additional Integration Notes

### Mod Support Validation

```rust
/// Validate modded sprite assets
pub fn validate_mod_sprites(
    path: &Path,
    expected_format: &SpriteFormat,
) -> Result<ModValidation, ValidationError> {
    let image = image::open(path)?;
    
    // Check dimensions
    if image.width() % expected_format.frame_width != 0 ||
       image.height() % expected_format.frame_height != 0 {
        return Err(ValidationError::InvalidDimensions);
    }
    
    // Check format
    if image.color() != expected_format.color_type {
        return Err(ValidationError::InvalidColorFormat);
    }
    
    // Validate animation frame counts
    let frames_per_row = image.width() / expected_format.frame_width;
    let total_rows = image.height() / expected_format.frame_height;
    let total_frames = frames_per_row * total_rows;
    
    if total_frames < expected_format.min_frames {
        return Err(ValidationError::InsufficientFrames);
    }
    
    Ok(ModValidation {
        sprite_count: total_frames,
        has_variations: total_rows > 1,
        supports_genetics: total_rows >= 5, // Normal + 4 variations
    })
}
```

This completes the implementation details for the cartoon graphics system, filling in all the identified gaps with specific formulas, algorithms, and integration points.