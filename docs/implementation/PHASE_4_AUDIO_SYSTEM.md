# Phase 4: Audio System Implementation

## Overview

This document provides the complete audio system implementation for Phase 4, including spatial audio, animation synchronization, dynamic sound effects, and performance optimization.

## Table of Contents
1. [Architecture Overview](#architecture-overview)
2. [Spatial Audio System](#spatial-audio-system)
3. [Animation Audio Synchronization](#animation-audio-synchronization)
4. [Dynamic Sound Effects](#dynamic-sound-effects)
5. [Environmental Audio](#environmental-audio)
6. [Audio Asset Management](#audio-asset-management)
7. [Performance Optimization](#performance-optimization)
8. [Integration Guidelines](#integration-guidelines)

## Architecture Overview

### Core Components

```rust
pub struct CartoonAudioPlugin;

impl Plugin for CartoonAudioPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<AudioSettings>()
            .init_resource::<SoundLibrary>()
            .init_resource::<AudioChannels>()
            .init_resource::<SpatialAudioConfig>()
            .add_systems(
                Update,
                (
                    update_audio_listeners,
                    process_spatial_audio,
                    sync_animation_audio,
                    update_environmental_audio,
                    manage_audio_channels,
                )
                    .chain()
                    .in_set(AudioSystemSet),
            )
            .add_systems(
                PostUpdate,
                cleanup_finished_sounds.in_set(AudioSystemSet),
            );
    }
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct AudioSystemSet;
```

### Audio Channel Management

```rust
#[derive(Resource)]
pub struct AudioChannels {
    pub master: AudioChannel,
    pub sfx: AudioChannel,
    pub ambient: AudioChannel,
    pub ui: AudioChannel,
    pub voice: AudioChannel,
}

pub struct AudioChannel {
    pub volume: f32,
    pub active_sounds: Vec<AudioInstance>,
    pub max_concurrent: usize,
    pub priority_queue: BinaryHeap<PrioritizedSound>,
    pub ducking_state: DuckingState,
}

#[derive(Clone)]
pub struct DuckingState {
    pub target_volume: f32,
    pub current_volume: f32,
    pub transition_speed: f32,
    pub priority_threshold: u8,
}

impl AudioChannel {
    pub fn play_sound(
        &mut self,
        sound: SoundHandle,
        priority: u8,
        params: PlaybackParams,
    ) -> Option<AudioInstance> {
        // Remove finished sounds
        self.active_sounds.retain(|s| !s.is_finished());
        
        // Check capacity
        if self.active_sounds.len() >= self.max_concurrent {
            // Find lowest priority sound
            if let Some(min_priority) = self.active_sounds.iter()
                .map(|s| s.priority)
                .min() {
                if priority <= min_priority {
                    return None; // Don't play if priority too low
                }
                
                // Stop lowest priority sound
                self.active_sounds.retain(|s| s.priority > min_priority);
            }
        }
        
        // Create and play instance
        let instance = AudioInstance {
            handle: sound,
            priority,
            volume: params.volume * self.current_volume,
            pitch: params.pitch,
            pan: params.pan,
            start_time: Instant::now(),
            duration: params.duration,
            looping: params.looping,
        };
        
        self.active_sounds.push(instance.clone());
        Some(instance)
    }
}
```

## Spatial Audio System

### 3D Audio Processing

```rust
#[derive(Component)]
pub struct AudioEmitter {
    pub sounds: Vec<PositionalSound>,
    pub attenuation_model: AttenuationModel,
    pub max_distance: f32,
    pub reference_distance: f32,
    pub cone: Option<AudioCone>,
    pub occlusion: OcclusionSettings,
}

#[derive(Clone)]
pub struct PositionalSound {
    pub sound_id: SoundId,
    pub base_volume: f32,
    pub pitch_variation: f32,
    pub trigger: SoundTrigger,
    pub cooldown: Timer,
    pub importance: f32, // For culling priority
}

#[derive(Clone)]
pub enum AttenuationModel {
    Linear,
    Inverse,
    Exponential,
    Custom(Box<dyn Fn(f32) -> f32 + Send + Sync>),
}

#[derive(Clone)]
pub struct AudioCone {
    pub inner_angle: f32,
    pub outer_angle: f32,
    pub outer_volume: f32,
}

#[derive(Clone)]
pub struct OcclusionSettings {
    pub enabled: bool,
    pub ray_count: u8,
    pub max_occlusion: f32,
    pub update_rate: f32,
}

pub fn process_spatial_audio(
    listener: Query<(&Transform, &AudioListener)>,
    emitters: Query<(&Transform, &AudioEmitter, &GlobalTransform)>,
    mut audio_channels: ResMut<AudioChannels>,
    sound_library: Res<SoundLibrary>,
    spatial_config: Res<SpatialAudioConfig>,
) {
    let (listener_transform, listener_settings) = match listener.get_single() {
        Ok(l) => l,
        Err(_) => return,
    };
    
    let listener_pos = listener_transform.translation;
    let listener_forward = listener_transform.forward();
    
    // Collect and sort emitters by distance
    let mut emitter_data: Vec<(f32, Entity, &Transform, &AudioEmitter)> = emitters
        .iter()
        .map(|(t, e, _)| {
            let distance = (t.translation - listener_pos).length();
            (distance, Entity::PLACEHOLDER, t, e)
        })
        .collect();
    
    emitter_data.sort_by_key(|&(dist, ..)| OrderedFloat(dist));
    
    // Process nearest emitters up to budget
    for (distance, _, emitter_transform, emitter) in emitter_data
        .iter()
        .take(spatial_config.max_simultaneous_3d_sounds) {
        
        if *distance > emitter.max_distance {
            continue;
        }
        
        // Calculate attenuation
        let attenuation = calculate_attenuation(
            *distance,
            emitter.reference_distance,
            emitter.max_distance,
            &emitter.attenuation_model,
        );
        
        // Calculate stereo panning
        let to_emitter = (emitter_transform.translation - listener_pos).normalize();
        let right = listener_transform.right();
        let pan = to_emitter.dot(right).clamp(-1.0, 1.0);
        
        // Calculate cone attenuation if applicable
        let cone_attenuation = if let Some(cone) = &emitter.cone {
            let emitter_forward = emitter_transform.forward();
            let angle_to_listener = emitter_forward.angle_between(-to_emitter);
            
            if angle_to_listener <= cone.inner_angle {
                1.0
            } else if angle_to_listener <= cone.outer_angle {
                let fraction = (angle_to_listener - cone.inner_angle) 
                    / (cone.outer_angle - cone.inner_angle);
                1.0 - fraction * (1.0 - cone.outer_volume)
            } else {
                cone.outer_volume
            }
        } else {
            1.0
        };
        
        // Calculate occlusion if enabled
        let occlusion = if emitter.occlusion.enabled {
            calculate_occlusion(
                emitter_transform.translation,
                listener_pos,
                emitter.occlusion.ray_count,
            )
        } else {
            0.0
        };
        
        let final_volume = attenuation 
            * cone_attenuation 
            * (1.0 - occlusion * emitter.occlusion.max_occlusion);
        
        // Play sounds with calculated parameters
        for sound in &emitter.sounds {
            if should_play_sound(sound) {
                let params = PlaybackParams {
                    volume: sound.base_volume * final_volume,
                    pitch: 1.0 + random_range(-sound.pitch_variation, sound.pitch_variation),
                    pan,
                    duration: None,
                    looping: false,
                };
                
                audio_channels.sfx.play_sound(
                    sound_library.get(sound.sound_id),
                    (sound.importance * 10.0) as u8,
                    params,
                );
            }
        }
    }
}

fn calculate_attenuation(
    distance: f32,
    reference: f32,
    max_distance: f32,
    model: &AttenuationModel,
) -> f32 {
    let clamped_distance = distance.clamp(reference, max_distance);
    
    match model {
        AttenuationModel::Linear => {
            1.0 - (clamped_distance - reference) / (max_distance - reference)
        }
        AttenuationModel::Inverse => {
            reference / (reference + (clamped_distance - reference))
        }
        AttenuationModel::Exponential => {
            (reference / clamped_distance).powf(2.0)
        }
        AttenuationModel::Custom(func) => func(clamped_distance),
    }
}
```

## Animation Audio Synchronization

### Frame-Accurate Audio Triggers

```rust
#[derive(Component)]
pub struct AnimationAudioSync {
    pub sound_map: HashMap<AnimationType, AnimationSoundSet>,
    pub active_sounds: Vec<ActiveAnimationSound>,
    pub last_frame: usize,
    pub surface_modifier: Option<SurfaceType>,
}

#[derive(Clone)]
pub struct AnimationSoundSet {
    pub frame_triggers: Vec<FrameTrigger>,
    pub continuous_sounds: Vec<ContinuousSound>,
    pub transition_sounds: HashMap<AnimationType, SoundId>,
}

#[derive(Clone)]
pub struct FrameTrigger {
    pub frame: usize,
    pub sound_id: SoundId,
    pub volume: f32,
    pub pitch_range: (f32, f32),
    pub surface_variants: HashMap<SurfaceType, SoundId>,
}

#[derive(Clone)]
pub struct ContinuousSound {
    pub sound_id: SoundId,
    pub fade_in: f32,
    pub fade_out: f32,
    pub volume_curve: AnimationCurve,
    pub pitch_curve: AnimationCurve,
}

pub fn sync_animation_audio(
    mut creatures: Query<(
        &CartoonSprite,
        &AnimationAudioSync,
        &Transform,
        Option<&OnSurface>,
    )>,
    mut audio_channels: ResMut<AudioChannels>,
    sound_library: Res<SoundLibrary>,
    time: Res<Time>,
) {
    for (sprite, mut audio_sync, transform, surface) in creatures.iter_mut() {
        let current_animation = &sprite.animation_state.current;
        let current_frame = sprite.current_frame;
        
        // Check for frame triggers
        if current_frame != audio_sync.last_frame {
            if let Some(sound_set) = audio_sync.sound_map.get(current_animation) {
                for trigger in &sound_set.frame_triggers {
                    if trigger.frame == current_frame {
                        // Select appropriate sound variant
                        let sound_id = if let Some(surface_type) = surface {
                            trigger.surface_variants
                                .get(&surface_type.0)
                                .unwrap_or(&trigger.sound_id)
                        } else {
                            &trigger.sound_id
                        };
                        
                        let pitch = random_range(trigger.pitch_range.0, trigger.pitch_range.1);
                        
                        audio_channels.sfx.play_sound(
                            sound_library.get(*sound_id),
                            5, // Medium priority
                            PlaybackParams {
                                volume: trigger.volume,
                                pitch,
                                pan: 0.0,
                                duration: None,
                                looping: false,
                            },
                        );
                    }
                }
            }
            
            audio_sync.last_frame = current_frame;
        }
        
        // Update continuous sounds
        if let Some(sound_set) = audio_sync.sound_map.get(current_animation) {
            for continuous in &sound_set.continuous_sounds {
                let progress = sprite.frame_progress;
                let volume = continuous.volume_curve.evaluate(progress);
                let pitch = continuous.pitch_curve.evaluate(progress);
                
                // Update or create continuous sound instance
                update_continuous_sound(
                    &mut audio_sync.active_sounds,
                    continuous.sound_id,
                    volume,
                    pitch,
                    continuous.fade_in,
                    continuous.fade_out,
                    time.delta_seconds(),
                );
            }
        }
    }
}

// Footstep audio example
pub fn create_walk_animation_sounds() -> AnimationSoundSet {
    AnimationSoundSet {
        frame_triggers: vec![
            FrameTrigger {
                frame: 1,
                sound_id: SoundId::FootstepLeft,
                volume: 0.5,
                pitch_range: (0.9, 1.1),
                surface_variants: HashMap::from([
                    (SurfaceType::Grass, SoundId::FootstepGrassLeft),
                    (SurfaceType::Stone, SoundId::FootstepStoneLeft),
                    (SurfaceType::Water, SoundId::FootstepWaterLeft),
                    (SurfaceType::Sand, SoundId::FootstepSandLeft),
                ]),
            },
            FrameTrigger {
                frame: 5,
                sound_id: SoundId::FootstepRight,
                volume: 0.5,
                pitch_range: (0.9, 1.1),
                surface_variants: HashMap::from([
                    (SurfaceType::Grass, SoundId::FootstepGrassRight),
                    (SurfaceType::Stone, SoundId::FootstepStoneRight),
                    (SurfaceType::Water, SoundId::FootstepWaterRight),
                    (SurfaceType::Sand, SoundId::FootstepSandRight),
                ]),
            },
        ],
        continuous_sounds: vec![],
        transition_sounds: HashMap::from([
            (AnimationType::Run, SoundId::WalkToRunTransition),
            (AnimationType::Idle, SoundId::StopMoving),
        ]),
    }
}
```

## Dynamic Sound Effects

### Emotion-Based Vocalizations

```rust
#[derive(Component)]
pub struct CreatureVoice {
    pub voice_type: VoiceType,
    pub pitch_base: f32,
    pub pitch_variance: f32,
    pub emotion_sounds: HashMap<EmotionType, VocalizationSet>,
    pub cooldown: Timer,
    pub vocalization_chance: f32,
}

#[derive(Clone)]
pub enum VoiceType {
    Cute,
    Deep,
    Squeaky,
    Melodic,
    Gruff,
}

#[derive(Clone)]
pub struct VocalizationSet {
    pub idle_sounds: Vec<SoundId>,
    pub active_sounds: Vec<SoundId>,
    pub reaction_sounds: Vec<SoundId>,
    pub volume_range: (f32, f32),
    pub frequency: f32, // How often to vocalize
}

pub fn update_creature_vocalizations(
    mut creatures: Query<(
        &CreatureVoice,
        &EmotionState,
        &Transform,
        Option<&InConversation>,
    )>,
    mut audio_channels: ResMut<AudioChannels>,
    sound_library: Res<SoundLibrary>,
    time: Res<Time>,
) {
    for (mut voice, emotion, transform, conversation) in creatures.iter_mut() {
        voice.cooldown.tick(time.delta());
        
        if !voice.cooldown.finished() {
            continue;
        }
        
        // Check if creature should vocalize
        let should_vocalize = random::<f32>() < voice.vocalization_chance
            || conversation.is_some()
            || emotion.intensity > 0.7;
        
        if !should_vocalize {
            continue;
        }
        
        if let Some(vocalization_set) = voice.emotion_sounds.get(&emotion.current) {
            // Select appropriate sound based on context
            let sounds = if conversation.is_some() {
                &vocalization_set.active_sounds
            } else if emotion.just_changed {
                &vocalization_set.reaction_sounds
            } else {
                &vocalization_set.idle_sounds
            };
            
            if let Some(sound_id) = sounds.choose(&mut thread_rng()) {
                let volume = random_range(
                    vocalization_set.volume_range.0,
                    vocalization_set.volume_range.1,
                );
                
                let pitch = voice.pitch_base 
                    + random_range(-voice.pitch_variance, voice.pitch_variance);
                
                audio_channels.voice.play_sound(
                    sound_library.get(*sound_id),
                    7, // High priority for voices
                    PlaybackParams {
                        volume,
                        pitch,
                        pan: 0.0,
                        duration: None,
                        looping: false,
                    },
                );
                
                // Reset cooldown based on frequency
                voice.cooldown = Timer::from_seconds(
                    1.0 / vocalization_set.frequency,
                    TimerMode::Once,
                );
            }
        }
    }
}

// Action feedback sounds
pub fn create_action_audio_effects() -> HashMap<ActionType, ActionAudioEffect> {
    HashMap::from([
        (ActionType::Eat, ActionAudioEffect {
            start_sound: Some(SoundId::EatStart),
            loop_sound: Some(SoundId::EatLoop),
            end_sound: Some(SoundId::EatEnd),
            particle_sounds: vec![SoundId::Chew1, SoundId::Chew2, SoundId::Chew3],
            volume: 0.6,
            pitch_variance: 0.15,
        }),
        (ActionType::Sleep, ActionAudioEffect {
            start_sound: Some(SoundId::Yawn),
            loop_sound: Some(SoundId::Snoring),
            end_sound: Some(SoundId::WakeUp),
            particle_sounds: vec![],
            volume: 0.3,
            pitch_variance: 0.1,
        }),
        (ActionType::Attack, ActionAudioEffect {
            start_sound: Some(SoundId::AttackSwing),
            loop_sound: None,
            end_sound: Some(SoundId::AttackHit),
            particle_sounds: vec![SoundId::Impact1, SoundId::Impact2],
            volume: 0.8,
            pitch_variance: 0.2,
        }),
    ])
}
```

## Environmental Audio

### Weather and Ambient Sounds

```rust
#[derive(Resource)]
pub struct EnvironmentalAudio {
    pub biome_ambience: HashMap<BiomeType, AmbienceSet>,
    pub weather_sounds: HashMap<WeatherType, WeatherAudioSet>,
    pub time_of_day_sounds: HashMap<TimeOfDay, SoundId>,
    pub active_ambience: Vec<ActiveAmbience>,
    pub transition_queue: VecDeque<AmbienceTransition>,
}

#[derive(Clone)]
pub struct AmbienceSet {
    pub base_loops: Vec<AmbienceLoop>,
    pub random_sounds: Vec<RandomSound>,
    pub volume_modifier: f32,
    pub filter_params: FilterParams,
}

#[derive(Clone)]
pub struct AmbienceLoop {
    pub sound_id: SoundId,
    pub volume: f32,
    pub crossfade_duration: f32,
    pub time_of_day_curve: Option<AnimationCurve>,
}

#[derive(Clone)]
pub struct RandomSound {
    pub sound_id: SoundId,
    pub volume_range: (f32, f32),
    pub interval_range: (f32, f32),
    pub spatial_range: f32, // Random position within this range
    pub height_range: (f32, f32),
}

#[derive(Clone)]
pub struct WeatherAudioSet {
    pub intensity_curve: AnimationCurve,
    pub base_sound: SoundId,
    pub detail_sounds: Vec<SoundId>,
    pub impact_sounds: Vec<SoundId>, // Rain drops, etc.
    pub indoor_filter: FilterParams,
}

pub fn update_environmental_audio(
    biome_map: Res<BiomeMap>,
    weather: Res<WeatherState>,
    time: Res<Time>,
    camera: Query<&Transform, With<Camera>>,
    mut env_audio: ResMut<EnvironmentalAudio>,
    mut audio_channels: ResMut<AudioChannels>,
    sound_library: Res<SoundLibrary>,
) {
    let camera_pos = camera.single().translation;
    let current_biome = biome_map.get_biome_at(camera_pos.xz());
    let time_of_day = calculate_time_of_day(time.elapsed_seconds());
    
    // Update biome ambience
    if let Some(ambience_set) = env_audio.biome_ambience.get(&current_biome) {
        // Crossfade base loops
        for loop_def in &ambience_set.base_loops {
            let target_volume = loop_def.volume 
                * ambience_set.volume_modifier
                * loop_def.time_of_day_curve
                    .as_ref()
                    .map(|c| c.evaluate(time_of_day.as_fraction()))
                    .unwrap_or(1.0);
            
            update_or_create_ambience_loop(
                &mut env_audio.active_ambience,
                loop_def.sound_id,
                target_volume,
                loop_def.crossfade_duration,
                time.delta_seconds(),
            );
        }
        
        // Trigger random sounds
        for random_sound in &ambience_set.random_sounds {
            if should_trigger_random_sound(random_sound, time.delta_seconds()) {
                let position = camera_pos + Vec3::new(
                    random_range(-random_sound.spatial_range, random_sound.spatial_range),
                    random_range(random_sound.height_range.0, random_sound.height_range.1),
                    random_range(-random_sound.spatial_range, random_sound.spatial_range),
                );
                
                // Spawn temporary audio emitter
                spawn_positional_sound(
                    position,
                    random_sound.sound_id,
                    random_range(random_sound.volume_range.0, random_sound.volume_range.1),
                );
            }
        }
    }
    
    // Update weather audio
    if let Some(weather_audio) = env_audio.weather_sounds.get(&weather.current_weather) {
        let intensity = weather.intensity;
        let weather_volume = weather_audio.intensity_curve.evaluate(intensity);
        
        // Base weather sound (rain, wind, etc.)
        audio_channels.ambient.play_sound(
            sound_library.get(weather_audio.base_sound),
            8,
            PlaybackParams {
                volume: weather_volume,
                pitch: 1.0,
                pan: 0.0,
                duration: None,
                looping: true,
            },
        );
        
        // Detail sounds based on intensity
        let detail_count = (intensity * weather_audio.detail_sounds.len() as f32) as usize;
        for i in 0..detail_count {
            if let Some(detail_sound) = weather_audio.detail_sounds.get(i) {
                audio_channels.ambient.play_sound(
                    sound_library.get(*detail_sound),
                    6,
                    PlaybackParams {
                        volume: weather_volume * 0.5,
                        pitch: random_range(0.9, 1.1),
                        pan: random_range(-0.5, 0.5),
                        duration: None,
                        looping: true,
                    },
                );
            }
        }
    }
}

// Thunder implementation for storms
pub fn create_thunder_system() -> ThunderAudioSystem {
    ThunderAudioSystem {
        min_interval: 5.0,
        max_interval: 30.0,
        distance_range: (100.0, 1000.0),
        pre_flash_delay: 0.1,
        sound_delay_per_meter: 0.003, // ~3ms per meter (speed of sound)
        rumble_sounds: vec![
            SoundId::ThunderClose,
            SoundId::ThunderMedium,
            SoundId::ThunderFar,
        ],
        crack_sounds: vec![
            SoundId::LightningCrack1,
            SoundId::LightningCrack2,
        ],
    }
}
```

## Audio Asset Management

### Sound Library and Loading

```rust
#[derive(Resource)]
pub struct SoundLibrary {
    sounds: HashMap<SoundId, Handle<AudioSource>>,
    loading_queue: VecDeque<(SoundId, String)>,
    variations: HashMap<SoundId, Vec<Handle<AudioSource>>>,
    metadata: HashMap<SoundId, SoundMetadata>,
}

#[derive(Clone)]
pub struct SoundMetadata {
    pub duration: f32,
    pub loop_points: Option<(f32, f32)>,
    pub base_volume: f32,
    pub category: SoundCategory,
    pub memory_size: usize,
    pub compression: CompressionType,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum SoundCategory {
    Footstep,
    Voice,
    Action,
    Ambient,
    Weather,
    UI,
    Music,
}

impl SoundLibrary {
    pub fn load_sounds(
        &mut self,
        asset_server: &AssetServer,
        sound_definitions: &SoundDefinitions,
    ) {
        for (sound_id, definition) in sound_definitions.iter() {
            // Load main sound
            let handle = asset_server.load(&definition.path);
            self.sounds.insert(*sound_id, handle);
            
            // Load variations if any
            if !definition.variations.is_empty() {
                let mut variation_handles = Vec::new();
                for variation_path in &definition.variations {
                    variation_handles.push(asset_server.load(variation_path));
                }
                self.variations.insert(*sound_id, variation_handles);
            }
            
            // Store metadata
            self.metadata.insert(*sound_id, SoundMetadata {
                duration: definition.duration,
                loop_points: definition.loop_points,
                base_volume: definition.base_volume,
                category: definition.category,
                memory_size: definition.estimated_size,
                compression: definition.compression,
            });
        }
    }
    
    pub fn get_with_variation(&self, sound_id: SoundId) -> Handle<AudioSource> {
        if let Some(variations) = self.variations.get(&sound_id) {
            if !variations.is_empty() {
                let index = thread_rng().gen_range(0..variations.len());
                return variations[index].clone();
            }
        }
        
        self.sounds.get(&sound_id)
            .cloned()
            .unwrap_or_else(|| self.get_fallback_sound())
    }
}

// Define all game sounds
pub fn create_sound_definitions() -> SoundDefinitions {
    let mut definitions = HashMap::new();
    
    // Footsteps
    definitions.insert(SoundId::FootstepGrassLeft, SoundDefinition {
        path: "audio/footsteps/grass_left.ogg".to_string(),
        variations: vec![
            "audio/footsteps/grass_left_1.ogg".to_string(),
            "audio/footsteps/grass_left_2.ogg".to_string(),
            "audio/footsteps/grass_left_3.ogg".to_string(),
        ],
        duration: 0.3,
        loop_points: None,
        base_volume: 0.5,
        category: SoundCategory::Footstep,
        estimated_size: 50_000,
        compression: CompressionType::Ogg,
    });
    
    // Creature voices
    definitions.insert(SoundId::CreatureHappy, SoundDefinition {
        path: "audio/voices/creature_happy.ogg".to_string(),
        variations: vec![
            "audio/voices/creature_happy_1.ogg".to_string(),
            "audio/voices/creature_happy_2.ogg".to_string(),
            "audio/voices/creature_happy_3.ogg".to_string(),
            "audio/voices/creature_happy_4.ogg".to_string(),
        ],
        duration: 0.5,
        loop_points: None,
        base_volume: 0.6,
        category: SoundCategory::Voice,
        estimated_size: 80_000,
        compression: CompressionType::Ogg,
    });
    
    // Weather
    definitions.insert(SoundId::RainLoop, SoundDefinition {
        path: "audio/weather/rain_loop.ogg".to_string(),
        variations: vec![],
        duration: 10.0,
        loop_points: Some((0.0, 10.0)),
        base_volume: 0.7,
        category: SoundCategory::Weather,
        estimated_size: 500_000,
        compression: CompressionType::Ogg,
    });
    
    definitions
}
```

## Performance Optimization

### Audio LOD System

```rust
pub struct AudioLODSystem {
    pub distance_thresholds: [f32; 4],
    pub quality_levels: [AudioLODLevel; 4],
}

#[derive(Clone)]
pub struct AudioLODLevel {
    pub max_variation_sounds: usize,
    pub spatial_resolution: f32,
    pub update_frequency: f32,
    pub filter_quality: FilterQuality,
    pub reverb_enabled: bool,
}

impl Default for AudioLODSystem {
    fn default() -> Self {
        Self {
            distance_thresholds: [25.0, 50.0, 100.0, 200.0],
            quality_levels: [
                AudioLODLevel {
                    max_variation_sounds: 5,
                    spatial_resolution: 0.1,
                    update_frequency: 60.0,
                    filter_quality: FilterQuality::High,
                    reverb_enabled: true,
                },
                AudioLODLevel {
                    max_variation_sounds: 3,
                    spatial_resolution: 0.5,
                    update_frequency: 30.0,
                    filter_quality: FilterQuality::Medium,
                    reverb_enabled: true,
                },
                AudioLODLevel {
                    max_variation_sounds: 1,
                    spatial_resolution: 1.0,
                    update_frequency: 15.0,
                    filter_quality: FilterQuality::Low,
                    reverb_enabled: false,
                },
                AudioLODLevel {
                    max_variation_sounds: 0,
                    spatial_resolution: 2.0,
                    update_frequency: 10.0,
                    filter_quality: FilterQuality::None,
                    reverb_enabled: false,
                },
            ],
        }
    }
}

// Audio culling and prioritization
pub fn cull_audio_sources(
    sources: &mut Vec<AudioSourceData>,
    listener_pos: Vec3,
    max_sources: usize,
    lod_system: &AudioLODSystem,
) -> Vec<AudioSourceData> {
    // Calculate priority scores
    for source in sources.iter_mut() {
        let distance = (source.position - listener_pos).length();
        let lod_level = lod_system.get_lod_level(distance);
        
        source.priority_score = calculate_priority_score(
            distance,
            source.importance,
            source.volume,
            source.category,
            lod_level,
        );
    }
    
    // Sort by priority
    sources.sort_by_key(|s| OrderedFloat(-s.priority_score));
    
    // Take top N sources
    sources.truncate(max_sources);
    sources.clone()
}

fn calculate_priority_score(
    distance: f32,
    importance: f32,
    volume: f32,
    category: SoundCategory,
    lod_level: usize,
) -> f32 {
    let distance_factor = 1.0 / (1.0 + distance * 0.01);
    let category_weight = match category {
        SoundCategory::Voice => 2.0,
        SoundCategory::Action => 1.5,
        SoundCategory::UI => 3.0,
        _ => 1.0,
    };
    
    importance * volume * distance_factor * category_weight * (5 - lod_level) as f32
}
```

## Integration Guidelines

### System Dependencies

```rust
// Audio system ordering
app.configure_sets(
    Update,
    (
        AudioSystemSet
            .after(AnimationSystemSet)
            .after(PhysicsSystemSet)
            .before(RenderSystemSet),
    ),
);

// Audio component bundles
#[derive(Bundle)]
pub struct CreatureAudioBundle {
    pub voice: CreatureVoice,
    pub animation_sync: AnimationAudioSync,
    pub emitter: AudioEmitter,
    pub audio_lod: AudioLOD,
}

#[derive(Bundle)]
pub struct EnvironmentalAudioBundle {
    pub transform: Transform,
    pub emitter: AudioEmitter,
    pub random_trigger: RandomSoundTrigger,
    pub weather_responsive: WeatherResponsiveAudio,
}

// Integration with existing systems
pub fn integrate_with_animation_system(
    mut commands: Commands,
    new_creatures: Query<Entity, Added<CartoonSprite>>,
    species_data: Res<SpeciesData>,
) {
    for entity in new_creatures.iter() {
        if let Some(species) = species_data.get_species(entity) {
            commands.entity(entity).insert(CreatureAudioBundle {
                voice: species.create_voice(),
                animation_sync: create_animation_audio_sync(&species),
                emitter: AudioEmitter {
                    sounds: vec![],
                    attenuation_model: AttenuationModel::Inverse,
                    max_distance: 50.0,
                    reference_distance: 1.0,
                    cone: None,
                    occlusion: OcclusionSettings::default(),
                },
                audio_lod: AudioLOD::default(),
            });
        }
    }
}

// Integration with particle effects
pub fn sync_particle_audio(
    particle_events: EventReader<ParticleSpawnEvent>,
    mut audio_channels: ResMut<AudioChannels>,
    sound_library: Res<SoundLibrary>,
) {
    for event in particle_events.iter() {
        if let Some(sound_id) = get_particle_sound(event.particle_type) {
            audio_channels.sfx.play_sound(
                sound_library.get(sound_id),
                4, // Low-medium priority
                PlaybackParams {
                    volume: 0.3,
                    pitch: random_range(0.8, 1.2),
                    pan: 0.0,
                    duration: None,
                    looping: false,
                },
            );
        }
    }
}
```

## Debug and Testing

```rust
#[derive(Resource)]
pub struct AudioDebugSettings {
    pub show_emitter_ranges: bool,
    pub show_active_sounds: bool,
    pub show_occlusion_rays: bool,
    pub log_performance: bool,
    pub mute_categories: HashSet<SoundCategory>,
}

pub fn render_audio_debug(
    mut gizmos: Gizmos,
    debug_settings: Res<AudioDebugSettings>,
    emitters: Query<(&Transform, &AudioEmitter)>,
    listener: Query<&Transform, With<AudioListener>>,
    audio_channels: Res<AudioChannels>,
) {
    if !debug_settings.show_emitter_ranges {
        return;
    }
    
    let listener_pos = listener.single().translation;
    
    for (transform, emitter) in emitters.iter() {
        // Draw emitter range
        gizmos.circle(
            transform.translation,
            Vec3::Y,
            emitter.max_distance,
            Color::rgba(0.0, 1.0, 0.0, 0.2),
        );
        
        // Draw reference distance
        gizmos.circle(
            transform.translation,
            Vec3::Y,
            emitter.reference_distance,
            Color::rgba(0.0, 1.0, 0.0, 0.5),
        );
        
        // Draw cone if present
        if let Some(cone) = &emitter.cone {
            draw_audio_cone(&mut gizmos, transform, cone);
        }
        
        // Draw line to listener
        let distance = (transform.translation - listener_pos).length();
        let color = if distance <= emitter.max_distance {
            Color::GREEN
        } else {
            Color::RED
        };
        
        gizmos.line(transform.translation, listener_pos, color);
    }
    
    if debug_settings.show_active_sounds {
        // Display active sound count per channel
        draw_audio_channel_info(&audio_channels);
    }
}
```

This completes the audio system documentation with detailed implementation guidelines, spatial audio processing, animation synchronization, dynamic effects, and performance optimization strategies.