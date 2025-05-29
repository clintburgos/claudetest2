# Phase 4: Weather Implementation Design

## Overview

This document provides the complete technical design for the weather system in Phase 4, including weather states, transitions, environmental effects, accumulation systems, and biome-specific weather patterns.

## Weather State Machine

### Core Components

```rust
// Weather state representation
#[derive(Resource, Clone)]
pub struct WeatherState {
    pub current: Weather,
    pub next: Option<Weather>,
    pub transition_progress: f32,
    pub duration_remaining: f32,
    pub wind_direction: Vec2,
    pub wind_speed: f32,
    pub temperature: f32,
    pub humidity: f32,
    pub time_of_day: f32, // 0.0-24.0
}

#[derive(Clone, Copy, PartialEq)]
pub enum Weather {
    Clear {
        cloud_coverage: f32, // 0.0-0.3
    },
    Cloudy {
        cloud_coverage: f32, // 0.3-0.8
        cloud_speed: f32,
    },
    Rain {
        intensity: f32, // 0.0-1.0
        wind_strength: f32,
        lightning_chance: f32,
    },
    Snow {
        intensity: f32,
        temperature: f32,
        accumulation_rate: f32,
    },
    Fog {
        density: f32, // 0.0-1.0
        height: f32,
        movement_speed: f32,
    },
    Storm {
        rain_intensity: f32,
        wind_intensity: f32,
        lightning_frequency: f32,
        duration: f32,
    },
}

// Weather transition rules
pub struct WeatherTransitionRules {
    pub transitions: HashMap<(Weather, Weather), TransitionConfig>,
    pub biome_modifiers: HashMap<BiomeType, WeatherModifier>,
    pub seasonal_modifiers: Vec<SeasonalModifier>,
}

pub struct TransitionConfig {
    pub duration: f32,
    pub probability: f32,
    pub required_conditions: WeatherConditions,
    pub blend_curve: AnimationCurve,
}
```

### Weather State Machine

```rust
// Weather state machine system
pub fn update_weather_state(
    mut weather: ResMut<WeatherState>,
    time: Res<Time>,
    world_info: Res<WorldInfo>,
    transition_rules: Res<WeatherTransitionRules>,
) {
    let dt = time.delta_seconds();
    
    // Update time of day
    weather.time_of_day = (weather.time_of_day + dt / 3600.0) % 24.0;
    
    // Handle ongoing transition
    if let Some(next) = weather.next {
        weather.transition_progress += dt / get_transition_duration(&weather.current, &next);
        
        if weather.transition_progress >= 1.0 {
            weather.current = next;
            weather.next = None;
            weather.transition_progress = 0.0;
            weather.duration_remaining = generate_weather_duration(&weather.current);
        }
    } else {
        // Check for new transition
        weather.duration_remaining -= dt;
        
        if weather.duration_remaining <= 0.0 {
            // Select next weather based on current state and conditions
            if let Some(next) = select_next_weather(
                &weather.current,
                &world_info,
                &transition_rules,
                weather.time_of_day,
            ) {
                weather.next = Some(next);
                weather.transition_progress = 0.0;
            } else {
                // Continue current weather
                weather.duration_remaining = generate_weather_duration(&weather.current);
            }
        }
    }
    
    // Update wind
    update_wind_state(&mut weather, &world_info.current_biome, dt);
}

fn select_next_weather(
    current: &Weather,
    world_info: &WorldInfo,
    rules: &WeatherTransitionRules,
    time_of_day: f32,
) -> Option<Weather> {
    let biome_modifier = rules.biome_modifiers.get(&world_info.current_biome)?;
    let season_modifier = get_current_season_modifier(world_info.day, &rules.seasonal_modifiers);
    
    // Get possible transitions
    let mut candidates = Vec::new();
    
    for (from, to) in rules.transitions.keys() {
        if weather_matches(current, from) {
            let config = &rules.transitions[&(*from, *to)];
            
            // Check conditions
            if check_weather_conditions(&config.required_conditions, world_info, time_of_day) {
                let probability = config.probability 
                    * biome_modifier.get_probability_modifier(to)
                    * season_modifier.get_probability_modifier(to);
                
                candidates.push((*to, probability));
            }
        }
    }
    
    // Weighted random selection
    select_weighted_random(&candidates)
}
```

### Weather Transitions

```rust
// Smooth weather transitions
pub fn interpolate_weather_effects(
    weather: Res<WeatherState>,
    mut fog_settings: ResMut<FogSettings>,
    mut lighting: ResMut<AmbientLight>,
    mut sky_material: ResMut<SkyMaterial>,
) {
    let (current_fog, current_light, current_sky) = get_weather_effects(&weather.current);
    
    if let Some(next) = &weather.next {
        let (next_fog, next_light, next_sky) = get_weather_effects(next);
        let t = weather.transition_progress;
        
        // Smooth interpolation using easing curves
        let ease_t = ease_in_out_cubic(t);
        
        // Interpolate fog
        fog_settings.color = current_fog.color.lerp(next_fog.color, ease_t);
        fog_settings.directional_light_color = current_fog.directional_light_color
            .lerp(next_fog.directional_light_color, ease_t);
        fog_settings.directional_light_exponent = lerp(
            current_fog.directional_light_exponent,
            next_fog.directional_light_exponent,
            ease_t
        );
        fog_settings.falloff = match (current_fog.falloff, next_fog.falloff) {
            (FogFalloff::Linear { start: s1, end: e1 }, FogFalloff::Linear { start: s2, end: e2 }) => {
                FogFalloff::Linear {
                    start: lerp(s1, s2, ease_t),
                    end: lerp(e1, e2, ease_t),
                }
            }
            _ => current_fog.falloff, // Keep current if types don't match
        };
        
        // Interpolate lighting
        lighting.color = current_light.lerp(next_light, ease_t);
        
        // Interpolate sky
        sky_material.sun_intensity = lerp(current_sky.sun_intensity, next_sky.sun_intensity, ease_t);
        sky_material.cloud_coverage = lerp(current_sky.cloud_coverage, next_sky.cloud_coverage, ease_t);
    } else {
        // Apply current weather effects
        *fog_settings = current_fog;
        lighting.color = current_light;
        *sky_material = current_sky;
    }
}
```

## Precipitation Systems

### Rain Implementation

```rust
// Rain system with puddle formation
#[derive(Component)]
pub struct RainSystem {
    pub particle_emitter: Entity,
    pub intensity: f32,
    pub wind_offset: Vec2,
    pub puddle_accumulation: f32,
    pub splash_positions: Vec<Vec3>,
}

pub fn update_rain_system(
    mut rain_systems: Query<&mut RainSystem>,
    weather: Res<WeatherState>,
    terrain: Query<(&Transform, &TerrainTile)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let dt = time.delta_seconds();
    
    if let Weather::Rain { intensity, wind_strength, .. } = weather.current {
        for mut rain in rain_systems.iter_mut() {
            rain.intensity = intensity;
            rain.wind_offset = weather.wind_direction * wind_strength;
            
            // Accumulate water for puddles
            rain.puddle_accumulation += intensity * dt * 0.1;
            
            // Find low terrain points for puddles
            if rain.puddle_accumulation > 1.0 {
                spawn_puddles(&mut commands, &terrain, rain.puddle_accumulation);
                rain.puddle_accumulation = 0.0;
            }
            
            // Update splash positions based on particle collisions
            update_splash_positions(&mut rain.splash_positions, &terrain);
        }
    }
}

// Puddle component and rendering
#[derive(Component)]
pub struct Puddle {
    pub water_level: f32,
    pub max_level: f32,
    pub evaporation_rate: f32,
    pub surface_area: f32,
}

pub fn render_puddles(
    puddles: Query<(&Transform, &Puddle)>,
    mut puddle_mesh: ResMut<PuddleMesh>,
) {
    for (transform, puddle) in puddles.iter() {
        // Dynamic mesh generation based on water level
        let vertices = generate_puddle_mesh(
            puddle.water_level,
            puddle.surface_area,
            transform.translation
        );
        
        // Update mesh with water shader
        puddle_mesh.update_instance(PuddleInstance {
            transform: transform.clone(),
            water_level: puddle.water_level,
            ripple_offset: time.elapsed_seconds(),
        });
    }
}
```

### Snow Accumulation

```rust
// Snow accumulation system
#[derive(Component)]
pub struct SnowAccumulation {
    pub depth: f32,
    pub density: f32,
    pub melt_rate: f32,
    pub grid: Vec<Vec<f32>>, // Height map for accumulation
}

#[derive(Component)]
pub struct SnowCoveredEntity {
    pub snow_amount: f32,
    pub max_accumulation: f32,
    pub surface_angle: f32,
}

pub fn update_snow_accumulation(
    weather: Res<WeatherState>,
    mut snow_surfaces: Query<&mut SnowCoveredEntity>,
    mut terrain_snow: Query<&mut SnowAccumulation>,
    temperature: Res<EnvironmentTemperature>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    
    match weather.current {
        Weather::Snow { intensity, accumulation_rate, .. } => {
            // Accumulate on surfaces based on angle
            for mut surface in snow_surfaces.iter_mut() {
                let angle_factor = surface.surface_angle.cos().max(0.0);
                surface.snow_amount = (surface.snow_amount + 
                    accumulation_rate * intensity * angle_factor * dt)
                    .min(surface.max_accumulation);
            }
            
            // Update terrain snow grid
            for mut snow in terrain_snow.iter_mut() {
                update_snow_grid(&mut snow.grid, intensity, accumulation_rate, dt);
                snow.depth = calculate_average_depth(&snow.grid);
            }
        }
        _ => {
            // Melt snow if temperature is above freezing
            if temperature.current > 0.0 {
                let melt_rate = (temperature.current * 0.1).min(1.0);
                
                for mut surface in snow_surfaces.iter_mut() {
                    surface.snow_amount = (surface.snow_amount - melt_rate * dt).max(0.0);
                }
            }
        }
    }
}

// Snow rendering with accumulation
pub fn render_snow_coverage(
    snow_entities: Query<(&Transform, &SnowCoveredEntity, &Handle<Mesh>)>,
    mut snow_material: ResMut<SnowMaterial>,
) {
    for (transform, snow, mesh) in snow_entities.iter() {
        if snow.snow_amount > 0.01 {
            // Update snow material properties
            snow_material.coverage = snow.snow_amount / snow.max_accumulation;
            snow_material.sparkle_intensity = 0.3;
            
            // Apply snow layer on top of existing mesh
            render_snow_layer(transform, mesh, &snow_material, snow.snow_amount);
        }
    }
}
```

## Environmental Effects

### Fog System

```rust
// Volumetric fog implementation
#[derive(Resource)]
pub struct FogSystem {
    pub density_map: Texture3d,
    pub base_density: f32,
    pub height_falloff: f32,
    pub wind_scroll: Vec3,
    pub time_offset: f32,
}

pub fn update_fog_system(
    mut fog: ResMut<FogSystem>,
    weather: Res<WeatherState>,
    time: Res<Time>,
) {
    if let Weather::Fog { density, height, movement_speed } = weather.current {
        fog.base_density = density;
        fog.height_falloff = 1.0 / height;
        
        // Animate fog movement
        fog.time_offset += time.delta_seconds() * movement_speed;
        fog.wind_scroll = Vec3::new(
            weather.wind_direction.x * weather.wind_speed * 0.1,
            0.0,
            weather.wind_direction.y * weather.wind_speed * 0.1,
        );
        
        // Update density map for volumetric rendering
        update_fog_density_map(&mut fog.density_map, fog.time_offset, density);
    }
}

// Fog rendering shader integration
pub const FOG_SHADER: &str = r#"
    fn apply_fog(
        world_position: vec3<f32>,
        original_color: vec3<f32>,
        view_distance: f32,
    ) -> vec3<f32> {
        let fog_density = sample_fog_density(world_position);
        let height_factor = exp(-world_position.y * fog.height_falloff);
        let distance_factor = 1.0 - exp(-view_distance * fog.base_density);
        
        let fog_amount = fog_density * height_factor * distance_factor;
        let fog_color = mix(fog.scatter_color, fog.absorb_color, height_factor);
        
        return mix(original_color, fog_color, fog_amount);
    }
"#;
```

### Wind Effects

```rust
// Wind influence on particles and vegetation
#[derive(Component)]
pub struct WindAffected {
    pub resistance: f32, // 0.0 = full influence, 1.0 = no influence
    pub oscillation_amplitude: f32,
    pub oscillation_frequency: f32,
}

pub fn apply_wind_effects(
    weather: Res<WeatherState>,
    mut affected: Query<(&mut Transform, &WindAffected)>,
    time: Res<Time>,
) {
    let wind_force = weather.wind_direction * weather.wind_speed;
    let time_offset = time.elapsed_seconds();
    
    for (mut transform, wind_affected) in affected.iter_mut() {
        // Base wind influence
        let influence = 1.0 - wind_affected.resistance;
        
        // Oscillating motion for vegetation
        let oscillation = (time_offset * wind_affected.oscillation_frequency).sin() 
            * wind_affected.oscillation_amplitude
            * weather.wind_speed.min(10.0) / 10.0;
        
        // Apply rotation based on wind direction
        let wind_angle = wind_force.y.atan2(wind_force.x);
        let bend_amount = influence * weather.wind_speed * 0.01;
        
        transform.rotation = Quat::from_euler(
            EulerRot::XYZ,
            oscillation * 0.1,
            wind_angle,
            bend_amount + oscillation,
        );
    }
}
```

### Day/Night Cycle

```rust
// Time of day lighting system
#[derive(Resource)]
pub struct DayNightCycle {
    pub sun_position: Vec3,
    pub moon_position: Vec3,
    pub ambient_color: Color,
    pub sun_color: Color,
    pub shadow_strength: f32,
}

pub fn update_day_night_cycle(
    mut cycle: ResMut<DayNightCycle>,
    weather: Res<WeatherState>,
    mut directional_light: Query<(&mut DirectionalLight, &mut Transform)>,
) {
    let time_radians = (weather.time_of_day / 24.0) * std::f32::consts::TAU;
    
    // Calculate sun position
    cycle.sun_position = Vec3::new(
        time_radians.cos(),
        time_radians.sin().abs(), // Always above horizon
        0.3
    ).normalize();
    
    // Calculate moon position (opposite sun)
    cycle.moon_position = -cycle.sun_position;
    
    // Time-based colors
    let (ambient, sun_color, intensity) = match weather.time_of_day {
        t if t < 5.0 => (
            // Night
            Color::rgb(0.05, 0.05, 0.1),
            Color::rgb(0.4, 0.4, 0.5),
            0.1
        ),
        t if t < 7.0 => (
            // Dawn
            Color::rgb(0.3, 0.2, 0.3),
            Color::rgb(1.0, 0.7, 0.5),
            0.4
        ),
        t if t < 9.0 => (
            // Morning
            Color::rgb(0.5, 0.5, 0.6),
            Color::rgb(1.0, 0.9, 0.7),
            0.8
        ),
        t if t < 17.0 => (
            // Day
            Color::rgb(0.6, 0.6, 0.7),
            Color::rgb(1.0, 0.95, 0.8),
            1.0
        ),
        t if t < 19.0 => (
            // Evening
            Color::rgb(0.5, 0.4, 0.4),
            Color::rgb(1.0, 0.7, 0.4),
            0.6
        ),
        t if t < 21.0 => (
            // Dusk
            Color::rgb(0.3, 0.2, 0.3),
            Color::rgb(0.8, 0.5, 0.5),
            0.3
        ),
        _ => (
            // Night
            Color::rgb(0.05, 0.05, 0.1),
            Color::rgb(0.4, 0.4, 0.5),
            0.1
        ),
    };
    
    cycle.ambient_color = ambient;
    cycle.sun_color = sun_color;
    
    // Update directional light
    if let Ok((mut light, mut transform)) = directional_light.get_single_mut() {
        light.color = sun_color;
        light.illuminance = intensity * 10000.0; // Lux
        light.shadows_enabled = intensity > 0.3;
        
        transform.rotation = Quat::from_rotation_arc(Vec3::NEG_Z, cycle.sun_position);
    }
}
```

## Biome-Specific Weather

```rust
// Biome weather configurations
pub fn get_biome_weather_config(biome: BiomeType) -> BiomeWeatherConfig {
    match biome {
        BiomeType::Forest => BiomeWeatherConfig {
            weather_weights: vec![
                (Weather::Clear { cloud_coverage: 0.2 }, 0.3),
                (Weather::Cloudy { cloud_coverage: 0.6, cloud_speed: 1.0 }, 0.3),
                (Weather::Rain { intensity: 0.5, wind_strength: 0.3, lightning_chance: 0.1 }, 0.3),
                (Weather::Fog { density: 0.4, height: 5.0, movement_speed: 0.5 }, 0.1),
            ],
            temperature_range: (10.0, 25.0),
            humidity_range: (0.4, 0.8),
            wind_speed_multiplier: 0.7,
        },
        
        BiomeType::Desert => BiomeWeatherConfig {
            weather_weights: vec![
                (Weather::Clear { cloud_coverage: 0.0 }, 0.7),
                (Weather::Cloudy { cloud_coverage: 0.3, cloud_speed: 2.0 }, 0.2),
                (Weather::Storm { rain_intensity: 0.8, wind_intensity: 0.9, lightning_frequency: 0.3, duration: 1800.0 }, 0.1),
            ],
            temperature_range: (20.0, 45.0),
            humidity_range: (0.0, 0.2),
            wind_speed_multiplier: 1.5,
        },
        
        BiomeType::Tundra => BiomeWeatherConfig {
            weather_weights: vec![
                (Weather::Clear { cloud_coverage: 0.1 }, 0.2),
                (Weather::Cloudy { cloud_coverage: 0.7, cloud_speed: 0.5 }, 0.3),
                (Weather::Snow { intensity: 0.6, temperature: -5.0, accumulation_rate: 0.1 }, 0.4),
                (Weather::Fog { density: 0.6, height: 3.0, movement_speed: 0.3 }, 0.1),
            ],
            temperature_range: (-20.0, 5.0),
            humidity_range: (0.3, 0.6),
            wind_speed_multiplier: 1.2,
        },
        
        BiomeType::Ocean => BiomeWeatherConfig {
            weather_weights: vec![
                (Weather::Clear { cloud_coverage: 0.1 }, 0.3),
                (Weather::Cloudy { cloud_coverage: 0.5, cloud_speed: 1.5 }, 0.3),
                (Weather::Rain { intensity: 0.4, wind_strength: 0.5, lightning_chance: 0.05 }, 0.2),
                (Weather::Storm { rain_intensity: 1.0, wind_intensity: 1.0, lightning_frequency: 0.5, duration: 3600.0 }, 0.1),
                (Weather::Fog { density: 0.7, height: 2.0, movement_speed: 0.8 }, 0.1),
            ],
            temperature_range: (15.0, 25.0),
            humidity_range: (0.7, 1.0),
            wind_speed_multiplier: 1.8,
        },
    }
}
```

## Performance Optimization

### Weather LOD System

```rust
// Level of detail for weather effects
pub fn apply_weather_lod(
    camera: Query<&Transform, With<Camera>>,
    mut weather_effects: Query<(&Transform, &mut WeatherEffect)>,
) {
    let camera_pos = camera.single().translation;
    
    for (transform, mut effect) in weather_effects.iter_mut() {
        let distance = (transform.translation - camera_pos).length();
        
        effect.lod_level = match distance {
            d if d < 50.0 => WeatherLOD::Full,
            d if d < 100.0 => WeatherLOD::Reduced,
            d if d < 200.0 => WeatherLOD::Minimal,
            _ => WeatherLOD::Disabled,
        };
    }
}

#[derive(Clone, Copy)]
pub enum WeatherLOD {
    Full,     // All effects, full particle count
    Reduced,  // 50% particles, simplified shaders
    Minimal,  // 10% particles, basic effects only
    Disabled, // No weather effects
}
```

### Weather Culling

```rust
// Frustum culling for weather effects
pub fn cull_weather_effects(
    camera: Query<(&Camera, &GlobalTransform)>,
    mut effects: Query<(&Transform, &mut Visibility), With<WeatherEffect>>,
) {
    let (camera, camera_transform) = camera.single();
    let frustum = camera.frustum(camera_transform);
    
    for (transform, mut visibility) in effects.iter_mut() {
        let in_frustum = frustum.intersects_sphere(
            transform.translation,
            WEATHER_EFFECT_RADIUS
        );
        
        visibility.is_visible = in_frustum;
    }
}
```

## Shader Integration

```wgsl
// Weather effects vertex shader
struct WeatherUniforms {
    time: f32,
    wind_direction: vec2<f32>,
    wind_speed: f32,
    precipitation_intensity: f32,
    fog_density: f32,
    fog_height: f32,
}

@group(1) @binding(0)
var<uniform> weather: WeatherUniforms;

@vertex
fn weather_vertex(
    vertex: Vertex,
    instance: WeatherInstance,
) -> VertexOutput {
    var position = vertex.position;
    
    // Apply wind displacement
    position.xz += weather.wind_direction * weather.wind_speed * vertex.uv.y;
    
    // Rain/snow fall animation
    position.y -= weather.time * instance.fall_speed;
    position.y = fract(position.y / WEATHER_HEIGHT) * WEATHER_HEIGHT;
    
    // Output with fog pre-calculation
    var output: VertexOutput;
    output.world_position = (instance.transform * vec4<f32>(position, 1.0)).xyz;
    output.fog_factor = calculate_fog(output.world_position);
    
    return output;
}

// Weather fragment shader
@fragment
fn weather_fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = in.color;
    
    // Apply fog
    color = mix(color, weather.fog_color, in.fog_factor);
    
    // Distance fade
    let fade = 1.0 - smoothstep(WEATHER_NEAR, WEATHER_FAR, in.view_distance);
    color.a *= fade;
    
    return color;
}
```

## Testing and Validation

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_weather_transitions() {
        let mut weather = WeatherState::default();
        weather.current = Weather::Clear { cloud_coverage: 0.1 };
        weather.next = Some(Weather::Rain { intensity: 0.5, wind_strength: 0.3, lightning_chance: 0.1 });
        
        // Test smooth transition
        for i in 0..100 {
            weather.transition_progress = i as f32 / 100.0;
            let effects = interpolate_weather_effects(&weather);
            
            // Ensure values are interpolated
            assert!(effects.fog_density >= 0.0 && effects.fog_density <= 1.0);
        }
    }
    
    #[test]
    fn test_biome_weather_probability() {
        let config = get_biome_weather_config(BiomeType::Desert);
        
        // Desert should rarely have rain
        let rain_weight = config.weather_weights.iter()
            .find(|(w, _)| matches!(w, Weather::Rain { .. }))
            .map(|(_, weight)| weight)
            .unwrap_or(&0.0);
            
        assert!(*rain_weight < 0.2);
    }
}
```