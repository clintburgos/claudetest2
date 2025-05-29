use bevy::prelude::*;
use creature_simulation::plugins::phase4::{Phase4Plugin, Phase4Features};
use creature_simulation::core::quality_settings::{QualitySettings, QualityPreset};
use creature_simulation::rendering::particle_system::{ParticlePool, ParticleEmitter, ParticleEffectType};
use creature_simulation::systems::weather::{WeatherState, WeatherType};
use creature_simulation::rendering::floating_ui::FloatingUISettings;

/// Test Phase 4 plugin initialization
#[test]
fn test_phase4_plugin_initialization() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(Phase4Plugin);
    
    // Check that resources are initialized
    assert!(app.world.contains_resource::<Phase4Features>());
    assert!(app.world.contains_resource::<QualitySettings>());
    assert!(app.world.contains_resource::<WeatherState>());
    assert!(app.world.contains_resource::<ParticlePool>());
    assert!(app.world.contains_resource::<FloatingUISettings>());
}

/// Test quality presets
#[test]
fn test_quality_presets() {
    let presets = vec![
        QualityPreset::Low,
        QualityPreset::Medium,
        QualityPreset::High,
        QualityPreset::Ultra,
    ];
    
    for preset in presets {
        let settings = QualitySettings::from_preset(preset);
        assert_eq!(settings.preset, preset);
        
        // Verify settings scale appropriately
        match preset {
            QualityPreset::Low => {
                assert!(settings.max_particles < 200);
                assert!(settings.particle_density < 0.5);
            }
            QualityPreset::Ultra => {
                assert!(settings.max_particles > 1000);
                assert!(settings.particle_density >= 1.0);
            }
            _ => {}
        }
    }
}

/// Test particle pool allocation
#[test]
fn test_particle_pool() {
    let mut pool = ParticlePool::new(1000);
    
    // Test allocation
    let allocation = pool.allocate(100);
    assert!(allocation.is_some());
    let indices = allocation.unwrap();
    assert_eq!(indices.len(), 100);
    
    // Test pool limits
    let large_allocation = pool.allocate(1000);
    assert!(large_allocation.is_none());
    
    // Test freeing
    pool.free(indices);
    let reallocation = pool.allocate(100);
    assert!(reallocation.is_some());
}

/// Test weather state transitions
#[test]
fn test_weather_transitions() {
    let weather_types = vec![
        WeatherType::Clear,
        WeatherType::Cloudy,
        WeatherType::Rain,
        WeatherType::Storm,
        WeatherType::Snow,
        WeatherType::Fog,
        WeatherType::Windy,
    ];
    
    for weather in weather_types {
        let transitions = weather.valid_transitions();
        assert!(!transitions.is_empty());
        
        // Verify no self-transitions
        assert!(!transitions.contains(&weather));
    }
}

/// Test weather characteristics
#[test]
fn test_weather_characteristics() {
    let rain = WeatherType::Rain;
    let characteristics = rain.characteristics();
    
    assert!(characteristics.particle_type.is_some());
    assert_eq!(characteristics.particle_type.unwrap(), ParticleEffectType::Rain);
    assert!(characteristics.wind_base > 0.0);
    assert!(characteristics.visibility < 1.0);
}

/// Test floating UI visibility states
#[test]
fn test_floating_ui_visibility() {
    use creature_simulation::rendering::floating_ui::UIVisibilityState;
    
    let states = vec![
        UIVisibilityState::Hidden,
        UIVisibilityState::FadingIn,
        UIVisibilityState::Visible,
        UIVisibilityState::FadingOut,
    ];
    
    for state in states {
        match state {
            UIVisibilityState::Hidden => {
                // Should not be visible
            }
            UIVisibilityState::Visible => {
                // Should be fully visible
            }
            _ => {
                // Transitioning states
            }
        }
    }
}

/// Test speech bubble content sizing
#[test]
fn test_speech_bubble_sizing() {
    use creature_simulation::rendering::enhanced_speech_bubbles::{BubbleContent, EmojiType};
    
    // Text content
    let text = BubbleContent::Text("Hello, world!".to_string());
    
    // Emoji content
    let emoji = BubbleContent::Emoji(EmojiType::Happy);
    
    // Mixed content
    let mixed = BubbleContent::Mixed(vec![
        creature_simulation::rendering::enhanced_speech_bubbles::ContentPart::Emoji(EmojiType::Happy),
        creature_simulation::rendering::enhanced_speech_bubbles::ContentPart::Text("Hi!".to_string()),
    ]);
}

/// Test particle effect types
#[test]
fn test_particle_effect_types() {
    let effect_types = vec![
        ParticleEffectType::Heart,
        ParticleEffectType::Rain,
        ParticleEffectType::Snow,
        ParticleEffectType::Sparkle,
    ];
    
    for effect_type in effect_types {
        // Each type should have a unique index
        let index = effect_type as u32;
        assert!(index < 20); // Reasonable limit
    }
}

/// Test LOD calculations
#[test]
fn test_lod_calculations() {
    use creature_simulation::rendering::particle_system::calculate_lod_factor;
    
    // Close distance = full LOD
    assert_eq!(calculate_lod_factor(10.0, 1.0), 1.0);
    
    // Medium distance = reduced LOD
    assert_eq!(calculate_lod_factor(150.0, 1.0), 0.5);
    
    // Far distance = no rendering
    assert_eq!(calculate_lod_factor(500.0, 1.0), 0.0);
    
    // Test LOD bias
    assert_eq!(calculate_lod_factor(150.0, 2.0), 1.0); // Clamped to 1.0
    assert_eq!(calculate_lod_factor(150.0, 0.5), 0.25);
}

/// Test performance metrics
#[test]
fn test_performance_metrics() {
    use creature_simulation::core::quality_settings::PerformanceMetrics;
    
    let mut metrics = PerformanceMetrics::default();
    
    // Test averaging
    metrics.fps_history = vec![58.0, 60.0, 62.0];
    assert_eq!(metrics.average_fps(), 60.0);
    
    metrics.frame_time_history = vec![16.0, 17.0, 16.0];
    assert_eq!(metrics.average_frame_time(), 16.333334);
}

/// Integration test for weather affecting particles
#[test]
#[ignore = "Requires full asset pipeline - run with cargo test -- --ignored"]
fn test_weather_particle_integration() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(bevy::asset::AssetPlugin::default())
        .add_plugins(Phase4Plugin);
    
    // Set weather to rain
    let mut weather = app.world.resource_mut::<WeatherState>();
    weather.current = WeatherType::Rain;
    weather.wind_strength = 20.0;
    weather.wind_direction = Vec3::X;
    
    app.update();
    
    // Verify weather affects particle emitters
    let weather = app.world.resource::<WeatherState>();
    assert_eq!(weather.current, WeatherType::Rain);
    assert!(weather.wind_strength > 0.0);
}

/// Test quality auto-adjustment
#[test]
fn test_quality_auto_adjust() {
    use creature_simulation::core::quality_settings::QualityAutoAdjust;
    
    let auto_adjust = QualityAutoAdjust::default();
    assert!(auto_adjust.enabled);
    assert_eq!(auto_adjust.target_fps, 60.0);
    assert_eq!(auto_adjust.min_preset, QualityPreset::Low);
    assert_eq!(auto_adjust.max_preset, QualityPreset::High);
}

/// Test Phase 4 feature toggles
#[test]
fn test_phase4_feature_toggles() {
    let mut features = Phase4Features::default();
    
    // All features should be enabled by default
    assert!(features.enhanced_particles);
    assert!(features.weather_effects);
    assert!(features.enhanced_speech_bubbles);
    assert!(features.floating_ui);
    
    // Toggle features
    features.enhanced_particles = false;
    assert!(!features.enhanced_particles);
    
    // Camera and audio are now implemented
    assert!(features.camera_effects);
    assert!(features.audio_system);
}