use bevy::prelude::*;
use creature_simulation::rendering::audio_system::*;
use std::collections::HashMap;

/// Test audio channel management and priority queuing
#[test]
fn test_audio_channel_priority() {
    let mut channels = AudioChannels::default();
    
    // Get SFX channel
    let sfx_channel = channels.channels.get_mut(&AudioChannelType::Sfx).unwrap();
    
    // Add sounds with different priorities
    for priority in [50, 150, 100, 200, 75] {
        sfx_channel.priority_queue.push(PrioritizedSound {
            sound: Handle::default(),
            priority,
            params: PlaybackParams::default(),
            source: None,
        });
    }
    
    // Verify they come out in priority order
    assert_eq!(sfx_channel.priority_queue.pop().unwrap().priority, 200);
    assert_eq!(sfx_channel.priority_queue.pop().unwrap().priority, 150);
    assert_eq!(sfx_channel.priority_queue.pop().unwrap().priority, 100);
    assert_eq!(sfx_channel.priority_queue.pop().unwrap().priority, 75);
    assert_eq!(sfx_channel.priority_queue.pop().unwrap().priority, 50);
}

/// Test spatial audio rolloff calculations
#[test]
fn test_spatial_audio_rolloff() {
    let config = SpatialAudioConfig {
        rolloff_model: RolloffModel::Linear,
        reference_distance: 50.0,
        max_distance: 500.0,
        doppler_factor: 0.0,
    };
    
    // Test linear rolloff at various distances
    let test_cases = vec![
        (25.0, 1.0),    // Closer than reference = full volume
        (50.0, 1.0),    // At reference distance = full volume
        (275.0, 0.5),   // Halfway between ref and max
        (500.0, 0.0),   // At max distance = no volume
        (600.0, 0.0),   // Beyond max = no volume
    ];
    
    for (distance, expected_attenuation) in test_cases {
        let attenuation = match config.rolloff_model {
            RolloffModel::Linear => {
                if distance <= config.reference_distance {
                    1.0
                } else if distance >= config.max_distance {
                    0.0
                } else {
                    1.0 - (distance - config.reference_distance) / 
                          (config.max_distance - config.reference_distance)
                }
            }
            _ => unreachable!(),
        };
        
        assert!((attenuation - expected_attenuation).abs() < 0.01,
                "Distance {} should have attenuation {}, got {}", 
                distance, expected_attenuation, attenuation);
    }
}

/// Test audio settings defaults
#[test]
fn test_audio_settings_defaults() {
    let settings = AudioSettings::default();
    
    assert_eq!(settings.master_volume, 1.0);
    assert!(settings.spatial_audio_enabled);
    assert_eq!(settings.max_simultaneous_sounds, 32);
    assert_eq!(settings.min_distance, 50.0);
    assert_eq!(settings.max_distance, 500.0);
    assert!(settings.use_audio_lod);
    
    // Check channel volumes
    assert_eq!(settings.channel_volumes.get(&AudioChannelType::Sfx), Some(&1.0));
    assert_eq!(settings.channel_volumes.get(&AudioChannelType::Ambient), Some(&0.8));
    assert_eq!(settings.channel_volumes.get(&AudioChannelType::Ui), Some(&1.0));
    assert_eq!(settings.channel_volumes.get(&AudioChannelType::Voice), Some(&1.0));
}

/// Test ducking state transitions
#[test]
fn test_ducking_state() {
    let mut ducking = DuckingState::default();
    
    assert_eq!(ducking.target_volume, 1.0);
    assert_eq!(ducking.current_volume, 1.0);
    
    // Simulate ducking
    ducking.target_volume = 0.5;
    
    // Simulate smooth transition
    let dt = 0.016; // ~60 FPS
    for _ in 0..10 {
        ducking.current_volume += (ducking.target_volume - ducking.current_volume) 
                                 * ducking.transition_speed * dt;
    }
    
    // Should be approaching target
    assert!(ducking.current_volume < 0.9);
    assert!(ducking.current_volume > 0.4);
}

/// Test animation audio cue system
#[test]
fn test_animation_audio_cues() {
    let mut cue = AnimationAudioCue {
        frame_cues: HashMap::new(),
        current_frame: 0,
        last_played_frame: None,
    };
    
    // Add cues for specific frames
    cue.frame_cues.insert(5, AnimationSound {
        sound_name: "footstep".to_string(),
        volume: 0.8,
        pitch_variance: 0.1,
        chance: 1.0,
    });
    
    cue.frame_cues.insert(15, AnimationSound {
        sound_name: "footstep".to_string(),
        volume: 0.8,
        pitch_variance: 0.1,
        chance: 0.5, // 50% chance
    });
    
    // Test frame detection
    cue.current_frame = 5;
    assert!(cue.frame_cues.contains_key(&cue.current_frame));
    assert_ne!(Some(cue.current_frame), cue.last_played_frame);
}

/// Test playback parameters
#[test]
fn test_playback_params() {
    let default_params = PlaybackParams::default();
    
    assert_eq!(default_params.volume, 1.0);
    assert_eq!(default_params.pitch, 1.0);
    assert!(!default_params.looping);
    assert_eq!(default_params.channel, AudioChannelType::Sfx);
    assert!(default_params.spatial);
    
    // Test custom params
    let custom_params = PlaybackParams {
        volume: 0.5,
        pitch: 1.2,
        looping: true,
        channel: AudioChannelType::Ambient,
        spatial: false,
    };
    
    assert_eq!(custom_params.volume, 0.5);
    assert_eq!(custom_params.pitch, 1.2);
    assert!(custom_params.looping);
    assert_eq!(custom_params.channel, AudioChannelType::Ambient);
    assert!(!custom_params.spatial);
}

/// Test audio channel limits
#[test]
fn test_channel_concurrent_limits() {
    let channels = AudioChannels::default();
    
    // Verify each channel has appropriate limits
    assert_eq!(channels.channels.get(&AudioChannelType::Sfx).unwrap().max_concurrent, 16);
    assert_eq!(channels.channels.get(&AudioChannelType::Ambient).unwrap().max_concurrent, 4);
    assert_eq!(channels.channels.get(&AudioChannelType::Voice).unwrap().max_concurrent, 8);
    assert_eq!(channels.channels.get(&AudioChannelType::Ui).unwrap().max_concurrent, 4);
}

/// Integration test for audio system initialization
#[test]
#[ignore = "Requires full Bevy app - run with cargo test -- --ignored"]
fn test_audio_system_integration() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(bevy::audio::AudioPlugin::default())
        .add_plugins(CartoonAudioPlugin);
    
    // Verify resources are initialized
    assert!(app.world.contains_resource::<AudioSettings>());
    assert!(app.world.contains_resource::<SoundLibrary>());
    assert!(app.world.contains_resource::<AudioChannels>());
    assert!(app.world.contains_resource::<SpatialAudioConfig>());
    
    // Run one update cycle
    app.update();
    
    // Audio system should be functional
    let channels = app.world.resource::<AudioChannels>();
    assert_eq!(channels.channels.len(), 4); // Should have 4 channel types
}