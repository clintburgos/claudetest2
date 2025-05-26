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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::determinism::DeterministicRng;
    
    #[test]
    fn test_resource_regeneration_config_default() {
        let config = ResourceRegenerationConfig::default();
        assert_eq!(config.check_interval, 1.0);
        assert_eq!(config.base_regen_rate, 5.0);
        assert_eq!(config.max_amount, 200.0);
        assert_eq!(config.min_amount, 5.0);
        assert_eq!(config.spawn_chance, 0.1);
        assert_eq!(config.target_food_count, 150);
        assert_eq!(config.target_water_count, 150);
        assert_eq!(config.spawn_radius, 400.0);
    }
    
    #[test]
    fn test_resource_regeneration_timer() {
        let timer = ResourceRegenerationTimer(Timer::from_seconds(1.0, TimerMode::Repeating));
        assert_eq!(timer.0.duration().as_secs_f32(), 1.0);
        assert_eq!(timer.0.mode(), TimerMode::Repeating);
    }
    
    #[test]
    fn test_regeneration_increases_resources() {
        let config = ResourceRegenerationConfig::default();
        let mut amount = ResourceAmount::new(50.0);
        
        // Simulate regeneration
        let regen_amount = config.base_regen_rate * config.check_interval;
        amount.current = (amount.current + regen_amount).min(config.max_amount);
        
        // Check resource was regenerated
        assert!(amount.current > 50.0);
        assert_eq!(amount.current, 55.0); // 50 + 5
    }
    
    #[test]
    fn test_regeneration_respects_max_amount() {
        let config = ResourceRegenerationConfig {
            max_amount: 100.0,
            base_regen_rate: 50.0,
            check_interval: 1.0,
            ..default()
        };
        
        let mut amount = ResourceAmount::new(95.0);
        
        // Simulate regeneration
        let regen_amount = config.base_regen_rate * config.check_interval;
        amount.current = (amount.current + regen_amount).min(config.max_amount);
        
        // Check resource is capped at max
        assert_eq!(amount.current, 100.0);
    }
    
    #[test]
    fn test_depleted_resource_removal() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<crate::plugins::ResourceDepletedEvent>();
        app.insert_resource(ResourceRegenerationConfig {
            min_amount: 10.0,
            ..default()
        });
        
        // Create depleted resources
        let depleted = app.world.spawn((
            ResourceMarker,
            ResourceAmount::new(5.0),
        )).id();
        
        let healthy = app.world.spawn((
            ResourceMarker,
            ResourceAmount::new(50.0),
        )).id();
        
        app.add_systems(Update, remove_depleted_resources);
        app.update();
        
        // Check depleted was removed
        assert!(app.world.get_entity(depleted).is_none());
        assert!(app.world.get_entity(healthy).is_some());
        
        // Check event was sent
        let events = app.world.resource::<Events<crate::plugins::ResourceDepletedEvent>>();
        assert_eq!(events.len(), 1);
    }
    
    #[test]
    fn test_resource_spawn_probability() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(DeterministicRng::new(12345));
        app.insert_resource(ResourceRegenerationConfig {
            spawn_chance: 1.0, // Always spawn
            target_food_count: 10,
            target_water_count: 10,
            ..default()
        });
        
        // Start with no resources
        app.add_systems(Update, spawn_new_resources);
        app.update();
        
        // Check that resources were spawned
        let resources = app.world.query::<&ResourceTypeComponent>().iter(&app.world).count();
        assert!(resources > 0);
    }
    
    #[test]
    fn test_resource_count_limits() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(DeterministicRng::new(12345));
        app.insert_resource(ResourceRegenerationConfig {
            spawn_chance: 1.0,
            target_food_count: 2,
            target_water_count: 2,
            ..default()
        });
        
        // Add exactly target count of resources
        app.world.spawn((
            ResourceMarker,
            ResourceTypeComponent(ResourceType::Food),
            ResourceAmount::new(100.0),
        ));
        app.world.spawn((
            ResourceMarker,
            ResourceTypeComponent(ResourceType::Food),
            ResourceAmount::new(100.0),
        ));
        app.world.spawn((
            ResourceMarker,
            ResourceTypeComponent(ResourceType::Water),
            ResourceAmount::new(100.0),
        ));
        app.world.spawn((
            ResourceMarker,
            ResourceTypeComponent(ResourceType::Water),
            ResourceAmount::new(100.0),
        ));
        
        let initial_count = app.world.query::<&ResourceMarker>().iter(&app.world).count();
        
        app.add_systems(Update, spawn_new_resources);
        app.update();
        
        // No new resources should be spawned
        let final_count = app.world.query::<&ResourceMarker>().iter(&app.world).count();
        assert_eq!(initial_count, final_count);
    }
    
    #[test]
    fn test_spawn_position_within_radius() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(DeterministicRng::new(12345));
        app.insert_resource(ResourceRegenerationConfig {
            spawn_chance: 1.0,
            target_food_count: 5,
            spawn_radius: 100.0,
            ..default()
        });
        
        app.add_systems(Update, spawn_new_resources);
        app.update();
        
        // Check all spawned resources are within radius
        for position in app.world.query::<&Position>().iter(&app.world) {
            let distance = position.0.length();
            assert!(distance <= 100.0);
        }
    }
}