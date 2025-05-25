use bevy::prelude::*;

pub struct IsometricPlugin;

impl Plugin for IsometricPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, (
            update_isometric_transforms,
            sort_isometric_sprites,
        ).chain());
    }
}

/// Converts world coordinates to isometric screen coordinates
pub fn world_to_isometric(world_pos: Vec2) -> Vec2 {
    Vec2::new(
        (world_pos.x - world_pos.y) * 0.5,
        (world_pos.x + world_pos.y) * 0.25,
    )
}

/// Converts isometric screen coordinates to world coordinates
pub fn isometric_to_world(iso_pos: Vec2) -> Vec2 {
    Vec2::new(
        iso_pos.x + iso_pos.y * 2.0,
        iso_pos.y * 2.0 - iso_pos.x,
    )
}

/// Updates transform positions based on isometric projection
fn update_isometric_transforms(
    mut query: Query<(&mut Transform, &crate::components::Position), Changed<crate::components::Position>>,
) {
    for (mut transform, position) in query.iter_mut() {
        let iso_pos = world_to_isometric(position.0);
        transform.translation.x = iso_pos.x;
        transform.translation.y = iso_pos.y;
    }
}

/// Sorts sprites for proper depth ordering in isometric view
fn sort_isometric_sprites(
    mut query: Query<(&mut Transform, &crate::components::Position, Option<&crate::components::IsometricSprite>)>,
) {
    for (mut transform, position, iso_sprite) in query.iter_mut() {
        // Calculate depth based on position
        // In isometric view, objects further back (higher y) should be drawn first (lower z)
        let base_depth = -position.0.y + position.0.x * 0.5;
        
        // Apply any custom offset
        let offset = iso_sprite.map(|s| s.z_offset).unwrap_or(0.0);
        
        transform.translation.z = base_depth + offset;
    }
}