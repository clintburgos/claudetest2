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

## Visual Testing Harness

```rust
/// Comprehensive testing harness for cartoon graphics implementation
pub struct CartoonTestScene {
    pub biome_types: Vec<BiomeType>,
    pub creature_count: usize,
    pub emotion_test_sequence: Vec<EmotionType>,
    pub particle_stress_test: bool,
    pub animation_cycles: HashMap<AnimationType, bool>,
    pub weather_cycle: bool,
    pub day_night_speed: f32,
}

impl CartoonTestScene {
    pub fn new_basic() -> Self {
        Self {
            biome_types: vec![BiomeType::Forest],
            creature_count: 10,
            emotion_test_sequence: vec![],
            particle_stress_test: false,
            animation_cycles: HashMap::new(),
            weather_cycle: false,
            day_night_speed: 1.0,
        }
    }
    
    pub fn new_comprehensive() -> Self {
        let mut animation_cycles = HashMap::new();
        for anim_type in AnimationType::iter() {
            animation_cycles.insert(anim_type, true);
        }
        
        Self {
            biome_types: BiomeType::iter().collect(),
            creature_count: 50,
            emotion_test_sequence: EmotionType::iter().collect(),
            particle_stress_test: true,
            animation_cycles,
            weather_cycle: true,
            day_night_speed: 10.0, // 10x speed for testing
        }
    }
    
    pub fn new_performance_test() -> Self {
        Self {
            biome_types: vec![BiomeType::Grassland], // Simple biome
            creature_count: 500,
            emotion_test_sequence: vec![],
            particle_stress_test: true,
            animation_cycles: HashMap::from([(AnimationType::Walk, true)]),
            weather_cycle: false,
            day_night_speed: 0.0, // Frozen time
        }
    }
    
    pub fn new_biome_transition_test() -> Self {
        Self {
            biome_types: BiomeType::iter().collect(),
            creature_count: 20,
            emotion_test_sequence: vec![],
            particle_stress_test: false,
            animation_cycles: HashMap::from([(AnimationType::Walk, true)]),
            weather_cycle: false,
            day_night_speed: 1.0,
        }
    }
}

/// Test scene spawner
pub fn spawn_test_scene(
    mut commands: Commands,
    test_scene: Res<CartoonTestScene>,
    asset_server: Res<AssetServer>,
) {
    // Spawn test biomes in a grid
    let biome_size = 20;
    let grid_size = (test_scene.biome_types.len() as f32).sqrt().ceil() as i32;
    
    for (idx, biome_type) in test_scene.biome_types.iter().enumerate() {
        let x = (idx as i32 % grid_size) * biome_size;
        let z = (idx as i32 / grid_size) * biome_size;
        
        spawn_biome_chunk(
            &mut commands,
            *biome_type,
            IVec2::new(x, z),
            IVec2::new(biome_size, biome_size),
        );
    }
    
    // Spawn test creatures
    let mut rng = thread_rng();
    for i in 0..test_scene.creature_count {
        let position = Vec3::new(
            rng.gen_range(-50.0..50.0),
            0.0,
            rng.gen_range(-50.0..50.0),
        );
        
        let mut creature = commands.spawn(CreatureBundle {
            creature: Creature {
                species: random_species(&mut rng),
                genetics: Genetics::random(&mut rng),
                ..default()
            },
            position: Position(position),
            ..default()
        });
        
        // Add test components
        if !test_scene.emotion_test_sequence.is_empty() {
            creature.insert(EmotionTestSequence {
                sequence: test_scene.emotion_test_sequence.clone(),
                current_index: 0,
                timer: Timer::from_seconds(2.0, TimerMode::Repeating),
            });
        }
        
        if !test_scene.animation_cycles.is_empty() {
            creature.insert(AnimationTestCycle {
                animations: test_scene.animation_cycles.keys().cloned().collect(),
                current_index: 0,
                timer: Timer::from_seconds(3.0, TimerMode::Repeating),
            });
        }
    }
    
    // Spawn particle stress test emitters
    if test_scene.particle_stress_test {
        for _ in 0..100 {
            let position = Vec3::new(
                rng.gen_range(-100.0..100.0),
                rng.gen_range(0.0..10.0),
                rng.gen_range(-100.0..100.0),
            );
            
            commands.spawn(ParticleEmitterBundle {
                emitter: ParticleEmitter {
                    effect_type: random_particle_effect(&mut rng),
                    spawn_rate: 50.0,
                    lifetime: 10.0,
                    ..default()
                },
                transform: Transform::from_translation(position),
                ..default()
            });
        }
    }
}

/// Visual regression test recorder
pub struct VisualTestRecorder {
    pub test_name: String,
    pub frames_to_capture: Vec<u32>,
    pub current_frame: u32,
    pub output_dir: PathBuf,
}

impl VisualTestRecorder {
    pub fn new(test_name: &str, frames: Vec<u32>) -> Self {
        let output_dir = PathBuf::from("tests/visual_regression")
            .join(test_name)
            .join(chrono::Local::now().format("%Y%m%d_%H%M%S").to_string());
        
        std::fs::create_dir_all(&output_dir).expect("Failed to create output directory");
        
        Self {
            test_name: test_name.to_string(),
            frames_to_capture: frames,
            current_frame: 0,
            output_dir,
        }
    }
}

pub fn visual_test_capture_system(
    mut recorder: ResMut<VisualTestRecorder>,
    windows: Query<&Window>,
    mut screenshot_manager: ResMut<ScreenshotManager>,
) {
    recorder.current_frame += 1;
    
    if recorder.frames_to_capture.contains(&recorder.current_frame) {
        let filename = format!("frame_{:06}.png", recorder.current_frame);
        let path = recorder.output_dir.join(filename);
        
        if let Ok(window) = windows.get_single() {
            screenshot_manager.save_screenshot_to_disk(window, path)
                .expect("Failed to save screenshot");
        }
    }
}
```

## Fallback Specifications

### Asset Loading Failures

```rust
/// Fallback system for missing or corrupted sprite assets
pub struct SpriteFallbackSystem {
    pub fallback_textures: HashMap<AssetCategory, Handle<Image>>,
    pub error_texture: Handle<Image>,
    pub placeholder_colors: HashMap<EntityType, Color>,
}

impl SpriteFallbackSystem {
    pub fn handle_sprite_load_failure(
        &self,
        entity_type: EntityType,
        error: AssetLoadError,
    ) -> FallbackSprite {
        match error {
            AssetLoadError::NotFound(path) => {
                warn!("Sprite not found: {}, using fallback", path);
                FallbackSprite::ColoredSquare {
                    color: self.placeholder_colors.get(&entity_type)
                        .copied()
                        .unwrap_or(Color::PINK),
                    size: Vec2::new(48.0, 48.0),
                }
            }
            AssetLoadError::InvalidFormat => {
                error!("Invalid sprite format, using error texture");
                FallbackSprite::ErrorTexture(self.error_texture.clone())
            }
            AssetLoadError::CorruptedData => {
                error!("Corrupted sprite data, regenerating placeholder");
                self.generate_procedural_sprite(entity_type)
            }
        }
    }
    
    fn generate_procedural_sprite(&self, entity_type: EntityType) -> FallbackSprite {
        // Generate a simple procedural sprite
        let mut image_data = vec![255u8; 48 * 48 * 4]; // White background
        let color = self.placeholder_colors.get(&entity_type)
            .copied()
            .unwrap_or(Color::GRAY);
        
        // Draw a simple shape based on entity type
        match entity_type {
            EntityType::Creature(_) => self.draw_circle(&mut image_data, 48, color),
            EntityType::Resource(_) => self.draw_diamond(&mut image_data, 48, color),
            EntityType::Terrain(_) => self.draw_square(&mut image_data, 48, color),
            _ => self.draw_x(&mut image_data, 48, color),
        }
        
        FallbackSprite::ProceduralImage(image_data)
    }
}

pub enum FallbackSprite {
    ColoredSquare { color: Color, size: Vec2 },
    ErrorTexture(Handle<Image>),
    ProceduralImage(Vec<u8>),
}
```

### Shader Compilation Failures

```rust
/// Shader fallback system for compilation errors
pub struct ShaderFallbackSystem {
    pub fallback_shaders: HashMap<ShaderType, Handle<Shader>>,
    pub disable_features: HashSet<GraphicsFeature>,
}

impl ShaderFallbackSystem {
    pub fn handle_shader_error(
        &mut self,
        shader_type: ShaderType,
        error: ShaderError,
    ) -> ShaderFallback {
        match error {
            ShaderError::CompilationFailed(msg) => {
                error!("Shader compilation failed: {}", msg);
                
                // Disable the feature that requires this shader
                match shader_type {
                    ShaderType::Water => {
                        self.disable_features.insert(GraphicsFeature::WaterAnimation);
                        ShaderFallback::UseSimple(self.fallback_shaders[&ShaderType::BasicSprite].clone())
                    }
                    ShaderType::DayNight => {
                        self.disable_features.insert(GraphicsFeature::DynamicLighting);
                        ShaderFallback::DisableEffect
                    }
                    _ => ShaderFallback::UseDefault
                }
            }
            ShaderError::UnsupportedGPU => {
                warn!("GPU doesn't support shader features, downgrading");
                ShaderFallback::DowngradeToCompatible
            }
        }
    }
}

pub enum ShaderFallback {
    UseSimple(Handle<Shader>),
    UseDefault,
    DisableEffect,
    DowngradeToCompatible,
}

/// Automatic quality degradation on shader failure
pub fn apply_shader_fallback(
    fallback: ShaderFallback,
    mut quality_settings: ResMut<CartoonQualitySettings>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    match fallback {
        ShaderFallback::DisableEffect => {
            quality_settings.enable_water_animation = false;
            quality_settings.enable_dynamic_shadows = false;
        }
        ShaderFallback::DowngradeToCompatible => {
            // Switch to compatibility mode
            quality_settings.shadow_quality = ShadowQuality::None;
            quality_settings.texture_filtering = TextureFiltering::Nearest;
        }
        _ => {}
    }
}
```

### Memory Budget Exceeded

```rust
/// Memory pressure response system
pub struct MemoryFallbackSystem {
    pub memory_limit: usize,
    pub current_usage: AtomicUsize,
    pub reduction_strategies: Vec<MemoryReductionStrategy>,
}

#[derive(Clone)]
pub enum MemoryReductionStrategy {
    UnloadDistantChunks { distance: f32 },
    ReduceTextureQuality { factor: f32 },
    DisableParticles,
    SimplifyAnimations,
    ReduceCreatureLimit { new_limit: usize },
}

impl MemoryFallbackSystem {
    pub fn handle_memory_pressure(&mut self) -> Vec<MemoryReductionStrategy> {
        let usage = self.current_usage.load(Ordering::Relaxed);
        let pressure_ratio = usage as f32 / self.memory_limit as f32;
        
        let mut strategies = Vec::new();
        
        if pressure_ratio > 0.9 {
            // Critical - apply aggressive reductions
            strategies.push(MemoryReductionStrategy::DisableParticles);
            strategies.push(MemoryReductionStrategy::SimplifyAnimations);
            strategies.push(MemoryReductionStrategy::ReduceCreatureLimit { new_limit: 200 });
        } else if pressure_ratio > 0.8 {
            // High - apply moderate reductions
            strategies.push(MemoryReductionStrategy::UnloadDistantChunks { distance: 100.0 });
            strategies.push(MemoryReductionStrategy::ReduceTextureQuality { factor: 0.5 });
        } else if pressure_ratio > 0.7 {
            // Moderate - apply light reductions
            strategies.push(MemoryReductionStrategy::UnloadDistantChunks { distance: 150.0 });
        }
        
        strategies
    }
}

pub fn apply_memory_reduction(
    strategies: Vec<MemoryReductionStrategy>,
    mut commands: Commands,
    mut quality_settings: ResMut<CartoonQualitySettings>,
    chunks: Query<(Entity, &ChunkPosition, &Transform)>,
    camera: Query<&Transform, With<Camera>>,
) {
    let camera_pos = camera.single().translation;
    
    for strategy in strategies {
        match strategy {
            MemoryReductionStrategy::UnloadDistantChunks { distance } => {
                for (entity, chunk_pos, transform) in chunks.iter() {
                    if transform.translation.distance(camera_pos) > distance {
                        commands.entity(entity).despawn_recursive();
                    }
                }
            }
            MemoryReductionStrategy::ReduceTextureQuality { factor } => {
                quality_settings.sprite_resolution_scale *= factor;
            }
            MemoryReductionStrategy::DisableParticles => {
                quality_settings.enable_particle_effects = false;
            }
            MemoryReductionStrategy::SimplifyAnimations => {
                quality_settings.animation_framerate_scale *= 0.5;
            }
            MemoryReductionStrategy::ReduceCreatureLimit { new_limit } => {
                quality_settings.max_visible_creatures = Some(new_limit);
            }
        }
    }
}
```

### Invalid Mod Sprites

```rust
/// Mod sprite validation and fallback
pub struct ModSpriteFallback {
    pub validation_rules: ModValidationRules,
    pub auto_fix_attempts: bool,
}

impl ModSpriteFallback {
    pub fn handle_invalid_mod_sprite(
        &self,
        mod_name: &str,
        sprite_path: &Path,
        validation_error: ValidationError,
    ) -> ModFallbackAction {
        match validation_error {
            ValidationError::InvalidDimensions => {
                if self.auto_fix_attempts {
                    ModFallbackAction::ResizeToValid
                } else {
                    ModFallbackAction::RejectWithMessage(
                        format!("Sprite dimensions must be multiples of 48x48")
                    )
                }
            }
            ValidationError::InvalidColorFormat => {
                ModFallbackAction::ConvertToRGBA
            }
            ValidationError::InsufficientFrames => {
                ModFallbackAction::UseDefaultAnimation
            }
            ValidationError::FileTooLarge => {
                ModFallbackAction::CompressAndRetry
            }
            _ => ModFallbackAction::UseVanillaSprite
        }
    }
}

pub enum ModFallbackAction {
    ResizeToValid,
    ConvertToRGBA,
    UseDefaultAnimation,
    CompressAndRetry,
    UseVanillaSprite,
    RejectWithMessage(String),
}

/// Apply mod fallback actions
pub fn apply_mod_fallback(
    action: ModFallbackAction,
    sprite_data: &mut Vec<u8>,
    mod_registry: &mut ModRegistry,
) -> Result<(), String> {
    match action {
        ModFallbackAction::ResizeToValid => {
            // Resize to nearest valid dimensions
            let resized = resize_to_grid(sprite_data, 48)?;
            *sprite_data = resized;
            Ok(())
        }
        ModFallbackAction::ConvertToRGBA => {
            // Convert color format
            let converted = convert_to_rgba(sprite_data)?;
            *sprite_data = converted;
            Ok(())
        }
        ModFallbackAction::UseDefaultAnimation => {
            // Fall back to vanilla animation
            warn!("Using default animation for invalid mod sprite");
            Ok(())
        }
        ModFallbackAction::CompressAndRetry => {
            // Compress the image
            let compressed = compress_image(sprite_data, 0.8)?;
            if compressed.len() < MAX_SPRITE_SIZE {
                *sprite_data = compressed;
                Ok(())
            } else {
                Err("Sprite still too large after compression".to_string())
            }
        }
        ModFallbackAction::UseVanillaSprite => {
            info!("Falling back to vanilla sprite");
            Ok(())
        }
        ModFallbackAction::RejectWithMessage(msg) => {
            Err(msg)
        }
    }
}
```

### Network Synchronization Failures

```rust
/// Network visual state fallback
pub struct NetworkFallbackSystem {
    pub interpolation_buffer: HashMap<Entity, Vec<VisualState>>,
    pub prediction_enabled: bool,
}

impl NetworkFallbackSystem {
    pub fn handle_missing_visual_update(
        &mut self,
        entity: Entity,
        last_known_state: &VisualState,
        time_since_update: f32,
    ) -> VisualState {
        if self.prediction_enabled && time_since_update < 1.0 {
            // Predict based on last known velocity
            self.predict_visual_state(last_known_state, time_since_update)
        } else {
            // Use interpolation buffer or freeze
            if let Some(buffer) = self.interpolation_buffer.get(&entity) {
                if buffer.len() >= 2 {
                    self.interpolate_states(&buffer[buffer.len()-2], &buffer[buffer.len()-1])
                } else {
                    last_known_state.clone()
                }
            } else {
                last_known_state.clone()
            }
        }
    }
    
    fn predict_visual_state(&self, state: &VisualState, delta: f32) -> VisualState {
        let mut predicted = state.clone();
        
        // Simple linear prediction
        if let Some(velocity) = state.velocity {
            predicted.position += velocity * delta;
        }
        
        // Continue animations
        predicted.animation_frame += state.animation_speed * delta;
        
        predicted
    }
}
```

## Concrete Performance Thresholds

### FPS-Based Quality Degradation

```rust
/// Performance thresholds for automatic quality adjustment
pub struct PerformanceThresholds {
    pub target_fps: f32,
    pub critical_fps: f32,
    pub excellent_fps: f32,
    pub measurement_window: Duration,
    pub reaction_time: Duration,
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            target_fps: 60.0,      // Target framerate
            critical_fps: 30.0,    // Below this is unplayable
            excellent_fps: 120.0,  // Above this, increase quality
            measurement_window: Duration::from_secs(2),
            reaction_time: Duration::from_millis(500),
        }
    }
}

/// Quality levels with specific performance targets
#[derive(Debug, Clone)]
pub struct QualityLevelThresholds {
    pub ultra: PerformanceTarget,
    pub high: PerformanceTarget,
    pub medium: PerformanceTarget,
    pub low: PerformanceTarget,
    pub minimum: PerformanceTarget,
}

#[derive(Debug, Clone)]
pub struct PerformanceTarget {
    pub min_fps: f32,
    pub target_fps: f32,
    pub max_creatures: usize,
    pub max_particles: usize,
    pub max_draw_calls: usize,
    pub memory_budget_mb: usize,
    pub view_distance: f32,
}

impl Default for QualityLevelThresholds {
    fn default() -> Self {
        Self {
            ultra: PerformanceTarget {
                min_fps: 60.0,
                target_fps: 120.0,
                max_creatures: 1000,
                max_particles: 10000,
                max_draw_calls: 5000,
                memory_budget_mb: 2048,
                view_distance: 500.0,
            },
            high: PerformanceTarget {
                min_fps: 60.0,
                target_fps: 90.0,
                max_creatures: 500,
                max_particles: 5000,
                max_draw_calls: 3000,
                memory_budget_mb: 1024,
                view_distance: 300.0,
            },
            medium: PerformanceTarget {
                min_fps: 45.0,
                target_fps: 60.0,
                max_creatures: 300,
                max_particles: 2000,
                max_draw_calls: 1500,
                memory_budget_mb: 512,
                view_distance: 200.0,
            },
            low: PerformanceTarget {
                min_fps: 30.0,
                target_fps: 45.0,
                max_creatures: 150,
                max_particles: 500,
                max_draw_calls: 750,
                memory_budget_mb: 256,
                view_distance: 150.0,
            },
            minimum: PerformanceTarget {
                min_fps: 20.0,
                target_fps: 30.0,
                max_creatures: 50,
                max_particles: 0,
                max_draw_calls: 300,
                memory_budget_mb: 128,
                view_distance: 100.0,
            },
        }
    }
}

/// Dynamic performance monitoring and adjustment
pub struct PerformanceMonitor {
    pub fps_history: VecDeque<f32>,
    pub frame_time_history: VecDeque<Duration>,
    pub quality_change_cooldown: Timer,
    pub current_quality: QualityLevel,
    pub thresholds: PerformanceThresholds,
}

impl PerformanceMonitor {
    pub fn update(&mut self, frame_time: Duration) {
        let fps = 1.0 / frame_time.as_secs_f32();
        self.fps_history.push_back(fps);
        self.frame_time_history.push_back(frame_time);
        
        // Keep only recent history
        while self.fps_history.len() > 120 { // 2 seconds at 60 FPS
            self.fps_history.pop_front();
            self.frame_time_history.pop_front();
        }
    }
    
    pub fn should_adjust_quality(&mut self, delta: Duration) -> Option<QualityAdjustment> {
        self.quality_change_cooldown.tick(delta);
        
        if !self.quality_change_cooldown.finished() {
            return None;
        }
        
        let avg_fps = self.calculate_average_fps();
        let fps_stability = self.calculate_fps_stability();
        
        // Don't adjust if FPS is unstable (loading, etc)
        if fps_stability > 0.3 {
            return None;
        }
        
        match self.current_quality {
            QualityLevel::Ultra => {
                if avg_fps < 55.0 {
                    self.trigger_quality_change();
                    Some(QualityAdjustment::Decrease)
                } else {
                    None
                }
            }
            QualityLevel::High => {
                if avg_fps < 55.0 {
                    self.trigger_quality_change();
                    Some(QualityAdjustment::Decrease)
                } else if avg_fps > 110.0 {
                    self.trigger_quality_change();
                    Some(QualityAdjustment::Increase)
                } else {
                    None
                }
            }
            QualityLevel::Medium => {
                if avg_fps < 40.0 {
                    self.trigger_quality_change();
                    Some(QualityAdjustment::Decrease)
                } else if avg_fps > 80.0 {
                    self.trigger_quality_change();
                    Some(QualityAdjustment::Increase)
                } else {
                    None
                }
            }
            QualityLevel::Low => {
                if avg_fps < 25.0 {
                    self.trigger_quality_change();
                    Some(QualityAdjustment::Decrease)
                } else if avg_fps > 55.0 {
                    self.trigger_quality_change();
                    Some(QualityAdjustment::Increase)
                } else {
                    None
                }
            }
            QualityLevel::Minimum => {
                if avg_fps > 45.0 {
                    self.trigger_quality_change();
                    Some(QualityAdjustment::Increase)
                } else {
                    None
                }
            }
        }
    }
    
    fn trigger_quality_change(&mut self) {
        self.quality_change_cooldown = Timer::from_seconds(5.0, TimerMode::Once);
    }
    
    fn calculate_average_fps(&self) -> f32 {
        if self.fps_history.is_empty() {
            return 60.0;
        }
        self.fps_history.iter().sum::<f32>() / self.fps_history.len() as f32
    }
    
    fn calculate_fps_stability(&self) -> f32 {
        if self.fps_history.len() < 10 {
            return 1.0; // Not enough data
        }
        
        let avg = self.calculate_average_fps();
        let variance = self.fps_history.iter()
            .map(|fps| (fps - avg).powi(2))
            .sum::<f32>() / self.fps_history.len() as f32;
        
        variance.sqrt() / avg // Coefficient of variation
    }
}

/// Specific degradation actions based on performance
pub fn apply_performance_based_quality(
    adjustment: QualityAdjustment,
    current_quality: &mut QualityLevel,
    settings: &mut CartoonQualitySettings,
    targets: &QualityLevelThresholds,
) {
    match adjustment {
        QualityAdjustment::Increase => {
            *current_quality = current_quality.increase();
            match current_quality {
                QualityLevel::Ultra => *settings = CartoonQualitySettings::preset_ultra(),
                QualityLevel::High => *settings = CartoonQualitySettings::preset_high(),
                QualityLevel::Medium => *settings = CartoonQualitySettings::preset_medium(),
                QualityLevel::Low => *settings = CartoonQualitySettings::preset_low(),
                QualityLevel::Minimum => *settings = CartoonQualitySettings::preset_minimum(),
            }
        }
        QualityAdjustment::Decrease => {
            *current_quality = current_quality.decrease();
            match current_quality {
                QualityLevel::Ultra => *settings = CartoonQualitySettings::preset_ultra(),
                QualityLevel::High => *settings = CartoonQualitySettings::preset_high(),
                QualityLevel::Medium => *settings = CartoonQualitySettings::preset_medium(),
                QualityLevel::Low => *settings = CartoonQualitySettings::preset_low(),
                QualityLevel::Minimum => *settings = CartoonQualitySettings::preset_minimum(),
            }
        }
    }
}

/// Frame time budget allocation
pub struct FrameBudget {
    pub total_ms: f32,
    pub simulation_ms: f32,
    pub rendering_ms: f32,
    pub particles_ms: f32,
    pub ui_ms: f32,
    pub audio_ms: f32,
}

impl FrameBudget {
    pub fn for_target_fps(fps: f32) -> Self {
        let total_ms = 1000.0 / fps;
        
        Self {
            total_ms,
            simulation_ms: total_ms * 0.3,     // 30% for simulation
            rendering_ms: total_ms * 0.4,      // 40% for rendering
            particles_ms: total_ms * 0.15,     // 15% for particles
            ui_ms: total_ms * 0.1,             // 10% for UI
            audio_ms: total_ms * 0.05,         // 5% for audio
        }
    }
    
    pub fn is_over_budget(&self, component: BudgetComponent, time_ms: f32) -> bool {
        match component {
            BudgetComponent::Simulation => time_ms > self.simulation_ms,
            BudgetComponent::Rendering => time_ms > self.rendering_ms,
            BudgetComponent::Particles => time_ms > self.particles_ms,
            BudgetComponent::UI => time_ms > self.ui_ms,
            BudgetComponent::Audio => time_ms > self.audio_ms,
        }
    }
}
```

## Asset Loading Priority System

```rust
/// Asset loading priorities based on gameplay context
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AssetPriority {
    Critical = 0,      // Must load before gameplay
    High = 1,          // Load ASAP for smooth experience
    Normal = 2,        // Standard loading priority
    Low = 3,           // Load when resources available
    Background = 4,    // Load only during idle time
}

/// Context-aware asset prioritization
pub struct AssetPrioritySystem {
    pub game_state: GameState,
    pub camera_position: Vec3,
    pub player_creatures: Vec<Entity>,
    pub view_distance: f32,
}

impl AssetPrioritySystem {
    pub fn calculate_priority(
        &self,
        asset_type: AssetType,
        world_position: Option<Vec3>,
    ) -> AssetPriority {
        match self.game_state {
            GameState::MainMenu => self.menu_priority(asset_type),
            GameState::Loading => self.loading_priority(asset_type),
            GameState::Playing => self.gameplay_priority(asset_type, world_position),
            GameState::Paused => AssetPriority::Low,
            GameState::Cutscene => self.cutscene_priority(asset_type),
        }
    }
    
    fn menu_priority(&self, asset_type: AssetType) -> AssetPriority {
        match asset_type {
            AssetType::UISprite => AssetPriority::Critical,
            AssetType::MenuBackground => AssetPriority::Critical,
            AssetType::ButtonSound => AssetPriority::High,
            AssetType::CreaturePreview => AssetPriority::High,
            _ => AssetPriority::Background,
        }
    }
    
    fn loading_priority(&self, asset_type: AssetType) -> AssetPriority {
        match asset_type {
            AssetType::TerrainTile => AssetPriority::Critical,
            AssetType::CreatureSprite => AssetPriority::Critical,
            AssetType::ResourceSprite => AssetPriority::High,
            AssetType::ParticleTexture => AssetPriority::Normal,
            AssetType::WeatherEffect => AssetPriority::Low,
            AssetType::AmbientSound => AssetPriority::Low,
            _ => AssetPriority::Normal,
        }
    }
    
    fn gameplay_priority(
        &self,
        asset_type: AssetType,
        world_position: Option<Vec3>,
    ) -> AssetPriority {
        // Distance-based priority for positional assets
        if let Some(pos) = world_position {
            let distance = pos.distance(self.camera_position);
            let distance_factor = distance / self.view_distance;
            
            match asset_type {
                AssetType::CreatureSprite => {
                    if distance_factor < 0.2 {
                        AssetPriority::Critical
                    } else if distance_factor < 0.5 {
                        AssetPriority::High
                    } else if distance_factor < 1.0 {
                        AssetPriority::Normal
                    } else {
                        AssetPriority::Low
                    }
                }
                AssetType::TerrainTile => {
                    if distance_factor < 0.3 {
                        AssetPriority::Critical
                    } else if distance_factor < 0.7 {
                        AssetPriority::High
                    } else {
                        AssetPriority::Normal
                    }
                }
                AssetType::ParticleTexture => {
                    if distance_factor < 0.5 {
                        AssetPriority::Normal
                    } else {
                        AssetPriority::Low
                    }
                }
                _ => AssetPriority::Normal,
            }
        } else {
            // Non-positional assets
            match asset_type {
                AssetType::UISprite => AssetPriority::Critical,
                AssetType::SoundEffect => AssetPriority::High,
                AssetType::Music => AssetPriority::Normal,
                _ => AssetPriority::Normal,
            }
        }
    }
    
    fn cutscene_priority(&self, asset_type: AssetType) -> AssetPriority {
        match asset_type {
            AssetType::CutsceneSprite => AssetPriority::Critical,
            AssetType::DialogueSound => AssetPriority::Critical,
            AssetType::BackgroundMusic => AssetPriority::High,
            _ => AssetPriority::Background,
        }
    }
}

/// Asset loading queue with priority handling
pub struct PriorityAssetQueue {
    queues: [VecDeque<AssetLoadRequest>; 5], // One queue per priority level
    active_loads: HashMap<Handle<LoadedAsset>, AssetLoadRequest>,
    max_concurrent_loads: usize,
    bandwidth_limit_bytes: Option<usize>,
    current_bandwidth_usage: AtomicUsize,
}

impl PriorityAssetQueue {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            queues: Default::default(),
            active_loads: HashMap::new(),
            max_concurrent_loads: max_concurrent,
            bandwidth_limit_bytes: None,
            current_bandwidth_usage: AtomicUsize::new(0),
        }
    }
    
    pub fn enqueue(&mut self, request: AssetLoadRequest) {
        let priority = request.priority as usize;
        self.queues[priority].push_back(request);
    }
    
    pub fn process(&mut self, asset_server: &AssetServer) -> Vec<Handle<LoadedAsset>> {
        let mut loaded_handles = Vec::new();
        
        // Check completed loads
        self.active_loads.retain(|handle, request| {
            if asset_server.is_loaded(handle) {
                loaded_handles.push(handle.clone());
                self.current_bandwidth_usage.fetch_sub(
                    request.estimated_size_bytes,
                    Ordering::Relaxed
                );
                false
            } else {
                true
            }
        });
        
        // Start new loads based on priority
        let current_loads = self.active_loads.len();
        let available_slots = self.max_concurrent_loads.saturating_sub(current_loads);
        
        let mut loads_started = 0;
        for priority_queue in &mut self.queues {
            while loads_started < available_slots {
                if let Some(request) = priority_queue.pop_front() {
                    if self.can_load(&request) {
                        let handle = self.start_load(&request, asset_server);
                        self.active_loads.insert(handle, request);
                        loads_started += 1;
                    } else {
                        // Put it back if we can't load due to bandwidth
                        priority_queue.push_front(request);
                        break;
                    }
                } else {
                    break;
                }
            }
            
            if loads_started >= available_slots {
                break;
            }
        }
        
        loaded_handles
    }
    
    fn can_load(&self, request: &AssetLoadRequest) -> bool {
        if let Some(limit) = self.bandwidth_limit_bytes {
            let current = self.current_bandwidth_usage.load(Ordering::Relaxed);
            current + request.estimated_size_bytes <= limit
        } else {
            true
        }
    }
    
    fn start_load(
        &self,
        request: &AssetLoadRequest,
        asset_server: &AssetServer,
    ) -> Handle<LoadedAsset> {
        self.current_bandwidth_usage.fetch_add(
            request.estimated_size_bytes,
            Ordering::Relaxed
        );
        
        match &request.asset_type {
            AssetType::CreatureSprite => {
                asset_server.load(&request.path)
            }
            AssetType::TerrainTile => {
                asset_server.load(&request.path)
            }
            AssetType::ParticleTexture => {
                asset_server.load(&request.path)
            }
            _ => asset_server.load(&request.path),
        }
    }
}

#[derive(Clone)]
pub struct AssetLoadRequest {
    pub path: String,
    pub asset_type: AssetType,
    pub priority: AssetPriority,
    pub world_position: Option<Vec3>,
    pub estimated_size_bytes: usize,
    pub retry_count: u32,
}

/// Preloading strategy for different game phases
pub struct AssetPreloader {
    pub preload_radius: f32,
    pub prediction_time: f32,
}

impl AssetPreloader {
    pub fn get_preload_list(
        &self,
        current_position: Vec3,
        velocity: Vec3,
        game_state: &GameState,
    ) -> Vec<AssetLoadRequest> {
        let mut requests = Vec::new();
        
        // Predict future position
        let future_position = current_position + velocity * self.prediction_time;
        
        match game_state {
            GameState::Playing => {
                // Preload terrain in movement direction
                let preload_center = future_position;
                requests.extend(self.get_terrain_preloads(preload_center));
                
                // Preload creature sprites for nearby spawns
                requests.extend(self.get_creature_preloads(preload_center));
                
                // Preload common particle effects
                requests.extend(self.get_particle_preloads());
            }
            GameState::MainMenu => {
                // Preload assets for quick game start
                requests.extend(self.get_quick_start_assets());
            }
            _ => {}
        }
        
        requests
    }
    
    fn get_terrain_preloads(&self, center: Vec3) -> Vec<AssetLoadRequest> {
        let mut requests = Vec::new();
        let chunk_size = 32.0;
        let preload_chunks = (self.preload_radius / chunk_size).ceil() as i32;
        
        for x in -preload_chunks..=preload_chunks {
            for z in -preload_chunks..=preload_chunks {
                let chunk_pos = center + Vec3::new(
                    x as f32 * chunk_size,
                    0.0,
                    z as f32 * chunk_size,
                );
                
                // Different biomes have different tile assets
                let biome = get_biome_at_position(chunk_pos);
                let tile_path = format!("sprites/tiles/{}/atlas.png", biome.name());
                
                requests.push(AssetLoadRequest {
                    path: tile_path,
                    asset_type: AssetType::TerrainTile,
                    priority: AssetPriority::High,
                    world_position: Some(chunk_pos),
                    estimated_size_bytes: 512 * 1024, // 512KB estimate
                    retry_count: 0,
                });
            }
        }
        
        requests
    }
    
    fn get_creature_preloads(&self, center: Vec3) -> Vec<AssetLoadRequest> {
        vec![
            AssetLoadRequest {
                path: "sprites/creatures/herbivore/atlas.png".to_string(),
                asset_type: AssetType::CreatureSprite,
                priority: AssetPriority::Normal,
                world_position: Some(center),
                estimated_size_bytes: 1024 * 1024, // 1MB
                retry_count: 0,
            },
            AssetLoadRequest {
                path: "sprites/creatures/carnivore/atlas.png".to_string(),
                asset_type: AssetType::CreatureSprite,
                priority: AssetPriority::Normal,
                world_position: Some(center),
                estimated_size_bytes: 1024 * 1024,
                retry_count: 0,
            },
        ]
    }
    
    fn get_particle_preloads(&self) -> Vec<AssetLoadRequest> {
        vec![
            AssetLoadRequest {
                path: "sprites/particles/emotion_hearts.png".to_string(),
                asset_type: AssetType::ParticleTexture,
                priority: AssetPriority::Low,
                world_position: None,
                estimated_size_bytes: 64 * 1024, // 64KB
                retry_count: 0,
            },
            AssetLoadRequest {
                path: "sprites/particles/dust_cloud.png".to_string(),
                asset_type: AssetType::ParticleTexture,
                priority: AssetPriority::Low,
                world_position: None,
                estimated_size_bytes: 64 * 1024,
                retry_count: 0,
            },
        ]
    }
    
    fn get_quick_start_assets(&self) -> Vec<AssetLoadRequest> {
        vec![
            AssetLoadRequest {
                path: "sprites/tiles/grass/atlas.png".to_string(),
                asset_type: AssetType::TerrainTile,
                priority: AssetPriority::High,
                world_position: None,
                estimated_size_bytes: 512 * 1024,
                retry_count: 0,
            },
            AssetLoadRequest {
                path: "sprites/creatures/herbivore/atlas.png".to_string(),
                asset_type: AssetType::CreatureSprite,
                priority: AssetPriority::High,
                world_position: None,
                estimated_size_bytes: 1024 * 1024,
                retry_count: 0,
            },
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AssetType {
    CreatureSprite,
    TerrainTile,
    ResourceSprite,
    ParticleTexture,
    WeatherEffect,
    UISprite,
    SoundEffect,
    Music,
    AmbientSound,
    MenuBackground,
    ButtonSound,
    CreaturePreview,
    CutsceneSprite,
    DialogueSound,
    BackgroundMusic,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameState {
    MainMenu,
    Loading,
    Playing,
    Paused,
    Cutscene,
}
```

## AI State to Emotion Mapping

```rust
/// Maps creature AI states and needs to visual emotions
pub struct EmotionMapper {
    pub state_mappings: HashMap<AIState, EmotionRule>,
    pub need_thresholds: NeedEmotionThresholds,
    pub social_triggers: SocialEmotionTriggers,
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
    SocialBased { relationship: f32, emotion: EmotionType },
    EnvironmentBased { condition: EnvironmentCondition, emotion: EmotionType },
}

impl EmotionMapper {
    pub fn new() -> Self {
        let mut state_mappings = HashMap::new();
        
        // Map AI states to base emotions
        state_mappings.insert(AIState::Idle, EmotionRule {
            primary_emotion: EmotionType::Content,
            intensity_base: 0.5,
            modifiers: vec![
                EmotionModifier::NeedBased {
                    need: NeedType::Social,
                    threshold: 0.3,
                    emotion: EmotionType::Lonely,
                },
            ],
        });
        
        state_mappings.insert(AIState::Foraging, EmotionRule {
            primary_emotion: EmotionType::Curious,
            intensity_base: 0.7,
            modifiers: vec![
                EmotionModifier::NeedBased {
                    need: NeedType::Hunger,
                    threshold: 0.7,
                    emotion: EmotionType::Desperate,
                },
            ],
        });
        
        state_mappings.insert(AIState::Eating, EmotionRule {
            primary_emotion: EmotionType::Happy,
            intensity_base: 0.8,
            modifiers: vec![
                EmotionModifier::NeedBased {
                    need: NeedType::Hunger,
                    threshold: 0.9,
                    emotion: EmotionType::Relieved,
                },
            ],
        });
        
        state_mappings.insert(AIState::Drinking, EmotionRule {
            primary_emotion: EmotionType::Satisfied,
            intensity_base: 0.7,
            modifiers: vec![],
        });
        
        state_mappings.insert(AIState::Sleeping, EmotionRule {
            primary_emotion: EmotionType::Peaceful,
            intensity_base: 0.9,
            modifiers: vec![
                EmotionModifier::EnvironmentBased {
                    condition: EnvironmentCondition::Dangerous,
                    emotion: EmotionType::Anxious,
                },
            ],
        });
        
        state_mappings.insert(AIState::Fleeing, EmotionRule {
            primary_emotion: EmotionType::Frightened,
            intensity_base: 0.95,
            modifiers: vec![
                EmotionModifier::HealthBased {
                    threshold: 0.3,
                    emotion: EmotionType::Panicked,
                },
            ],
        });
        
        state_mappings.insert(AIState::Fighting, EmotionRule {
            primary_emotion: EmotionType::Angry,
            intensity_base: 0.9,
            modifiers: vec![
                EmotionModifier::HealthBased {
                    threshold: 0.5,
                    emotion: EmotionType::Desperate,
                },
            ],
        });
        
        state_mappings.insert(AIState::Socializing, EmotionRule {
            primary_emotion: EmotionType::Happy,
            intensity_base: 0.7,
            modifiers: vec![
                EmotionModifier::SocialBased {
                    relationship: 0.8,
                    emotion: EmotionType::Love,
                },
            ],
        });
        
        state_mappings.insert(AIState::Mating, EmotionRule {
            primary_emotion: EmotionType::Love,
            intensity_base: 0.85,
            modifiers: vec![],
        });
        
        state_mappings.insert(AIState::Exploring, EmotionRule {
            primary_emotion: EmotionType::Curious,
            intensity_base: 0.6,
            modifiers: vec![
                EmotionModifier::EnvironmentBased {
                    condition: EnvironmentCondition::NewTerritory,
                    emotion: EmotionType::Excited,
                },
            ],
        });
        
        Self {
            state_mappings,
            need_thresholds: NeedEmotionThresholds::default(),
            social_triggers: SocialEmotionTriggers::default(),
        }
    }
    
    pub fn calculate_emotion(
        &self,
        ai_state: &AIState,
        creature_state: &CreatureState,
        environment: &EnvironmentInfo,
    ) -> (EmotionType, f32) {
        // Start with base emotion for AI state
        let rule = self.state_mappings.get(ai_state)
            .unwrap_or(&EmotionRule {
                primary_emotion: EmotionType::Neutral,
                intensity_base: 0.5,
                modifiers: vec![],
            });
        
        let mut emotion = rule.primary_emotion;
        let mut intensity = rule.intensity_base;
        
        // Apply modifiers
        for modifier in &rule.modifiers {
            if let Some((mod_emotion, mod_intensity)) = self.evaluate_modifier(
                modifier,
                creature_state,
                environment,
            ) {
                // Higher intensity modifiers override
                if mod_intensity > intensity {
                    emotion = mod_emotion;
                    intensity = mod_intensity;
                }
            }
        }
        
        // Check critical needs that override everything
        if let Some((critical_emotion, critical_intensity)) = 
            self.check_critical_needs(creature_state) {
            if critical_intensity > intensity {
                emotion = critical_emotion;
                intensity = critical_intensity;
            }
        }
        
        (emotion, intensity)
    }
    
    fn evaluate_modifier(
        &self,
        modifier: &EmotionModifier,
        state: &CreatureState,
        environment: &EnvironmentInfo,
    ) -> Option<(EmotionType, f32)> {
        match modifier {
            EmotionModifier::NeedBased { need, threshold, emotion } => {
                let need_value = state.get_need_value(need);
                if need_value < *threshold {
                    Some((*emotion, 1.0 - need_value))
                } else {
                    None
                }
            }
            EmotionModifier::HealthBased { threshold, emotion } => {
                if state.health < *threshold {
                    Some((*emotion, 1.0 - state.health))
                } else {
                    None
                }
            }
            EmotionModifier::SocialBased { relationship, emotion } => {
                if state.highest_relationship > *relationship {
                    Some((*emotion, state.highest_relationship))
                } else {
                    None
                }
            }
            EmotionModifier::EnvironmentBased { condition, emotion } => {
                if environment.matches_condition(condition) {
                    Some((*emotion, 0.8))
                } else {
                    None
                }
            }
        }
    }
    
    fn check_critical_needs(&self, state: &CreatureState) -> Option<(EmotionType, f32)> {
        // Starvation
        if state.hunger < 0.1 {
            return Some((EmotionType::Desperate, 1.0));
        }
        
        // Dehydration
        if state.thirst < 0.1 {
            return Some((EmotionType::Desperate, 0.95));
        }
        
        // Exhaustion
        if state.energy < 0.05 {
            return Some((EmotionType::Exhausted, 0.9));
        }
        
        // Near death
        if state.health < 0.2 {
            return Some((EmotionType::Suffering, 0.95));
        }
        
        None
    }
}

#[derive(Default)]
pub struct NeedEmotionThresholds {
    pub hunger_happy: f32,      // Above this, happy when eating
    pub hunger_content: f32,    // Above this, content
    pub hunger_worried: f32,    // Below this, worried
    pub hunger_desperate: f32,  // Below this, desperate
    
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
            hunger_happy: 0.8,
            hunger_content: 0.5,
            hunger_worried: 0.3,
            hunger_desperate: 0.1,
            
            thirst_happy: 0.8,
            thirst_content: 0.5,
            thirst_worried: 0.3,
            thirst_desperate: 0.1,
            
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

#[derive(Default)]
pub struct SocialEmotionTriggers {
    pub friendship_threshold: f32,
    pub love_threshold: f32,
    pub rival_threshold: f32,
    pub fear_threshold: f32,
}

impl Default for SocialEmotionTriggers {
    fn default() -> Self {
        Self {
            friendship_threshold: 0.6,
            love_threshold: 0.8,
            rival_threshold: -0.5,
            fear_threshold: -0.8,
        }
    }
}

/// Emotion transitions based on events
pub struct EmotionEventHandler {
    pub event_emotions: HashMap<EventType, EmotionResponse>,
}

#[derive(Clone)]
pub struct EmotionResponse {
    pub emotion: EmotionType,
    pub intensity: f32,
    pub duration: f32,
    pub override_current: bool,
}

impl EmotionEventHandler {
    pub fn new() -> Self {
        let mut event_emotions = HashMap::new();
        
        // Success events
        event_emotions.insert(EventType::FoundFood, EmotionResponse {
            emotion: EmotionType::Excited,
            intensity: 0.8,
            duration: 2.0,
            override_current: true,
        });
        
        event_emotions.insert(EventType::FoundWater, EmotionResponse {
            emotion: EmotionType::Relieved,
            intensity: 0.7,
            duration: 2.0,
            override_current: true,
        });
        
        event_emotions.insert(EventType::MadeFirend, EmotionResponse {
            emotion: EmotionType::Happy,
            intensity: 0.8,
            duration: 5.0,
            override_current: true,
        });
        
        event_emotions.insert(EventType::HadBaby, EmotionResponse {
            emotion: EmotionType::Love,
            intensity: 0.95,
            duration: 10.0,
            override_current: true,
        });
        
        // Danger events
        event_emotions.insert(EventType::PredatorSighted, EmotionResponse {
            emotion: EmotionType::Frightened,
            intensity: 0.9,
            duration: 5.0,
            override_current: true,
        });
        
        event_emotions.insert(EventType::Attacked, EmotionResponse {
            emotion: EmotionType::Panicked,
            intensity: 1.0,
            duration: 3.0,
            override_current: true,
        });
        
        // Loss events
        event_emotions.insert(EventType::FriendDied, EmotionResponse {
            emotion: EmotionType::Sad,
            intensity: 0.9,
            duration: 20.0,
            override_current: false, // Don't override if in danger
        });
        
        event_emotions.insert(EventType::LostTerritory, EmotionResponse {
            emotion: EmotionType::Angry,
            intensity: 0.7,
            duration: 10.0,
            override_current: false,
        });
        
        Self { event_emotions }
    }
    
    pub fn handle_event(
        &self,
        event: &EventType,
        current_emotion: &EmotionType,
        current_intensity: f32,
    ) -> Option<(EmotionType, f32, f32)> {
        if let Some(response) = self.event_emotions.get(event) {
            // Check if we should override current emotion
            if response.override_current || current_intensity < response.intensity {
                Some((response.emotion, response.intensity, response.duration))
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// Visual emotion feedback rules
pub struct EmotionVisualizationRules {
    pub particle_triggers: HashMap<EmotionType, ParticleEffectType>,
    pub animation_overrides: HashMap<EmotionType, AnimationType>,
    pub facial_expressions: HashMap<EmotionType, FacialExpression>,
    pub body_language: HashMap<EmotionType, BodyLanguage>,
}

impl Default for EmotionVisualizationRules {
    fn default() -> Self {
        let mut particle_triggers = HashMap::new();
        particle_triggers.insert(EmotionType::Happy, ParticleEffectType::Hearts);
        particle_triggers.insert(EmotionType::Love, ParticleEffectType::BigHearts);
        particle_triggers.insert(EmotionType::Angry, ParticleEffectType::Steam);
        particle_triggers.insert(EmotionType::Sad, ParticleEffectType::Tears);
        particle_triggers.insert(EmotionType::Frightened, ParticleEffectType::SweatDrops);
        particle_triggers.insert(EmotionType::Tired, ParticleEffectType::Zzz);
        particle_triggers.insert(EmotionType::Confused, ParticleEffectType::QuestionMarks);
        particle_triggers.insert(EmotionType::Excited, ParticleEffectType::Sparkles);
        
        let mut animation_overrides = HashMap::new();
        animation_overrides.insert(EmotionType::Tired, AnimationType::SlowWalk);
        animation_overrides.insert(EmotionType::Excited, AnimationType::Bounce);
        animation_overrides.insert(EmotionType::Frightened, AnimationType::Cower);
        animation_overrides.insert(EmotionType::Angry, AnimationType::Stomp);
        
        Self {
            particle_triggers,
            animation_overrides,
            facial_expressions: Self::default_facial_expressions(),
            body_language: Self::default_body_language(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AIState {
    Idle,
    Foraging,
    Eating,
    Drinking,
    Sleeping,
    Fleeing,
    Fighting,
    Socializing,
    Mating,
    Exploring,
    Hunting,
    Building,
    Teaching,
    Learning,
    Playing,
    Mourning,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EventType {
    FoundFood,
    FoundWater,
    MadeFirend,
    HadBaby,
    PredatorSighted,
    Attacked,
    FriendDied,
    LostTerritory,
    WonFight,
    FoundMate,
    DiscoveredLocation,
    LearnedSkill,
}
```

This completes the implementation details for the cartoon graphics system, filling in all the identified gaps with specific formulas, algorithms, and integration points.