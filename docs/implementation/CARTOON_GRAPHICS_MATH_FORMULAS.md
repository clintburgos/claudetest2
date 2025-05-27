# Cartoon Graphics Mathematical Formulas

This document provides the exact mathematical formulas and algorithms needed for the isometric cartoon graphics implementation.

## Coordinate System Transformations

### World to Screen (Isometric Projection)

```rust
/// Convert 3D world coordinates to 2D screen coordinates
/// Uses a 2:1 isometric projection (30-degree angle)
pub fn world_to_screen(world_pos: Vec3) -> Vec2 {
    let x = (world_pos.x - world_pos.z) * TILE_WIDTH / 2.0;
    let y = (world_pos.x + world_pos.z) * TILE_HEIGHT / 2.0 - world_pos.y * TILE_HEIGHT;
    Vec2::new(x, y)
}

/// Constants for isometric projection
const TILE_WIDTH: f32 = 64.0;   // Base tile width in pixels
const TILE_HEIGHT: f32 = 32.0;  // Base tile height in pixels (2:1 ratio)
const ISO_ANGLE: f32 = 30.0;    // Isometric angle in degrees
```

### Screen to World (Unprojection for Mouse Picking)

```rust
/// Convert 2D screen coordinates to 3D world coordinates
/// Assumes y=0 (ground plane) for mouse picking
pub fn screen_to_world(screen_pos: Vec2, camera: &Camera) -> Vec3 {
    // Account for camera zoom and offset
    let adjusted_pos = (screen_pos - camera.offset) / camera.zoom;
    
    // Inverse isometric transformation
    let x = (adjusted_pos.x / (TILE_WIDTH / 2.0) + adjusted_pos.y / (TILE_HEIGHT / 2.0)) / 2.0;
    let z = (adjusted_pos.y / (TILE_HEIGHT / 2.0) - adjusted_pos.x / (TILE_WIDTH / 2.0)) / 2.0;
    let y = 0.0; // Ground plane
    
    Vec3::new(x, y, z)
}

/// Enhanced version with ray casting for elevated objects
pub fn screen_to_world_raycast(
    screen_pos: Vec2,
    camera: &Camera,
    spatial_grid: &SpatialGrid,
) -> Option<(Vec3, Entity)> {
    let ray_origin = screen_to_world(screen_pos, camera);
    let ray_direction = Vec3::new(0.0, -1.0, 0.0); // Straight down
    
    // Check intersections with entities in spatial grid
    spatial_grid.raycast(ray_origin, ray_direction, f32::MAX)
}
```

### Tile Coordinate Conversions

```rust
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
```

## Depth Sorting Algorithm

```rust
/// Calculate depth layer for isometric rendering
/// Lower values render behind higher values
pub fn calculate_depth(world_pos: Vec3, entity_height: f32) -> f32 {
    // Base depth from position
    let base_depth = world_pos.x + world_pos.z;
    
    // Adjust for elevation
    let elevation_factor = world_pos.y * 0.1;
    
    // Fine-tune for entity height (taller entities need adjustment)
    let height_offset = entity_height * 0.001;
    
    base_depth + elevation_factor - height_offset
}

/// Sort entities for isometric rendering
pub fn sort_isometric_entities(entities: &mut Vec<(Entity, Transform)>) {
    entities.sort_by(|a, b| {
        let depth_a = calculate_depth(a.1.translation, a.1.scale.y);
        let depth_b = calculate_depth(b.1.translation, b.1.scale.y);
        depth_a.partial_cmp(&depth_b).unwrap_or(Ordering::Equal)
    });
}
```

## Biome Transition Blending

```rust
/// Calculate blend weights for smooth biome transitions
pub fn calculate_biome_blend_weights(
    world_pos: Vec2,
    biome_map: &BiomeMap,
    blend_radius: f32,
) -> Vec<(BiomeType, f32)> {
    let mut weights = Vec::new();
    let center_biome = biome_map.get_biome(world_pos);
    
    // Sample in a circle around the position
    const SAMPLE_COUNT: usize = 8;
    let mut biome_distances: HashMap<BiomeType, f32> = HashMap::new();
    
    for i in 0..SAMPLE_COUNT {
        let angle = (i as f32 / SAMPLE_COUNT as f32) * TAU;
        let sample_offset = Vec2::new(angle.cos(), angle.sin()) * blend_radius;
        let sample_pos = world_pos + sample_offset;
        let sample_biome = biome_map.get_biome(sample_pos);
        
        // Find distance to biome boundary
        let distance = find_biome_boundary_distance(world_pos, sample_pos, biome_map);
        biome_distances.entry(sample_biome)
            .and_modify(|d| *d = d.min(distance))
            .or_insert(distance);
    }
    
    // Convert distances to weights (inverse distance weighting)
    let total_inv_distance: f32 = biome_distances.values()
        .map(|d| 1.0 / (d + 0.1)) // Add small epsilon to avoid division by zero
        .sum();
    
    for (biome, distance) in biome_distances {
        let weight = (1.0 / (distance + 0.1)) / total_inv_distance;
        if weight > 0.01 { // Ignore very small weights
            weights.push((biome, weight));
        }
    }
    
    // Ensure weights sum to 1.0
    let weight_sum: f32 = weights.iter().map(|(_, w)| w).sum();
    for (_, weight) in &mut weights {
        *weight /= weight_sum;
    }
    
    weights
}

/// Select transition tiles based on neighbor biomes
pub fn select_transition_tile(
    tile_pos: IVec2,
    biome_map: &BiomeMap,
    tile_sets: &HashMap<BiomeType, TileSet>,
) -> TileId {
    let center_biome = biome_map.get_biome_tile(tile_pos);
    let mut neighbor_mask = 0u8;
    
    // Check 8 neighbors and build a bitmask
    const NEIGHBOR_OFFSETS: [(i32, i32); 8] = [
        (-1, -1), (0, -1), (1, -1),
        (-1,  0),          (1,  0),
        (-1,  1), (0,  1), (1,  1),
    ];
    
    for (i, (dx, dy)) in NEIGHBOR_OFFSETS.iter().enumerate() {
        let neighbor_pos = tile_pos + IVec2::new(*dx, *dy);
        let neighbor_biome = biome_map.get_biome_tile(neighbor_pos);
        if neighbor_biome != center_biome {
            neighbor_mask |= 1 << i;
        }
    }
    
    // Select appropriate transition tile based on mask
    tile_sets[&center_biome].get_transition_tile(neighbor_mask)
}
```

## LOD Distance Calculations

```rust
/// LOD level thresholds and calculations
pub struct LODThresholds {
    pub full: f32,      // 0-50 units
    pub high: f32,      // 50-100 units
    pub medium: f32,    // 100-200 units
    pub low: f32,       // 200-400 units
    pub minimal: f32,   // 400+ units
}

impl Default for LODThresholds {
    fn default() -> Self {
        Self {
            full: 50.0,
            high: 100.0,
            medium: 200.0,
            low: 400.0,
            minimal: f32::MAX,
        }
    }
}

/// Calculate LOD level based on distance and quality settings
pub fn calculate_lod_level(
    camera_pos: Vec3,
    entity_pos: Vec3,
    entity_importance: f32, // 0.0-1.0, higher = more important
    quality_multiplier: f32, // From quality settings
) -> LODLevel {
    let distance = camera_pos.distance(entity_pos);
    
    // Adjust thresholds based on importance and quality
    let importance_factor = 1.0 + entity_importance * 0.5;
    let adjusted_distance = distance / (importance_factor * quality_multiplier);
    
    let thresholds = LODThresholds::default();
    
    match adjusted_distance {
        d if d < thresholds.full => LODLevel::Full,
        d if d < thresholds.high => LODLevel::High,
        d if d < thresholds.medium => LODLevel::Medium,
        d if d < thresholds.low => LODLevel::Low,
        _ => LODLevel::Minimal,
    }
}

/// Dynamic LOD adjustment based on performance
pub fn adjust_lod_thresholds(
    thresholds: &mut LODThresholds,
    current_fps: f32,
    target_fps: f32,
) {
    let performance_ratio = current_fps / target_fps;
    
    if performance_ratio < 0.9 {
        // Reduce quality - decrease thresholds
        let reduction = 0.9;
        thresholds.full *= reduction;
        thresholds.high *= reduction;
        thresholds.medium *= reduction;
        thresholds.low *= reduction;
    } else if performance_ratio > 1.1 {
        // Increase quality - increase thresholds
        let increase = 1.1;
        thresholds.full = (thresholds.full * increase).min(50.0);
        thresholds.high = (thresholds.high * increase).min(100.0);
        thresholds.medium = (thresholds.medium * increase).min(200.0);
        thresholds.low = (thresholds.low * increase).min(400.0);
    }
}
```

## Camera Mathematics

```rust
/// Isometric camera configuration
pub struct IsometricCamera {
    pub position: Vec3,      // World position camera is looking at
    pub zoom: f32,          // Zoom level (1.0 = default)
    pub viewport_size: Vec2, // Screen dimensions
    pub offset: Vec2,       // Screen space offset for centering
}

impl IsometricCamera {
    /// Calculate visible world bounds for culling
    pub fn calculate_visible_bounds(&self) -> (Vec3, Vec3) {
        // Account for isometric projection angle
        let half_width = (self.viewport_size.x / 2.0) / self.zoom;
        let half_height = (self.viewport_size.y / 2.0) / self.zoom;
        
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
            let world_pos = screen_to_world(*corner + self.offset, self);
            min_world = min_world.min(world_pos);
            max_world = max_world.max(world_pos);
        }
        
        // Expand bounds to ensure we don't cull edge cases
        let padding = Vec3::new(2.0, 10.0, 2.0);
        (min_world - padding, max_world + padding)
    }
    
    /// Smooth camera movement with easing
    pub fn smooth_move_to(&mut self, target: Vec3, delta_time: f32, smoothness: f32) {
        let t = 1.0 - (-delta_time * smoothness).exp();
        self.position = self.position.lerp(target, t);
    }
    
    /// Smooth zoom with constraints
    pub fn smooth_zoom(&mut self, target_zoom: f32, delta_time: f32) {
        const MIN_ZOOM: f32 = 0.25;
        const MAX_ZOOM: f32 = 4.0;
        const ZOOM_SPEED: f32 = 5.0;
        
        let clamped_target = target_zoom.clamp(MIN_ZOOM, MAX_ZOOM);
        let t = 1.0 - (-delta_time * ZOOM_SPEED).exp();
        self.zoom = self.zoom.lerp(clamped_target, t);
    }
}
```

## Sprite Atlas Optimization

```rust
/// Rectangle packing for sprite atlas generation
pub struct AtlasPacker {
    width: u32,
    height: u32,
    free_rects: Vec<Rect>,
}

impl AtlasPacker {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            free_rects: vec![Rect { x: 0, y: 0, width, height }],
        }
    }
    
    /// Pack a rectangle using the MaxRects algorithm
    pub fn pack(&mut self, width: u32, height: u32) -> Option<Rect> {
        let (best_rect, best_index) = self.find_best_position(width, height)?;
        
        // Remove the used rectangle
        self.free_rects.remove(best_index);
        
        // Split remaining free space
        self.split_free_rect(best_rect, width, height);
        
        Some(Rect {
            x: best_rect.x,
            y: best_rect.y,
            width,
            height,
        })
    }
    
    fn find_best_position(&self, width: u32, height: u32) -> Option<(Rect, usize)> {
        let mut best_score = u32::MAX;
        let mut best_rect = None;
        let mut best_index = 0;
        
        for (index, rect) in self.free_rects.iter().enumerate() {
            if rect.width >= width && rect.height >= height {
                // Best short side fit heuristic
                let leftover_x = rect.width - width;
                let leftover_y = rect.height - height;
                let score = leftover_x.min(leftover_y);
                
                if score < best_score {
                    best_score = score;
                    best_rect = Some(*rect);
                    best_index = index;
                }
            }
        }
        
        best_rect.map(|rect| (rect, best_index))
    }
    
    fn split_free_rect(&mut self, rect: Rect, used_width: u32, used_height: u32) {
        // Right side remainder
        if rect.width > used_width {
            self.free_rects.push(Rect {
                x: rect.x + used_width,
                y: rect.y,
                width: rect.width - used_width,
                height: used_height,
            });
        }
        
        // Bottom remainder  
        if rect.height > used_height {
            self.free_rects.push(Rect {
                x: rect.x,
                y: rect.y + used_height,
                width: rect.width,
                height: rect.height - used_height,
            });
        }
    }
}
```

## Animation Interpolation

```rust
/// Smooth animation frame interpolation
pub fn interpolate_animation_frame(
    current_frame: f32,
    frame_count: usize,
    loop_mode: AnimationLoopMode,
) -> (usize, f32) {
    match loop_mode {
        AnimationLoopMode::Loop => {
            let frame = current_frame as usize % frame_count;
            let blend = current_frame.fract();
            (frame, blend)
        }
        AnimationLoopMode::PingPong => {
            let cycle_length = (frame_count - 1) * 2;
            let cycle_pos = current_frame as usize % cycle_length;
            
            let (frame, blend) = if cycle_pos < frame_count {
                (cycle_pos, current_frame.fract())
            } else {
                let reverse_pos = cycle_length - cycle_pos;
                (reverse_pos, current_frame.fract())
            };
            
            (frame, blend)
        }
        AnimationLoopMode::Once => {
            if current_frame >= frame_count as f32 - 1.0 {
                (frame_count - 1, 0.0)
            } else {
                (current_frame as usize, current_frame.fract())
            }
        }
    }
}

/// Calculate sprite UV coordinates for animation frame
pub fn calculate_frame_uvs(
    frame_index: usize,
    sprite_sheet_columns: usize,
    sprite_size: Vec2,
    texture_size: Vec2,
) -> [Vec2; 4] {
    let col = frame_index % sprite_sheet_columns;
    let row = frame_index / sprite_sheet_columns;
    
    let uv_width = sprite_size.x / texture_size.x;
    let uv_height = sprite_size.y / texture_size.y;
    
    let uv_x = col as f32 * uv_width;
    let uv_y = row as f32 * uv_height;
    
    [
        Vec2::new(uv_x, uv_y),                          // Top-left
        Vec2::new(uv_x + uv_width, uv_y),              // Top-right
        Vec2::new(uv_x + uv_width, uv_y + uv_height),  // Bottom-right
        Vec2::new(uv_x, uv_y + uv_height),             // Bottom-left
    ]
}
```