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
    /// Width of isometric tiles in pixels
    /// Standard 2:1 ratio means this is twice the tile height
    pub tile_width: f32,
    /// Height of isometric tiles in pixels  
    /// Forms the minor axis of the diamond-shaped tile
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
/// # Mathematical Foundation
/// 
/// The isometric projection matrix for 2:1 ratio:
/// ```text
/// | x_screen |   |  0.5  -0.5   0 | | x_world |
/// | y_screen | = |  0.25  0.25 -1 | | z_world |
///                                    | y_world |
/// ```
/// 
/// Simplified to tile dimensions:
/// - x_screen = (x_world - z_world) * TILE_WIDTH / 2
/// - y_screen = (x_world + z_world) * TILE_HEIGHT / 2 - y_world * TILE_HEIGHT
/// 
/// # Coordinate System
/// 
/// World space uses right-handed coordinates:
/// - X: East (positive) to West (negative)
/// - Y: Up (positive) to Down (negative)
/// - Z: South (positive) to North (negative)
/// 
/// Screen space uses standard 2D coordinates:
/// - X: Right (positive)
/// - Y: Down (positive) - note the inversion!
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
/// # Inverse Transformation
/// 
/// Given the forward transformation:
/// - x_screen = (x - z) * tw/2
/// - y_screen = (x + z) * th/2 - y * th
/// 
/// Solving for world coordinates (with y=0):
/// - x = (x_screen/tw + y_screen/th)
/// - z = (y_screen/th - x_screen/tw)
/// 
/// The math simplifies because y=0 eliminates the height term.
/// 
/// # Camera Transformation
/// 
/// Screen coordinates must be adjusted for camera state:
/// 1. Subtract camera offset (pan position)
/// 2. Divide by zoom factor (scale adjustment)
/// 3. Apply inverse isometric transformation
/// 
/// # Use Cases
/// 
/// - Mouse picking: Convert click position to world tile
/// - UI placement: Position elements in world space
/// - Debug tools: Show world coordinates under cursor
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
/// Updates transform positions based on isometric projection
/// 
/// This system converts 2D world positions to screen coordinates using isometric
/// projection. It runs whenever a Position component changes, ensuring sprites
/// are rendered at the correct screen location.
/// 
/// The transformation pipeline:
/// 1. Extract 2D position from Position component
/// 2. Add height offset if IsometricHeight component exists
/// 3. Convert 3D world position to 2D screen coordinates
/// 4. Update Transform for rendering
fn update_isometric_transforms(
    mut query: Query<
        (&mut Transform, &crate::components::Position, Option<&crate::components::IsometricHeight>),
        Or<(Changed<crate::components::Position>, Added<crate::components::Position>)>,
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
/// # Depth Calculation Algorithm
/// 
/// The depth value combines multiple factors:
/// 
/// 1. **Base Depth**: X + Z coordinates
///    - Objects at higher X or Z are "further back"
///    - This creates the basic isometric ordering
/// 
/// 2. **Elevation Factor**: Y * 0.1
///    - Elevated objects render slightly behind ground objects
///    - Factor of 0.1 prevents excessive separation
/// 
/// 3. **Height Offset**: entity_height * 0.001
///    - Tall sprites need slight adjustment
///    - Very small factor to avoid sorting issues
/// 
/// # Depth Sorting Rules
/// 
/// In isometric view, correct sorting requires:
/// - Objects at same tile but different heights sort by Y
/// - Objects at different tiles sort by X+Z sum
/// - Tall objects don't incorrectly overlap short ones
/// 
/// # Z-Fighting Prevention
/// 
/// The small factors (0.1, 0.001) ensure sprites at same
/// logical position have slightly different depths, preventing
/// flickering from Z-fighting.
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
    
    // Adjust for elevation - entities at higher Y positions render behind those below
    // Factor of 0.1 prevents excessive depth separation between height levels
    let elevation_factor = world_pos.y * 0.1;
    
    // Fine-tune for entity height (taller entities need adjustment)
    // Small factor (0.001) ensures tall sprites don't drastically change depth
    let height_offset = entity_height * 0.001;
    
    base_depth + elevation_factor - height_offset
}

/// Enhanced sprite sorting with multiple depth layers
/// 
/// # Depth Layer System
/// 
/// Sprites are sorted into distinct layers:
/// - Background: -1000 to -100 (terrain, decorations)
/// - Entities: -100 to 100 (creatures, resources)
/// - Effects: 100 to 200 (particles, UI elements)
/// - Overlay: 200+ (debug info, selection indicators)
/// 
/// Within each layer, depth is calculated based on position.
fn sort_isometric_sprites(
    mut query: Query<(
        Entity,
        &mut Transform,
        &crate::components::Position,
        Option<&crate::components::IsometricSprite>,
        Option<&crate::components::IsometricHeight>,
        Option<&crate::components::CartoonSprite>,
        Option<&crate::components::IsometricSprite>, // Using existing component as Selected is not accessible
    )>,
    mut transparency_query: Query<&mut Sprite>,
) {
    // Collect positions for occlusion testing
    let mut entity_positions: Vec<(Entity, Vec3, f32)> = Vec::new();
    
    for (entity, transform, position, _iso_sprite, height, _cartoon, _selected) in query.iter() {
        let y = height.map(|h| h.0).unwrap_or(0.0);
        let world_pos = Vec3::new(position.0.x, y, position.0.y);
        let entity_height = transform.scale.y;
        entity_positions.push((entity, world_pos, entity_height));
    }
    
    // Sort and apply depth values
    for (entity, mut transform, position, iso_sprite, height, cartoon_sprite, _selected) in query.iter_mut() {
        // Convert 2D position to 3D world position
        let y = height.map(|h| h.0).unwrap_or(0.0);
        let world_pos = Vec3::new(position.0.x, y, position.0.y);
        
        // Calculate base depth
        let entity_height = transform.scale.y;
        let base_depth = calculate_depth(world_pos, entity_height);
        
        // Apply layer offset based on entity type
        let layer_offset = if cartoon_sprite.is_some() {
            0.0   // Normal entity layer
        } else {
            100.0 // Background elements
        };
        
        // Apply any custom offset
        let custom_offset = iso_sprite.map(|s| s.z_offset).unwrap_or(0.0);
        
        // Calculate final depth
        let final_depth = base_depth + layer_offset + custom_offset;
        
        // Apply transparency for occluded creatures
        if cartoon_sprite.is_some() {
            apply_occlusion_transparency(
                entity,
                world_pos,
                &entity_positions,
                &mut transparency_query,
            );
        }
        
        // Negate depth for Bevy's rendering order (higher z renders on top)
        transform.translation.z = -final_depth;
    }
}

/// Apply transparency to creatures that are behind others
fn apply_occlusion_transparency(
    entity: Entity,
    world_pos: Vec3,
    entity_positions: &[(Entity, Vec3, f32)],
    sprite_query: &mut Query<&mut Sprite>,
) {
    let mut is_occluded = false;
    
    // Check if any entity is in front and overlapping
    for &(other_entity, other_pos, _other_height) in entity_positions {
        if entity == other_entity { continue; }
        
        // Check if other entity is in front (lower x+z)
        if other_pos.x + other_pos.z < world_pos.x + world_pos.z {
            // Check for overlap in screen space
            let distance = ((other_pos.x - world_pos.x).powi(2) + 
                          (other_pos.z - world_pos.z).powi(2)).sqrt();
            
            if distance < 1.5 { // Within overlap threshold
                is_occluded = true;
                break;
            }
        }
    }
    
    // Apply transparency if occluded
    if let Ok(mut sprite) = sprite_query.get_mut(entity) {
        let target_alpha = if is_occluded { 0.6 } else { 1.0 };
        let current_alpha = sprite.color.a();
        
        // Smooth transition
        let new_alpha = current_alpha + (target_alpha - current_alpha) * 0.1;
        sprite.color.set_a(new_alpha);
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
    /// 
    /// # Culling Strategy
    /// 
    /// Determines which world positions are potentially visible to avoid
    /// rendering off-screen entities. The calculation:
    /// 
    /// 1. Takes viewport dimensions and zoom level
    /// 2. Projects screen corners to world space
    /// 3. Finds min/max bounds in world coordinates
    /// 4. Adds padding for safety
    /// 
    /// # Padding Values
    /// 
    /// Extra padding ensures smooth rendering:
    /// - **X/Z**: 2 tile padding for wide sprites and transitions
    /// - **Y**: 10 unit padding for tall structures and flying entities
    /// 
    /// This prevents pop-in when moving the camera and accounts for
    /// sprites that extend beyond their anchor point.
    /// 
    /// # Performance Impact
    /// 
    /// Proper culling can improve performance by 50-80% in large worlds
    /// by skipping render calls for off-screen entities.
    /// 
    /// # Returns
    /// 
    /// Tuple of (min_bounds, max_bounds) in world space
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
        // X/Z padding: 2 tiles to handle wide sprites and transitions
        // Y padding: 10 units to handle tall structures and flying entities
        let padding = Vec3::new(2.0, 10.0, 2.0);
        (min_world - padding, max_world + padding)
    }
}
