use bevy::prelude::*;

pub struct IsometricPlugin;

impl Plugin for IsometricPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(IsometricSettings::default())
            .add_systems(
                PostUpdate,
                (update_isometric_transforms, sort_isometric_sprites).chain(),
            );
    }
}

/// Constants for isometric projection
/// 
/// These define the standard 2:1 isometric projection used throughout the game.
/// The 64x32 tile size provides a good balance between detail and performance.
pub const TILE_WIDTH: f32 = 64.0;   // Base tile width in pixels
pub const TILE_HEIGHT: f32 = 32.0;  // Base tile height in pixels (2:1 ratio)  
pub const ISO_ANGLE: f32 = 30.0;    // Isometric angle in degrees (arctan(0.5))

#[derive(Resource)]
pub struct IsometricSettings {
    pub tile_width: f32,
    pub tile_height: f32,
}

impl Default for IsometricSettings {
    fn default() -> Self {
        Self {
            tile_width: TILE_WIDTH,
            tile_height: TILE_HEIGHT,
        }
    }
}

/// Convert 3D world coordinates to 2D screen coordinates
/// 
/// Uses a 2:1 isometric projection (30-degree angle) where:
/// - X axis projects to (1, 0.5) in screen space
/// - Z axis projects to (-1, 0.5) in screen space  
/// - Y axis (height) projects straight down
/// 
/// # Arguments
/// * `world_pos` - 3D position in world space (X=east/west, Y=up/down, Z=north/south)
/// 
/// # Returns
/// 2D position in screen space centered at origin
pub fn world_to_screen(world_pos: Vec3) -> Vec2 {
    let x = (world_pos.x - world_pos.z) * TILE_WIDTH / 2.0;
    let y = (world_pos.x + world_pos.z) * TILE_HEIGHT / 2.0 - world_pos.y * TILE_HEIGHT;
    Vec2::new(x, y)
}

/// Convert 2D screen coordinates to 3D world coordinates
/// 
/// Performs inverse isometric transformation for mouse picking and UI interaction.
/// Assumes y=0 (ground plane) since we can't determine height from 2D position alone.
/// 
/// # Arguments
/// * `screen_pos` - 2D position in screen space
/// * `camera_offset` - Camera pan offset to account for
/// * `camera_zoom` - Camera zoom level to account for
/// 
/// # Returns
/// 3D position in world space at ground level (y=0)
pub fn screen_to_world(screen_pos: Vec2, camera_offset: Vec2, camera_zoom: f32) -> Vec3 {
    // Account for camera zoom and offset
    let adjusted_pos = (screen_pos - camera_offset) / camera_zoom;
    
    // Inverse isometric transformation
    let x = (adjusted_pos.x / (TILE_WIDTH / 2.0) + adjusted_pos.y / (TILE_HEIGHT / 2.0)) / 2.0;
    let z = (adjusted_pos.y / (TILE_HEIGHT / 2.0) - adjusted_pos.x / (TILE_WIDTH / 2.0)) / 2.0;
    let y = 0.0; // Ground plane
    
    Vec3::new(x, y, z)
}

/// Legacy 2D world to isometric conversion (for backward compatibility)
pub fn world_to_isometric(world_pos: Vec2) -> Vec2 {
    world_to_screen(Vec3::new(world_pos.x, 0.0, world_pos.y))
}

/// Legacy isometric to 2D world conversion (for backward compatibility)
pub fn isometric_to_world(iso_pos: Vec2) -> Vec2 {
    let world_3d = screen_to_world(iso_pos, Vec2::ZERO, 1.0);
    Vec2::new(world_3d.x, world_3d.z)
}

/// Updates transform positions based on isometric projection
fn update_isometric_transforms(
    mut query: Query<
        (&mut Transform, &crate::components::Position, Option<&crate::components::IsometricHeight>),
        Changed<crate::components::Position>,
    >,
) {
    for (mut transform, position, height) in query.iter_mut() {
        // Use height if available, otherwise default to ground level
        let y = height.map(|h| h.0).unwrap_or(0.0);
        let world_pos = Vec3::new(position.0.x, y, position.0.y);
        let screen_pos = world_to_screen(world_pos);
        transform.translation.x = screen_pos.x;
        transform.translation.y = screen_pos.y;
    }
}

/// Calculate depth layer for isometric rendering
/// 
/// Determines draw order for sprites to create proper depth illusion.
/// Objects further "back" (higher X+Z) render behind nearer objects.
/// 
/// # Arguments
/// * `world_pos` - 3D position of the entity
/// * `entity_height` - Height of the entity sprite (for tall objects)
/// 
/// # Returns
/// Depth value where lower values render behind higher values
pub fn calculate_depth(world_pos: Vec3, entity_height: f32) -> f32 {
    // Base depth from position
    let base_depth = world_pos.x + world_pos.z;
    
    // Adjust for elevation
    let elevation_factor = world_pos.y * 0.1;
    
    // Fine-tune for entity height (taller entities need adjustment)
    let height_offset = entity_height * 0.001;
    
    base_depth + elevation_factor - height_offset
}

/// Sorts sprites for proper depth ordering in isometric view
fn sort_isometric_sprites(
    mut query: Query<(
        &mut Transform,
        &crate::components::Position,
        Option<&crate::components::IsometricSprite>,
        Option<&crate::components::IsometricHeight>,
    )>,
) {
    for (mut transform, position, iso_sprite, height) in query.iter_mut() {
        // Convert 2D position to 3D world position
        let y = height.map(|h| h.0).unwrap_or(0.0);
        let world_pos = Vec3::new(position.0.x, y, position.0.y);
        
        // Use the proper depth calculation
        let entity_height = transform.scale.y;
        let depth = calculate_depth(world_pos, entity_height);
        
        // Apply any custom offset
        let offset = iso_sprite.map(|s| s.z_offset).unwrap_or(0.0);
        
        // Negate depth for Bevy's rendering order (higher z renders on top)
        transform.translation.z = -(depth + offset);
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
        // With tile size 64x32, 1 world unit projects to 32,16 in screen space
        let iso = world_to_isometric(Vec2::new(1.0, 0.0));
        assert_eq!(iso, Vec2::new(32.0, 16.0));

        let iso = world_to_isometric(Vec2::new(0.0, 1.0));
        assert_eq!(iso, Vec2::new(-32.0, 16.0));

        let iso = world_to_isometric(Vec2::new(1.0, 1.0));
        assert_eq!(iso, Vec2::new(0.0, 32.0));

        // Test negative coordinates
        let iso = world_to_isometric(Vec2::new(-1.0, -1.0));
        assert_eq!(iso, Vec2::new(0.0, -32.0));
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

        // Test specific cases with correct tile dimensions
        let world = isometric_to_world(Vec2::new(32.0, 16.0));
        assert_eq!(world, Vec2::new(1.0, 0.0));

        let world = isometric_to_world(Vec2::new(-32.0, 16.0));
        assert_eq!(world, Vec2::new(0.0, 1.0));
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

/// Tile coordinate helper functions
pub mod tiles {
    use super::*;
    
    /// Convert tile grid coordinates to world position
    pub fn tile_to_world(tile_coord: IVec2) -> Vec3 {
        Vec3::new(
            tile_coord.x as f32,
            0.0, // Ground level
            tile_coord.y as f32,
        )
    }
    
    /// Convert world position to tile coordinates
    pub fn world_to_tile(world_pos: Vec3) -> IVec2 {
        IVec2::new(
            world_pos.x.round() as i32,
            world_pos.z.round() as i32,
        )
    }
    
    /// Get tile corners in screen space (for rendering)
    pub fn tile_screen_corners(tile_coord: IVec2) -> [Vec2; 4] {
        let world_pos = tile_to_world(tile_coord);
        [
            world_to_screen(world_pos + Vec3::new(-0.5, 0.0, -0.5)), // Top
            world_to_screen(world_pos + Vec3::new(0.5, 0.0, -0.5)),  // Right
            world_to_screen(world_pos + Vec3::new(0.5, 0.0, 0.5)),   // Bottom
            world_to_screen(world_pos + Vec3::new(-0.5, 0.0, 0.5)),  // Left
        ]
    }
}

/// Camera helper functions for isometric view
pub mod camera {
    use super::*;
    
    /// Calculate visible world bounds for culling
    pub fn calculate_visible_bounds(
        camera_pos: Vec3,
        viewport_size: Vec2,
        zoom: f32,
    ) -> (Vec3, Vec3) {
        // Account for isometric projection angle
        let half_width = (viewport_size.x / 2.0) / zoom;
        let half_height = (viewport_size.y / 2.0) / zoom;
        
        // Convert screen bounds to world space
        let screen_corners = [
            Vec2::new(-half_width, -half_height),
            Vec2::new(half_width, -half_height),
            Vec2::new(half_width, half_height),
            Vec2::new(-half_width, half_height),
        ];
        
        let mut min_world = Vec3::splat(f32::MAX);
        let mut max_world = Vec3::splat(f32::MIN);
        
        for corner in &screen_corners {
            let world_pos = screen_to_world(*corner, Vec2::ZERO, zoom) + camera_pos;
            min_world = min_world.min(world_pos);
            max_world = max_world.max(world_pos);
        }
        
        // Expand bounds to ensure we don't cull edge cases
        let padding = Vec3::new(2.0, 10.0, 2.0);
        (min_world - padding, max_world + padding)
    }
}
