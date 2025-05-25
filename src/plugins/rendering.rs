use bevy::prelude::*;
use crate::rendering::IsometricPlugin;

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(IsometricPlugin)
            .add_systems(Startup, setup_rendering)
            .add_systems(Update, (update_creature_sprites, update_resource_sprites));
    }
}

#[derive(Component)]
pub struct CreatureSprite;

#[derive(Component)]
pub struct ResourceSprite {
    pub resource_type: crate::simulation::ResourceType,
}

#[derive(Resource)]
pub struct RenderAssets {
    pub creature_texture: Handle<Image>,
    pub food_texture: Handle<Image>,
    pub water_texture: Handle<Image>,
}

fn setup_rendering(mut commands: Commands, asset_server: Res<AssetServer>) {
    // For Phase 1, we'll use simple colored squares as placeholders
    // In a real implementation, you'd load actual sprite assets
    
    // Store handles to our render assets
    commands.insert_resource(RenderAssets {
        creature_texture: asset_server.load("sprites/creature.png"),
        food_texture: asset_server.load("sprites/food.png"),
        water_texture: asset_server.load("sprites/water.png"),
    });
}

fn update_creature_sprites(
    mut commands: Commands,
    creatures: Query<(Entity, &crate::components::Position), (With<crate::components::Creature>, Without<CreatureSprite>)>,
    mut sprite_positions: Query<(&mut Transform, &crate::components::Position), With<CreatureSprite>>,
) {
    // Spawn sprites for new creatures
    for (entity, position) in creatures.iter() {
        commands.entity(entity).insert((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.3, 0.7, 0.3), // Green for creatures
                    custom_size: Some(Vec2::new(20.0, 20.0)),
                    ..default()
                },
                transform: Transform::from_xyz(position.0.x, position.0.y, 1.0),
                ..default()
            },
            CreatureSprite,
        ));
    }

    // Update positions of existing sprites
    for (mut transform, position) in sprite_positions.iter_mut() {
        transform.translation.x = position.0.x;
        transform.translation.y = position.0.y;
    }
}

fn update_resource_sprites(
    mut commands: Commands,
    resources: Query<(Entity, &crate::components::Position, &crate::components::ResourceTypeComponent), (With<crate::components::ResourceMarker>, Without<ResourceSprite>)>,
) {
    // Spawn sprites for new resources
    for (entity, position, resource_type) in resources.iter() {
        let color = match resource_type.0 {
            crate::simulation::ResourceType::Food => Color::rgb(0.8, 0.6, 0.2), // Brown for food
            crate::simulation::ResourceType::Water => Color::rgb(0.2, 0.6, 0.8), // Blue for water
        };

        commands.entity(entity).insert((
            SpriteBundle {
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::new(15.0, 15.0)),
                    ..default()
                },
                transform: Transform::from_xyz(position.0.x, position.0.y, 0.0),
                ..default()
            },
            ResourceSprite {
                resource_type: resource_type.0,
            },
        ));
    }
}