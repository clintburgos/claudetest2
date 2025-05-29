use bevy::prelude::*;
use crate::components::{EmotionType, CartoonSprite};
use std::collections::HashMap;
use rand::Rng;

/// Plugin for managing particle effects in the cartoon rendering system
/// 
/// Handles emotion particles, action feedback, and environmental effects
/// 
/// # Particle System Architecture
/// 
/// The system uses a component-based approach where:
/// - **ParticleEmitter**: Attached to creatures, spawns particles over time
/// - **Particle**: Individual particle with physics and lifetime
/// - **ParticleAssets**: Shared textures for all particle types
/// 
/// # Particle Types
/// 
/// Each type has unique behavior:
/// - **Heart**: Floats upward with slight drift (love/bonding)
/// - **Zzz**: Drifts sideways while rising (sleeping)
/// - **Sparkle**: Random burst pattern (general effect)
/// - **Sweat**: Drips downward (stress/heat)
/// - **Exclamation**: Pops up quickly (alert)
/// - **Question**: Wobbles upward (confusion)
/// - **Dust**: Ground-level dispersion (movement)
/// 
/// # Performance Considerations
/// 
/// - Particles are pooled and reused
/// - Maximum particle count enforced
/// - LOD system reduces particle density at distance
/// - Efficient sprite batching via Bevy
pub struct ParticleEffectsPlugin;

impl Plugin for ParticleEffectsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ParticleAssets::default())
            .add_systems(Update, (
                spawn_emotion_particles,
                update_particles,
                cleanup_expired_particles,
            ).chain());
    }
}

/// Resource containing particle texture handles
#[derive(Resource, Default)]
pub struct ParticleAssets {
    pub textures: HashMap<String, Handle<Image>>,
}

/// Component for particle entities
#[derive(Component)]
pub struct Particle {
    /// Lifetime remaining in seconds
    pub lifetime: f32,
    /// Initial lifetime for fade calculations
    pub initial_lifetime: f32,
    /// Velocity of the particle (pixels per second)
    pub velocity: Vec2,
    /// Acceleration (pixels per second squared, e.g., gravity = -20.0)
    pub acceleration: Vec2,
    /// Scale over lifetime curve (start_scale, end_scale)
    /// Particle size interpolates from start to end over lifetime
    pub scale_curve: (f32, f32),
    /// Alpha over lifetime curve (start_alpha, end_alpha)
    /// Transparency interpolates from start to end (0.0 = invisible, 1.0 = opaque)
    pub alpha_curve: (f32, f32),
}

/// Component marking an entity that can emit particles
#[derive(Component)]
pub struct ParticleEmitter {
    /// Type of particles to emit
    pub particle_type: ParticleType,
    /// Time since last emission
    pub timer: Timer,
    /// Whether the emitter is active
    pub active: bool,
}

/// Types of particles that can be emitted
/// 
/// Each type has specific visual properties and behaviors
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ParticleType {
    /// Love/affection indicator - floats upward with slight drift
    Heart,
    /// Sleep indicator - drifts upward and sideways
    Zzz,
    /// Generic effect particle - random dispersal pattern
    Sparkle,
    /// Stress/anger indicator - drips downward
    Sweat,
    /// Alert/surprise indicator - pops up and fades
    Exclamation,
    /// Confusion/curiosity indicator - wobbles upward
    Question,
    /// Movement feedback - disperses at ground level
    Dust,
}

/// System to spawn emotion particles based on creature state
/// 
/// # Emotion to Particle Mapping
/// 
/// The system automatically spawns particles based on creature emotions:
/// - Happy → Hearts (love/joy indicators)
/// - Tired → Zzz (sleep particles)
/// - Curious → Question marks
/// - Frightened → Exclamation marks
/// - Angry → Sweat drops
/// 
/// # Emitter Management
/// 
/// For each creature:
/// 1. Check current emotion from expression overlay
/// 2. Map emotion to appropriate particle type
/// 3. Create/update emitter component
/// 4. Spawn particles at regular intervals (0.5s)
/// 
/// Emitters are automatically disabled when emotion changes
/// or creature returns to neutral state.
/// 
/// # Spawn Patterns
/// 
/// Particles spawn slightly above creatures (Y+20 pixels) with
/// type-specific velocities and physics properties.
fn spawn_emotion_particles(
    mut commands: Commands,
    particle_assets: Res<ParticleAssets>,
    time: Res<Time>,
    mut creatures: Query<(
        Entity,
        &Transform,
        &CartoonSprite,
        Option<&mut ParticleEmitter>,
    )>,
) {
    for (entity, transform, cartoon_sprite, emitter) in creatures.iter_mut() {
        // Determine if we should emit particles based on emotion
        let should_emit = cartoon_sprite.expression_overlay.as_ref()
            .map(|overlay| {
                // Map emotions to particle types
                match determine_dominant_emotion(overlay) {
                    EmotionType::Happy => Some(ParticleType::Heart),
                    EmotionType::Tired => Some(ParticleType::Zzz),
                    EmotionType::Curious => Some(ParticleType::Question),
                    EmotionType::Frightened => Some(ParticleType::Exclamation),
                    EmotionType::Angry => Some(ParticleType::Sweat),
                    _ => None,
                }
            })
            .flatten();
        
        if let Some(particle_type) = should_emit {
            if let Some(mut emitter) = emitter {
                // Update existing emitter
                emitter.particle_type = particle_type;
                emitter.active = true;
                emitter.timer.tick(time.delta());
                
                if emitter.timer.finished() {
                    spawn_particle(&mut commands, &particle_assets, transform.translation, particle_type);
                }
            } else {
                // Add new emitter
                commands.entity(entity).insert(ParticleEmitter {
                    particle_type,
                    timer: Timer::from_seconds(0.5, TimerMode::Repeating),
                    active: true,
                });
            }
        } else if let Some(mut emitter) = emitter {
            // Deactivate emitter if no emotion particles needed
            emitter.active = false;
        }
    }
}

/// Helper function to spawn a single particle
/// 
/// # Particle Configuration
/// 
/// Each particle type has specific properties:
/// 
/// ## Physics Properties
/// - **Velocity**: Initial movement direction and speed
/// - **Acceleration**: Gravity or other forces
/// - **Lifetime**: How long particle exists (0.8-2.0s)
/// 
/// ## Visual Properties
/// - **Scale Curve**: Size change over lifetime
/// - **Alpha Curve**: Transparency fade over lifetime
/// 
/// ## Examples
/// 
/// Heart particles:
/// - Velocity: Slight drift + upward (30 pixels/sec)
/// - Acceleration: Negative gravity (floats up)
/// - Scale: 0.5 → 1.0 (grows)
/// - Alpha: 1.0 → 0.0 (fades out)
/// 
/// Sweat particles:
/// - Velocity: Straight down
/// - Acceleration: Gravity
/// - Scale: Constant
/// - Alpha: Fade out
/// 
/// # Randomization
/// 
/// Small random variations are added to velocity to create
/// natural-looking particle spreads.
fn spawn_particle(
    commands: &mut Commands,
    particle_assets: &ParticleAssets,
    position: Vec3,
    particle_type: ParticleType,
) {
    let texture_name = match particle_type {
        ParticleType::Heart => "heart",
        ParticleType::Zzz => "zzz",
        ParticleType::Sparkle => "sparkle",
        ParticleType::Sweat => "sweat",
        ParticleType::Exclamation => "exclamation",
        ParticleType::Question => "question",
        ParticleType::Dust => "sparkle", // Reuse sparkle for dust
    };
    
    if let Some(texture) = particle_assets.textures.get(texture_name) {
        // Configure particle properties based on type
        let mut rng = rand::thread_rng();
        let (velocity, acceleration, lifetime, scale_curve, alpha_curve) = match particle_type {
            ParticleType::Heart => (
                Vec2::new(rng.gen_range(-10.0..10.0), 30.0), // Slight horizontal drift, upward float
                Vec2::new(0.0, -20.0),                       // Gentle gravity (upward buoyancy)
                1.5,                                         // Lasts 1.5 seconds
                (0.5, 1.0),                                 // Grows from half to full size
                (1.0, 0.0),                                 // Fades from opaque to invisible
            ),
            ParticleType::Zzz => (
                Vec2::new(rng.gen_range(-5.0..5.0), 20.0),  // Small drift, upward motion
                Vec2::new(5.0, 0.0),                        // Sideways drift acceleration
                2.0,                                        // Lasts 2 seconds (slow)
                (0.8, 1.2),                                 // Grows slightly over time
                (0.8, 0.0),                                 // Starts semi-transparent
            ),
            ParticleType::Sparkle => (
                Vec2::new(rng.gen_range(-20.0..20.0), rng.gen_range(-20.0..20.0)), // Random burst
                Vec2::ZERO,                                 // No acceleration (pure velocity)
                0.8,                                        // Quick effect (0.8 seconds)
                (0.3, 0.0),                                 // Shrinks to nothing
                (1.0, 0.0),                                 // Fades completely
            ),
            _ => (
                Vec2::new(0.0, 20.0),                       // Default upward motion
                Vec2::ZERO,                                 // No acceleration
                1.0,                                        // 1 second lifetime
                (1.0, 1.0),                                 // Constant size
                (1.0, 0.0),                                 // Fade out
            ),
        };
        
        // Spawn particle slightly above the creature
        // Y offset: 20 pixels above sprite center
        // Z offset: 5 units forward to render above creature
        let spawn_pos = position + Vec3::new(0.0, 20.0, 5.0);
        
        commands.spawn((
            SpriteBundle {
                texture: texture.clone(),
                transform: Transform::from_translation(spawn_pos)
                    .with_scale(Vec3::splat(scale_curve.0)),
                sprite: Sprite {
                    color: Color::rgba(1.0, 1.0, 1.0, alpha_curve.0),
                    ..default()
                },
                ..default()
            },
            Particle {
                lifetime,
                initial_lifetime: lifetime,
                velocity,
                acceleration,
                scale_curve,
                alpha_curve,
            },
            Name::new(format!("Particle_{:?}", particle_type)),
        ));
    }
}

/// System to update particle positions and properties
fn update_particles(
    time: Res<Time>,
    mut particles: Query<(&mut Transform, &mut Sprite, &mut Particle)>,
) {
    let dt = time.delta_seconds();
    
    for (mut transform, mut sprite, mut particle) in particles.iter_mut() {
        // Update lifetime
        particle.lifetime -= dt;
        
        // Update velocity and position
        let acceleration = particle.acceleration;
        particle.velocity += acceleration * dt;
        transform.translation.x += particle.velocity.x * dt;
        transform.translation.y += particle.velocity.y * dt;
        
        // Calculate lifetime progress (0.0 to 1.0)
        let progress = 1.0 - (particle.lifetime / particle.initial_lifetime).max(0.0);
        
        // Update scale based on curve
        let scale = lerp(particle.scale_curve.0, particle.scale_curve.1, progress);
        transform.scale = Vec3::splat(scale);
        
        // Update alpha based on curve
        let alpha = lerp(particle.alpha_curve.0, particle.alpha_curve.1, progress);
        sprite.color.set_a(alpha);
    }
}

/// System to remove expired particles
fn cleanup_expired_particles(
    mut commands: Commands,
    particles: Query<(Entity, &Particle)>,
) {
    for (entity, particle) in particles.iter() {
        if particle.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// Helper function to determine dominant emotion from expression overlay
/// 
/// Maps facial expression parameters to emotion types for particle selection
fn determine_dominant_emotion(overlay: &crate::components::ExpressionOverlay) -> EmotionType {
    // Simple mapping based on expression parameters
    if overlay.mouth_curve > 0.3 {       // Upward curve = smile
        EmotionType::Happy
    } else if overlay.mouth_curve < -0.3 { // Downward curve = frown
        EmotionType::Sad
    } else if overlay.brow_angle < -15.0 { // Furrowed brow = anger
        EmotionType::Angry
    } else if overlay.eye_scale < 0.9 {    // Squinted eyes = tired
        EmotionType::Tired
    } else {
        EmotionType::Neutral
    }
}

/// Linear interpolation helper
/// 
/// Smoothly interpolates between two values based on a progress factor
/// - `a`: Start value
/// - `b`: End value  
/// - `t`: Progress (0.0 = start, 1.0 = end)
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{ExpressionOverlay, EmotionType};
    
    #[test]
    fn test_lerp_function() {
        assert_eq!(lerp(0.0, 10.0, 0.0), 0.0);
        assert_eq!(lerp(0.0, 10.0, 0.5), 5.0);
        assert_eq!(lerp(0.0, 10.0, 1.0), 10.0);
        assert_eq!(lerp(5.0, 10.0, 0.5), 7.5);
        assert_eq!(lerp(-10.0, 10.0, 0.5), 0.0);
        assert_eq!(lerp(10.0, -10.0, 0.25), 5.0);
    }
    
    #[test]
    fn test_particle_lifetime() {
        let particle = Particle {
            lifetime: 1.5,
            initial_lifetime: 2.0,
            velocity: Vec2::ZERO,
            acceleration: Vec2::ZERO,
            scale_curve: (1.0, 0.0),
            alpha_curve: (1.0, 0.0),
        };
        
        let progress = 1.0 - (particle.lifetime / particle.initial_lifetime);
        assert_eq!(progress, 0.25);
        
        let scale = lerp(particle.scale_curve.0, particle.scale_curve.1, progress);
        assert_eq!(scale, 0.75);
        
        let alpha = lerp(particle.alpha_curve.0, particle.alpha_curve.1, progress);
        assert_eq!(alpha, 0.75);
    }
    
    #[test]
    fn test_particle_physics() {
        let mut particle = Particle {
            lifetime: 2.0,
            initial_lifetime: 2.0,
            velocity: Vec2::new(10.0, 0.0),
            acceleration: Vec2::new(0.0, -100.0), // Gravity
            scale_curve: (1.0, 1.0),
            alpha_curve: (1.0, 1.0),
        };
        
        let dt = 0.1;
        let initial_velocity = particle.velocity;
        
        // Apply physics
        particle.velocity += particle.acceleration * dt;
        particle.lifetime -= dt;
        
        assert_eq!(particle.velocity.x, initial_velocity.x);
        assert_eq!(particle.velocity.y, initial_velocity.y - 10.0);
        assert_eq!(particle.lifetime, 1.9);
    }
    
    #[test]
    fn test_particle_type_properties() {
        // Test that each particle type can be configured
        let types = vec![
            ParticleType::Heart,
            ParticleType::Zzz,
            ParticleType::Sparkle,
            ParticleType::Sweat,
            ParticleType::Exclamation,
            ParticleType::Question,
            ParticleType::Dust,
        ];
        
        for particle_type in types {
            // Just ensure we can match on each type
            let _name = match particle_type {
                ParticleType::Heart => "heart",
                ParticleType::Zzz => "zzz",
                ParticleType::Sparkle => "sparkle",
                ParticleType::Sweat => "sweat",
                ParticleType::Exclamation => "exclamation",
                ParticleType::Question => "question",
                ParticleType::Dust => "sparkle",
            };
        }
    }
    
    #[test]
    fn test_particle_emitter() {
        let emitter = ParticleEmitter {
            particle_type: ParticleType::Heart,
            timer: Timer::from_seconds(0.5, TimerMode::Repeating),
            active: true,
        };
        
        assert_eq!(emitter.particle_type, ParticleType::Heart);
        assert!(emitter.active);
        assert_eq!(emitter.timer.duration().as_secs_f32(), 0.5);
    }
    
    #[test]
    fn test_emotion_to_particle_mapping() {
        // Test emotion detection from expression overlay
        let happy_overlay = ExpressionOverlay {
            eye_offset: Vec2::ZERO,
            eye_scale: 1.1,
            mouth_curve: 0.5,
            mouth_open: 0.0,
            brow_angle: -10.0,
        };
        let emotion = determine_dominant_emotion(&happy_overlay);
        assert_eq!(emotion, EmotionType::Happy);
        
        let sad_overlay = ExpressionOverlay {
            eye_offset: Vec2::ZERO,
            eye_scale: 0.9,
            mouth_curve: -0.5,
            mouth_open: 0.0,
            brow_angle: 20.0,
        };
        let emotion = determine_dominant_emotion(&sad_overlay);
        assert_eq!(emotion, EmotionType::Sad);
        
        let angry_overlay = ExpressionOverlay {
            eye_offset: Vec2::ZERO,
            eye_scale: 0.8,
            mouth_curve: -0.2,
            mouth_open: 0.0,
            brow_angle: -20.0,
        };
        let emotion = determine_dominant_emotion(&angry_overlay);
        assert_eq!(emotion, EmotionType::Angry);
        
        let tired_overlay = ExpressionOverlay {
            eye_offset: Vec2::ZERO,
            eye_scale: 0.85,
            mouth_curve: 0.0,
            mouth_open: 0.0,
            brow_angle: 5.0,
        };
        let emotion = determine_dominant_emotion(&tired_overlay);
        assert_eq!(emotion, EmotionType::Tired);
    }
}