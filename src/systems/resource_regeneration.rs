//! Resource regeneration system for Phase 1
//!
//! This system handles resource regeneration to maintain a balanced ecosystem.

use bevy::prelude::*;
use crate::components::*;
use crate::core::determinism::{DeterministicRng, SeededRandom, SystemId};

/// Configuration for resource regeneration
#[derive(Resource)]
pub struct ResourceRegenerationConfig {
    /// Minimum time between regeneration checks (seconds)
    pub check_interval: f32,
    /// Base regeneration rate per second
    pub base_regen_rate: f32,
    /// Maximum amount for resources
    pub max_amount: f32,
    /// Minimum amount below which resources despawn
    pub min_amount: f32,
    /// Chance per frame to spawn new resource when below target count
    pub spawn_chance: f32,
    /// Target number of food resources
    pub target_food_count: usize,
    /// Target number of water resources
    pub target_water_count: usize,
    /// Spawn radius from world center
    pub spawn_radius: f32,
}

impl Default for ResourceRegenerationConfig {
    fn default() -> Self {
        Self {
            check_interval: 1.0,
            base_regen_rate: 5.0, // 5 units per second
            max_amount: 200.0,
            min_amount: 5.0,
            spawn_chance: 0.1,
            target_food_count: 150,
            target_water_count: 150,
            spawn_radius: 400.0,
        }
    }
}

/// Timer for regeneration checks
#[derive(Resource, Default)]
pub struct ResourceRegenerationTimer(pub Timer);

/// Plugin for resource regeneration
pub struct ResourceRegenerationPlugin;

impl Plugin for ResourceRegenerationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ResourceRegenerationConfig>()
            .insert_resource(ResourceRegenerationTimer(Timer::from_seconds(1.0, TimerMode::Repeating)))
            .add_systems(Update, (
                regenerate_resources,
                spawn_new_resources,
                remove_depleted_resources,
            ).chain());
    }
}

/// Regenerate existing resources over time
fn regenerate_resources(
    time: Res<Time>,
    config: Res<ResourceRegenerationConfig>,
    mut timer: ResMut<ResourceRegenerationTimer>,
    mut resources: Query<&mut ResourceAmount, With<ResourceMarker>>,
) {
    timer.0.tick(time.delta());
    
    if !timer.0.just_finished() {
        return;
    }
    
    let regen_amount = config.base_regen_rate * config.check_interval;
    
    for mut amount in resources.iter_mut() {
        if amount.current < config.max_amount {
            amount.current = (amount.current + regen_amount).min(config.max_amount);
        }
    }
}

/// Spawn new resources when below target count
fn spawn_new_resources(
    mut commands: Commands,
    mut rng: ResMut<DeterministicRng>,
    config: Res<ResourceRegenerationConfig>,
    resources_query: Query<(Entity, &ResourceTypeComponent), With<ResourceMarker>>,
) {
    // Count current resources by type
    let mut food_count = 0;
    let mut water_count = 0;
    
    for (_entity, resource_type) in resources_query.iter() {
        match resource_type.0 {
            ResourceType::Food => food_count += 1,
            ResourceType::Water => water_count += 1,
        }
    }
    
    // Spawn food if needed
    if food_count < config.target_food_count {
        if rng.random_bool(SystemId::ResourceSpawn, config.spawn_chance) {
            let angle = rng.random_f32(SystemId::ResourceSpawn) * std::f32::consts::TAU;
            let radius = rng.random_f32(SystemId::ResourceSpawn) * config.spawn_radius;
            let position = Vec2::new(angle.cos() * radius, angle.sin() * radius);
            
            commands.spawn((
                ResourceBundle::new(position, ResourceType::Food, config.max_amount * 0.75),
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::rgb(0.8, 0.6, 0.2),
                        custom_size: Some(Vec2::new(15.0, 15.0)),
                        ..default()
                    },
                    transform: Transform::from_xyz(position.x, position.y, 0.0),
                    ..default()
                },
                crate::plugins::ResourceSprite {
                    resource_type: ResourceType::Food,
                },
                Name::new("Food (Regenerated)"),
            ));
            
            info!("Spawned new food resource at {:?}", position);
        }
    }
    
    // Spawn water if needed
    if water_count < config.target_water_count {
        if rng.random_bool(SystemId::ResourceSpawn, config.spawn_chance) {
            let angle = rng.random_f32(SystemId::ResourceSpawn) * std::f32::consts::TAU;
            let radius = rng.random_f32(SystemId::ResourceSpawn) * config.spawn_radius;
            let position = Vec2::new(angle.cos() * radius, angle.sin() * radius);
            
            commands.spawn((
                ResourceBundle::new(position, ResourceType::Water, config.max_amount * 0.75),
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::rgb(0.2, 0.6, 0.8),
                        custom_size: Some(Vec2::new(15.0, 15.0)),
                        ..default()
                    },
                    transform: Transform::from_xyz(position.x, position.y, 0.0),
                    ..default()
                },
                crate::plugins::ResourceSprite {
                    resource_type: ResourceType::Water,
                },
                Name::new("Water (Regenerated)"),
            ));
            
            info!("Spawned new water resource at {:?}", position);
        }
    }
}

/// Remove resources that are too depleted
fn remove_depleted_resources(
    mut commands: Commands,
    config: Res<ResourceRegenerationConfig>,
    resources: Query<(Entity, &ResourceAmount), With<ResourceMarker>>,
    mut events: EventWriter<crate::plugins::ResourceDepletedEvent>,
) {
    for (entity, amount) in resources.iter() {
        if amount.current < config.min_amount {
            commands.entity(entity).despawn();
            events.send(crate::plugins::ResourceDepletedEvent { entity });
        }
    }
}