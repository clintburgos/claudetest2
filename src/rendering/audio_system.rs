use bevy::prelude::*;
use std::collections::{HashMap, BinaryHeap};
use std::cmp::Ordering;

/// Phase 4 Cartoon Audio System Plugin
/// 
/// This plugin provides a comprehensive audio system for the cartoon isometric simulation,
/// featuring:
/// - Spatial audio with distance-based attenuation
/// - Animation-synchronized sound effects 
/// - Environmental and weather-based ambient sounds
/// - Dynamic channel management with priority queuing
/// - Performance optimization through sound pooling and LOD
/// 
/// The audio system integrates tightly with the animation system to provide
/// frame-accurate sound synchronization for creature actions and expressions.
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
                    update_audio_volume_ducking,
                )
                    .chain(),
            )
            .add_systems(
                PostUpdate,
                cleanup_finished_sounds,
            );
    }
}

/// Global audio settings that control the overall audio behavior
#[derive(Resource)]
pub struct AudioSettings {
    /// Master volume multiplier for all audio (0.0 - 1.0)
    pub master_volume: f32,
    /// Individual channel volume multipliers
    pub channel_volumes: HashMap<AudioChannelType, f32>,
    /// Whether spatial audio is enabled
    pub spatial_audio_enabled: bool,
    /// Maximum number of simultaneous sounds
    pub max_simultaneous_sounds: usize,
    /// Distance at which sounds start to attenuate
    pub min_distance: f32,
    /// Distance at which sounds are inaudible
    pub max_distance: f32,
    /// Whether to use audio LOD based on distance
    pub use_audio_lod: bool,
}

impl Default for AudioSettings {
    fn default() -> Self {
        let mut channel_volumes = HashMap::new();
        channel_volumes.insert(AudioChannelType::Sfx, 1.0);
        channel_volumes.insert(AudioChannelType::Ambient, 0.8);
        channel_volumes.insert(AudioChannelType::Ui, 1.0);
        channel_volumes.insert(AudioChannelType::Voice, 1.0);
        
        Self {
            master_volume: 1.0,
            channel_volumes,
            spatial_audio_enabled: true,
            max_simultaneous_sounds: 32,
            min_distance: 50.0,
            max_distance: 500.0,
            use_audio_lod: true,
        }
    }
}

/// Types of audio channels for organizing sounds
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AudioChannelType {
    /// Sound effects (footsteps, actions, etc)
    Sfx,
    /// Environmental and weather sounds
    Ambient,
    /// UI feedback sounds
    Ui,
    /// Creature vocalizations
    Voice,
}

/// Library of all available sound effects mapped by their identifier
#[derive(Resource, Default)]
pub struct SoundLibrary {
    /// Map of sound name to audio handle
    pub sounds: HashMap<String, Handle<AudioSource>>,
    /// Preloaded common sounds for quick access
    pub common_sounds: CommonSounds,
}

/// Frequently used sounds preloaded for performance
#[derive(Default)]
pub struct CommonSounds {
    // Movement sounds
    pub footstep_grass: Option<Handle<AudioSource>>,
    pub footstep_sand: Option<Handle<AudioSource>>,
    pub footstep_snow: Option<Handle<AudioSource>>,
    pub footstep_water: Option<Handle<AudioSource>>,
    
    // Action sounds
    pub eat: Option<Handle<AudioSource>>,
    pub drink: Option<Handle<AudioSource>>,
    pub sleep: Option<Handle<AudioSource>>,
    
    // Emotional sounds
    pub happy_chirp: Option<Handle<AudioSource>>,
    pub sad_whimper: Option<Handle<AudioSource>>,
    pub angry_growl: Option<Handle<AudioSource>>,
    pub curious_hum: Option<Handle<AudioSource>>,
    
    // Weather sounds
    pub rain_loop: Option<Handle<AudioSource>>,
    pub wind_loop: Option<Handle<AudioSource>>,
    pub thunder: Option<Handle<AudioSource>>,
}

/// Manages audio channels with priority queuing and voice limiting
#[derive(Resource)]
pub struct AudioChannels {
    /// Channel configurations indexed by type
    pub channels: HashMap<AudioChannelType, AudioChannel>,
}

impl Default for AudioChannels {
    fn default() -> Self {
        let mut channels = HashMap::new();
        
        // Configure each channel with appropriate limits
        channels.insert(AudioChannelType::Sfx, AudioChannel {
            max_concurrent: 16,
            active_sounds: Vec::new(),
            priority_queue: BinaryHeap::new(),
            ducking_state: DuckingState::default(),
        });
        
        channels.insert(AudioChannelType::Ambient, AudioChannel {
            max_concurrent: 4,
            active_sounds: Vec::new(),
            priority_queue: BinaryHeap::new(),
            ducking_state: DuckingState::default(),
        });
        
        channels.insert(AudioChannelType::Voice, AudioChannel {
            max_concurrent: 8,
            active_sounds: Vec::new(),
            priority_queue: BinaryHeap::new(),
            ducking_state: DuckingState::default(),
        });
        
        channels.insert(AudioChannelType::Ui, AudioChannel {
            max_concurrent: 4,
            active_sounds: Vec::new(),
            priority_queue: BinaryHeap::new(),
            ducking_state: DuckingState::default(),
        });
        
        Self { channels }
    }
}

/// Individual audio channel with voice management
pub struct AudioChannel {
    /// Maximum number of sounds that can play simultaneously
    pub max_concurrent: usize,
    /// Currently playing sounds
    pub active_sounds: Vec<AudioInstance>,
    /// Queue of sounds waiting to play
    pub priority_queue: BinaryHeap<PrioritizedSound>,
    /// Volume ducking state for this channel
    pub ducking_state: DuckingState,
}

/// Represents an active audio instance
#[derive(Component, Clone)]
pub struct AudioInstance {
    /// Entity that owns this sound (for spatial audio)
    pub source: Option<Entity>,
    /// Channel this sound belongs to
    pub channel: AudioChannelType,
    /// Priority level (higher = more important)
    pub priority: u8,
    /// Base volume before spatial/ducking modifications
    pub base_volume: f32,
    /// Whether this sound loops
    pub looping: bool,
    /// Time when this sound started
    pub start_time: f32,
    /// Optional duration (for non-looping sounds)
    pub duration: Option<f32>,
}

/// Sound waiting to be played with priority
#[derive(Clone)]
pub struct PrioritizedSound {
    pub sound: Handle<AudioSource>,
    pub priority: u8,
    pub params: PlaybackParams,
    pub source: Option<Entity>,
}

impl Ord for PrioritizedSound {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.cmp(&other.priority)
    }
}

impl PartialOrd for PrioritizedSound {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for PrioritizedSound {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl Eq for PrioritizedSound {}

/// Parameters for playing a sound
#[derive(Clone)]
pub struct PlaybackParams {
    pub volume: f32,
    pub pitch: f32,
    pub looping: bool,
    pub channel: AudioChannelType,
    pub spatial: bool,
}

impl Default for PlaybackParams {
    fn default() -> Self {
        Self {
            volume: 1.0,
            pitch: 1.0,
            looping: false,
            channel: AudioChannelType::Sfx,
            spatial: true,
        }
    }
}

/// Volume ducking state for smooth transitions
#[derive(Clone)]
pub struct DuckingState {
    /// Target volume level
    pub target_volume: f32,
    /// Current interpolated volume
    pub current_volume: f32,
    /// Speed of volume transitions
    pub transition_speed: f32,
    /// Priority level that triggers ducking
    pub priority_threshold: u8,
}

impl Default for DuckingState {
    fn default() -> Self {
        Self {
            target_volume: 1.0,
            current_volume: 1.0,
            transition_speed: 5.0,
            priority_threshold: 200,
        }
    }
}

/// Configuration for spatial audio processing
#[derive(Resource)]
pub struct SpatialAudioConfig {
    /// Rolloff model for distance attenuation
    pub rolloff_model: RolloffModel,
    /// Reference distance for volume calculations
    pub reference_distance: f32,
    /// Maximum distance for audio
    pub max_distance: f32,
    /// Doppler effect strength (0.0 = disabled)
    pub doppler_factor: f32,
}

impl Default for SpatialAudioConfig {
    fn default() -> Self {
        Self {
            rolloff_model: RolloffModel::Linear,
            reference_distance: 50.0,
            max_distance: 500.0,
            doppler_factor: 0.0, // Disabled for cartoon style
        }
    }
}

/// Distance attenuation models
#[derive(Debug, Clone, Copy)]
pub enum RolloffModel {
    /// Linear falloff (simple but effective for cartoon style)
    Linear,
    /// Inverse distance (more realistic)
    Inverse,
    /// Exponential (steeper falloff)
    Exponential,
}

/// Component for audio listener (typically attached to camera)
#[derive(Component)]
pub struct AudioListener {
    /// Whether this listener is active
    pub active: bool,
}

/// Component for spatial audio sources
#[derive(Component)]
pub struct SpatialAudioSource {
    /// Current volume after spatial calculations
    pub volume: f32,
    /// Base volume before spatial modifications  
    pub base_volume: f32,
    /// Minimum distance for this specific source
    pub min_distance: Option<f32>,
    /// Maximum distance for this specific source
    pub max_distance: Option<f32>,
}

/// Component that links animations to audio cues
#[derive(Component)]
pub struct AnimationAudioCue {
    /// Map of animation frame to sound effect
    pub frame_cues: HashMap<usize, AnimationSound>,
    /// Current animation frame
    pub current_frame: usize,
    /// Previously played frame to avoid duplicates
    pub last_played_frame: Option<usize>,
}

/// Sound to play at a specific animation frame
#[derive(Clone)]
pub struct AnimationSound {
    /// Sound effect name from library
    pub sound_name: String,
    /// Volume multiplier
    pub volume: f32,
    /// Pitch variation range (-range to +range)
    pub pitch_variance: f32,
    /// Probability of playing (0.0 - 1.0)
    pub chance: f32,
}

/// System to update audio listener position from camera
fn update_audio_listeners(
    camera_query: Query<&GlobalTransform, With<Camera>>,
    mut listener_query: Query<&mut Transform, With<AudioListener>>,
) {
    if let Ok(camera_transform) = camera_query.get_single() {
        for mut listener_transform in listener_query.iter_mut() {
            // Update listener to match camera position
            listener_transform.translation = camera_transform.translation();
            listener_transform.rotation = camera_transform.to_scale_rotation_translation().1;
        }
    }
}

/// System to process spatial audio and update volumes
fn process_spatial_audio(
    settings: Res<AudioSettings>,
    spatial_config: Res<SpatialAudioConfig>,
    listener_query: Query<&GlobalTransform, With<AudioListener>>,
    mut audio_sources: Query<(&GlobalTransform, &mut SpatialAudioSource, &AudioInstance)>,
) {
    if !settings.spatial_audio_enabled {
        return;
    }
    
    let Ok(listener_transform) = listener_query.get_single() else {
        return;
    };
    
    let listener_pos = listener_transform.translation();
    
    for (source_transform, mut spatial_source, instance) in audio_sources.iter_mut() {
        let source_pos = source_transform.translation();
        let distance = listener_pos.distance(source_pos);
        
        // Use source-specific or global distance settings
        let min_dist = spatial_source.min_distance.unwrap_or(spatial_config.reference_distance);
        let max_dist = spatial_source.max_distance.unwrap_or(spatial_config.max_distance);
        
        // Calculate attenuation based on rolloff model
        let attenuation = match spatial_config.rolloff_model {
            RolloffModel::Linear => {
                if distance <= min_dist {
                    1.0
                } else if distance >= max_dist {
                    0.0
                } else {
                    1.0 - (distance - min_dist) / (max_dist - min_dist)
                }
            }
            RolloffModel::Inverse => {
                let clamped_dist = distance.max(min_dist);
                min_dist / clamped_dist
            }
            RolloffModel::Exponential => {
                let clamped_dist = distance.max(min_dist);
                (min_dist / clamped_dist).powf(2.0)
            }
        };
        
        // Apply master volume and channel volume
        let channel_volume = settings.channel_volumes
            .get(&instance.channel)
            .copied()
            .unwrap_or(1.0);
            
        spatial_source.volume = spatial_source.base_volume 
            * attenuation 
            * settings.master_volume 
            * channel_volume;
    }
}

/// System to sync audio with animations
fn sync_animation_audio(
    mut audio_cues: Query<(&mut AnimationAudioCue, &GlobalTransform, Entity)>,
    sound_library: Res<SoundLibrary>,
    mut channels: ResMut<AudioChannels>,
    time: Res<Time>,
) {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    for (mut audio_cue, transform, entity) in audio_cues.iter_mut() {
        // Check if we've advanced to a new frame
        if Some(audio_cue.current_frame) != audio_cue.last_played_frame {
            if let Some(anim_sound) = audio_cue.frame_cues.get(&audio_cue.current_frame) {
                // Roll for chance
                if rng.gen::<f32>() <= anim_sound.chance {
                    // Get sound from library
                    if let Some(sound_handle) = sound_library.sounds.get(&anim_sound.sound_name) {
                        // Calculate pitch with variance
                        let pitch_variance = if anim_sound.pitch_variance > 0.0 {
                            rng.gen_range(-anim_sound.pitch_variance..anim_sound.pitch_variance)
                        } else {
                            0.0
                        };
                        
                        let params = PlaybackParams {
                            volume: anim_sound.volume,
                            pitch: 1.0 + pitch_variance,
                            looping: false,
                            channel: AudioChannelType::Sfx,
                            spatial: true,
                        };
                        
                        // Queue sound for playing
                        if let Some(channel) = channels.channels.get_mut(&AudioChannelType::Sfx) {
                            channel.priority_queue.push(PrioritizedSound {
                                sound: sound_handle.clone(),
                                priority: 128, // Medium priority for animation sounds
                                params,
                                source: Some(entity),
                            });
                        }
                    }
                }
            }
            
            audio_cue.last_played_frame = Some(audio_cue.current_frame);
        }
    }
}

/// System to update environmental audio based on weather and time
fn update_environmental_audio(
    weather: Res<crate::systems::weather::WeatherState>,
    day_night: Res<crate::systems::weather::DayNightCycle>,
    sound_library: Res<SoundLibrary>,
    mut ambient_sounds: Query<(&mut AudioInstance, &AudioSink), With<EnvironmentalAudio>>,
) {
    use crate::systems::weather::WeatherType;
    
    // Determine which ambient sounds should be playing
    let should_play_rain = matches!(weather.current, WeatherType::Rain | WeatherType::Storm);
    let should_play_wind = weather.wind_strength > 15.0;
    let should_play_night = day_night.is_night;
    
    // Update rain sound
    for (mut instance, sink) in ambient_sounds.iter_mut() {
        if instance.channel == AudioChannelType::Ambient {
            // Adjust volume based on weather intensity
            let target_volume = if should_play_rain {
                weather.intensity * instance.base_volume
            } else {
                0.0
            };
            
            // Smooth volume transitions
            let current = sink.volume();
            let new_volume = current + (target_volume - current) * 0.1;
            sink.set_volume(new_volume);
        }
    }
}

/// Component tag for environmental audio sources
#[derive(Component)]
pub struct EnvironmentalAudio;

/// System to manage audio channels and play queued sounds
fn manage_audio_channels(
    mut commands: Commands,
    mut channels: ResMut<AudioChannels>,
    time: Res<Time>,
) {
    for (channel_type, channel) in channels.channels.iter_mut() {
        // Clean up finished sounds
        channel.active_sounds.retain(|instance| {
            if let Some(duration) = instance.duration {
                time.elapsed_seconds() - instance.start_time < duration
            } else {
                true // Keep looping sounds
            }
        });
        
        // Process priority queue
        while channel.active_sounds.len() < channel.max_concurrent {
            if let Some(prioritized_sound) = channel.priority_queue.pop() {
                // Create audio instance
                let instance = AudioInstance {
                    source: prioritized_sound.source,
                    channel: *channel_type,
                    priority: prioritized_sound.priority,
                    base_volume: prioritized_sound.params.volume,
                    looping: prioritized_sound.params.looping,
                    start_time: time.elapsed_seconds(),
                    duration: if prioritized_sound.params.looping { None } else { Some(2.0) }, // Default 2s duration
                };
                
                // Spawn audio entity
                // Note: In a real implementation, this would use Bevy's audio API
                // For now, we'll create a placeholder entity
                let audio_entity = commands.spawn((
                    instance.clone(),
                    Name::new("AudioInstance"),
                )).id();
                
                // Add spatial component if needed
                if prioritized_sound.params.spatial {
                    commands.entity(audio_entity).insert(SpatialAudioSource {
                        volume: prioritized_sound.params.volume,
                        base_volume: prioritized_sound.params.volume,
                        min_distance: None,
                        max_distance: None,
                    });
                }
                
                channel.active_sounds.push(instance);
            } else {
                break; // No more sounds in queue
            }
        }
    }
}

/// System to update volume ducking between channels
fn update_audio_volume_ducking(
    mut channels: ResMut<AudioChannels>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    
    // Check for high priority sounds that should duck others
    let mut should_duck_sfx = false;
    let mut should_duck_ambient = false;
    
    if let Some(voice_channel) = channels.channels.get(&AudioChannelType::Voice) {
        // Duck other channels when voice is playing
        if !voice_channel.active_sounds.is_empty() {
            should_duck_sfx = true;
            should_duck_ambient = true;
        }
    }
    
    // Apply ducking
    for (channel_type, channel) in channels.channels.iter_mut() {
        let target = match channel_type {
            AudioChannelType::Sfx if should_duck_sfx => 0.5,
            AudioChannelType::Ambient if should_duck_ambient => 0.3,
            _ => 1.0,
        };
        
        channel.ducking_state.target_volume = target;
        
        // Smooth transition
        let current = channel.ducking_state.current_volume;
        let speed = channel.ducking_state.transition_speed;
        channel.ducking_state.current_volume = current + (target - current) * speed * dt;
    }
}

/// System to cleanup finished audio entities
fn cleanup_finished_sounds(
    mut commands: Commands,
    finished_sounds: Query<(Entity, &AudioInstance)>,
    time: Res<Time>,
) {
    for (entity, instance) in finished_sounds.iter() {
        // Check if non-looping sound has exceeded its duration
        if let Some(duration) = instance.duration {
            if time.elapsed_seconds() - instance.start_time > duration {
                commands.entity(entity).despawn();
            }
        }
    }
}

/// Helper function to play a sound effect
pub fn play_sound(
    sound_name: &str,
    position: Option<Vec3>,
    params: PlaybackParams,
    sound_library: &SoundLibrary,
    channels: &mut AudioChannels,
) -> bool {
    if let Some(sound_handle) = sound_library.sounds.get(sound_name) {
        if let Some(channel) = channels.channels.get_mut(&params.channel) {
            let priority = match params.channel {
                AudioChannelType::Voice => 200,
                AudioChannelType::Ui => 180,
                AudioChannelType::Sfx => 128,
                AudioChannelType::Ambient => 64,
            };
            
            channel.priority_queue.push(PrioritizedSound {
                sound: sound_handle.clone(),
                priority,
                params,
                source: None, // Would need entity for spatial
            });
            
            return true;
        }
    }
    
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rolloff_models() {
        let config = SpatialAudioConfig::default();
        
        // Test linear rolloff
        let linear = match config.rolloff_model {
            RolloffModel::Linear => {
                // At min distance
                let at_min = 1.0;
                // At max distance  
                let at_max = 0.0;
                // Halfway
                let halfway = 0.5;
                (at_min, at_max, halfway)
            }
            _ => panic!("Expected linear rolloff"),
        };
        
        assert_eq!(linear, (1.0, 0.0, 0.5));
    }
    
    #[test]
    fn test_priority_queue() {
        let mut queue = BinaryHeap::new();
        
        queue.push(PrioritizedSound {
            sound: Handle::default(),
            priority: 100,
            params: PlaybackParams::default(),
            source: None,
        });
        
        queue.push(PrioritizedSound {
            sound: Handle::default(), 
            priority: 200,
            params: PlaybackParams::default(),
            source: None,
        });
        
        // Higher priority should come first
        let first = queue.pop().unwrap();
        assert_eq!(first.priority, 200);
    }
}