//! Minimal Phase 4 test without UI
//! 
//! Tests Phase 4 features without egui to isolate issues

use bevy::prelude::*;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use creature_simulation::{
    plugins::{
        CameraPlugin, RenderingPlugin, Phase4Plugin,
    },
    systems::weather::{WeatherState, WeatherType},
    core::quality_settings::{QualitySettings, QualityPreset},
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Phase 4 Test".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_plugins(LogDiagnosticsPlugin::default())
        // Core plugins (no UI)
        .add_plugins((
            CameraPlugin,
            RenderingPlugin,
        ))
        // Phase 4 plugin
        .add_plugins(Phase4Plugin)
        // Test systems
        .add_systems(Startup, setup_test)
        .add_systems(Update, test_controls)
        .run();
}

fn setup_test(
    mut commands: Commands,
    mut quality: ResMut<QualitySettings>,
    mut weather: ResMut<WeatherState>,
) {
    // Set initial quality to High
    *quality = QualitySettings::from_preset(QualityPreset::High);
    
    // Set initial weather to Rain for particle effects
    weather.current = WeatherType::Rain;
    weather.intensity = 0.7;
    
    // Spawn camera
    commands.spawn(Camera2dBundle::default());
    
    // Add ground plane
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(0.4, 0.6, 0.3),
            custom_size: Some(Vec2::new(2000.0, 2000.0)),
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, -10.0)),
        ..default()
    });
    
    info!("=== Phase 4 Test (No UI) ===");
    info!("F10: Cycle quality presets");
    info!("F11: Cycle weather types");
    info!("ESC: Exit");
}

fn test_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut quality_commands: Commands,
    mut weather: ResMut<WeatherState>,
    mut next_preset: Local<usize>,
    mut exit: EventWriter<AppExit>,
) {
    // ESC: Exit
    if keyboard.just_pressed(KeyCode::Escape) {
        exit.send(AppExit);
    }
    
    // F10: Cycle quality presets
    if keyboard.just_pressed(KeyCode::F10) {
        let presets = vec![
            QualityPreset::Low,
            QualityPreset::Medium,
            QualityPreset::High,
            QualityPreset::Ultra,
        ];
        
        *next_preset = (*next_preset + 1) % presets.len();
        let preset = presets[*next_preset];
        
        quality_commands.insert_resource(QualitySettings::from_preset(preset));
        info!("Quality preset changed to: {:?}", preset);
    }
    
    // F11: Cycle weather
    if keyboard.just_pressed(KeyCode::F11) {
        let next_weather = match weather.current {
            WeatherType::Clear => WeatherType::Rain,
            WeatherType::Rain => WeatherType::Snow,
            WeatherType::Snow => WeatherType::Fog,
            WeatherType::Fog => WeatherType::Storm,
            WeatherType::Storm => WeatherType::Clear,
            _ => WeatherType::Clear,
        };
        
        weather.current = next_weather;
        info!("Weather changed to: {:?}", next_weather);
    }
}