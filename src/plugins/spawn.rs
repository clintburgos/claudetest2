use crate::components::ResourceType;
use crate::components::*;
use crate::core::determinism::DeterministicRng;
use crate::plugins::{CreatureSprite, ResourceSprite};
use bevy::prelude::*;

pub struct SpawnPlugin;

impl Plugin for SpawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_initial_entities);
    }
}

fn spawn_initial_entities(mut commands: Commands, _rng: ResMut<DeterministicRng>) {
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

    // Spawn initial resources - scale with creature count
    let resource_count = creature_count / 2; // More resources for more creatures

    for i in 0..resource_count {
        // Spawn food
        let x = (i as f32 % 6.0) * 80.0 - world_size / 2.0 + 40.0;
        let y = (i as f32 / 6.0).floor() * 80.0 - world_size / 2.0 + 40.0;
        let position = Vec2::new(x, y);

        commands.spawn((
            ResourceBundle::new(position, ResourceType::Food, 100.0),
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.8, 0.6, 0.2),
                    custom_size: Some(Vec2::new(15.0, 15.0)),
                    ..default()
                },
                transform: Transform::from_xyz(position.x, position.y, 0.0),
                ..default()
            },
            ResourceSprite {
                resource_type: ResourceType::Food,
            },
            Name::new(format!("Food {}", i)),
        ));

        // Spawn water
        let x = (i as f32 % 6.0) * 80.0 - world_size / 2.0;
        let y = (i as f32 / 6.0).floor() * 80.0 - world_size / 2.0;
        let position = Vec2::new(x, y);

        commands.spawn((
            ResourceBundle::new(position, ResourceType::Water, 100.0),
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.2, 0.6, 0.8),
                    custom_size: Some(Vec2::new(15.0, 15.0)),
                    ..default()
                },
                transform: Transform::from_xyz(position.x, position.y, 0.0),
                ..default()
            },
            ResourceSprite {
                resource_type: ResourceType::Water,
            },
            Name::new(format!("Water {}", i)),
        ));
    }

    info!(
        "Spawned {} creatures and {} resources",
        creature_count,
        resource_count * 2
    );
}
