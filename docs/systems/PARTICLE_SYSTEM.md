# Particle System Architecture

## Overview

The particle system provides efficient, scalable visual effects for thousands of simultaneous particle emitters. It supports various particle types, behaviors, and rendering techniques while maintaining performance through pooling, batching, and LOD integration.

## Core Architecture

### Particle System Manager

```rust
pub struct ParticleSystem {
    // Particle pools by type
    pools: HashMap<ParticleType, ParticlePool>,
    
    // Active emitters
    emitters: Vec<ParticleEmitter>,
    emitter_pool: ObjectPool<ParticleEmitter>,
    
    // Rendering
    render_batches: HashMap<MaterialId, ParticleBatch>,
    instance_buffer: InstanceBuffer,
    
    // Performance
    max_particles: u32,
    active_particles: u32,
    lod_settings: ParticleLODSettings,
}

pub struct Particle {
    // Transform
    position: Vec3,
    velocity: Vec3,
    rotation: f32,
    angular_velocity: f32,
    scale: Vec2,
    
    // Lifetime
    age: f32,
    lifetime: f32,
    
    // Appearance
    color: Color,
    texture_index: u8,
    
    // Physics
    gravity_multiplier: f32,
    drag: f32,
    
    // Custom data
    custom_data: [f32; 4],
}

pub struct ParticlePool {
    particles: Vec<Particle>,
    alive_count: usize,
    particle_type: ParticleType,
    update_behavior: Box<dyn ParticleBehavior>,
}
```

### Particle Types

```rust
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum ParticleType {
    // Environmental
    Rain,
    Snow,
    Leaves,
    Dust,
    Pollen,
    
    // Effects
    Fire,
    Smoke,
    Steam,
    Bubbles,
    Sparkles,
    
    // Creature-related
    Footsteps,
    BreathVapor,
    Sweat,
    Blood,
    Tears,
    
    // Emotional
    Hearts,
    Anger,
    Confusion,
    Joy,
    Fear,
    
    // Combat
    Impact,
    Slash,
    Magic,
    Poison,
    
    // UI
    Selection,
    Notification,
    Highlight,
}

impl ParticleType {
    pub fn default_settings(&self) -> ParticleSettings {
        match self {
            ParticleType::Fire => ParticleSettings {
                texture: TextureAtlas::Fire,
                size_range: (0.5, 2.0),
                lifetime_range: (0.5, 1.5),
                emission_rate: 20.0,
                start_color: Color::rgb(1.0, 0.8, 0.2),
                end_color: Color::rgba(0.8, 0.2, 0.0, 0.0),
                velocity_range: (Vec3::new(-1.0, 2.0, -1.0), Vec3::new(1.0, 5.0, 1.0)),
                gravity: -2.0,
                drag: 2.0,
                blend_mode: BlendMode::Additive,
            },
            ParticleType::Snow => ParticleSettings {
                texture: TextureAtlas::Snow,
                size_range: (0.1, 0.3),
                lifetime_range: (5.0, 10.0),
                emission_rate: 100.0,
                start_color: Color::WHITE,
                end_color: Color::rgba(1.0, 1.0, 1.0, 0.0),
                velocity_range: (Vec3::new(-1.0, -2.0, -1.0), Vec3::new(1.0, -1.0, 1.0)),
                gravity: 0.5,
                drag: 5.0,
                blend_mode: BlendMode::Alpha,
            },
            // ... other types
        }
    }
}
```

### Particle Emitters

```rust
pub struct ParticleEmitter {
    // Identity
    id: EmitterId,
    particle_type: ParticleType,
    
    // Transform
    position: Vec3,
    rotation: Quat,
    attached_to: Option<Entity>,
    attachment_offset: Vec3,
    
    // Emission
    emission_shape: EmissionShape,
    emission_rate: f32,
    burst_config: Option<BurstConfig>,
    particles_to_emit: f32,
    
    // Lifetime
    duration: EmitterDuration,
    elapsed_time: f32,
    
    // Modifiers
    force_fields: Vec<ForceField>,
    color_over_lifetime: ColorCurve,
    size_over_lifetime: SizeCurve,
    
    // Performance
    lod_bias: f32,
    importance: f32,
}

pub enum EmissionShape {
    Point,
    Sphere { radius: f32 },
    Cone { angle: f32, radius: f32 },
    Box { size: Vec3 },
    Circle { radius: f32, thickness: f32 },
    Mesh { vertices: Vec<Vec3> },
    Line { start: Vec3, end: Vec3 },
}

pub enum EmitterDuration {
    Infinite,
    Timed(f32),
    UntilParticlesDead,
}

pub struct BurstConfig {
    time: f32,
    count: u32,
    repeat_interval: Option<f32>,
    probability: f32,
}
```

### Particle Behaviors

```rust
pub trait ParticleBehavior: Send + Sync {
    fn update(&self, particles: &mut [Particle], dt: f32, context: &UpdateContext);
    fn can_batch_with(&self, other: &dyn ParticleBehavior) -> bool;
}

pub struct StandardBehavior {
    gravity: Vec3,
    wind: Vec3,
    turbulence: TurbulenceSettings,
}

impl ParticleBehavior for StandardBehavior {
    fn update(&self, particles: &mut [Particle], dt: f32, context: &UpdateContext) {
        for particle in particles {
            // Apply forces
            particle.velocity += self.gravity * particle.gravity_multiplier * dt;
            particle.velocity += self.wind * dt;
            
            // Apply turbulence
            if self.turbulence.strength > 0.0 {
                let noise = self.calculate_turbulence(particle.position, context.time);
                particle.velocity += noise * self.turbulence.strength * dt;
            }
            
            // Apply drag
            particle.velocity *= 1.0 - particle.drag * dt;
            
            // Update position
            particle.position += particle.velocity * dt;
            
            // Update rotation
            particle.rotation += particle.angular_velocity * dt;
            
            // Update age
            particle.age += dt;
        }
    }
}

pub struct FlockingBehavior {
    cohesion: f32,
    separation: f32,
    alignment: f32,
    neighbor_distance: f32,
}

pub struct AttractorBehavior {
    attractors: Vec<Attractor>,
    repulsors: Vec<Repulsor>,
}
```

### Force Fields

```rust
pub enum ForceField {
    Gravity {
        direction: Vec3,
        strength: f32,
    },
    Point {
        position: Vec3,
        strength: f32,
        falloff: Falloff,
    },
    Vortex {
        position: Vec3,
        axis: Vec3,
        strength: f32,
        inward_force: f32,
    },
    Turbulence {
        scale: f32,
        strength: f32,
        octaves: u32,
    },
    Directional {
        direction: Vec3,
        strength: f32,
        randomness: f32,
    },
}

pub enum Falloff {
    Linear,
    Quadratic,
    Exponential { rate: f32 },
    Custom(Box<dyn Fn(f32) -> f32>),
}
```

## Rendering

### Particle Batching

```rust
pub struct ParticleBatch {
    material: MaterialId,
    instances: Vec<ParticleInstance>,
    vertex_buffer: Buffer,
    instance_buffer: Buffer,
    draw_count: u32,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ParticleInstance {
    position: [f32; 3],
    scale: f32,
    rotation: [f32; 4], // Quaternion
    color: [f32; 4],
    uv_offset_scale: [f32; 4], // For texture atlas
    custom_data: [f32; 4],
}

impl ParticleSystem {
    pub fn prepare_render_batches(&mut self) {
        self.render_batches.clear();
        
        // Sort particles by material/texture
        for pool in self.pools.values() {
            let material_id = pool.particle_type.material_id();
            let batch = self.render_batches
                .entry(material_id)
                .or_insert_with(|| ParticleBatch::new(material_id));
                
            // Convert particles to instances
            for i in 0..pool.alive_count {
                let particle = &pool.particles[i];
                
                // Frustum culling
                if !self.is_particle_visible(particle) {
                    continue;
                }
                
                batch.instances.push(ParticleInstance {
                    position: particle.position.to_array(),
                    scale: particle.scale.x,
                    rotation: Quat::from_rotation_z(particle.rotation).to_array(),
                    color: particle.color.as_rgba_f32(),
                    uv_offset_scale: self.calculate_uv(particle),
                    custom_data: particle.custom_data,
                });
            }
        }
        
        // Update GPU buffers
        for batch in self.render_batches.values_mut() {
            batch.update_buffers();
        }
    }
}
```

### Particle Shaders

```wgsl
// Vertex shader
struct ParticleInput {
    @location(0) position: vec3<f32>,
    @location(1) scale: f32,
    @location(2) rotation: vec4<f32>,
    @location(3) color: vec4<f32>,
    @location(4) uv_offset_scale: vec4<f32>,
    @location(5) custom_data: vec4<f32>,
}

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    instance: ParticleInput,
) -> VertexOutput {
    // Billboard quad vertices
    let quad_vertices = array<vec2<f32>, 4>(
        vec2<f32>(-0.5, -0.5),
        vec2<f32>(0.5, -0.5),
        vec2<f32>(-0.5, 0.5),
        vec2<f32>(0.5, 0.5),
    );
    
    let vertex_pos = quad_vertices[vertex_index];
    
    // Apply rotation and scale
    let rotated = rotate_2d(vertex_pos, instance.rotation);
    let scaled = rotated * instance.scale;
    
    // Billboard towards camera
    let world_pos = instance.position + 
        camera.right * scaled.x + 
        camera.up * scaled.y;
    
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(world_pos, 1.0);
    out.color = instance.color;
    out.uv = vertex_pos + 0.5;
    out.custom = instance.custom_data;
    
    return out;
}

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(particle_texture, particle_sampler, in.uv);
    
    // Soft particles (fade near geometry)
    let depth = textureSample(depth_texture, depth_sampler, in.clip_position.xy);
    let fade = calculate_soft_particle_fade(in.clip_position.z, depth);
    
    return tex_color * in.color * fade;
}
```

## LOD Integration

```rust
pub struct ParticleLODSettings {
    pub lod_distances: [f32; 4],
    pub emission_scale: [f32; 4],
    pub lifetime_scale: [f32; 4],
    pub size_scale: [f32; 4],
    pub update_frequency: [u32; 4],
}

impl ParticleEmitter {
    pub fn get_lod_level(&self, camera_distance: f32) -> u8 {
        let adjusted_distance = camera_distance / self.importance;
        
        for (i, &threshold) in self.lod_settings.lod_distances.iter().enumerate() {
            if adjusted_distance < threshold {
                return i as u8;
            }
        }
        
        return 4; // Culled
    }
    
    pub fn update_with_lod(&mut self, dt: f32, lod: u8) {
        if lod >= 4 {
            return; // Culled
        }
        
        // Scale emission rate
        let emission_scale = self.lod_settings.emission_scale[lod as usize];
        let actual_emission_rate = self.emission_rate * emission_scale;
        
        // Scale lifetime
        let lifetime_scale = self.lod_settings.lifetime_scale[lod as usize];
        
        // Update less frequently at lower LODs
        let update_frequency = self.lod_settings.update_frequency[lod as usize];
        if self.frame_count % update_frequency != 0 {
            return;
        }
        
        // Emit particles
        self.particles_to_emit += actual_emission_rate * dt * update_frequency as f32;
        
        while self.particles_to_emit >= 1.0 {
            self.emit_particle(lifetime_scale);
            self.particles_to_emit -= 1.0;
        }
    }
}
```

## Pooling System

```rust
pub struct ObjectPool<T> {
    available: Vec<T>,
    in_use: Vec<T>,
    generator: Box<dyn Fn() -> T>,
    reset_fn: Box<dyn Fn(&mut T)>,
    max_size: usize,
}

impl<T> ObjectPool<T> {
    pub fn acquire(&mut self) -> T {
        if let Some(mut item) = self.available.pop() {
            (self.reset_fn)(&mut item);
            item
        } else if self.in_use.len() < self.max_size {
            (self.generator)()
        } else {
            panic!("Object pool exhausted");
        }
    }
    
    pub fn release(&mut self, item: T) {
        if self.available.len() < self.max_size {
            self.available.push(item);
        }
    }
}

impl ParticleSystem {
    pub fn create_emitter(&mut self, config: EmitterConfig) -> EmitterId {
        let mut emitter = self.emitter_pool.acquire();
        emitter.configure(config);
        
        let id = EmitterId::new();
        emitter.id = id;
        
        self.emitters.push(emitter);
        id
    }
    
    pub fn destroy_emitter(&mut self, id: EmitterId) {
        if let Some(index) = self.emitters.iter().position(|e| e.id == id) {
            let emitter = self.emitters.swap_remove(index);
            self.emitter_pool.release(emitter);
        }
    }
}
```

## Special Effects

### Weather Particles

```rust
pub struct WeatherParticleSystem {
    rain_emitters: Vec<ParticleEmitter>,
    snow_emitters: Vec<ParticleEmitter>,
    fog_particles: FogSystem,
    wind_field: WindField,
}

impl WeatherParticleSystem {
    pub fn update_weather(&mut self, weather: &Weather, bounds: &AABB) {
        match weather.weather_type {
            WeatherType::Rain => {
                self.update_rain(weather.intensity, bounds);
            }
            WeatherType::Snow => {
                self.update_snow(weather.intensity, bounds);
            }
            WeatherType::Fog => {
                self.update_fog(weather.intensity, bounds);
            }
            _ => {}
        }
    }
    
    fn update_rain(&mut self, intensity: f32, bounds: &AABB) {
        // Create rain emitters at top of bounds
        let emitter_count = (intensity * 10.0) as usize;
        let spacing = bounds.size().x / emitter_count as f32;
        
        for i in 0..emitter_count {
            let x = bounds.min.x + spacing * i as f32;
            let emitter = ParticleEmitter {
                position: Vec3::new(x, bounds.max.y, bounds.center().z),
                emission_shape: EmissionShape::Box { 
                    size: Vec3::new(spacing, 0.1, bounds.size().z) 
                },
                particle_type: ParticleType::Rain,
                emission_rate: intensity * 100.0,
                // ... other settings
            };
            self.rain_emitters.push(emitter);
        }
    }
}
```

### Creature Effect Particles

```rust
pub struct CreatureParticleEffects {
    effect_emitters: HashMap<Entity, Vec<EmitterId>>,
    particle_system: Arc<Mutex<ParticleSystem>>,
}

impl CreatureParticleEffects {
    pub fn add_emotion_particles(&mut self, entity: Entity, emotion: Emotion) {
        let emitter_config = match emotion {
            Emotion::Joy => EmitterConfig {
                particle_type: ParticleType::Hearts,
                emission_shape: EmissionShape::Sphere { radius: 1.0 },
                emission_rate: 5.0,
                duration: EmitterDuration::Timed(2.0),
                // ... hearts floating upward
            },
            Emotion::Anger => EmitterConfig {
                particle_type: ParticleType::Anger,
                emission_shape: EmissionShape::Point,
                emission_rate: 20.0,
                duration: EmitterDuration::Timed(1.0),
                // ... steam/smoke effect
            },
            // ... other emotions
        };
        
        let emitter_id = self.particle_system.lock()
            .unwrap()
            .create_emitter(emitter_config);
            
        self.effect_emitters
            .entry(entity)
            .or_default()
            .push(emitter_id);
    }
    
    pub fn add_movement_particles(&mut self, entity: Entity, surface: SurfaceType) {
        match surface {
            SurfaceType::Water => {
                // Create splash/ripple particles
            }
            SurfaceType::Sand => {
                // Create dust cloud particles
            }
            SurfaceType::Snow => {
                // Create snow puff particles
            }
            _ => {}
        }
    }
}
```

## Performance Optimization

### Particle Culling

```rust
impl ParticleSystem {
    pub fn update_with_culling(&mut self, dt: f32, frustum: &Frustum) {
        for emitter in &mut self.emitters {
            // Cull entire emitter if outside frustum
            if !frustum.intersects_sphere(emitter.position, emitter.cull_radius()) {
                continue;
            }
            
            emitter.update(dt);
        }
        
        // Update particle pools with spatial partitioning
        for pool in self.pools.values_mut() {
            pool.update_partitioned(dt, frustum);
        }
    }
}

impl ParticlePool {
    pub fn update_partitioned(&mut self, dt: f32, frustum: &Frustum) {
        // Spatial partitioning for large particle counts
        if self.alive_count > 1000 {
            self.update_with_grid(dt, frustum);
        } else {
            self.update_simple(dt, frustum);
        }
    }
}
```

### GPU Simulation

```rust
pub struct GPUParticleSystem {
    particle_buffer: Buffer,
    emitter_buffer: Buffer,
    compute_pipeline: ComputePipeline,
    
    max_particles: u32,
    dispatch_size: (u32, u32, u32),
}

// Compute shader for particle updates
const PARTICLE_UPDATE_SHADER: &str = r#"
@group(0) @binding(0) var<storage, read_write> particles: array<Particle>;
@group(0) @binding(1) var<uniform> params: SimulationParams;

@compute @workgroup_size(64)
fn update_particles(@builtin(global_invocation_id) id: vec3<u32>) {
    let index = id.x;
    if (index >= params.particle_count) {
        return;
    }
    
    var particle = particles[index];
    
    // Update physics
    particle.velocity += params.gravity * params.dt;
    particle.position += particle.velocity * params.dt;
    particle.age += params.dt;
    
    // Check lifetime
    if (particle.age >= particle.lifetime) {
        particle.alive = 0u;
    }
    
    particles[index] = particle;
}
"#;
```

## Debug Visualization

```rust
pub struct ParticleDebugger {
    show_emitters: bool,
    show_forces: bool,
    show_bounds: bool,
    show_stats: bool,
    
    selected_emitter: Option<EmitterId>,
}

impl ParticleDebugger {
    pub fn render_debug(&self, particle_system: &ParticleSystem) {
        if self.show_emitters {
            for emitter in &particle_system.emitters {
                self.draw_emitter_gizmo(emitter);
            }
        }
        
        if self.show_forces {
            for emitter in &particle_system.emitters {
                for force in &emitter.force_fields {
                    self.draw_force_field(force);
                }
            }
        }
        
        if self.show_stats {
            self.draw_stats_overlay(particle_system);
        }
    }
    
    fn draw_stats_overlay(&self, system: &ParticleSystem) {
        let stats = format!(
            "Active Particles: {}/{}
Emitters: {}
Draw Calls: {}
Memory: {:.2} MB",
            system.active_particles,
            system.max_particles,
            system.emitters.len(),
            system.render_batches.len(),
            system.memory_usage() / 1024.0 / 1024.0
        );
        
        draw_text(&stats, Vec2::new(10.0, 10.0), Color::WHITE);
    }
}
```

This particle system provides rich visual effects while maintaining performance through batching, LOD, pooling, and GPU acceleration.