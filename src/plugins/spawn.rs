use crate::components::ResourceType;
use crate::components::*;
use crate::core::determinism::DeterministicRng;
use crate::plugins::{CreatureSprite, ResourceSprite};
use crate::systems::biome::BiomeMap;
use bevy::prelude::*;

pub struct SpawnPlugin;

impl Plugin for SpawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_initial_entities);
    }
}

/// Spawn initial entities with biome-aware resource placement
fn spawn_initial_entities(
    mut commands: Commands, 
    mut rng: ResMut<DeterministicRng>,
    mut biome_map: Option<ResMut<BiomeMap>>,
) {
    // Spawn initial creatures - Phase 1 requires 500 creatures
    let creature_count = if cfg!(debug_assertions) { 50 } else { 500 };
    let world_size = 1000.0; // Larger world for more creatures

    for i in 0..creature_count {
        // Spread creatures more evenly across the larger world
        let grid_size = (creature_count as f32).sqrt().ceil() as i32;
        let x = (i as i32 % grid_size) as f32 * (world_size / grid_size as f32) - world_size / 2.0;
        let y = (i as i32 / grid_size) as f32 * (world_size / grid_size as f32) - world_size / 2.0;
        let position = Vec2::new(x, y);

        commands.spawn((
            CreatureBundle::new(position, 10.0),
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.3, 0.7, 0.3),
                    custom_size: Some(Vec2::new(20.0, 20.0)),
                    ..default()
                },
                transform: Transform::from_xyz(position.x, position.y, 1.0),
                ..default()
            },
            CreatureSprite,
            Name::new(format!("Creature {}", i)),
        ));
    }

    // Spawn initial resources - biome-aware placement
    let resource_count = creature_count / 2; // More resources for more creatures
    let mut spawned_resources = 0;

    // Try to spawn resources in appropriate biomes
    for _ in 0..resource_count * 4 { // Try more positions to ensure good distribution
        // Generate random position
        let x = rng.gen_range(crate::core::determinism::SystemId::ResourceSpawn, -world_size / 2.0, world_size / 2.0);
        let y = rng.gen_range(crate::core::determinism::SystemId::ResourceSpawn, -world_size / 2.0, world_size / 2.0);
        let position = Vec2::new(x, y);
        
        // Get biome at this position if biome system is available
        let (biome, biome_resources, abundance) = if let Some(biome_map) = biome_map.as_mut() {
            let biome = biome_map.get_biome(position);
            let resources = BiomeMap::get_biome_resources(biome);
            let abundance = BiomeMap::get_biome_abundance(biome);
            (Some(biome), resources, abundance)
        } else {
            // Fallback to default resource distribution
            let resources = vec![
                (ResourceType::Food, 0.5),
                (ResourceType::Water, 0.5),
            ];
            (None, resources, 1.0)
        };
        
        // Skip if no resources in this biome
        if biome_resources.is_empty() {
            continue;
        }
        
        // Choose resource type based on biome weights
        let total_weight: f32 = biome_resources.iter().map(|(_, w)| w).sum();
        let mut roll = rng.gen_range(crate::core::determinism::SystemId::ResourceSpawn, 0.0, total_weight);
        let mut chosen_resource = ResourceType::Food; // Default
        
        for (resource_type, weight) in biome_resources {
            roll -= weight;
            if roll <= 0.0 {
                chosen_resource = resource_type;
                break;
            }
        }
        
        // Determine resource amount based on biome abundance
        let base_amount = 100.0;
        let amount = base_amount * abundance;
        
        // Get color based on resource type and biome
        let color = if let Some(biome) = biome {
            match (chosen_resource, biome) {
                (ResourceType::Food, crate::rendering::BiomeType::Forest) => Color::rgb(0.8, 0.2, 0.8), // Berries (purple)
                (ResourceType::Food, crate::rendering::BiomeType::Desert) => Color::rgb(1.0, 0.6, 0.0), // Desert fruit (orange)
                (ResourceType::Food, crate::rendering::BiomeType::Tundra) => Color::rgb(0.6, 0.8, 1.0), // Snow berries (light blue)
                (ResourceType::Food, _) => Color::rgb(0.8, 0.6, 0.2), // Default food (brown)
                (ResourceType::Water, crate::rendering::BiomeType::Desert) => Color::rgb(0.2, 0.8, 0.2), // Cactus water (green)
                (ResourceType::Water, _) => Color::rgb(0.2, 0.6, 0.8), // Default water (blue)
            }
        } else {
            // Default colors when biome system not available
            match chosen_resource {
                ResourceType::Food => Color::rgb(0.8, 0.6, 0.2), // Brown for food
                ResourceType::Water => Color::rgb(0.2, 0.6, 0.8), // Blue for water
            }
        };

        commands.spawn((
            ResourceBundle::new(position, chosen_resource, amount),
            SpriteBundle {
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::new(15.0, 15.0)),
                    ..default()
                },
                transform: Transform::from_xyz(position.x, position.y, 0.0),
                ..default()
            },
            ResourceSprite {
                resource_type: chosen_resource,
            },
            Name::new(format!("{:?} {}", chosen_resource, spawned_resources)),
        ));
        
        spawned_resources += 1;
        if spawned_resources >= resource_count * 2 {
            break;
        }
    }

    info!(
        "Spawned {} creatures and {} resources",
        creature_count,
        resource_count * 2
    );
}
