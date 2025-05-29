# Phase 4: Particle System Design

## Overview

This document provides the complete technical design for the particle system in Phase 4 of the cartoon isometric implementation. The system handles emotion particles, weather effects, action feedback, and environmental particles with GPU optimization and performance scaling.

## Architecture

### Core Components

```rust
// Particle instance data for GPU instancing
#[derive(Component, Clone)]
pub struct ParticleInstance {
    pub position: Vec3,
    pub velocity: Vec3,
    pub acceleration: Vec3,
    pub rotation: f32,
    pub angular_velocity: f32,
    pub scale: Vec2,
    pub color: Color,
    pub lifetime: f32,
    pub age: f32,
    pub texture_index: u32,
}

// Particle emitter component
#[derive(Component)]
pub struct ParticleEmitter {
    pub effect_type: ParticleEffectType,
    pub spawn_pattern: SpawnPattern,
    pub spawn_rate: f32,
    pub initial_velocity: VelocityDistribution,
    pub lifetime: LifetimeDistribution,
    pub max_particles: u32,
    pub active_particles: Vec<ParticleInstance>,
    pub warmup_time: f32,
    pub loop_duration: Option<f32>,
    pub world_space: bool,
    pub lod_scale: f32,
}

// Particle effect types
#[derive(Clone, Copy, PartialEq)]
pub enum ParticleEffectType {
    // Emotion particles
    Heart { size: f32, float_speed: f32 },
    Zzz { wave_amplitude: f32 },
    Sparkle { twinkle_rate: f32 },
    Sweat { gravity: f32 },
    Exclamation { shake_intensity: f32 },
    Question { rotation_speed: f32 },
    
    // Weather particles
    Rain { wind_influence: f32, splash_chance: f32 },
    Snow { flutter_amount: f32, accumulation: bool },
    Fog { density: f32, movement_speed: f32 },
    Wind { dust_particles: bool, strength: f32 },
    
    // Action feedback
    Impact { debris_count: u32, force: f32 },
    Footstep { dust_amount: f32, surface_type: SurfaceType },
    Splash { droplet_count: u32, height: f32 },
    
    // Environmental
    Leaves { fall_pattern: FallPattern, colors: Vec<Color> },
    Fireflies { glow_intensity: f32, movement_pattern: MovementPattern },
    Pollen { drift_speed: f32, density: f32 },
}
```

### Particle Pool System

```rust
// Global particle pool for performance
pub struct ParticlePool {
    // Pre-allocated particle buffers per effect type
    pools: HashMap<ParticleEffectType, ParticleBuffer>,
    // GPU instance buffer
    instance_buffer: Buffer,
    // Maximum particles across all systems
    global_max_particles: u32,
    // Current particle count
    active_particle_count: u32,
}

pub struct ParticleBuffer {
    particles: Vec<ParticleInstance>,
    free_indices: Vec<usize>,
    active_count: u32,
    max_size: u32,
}

impl ParticlePool {
    pub fn allocate(&mut self, effect_type: ParticleEffectType, count: u32) -> Option<ParticleAllocation> {
        // Check global limit
        if self.active_particle_count + count > self.global_max_particles {
            return None;
        }
        
        // Get buffer for effect type
        let buffer = self.pools.get_mut(&effect_type)?;
        
        // Allocate from free list
        if buffer.free_indices.len() >= count as usize {
            let indices: Vec<usize> = buffer.free_indices.drain(..count as usize).collect();
            buffer.active_count += count;
            self.active_particle_count += count;
            
            Some(ParticleAllocation {
                effect_type,
                indices,
                buffer_offset: buffer.calculate_gpu_offset(),
            })
        } else {
            None
        }
    }
}
```

### GPU Optimization

```rust
// Particle rendering system with instancing
pub fn render_particles(
    mut commands: Commands,
    particle_pool: Res<ParticlePool>,
    emitters: Query<&ParticleEmitter>,
    camera: Query<&Transform, With<Camera>>,
) {
    let camera_pos = camera.single().translation;
    
    // Group particles by texture for batching
    let mut particle_batches: HashMap<Handle<Image>, Vec<ParticleInstance>> = HashMap::new();
    
    for emitter in emitters.iter() {
        // LOD calculation based on distance
        let distance = (emitter.transform.translation - camera_pos).length();
        let lod_factor = calculate_lod_factor(distance, emitter.lod_scale);
        
        // Skip if too far
        if lod_factor <= 0.0 {
            continue;
        }
        
        // Reduce particle count based on LOD
        let visible_particles = (emitter.active_particles.len() as f32 * lod_factor) as usize;
        
        // Add to appropriate batch
        let texture = get_particle_texture(emitter.effect_type);
        particle_batches.entry(texture)
            .or_insert_with(Vec::new)
            .extend(emitter.active_particles.iter().take(visible_particles));
    }
    
    // Submit batched draw calls
    for (texture, instances) in particle_batches {
        if instances.is_empty() {
            continue;
        }
        
        // Update GPU instance buffer
        particle_pool.instance_buffer.write(&instances);
        
        // Single draw call for all particles of this texture
        commands.spawn(ParticleRenderBatch {
            texture,
            instance_count: instances.len() as u32,
            instance_buffer: particle_pool.instance_buffer.clone(),
        });
    }
}
```

### Particle Behaviors

```rust
// Particle update system
pub fn update_particles(
    time: Res<Time>,
    mut emitters: Query<&mut ParticleEmitter>,
    weather: Res<WeatherState>,
) {
    let dt = time.delta_seconds();
    let wind = weather.wind_vector();
    
    for mut emitter in emitters.iter_mut() {
        // Update existing particles
        emitter.active_particles.retain_mut(|particle| {
            particle.age += dt;
            
            // Check lifetime
            if particle.age >= particle.lifetime {
                return false;
            }
            
            // Apply physics based on effect type
            match emitter.effect_type {
                ParticleEffectType::Heart { float_speed, .. } => {
                    // Float upward with slight wave
                    particle.velocity.y = float_speed;
                    particle.position.x += (particle.age * 2.0).sin() * 0.5 * dt;
                }
                
                ParticleEffectType::Rain { wind_influence, .. } => {
                    // Gravity + wind
                    particle.acceleration.y = -9.8;
                    particle.velocity += wind * wind_influence * dt;
                }
                
                ParticleEffectType::Snow { flutter_amount, .. } => {
                    // Gentle falling with flutter
                    particle.velocity.y = -1.0;
                    particle.position.x += (particle.age * 3.0).sin() * flutter_amount * dt;
                    particle.rotation += dt * 0.5;
                }
                
                ParticleEffectType::Sparkle { twinkle_rate } => {
                    // Pulsing alpha based on time
                    let alpha = ((particle.age * twinkle_rate).sin() + 1.0) * 0.5;
                    particle.color.set_a(alpha);
                }
                
                _ => {}
            }
            
            // Update position
            particle.velocity += particle.acceleration * dt;
            particle.position += particle.velocity * dt;
            particle.rotation += particle.angular_velocity * dt;
            
            true
        });
        
        // Spawn new particles
        spawn_new_particles(&mut emitter, dt);
    }
}
```

## Performance Budgets

### Particle Limits by Priority

| Effect Type | Max Particles | LOD Range | Priority |
|------------|---------------|-----------|----------|
| Emotion (Heart, Zzz, etc.) | 50 per creature | 0-50 units | High |
| Weather (Rain/Snow) | 2000 global | 0-100 units | Medium |
| Action Feedback | 100 per action | 0-30 units | High |
| Environmental | 500 global | 0-80 units | Low |
| **Total Budget** | **5000** | - | - |

### LOD Scaling

```rust
pub fn calculate_lod_factor(distance: f32, base_scale: f32) -> f32 {
    let lod_ranges = [
        (0.0, 20.0, 1.0),    // Full detail
        (20.0, 50.0, 0.6),   // Reduced
        (50.0, 80.0, 0.3),   // Minimal
        (80.0, 100.0, 0.1),  // Very sparse
    ];
    
    for (min, max, factor) in lod_ranges {
        if distance >= min && distance < max {
            // Smooth interpolation within range
            let t = (distance - min) / (max - min);
            let prev_factor = if min == 0.0 { 1.0 } else { lod_ranges.iter().find(|(_, m, _)| *m == min).unwrap().2 };
            return prev_factor + (factor - prev_factor) * t * base_scale;
        }
    }
    
    0.0 // Beyond max range
}
```

### Memory Management

```rust
// Particle memory budget: ~20MB
// Each particle: 64 bytes
// Max particles: 5000
// Buffer size: 5000 * 64 = 320KB base
// With pooling overhead: ~1MB
// Texture atlas: 2048x2048 RGBA = 16MB
// Total: ~20MB

pub const PARTICLE_BUDGET: ParticleBudget = ParticleBudget {
    max_global_particles: 5000,
    max_emitters: 200,
    max_textures: 32,
    instance_buffer_size: 1024 * 1024, // 1MB
    texture_atlas_size: (2048, 2048),
};
```

## Integration Points

### Animation System Integration

```rust
// Spawn particles at animation events
pub fn handle_animation_particles(
    mut commands: Commands,
    mut animation_events: EventReader<AnimationEvent>,
    creatures: Query<&Transform, With<Creature>>,
) {
    for event in animation_events.iter() {
        if let AnimationEvent::FootstepFrame { entity, foot } = event {
            if let Ok(transform) = creatures.get(*entity) {
                let foot_offset = match foot {
                    Foot::Left => Vec3::new(-0.2, 0.0, 0.0),
                    Foot::Right => Vec3::new(0.2, 0.0, 0.0),
                };
                
                commands.spawn(ParticleEmitter {
                    effect_type: ParticleEffectType::Footstep {
                        dust_amount: 0.5,
                        surface_type: SurfaceType::Dirt,
                    },
                    spawn_pattern: SpawnPattern::Burst { count: 5 },
                    transform: Transform::from_translation(
                        transform.translation + foot_offset
                    ),
                    ..default()
                });
            }
        }
    }
}
```

### Weather System Integration

```rust
// Global weather particles
pub fn spawn_weather_particles(
    mut commands: Commands,
    weather: Res<WeatherState>,
    mut particle_manager: ResMut<GlobalParticleManager>,
) {
    match weather.current {
        Weather::Rain { intensity } => {
            particle_manager.set_weather_particles(ParticleEffectType::Rain {
                wind_influence: 0.3,
                splash_chance: intensity * 0.1,
            }, intensity * 2000.0);
        }
        Weather::Snow { intensity } => {
            particle_manager.set_weather_particles(ParticleEffectType::Snow {
                flutter_amount: 2.0,
                accumulation: intensity > 0.5,
            }, intensity * 1500.0);
        }
        _ => {
            particle_manager.clear_weather_particles();
        }
    }
}
```

## Shader Implementation

```wgsl
// Particle vertex shader with instancing
struct ParticleInstance {
    @location(0) position: vec3<f32>,
    @location(1) velocity: vec3<f32>,
    @location(2) rotation: f32,
    @location(3) scale: vec2<f32>,
    @location(4) color: vec4<f32>,
    @location(5) texture_index: u32,
    @location(6) age_lifetime: vec2<f32>, // (age, lifetime)
};

@vertex
fn particle_vertex(
    @builtin(vertex_index) vertex_index: u32,
    instance: ParticleInstance,
) -> VertexOutput {
    // Billboard quad vertices
    let vertices = array<vec2<f32>, 4>(
        vec2<f32>(-0.5, -0.5),
        vec2<f32>(0.5, -0.5),
        vec2<f32>(-0.5, 0.5),
        vec2<f32>(0.5, 0.5)
    );
    
    let vertex = vertices[vertex_index];
    
    // Apply rotation
    let cos_r = cos(instance.rotation);
    let sin_r = sin(instance.rotation);
    let rotated = vec2<f32>(
        vertex.x * cos_r - vertex.y * sin_r,
        vertex.x * sin_r + vertex.y * cos_r
    );
    
    // Scale and position
    let world_pos = instance.position + vec3<f32>(
        rotated * instance.scale,
        0.0
    );
    
    // Age-based fade
    let age_factor = instance.age_lifetime.x / instance.age_lifetime.y;
    let alpha = instance.color.a * (1.0 - smoothstep(0.8, 1.0, age_factor));
    
    var output: VertexOutput;
    output.position = view_projection * vec4<f32>(world_pos, 1.0);
    output.uv = (vertex + 0.5) * get_atlas_uv_scale(instance.texture_index);
    output.color = vec4<f32>(instance.color.rgb, alpha);
    
    return output;
}
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_particle_pool_allocation() {
        let mut pool = ParticlePool::new(1000);
        
        // Allocate particles
        let allocation = pool.allocate(ParticleEffectType::Heart, 50).unwrap();
        assert_eq!(allocation.indices.len(), 50);
        assert_eq!(pool.active_particle_count, 50);
        
        // Free particles
        pool.free(allocation);
        assert_eq!(pool.active_particle_count, 0);
    }
    
    #[test]
    fn test_lod_scaling() {
        assert_eq!(calculate_lod_factor(0.0, 1.0), 1.0);
        assert_eq!(calculate_lod_factor(35.0, 1.0), 0.6); // Interpolated
        assert_eq!(calculate_lod_factor(100.0, 1.0), 0.0);
    }
    
    #[test]
    fn test_particle_physics() {
        let mut particle = ParticleInstance {
            position: Vec3::ZERO,
            velocity: Vec3::new(1.0, 0.0, 0.0),
            acceleration: Vec3::new(0.0, -9.8, 0.0),
            ..default()
        };
        
        update_particle_physics(&mut particle, 0.1);
        
        assert!((particle.position.x - 0.1).abs() < 0.001);
        assert!((particle.velocity.y - -0.98).abs() < 0.001);
    }
}
```

### Performance Benchmarks

```rust
#[bench]
fn bench_particle_update_5000(b: &mut Bencher) {
    let mut emitters = create_test_emitters(5000);
    
    b.iter(|| {
        update_particles(&mut emitters, 0.016);
    });
}

#[bench]
fn bench_particle_rendering_batched(b: &mut Bencher) {
    let particles = create_test_particles(5000);
    
    b.iter(|| {
        render_particles_batched(&particles);
    });
}
```

## Usage Examples

### Emotion Particle Spawning

```rust
// Spawn hearts when creatures become happy
pub fn spawn_happiness_particles(
    mut commands: Commands,
    happy_creatures: Query<(&Transform, &EmotionState), Changed<EmotionState>>,
) {
    for (transform, emotion) in happy_creatures.iter() {
        if emotion.current == Emotion::Happy {
            commands.spawn(ParticleEmitterBundle {
                emitter: ParticleEmitter {
                    effect_type: ParticleEffectType::Heart {
                        size: 0.5,
                        float_speed: 2.0,
                    },
                    spawn_pattern: SpawnPattern::Burst { count: 3 },
                    lifetime: LifetimeDistribution::Uniform { min: 1.0, max: 2.0 },
                    ..default()
                },
                transform: transform.clone(),
            });
        }
    }
}
```

### Weather Particle Configuration

```rust
// Configure rain particles
let rain_emitter = ParticleEmitter {
    effect_type: ParticleEffectType::Rain {
        wind_influence: 0.5,
        splash_chance: 0.1,
    },
    spawn_pattern: SpawnPattern::Continuous {
        rate: 100.0, // particles per second
    },
    initial_velocity: VelocityDistribution::Cone {
        direction: Vec3::new(0.0, -1.0, 0.0),
        angle: 0.1,
        speed_range: (5.0, 8.0),
    },
    lifetime: LifetimeDistribution::Normal {
        mean: 2.0,
        std_dev: 0.2,
    },
    max_particles: 500,
    world_space: true,
    ..default()
};
```

## Future Enhancements

1. **GPU Compute Particles**: Move particle physics to compute shaders
2. **Collision Detection**: Particles reacting to terrain/obstacles
3. **Particle Lighting**: Emissive particles affecting scene lighting
4. **Advanced Behaviors**: Flocking, attraction/repulsion fields
5. **Particle Trails**: Motion blur and trail effects
6. **Sub-Emitters**: Particles spawning other particles