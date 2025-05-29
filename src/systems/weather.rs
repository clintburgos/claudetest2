use bevy::prelude::*;
use crate::rendering::BiomeType;
use crate::rendering::particle_system::{ParticleEmitter, ParticleEffectType, SpawnPattern, VelocityDistribution};
use rand::Rng;
use std::time::Duration;

/// Weather system plugin for Phase 4
/// 
/// Implements a state machine for weather transitions, environmental effects,
/// and biome-specific weather patterns
pub struct WeatherSystemPlugin;

impl Plugin for WeatherSystemPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WeatherState::default())
            .insert_resource(WeatherTransitionTimer::default())
            .insert_resource(DayNightCycle::default())
            .add_systems(Update, (
                update_weather_state_machine,
                update_day_night_cycle,
                spawn_weather_particles,
                update_environmental_effects,
                apply_weather_to_creatures,
            ).chain());
    }
}

/// Current weather state
#[derive(Resource, Default, Debug, Clone)]
pub struct WeatherState {
    pub current: WeatherType,
    pub next: Option<WeatherType>,
    pub transition_progress: f32,
    pub intensity: f32,
    pub wind_strength: f32,
    pub wind_direction: Vec3,
    pub temperature_modifier: f32,
    pub visibility: f32,
}

impl WeatherState {
    pub fn wind_vector(&self) -> Vec3 {
        self.wind_direction.normalize() * self.wind_strength
    }
    
    pub fn is_transitioning(&self) -> bool {
        self.next.is_some() && self.transition_progress < 1.0
    }
}

/// Weather types with different characteristics
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum WeatherType {
    #[default]
    Clear,
    Cloudy,
    Rain,
    Storm,
    Snow,
    Fog,
    Windy,
    Heatwave,
}

impl WeatherType {
    /// Get valid transitions from this weather type
    pub fn valid_transitions(&self) -> Vec<WeatherType> {
        match self {
            WeatherType::Clear => vec![WeatherType::Cloudy, WeatherType::Windy, WeatherType::Heatwave],
            WeatherType::Cloudy => vec![WeatherType::Clear, WeatherType::Rain, WeatherType::Fog],
            WeatherType::Rain => vec![WeatherType::Cloudy, WeatherType::Storm, WeatherType::Clear],
            WeatherType::Storm => vec![WeatherType::Rain, WeatherType::Cloudy],
            WeatherType::Snow => vec![WeatherType::Cloudy, WeatherType::Clear],
            WeatherType::Fog => vec![WeatherType::Cloudy, WeatherType::Clear],
            WeatherType::Windy => vec![WeatherType::Clear, WeatherType::Cloudy, WeatherType::Storm],
            WeatherType::Heatwave => vec![WeatherType::Clear, WeatherType::Windy],
        }
    }
    
    /// Get biome-specific probability for this weather
    pub fn biome_probability(&self, biome: BiomeType) -> f32 {
        match (self, biome) {
            // Desert favors clear, heatwave, and windy
            (WeatherType::Clear, BiomeType::Desert) => 0.6,
            (WeatherType::Heatwave, BiomeType::Desert) => 0.3,
            (WeatherType::Windy, BiomeType::Desert) => 0.2,
            (WeatherType::Rain, BiomeType::Desert) => 0.05,
            
            // Forest favors rain and fog
            (WeatherType::Rain, BiomeType::Forest) => 0.4,
            (WeatherType::Fog, BiomeType::Forest) => 0.3,
            (WeatherType::Clear, BiomeType::Forest) => 0.3,
            
            // Tundra favors snow and wind
            (WeatherType::Snow, BiomeType::Tundra) => 0.5,
            (WeatherType::Windy, BiomeType::Tundra) => 0.3,
            (WeatherType::Storm, BiomeType::Tundra) => 0.2,
            
            // Ocean favors storms and wind
            (WeatherType::Storm, BiomeType::Ocean) => 0.3,
            (WeatherType::Windy, BiomeType::Ocean) => 0.4,
            (WeatherType::Rain, BiomeType::Ocean) => 0.3,
            
            // Default probabilities
            _ => 0.1,
        }
    }
    
    /// Get weather characteristics
    pub fn characteristics(&self) -> WeatherCharacteristics {
        match self {
            WeatherType::Clear => WeatherCharacteristics {
                particle_type: None,
                wind_base: 5.0,
                wind_variance: 2.0,
                temperature_change: 0.0,
                visibility: 1.0,
                ambient_sound: "ambient_birds",
                light_modifier: 1.0,
            },
            WeatherType::Rain => WeatherCharacteristics {
                particle_type: Some(ParticleEffectType::Rain),
                wind_base: 15.0,
                wind_variance: 5.0,
                temperature_change: -5.0,
                visibility: 0.7,
                ambient_sound: "rain_medium",
                light_modifier: 0.7,
            },
            WeatherType::Storm => WeatherCharacteristics {
                particle_type: Some(ParticleEffectType::Rain),
                wind_base: 30.0,
                wind_variance: 10.0,
                temperature_change: -10.0,
                visibility: 0.5,
                ambient_sound: "storm_heavy",
                light_modifier: 0.4,
            },
            WeatherType::Snow => WeatherCharacteristics {
                particle_type: Some(ParticleEffectType::Snow),
                wind_base: 10.0,
                wind_variance: 3.0,
                temperature_change: -15.0,
                visibility: 0.6,
                ambient_sound: "wind_soft",
                light_modifier: 0.8,
            },
            WeatherType::Fog => WeatherCharacteristics {
                particle_type: Some(ParticleEffectType::Fog),
                wind_base: 2.0,
                wind_variance: 1.0,
                temperature_change: -2.0,
                visibility: 0.3,
                ambient_sound: "ambient_quiet",
                light_modifier: 0.6,
            },
            _ => WeatherCharacteristics::default(),
        }
    }
}

/// Weather characteristics
#[derive(Debug, Clone)]
pub struct WeatherCharacteristics {
    pub particle_type: Option<ParticleEffectType>,
    pub wind_base: f32,
    pub wind_variance: f32,
    pub temperature_change: f32,
    pub visibility: f32,
    pub ambient_sound: &'static str,
    pub light_modifier: f32,
}

impl Default for WeatherCharacteristics {
    fn default() -> Self {
        Self {
            particle_type: None,
            wind_base: 5.0,
            wind_variance: 2.0,
            temperature_change: 0.0,
            visibility: 1.0,
            ambient_sound: "ambient_nature",
            light_modifier: 1.0,
        }
    }
}

/// Timer for weather transitions
#[derive(Resource)]
pub struct WeatherTransitionTimer {
    pub timer: Timer,
    pub transition_duration: Duration,
}

impl Default for WeatherTransitionTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(300.0, TimerMode::Repeating), // Check every 5 minutes
            transition_duration: Duration::from_secs(30), // 30 second transitions
        }
    }
}

/// Day/night cycle state
#[derive(Resource)]
pub struct DayNightCycle {
    pub time_of_day: f32, // 0.0 - 24.0 hours
    pub speed: f32, // Hours per real second
    pub sun_position: Vec3,
    pub moon_position: Vec3,
    pub ambient_light: Color,
    pub is_night: bool,
}

impl Default for DayNightCycle {
    fn default() -> Self {
        Self {
            time_of_day: 12.0,
            speed: 0.1, // 1 game hour = 10 real seconds
            sun_position: Vec3::new(0.0, 1.0, 0.0),
            moon_position: Vec3::new(0.0, -1.0, 0.0),
            ambient_light: Color::WHITE,
            is_night: false,
        }
    }
}

/// System to update weather state machine
fn update_weather_state_machine(
    time: Res<Time>,
    mut weather: ResMut<WeatherState>,
    mut timer: ResMut<WeatherTransitionTimer>,
    // TODO: Get biome from world or terrain system
    // biome_query: Query<&BiomeType>,
) {
    timer.timer.tick(time.delta());
    
    // Update transition progress
    if weather.is_transitioning() {
        weather.transition_progress += time.delta_seconds() / timer.transition_duration.as_secs_f32();
        
        if weather.transition_progress >= 1.0 {
            // Complete transition
            weather.current = weather.next.unwrap();
            weather.next = None;
            weather.transition_progress = 0.0;
            
            // Set new intensity
            let mut rng = rand::thread_rng();
            weather.intensity = rng.gen_range(0.3..1.0);
        }
    }
    
    // Check for new weather transition
    if timer.timer.just_finished() && weather.next.is_none() {
        let mut rng = rand::thread_rng();
        
        // Get dominant biome (simplified - in reality would analyze world regions)
        let dominant_biome = BiomeType::Grassland; // Default for now
        
        // Calculate transition probabilities
        let valid_transitions = weather.current.valid_transitions();
        let mut weights = Vec::new();
        
        for weather_type in &valid_transitions {
            weights.push(weather_type.biome_probability(dominant_biome));
        }
        
        // Normalize weights
        let total_weight: f32 = weights.iter().sum();
        if total_weight > 0.0 {
            // Random selection based on weights
            let mut random_value = rng.gen_range(0.0..total_weight);
            
            for (i, weight) in weights.iter().enumerate() {
                random_value -= weight;
                if random_value <= 0.0 {
                    weather.next = Some(valid_transitions[i]);
                    break;
                }
            }
        }
    }
    
    // Update wind
    let characteristics = weather.current.characteristics();
    let mut rng = rand::thread_rng();
    
    weather.wind_strength = characteristics.wind_base + 
        rng.gen_range(-characteristics.wind_variance..characteristics.wind_variance);
    
    // Rotate wind direction slowly
    let rotation = Quat::from_rotation_y(time.delta_seconds() * 0.1);
    weather.wind_direction = rotation * weather.wind_direction;
    
    // Update other properties
    weather.temperature_modifier = characteristics.temperature_change * weather.intensity;
    weather.visibility = characteristics.visibility;
}

/// System to update day/night cycle
fn update_day_night_cycle(
    time: Res<Time>,
    mut cycle: ResMut<DayNightCycle>,
    mut ambient_light: ResMut<AmbientLight>,
) {
    // Update time of day
    cycle.time_of_day += time.delta_seconds() * cycle.speed;
    if cycle.time_of_day >= 24.0 {
        cycle.time_of_day -= 24.0;
    }
    
    // Calculate sun/moon positions
    let sun_angle = (cycle.time_of_day - 6.0) / 12.0 * std::f32::consts::PI;
    cycle.sun_position = Vec3::new(
        sun_angle.cos(),
        sun_angle.sin(),
        0.0,
    );
    
    let moon_angle = (cycle.time_of_day - 18.0) / 12.0 * std::f32::consts::PI;
    cycle.moon_position = Vec3::new(
        moon_angle.cos(),
        moon_angle.sin(),
        0.0,
    );
    
    // Calculate ambient light color
    cycle.is_night = cycle.time_of_day < 6.0 || cycle.time_of_day > 18.0;
    
    let light_intensity = if cycle.is_night {
        0.2 // Night time
    } else {
        let morning = ((cycle.time_of_day - 6.0) / 6.0).clamp(0.0, 1.0);
        let evening = ((18.0 - cycle.time_of_day) / 6.0).clamp(0.0, 1.0);
        morning.min(evening)
    };
    
    // Dawn/dusk colors
    let color = if cycle.time_of_day >= 5.0 && cycle.time_of_day <= 7.0 {
        // Dawn - pinkish
        Color::rgb(1.0, 0.7, 0.6)
    } else if cycle.time_of_day >= 17.0 && cycle.time_of_day <= 19.0 {
        // Dusk - orange
        Color::rgb(1.0, 0.6, 0.4)
    } else if cycle.is_night {
        // Night - bluish
        Color::rgb(0.4, 0.5, 0.7)
    } else {
        // Day - white
        Color::WHITE
    };
    
    cycle.ambient_light = color * light_intensity;
    ambient_light.color = cycle.ambient_light;
    ambient_light.brightness = light_intensity;
}

/// Component for weather particle emitters
#[derive(Component)]
pub struct WeatherEmitter {
    pub weather_type: WeatherType,
    pub coverage_area: Vec2,
}

/// System to spawn weather particles
fn spawn_weather_particles(
    mut commands: Commands,
    weather: Res<WeatherState>,
    existing_emitters: Query<(Entity, &WeatherEmitter)>,
    camera: Query<&Transform, With<Camera>>,
) {
    let characteristics = weather.current.characteristics();
    
    // Remove emitters if weather changed
    for (entity, emitter) in existing_emitters.iter() {
        if emitter.weather_type != weather.current {
            commands.entity(entity).despawn_recursive();
        }
    }
    
    // Spawn new emitters if needed
    if let Some(particle_type) = characteristics.particle_type {
        if existing_emitters.is_empty() {
            let camera_pos = camera.single().translation;
            
            // Create weather emitter covering visible area
            let coverage = Vec2::new(800.0, 600.0); // Adjust based on camera zoom
            
            commands.spawn((
                SpatialBundle {
                    transform: Transform::from_translation(camera_pos + Vec3::new(0.0, 300.0, 0.0)),
                    ..default()
                },
                ParticleEmitter {
                    effect_type: particle_type,
                    spawn_pattern: SpawnPattern::Box { 
                        size: Vec3::new(coverage.x, 0.0, coverage.y) 
                    },
                    spawn_rate: match particle_type {
                        ParticleEffectType::Rain => 100.0,
                        ParticleEffectType::Snow => 50.0,
                        ParticleEffectType::Fog => 10.0,
                        _ => 20.0,
                    },
                    particles_per_spawn: 5,
                    initial_velocity: match particle_type {
                        ParticleEffectType::Rain => VelocityDistribution::Constant(Vec3::new(0.0, -200.0, 0.0)),
                        ParticleEffectType::Snow => VelocityDistribution::Random { 
                            min: Vec3::new(-10.0, -30.0, -10.0), 
                            max: Vec3::new(10.0, -20.0, 10.0) 
                        },
                        _ => VelocityDistribution::Random { 
                            min: Vec3::new(-5.0, -5.0, -5.0), 
                            max: Vec3::new(5.0, 5.0, 5.0) 
                        },
                    },
                    lifetime_range: (2.0, 4.0),
                    max_particles: 1000,
                    spawn_timer: Timer::from_seconds(0.01, TimerMode::Repeating),
                    world_space: true,
                    lod_bias: 1.0,
                    active: true,
                },
                WeatherEmitter {
                    weather_type: weather.current,
                    coverage_area: coverage,
                },
                Name::new("WeatherEmitter"),
            ));
        }
    }
}

/// System to update environmental effects based on weather
fn update_environmental_effects(
    weather: Res<WeatherState>,
    // TODO: Add fog settings when available
    // mut fog_settings: ResMut<FogSettings>,
    mut directional_light: Query<&mut DirectionalLight>,
) {
    // TODO: Update fog based on weather when fog settings are available
    // match weather.current {
    //     WeatherType::Fog => {
    //         fog_settings.color = Color::rgba(0.7, 0.7, 0.7, 1.0);
    //         fog_settings.falloff = FogFalloff::Linear {
    //             start: 50.0,
    //             end: 200.0,
    //         };
    //     }
    //     WeatherType::Rain | WeatherType::Storm => {
    //         fog_settings.color = Color::rgba(0.6, 0.6, 0.7, 1.0);
    //         fog_settings.falloff = FogFalloff::Linear {
    //             start: 100.0,
    //             end: 400.0,
    //         };
    //     }
    //     _ => {
    //         fog_settings.falloff = FogFalloff::Linear {
    //             start: 200.0,
    //             end: 1000.0,
    //         };
    //     }
    // }
    
    // Update directional light intensity
    if let Ok(mut light) = directional_light.get_single_mut() {
        let characteristics = weather.current.characteristics();
        light.illuminance = 100000.0 * characteristics.light_modifier * weather.visibility;
    }
}

/// System to apply weather effects to creatures
fn apply_weather_to_creatures(
    weather: Res<WeatherState>,
    mut creatures: Query<(&mut Transform, &mut crate::components::Needs)>,
) {
    for (mut transform, mut needs) in creatures.iter_mut() {
        // Apply wind force to movement
        let wind_force = weather.wind_vector() * 0.1;
        transform.translation.x += wind_force.x;
        transform.translation.z += wind_force.z;
        
        // Weather affects needs
        match weather.current {
            WeatherType::Rain | WeatherType::Storm => {
                needs.thirst = (needs.thirst - 0.01).max(0.0); // Rain provides water
                // No comfort field in current implementation
            }
            WeatherType::Heatwave => {
                needs.thirst = (needs.thirst + 0.02).min(1.0); // Increased thirst
                needs.energy = (needs.energy - 0.005).max(0.0); // Heat exhaustion
            }
            WeatherType::Snow => {
                // No comfort field in current implementation
                needs.energy = (needs.energy - 0.003).max(0.0); // Uses energy to stay warm
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_weather_transitions() {
        let clear = WeatherType::Clear;
        let transitions = clear.valid_transitions();
        assert!(transitions.contains(&WeatherType::Cloudy));
        assert!(transitions.contains(&WeatherType::Windy));
        assert!(!transitions.contains(&WeatherType::Storm)); // Not a direct transition
    }
    
    #[test]
    fn test_biome_weather_probability() {
        assert!(WeatherType::Rain.biome_probability(BiomeType::Forest) > 
                WeatherType::Rain.biome_probability(BiomeType::Desert));
        
        assert!(WeatherType::Snow.biome_probability(BiomeType::Tundra) > 
                WeatherType::Snow.biome_probability(BiomeType::Forest));
    }
    
    #[test]
    fn test_day_night_cycle() {
        let mut cycle = DayNightCycle::default();
        cycle.time_of_day = 0.0; // Midnight
        assert!(cycle.time_of_day < 6.0 || cycle.time_of_day > 18.0);
        
        cycle.time_of_day = 12.0; // Noon
        assert!(cycle.time_of_day >= 6.0 && cycle.time_of_day <= 18.0);
    }
}