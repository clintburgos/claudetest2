use bevy::prelude::*;

/// Phase 4 Cartoon Isometric Graphics Plugin
/// 
/// Integrates all Phase 4 enhancements:
/// - Enhanced particle system with GPU optimization
/// - Weather system with environmental effects
/// - Enhanced speech bubbles with dynamic sizing
/// - Floating UI elements
/// - Quality settings and performance optimization
/// - Spatial audio system with animation synchronization
/// - Dynamic camera effects (shake, zoom, picture-in-picture)
pub struct Phase4Plugin;

impl Plugin for Phase4Plugin {
    fn build(&self, app: &mut App) {
        // Add quality settings first (other systems depend on it)
        app.add_plugins(crate::core::quality_settings::QualitySettingsPlugin);
        
        // Add enhanced particle system
        app.add_plugins(crate::rendering::particle_system::EnhancedParticlePlugin);
        
        // Add weather system
        app.add_plugins(crate::systems::weather::WeatherSystemPlugin);
        
        // Add enhanced UI systems
        app.add_plugins((
            crate::rendering::enhanced_speech_bubbles::EnhancedSpeechBubblePlugin,
            crate::rendering::floating_ui::FloatingUIPlugin,
        ));
        
        // Add audio system
        app.add_plugins(crate::rendering::audio_system::CartoonAudioPlugin);
        
        // Add camera effects
        app.add_plugins(crate::rendering::camera_effects::CameraEffectsPlugin);
        
        // Add Phase 4 specific resources
        app.insert_resource(Phase4Features::default());
        
        // Add integration systems
        app.add_systems(Startup, setup_phase4_defaults)
            .add_systems(Update, (
                integrate_weather_with_particles,
                update_ui_based_on_quality,
                handle_phase4_debug_input,
            ));
    }
}

/// Phase 4 feature toggles
#[derive(Resource)]
pub struct Phase4Features {
    pub enhanced_particles: bool,
    pub weather_effects: bool,
    pub enhanced_speech_bubbles: bool,
    pub floating_ui: bool,
    pub camera_effects: bool,
    pub audio_system: bool,
}

impl Default for Phase4Features {
    fn default() -> Self {
        Self {
            enhanced_particles: true,
            weather_effects: true,
            enhanced_speech_bubbles: true,
            floating_ui: true,
            camera_effects: true,  // Now implemented
            audio_system: true,    // Now implemented
        }
    }
}

/// Setup default Phase 4 configurations
fn setup_phase4_defaults(
    mut commands: Commands,
    mut quality_settings: ResMut<crate::core::quality_settings::QualitySettings>,
) {
    // Set default quality to Medium
    *quality_settings = crate::core::quality_settings::QualitySettings::from_preset(
        crate::core::quality_settings::QualityPreset::Medium
    );
    
    // Log Phase 4 initialization
    info!("Phase 4 Cartoon Isometric Graphics initialized");
    info!("Features enabled:");
    info!("  - Enhanced particle system with GPU instancing");
    info!("  - Weather state machine with environmental effects");
    info!("  - Dynamic speech bubbles with emoji support");
    info!("  - Floating UI elements (health bars, need indicators)");
    info!("  - Quality presets and auto-adjustment");
}

/// Integrate weather system with particle effects
fn integrate_weather_with_particles(
    weather: Res<crate::systems::weather::WeatherState>,
    mut particle_emitters: Query<&mut crate::rendering::particle_system::ParticleEmitter>,
    features: Res<Phase4Features>,
) {
    if !features.weather_effects || !features.enhanced_particles {
        return;
    }
    
    // Adjust particle emitters based on weather
    let wind = weather.wind_vector();
    
    for mut emitter in particle_emitters.iter_mut() {
        // Apply wind to certain particle types
        match emitter.effect_type {
            crate::rendering::particle_system::ParticleEffectType::Rain |
            crate::rendering::particle_system::ParticleEffectType::Snow |
            crate::rendering::particle_system::ParticleEffectType::Leaves => {
                // Modify initial velocity based on wind
                if let crate::rendering::particle_system::VelocityDistribution::Constant(ref mut vel) = emitter.initial_velocity {
                    *vel += wind * 0.5;
                }
            }
            _ => {}
        }
    }
}

/// Update UI visibility based on quality settings
fn update_ui_based_on_quality(
    quality: Res<crate::core::quality_settings::QualitySettings>,
    mut floating_ui_settings: ResMut<crate::rendering::floating_ui::FloatingUISettings>,
    features: Res<Phase4Features>,
) {
    if !quality.is_changed() {
        return;
    }
    
    // Apply quality settings to UI
    if features.floating_ui {
        floating_ui_settings.ui_scale = quality.ui_scale;
        floating_ui_settings.animation_speed = if quality.ui_animations { 1.0 } else { 0.0 };
        
        // Adjust visibility settings based on quality
        match quality.preset {
            crate::core::quality_settings::QualityPreset::Low => {
                floating_ui_settings.show_health_always = false;
                floating_ui_settings.show_needs_when_critical = true;
                floating_ui_settings.fade_distance = 150.0;
            }
            crate::core::quality_settings::QualityPreset::Ultra => {
                floating_ui_settings.show_health_when_damaged = true;
                floating_ui_settings.show_needs_when_selected = true;
                floating_ui_settings.fade_distance = 300.0;
            }
            _ => {}
        }
    }
}

/// Handle debug input for Phase 4 features
fn handle_phase4_debug_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut features: ResMut<Phase4Features>,
    mut quality_commands: Commands,
    mut weather: ResMut<crate::systems::weather::WeatherState>,
) {
    // F10: Toggle Phase 4 features
    if keyboard.just_pressed(KeyCode::F10) {
        if keyboard.pressed(KeyCode::ShiftLeft) {
            // Shift+F10: Toggle all Phase 4 features
            let all_enabled = features.enhanced_particles && 
                             features.weather_effects && 
                             features.enhanced_speech_bubbles && 
                             features.floating_ui;
            
            features.enhanced_particles = !all_enabled;
            features.weather_effects = !all_enabled;
            features.enhanced_speech_bubbles = !all_enabled;
            features.floating_ui = !all_enabled;
            
            info!("Phase 4 features {}", if !all_enabled { "enabled" } else { "disabled" });
        } else {
            // F10: Cycle quality presets
            use crate::core::quality_settings::{QualityPreset, QualityCommands};
            
            let current_preset = quality_commands.set_quality_preset(QualityPreset::Low);
            info!("Quality preset: Low");
        }
    }
    
    // F11: Cycle weather
    if keyboard.just_pressed(KeyCode::F11) && features.weather_effects {
        use crate::systems::weather::WeatherType;
        
        let next_weather = match weather.current {
            WeatherType::Clear => WeatherType::Cloudy,
            WeatherType::Cloudy => WeatherType::Rain,
            WeatherType::Rain => WeatherType::Storm,
            WeatherType::Storm => WeatherType::Snow,
            WeatherType::Snow => WeatherType::Fog,
            WeatherType::Fog => WeatherType::Windy,
            WeatherType::Windy => WeatherType::Clear,
            _ => WeatherType::Clear,
        };
        
        weather.current = next_weather;
        weather.intensity = 0.8;
        info!("Weather changed to: {:?}", next_weather);
    }
    
    // F12: Show Phase 4 debug info
    if keyboard.just_pressed(KeyCode::F12) {
        info!("=== Phase 4 Debug Info ===");
        info!("Features:");
        info!("  Enhanced Particles: {}", features.enhanced_particles);
        info!("  Weather Effects: {}", features.weather_effects);
        info!("  Enhanced Speech Bubbles: {}", features.enhanced_speech_bubbles);
        info!("  Floating UI: {}", features.floating_ui);
        info!("Weather: {:?} (intensity: {:.2})", weather.current, weather.intensity);
        info!("Wind: {:.1} units in direction {:?}", weather.wind_strength, weather.wind_direction);
    }
}

/// Extension trait for easy Phase 4 setup
pub trait Phase4Extension {
    fn add_phase4_graphics(&mut self) -> &mut Self;
}

impl Phase4Extension for App {
    fn add_phase4_graphics(&mut self) -> &mut Self {
        self.add_plugins(Phase4Plugin)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_phase4_features_default() {
        let features = Phase4Features::default();
        assert!(features.enhanced_particles);
        assert!(features.weather_effects);
        assert!(features.enhanced_speech_bubbles);
        assert!(features.floating_ui);
        assert!(features.camera_effects); // Now implemented
        assert!(features.audio_system);   // Now implemented
    }
}