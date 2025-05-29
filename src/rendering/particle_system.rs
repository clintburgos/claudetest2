use bevy::prelude::*;
use std::collections::HashMap;
use rand::Rng;

/// Enhanced particle system with GPU instancing and pooling for Phase 4
/// 
/// Features:
/// - GPU instanced rendering for massive particle counts
/// - Object pooling to eliminate allocations
/// - LOD system for performance scaling
/// - Weather effects support
/// - Per-particle physics simulation
pub struct EnhancedParticlePlugin;

impl Plugin for EnhancedParticlePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ParticlePool::new(10000)) // Global pool of 10k particles
            .insert_resource(ParticleTextures::default())
            .add_systems(Startup, setup_particle_assets)
            .add_systems(Update, (
                spawn_particles_from_emitters,
                update_particle_physics,
                update_particle_buffers,
                cleanup_inactive_particles,
            ).chain())
            .add_systems(PostUpdate, render_particle_batches);
    }
}

/// Particle instance data for GPU instancing
#[derive(Component, Clone, Copy, Debug)]
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

/// Enhanced particle emitter with spawn patterns
#[derive(Component)]
pub struct ParticleEmitter {
    pub effect_type: ParticleEffectType,
    pub spawn_pattern: SpawnPattern,
    pub spawn_rate: f32,
    pub particles_per_spawn: u32,
    pub initial_velocity: VelocityDistribution,
    pub lifetime_range: (f32, f32),
    pub max_particles: u32,
    pub spawn_timer: Timer,
    pub world_space: bool,
    pub lod_bias: f32,
    pub active: bool,
}

/// Particle effect types including weather
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ParticleEffectType {
    // Emotion particles
    Heart,
    Zzz,
    Sparkle,
    Sweat,
    Exclamation,
    Question,
    
    // Weather particles
    Rain,
    Snow,
    Fog,
    Wind,
    Lightning,
    
    // Action feedback
    Impact,
    Footstep,
    Splash,
    Dust,
    
    // Environmental
    Leaves,
    Fireflies,
    Pollen,
    Smoke,
}

/// Spawn patterns for particle emission
#[derive(Clone, Debug)]
pub enum SpawnPattern {
    Point,
    Circle { radius: f32 },
    Sphere { radius: f32 },
    Cone { angle: f32, direction: Vec3 },
    Box { size: Vec3 },
    Line { start: Vec3, end: Vec3 },
}

/// Velocity distribution for initial particle velocity
#[derive(Clone, Debug)]
pub enum VelocityDistribution {
    Constant(Vec3),
    Random { min: Vec3, max: Vec3 },
    Radial { speed: f32 },
    Cone { direction: Vec3, angle: f32, speed: f32 },
}

/// Global particle pool for performance
#[derive(Resource)]
pub struct ParticlePool {
    particles: Vec<ParticleInstance>,
    free_indices: Vec<usize>,
    active_count: u32,
    max_particles: u32,
    // TODO: Add GPU buffer when render pipeline is implemented
    // instance_buffer: Option<Buffer>,
}

impl ParticlePool {
    pub fn new(max_particles: u32) -> Self {
        let mut particles = Vec::with_capacity(max_particles as usize);
        let mut free_indices = Vec::with_capacity(max_particles as usize);
        
        // Pre-allocate all particles
        for i in 0..max_particles {
            particles.push(ParticleInstance {
                position: Vec3::ZERO,
                velocity: Vec3::ZERO,
                acceleration: Vec3::ZERO,
                rotation: 0.0,
                angular_velocity: 0.0,
                scale: Vec2::ONE,
                color: Color::WHITE,
                lifetime: 0.0,
                age: 0.0,
                texture_index: 0,
            });
            free_indices.push(i as usize);
        }
        
        Self {
            particles,
            free_indices,
            active_count: 0,
            max_particles,
            // instance_buffer: None,
        }
    }
    
    pub fn allocate(&mut self, count: u32) -> Option<Vec<usize>> {
        if self.free_indices.len() < count as usize {
            return None;
        }
        
        let indices: Vec<usize> = self.free_indices.drain(..count as usize).collect();
        self.active_count += count;
        Some(indices)
    }
    
    pub fn free(&mut self, indices: Vec<usize>) {
        self.active_count -= indices.len() as u32;
        self.free_indices.extend(indices);
    }
}

/// Resource containing particle textures
#[derive(Resource, Default)]
pub struct ParticleTextures {
    pub atlas: Handle<Image>,
    pub texture_coords: HashMap<ParticleEffectType, (u32, u32, u32, u32)>, // x, y, width, height
}

/// System to setup particle assets
fn setup_particle_assets(
    mut textures: ResMut<ParticleTextures>,
    asset_server: Res<AssetServer>,
) {
    // Load particle atlas
    textures.atlas = asset_server.load("sprites/particles/particle_atlas.png");
    
    // Define texture coordinates for each particle type
    // Assuming 256x256 atlas with 32x32 particles
    textures.texture_coords.insert(ParticleEffectType::Heart, (0, 0, 32, 32));
    textures.texture_coords.insert(ParticleEffectType::Zzz, (32, 0, 32, 32));
    textures.texture_coords.insert(ParticleEffectType::Sparkle, (64, 0, 32, 32));
    textures.texture_coords.insert(ParticleEffectType::Sweat, (96, 0, 32, 32));
    textures.texture_coords.insert(ParticleEffectType::Exclamation, (128, 0, 32, 32));
    textures.texture_coords.insert(ParticleEffectType::Question, (160, 0, 32, 32));
    textures.texture_coords.insert(ParticleEffectType::Rain, (0, 32, 8, 16));
    textures.texture_coords.insert(ParticleEffectType::Snow, (32, 32, 16, 16));
    textures.texture_coords.insert(ParticleEffectType::Dust, (64, 32, 16, 16));
}

/// System to spawn particles from emitters
fn spawn_particles_from_emitters(
    time: Res<Time>,
    mut pool: ResMut<ParticlePool>,
    mut emitters: Query<(&mut ParticleEmitter, &GlobalTransform)>,
    camera: Query<&GlobalTransform, With<Camera>>,
) {
    let camera_pos = camera.single().translation();
    
    for (mut emitter, transform) in emitters.iter_mut() {
        if !emitter.active {
            continue;
        }
        
        // LOD check
        let distance = (transform.translation() - camera_pos).length();
        let lod_factor = calculate_lod_factor(distance, emitter.lod_bias);
        
        if lod_factor <= 0.0 {
            continue;
        }
        
        // Update spawn timer
        emitter.spawn_timer.tick(time.delta());
        
        if emitter.spawn_timer.finished() {
            // Calculate particles to spawn based on LOD
            let spawn_count = (emitter.particles_per_spawn as f32 * lod_factor) as u32;
            
            if let Some(indices) = pool.allocate(spawn_count) {
                // Initialize particles
                for idx in indices {
                    let particle = &mut pool.particles[idx];
                    
                    // Set position based on spawn pattern
                    particle.position = calculate_spawn_position(
                        &emitter.spawn_pattern,
                        transform.translation(),
                    );
                    
                    // Set velocity
                    particle.velocity = calculate_initial_velocity(&emitter.initial_velocity);
                    
                    // Set particle properties based on effect type
                    apply_effect_properties(particle, emitter.effect_type);
                    
                    // Set lifetime
                    let mut rng = rand::thread_rng();
                    particle.lifetime = if emitter.lifetime_range.0 < emitter.lifetime_range.1 {
                        rng.gen_range(emitter.lifetime_range.0..emitter.lifetime_range.1)
                    } else {
                        emitter.lifetime_range.0 // Use first value if range is invalid
                    };
                    particle.age = 0.0;
                }
            }
        }
    }
}

/// System to update particle physics
fn update_particle_physics(
    time: Res<Time>,
    mut pool: ResMut<ParticlePool>,
    weather: Option<Res<WeatherState>>,
) {
    let dt = time.delta_seconds();
    let wind = weather.map(|w| w.wind_vector()).unwrap_or(Vec3::ZERO);
    
    for particle in pool.particles.iter_mut() {
        if particle.age >= particle.lifetime {
            continue;
        }
        
        // Update age
        particle.age += dt;
        
        // Apply physics
        particle.velocity += particle.acceleration * dt;
        
        // Apply wind to certain particle types
        match ParticleEffectType::from_index(particle.texture_index) {
            Some(ParticleEffectType::Snow) | 
            Some(ParticleEffectType::Leaves) |
            Some(ParticleEffectType::Pollen) => {
                particle.velocity += wind * 0.5 * dt;
            }
            Some(ParticleEffectType::Rain) => {
                particle.velocity += wind * 0.3 * dt;
            }
            _ => {}
        }
        
        // Update position
        particle.position += particle.velocity * dt;
        
        // Update rotation
        particle.rotation += particle.angular_velocity * dt;
        
        // Update scale and alpha based on age
        let age_ratio = particle.age / particle.lifetime;
        particle.color.set_a(1.0 - age_ratio); // Fade out over lifetime
    }
}

/// System to update GPU buffers
fn update_particle_buffers(
    pool: Res<ParticlePool>,
    // TODO: Implement custom render pipeline
    // mut render_assets: ResMut<Assets<ParticleRenderData>>,
) {
    // Group active particles by texture
    let mut batches: HashMap<u32, Vec<ParticleInstance>> = HashMap::new();
    
    for (i, particle) in pool.particles.iter().enumerate() {
        if particle.age < particle.lifetime && !pool.free_indices.contains(&i) {
            batches.entry(particle.texture_index)
                .or_insert_with(Vec::new)
                .push(*particle);
        }
    }
    
    // Update render data for each batch
    for (texture_index, instances) in batches {
        // Create or update render data asset
        // This would be consumed by the custom render pipeline
    }
}

/// System to render particle batches
fn render_particle_batches(
    // This would integrate with Bevy's render graph
    // For now, we'll use sprite batching
    mut commands: Commands,
    pool: Res<ParticlePool>,
    textures: Res<ParticleTextures>,
) {
    // Implementation would involve custom render pipeline
    // For Phase 4, we're preparing the architecture
}

/// System to cleanup inactive particles
fn cleanup_inactive_particles(
    mut pool: ResMut<ParticlePool>,
) {
    let mut to_free = Vec::new();
    
    // First pass: identify particles to free
    for i in 0..pool.particles.len() {
        if pool.particles[i].age >= pool.particles[i].lifetime && !pool.free_indices.contains(&i) {
            to_free.push(i);
        }
    }
    
    // Second pass: reset particles
    for &i in &to_free {
        pool.particles[i] = ParticleInstance {
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            acceleration: Vec3::ZERO,
            rotation: 0.0,
            angular_velocity: 0.0,
            scale: Vec2::ONE,
            color: Color::WHITE,
            lifetime: 0.0,
            age: 0.0,
            texture_index: 0,
        };
    }
    
    // Free the indices
    pool.free(to_free);
}

// Helper functions

pub fn calculate_lod_factor(distance: f32, lod_bias: f32) -> f32 {
    let base_factor = match distance {
        d if d < 50.0 => 1.0,
        d if d < 100.0 => 0.75,
        d if d < 200.0 => 0.5,
        d if d < 400.0 => 0.25,
        _ => 0.0,
    };
    
    (base_factor * lod_bias).clamp(0.0, 1.0)
}

fn calculate_spawn_position(pattern: &SpawnPattern, origin: Vec3) -> Vec3 {
    let mut rng = rand::thread_rng();
    
    match pattern {
        SpawnPattern::Point => origin,
        SpawnPattern::Circle { radius } => {
            if *radius > 0.0 {
                let angle = rng.gen_range(0.0..std::f32::consts::TAU);
                let r = rng.gen_range(0.0..*radius);
                origin + Vec3::new(r * angle.cos(), 0.0, r * angle.sin())
            } else {
                origin
            }
        }
        SpawnPattern::Sphere { radius } => {
            if *radius > 0.0 {
                let theta = rng.gen_range(0.0..std::f32::consts::TAU);
                let phi = rng.gen_range(0.0..std::f32::consts::PI);
                let r = rng.gen_range(0.0..*radius);
                origin + Vec3::new(
                    r * phi.sin() * theta.cos(),
                    r * phi.cos(),
                    r * phi.sin() * theta.sin(),
                )
            } else {
                origin
            }
        }
        SpawnPattern::Box { size } => {
            origin + Vec3::new(
                if size.x > 0.0 { rng.gen_range(-size.x/2.0..size.x/2.0) } else { 0.0 },
                if size.y > 0.0 { rng.gen_range(-size.y/2.0..size.y/2.0) } else { 0.0 },
                if size.z > 0.0 { rng.gen_range(-size.z/2.0..size.z/2.0) } else { 0.0 },
            )
        }
        _ => origin,
    }
}

fn calculate_initial_velocity(distribution: &VelocityDistribution) -> Vec3 {
    let mut rng = rand::thread_rng();
    
    match distribution {
        VelocityDistribution::Constant(vel) => *vel,
        VelocityDistribution::Random { min, max } => Vec3::new(
            if min.x < max.x { rng.gen_range(min.x..max.x) } else { min.x },
            if min.y < max.y { rng.gen_range(min.y..max.y) } else { min.y },
            if min.z < max.z { rng.gen_range(min.z..max.z) } else { min.z },
        ),
        VelocityDistribution::Radial { speed } => {
            let theta = rng.gen_range(0.0..std::f32::consts::TAU);
            let phi = rng.gen_range(0.0..std::f32::consts::PI);
            Vec3::new(
                speed * phi.sin() * theta.cos(),
                speed * phi.cos(),
                speed * phi.sin() * theta.sin(),
            )
        }
        VelocityDistribution::Cone { direction, angle, speed } => {
            let cone_angle = rng.gen_range(0.0..*angle);
            let rotation = rng.gen_range(0.0..std::f32::consts::TAU);
            
            // Create perpendicular vectors
            let right = direction.cross(Vec3::Y).normalize();
            let up = direction.cross(right).normalize();
            
            // Calculate cone direction
            let offset = cone_angle.cos() * direction.normalize()
                + cone_angle.sin() * (rotation.cos() * right + rotation.sin() * up);
            
            offset.normalize() * *speed
        }
    }
}

fn apply_effect_properties(particle: &mut ParticleInstance, effect_type: ParticleEffectType) {
    match effect_type {
        ParticleEffectType::Heart => {
            particle.acceleration = Vec3::new(0.0, -20.0, 0.0); // Float upward
            particle.scale = Vec2::splat(0.5);
            particle.color = Color::rgb(1.0, 0.3, 0.3);
            particle.angular_velocity = 0.5;
        }
        ParticleEffectType::Rain => {
            particle.acceleration = Vec3::new(0.0, -300.0, 0.0); // Fall fast
            particle.scale = Vec2::new(0.2, 1.0); // Stretched
            particle.color = Color::rgba(0.6, 0.6, 1.0, 0.7);
        }
        ParticleEffectType::Snow => {
            particle.acceleration = Vec3::new(0.0, -30.0, 0.0); // Fall slowly
            particle.scale = Vec2::splat(0.3);
            particle.color = Color::WHITE;
            particle.angular_velocity = 1.0;
        }
        ParticleEffectType::Sparkle => {
            particle.acceleration = Vec3::ZERO;
            particle.scale = Vec2::splat(0.3);
            particle.color = Color::rgb(1.0, 1.0, 0.3);
            particle.angular_velocity = 3.0;
        }
        _ => {
            particle.acceleration = Vec3::new(0.0, -50.0, 0.0);
            particle.scale = Vec2::splat(0.5);
            particle.color = Color::WHITE;
        }
    }
    
    particle.texture_index = effect_type as u32;
}

impl ParticleEffectType {
    fn from_index(index: u32) -> Option<Self> {
        match index {
            0 => Some(Self::Heart),
            1 => Some(Self::Zzz),
            2 => Some(Self::Sparkle),
            3 => Some(Self::Sweat),
            4 => Some(Self::Exclamation),
            5 => Some(Self::Question),
            6 => Some(Self::Rain),
            7 => Some(Self::Snow),
            8 => Some(Self::Fog),
            9 => Some(Self::Wind),
            10 => Some(Self::Lightning),
            11 => Some(Self::Impact),
            12 => Some(Self::Footstep),
            13 => Some(Self::Splash),
            14 => Some(Self::Dust),
            15 => Some(Self::Leaves),
            16 => Some(Self::Fireflies),
            17 => Some(Self::Pollen),
            18 => Some(Self::Smoke),
            _ => None,
        }
    }
}

// Placeholder for weather state
#[derive(Resource, Default)]
pub struct WeatherState {
    pub wind_strength: f32,
    pub wind_direction: Vec3,
}

impl WeatherState {
    pub fn wind_vector(&self) -> Vec3 {
        self.wind_direction.normalize() * self.wind_strength
    }
}

// TODO: Placeholder for custom render data
// struct ParticleRenderData;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_particle_pool_allocation() {
        let mut pool = ParticlePool::new(100);
        
        // Test allocation
        let indices = pool.allocate(10).unwrap();
        assert_eq!(indices.len(), 10);
        assert_eq!(pool.active_count, 10);
        assert_eq!(pool.free_indices.len(), 90);
        
        // Test freeing
        pool.free(indices);
        assert_eq!(pool.active_count, 0);
        assert_eq!(pool.free_indices.len(), 100);
    }
    
    #[test]
    fn test_lod_factor_calculation() {
        assert_eq!(calculate_lod_factor(25.0, 1.0), 1.0);
        assert_eq!(calculate_lod_factor(75.0, 1.0), 0.75);
        assert_eq!(calculate_lod_factor(150.0, 1.0), 0.5);
        assert_eq!(calculate_lod_factor(300.0, 1.0), 0.25);
        assert_eq!(calculate_lod_factor(500.0, 1.0), 0.0);
        
        // Test with bias
        assert_eq!(calculate_lod_factor(75.0, 0.5), 0.375);
        assert_eq!(calculate_lod_factor(75.0, 2.0), 1.0); // Clamped to 1.0
    }
}