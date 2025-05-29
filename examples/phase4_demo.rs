//! Phase 4 Cartoon Isometric Graphics Demo
//! 
//! Demonstrates all Phase 4 features:
//! - Enhanced particle system with GPU optimization
//! - Weather system with environmental effects
//! - Dynamic speech bubbles
//! - Floating UI elements
//! - Quality settings and auto-adjustment

use bevy::prelude::*;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use creature_simulation::{
    plugins::{
        CameraPlugin, RenderingPlugin, SelectionPlugin, UiEguiPlugin,
        Phase4Plugin, CreatureSprite, ResourceSprite,
    },
    components::{
        Position, Velocity, Health, Needs, CreatureBundle,
        CreatureType, CreatureState, Age, Size, Genetics,
        MaxSpeed, DecisionTimer, CurrentTarget, Creature,
        ConversationState, ConversationTopic, Decision,
    },
    systems::weather::{WeatherState, WeatherType},
    core::quality_settings::{QualitySettings, QualityPreset},
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_plugins(LogDiagnosticsPlugin::default())
        // Initialize required resources
        .init_resource::<creature_simulation::plugins::SimulationSettings>()
        .init_resource::<creature_simulation::core::simulation_control::SimulationControl>()
        .init_resource::<creature_simulation::core::performance_monitor::PerformanceMonitor>()
        .init_resource::<creature_simulation::systems::observation_goals::ObservationGoals>()
        // Core plugins
        .add_plugins((
            CameraPlugin,
            RenderingPlugin,
            SelectionPlugin,
            UiEguiPlugin,
        ))
        // Phase 4 plugin
        .add_plugins(Phase4Plugin)
        // Demo systems
        .add_systems(Startup, (setup_demo, spawn_demo_creatures))
        .add_systems(Update, (
            demo_controls,
            move_creatures,
            trigger_speech_bubbles,
        ))
        .run();
}

/// Setup demo scene
fn setup_demo(
    mut commands: Commands,
    mut quality: ResMut<QualitySettings>,
    mut weather: ResMut<WeatherState>,
) {
    // Set initial quality to High
    *quality = QualitySettings::from_preset(QualityPreset::High);
    
    // Set initial weather to Rain for particle effects
    weather.current = WeatherType::Rain;
    weather.intensity = 0.7;
    weather.wind_strength = 15.0;
    weather.wind_direction = Vec3::new(1.0, 0.0, 0.5).normalize();
    
    // Add ground plane
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.4, 0.6, 0.3),
                custom_size: Some(Vec2::new(2000.0, 2000.0)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, -10.0)),
            ..default()
        },
        Name::new("Ground"),
    ));
    
    info!("=== Phase 4 Demo Controls ===");
    info!("F10: Cycle quality presets");
    info!("F11: Cycle weather types");
    info!("F12: Show debug info");
    info!("Space: Trigger random speech bubble");
    info!("Click: Select creature");
    info!("1-5: Damage selected creature");
}

/// Spawn demo creatures
fn spawn_demo_creatures(mut commands: Commands) {
    let positions = vec![
        Vec2::new(-200.0, -100.0),
        Vec2::new(0.0, -100.0),
        Vec2::new(200.0, -100.0),
        Vec2::new(-100.0, 100.0),
        Vec2::new(100.0, 100.0),
    ];
    
    for (i, pos) in positions.iter().enumerate() {
        commands.spawn((
            CreatureBundle {
                creature: Creature,
                creature_type: if i % 2 == 0 { 
                    CreatureType::Herbivore 
                } else { 
                    CreatureType::Carnivore 
                },
                position: Position(*pos),
                velocity: Velocity(Vec2::ZERO),
                health: Health {
                    current: 80.0 - (i as f32 * 10.0), // Varying health levels
                    max: 100.0,
                },
                needs: Needs {
                    hunger: 0.2 + (i as f32 * 0.1),
                    thirst: 0.3 + (i as f32 * 0.05),
                    energy: 0.9 - (i as f32 * 0.1),
                    social: 0.5,
                },
                state: CreatureState::Idle,
                age: Age(30.0 + (i as f32 * 10.0)),
                size: Size(1.0 + (i as f32 * 0.1)),
                genetics: Genetics::default(),
                max_speed: MaxSpeed(50.0),
                decision_timer: DecisionTimer::default(),
                current_target: CurrentTarget::None,
            },
            CreatureSprite,
            SpriteBundle {
                transform: Transform::from_translation(pos.extend(0.0)),
                ..default()
            },
            Name::new(format!("DemoCreature_{}", i)),
        ));
    }
}

/// Demo control system
fn demo_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut quality_commands: Commands,
    mut weather: ResMut<WeatherState>,
    mut next_preset: Local<usize>,
) {
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
        weather.intensity = 0.8;
        info!("Weather changed to: {:?}", next_weather);
    }
}

/// Simple creature movement
fn move_creatures(
    time: Res<Time>,
    mut creatures: Query<(&mut Position, &mut Velocity, &MaxSpeed)>,
) {
    for (mut pos, mut vel, max_speed) in creatures.iter_mut() {
        // Simple wandering behavior
        let change = Vec2::new(
            (time.elapsed_seconds() * 0.5 + pos.0.x * 0.01).sin() * 10.0,
            (time.elapsed_seconds() * 0.3 + pos.0.y * 0.01).cos() * 10.0,
        );
        
        vel.0 = (vel.0 + change * time.delta_seconds()).clamp_length_max(max_speed.0);
        pos.0 += vel.0 * time.delta_seconds();
    }
}

/// Trigger speech bubbles on spacebar
fn trigger_speech_bubbles(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    creatures: Query<Entity, With<Creature>>,
    mut bubble_timer: Local<f32>,
    time: Res<Time>,
) {
    *bubble_timer -= time.delta_seconds();
    
    if keyboard.just_pressed(KeyCode::Space) && *bubble_timer <= 0.0 {
        // Add ConversationState to a random creature
        let entities: Vec<Entity> = creatures.iter().collect();
        if !entities.is_empty() {
            let random_index = rand::random::<usize>() % entities.len();
            let entity = entities[random_index];
            
            let states = vec![
                ConversationState::Greeting,
                ConversationState::ShareInfo(ConversationTopic::FoodLocation),
                ConversationState::RequestHelp,
                ConversationState::OfferHelp,
            ];
            let random_state = states[rand::random::<usize>() % states.len()].clone();
            
            commands.entity(entity).insert(random_state);
            *bubble_timer = 5.0; // Reset timer
            info!("Triggered speech bubble");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_demo_setup() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(Phase4Plugin);
        
        // Should initialize without panicking
        app.update();
    }
}