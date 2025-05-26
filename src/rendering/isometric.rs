use bevy::prelude::*;

pub struct IsometricPlugin;

impl Plugin for IsometricPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (update_isometric_transforms, sort_isometric_sprites).chain(),
        );
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
    Vec2::new(iso_pos.x + iso_pos.y * 2.0, iso_pos.y * 2.0 - iso_pos.x)
}

/// Updates transform positions based on isometric projection
fn update_isometric_transforms(
    mut query: Query<
        (&mut Transform, &crate::components::Position),
        Changed<crate::components::Position>,
    >,
) {
    for (mut transform, position) in query.iter_mut() {
        let iso_pos = world_to_isometric(position.0);
        transform.translation.x = iso_pos.x;
        transform.translation.y = iso_pos.y;
    }
}

/// Sorts sprites for proper depth ordering in isometric view
fn sort_isometric_sprites(
    mut query: Query<(
        &mut Transform,
        &crate::components::Position,
        Option<&crate::components::IsometricSprite>,
    )>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_to_isometric_conversion() {
        // Test origin
        let iso = world_to_isometric(Vec2::ZERO);
        assert_eq!(iso, Vec2::ZERO);

        // Test positive coordinates
        let iso = world_to_isometric(Vec2::new(100.0, 0.0));
        assert_eq!(iso, Vec2::new(50.0, 25.0));

        let iso = world_to_isometric(Vec2::new(0.0, 100.0));
        assert_eq!(iso, Vec2::new(-50.0, 25.0));

        let iso = world_to_isometric(Vec2::new(100.0, 100.0));
        assert_eq!(iso, Vec2::new(0.0, 50.0));

        // Test negative coordinates
        let iso = world_to_isometric(Vec2::new(-100.0, -100.0));
        assert_eq!(iso, Vec2::new(0.0, -50.0));
    }

    #[test]
    fn test_isometric_to_world_conversion() {
        // Test origin
        let world = isometric_to_world(Vec2::ZERO);
        assert_eq!(world, Vec2::ZERO);

        // Test conversions are inverse operations
        let original = Vec2::new(50.0, 75.0);
        let iso = world_to_isometric(original);
        let back = isometric_to_world(iso);
        assert!(
            (back - original).length() < 0.001,
            "Conversion should be reversible"
        );

        // Test specific cases
        let world = isometric_to_world(Vec2::new(50.0, 25.0));
        assert_eq!(world, Vec2::new(100.0, 0.0));

        let world = isometric_to_world(Vec2::new(-50.0, 25.0));
        assert_eq!(world, Vec2::new(0.0, 100.0));
    }

    #[test]
    fn test_round_trip_conversions() {
        // Test that converting back and forth gives the original value
        let test_points = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 200.0),
            Vec2::new(-50.0, 75.0),
            Vec2::new(123.45, -67.89),
            Vec2::new(-999.0, -999.0),
        ];

        for point in test_points {
            let iso = world_to_isometric(point);
            let world = isometric_to_world(iso);
            assert!(
                (world - point).length() < 0.001,
                "Round trip conversion failed for {:?}",
                point
            );
        }
    }
}
