use bevy::prelude::*;
use noise::{NoiseFn, Perlin, Fbm};
use crate::rendering::{BiomeType, world_to_screen};
use crate::rendering::isometric::tiles::{tile_to_world, world_to_tile};
use crate::components::ResourceType;
use std::collections::HashMap;

/// Resource for managing biome generation and biome-specific resources
/// 
/// Uses Perlin noise to generate temperature and moisture maps that determine
/// biome placement. Also manages biome-specific resource types.
/// 
/// # Biome Generation Algorithm
/// 
/// The system uses two independent Perlin noise functions:
/// 1. **Temperature map**: Ranges from -1.0 (cold) to 1.0 (hot)
/// 2. **Moisture map**: Ranges from -1.0 (dry) to 1.0 (wet)
/// 
/// These values combine to determine biome types:
/// - Hot + Dry = Desert
/// - Cold + Wet = Tundra  
/// - Warm + Wet = Forest
/// - Very Wet = Ocean (overrides temperature)
/// - Moderate = Grassland (default)
/// 
/// # Caching Strategy
/// 
/// Computed biome values are cached in a HashMap to avoid recalculation.
/// The cache is periodically cleaned to prevent unbounded memory growth,
/// keeping only tiles within a certain radius of the camera.
#[derive(Resource)]
pub struct BiomeMap {
    /// Random seed for deterministic generation
    pub seed: u32,
    /// Multi-layer noise for temperature (fractal brownian motion)
    pub temperature_noise: Fbm<Perlin>,
    /// Multi-layer noise for moisture  
    pub moisture_noise: Fbm<Perlin>,
    /// Additional noise layer for elevation
    pub elevation_noise: Fbm<Perlin>,
    /// Fine detail noise for variations
    pub detail_noise: Perlin,
    /// Cache of computed biome types to avoid recalculation
    pub biome_cache: HashMap<IVec2, BiomeData>,
    /// Transition blend cache for smooth borders
    pub transition_cache: HashMap<IVec2, TransitionData>,
}

/// Enhanced biome data with additional properties
#[derive(Clone, Debug)]
pub struct BiomeData {
    pub biome_type: BiomeType,
    pub temperature: f32,
    pub moisture: f32,
    pub elevation: f32,
    pub variation: f32,
}

/// Data for smooth biome transitions
#[derive(Clone, Debug)]
pub struct TransitionData {
    pub primary_biome: BiomeType,
    pub secondary_biome: Option<BiomeType>,
    pub blend_factor: f32,
    pub edge_distance: f32,
}

impl BiomeMap {
    pub fn new(seed: u32) -> Self {
        // Create multi-octave noise for more realistic terrain
        let mut temp_noise = Fbm::<Perlin>::new(seed);
        temp_noise.octaves = 6;
        temp_noise.frequency = 0.8;
        temp_noise.lacunarity = 2.0;
        temp_noise.persistence = 0.5;
        
        let mut moist_noise = Fbm::<Perlin>::new(seed + 1);
        moist_noise.octaves = 6;
        moist_noise.frequency = 0.9;
        moist_noise.lacunarity = 2.2;
        moist_noise.persistence = 0.45;
        
        let mut elev_noise = Fbm::<Perlin>::new(seed + 2);
        elev_noise.octaves = 4;
        elev_noise.frequency = 0.4;
        elev_noise.lacunarity = 2.5;
        elev_noise.persistence = 0.6;
        
        Self {
            seed,
            temperature_noise: temp_noise,
            moisture_noise: moist_noise,
            elevation_noise: elev_noise,
            detail_noise: Perlin::new(seed + 3),
            biome_cache: HashMap::new(),
            transition_cache: HashMap::new(),
        }
    }
    
    /// Get biome-specific resource types that can spawn in a given biome
    /// Returns a list of resource types with their relative spawn weights
    /// 
    /// # Spawn Weight System
    /// 
    /// Weights represent probability ratios - higher weight = more likely to spawn
    /// Example: Forest (0.7 food, 0.3 water) = 70% chance for food, 30% for water
    /// 
    /// # Biome Resource Design
    /// 
    /// Each biome has unique resource distributions reflecting its environment:
    /// - **Forest**: Abundant food (berries, nuts), moderate water
    /// - **Desert**: Scarce food, valuable water sources (cacti)
    /// - **Grassland**: Balanced resources (seeds, water)
    /// - **Tundra**: Moderate food (fish, berries), some water
    /// - **Ocean**: Rich food (marine life), less useful water
    /// 
    /// This creates strategic value in different biomes and encourages
    /// creature migration patterns based on their current needs.
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
    /// 
    /// Multiplier for resource spawn rates and amounts:
    /// - 1.0 = baseline abundance
    /// - >1.0 = more resources than average
    /// - <1.0 = fewer resources than average
    pub fn get_biome_abundance(biome: BiomeType) -> f32 {
        match biome {
            BiomeType::Forest => 1.2,      // 20% more resources (lush environment)
            BiomeType::Desert => 0.6,      // 40% fewer resources (harsh conditions)
            BiomeType::Grassland => 1.0,   // Baseline abundance
            BiomeType::Tundra => 0.7,      // 30% fewer resources (cold limits growth)
            BiomeType::Ocean => 1.1,       // 10% more resources (rich marine life)
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
    
    /// Get biome at tile coordinates with enhanced multi-layer generation
    pub fn get_biome_tile(&mut self, tile_pos: IVec2) -> BiomeType {
        self.get_biome_data(tile_pos).biome_type
    }
    
    /// Get full biome data including environmental values
    pub fn get_biome_data(&mut self, tile_pos: IVec2) -> BiomeData {
        // Check cache first
        if let Some(data) = self.biome_cache.get(&tile_pos) {
            return data.clone();
        }
        
        // Multi-scale sampling for more interesting terrain
        let coarse_scale = 0.01;  // Continental scale
        let medium_scale = 0.02;  // Regional scale  
        let fine_scale = 0.05;    // Local variations
        
        let x = tile_pos.x as f64;
        let y = tile_pos.y as f64;
        
        // Sample noise at different scales
        let temp_coarse = self.temperature_noise.get([x * coarse_scale, y * coarse_scale]) as f32;
        let moist_coarse = self.moisture_noise.get([x * coarse_scale, y * coarse_scale]) as f32;
        let elevation = self.elevation_noise.get([x * medium_scale, y * medium_scale]) as f32;
        let detail = self.detail_noise.get([x * fine_scale, y * fine_scale]) as f32;
        
        // Combine scales with elevation influence
        let temperature = temp_coarse - elevation * 0.3; // Higher = colder
        let moisture = moist_coarse + elevation * 0.1;   // Higher = slightly wetter
        
        // Enhanced biome selection with elevation consideration
        let biome_type = self.select_biome(temperature, moisture, elevation);
        
        let biome_data = BiomeData {
            biome_type,
            temperature,
            moisture,
            elevation,
            variation: detail,
        };
        
        // Cache the result
        self.biome_cache.insert(tile_pos, biome_data.clone());
        biome_data
    }
    
    /// Enhanced biome selection with more nuanced rules
    fn select_biome(&self, temperature: f32, moisture: f32, elevation: f32) -> BiomeType {
        // Ocean at very low elevation
        if elevation < -0.3 {
            return BiomeType::Ocean;
        }
        
        // Mountain peaks become tundra regardless of latitude
        if elevation > 0.6 {
            return BiomeType::Tundra;
        }
        
        // Temperature-moisture grid with smooth transitions
        match (temperature, moisture) {
            // Hot regions
            (t, m) if t > 0.4 => {
                if m < -0.2 { BiomeType::Desert }
                else if m > 0.3 { BiomeType::Forest }
                else { BiomeType::Grassland }
            },
            // Cold regions
            (t, m) if t < -0.3 => {
                if m > 0.1 { BiomeType::Tundra }
                else { BiomeType::Grassland } // Cold steppes
            },
            // Temperate regions
            (_, m) => {
                if m > 0.4 { BiomeType::Forest }
                else if m < -0.3 { BiomeType::Desert }
                else { BiomeType::Grassland }
            },
        }
    }
    
    /// Calculate biome transition data for smooth borders
    pub fn get_transition_data(&mut self, tile_pos: IVec2) -> TransitionData {
        // Check cache
        if let Some(data) = self.transition_cache.get(&tile_pos) {
            return data.clone();
        }
        
        let center_biome = self.get_biome_data(tile_pos);
        let mut neighbor_biomes = Vec::new();
        let mut min_distance = f32::MAX;
        let mut secondary_biome = None;
        
        // Check surrounding tiles for different biomes
        for dy in -2..=2 {
            for dx in -2..=2 {
                if dx == 0 && dy == 0 { continue; }
                
                let neighbor_pos = tile_pos + IVec2::new(dx, dy);
                let neighbor_data = self.get_biome_data(neighbor_pos);
                
                if neighbor_data.biome_type != center_biome.biome_type {
                    let distance = ((dx * dx + dy * dy) as f32).sqrt();
                    if distance < min_distance {
                        min_distance = distance;
                        secondary_biome = Some(neighbor_data.biome_type);
                    }
                    neighbor_biomes.push((neighbor_data.biome_type, distance));
                }
            }
        }
        
        // Calculate blend factor based on distance to nearest different biome
        let blend_factor = if min_distance <= 1.5 {
            1.0 - (min_distance - 1.0).max(0.0) / 0.5
        } else {
            0.0
        };
        
        let transition_data = TransitionData {
            primary_biome: center_biome.biome_type,
            secondary_biome,
            blend_factor,
            edge_distance: min_distance,
        };
        
        self.transition_cache.insert(tile_pos, transition_data.clone());
        transition_data
    }
    
    /// Clear cache for memory management
    pub fn clear_distant_cache(&mut self, center: IVec2, radius: i32) {
        self.biome_cache.retain(|&pos, _| {
            (pos.x - center.x).abs() <= radius && (pos.y - center.y).abs() <= radius
        });
        self.transition_cache.retain(|&pos, _| {
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
    pub elevation: f32,
    pub has_decoration: bool,
}

/// Component for tile decorations (rocks, plants, etc)
#[derive(Component)]
pub struct TileDecoration {
    pub decoration_type: DecorationType,
    pub parent_tile: Entity,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DecorationType {
    // Forest decorations
    Tree,
    Bush,
    Mushroom,
    // Desert decorations  
    Cactus,
    Rock,
    DeadTree,
    // Grassland decorations
    Flowers,
    TallGrass,
    // Tundra decorations
    IceRock,
    SnowDrift,
    // Ocean decorations
    Coral,
    Seaweed,
}

/// Bundle for spawning terrain tiles
#[derive(Bundle)]
pub struct TerrainTileBundle {
    pub tile: TerrainTile,
    pub sprite: SpriteBundle,
    pub name: Name,
}

/// Get appropriate decorations for a biome
fn get_biome_decorations(biome: BiomeType) -> Vec<(DecorationType, f32)> {
    match biome {
        BiomeType::Forest => vec![
            (DecorationType::Tree, 0.3),
            (DecorationType::Bush, 0.2),
            (DecorationType::Mushroom, 0.1),
        ],
        BiomeType::Desert => vec![
            (DecorationType::Cactus, 0.15),
            (DecorationType::Rock, 0.2),
            (DecorationType::DeadTree, 0.05),
        ],
        BiomeType::Grassland => vec![
            (DecorationType::Flowers, 0.25),
            (DecorationType::TallGrass, 0.3),
        ],
        BiomeType::Tundra => vec![
            (DecorationType::IceRock, 0.1),
            (DecorationType::SnowDrift, 0.15),
        ],
        BiomeType::Ocean => vec![
            (DecorationType::Coral, 0.1),
            (DecorationType::Seaweed, 0.2),
        ],
    }
}

/// Enhanced chunk data for better organization
#[derive(Default)]
pub struct ChunkData {
    pub tiles: HashMap<IVec2, Entity>,
    pub decorations: HashMap<IVec2, Vec<Entity>>,
    pub chunk_coord: IVec2,
}

/// System to generate visible terrain chunks around the camera
/// 
/// This system:
/// 1. Determines which tiles are visible based on camera position
/// 2. Spawns terrain tile entities for newly visible areas
/// 3. Despawns tiles that have moved too far from view
/// 4. Manages the biome cache to prevent memory growth
/// 
/// # Chunk Management Strategy
/// 
/// The system maintains a view radius of 20 tiles (1280 pixels) around
/// the camera. This provides good visibility without overwhelming the
/// renderer with too many tiles.
/// 
/// ## Tile Generation
/// 
/// For each visible tile position:
/// 1. Check if tile already exists (skip if so)
/// 2. Query biome type from BiomeMap
/// 3. Calculate isometric screen position
/// 4. Choose visual variant (0-3) using position hash
/// 5. Spawn colored sprite at correct depth
/// 
/// ## Cleanup Strategy  
/// 
/// Tiles beyond view_radius + 10 are despawned to free memory.
/// The extra buffer prevents visible popping during movement.
/// 
/// # Performance Optimizations
/// 
/// - Only processes tiles in view frustum
/// - Caches biome calculations
/// - Batches entity spawning
/// - Uses simple hash for tile variants
/// - Cleans distant cache entries
/// 
/// # Visual Variety
/// 
/// Tile variants use a deterministic hash function:
/// `variant = (x * 7 + y * 13) % 4`
/// 
/// This creates pseudo-random patterns that remain consistent
/// across game sessions for the same world seed.
pub fn generate_terrain_chunks(
    mut commands: Commands,
    mut biome_map: ResMut<BiomeMap>,
    camera_query: Query<&Transform, With<crate::plugins::MainCamera>>,
    _existing_tiles: Query<&TerrainTile>,
    _chunk_data: Local<HashMap<IVec2, ChunkData>>,
    mut tile_entities: Local<HashMap<IVec2, Entity>>,
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
    // View radius in tiles - balance between visual range and performance
    // 20 tiles = roughly 1280 pixels at standard zoom
    let view_radius = 20;
    let center_tile = world_to_tile(camera_world);
    
    // Generate tiles in visible range
    for dy in -view_radius..=view_radius {
        for dx in -view_radius..=view_radius {
            let tile_coord = IVec2::new(center_tile.x + dx, center_tile.y + dy);
            
            // Skip if tile already exists
            if tile_entities.contains_key(&tile_coord) {
                continue;
            }
            
            // Get full biome data for this tile
            let biome_data = biome_map.get_biome_data(tile_coord);
            let transition_data = biome_map.get_transition_data(tile_coord);
            
            // Calculate world position with elevation
            let elevation_offset = biome_data.elevation * 5.0; // Visual elevation scaling
            let world_pos = tile_to_world(tile_coord) + Vec3::new(0.0, elevation_offset, 0.0);
            let screen_pos = world_to_screen(world_pos);
            
            // Choose tile variant with more variety
            let variant_hash = ((tile_coord.x * 7 + tile_coord.y * 13) ^ (biome_data.variation * 100.0) as i32) % 8;
            let tile_variant = variant_hash.abs() as u8;
            
            // Determine if this tile should have decoration
            let decoration_roll = ((tile_coord.x * 11 + tile_coord.y * 17) % 100) as f32 / 100.0;
            let should_decorate = decoration_roll < 0.3 && transition_data.blend_factor < 0.1;
            
            // Get tile color with transition blending
            let base_color = get_biome_color(biome_data.biome_type);
            let tile_color = if let Some(secondary) = transition_data.secondary_biome {
                let secondary_color = get_biome_color(secondary);
                Color::rgba(
                    base_color.r() * (1.0 - transition_data.blend_factor) + secondary_color.r() * transition_data.blend_factor,
                    base_color.g() * (1.0 - transition_data.blend_factor) + secondary_color.g() * transition_data.blend_factor,
                    base_color.b() * (1.0 - transition_data.blend_factor) + secondary_color.b() * transition_data.blend_factor,
                    1.0
                )
            } else {
                base_color
            };
            
            // Apply elevation shading
            let elevation_shade = 1.0 - biome_data.elevation.abs() * 0.2;
            let final_color = Color::rgba(
                tile_color.r() * elevation_shade,
                tile_color.g() * elevation_shade,
                tile_color.b() * elevation_shade,
                tile_color.a()
            );
            
            // Spawn tile entity
            let tile_entity = commands.spawn(TerrainTileBundle {
                tile: TerrainTile {
                    biome: biome_data.biome_type,
                    tile_coord,
                    tile_variant,
                    elevation: biome_data.elevation,
                    has_decoration: should_decorate,
                },
                sprite: SpriteBundle {
                    sprite: Sprite {
                        color: final_color,
                        custom_size: Some(Vec2::new(64.0, 32.0)), // Isometric tile size
                        ..default()
                    },
                    transform: Transform::from_xyz(screen_pos.x, screen_pos.y, -100.0 + elevation_offset), // Adjust Z for elevation
                    ..default()
                },
                name: Name::new(format!("Tile({}, {})", tile_coord.x, tile_coord.y)),
            }).id();
            
            tile_entities.insert(tile_coord, tile_entity);
            
            // Spawn decorations if appropriate
            if should_decorate {
                spawn_tile_decorations(
                    &mut commands,
                    tile_entity,
                    tile_coord,
                    biome_data.biome_type,
                    screen_pos,
                    &biome_data,
                );
            }
        }
    }
    
    // Clean up distant tiles
    // Extra buffer prevents visible tile popping during movement
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

/// Spawn decorations for a tile based on biome type
fn spawn_tile_decorations(
    commands: &mut Commands,
    tile_entity: Entity,
    tile_coord: IVec2,
    biome: BiomeType,
    base_screen_pos: Vec2,
    biome_data: &BiomeData,
) {
    let decorations = get_biome_decorations(biome);
    
    // Use deterministic random for decoration selection
    let decoration_seed = (tile_coord.x * 31 + tile_coord.y * 37) as f32;
    let decoration_roll = (decoration_seed % 100.0) / 100.0;
    
    // Select decoration type based on weighted probabilities
    let mut cumulative = 0.0;
    for (decoration_type, probability) in decorations {
        cumulative += probability;
        if decoration_roll < cumulative {
            // Random offset within tile for variety
            let offset_x = ((tile_coord.x * 23 + tile_coord.y * 29) % 20) as f32 - 10.0;
            let offset_y = ((tile_coord.x * 19 + tile_coord.y * 31) % 10) as f32 - 5.0;
            
            let decoration_pos = Vec2::new(
                base_screen_pos.x + offset_x,
                base_screen_pos.y + offset_y + 10.0, // Slight vertical offset
            );
            
            let decoration_color = get_decoration_color(decoration_type, biome_data);
            let decoration_size = get_decoration_size(decoration_type);
            
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: decoration_color,
                        custom_size: Some(decoration_size),
                        ..default()
                    },
                    transform: Transform::from_xyz(
                        decoration_pos.x,
                        decoration_pos.y,
                        -90.0 + biome_data.elevation * 5.0, // Above tile but below entities
                    ),
                    ..default()
                },
                TileDecoration {
                    decoration_type,
                    parent_tile: tile_entity,
                },
                Name::new(format!("{:?} at ({}, {})", decoration_type, tile_coord.x, tile_coord.y)),
            ));
            
            break;
        }
    }
}

/// Get color for decoration based on type and environment
fn get_decoration_color(decoration: DecorationType, biome_data: &BiomeData) -> Color {
    let base_color = match decoration {
        DecorationType::Tree => Color::rgb(0.2, 0.5, 0.2),
        DecorationType::Bush => Color::rgb(0.3, 0.6, 0.3),
        DecorationType::Mushroom => Color::rgb(0.8, 0.4, 0.3),
        DecorationType::Cactus => Color::rgb(0.4, 0.7, 0.4),
        DecorationType::Rock => Color::rgb(0.5, 0.5, 0.5),
        DecorationType::DeadTree => Color::rgb(0.4, 0.3, 0.2),
        DecorationType::Flowers => Color::rgb(0.9, 0.7, 0.9),
        DecorationType::TallGrass => Color::rgb(0.5, 0.8, 0.3),
        DecorationType::IceRock => Color::rgb(0.8, 0.85, 0.9),
        DecorationType::SnowDrift => Color::rgb(0.95, 0.95, 1.0),
        DecorationType::Coral => Color::rgb(0.9, 0.5, 0.6),
        DecorationType::Seaweed => Color::rgb(0.2, 0.6, 0.4),
    };
    
    // Apply environmental variations
    let variation = biome_data.variation * 0.1;
    Color::rgb(
        (base_color.r() + variation).clamp(0.0, 1.0),
        (base_color.g() + variation * 0.5).clamp(0.0, 1.0),
        (base_color.b() - variation * 0.5).clamp(0.0, 1.0),
    )
}

/// Get size for decoration sprite
fn get_decoration_size(decoration: DecorationType) -> Vec2 {
    match decoration {
        DecorationType::Tree => Vec2::new(24.0, 32.0),
        DecorationType::Bush => Vec2::new(16.0, 12.0),
        DecorationType::Mushroom => Vec2::new(8.0, 8.0),
        DecorationType::Cactus => Vec2::new(12.0, 20.0),
        DecorationType::Rock => Vec2::new(12.0, 8.0),
        DecorationType::DeadTree => Vec2::new(20.0, 24.0),
        DecorationType::Flowers => Vec2::new(10.0, 8.0),
        DecorationType::TallGrass => Vec2::new(8.0, 12.0),
        DecorationType::IceRock => Vec2::new(14.0, 10.0),
        DecorationType::SnowDrift => Vec2::new(16.0, 6.0),
        DecorationType::Coral => Vec2::new(10.0, 12.0),
        DecorationType::Seaweed => Vec2::new(8.0, 16.0),
    }
}

/// Plugin for biome and terrain systems
pub struct BiomePlugin;

impl Plugin for BiomePlugin {
    fn build(&self, app: &mut App) {
        // Use a fixed seed for now (can be randomized later)
        // TODO: Make seed configurable or random for production
        let biome_map = BiomeMap::new(12345);
        
        app.insert_resource(biome_map)
            .add_systems(Update, (generate_terrain_chunks, update_tile_visuals).chain());
    }
}

/// System to update tile visuals based on camera distance (LOD)
fn update_tile_visuals(
    camera_query: Query<&Transform, With<crate::plugins::MainCamera>>,
    mut tile_query: Query<(&TerrainTile, &mut Sprite, &Transform)>,
) {
    let Ok(camera_transform) = camera_query.get_single() else { return };
    let camera_pos = Vec2::new(camera_transform.translation.x, camera_transform.translation.y);
    
    for (_tile, mut sprite, transform) in tile_query.iter_mut() {
        let tile_pos = Vec2::new(transform.translation.x, transform.translation.y);
        let distance = camera_pos.distance(tile_pos);
        
        // Simple LOD: fade distant tiles
        let alpha = if distance > 800.0 {
            0.5 + (1000.0 - distance) / 400.0
        } else {
            1.0
        };
        
        sprite.color.set_a(alpha.clamp(0.5, 1.0));
    }
}