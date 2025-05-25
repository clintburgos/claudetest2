//! Central world state container for the simulation.
//! 
//! The World struct serves as the main data container, holding all
//! entities, components, and systems. It provides a unified interface
//! for querying and modifying the simulation state.
//! 
//! # Architecture
//! The World follows an ECS-like pattern where:
//! - Entities are just IDs
//! - Components (Creature, Resource) are stored in separate HashMaps
//! - Systems operate on the World to update state

use crate::Vec2;
use crate::core::{Entity, EntityManager, SpatialGrid, EventBus, GameTime, ErrorBoundary};
use crate::simulation::{Creature, Resource};
use crate::config::spatial::DEFAULT_CELL_SIZE;
use ahash::AHashMap as HashMap;

/// World boundaries for creature movement
#[derive(Debug, Clone)]
pub struct WorldBounds {
    pub min: Vec2,
    pub max: Vec2,
}

impl WorldBounds {
    /// Creates new world bounds
    pub fn new(min: Vec2, max: Vec2) -> Self {
        assert!(min.x < max.x && min.y < max.y, "Invalid bounds: min must be less than max");
        Self { min, max }
    }
    
    /// Checks if a position is within bounds
    pub fn contains(&self, pos: Vec2) -> bool {
        pos.x >= self.min.x && pos.x <= self.max.x &&
        pos.y >= self.min.y && pos.y <= self.max.y
    }
    
    /// Clamps a position to be within bounds
    pub fn clamp(&self, pos: Vec2) -> Vec2 {
        Vec2::new(
            pos.x.clamp(self.min.x, self.max.x),
            pos.y.clamp(self.min.y, self.max.y)
        )
    }
    
    /// Returns the size of the world
    pub fn size(&self) -> Vec2 {
        self.max - self.min
    }
    
    /// Returns the center of the world
    pub fn center(&self) -> Vec2 {
        (self.min + self.max) * 0.5
    }
}

/// World statistics for performance monitoring
#[derive(Debug, Default, Clone)]
pub struct WorldStats {
    pub creature_count: usize,
    pub resource_count: usize,
    pub events_processed: u64,
    pub update_time_ms: f32,
}

/// Central world state containing all entities and systems.
/// 
/// The World struct is the heart of the simulation, containing:
/// - All game entities (creatures, resources)
/// - Spatial indexing for efficient queries
/// - Event system for decoupled communication
/// - Error handling and recovery
/// - Performance statistics
/// 
/// # Usage
/// Systems should take `&mut World` and query/modify components as needed.
/// The World ensures consistency and provides helper methods for common
/// operations like spatial queries.
/// 
/// # Thread Safety
/// The World is not thread-safe. Systems should run sequentially,
/// though they may use parallel iteration internally.
pub struct World {
    pub entities: EntityManager,
    pub creatures: HashMap<Entity, Creature>,
    pub resources: HashMap<Entity, Resource>,
    pub spatial_grid: SpatialGrid,
    pub events: EventBus,
    pub time: GameTime,
    pub error_boundary: ErrorBoundary,
    /// Optional world boundaries for creature movement
    pub bounds: Option<WorldBounds>,
    /// Performance statistics
    pub stats: WorldStats,
}

impl World {
    /// Creates a new world with default settings
    pub fn new() -> Self {
        Self {
            entities: EntityManager::new(),
            creatures: HashMap::new(),
            resources: HashMap::new(),
            spatial_grid: SpatialGrid::new(DEFAULT_CELL_SIZE),
            events: EventBus::new(),
            time: GameTime::new(),
            error_boundary: ErrorBoundary::new(),
            bounds: None,
            stats: WorldStats::default(),
        }
    }
    
    /// Creates a world with custom cell size
    pub fn with_cell_size(cell_size: f32) -> Self {
        assert!(cell_size > 0.0, "Cell size must be positive");
        let mut world = Self::new();
        world.spatial_grid = SpatialGrid::new(cell_size);
        world
    }
    
    /// Creates a world with bounds
    pub fn with_bounds(min: Vec2, max: Vec2) -> Self {
        let mut world = Self::new();
        world.bounds = Some(WorldBounds::new(min, max));
        world
    }
    
    /// Returns the number of creatures
    pub fn creature_count(&self) -> usize {
        self.creatures.len()
    }
    
    /// Returns the number of resources
    pub fn resource_count(&self) -> usize {
        self.resources.len()
    }
    
    /// Returns the total number of active entities
    pub fn total_entity_count(&self) -> usize {
        self.entities.active_count()
    }
    
    /// Updates world statistics
    pub fn update_stats(&mut self) {
        self.stats.creature_count = self.creatures.len();
        self.stats.resource_count = self.resources.len();
    }
    
    /// Finds resources near a position using spatial grid for efficiency
    /// 
    /// # Arguments
    /// * `position` - Center position for search
    /// * `radius` - Search radius
    /// * `resource_type` - Type of resource to find
    /// 
    /// # Returns
    /// Vector of (entity, distance) pairs sorted by distance
    pub fn find_resources_near(
        &self, 
        position: Vec2, 
        radius: f32, 
        resource_type: crate::simulation::ResourceType
    ) -> Vec<(Entity, f32)> {
        let mut results = Vec::new();
        
        // Use spatial grid for efficient queries
        let entities = self.spatial_grid.query_radius(position, radius);
        
        for &entity in &entities {
            if let Some(resource) = self.resources.get(&entity) {
                if resource.resource_type == resource_type && !resource.is_depleted() {
                    let distance = (resource.position - position).length();
                    results.push((entity, distance));
                }
            }
        }
        
        // Sort by distance
        results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        results
    }
    
    /// Finds nearest resource of a specific type
    /// 
    /// # Arguments
    /// * `position` - Center position for search
    /// * `resource_type` - Type of resource to find
    /// * `max_radius` - Maximum search radius (None for unlimited)
    /// 
    /// # Returns
    /// Option of (entity, distance) for nearest resource
    pub fn find_nearest_resource(
        &self,
        position: Vec2,
        resource_type: crate::simulation::ResourceType,
        max_radius: Option<f32>
    ) -> Option<(Entity, f32)> {
        let search_radius = max_radius.unwrap_or(f32::MAX);
        self.find_resources_near(position, search_radius, resource_type)
            .into_iter()
            .next()
    }
    
    /// Clears all simulation data (but preserves entity IDs)
    pub fn clear(&mut self) {
        self.creatures.clear();
        self.resources.clear();
        self.spatial_grid.clear();
        self.events.clear();
        self.stats = WorldStats::default();
        // Note: We don't clear entities as that would invalidate IDs
        // Note: We don't clear bounds as they're part of world definition
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Vec2;

    #[test]
    fn world_new() {
        let world = World::new();
        assert_eq!(world.creature_count(), 0);
        assert_eq!(world.resource_count(), 0);
        assert_eq!(world.total_entity_count(), 0);
    }
    
    #[test]
    fn world_with_cell_size() {
        let mut world = World::with_cell_size(25.0);
        assert_eq!(world.creature_count(), 0);
        
        // Test that spatial grid works with custom cell size
        let entity = world.entities.create();
        world.spatial_grid.insert(entity, Vec2::new(30.0, 30.0));
        assert_eq!(world.spatial_grid.entity_count(), 1);
    }
    
    #[test]
    fn world_entity_counts() {
        let mut world = World::new();
        
        // Create some entities
        let e1 = world.entities.create();
        let e2 = world.entities.create();
        let e3 = world.entities.create();
        
        // Add creatures
        world.creatures.insert(e1, Creature::new(e1, Vec2::ZERO));
        world.creatures.insert(e2, Creature::new(e2, Vec2::ZERO));
        
        // Add resource
        world.resources.insert(e3, Resource::new(e3, Vec2::ZERO, crate::simulation::ResourceType::Food));
        
        assert_eq!(world.creature_count(), 2);
        assert_eq!(world.resource_count(), 1);
        assert_eq!(world.total_entity_count(), 3);
    }
    
    #[test]
    fn world_clear() {
        let mut world = World::new();
        
        // Add some data
        let e1 = world.entities.create();
        let e2 = world.entities.create();
        
        world.creatures.insert(e1, Creature::new(e1, Vec2::ZERO));
        world.resources.insert(e2, Resource::new(e2, Vec2::ZERO, crate::simulation::ResourceType::Water));
        world.spatial_grid.insert(e1, Vec2::new(5.0, 5.0));
        world.spatial_grid.insert(e2, Vec2::new(10.0, 10.0));
        world.events.emit(crate::core::GameEvent::CreatureSpawned { entity: e1, position: Vec2::ZERO });
        
        // Clear everything
        world.clear();
        
        assert_eq!(world.creature_count(), 0);
        assert_eq!(world.resource_count(), 0);
        assert_eq!(world.spatial_grid.entity_count(), 0);
        assert!(world.events.is_empty());
        
        // Note: entities are NOT cleared to preserve ID validity
        assert_eq!(world.total_entity_count(), 2);
    }
    
    #[test]
    fn world_default() {
        let world = World::default();
        assert_eq!(world.creature_count(), 0);
        assert_eq!(world.resource_count(), 0);
        assert_eq!(world.total_entity_count(), 0);
    }
    
    #[test]
    fn world_with_bounds() {
        let world = World::with_bounds(Vec2::ZERO, Vec2::new(100.0, 100.0));
        assert!(world.bounds.is_some());
        
        let bounds = world.bounds.as_ref().unwrap();
        assert_eq!(bounds.min, Vec2::ZERO);
        assert_eq!(bounds.max, Vec2::new(100.0, 100.0));
    }
    
    #[test]
    fn world_find_resources_near() {
        let mut world = World::new();
        
        // Create resources at different positions
        let food1 = world.entities.create();
        let food2 = world.entities.create();
        let water = world.entities.create();
        let far_food = world.entities.create();
        
        // Add resources
        world.resources.insert(food1, Resource::new(food1, Vec2::new(10.0, 10.0), crate::simulation::ResourceType::Food));
        world.resources.insert(food2, Resource::new(food2, Vec2::new(15.0, 15.0), crate::simulation::ResourceType::Food));
        world.resources.insert(water, Resource::new(water, Vec2::new(12.0, 12.0), crate::simulation::ResourceType::Water));
        world.resources.insert(far_food, Resource::new(far_food, Vec2::new(100.0, 100.0), crate::simulation::ResourceType::Food));
        
        // Update spatial grid
        world.spatial_grid.insert(food1, Vec2::new(10.0, 10.0));
        world.spatial_grid.insert(food2, Vec2::new(15.0, 15.0));
        world.spatial_grid.insert(water, Vec2::new(12.0, 12.0));
        world.spatial_grid.insert(far_food, Vec2::new(100.0, 100.0));
        
        // Find food resources near (10, 10) within radius 10
        let nearby_food = world.find_resources_near(Vec2::new(10.0, 10.0), 10.0, crate::simulation::ResourceType::Food);
        
        // Should find food1 and food2, but not water or far_food
        assert_eq!(nearby_food.len(), 2);
        assert_eq!(nearby_food[0].0, food1); // Closest first
        assert_eq!(nearby_food[1].0, food2);
        
        // Check distances are correct
        assert!((nearby_food[0].1 - 0.0).abs() < 0.001);
        assert!((nearby_food[1].1 - 7.071).abs() < 0.1); // sqrt(5^2 + 5^2)
    }
    
    #[test]
    fn world_find_nearest_resource() {
        let mut world = World::new();
        
        // Create resources
        let food1 = world.entities.create();
        let food2 = world.entities.create();
        
        world.resources.insert(food1, Resource::new(food1, Vec2::new(20.0, 20.0), crate::simulation::ResourceType::Food));
        world.resources.insert(food2, Resource::new(food2, Vec2::new(30.0, 30.0), crate::simulation::ResourceType::Food));
        
        world.spatial_grid.insert(food1, Vec2::new(20.0, 20.0));
        world.spatial_grid.insert(food2, Vec2::new(30.0, 30.0));
        
        // Find nearest from origin
        let nearest = world.find_nearest_resource(Vec2::ZERO, crate::simulation::ResourceType::Food, None);
        assert!(nearest.is_some());
        assert_eq!(nearest.unwrap().0, food1);
        
        // Find nearest with max radius that excludes both
        let nearest_limited = world.find_nearest_resource(Vec2::ZERO, crate::simulation::ResourceType::Food, Some(10.0));
        assert!(nearest_limited.is_none());
        
        // Find nearest water (none exists)
        let no_water = world.find_nearest_resource(Vec2::ZERO, crate::simulation::ResourceType::Water, None);
        assert!(no_water.is_none());
    }
    
    #[test]
    fn world_find_depleted_resources() {
        let mut world = World::new();
        
        // Create a depleted resource
        let food = world.entities.create();
        let mut resource = Resource::new(food, Vec2::new(10.0, 10.0), crate::simulation::ResourceType::Food);
        resource.amount = 0.0; // Deplete it
        
        world.resources.insert(food, resource);
        world.spatial_grid.insert(food, Vec2::new(10.0, 10.0));
        
        // Should not find depleted resources
        let nearby = world.find_resources_near(Vec2::new(10.0, 10.0), 20.0, crate::simulation::ResourceType::Food);
        assert_eq!(nearby.len(), 0);
    }
    
    #[test]
    fn world_update_stats() {
        let mut world = World::new();
        
        // Add some entities
        let e1 = world.entities.create();
        let e2 = world.entities.create();
        world.creatures.insert(e1, Creature::new(e1, Vec2::ZERO));
        world.resources.insert(e2, Resource::new(e2, Vec2::ZERO, crate::simulation::ResourceType::Food));
        
        // Update stats
        world.update_stats();
        
        assert_eq!(world.stats.creature_count, 1);
        assert_eq!(world.stats.resource_count, 1);
    }
    
    #[test]
    fn world_bounds_operations() {
        let bounds = WorldBounds::new(Vec2::new(-50.0, -50.0), Vec2::new(50.0, 50.0));
        
        // Test contains
        assert!(bounds.contains(Vec2::ZERO));
        assert!(bounds.contains(Vec2::new(49.0, 49.0)));
        assert!(!bounds.contains(Vec2::new(51.0, 0.0)));
        assert!(!bounds.contains(Vec2::new(0.0, -51.0)));
        
        // Test clamp
        assert_eq!(bounds.clamp(Vec2::new(0.0, 0.0)), Vec2::new(0.0, 0.0));
        assert_eq!(bounds.clamp(Vec2::new(100.0, 100.0)), Vec2::new(50.0, 50.0));
        assert_eq!(bounds.clamp(Vec2::new(-100.0, -100.0)), Vec2::new(-50.0, -50.0));
        
        // Test size
        assert_eq!(bounds.size(), Vec2::new(100.0, 100.0));
        
        // Test center
        assert_eq!(bounds.center(), Vec2::ZERO);
    }
    
    #[test]
    #[should_panic(expected = "Invalid bounds: min must be less than max")]
    fn world_bounds_invalid() {
        let _ = WorldBounds::new(Vec2::new(50.0, 50.0), Vec2::new(-50.0, -50.0));
    }
    
    #[test]
    #[should_panic(expected = "Cell size must be positive")]
    fn world_with_invalid_cell_size() {
        let _ = World::with_cell_size(0.0);
    }
}