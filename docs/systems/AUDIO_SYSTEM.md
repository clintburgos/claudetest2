# Audio System Design

## Overview

The audio system provides immersive soundscapes, creature vocalizations, and UI feedback. It uses 3D spatial audio to enhance the isometric view and provides clear audio cues for creature behaviors and world events.

## Core Components

### Audio Categories

1. **Creature Vocalizations**
   - Happy chirps, sad whimpers, hungry growls
   - Mating calls, warning cries, social chatter
   - Species-specific voice synthesis
   - Emotional modulation based on mood

2. **Ambient Soundscapes**
   - Biome-specific ambience (forest rustling, desert winds)
   - Weather effects (rain, thunder, wind)
   - Day/night transitions (morning birds, evening crickets)
   - Seasonal variations

3. **Action Sounds**
   - Footsteps (varied by terrain and creature size)
   - Eating, drinking, swimming
   - Combat impacts, fleeing sounds
   - Tool use, resource gathering

4. **UI Feedback**
   - Button clicks, hover effects
   - Notification chimes
   - Time control feedback
   - Selection confirmation

### Technical Architecture

```rust
// Core audio components
pub struct AudioSource {
    pub position: Vec3,
    pub volume: f32,
    pub pitch: f32,
    pub category: AudioCategory,
    pub loop_behavior: LoopBehavior,
    pub falloff: AudioFalloff,
}

pub enum AudioCategory {
    Vocalization,
    Ambient,
    Action,
    UI,
    Music,
}

pub struct AudioListener {
    pub position: Vec3,
    pub forward: Vec3,
    pub up: Vec3,
}

// Spatial audio system
pub struct SpatialAudioSystem {
    max_audible_distance: f32,
    doppler_factor: f32,
    rolloff_factor: f32,
}

// Voice synthesis for creatures
pub struct CreatureVoice {
    base_pitch: f32,
    pitch_variance: f32,
    timbre: VoiceTimbre,
    emotional_modulation: f32,
}

pub enum VoiceTimbre {
    Chirpy,
    Growly,
    Melodic,
    Squeaky,
    Deep,
}
```

### 3D Spatial Audio

```rust
// Calculate audio parameters based on listener distance
fn calculate_spatial_audio(
    source: &AudioSource,
    listener: &AudioListener,
) -> SpatialAudioParams {
    let direction = source.position - listener.position;
    let distance = direction.length();
    
    // Volume falloff
    let volume = match source.falloff {
        AudioFalloff::Linear => {
            1.0 - (distance / source.max_distance).clamp(0.0, 1.0)
        }
        AudioFalloff::Exponential => {
            (1.0 / (1.0 + distance * self.rolloff_factor)).clamp(0.0, 1.0)
        }
        AudioFalloff::None => 1.0,
    };
    
    // Stereo panning for 2D output
    let right_ear = listener.position + listener.right() * EAR_SEPARATION;
    let left_ear = listener.position - listener.right() * EAR_SEPARATION;
    let right_distance = (source.position - right_ear).length();
    let left_distance = (source.position - left_ear).length();
    let pan = (right_distance - left_distance).clamp(-1.0, 1.0);
    
    // Doppler effect
    let relative_velocity = source.velocity - listener.velocity;
    let speed_of_sound = 343.0;
    let doppler = 1.0 + (relative_velocity.dot(direction.normalize()) / speed_of_sound);
    
    SpatialAudioParams {
        volume: volume * source.volume,
        pan,
        pitch: source.pitch * doppler,
    }
}
```

### Creature Voice Synthesis

```rust
// Generate creature vocalizations
impl CreatureVoiceSystem {
    fn generate_vocalization(
        &self,
        creature: &Creature,
        vocalization_type: VocalizationType,
    ) -> AudioBuffer {
        let voice = &creature.voice;
        let emotion = &creature.emotional_state;
        
        // Base frequency based on creature size and species
        let base_freq = match creature.size {
            CreatureSize::Tiny => 800.0 + voice.base_pitch * 400.0,
            CreatureSize::Small => 600.0 + voice.base_pitch * 300.0,
            CreatureSize::Medium => 400.0 + voice.base_pitch * 200.0,
            CreatureSize::Large => 200.0 + voice.base_pitch * 100.0,
        };
        
        // Emotional modulation
        let freq_modulation = match emotion.dominant_emotion() {
            Emotion::Happy => 1.2,
            Emotion::Sad => 0.8,
            Emotion::Angry => 0.9,
            Emotion::Scared => 1.3,
            Emotion::Curious => 1.1,
            _ => 1.0,
        };
        
        // Generate waveform
        match vocalization_type {
            VocalizationType::Chirp => {
                self.generate_chirp(base_freq * freq_modulation, 0.2)
            }
            VocalizationType::Growl => {
                self.generate_growl(base_freq * freq_modulation * 0.5, 0.5)
            }
            VocalizationType::Call => {
                self.generate_call(base_freq * freq_modulation, 1.0)
            }
        }
    }
}
```

### Ambient Soundscape System

```rust
pub struct AmbienceSystem {
    active_layers: Vec<AmbienceLayer>,
    crossfade_duration: f32,
}

pub struct AmbienceLayer {
    biome: BiomeType,
    time_of_day: TimeOfDay,
    weather: WeatherType,
    season: Season,
    volume: f32,
    audio_source: AudioSource,
}

impl AmbienceSystem {
    fn update_ambience(&mut self, world_state: &WorldState, delta_time: f32) {
        let target_layers = self.calculate_target_layers(world_state);
        
        // Crossfade between ambience layers
        for layer in &mut self.active_layers {
            if !target_layers.contains(&layer.id) {
                layer.volume -= delta_time / self.crossfade_duration;
            }
        }
        
        // Add new layers
        for target in target_layers {
            if !self.has_layer(target) {
                self.add_layer(target, 0.0);
            }
        }
        
        // Update volumes
        for layer in &mut self.active_layers {
            if target_layers.contains(&layer.id) {
                layer.volume += delta_time / self.crossfade_duration;
            }
            layer.volume = layer.volume.clamp(0.0, 1.0);
        }
    }
}
```

### Audio Priority System

```rust
pub struct AudioPrioritySystem {
    max_simultaneous_sounds: usize,
    priority_buckets: Vec<PriorityBucket>,
}

pub struct PriorityBucket {
    category: AudioCategory,
    max_sounds: usize,
    priority_weight: f32,
}

impl AudioPrioritySystem {
    fn get_active_sounds(&self, all_sounds: &[AudioSource]) -> Vec<&AudioSource> {
        let mut prioritized = all_sounds.to_vec();
        
        // Sort by priority score
        prioritized.sort_by(|a, b| {
            self.calculate_priority(b).partial_cmp(&self.calculate_priority(a))
                .unwrap_or(Ordering::Equal)
        });
        
        // Respect category limits
        let mut category_counts = HashMap::new();
        let mut selected = Vec::new();
        
        for sound in prioritized {
            let count = category_counts.entry(sound.category).or_insert(0);
            let bucket = self.get_bucket(sound.category);
            
            if *count < bucket.max_sounds && selected.len() < self.max_simultaneous_sounds {
                selected.push(sound);
                *count += 1;
            }
        }
        
        selected
    }
    
    fn calculate_priority(&self, source: &AudioSource) -> f32 {
        let distance_factor = 1.0 / (1.0 + source.distance_to_listener);
        let category_weight = self.get_bucket(source.category).priority_weight;
        let volume_factor = source.volume;
        
        distance_factor * category_weight * volume_factor
    }
}
```

### Dynamic Music System

```rust
pub struct DynamicMusicSystem {
    layers: Vec<MusicLayer>,
    current_intensity: f32,
    transition_speed: f32,
}

pub struct MusicLayer {
    intensity_range: (f32, f32),
    track: AudioTrack,
    volume: f32,
    fade_curve: FadeCurve,
}

impl DynamicMusicSystem {
    fn update_music(&mut self, game_state: &GameState, delta_time: f32) {
        // Calculate music intensity based on game events
        let target_intensity = self.calculate_intensity(game_state);
        
        // Smooth transition
        self.current_intensity = lerp(
            self.current_intensity,
            target_intensity,
            self.transition_speed * delta_time
        );
        
        // Update layer volumes
        for layer in &mut self.layers {
            let in_range = self.current_intensity >= layer.intensity_range.0
                && self.current_intensity <= layer.intensity_range.1;
            
            let target_volume = if in_range { 1.0 } else { 0.0 };
            layer.volume = lerp(layer.volume, target_volume, delta_time * 2.0);
        }
    }
    
    fn calculate_intensity(&self, game_state: &GameState) -> f32 {
        let mut intensity = 0.3; // Base intensity
        
        // Increase for dramatic events
        if game_state.active_combat_count > 0 {
            intensity += 0.3;
        }
        if game_state.creatures_in_danger > 5 {
            intensity += 0.2;
        }
        if game_state.major_event_active {
            intensity += 0.4;
        }
        
        intensity.clamp(0.0, 1.0)
    }
}
```

### Performance Optimizations

```rust
pub struct AudioLODSystem {
    detail_levels: Vec<AudioLOD>,
}

pub struct AudioLOD {
    distance: f32,
    max_sounds_per_creature: usize,
    voice_quality: VoiceQuality,
    update_rate: f32,
}

impl AudioLODSystem {
    fn get_lod_for_distance(&self, distance: f32) -> &AudioLOD {
        self.detail_levels.iter()
            .find(|lod| distance <= lod.distance)
            .unwrap_or(self.detail_levels.last().unwrap())
    }
    
    fn should_play_sound(&self, source: &AudioSource, lod: &AudioLOD) -> bool {
        match source.category {
            AudioCategory::UI => true, // Always play UI sounds
            AudioCategory::Vocalization => {
                source.priority > 0.5 || lod.distance < 50.0
            }
            AudioCategory::Action => lod.distance < 100.0,
            _ => true,
        }
    }
}
```

### Audio Settings & Accessibility

```rust
pub struct AudioSettings {
    pub master_volume: f32,
    pub category_volumes: HashMap<AudioCategory, f32>,
    pub enable_3d_audio: bool,
    pub subtitle_settings: SubtitleSettings,
    pub audio_visualization: bool,
    pub reduce_audio_complexity: bool,
}

pub struct SubtitleSettings {
    pub enabled: bool,
    pub show_speaker: bool,
    pub show_sound_effects: bool,
    pub background_opacity: f32,
    pub text_size: f32,
}

pub struct AudioAccessibility {
    // Visual indicators for audio events
    pub visual_sound_indicators: bool,
    pub sound_radar: bool,
    pub vibration_feedback: bool,
    
    // Audio descriptions
    pub describe_creature_actions: bool,
    pub describe_environment: bool,
}
```

### Integration with Game Systems

```rust
// Audio event triggers
pub enum AudioEvent {
    CreatureVocalization {
        creature_id: EntityId,
        vocalization_type: VocalizationType,
        emotion: Emotion,
    },
    EnvironmentSound {
        position: Vec3,
        sound_type: EnvironmentSoundType,
        volume: f32,
    },
    UIFeedback {
        feedback_type: UIFeedbackType,
    },
    MusicTrigger {
        trigger_type: MusicTriggerType,
        intensity: f32,
    },
}

// Integration with existing systems
impl AudioIntegration {
    fn on_creature_emotion_change(&mut self, event: &EmotionChangeEvent) {
        if event.intensity > 0.7 {
            self.queue_audio_event(AudioEvent::CreatureVocalization {
                creature_id: event.creature_id,
                vocalization_type: VocalizationType::from_emotion(event.new_emotion),
                emotion: event.new_emotion,
            });
        }
    }
    
    fn on_creature_action(&mut self, event: &CreatureActionEvent) {
        match event.action {
            CreatureAction::Eat => {
                self.play_eating_sound(event.creature_id, event.position);
            }
            CreatureAction::Attack => {
                self.play_combat_sound(event.position, CombatSoundType::Impact);
            }
            _ => {}
        }
    }
}
```

## Memory & Performance Budgets

- **Audio Memory Budget**: 128MB
  - Loaded samples: 64MB
  - Streaming buffers: 32MB
  - Voice synthesis: 16MB
  - Effect processing: 16MB

- **CPU Budget**: 5% of frame time
  - Spatial calculations: 2%
  - Voice synthesis: 1.5%
  - Effect processing: 1%
  - Streaming: 0.5%

- **Simultaneous Sounds**:
  - Near field (< 50m): 32 sounds
  - Mid field (50-200m): 16 sounds
  - Far field (> 200m): 8 sounds
  - UI/Music: Always active

## Implementation Priority

1. **Phase 1**: Core spatial audio and basic creature vocalizations
2. **Phase 2**: Ambient soundscapes and weather integration
3. **Phase 3**: Dynamic music system
4. **Phase 4**: Advanced voice synthesis and emotional modulation
5. **Phase 5**: Accessibility features and audio visualization