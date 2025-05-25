use crate::rendering::IsometricPlugin;
use bevy::prelude::*;

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
    creatures: Query<
        (
            Entity,
            &crate::components::Position,
            &crate::components::CreatureType,
            &crate::components::CreatureState,
            &crate::components::Health,
            &crate::components::Size,
        ),
        (With<crate::components::Creature>, Without<CreatureSprite>),
    >,
    mut sprite_query: Query<
        (
            &mut Transform,
            &mut Sprite,
            &crate::components::Position,
            &crate::components::CreatureState,
            &crate::components::Health,
        ),
        With<CreatureSprite>,
    >,
) {
    // Spawn sprites for new creatures
    for (entity, position, creature_type, state, health, size) in creatures.iter() {
        let base_color = match creature_type {
            crate::components::CreatureType::Herbivore => Color::rgb(0.2, 0.7, 0.2), // Green
            crate::components::CreatureType::Carnivore => Color::rgb(0.8, 0.2, 0.2), // Red
            crate::components::CreatureType::Omnivore => Color::rgb(0.6, 0.4, 0.2),  // Brown
        };

        commands.entity(entity).insert((
            SpriteBundle {
                sprite: Sprite {
                    color: base_color,
                    custom_size: Some(Vec2::new(15.0 + size.0 * 5.0, 15.0 + size.0 * 5.0)),
                    ..default()
                },
                transform: Transform::from_xyz(position.0.x, position.0.y, 1.0),
                ..default()
            },
            CreatureSprite,
        ));
    }

    // Update positions and colors of existing sprites
    for (mut transform, mut sprite, position, state, health) in sprite_query.iter_mut() {
        // Update position
        transform.translation.x = position.0.x;
        transform.translation.y = position.0.y;

        // Update color based on state and health
        let health_factor = health.current / health.max;
        let alpha = match state {
            crate::components::CreatureState::Dead => 0.3,
            _ => 0.8 + 0.2 * health_factor,
        };

        // Tint based on state
        let state_tint = match state {
            crate::components::CreatureState::Eating => Color::rgba(1.0, 1.0, 0.5, alpha), // Yellow tint
            crate::components::CreatureState::Drinking => Color::rgba(0.5, 0.5, 1.0, alpha), // Blue tint
            crate::components::CreatureState::Resting => Color::rgba(0.7, 0.7, 0.7, alpha), // Gray tint
            crate::components::CreatureState::Moving { .. } => Color::rgba(1.0, 1.0, 1.0, alpha), // Normal
            crate::components::CreatureState::Idle => Color::rgba(0.9, 0.9, 0.9, alpha), // Slightly gray
            crate::components::CreatureState::Dead => Color::rgba(0.3, 0.3, 0.3, alpha), // Dark gray
        };

        // Blend with health (redder when low health)
        if health_factor < 0.3 && !matches!(state, crate::components::CreatureState::Dead) {
            let red_tint = Color::rgba(1.0, 0.3, 0.3, alpha);
            sprite.color = Color::rgba(
                sprite.color.r() * 0.5 + red_tint.r() * 0.5,
                sprite.color.g() * 0.5 + red_tint.g() * 0.5,
                sprite.color.b() * 0.5 + red_tint.b() * 0.5,
                alpha,
            );
        } else {
            sprite.color = Color::rgba(
                sprite.color.r() * 0.3 + state_tint.r() * 0.7,
                sprite.color.g() * 0.3 + state_tint.g() * 0.7,
                sprite.color.b() * 0.3 + state_tint.b() * 0.7,
                alpha,
            );
        }
    }
}

fn update_resource_sprites(
    mut commands: Commands,
    resources: Query<
        (
            Entity,
            &crate::components::Position,
            &crate::components::ResourceTypeComponent,
            &crate::components::ResourceAmount,
        ),
        (
            With<crate::components::ResourceMarker>,
            Without<ResourceSprite>,
        ),
    >,
    mut sprite_query: Query<
        (
            &mut Transform,
            &mut Sprite,
            &crate::components::Position,
            &crate::components::ResourceAmount,
        ),
        With<ResourceSprite>,
    >,
) {
    // Spawn sprites for new resources
    for (entity, position, resource_type, amount) in resources.iter() {
        let color = match resource_type.0 {
            crate::simulation::ResourceType::Food => Color::rgb(0.8, 0.6, 0.2), // Brown for food
            crate::simulation::ResourceType::Water => Color::rgb(0.2, 0.6, 0.8), // Blue for water
        };

        let size_factor = (amount.current / amount.max).max(0.3);

        commands.entity(entity).insert((
            SpriteBundle {
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::new(
                        10.0 + size_factor * 10.0,
                        10.0 + size_factor * 10.0,
                    )),
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

    // Update existing resource sprites
    for (mut transform, mut sprite, position, amount) in sprite_query.iter_mut() {
        transform.translation.x = position.0.x;
        transform.translation.y = position.0.y;

        // Scale based on remaining amount
        let size_factor = (amount.current / amount.max).max(0.3);
        sprite.custom_size = Some(Vec2::new(
            10.0 + size_factor * 10.0,
            10.0 + size_factor * 10.0,
        ));

        // Fade out depleted resources
        if amount.is_depleted() {
            sprite.color.set_a(0.3);
        }
    }
}
