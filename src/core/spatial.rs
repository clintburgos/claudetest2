//! Spatial partitioning system for efficient proximity queries.
//!
//! This module implements a uniform grid spatial index that divides
//! the world into fixed-size cells. This allows O(1) insertion/removal
//! and efficient radius queries that only check nearby cells.
//!
//! # Performance Characteristics
//! - Insert: O(1)
//! - Remove: O(1)
//! - Update: O(1)
//! - Radius query: O(k) where k is entities in nearby cells
//! - Memory: O(n) where n is number of entities

use crate::config::spatial::*;
use crate::core::Entity;
use crate::Vec2;
use ahash::AHashMap as HashMap;
use ahash::AHashSet as HashSet;

/// Represents a grid coordinate in the spatial partitioning system
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct GridCoord {
    pub x: i32,
    pub y: i32,
}

impl GridCoord {
    /// Creates a new grid coordinate
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// Statistics for spatial grid performance monitoring
#[derive(Debug, Default, Clone)]
pub struct SpatialStats {
    pub total_queries: u64,
    pub cells_checked: u64,
    pub entities_checked: u64,
}

/// Spatial partitioning grid for efficient proximity queries.
///
/// The grid divides 2D space into uniform cells, with each entity
/// stored in exactly one cell based on its position. This structure
/// enables fast "find all entities within radius R of point P" queries
/// by only checking cells that intersect the search area.
///
/// # Design Decisions
/// - **Uniform grid**: Simple and cache-friendly
/// - **Single cell per entity**: No boundary straddling complexity
/// - **ahash HashMap**: Fast hashing for grid coordinates
/// - **Pre-allocated buffers**: Reduces allocation in hot paths
///
/// # Usage Patterns
/// ```
/// use creature_simulation::core::{SpatialGrid, Entity};
/// use creature_simulation::Vec2;
///
/// let mut grid = SpatialGrid::new(50.0); // Cell size based on typical query radius
/// let entity = Entity::new(1);
/// grid.insert(entity, Vec2::new(100.0, 100.0));
/// let nearby = grid.query_radius(Vec2::new(100.0, 100.0), 25.0);
/// ```
///
/// # Cell Size Selection
/// - Too small: Many cells checked per query
/// - Too large: Many entities per cell to filter
/// - Optimal: ~2x typical query radius
pub struct SpatialGrid {
    cell_size: f32,
    cells: HashMap<GridCoord, Vec<Entity>>,
    pub(crate) entity_positions: HashMap<Entity, (GridCoord, Vec2)>,
    /// Pre-allocated buffer for query results
    query_buffer: Vec<Entity>,
    /// Statistics for profiling
    stats: SpatialStats,
}

impl SpatialGrid {
    /// Creates a new spatial grid with the specified cell size
    ///
    /// # Arguments
    /// * `cell_size` - Size of each grid cell (must be positive)
    ///
    /// # Panics
    /// Panics if cell_size <= 0
    pub fn new(cell_size: f32) -> Self {
        assert!(cell_size > 0.0, "Cell size must be positive");

        Self {
            cell_size,
            cells: HashMap::with_capacity(CELL_CAPACITY),
            entity_positions: HashMap::with_capacity(ENTITY_CAPACITY),
            query_buffer: Vec::with_capacity(QUERY_BUFFER_CAPACITY),
            stats: SpatialStats::default(),
        }
    }

    /// Inserts or updates an entity's position
    ///
    /// # Arguments
    /// * `entity` - The entity to insert/update
    /// * `position` - The world position
    pub fn insert(&mut self, entity: Entity, position: Vec2) {
        let coord = self.world_to_grid(position);

        // Remove from old cell if exists
        if let Some((old_coord, _)) = self.entity_positions.get(&entity) {
            if let Some(entities) = self.cells.get_mut(old_coord) {
                entities.retain(|&e| e != entity);
                if entities.is_empty() {
                    self.cells.remove(old_coord);
                }
            }
        }

        // Add to new cell
        self.cells.entry(coord).or_default().push(entity);
        self.entity_positions.insert(entity, (coord, position));
    }

    /// Removes an entity from the grid
    ///
    /// # Arguments
    /// * `entity` - The entity to remove
    ///
    /// # Returns
    /// The entity's last position if it existed
    pub fn remove(&mut self, entity: Entity) -> Option<Vec2> {
        if let Some((coord, position)) = self.entity_positions.remove(&entity) {
            if let Some(entities) = self.cells.get_mut(&coord) {
                entities.retain(|&e| e != entity);
                if entities.is_empty() {
                    self.cells.remove(&coord);
                }
            }
            Some(position)
        } else {
            None
        }
    }

    /// Gets an entity's current position
    pub fn get_position(&self, entity: Entity) -> Option<Vec2> {
        self.entity_positions.get(&entity).map(|(_, pos)| *pos)
    }

    /// Performs bulk position updates for better cache efficiency
    ///
    /// # Arguments
    /// * `updates` - Slice of (entity, position) pairs to update
    pub fn update_bulk(&mut self, updates: &[(Entity, Vec2)]) {
        for &(entity, position) in updates {
            self.insert(entity, position);
        }
    }

    /// Queries for entities within a radius (immutable version)
    ///
    /// # Arguments
    /// * `center` - Center point of the query
    /// * `radius` - Search radius
    ///
    /// # Returns
    /// Vector of entities within the radius
    pub fn query_radius(&self, center: Vec2, radius: f32) -> Vec<Entity> {
        let mut result = Vec::new();
        let mut seen = HashSet::new();

        // Clamp radius to prevent excessive iteration
        let clamped_radius = radius.min(10000.0);

        let radius_squared = clamped_radius * clamped_radius;
        let min_coord = self.world_to_grid(center - Vec2::splat(clamped_radius));
        let max_coord = self.world_to_grid(center + Vec2::splat(clamped_radius));

        for x in min_coord.x..=max_coord.x {
            for y in min_coord.y..=max_coord.y {
                if let Some(entities) = self.cells.get(&GridCoord { x, y }) {
                    for &entity in entities {
                        if seen.insert(entity) {
                            if let Some((_, pos)) = self.entity_positions.get(&entity) {
                                let dist_squared = (*pos - center).length_squared();
                                if dist_squared <= radius_squared {
                                    result.push(entity);
                                }
                            }
                        }
                    }
                }
            }
        }

        result
    }

    /// Queries for entities within a radius (mutable version with stats)
    ///
    /// # Arguments
    /// * `center` - Center point of the query
    /// * `radius` - Search radius
    ///
    /// # Returns
    /// Vector of entities within the radius
    pub fn query_radius_mut(&mut self, center: Vec2, radius: f32) -> Vec<Entity> {
        self.query_buffer.clear();
        let mut seen = HashSet::new();

        // Clamp radius to prevent excessive iteration
        let clamped_radius = radius.min(10000.0);

        let radius_squared = clamped_radius * clamped_radius;
        let min_coord = self.world_to_grid(center - Vec2::splat(clamped_radius));
        let max_coord = self.world_to_grid(center + Vec2::splat(clamped_radius));

        // Update statistics
        self.stats.total_queries += 1;
        let cells_to_check =
            ((max_coord.x - min_coord.x + 1) * (max_coord.y - min_coord.y + 1)) as u64;
        self.stats.cells_checked += cells_to_check;

        for x in min_coord.x..=max_coord.x {
            for y in min_coord.y..=max_coord.y {
                if let Some(entities) = self.cells.get(&GridCoord { x, y }) {
                    self.stats.entities_checked += entities.len() as u64;
                    for &entity in entities {
                        if seen.insert(entity) {
                            if let Some((_, pos)) = self.entity_positions.get(&entity) {
                                let dist_squared = (*pos - center).length_squared();
                                if dist_squared <= radius_squared {
                                    self.query_buffer.push(entity);
                                }
                            }
                        }
                    }
                }
            }
        }

        self.query_buffer.clone()
    }

    /// Queries for entities within a rectangle
    ///
    /// # Arguments
    /// * `min` - Minimum corner of the rectangle
    /// * `max` - Maximum corner of the rectangle
    ///
    /// # Returns
    /// Vector of entities within the rectangle
    pub fn query_rect(&mut self, min: Vec2, max: Vec2) -> Vec<Entity> {
        self.query_buffer.clear();
        let mut seen = HashSet::new();

        let min_coord = self.world_to_grid(min);
        let max_coord = self.world_to_grid(max);

        // Update statistics
        self.stats.total_queries += 1;
        let cells_to_check =
            ((max_coord.x - min_coord.x + 1) * (max_coord.y - min_coord.y + 1)) as u64;
        self.stats.cells_checked += cells_to_check;

        for x in min_coord.x..=max_coord.x {
            for y in min_coord.y..=max_coord.y {
                if let Some(entities) = self.cells.get(&GridCoord { x, y }) {
                    self.stats.entities_checked += entities.len() as u64;
                    for &entity in entities {
                        if seen.insert(entity) {
                            if let Some((_, pos)) = self.entity_positions.get(&entity) {
                                if pos.x >= min.x
                                    && pos.x <= max.x
                                    && pos.y >= min.y
                                    && pos.y <= max.y
                                {
                                    self.query_buffer.push(entity);
                                }
                            }
                        }
                    }
                }
            }
        }

        self.query_buffer.clone()
    }

    /// Queries entities in a specific cell
    pub fn query_cell(&self, coord: GridCoord) -> &[Entity] {
        self.cells.get(&coord).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Clears all entities from the grid
    pub fn clear(&mut self) {
        self.cells.clear();
        self.entity_positions.clear();
        self.query_buffer.clear();
        self.stats = SpatialStats::default();
    }

    /// Returns the number of entities in the grid
    pub fn entity_count(&self) -> usize {
        self.entity_positions.len()
    }

    /// Returns the number of occupied cells
    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }

    /// Returns current query statistics for profiling
    ///
    /// # Returns
    /// Tuple of (total_queries, cells_checked, entities_checked)
    pub fn stats(&self) -> (u64, u64, u64) {
        (
            self.stats.total_queries,
            self.stats.cells_checked,
            self.stats.entities_checked,
        )
    }

    /// Resets query statistics
    pub fn reset_stats(&mut self) {
        self.stats = SpatialStats::default();
    }

    /// Converts world position to grid coordinate
    fn world_to_grid(&self, pos: Vec2) -> GridCoord {
        GridCoord {
            x: (pos.x / self.cell_size).floor() as i32,
            y: (pos.y / self.cell_size).floor() as i32,
        }
    }

    /// Iterator over all tracked entities
    pub fn iter_entities(&self) -> impl Iterator<Item = Entity> + '_ {
        self.entity_positions.keys().copied()
    }

    /// Iterator over all entity positions
    pub fn iter_positions(&self) -> impl Iterator<Item = (Entity, Vec2)> + '_ {
        self.entity_positions.iter().map(|(e, (_, pos))| (*e, *pos))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spatial_grid_basic() {
        let mut grid = SpatialGrid::new(10.0);
        let entity = Entity::new(1);
        let pos = Vec2::new(5.0, 5.0);

        grid.insert(entity, pos);
        assert_eq!(grid.entity_count(), 1);
        assert_eq!(grid.get_position(entity), Some(pos));
    }

    #[test]
    fn spatial_grid_movement() {
        let mut grid = SpatialGrid::new(10.0);
        let entity = Entity::new(1);

        grid.insert(entity, Vec2::new(5.0, 5.0));
        assert_eq!(grid.cell_count(), 1);

        // Move to different cell
        grid.insert(entity, Vec2::new(15.0, 15.0));
        assert_eq!(grid.entity_count(), 1);
        assert_eq!(grid.get_position(entity), Some(Vec2::new(15.0, 15.0)));
    }

    #[test]
    fn spatial_grid_query_radius() {
        let mut grid = SpatialGrid::new(10.0);

        let e1 = Entity::new(1);
        let e2 = Entity::new(2);
        let e3 = Entity::new(3);

        grid.insert(e1, Vec2::new(0.0, 0.0));
        grid.insert(e2, Vec2::new(5.0, 0.0));
        grid.insert(e3, Vec2::new(20.0, 0.0));

        let nearby = grid.query_radius(Vec2::new(0.0, 0.0), 10.0);
        assert_eq!(nearby.len(), 2);
        assert!(nearby.contains(&e1));
        assert!(nearby.contains(&e2));
        assert!(!nearby.contains(&e3));
    }

    #[test]
    fn spatial_grid_query_rect() {
        let mut grid = SpatialGrid::new(10.0);

        let e1 = Entity::new(1);
        let e2 = Entity::new(2);
        let e3 = Entity::new(3);

        grid.insert(e1, Vec2::new(5.0, 5.0));
        grid.insert(e2, Vec2::new(15.0, 15.0));
        grid.insert(e3, Vec2::new(25.0, 25.0));

        let in_rect = grid.query_rect(Vec2::new(0.0, 0.0), Vec2::new(20.0, 20.0));
        assert_eq!(in_rect.len(), 2);
        assert!(in_rect.contains(&e1));
        assert!(in_rect.contains(&e2));
        assert!(!in_rect.contains(&e3));
    }

    #[test]
    fn spatial_grid_bulk_update() {
        let mut grid = SpatialGrid::new(10.0);

        let updates = vec![
            (Entity::new(1), Vec2::new(5.0, 5.0)),
            (Entity::new(2), Vec2::new(15.0, 15.0)),
            (Entity::new(3), Vec2::new(25.0, 25.0)),
        ];

        grid.update_bulk(&updates);

        assert_eq!(grid.entity_count(), 3);
        assert_eq!(grid.get_position(Entity::new(1)), Some(Vec2::new(5.0, 5.0)));
        assert_eq!(
            grid.get_position(Entity::new(2)),
            Some(Vec2::new(15.0, 15.0))
        );
        assert_eq!(
            grid.get_position(Entity::new(3)),
            Some(Vec2::new(25.0, 25.0))
        );
    }

    #[test]
    fn spatial_grid_statistics() {
        let mut grid = SpatialGrid::new(10.0);

        // Add some entities
        for i in 0..10 {
            grid.insert(Entity::new(i), Vec2::new(i as f32 * 5.0, 0.0));
        }

        // Perform queries and check stats
        grid.query_radius_mut(Vec2::new(25.0, 0.0), 20.0);
        let (queries, cells, entities) = grid.stats();
        assert_eq!(queries, 1);
        assert!(cells > 0);
        assert!(entities > 0);

        // Reset and verify
        grid.reset_stats();
        let (queries, cells, entities) = grid.stats();
        assert_eq!(queries, 0);
        assert_eq!(cells, 0);
        assert_eq!(entities, 0);
    }

    #[test]
    fn spatial_grid_remove() {
        let mut grid = SpatialGrid::new(10.0);
        let entity = Entity::new(1);
        let pos = Vec2::new(5.0, 5.0);

        grid.insert(entity, pos);
        assert_eq!(grid.remove(entity), Some(pos));
        assert_eq!(grid.entity_count(), 0);
        assert_eq!(grid.cell_count(), 0);
    }

    #[test]
    fn grid_coord_new() {
        let coord = GridCoord::new(5, -3);
        assert_eq!(coord.x, 5);
        assert_eq!(coord.y, -3);
    }

    #[test]
    fn spatial_grid_query_cell() {
        let mut grid = SpatialGrid::new(10.0);
        let e1 = Entity::new(1);
        let e2 = Entity::new(2);

        // Insert in same cell
        grid.insert(e1, Vec2::new(5.0, 5.0));
        grid.insert(e2, Vec2::new(8.0, 8.0));

        let coord = GridCoord::new(0, 0);
        let entities = grid.query_cell(coord);
        assert_eq!(entities.len(), 2);

        // Query empty cell
        let empty_coord = GridCoord::new(10, 10);
        let empty = grid.query_cell(empty_coord);
        assert_eq!(empty.len(), 0);
    }

    #[test]
    fn spatial_grid_clear() {
        let mut grid = SpatialGrid::new(10.0);

        grid.insert(Entity::new(1), Vec2::new(5.0, 5.0));
        grid.insert(Entity::new(2), Vec2::new(15.0, 15.0));

        assert_eq!(grid.entity_count(), 2);
        assert!(grid.cell_count() > 0);

        grid.clear();

        assert_eq!(grid.entity_count(), 0);
        assert_eq!(grid.cell_count(), 0);
    }

    #[test]
    #[should_panic(expected = "Cell size must be positive")]
    fn spatial_grid_invalid_cell_size() {
        let _ = SpatialGrid::new(0.0);
    }

    #[test]
    fn spatial_grid_query_radius_immutable() {
        let mut grid = SpatialGrid::new(10.0);

        // Add some entities
        grid.insert(Entity::new(1), Vec2::new(5.0, 5.0));
        grid.insert(Entity::new(2), Vec2::new(15.0, 5.0));
        grid.insert(Entity::new(3), Vec2::new(50.0, 50.0));

        // Test immutable query (should not update stats)
        let initial_stats = grid.stats();
        let results = grid.query_radius(Vec2::new(10.0, 5.0), 10.0);
        let after_stats = grid.stats();

        assert_eq!(results.len(), 2); // Should find entities 1 and 2
        assert_eq!(initial_stats, after_stats); // Stats should not change
    }

    #[test]
    fn spatial_grid_get_position() {
        let mut grid = SpatialGrid::new(10.0);
        let entity = Entity::new(1);
        let pos = Vec2::new(25.0, 30.0);

        // Before insertion
        assert_eq!(grid.get_position(entity), None);

        // After insertion
        grid.insert(entity, pos);
        assert_eq!(grid.get_position(entity), Some(pos));

        // After removal
        grid.remove(entity);
        assert_eq!(grid.get_position(entity), None);
    }

    #[test]
    fn spatial_grid_update_bulk() {
        let mut grid = SpatialGrid::new(10.0);

        let updates = vec![
            (Entity::new(1), Vec2::new(5.0, 5.0)),
            (Entity::new(2), Vec2::new(15.0, 15.0)),
            (Entity::new(3), Vec2::new(25.0, 25.0)),
        ];

        grid.update_bulk(&updates);

        assert_eq!(grid.entity_count(), 3);
        assert_eq!(grid.get_position(Entity::new(1)), Some(Vec2::new(5.0, 5.0)));
        assert_eq!(
            grid.get_position(Entity::new(2)),
            Some(Vec2::new(15.0, 15.0))
        );
        assert_eq!(
            grid.get_position(Entity::new(3)),
            Some(Vec2::new(25.0, 25.0))
        );
    }

    #[test]
    fn spatial_grid_iter_entities() {
        let mut grid = SpatialGrid::new(10.0);

        grid.insert(Entity::new(1), Vec2::new(5.0, 5.0));
        grid.insert(Entity::new(2), Vec2::new(15.0, 15.0));
        grid.insert(Entity::new(3), Vec2::new(25.0, 25.0));

        let entities: Vec<Entity> = grid.iter_entities().collect();
        assert_eq!(entities.len(), 3);
        assert!(entities.contains(&Entity::new(1)));
        assert!(entities.contains(&Entity::new(2)));
        assert!(entities.contains(&Entity::new(3)));
    }

    #[test]
    fn spatial_grid_iter_positions() {
        let mut grid = SpatialGrid::new(10.0);

        let e1 = Entity::new(1);
        let e2 = Entity::new(2);
        let p1 = Vec2::new(5.0, 5.0);
        let p2 = Vec2::new(15.0, 15.0);

        grid.insert(e1, p1);
        grid.insert(e2, p2);

        let positions: Vec<(Entity, Vec2)> = grid.iter_positions().collect();
        assert_eq!(positions.len(), 2);
        assert!(positions.contains(&(e1, p1)));
        assert!(positions.contains(&(e2, p2)));
    }

    #[test]
    fn spatial_grid_edge_cases() {
        let mut grid = SpatialGrid::new(10.0);

        // Test negative coordinates
        let neg_pos = Vec2::new(-15.0, -25.0);
        grid.insert(Entity::new(1), neg_pos);
        assert_eq!(grid.get_position(Entity::new(1)), Some(neg_pos));

        // Test very large coordinates
        let large_pos = Vec2::new(10000.0, 10000.0);
        grid.insert(Entity::new(2), large_pos);
        assert_eq!(grid.get_position(Entity::new(2)), Some(large_pos));

        // Test updating same entity multiple times
        let entity = Entity::new(3);
        grid.insert(entity, Vec2::new(0.0, 0.0));
        grid.insert(entity, Vec2::new(10.0, 10.0));
        grid.insert(entity, Vec2::new(20.0, 20.0));
        assert_eq!(grid.entity_count(), 3); // Only 3 entities total
        assert_eq!(grid.get_position(entity), Some(Vec2::new(20.0, 20.0)));
    }
}
