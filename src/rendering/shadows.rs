use bevy::prelude::*;
use crate::components::{Position, CartoonSprite, IsometricHeight};
use crate::rendering::world_to_screen;

/// Plugin for rendering shadows to enhance depth perception
pub struct ShadowRenderingPlugin;

impl Plugin for ShadowRenderingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ShadowSettings::default())
            .add_systems(Update, (
                spawn_shadows,
                update_shadow_positions,
                cleanup_orphaned_shadows,
            ).chain());
    }
}

/// Settings for shadow rendering
#[derive(Resource)]
pub struct ShadowSettings {
    /// Global shadow opacity (0.0 = invisible, 1.0 = fully opaque)
    pub opacity: f32,
    /// Shadow offset from entity position
    pub offset: Vec2,
    /// Shadow scale relative to entity
    pub scale: Vec2,
    /// Enable/disable shadows globally
    pub enabled: bool,
    /// Maximum distance to render shadows (for performance)
    pub max_render_distance: f32,
}

impl Default for ShadowSettings {
    fn default() -> Self {
        Self {
            opacity: 0.3,
            offset: Vec2::new(0.0, -8.0), // 8 pixels below entity
            scale: Vec2::new(0.8, 0.4),    // Elliptical shadow
            enabled: true,
            max_render_distance: 500.0,
        }
    }
}

/// Component marking shadow entities
#[derive(Component)]
pub struct Shadow {
    /// Entity this shadow belongs to
    pub owner: Entity,
    /// Base size of the shadow
    pub base_size: Vec2,
}

/// System to spawn shadows for entities that need them
fn spawn_shadows(
    mut commands: Commands,
    shadow_settings: Res<ShadowSettings>,
    entities_without_shadows: Query<
        (Entity, &Position, Option<&CartoonSprite>),
        (Without<Shadow>, Or<(With<CartoonSprite>, With<crate::components::Creature>)>)
    >,
    existing_shadows: Query<&Shadow>,
) {
    if !shadow_settings.enabled {
        return;
    }
    
    for (entity, position, cartoon_sprite) in entities_without_shadows.iter() {
        // Check if shadow already exists
        let has_shadow = existing_shadows.iter().any(|s| s.owner == entity);
        if has_shadow {
            continue;
        }
        
        // Determine shadow size based on entity type
        let base_size = if cartoon_sprite.is_some() {
            Vec2::new(40.0, 20.0) // Creature shadow
        } else {
            Vec2::new(30.0, 15.0) // Default shadow
        };
        
        // Calculate shadow position
        let world_pos = Vec3::new(position.0.x, 0.0, position.0.y);
        let screen_pos = world_to_screen(world_pos);
        
        // Spawn shadow entity
        let shadow_entity = commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgba(0.0, 0.0, 0.0, shadow_settings.opacity),
                    custom_size: Some(base_size * shadow_settings.scale),
                    ..default()
                },
                transform: Transform::from_xyz(
                    screen_pos.x + shadow_settings.offset.x,
                    screen_pos.y + shadow_settings.offset.y,
                    -150.0, // Render below terrain
                ),
                ..default()
            },
            Shadow {
                owner: entity,
                base_size,
            },
            Name::new(format!("Shadow for {:?}", entity)),
        )).id();
        
        // Store shadow reference on owner (optional, for faster lookup)
        commands.entity(entity).insert(ShadowEntity(shadow_entity));
    }
}

/// Marker component storing shadow entity reference
#[derive(Component)]
pub struct ShadowEntity(pub Entity);

/// System to update shadow positions and properties
fn update_shadow_positions(
    shadow_settings: Res<ShadowSettings>,
    camera_query: Query<&Transform, With<crate::plugins::MainCamera>>,
    mut shadows: Query<(&mut Transform, &mut Sprite, &Shadow), Without<crate::plugins::MainCamera>>,
    owners: Query<(&Position, Option<&IsometricHeight>, Option<&crate::components::Health>)>,
) {
    if !shadow_settings.enabled {
        return;
    }
    
    let Ok(camera_transform) = camera_query.get_single() else { return; };
    let camera_pos = Vec2::new(camera_transform.translation.x, camera_transform.translation.y);
    
    for (mut shadow_transform, mut shadow_sprite, shadow) in shadows.iter_mut() {
        let Ok((position, height, health)) = owners.get(shadow.owner) else {
            continue;
        };
        
        // Calculate world position
        let elevation = height.map(|h| h.0).unwrap_or(0.0);
        let world_pos = Vec3::new(position.0.x, elevation, position.0.y);
        let screen_pos = world_to_screen(world_pos);
        
        // Check distance for LOD
        let distance = camera_pos.distance(Vec2::new(screen_pos.x, screen_pos.y));
        if distance > shadow_settings.max_render_distance {
            shadow_sprite.color.set_a(0.0); // Hide distant shadows
            continue;
        }
        
        // Update shadow position
        shadow_transform.translation.x = screen_pos.x + shadow_settings.offset.x;
        shadow_transform.translation.y = screen_pos.y + shadow_settings.offset.y - elevation * 10.0; // Offset based on height
        
        // Scale shadow based on elevation (higher = smaller shadow)
        let height_scale = 1.0 / (1.0 + elevation * 0.1);
        let final_size = shadow.base_size * shadow_settings.scale * height_scale;
        shadow_sprite.custom_size = Some(final_size);
        
        // Adjust opacity based on health (optional effect)
        let health_factor = health.map(|h| h.current / h.max).unwrap_or(1.0);
        let distance_fade = 1.0 - (distance / shadow_settings.max_render_distance).min(1.0);
        let final_opacity = shadow_settings.opacity * health_factor * distance_fade;
        
        shadow_sprite.color.set_a(final_opacity);
    }
}

/// System to remove shadows when their owner is despawned
fn cleanup_orphaned_shadows(
    mut commands: Commands,
    shadows: Query<(Entity, &Shadow)>,
    owners: Query<Entity>,
) {
    for (shadow_entity, shadow) in shadows.iter() {
        if owners.get(shadow.owner).is_err() {
            // Owner no longer exists, remove shadow
            commands.entity(shadow_entity).despawn_recursive();
        }
    }
}