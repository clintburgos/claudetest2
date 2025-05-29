//! Simple Phase 4 Demo - No UI Dependencies
//! 
//! Demonstrates Phase 4 features with minimal dependencies

use bevy::prelude::*;
use bevy::app::AppExit;
use creature_simulation::{
    plugins::Phase4Plugin,
    systems::weather::{WeatherState, WeatherType},
    core::quality_settings::{QualitySettings, QualityPreset},
};

fn main() {
    println!("Starting Phase 4 Simple Demo...");
    println!("Controls:");
    println!("  Q: Change quality preset");
    println!("  W: Change weather");
    println!("  ESC: Exit");
    
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Phase 4 Simple Demo".to_string(),
                resolution: (800.0, 600.0).into(),
                ..default()
            }),
            ..default()
        }))
        // Phase 4 plugin
        .add_plugins(Phase4Plugin)
        // Demo systems
        .add_systems(Startup, setup)
        .add_systems(Update, controls)
        .run();
}

fn setup(
    mut commands: Commands,
    quality: Res<QualitySettings>,
    weather: Res<WeatherState>,
) {
    // Spawn camera
    commands.spawn(Camera2dBundle::default());
    
    // Log initial state
    info!("Phase 4 initialized!");
    info!("Quality preset: {:?}", quality.preset);
    info!("Weather: {:?}", weather.current);
    info!("Max particles: {}", quality.max_particles);
    info!("Particle density: {}", quality.particle_density);
}

fn controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut quality: ResMut<QualitySettings>,
    mut weather: ResMut<WeatherState>,
    mut exit: EventWriter<AppExit>,
) {
    // ESC: Exit
    if keyboard.just_pressed(KeyCode::Escape) {
        exit.send(AppExit);
    }
    
    // Q: Cycle quality
    if keyboard.just_pressed(KeyCode::KeyQ) {
        let next_preset = match quality.preset {
            QualityPreset::Low => QualityPreset::Medium,
            QualityPreset::Medium => QualityPreset::High,
            QualityPreset::High => QualityPreset::Ultra,
            QualityPreset::Ultra => QualityPreset::Low,
            QualityPreset::Custom => QualityPreset::Low,
        };
        *quality = QualitySettings::from_preset(next_preset);
        info!("Quality changed to: {:?}", next_preset);
        info!("  Max particles: {}", quality.max_particles);
        info!("  Render distance: {}", quality.render_distance);
    }
    
    // W: Cycle weather
    if keyboard.just_pressed(KeyCode::KeyW) {
        let next_weather = match weather.current {
            WeatherType::Clear => WeatherType::Rain,
            WeatherType::Rain => WeatherType::Snow,
            WeatherType::Snow => WeatherType::Fog,
            WeatherType::Fog => WeatherType::Storm,
            WeatherType::Storm => WeatherType::Windy,
            WeatherType::Windy => WeatherType::Cloudy,
            WeatherType::Cloudy => WeatherType::Heatwave,
            WeatherType::Heatwave => WeatherType::Clear,
        };
        weather.current = next_weather;
        info!("Weather changed to: {:?}", next_weather);
        let characteristics = next_weather.characteristics();
        info!("  Wind: {}", characteristics.wind_base);
        info!("  Visibility: {}", characteristics.visibility);
        if let Some(particle) = characteristics.particle_type {
            info!("  Particles: {:?}", particle);
        }
    }
}