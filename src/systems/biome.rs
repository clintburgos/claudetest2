use bevy::prelude::*;
use noise::{NoiseFn, Perlin};
use crate::rendering::{BiomeType, world_to_screen};
use crate::rendering::isometric::tiles::{tile_to_world, world_to_tile};
use crate::components::ResourceType;

/// Resource for managing biome generation and biome-specific resources
/// 
/// Uses Perlin noise to generate temperature and moisture maps that determine
/// biome placement. Also manages biome-specific resource types.
#[derive(Resource)]
pub struct BiomeMap {
    /// Random seed for deterministic generation
    pub seed: u32,
    /// Perlin noise generator for temperature values
    pub temperature_noise: Perlin,
    /// Perlin noise generator for moisture values
    pub moisture_noise: Perlin,
    /// Cache of computed biome types to avoid recalculation
    pub biome_cache: std::collections::HashMap<IVec2, BiomeType>,
}

impl BiomeMap {
    pub fn new(seed: u32) -> Self {
        Self {
            seed,
            temperature_noise: Perlin::new(seed),
            moisture_noise: Perlin::new(seed + 1),
            biome_cache: std::collections::HashMap::new(),
        }
    }
    
    /// Get biome-specific resource types that can spawn in a given biome
    /// Returns a list of resource types with their relative spawn weights
    pub fn get_biome_resources(biome: BiomeType) -> Vec<(ResourceType, f32)> {
        match biome {
            BiomeType::Forest => vec![
                (ResourceType::Food, 0.7),  // Berries and nuts are common
                (ResourceType::Water, 0.3), // Some water sources
            ],
            BiomeType::Desert => vec![
                (ResourceType::Water, 0.8), // Cacti water is primary resource
                (ResourceType::Food, 0.2),  // Desert fruits are rare
            ],
            BiomeType::Grassland => vec![
                (ResourceType::Food, 0.5),  // Seeds and grasses
                (ResourceType::Water, 0.5), // Balanced resources
            ],
            BiomeType::Tundra => vec![
                (ResourceType::Food, 0.6),  // Snow berries, ice fish
                (ResourceType::Water, 0.4), // Ice/snow for water
            ],
            BiomeType::Ocean => vec![
                (ResourceType::Food, 0.9),  // Fish, shellfish, seaweed
                (ResourceType::Water, 0.1), // Salt water (less useful)
            ],
        }
    }
    
    /// Get resource abundance modifier for a biome
    /// Some biomes have more abundant resources than others
    pub fn get_biome_abundance(biome: BiomeType) -> f32 {
        match biome {
            BiomeType::Forest => 1.2,      // Rich in resources
            BiomeType::Desert => 0.6,      // Scarce resources
            BiomeType::Grassland => 1.0,   // Average abundance
            BiomeType::Tundra => 0.7,      // Limited resources
            BiomeType::Ocean => 1.1,       // Good food availability
        }
    }
    
    /// Get biome type at a world position
    /// 
    /// Converts world coordinates to tile coordinates and retrieves the biome.
    /// Results are cached for performance.
    /// 
    /// # Arguments
    /// * `world_pos` - 2D position in world space
    /// 
    /// # Returns
    /// The biome type at the given position
    pub fn get_biome(&mut self, world_pos: Vec2) -> BiomeType {
        let tile_pos = world_to_tile(Vec3::new(world_pos.x, 0.0, world_pos.y));
        self.get_biome_tile(tile_pos)
    }
    
    /// Get biome at tile coordinates
    pub fn get_biome_tile(&mut self, tile_pos: IVec2) -> BiomeType {
        // Check cache first
        if let Some(&biome) = self.biome_cache.get(&tile_pos) {
            return biome;
        }
        
        // Generate biome based on temperature and moisture
        let scale = 0.02; // Noise scale for larger biomes
        let x = tile_pos.x as f64 * scale;
        let y = tile_pos.y as f64 * scale;
        
        let temperature = self.temperature_noise.get([x, y]) as f32;
        let moisture = self.moisture_noise.get([x, y]) as f32;
        
        // Map temperature and moisture to biomes
        let biome = match (temperature, moisture) {
            (t, m) if t > 0.3 && m < -0.2 => BiomeType::Desert,
            (t, m) if t < -0.3 && m > 0.2 => BiomeType::Tundra,
            (t, m) if t > 0.0 && m > 0.3 => BiomeType::Forest,
            (_t, m) if m > 0.5 => BiomeType::Ocean,
            _ => BiomeType::Grassland,
        };
        
        // Cache the result
        self.biome_cache.insert(tile_pos, biome);
        biome
    }
    
    /// Clear cache for memory management
    pub fn clear_distant_cache(&mut self, center: IVec2, radius: i32) {
        self.biome_cache.retain(|&pos, _| {
            (pos.x - center.x).abs() <= radius && (pos.y - center.y).abs() <= radius
        });
    }
}

/// Component for terrain tiles
#[derive(Component)]
pub struct TerrainTile {
    pub biome: BiomeType,
    pub tile_coord: IVec2,
    pub tile_variant: u8,
}

/// Bundle for spawning terrain tiles
#[derive(Bundle)]
pub struct TerrainTileBundle {
    pub tile: TerrainTile,
    pub sprite: SpriteBundle,
    pub name: Name,
}

/// System to generate visible terrain chunks around the camera
/// 
/// This system:
/// 1. Determines which tiles are visible based on camera position
/// 2. Spawns terrain tile entities for newly visible areas
/// 3. Despawns tiles that have moved too far from view
/// 4. Manages the biome cache to prevent memory growth
/// 
/// Uses a fixed view radius to balance visual range with performance
pub fn generate_terrain_chunks(
    mut commands: Commands,
    mut biome_map: ResMut<BiomeMap>,
    camera_query: Query<&Transform, With<crate::plugins::MainCamera>>,
    _existing_tiles: Query<&TerrainTile>,
    mut tile_entities: Local<std::collections::HashMap<IVec2, Entity>>,
) {
    let Ok(camera_transform) = camera_query.get_single() else {
        return;
    };
    
    // Get camera position in world space
    let camera_world = Vec3::new(
        camera_transform.translation.x,
        0.0,
        camera_transform.translation.y,
    );
    
    // Calculate visible tile range
    let view_radius = 20; // Tiles to render around camera
    let center_tile = world_to_tile(camera_world);
    
    // Generate tiles in visible range
    for dy in -view_radius..=view_radius {
        for dx in -view_radius..=view_radius {
            let tile_coord = IVec2::new(center_tile.x + dx, center_tile.y + dy);
            
            // Skip if tile already exists
            if tile_entities.contains_key(&tile_coord) {
                continue;
            }
            
            // Get biome for this tile
            let biome = biome_map.get_biome_tile(tile_coord);
            
            // Calculate world position
            let world_pos = tile_to_world(tile_coord);
            let screen_pos = world_to_screen(world_pos);
            
            // Choose tile variant (for visual variety)
            let tile_variant = ((tile_coord.x * 7 + tile_coord.y * 13) % 4) as u8;
            
            // Spawn tile entity
            let entity = commands.spawn(TerrainTileBundle {
                tile: TerrainTile {
                    biome,
                    tile_coord,
                    tile_variant,
                },
                sprite: SpriteBundle {
                    sprite: Sprite {
                        color: get_biome_color(biome),
                        custom_size: Some(Vec2::new(64.0, 32.0)), // Isometric tile size
                        ..default()
                    },
                    transform: Transform::from_xyz(screen_pos.x, screen_pos.y, -100.0), // Below creatures
                    ..default()
                },
                name: Name::new(format!("Tile({}, {})", tile_coord.x, tile_coord.y)),
            }).id();
            
            tile_entities.insert(tile_coord, entity);
        }
    }
    
    // Clean up distant tiles
    let cleanup_radius = view_radius + 10;
    tile_entities.retain(|&coord, &mut entity| {
        let distance = (coord - center_tile).abs();
        if distance.x > cleanup_radius || distance.y > cleanup_radius {
            commands.entity(entity).despawn();
            false
        } else {
            true
        }
    });
    
    // Clean biome cache
    biome_map.clear_distant_cache(center_tile, cleanup_radius);
}

/// Get placeholder color for biome types
/// 
/// These colors are used until proper tile sprites are loaded.
/// Each biome has a distinct color for visual identification.
/// 
/// # Arguments
/// * `biome` - The biome type to get color for
/// 
/// # Returns  
/// A color representing the biome
fn get_biome_color(biome: BiomeType) -> Color {
    match biome {
        BiomeType::Forest => Color::rgb(0.13, 0.55, 0.13), // Forest Green
        BiomeType::Desert => Color::rgb(0.93, 0.79, 0.69), // Sand
        BiomeType::Grassland => Color::rgb(0.49, 0.98, 0.0), // Grass Green  
        BiomeType::Tundra => Color::rgb(0.9, 0.9, 0.95), // Snow White
        BiomeType::Ocean => Color::rgb(0.0, 0.46, 0.74), // Ocean Blue
    }
}

/// Plugin for biome and terrain systems
pub struct BiomePlugin;

impl Plugin for BiomePlugin {
    fn build(&self, app: &mut App) {
        // Use a fixed seed for now (can be randomized later)
        let biome_map = BiomeMap::new(12345);
        
        app.insert_resource(biome_map)
            .add_systems(Update, generate_terrain_chunks);
    }
}