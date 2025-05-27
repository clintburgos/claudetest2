use crate::rendering::{CartoonRenderingPlugin, IsometricPlugin};
use crate::systems::biome::BiomePlugin;
use bevy::prelude::*;

// Type aliases to reduce complexity
type CreatureRenderComponents<'a> = (
    Entity,
    &'a crate::components::Position,
    &'a crate::components::CreatureType,
    &'a crate::components::CreatureState,
    &'a crate::components::Health,
    &'a crate::components::Size,
);

type ResourceRenderComponents<'a> = (
    Entity,
    &'a crate::components::Position,
    &'a crate::components::ResourceTypeComponent,
    &'a crate::components::ResourceAmount,
);

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        // Check if we should use cartoon rendering (can be made configurable later)
        let use_cartoon_rendering = true;
        
        if use_cartoon_rendering {
            app.add_plugins((
                IsometricPlugin,
                CartoonRenderingPlugin,
                BiomePlugin,
            ));
        } else {
            // Legacy colored square rendering
            app.add_plugins(IsometricPlugin)
                .add_systems(Startup, setup_rendering)
                .add_systems(Update, (update_creature_sprites, update_resource_sprites));
        }
    }
}

#[derive(Component)]
pub struct CreatureSprite;

#[derive(Component)]
pub struct ResourceSprite {
    #[allow(dead_code)]
    pub resource_type: crate::components::ResourceType,
}

#[derive(Resource)]
#[allow(dead_code)]
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
    config: Res<crate::core::performance_config::PerformanceConfig>,
    camera_query: Query<&Transform, With<crate::plugins::camera::MainCamera>>,
    creatures: Query<
        CreatureRenderComponents,
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
        (With<CreatureSprite>, Without<crate::plugins::camera::MainCamera>),
    >,
) {
    // Get camera position for culling
    let camera_pos = camera_query
        .get_single()
        .map(|t| Vec2::new(t.translation.x, t.translation.y))
        .unwrap_or_default();

    // Spawn sprites for new creatures (only if within cull distance)
    for (entity, position, creature_type, _state, _health, size) in creatures.iter() {
        // Don't create sprites for creatures too far from camera
        if position.0.distance(camera_pos) > config.lod_settings.cull_distance {
            continue;
        }
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
        ResourceRenderComponents,
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
            crate::components::ResourceType::Food => Color::rgb(0.8, 0.6, 0.2), // Brown for food
            crate::components::ResourceType::Water => Color::rgb(0.2, 0.6, 0.8), // Blue for water
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
