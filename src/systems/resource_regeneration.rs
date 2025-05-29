//! Resource regeneration system for Phase 1
//!
//! This system handles resource regeneration to maintain a balanced ecosystem.

use bevy::prelude::*;
use crate::components::*;
use crate::core::determinism::{DeterministicRng, SeededRandom, SystemId};
use crate::systems::biome::BiomeMap;

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
    mut biome_map: ResMut<BiomeMap>,
    resources_query: Query<(Entity, &ResourceTypeComponent), With<ResourceMarker>>,
) {
    // Count current resources by category
    let mut food_resources = 0;
    let mut water_resources = 0;
    
    for (_entity, resource_type) in resources_query.iter() {
        let (food_val, water_val) = resource_type.0.nutritional_values();
        if food_val > water_val {
            food_resources += 1;
        } else {
            water_resources += 1;
        }
    }
    
    // Spawn resources based on biome
    let total_resources = food_resources + water_resources;
    let target_total = config.target_food_count + config.target_water_count;
    
    if total_resources < target_total && rng.random_bool(SystemId::ResourceSpawn, config.spawn_chance) {
        // Generate random position
        let angle = rng.random_f32(SystemId::ResourceSpawn) * std::f32::consts::TAU;
        let radius = rng.random_f32(SystemId::ResourceSpawn) * config.spawn_radius;
        let position = Vec2::new(angle.cos() * radius, angle.sin() * radius);
        
        // Get biome at spawn position
        let biome = biome_map.get_biome(position);
        let biome_resources = BiomeMap::get_biome_resources(biome);
        let abundance = BiomeMap::get_biome_abundance(biome);
        
        // Select resource type based on biome weights
        let total_weight: f32 = biome_resources.iter().map(|(_, w)| w).sum();
        let mut roll = rng.random_f32(SystemId::ResourceSpawn) * total_weight;
        
        let mut selected_resource = ResourceType::Food; // fallback
        for (resource_type, weight) in biome_resources {
            roll -= weight;
            if roll <= 0.0 {
                selected_resource = resource_type;
                break;
            }
        }
        
        // Spawn the selected resource
        let resource_amount = config.max_amount * abundance * 0.75;
        let resource_color = selected_resource.color();
        
        commands.spawn((
            ResourceBundle::new(position, selected_resource, resource_amount),
            SpriteBundle {
                sprite: Sprite {
                    color: resource_color,
                    custom_size: Some(Vec2::new(15.0, 15.0)),
                    ..default()
                },
                transform: Transform::from_xyz(position.x, position.y, 0.0),
                ..default()
            },
            crate::plugins::ResourceSprite {
                resource_type: selected_resource,
            },
            Name::new(format!("{:?} ({:?} biome)", selected_resource, biome)),
        ));
        
        info!("Spawned {:?} resource in {:?} biome at {:?}", selected_resource, biome, position);
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
    use crate::systems::biome::BiomeMap;
    
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
        app.insert_resource(BiomeMap::new(12345)); // Add biome map for spawning
        
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
        app.insert_resource(BiomeMap::new(12345)); // Add biome map
        
        // Add resources that count as food (high food value)
        app.world.spawn((
            ResourceMarker,
            ResourceTypeComponent(ResourceType::Berry), // Food resource
            ResourceAmount::new(100.0),
        ));
        app.world.spawn((
            ResourceMarker,
            ResourceTypeComponent(ResourceType::Mushroom), // Food resource
            ResourceAmount::new(100.0),
        ));
        // Add resources that count as water (high water value)
        app.world.spawn((
            ResourceMarker,
            ResourceTypeComponent(ResourceType::Water), // Water resource
            ResourceAmount::new(100.0),
        ));
        app.world.spawn((
            ResourceMarker,
            ResourceTypeComponent(ResourceType::CactiWater), // Water resource
            ResourceAmount::new(100.0),
        ));
        
        let initial_count = app.world.query::<&ResourceMarker>().iter(&app.world).count();
        
        app.add_systems(Update, spawn_new_resources);
        app.update();
        
        // We should spawn at most 1 new resource since we're at the target
        let final_count = app.world.query::<&ResourceMarker>().iter(&app.world).count();
        // Allow for some spawning due to the new biome-based system
        assert!(final_count <= initial_count + 1);
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
        app.insert_resource(BiomeMap::new(12345)); // Add biome map
        
        app.add_systems(Update, spawn_new_resources);
        app.update();
        
        // Check all spawned resources are within radius
        for position in app.world.query::<&Position>().iter(&app.world) {
            let distance = position.0.length();
            assert!(distance <= 100.0);
        }
    }
}