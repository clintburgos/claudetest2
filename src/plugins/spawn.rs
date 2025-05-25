use crate::components::*;
use crate::simulation::ResourceType;
use bevy::prelude::*;

pub struct SpawnPlugin;

impl Plugin for SpawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_initial_entities);
    }
}

fn spawn_initial_entities(mut commands: Commands) {
    // Spawn initial creatures
    let creature_count = 50; // Start with fewer for testing
    let world_size = 500.0;

    for i in 0..creature_count {
        let x = (i as f32 % 10.0) * 50.0 - world_size / 2.0;
        let y = (i as f32 / 10.0).floor() * 50.0 - world_size / 2.0;
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

    // Spawn initial resources
    let resource_count = 30;

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
            ResourceSprite,
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
            ResourceSprite,
            Name::new(format!("Water {}", i)),
        ));
    }

    info!(
        "Spawned {} creatures and {} resources",
        creature_count,
        resource_count * 2
    );
}
