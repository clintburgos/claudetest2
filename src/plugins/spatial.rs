//! Spatial indexing plugin for efficient queries

use crate::components::Position;
use bevy::prelude::*;
use bevy::transform::TransformSystem;
use std::collections::{HashMap, HashSet};

/// Plugin for spatial indexing system
pub struct SpatialPlugin;

impl Plugin for SpatialPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpatialGrid>().add_systems(
            PostUpdate,
            update_spatial_grid.before(TransformSystem::TransformPropagate),
        );
    }
}

/// Grid cell coordinates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridCoord {
    pub x: i32,
    pub y: i32,
}

/// Spatial grid for efficient neighbor queries
#[derive(Resource)]
pub struct SpatialGrid {
    cell_size: f32,
    /// Uses HashSet for O(1) removal instead of Vec
    cells: HashMap<GridCoord, HashSet<Entity>>,
    /// Track which cell each entity is in for fast updates
    entity_positions: HashMap<Entity, GridCoord>,
}

impl Default for SpatialGrid {
    fn default() -> Self {
        Self::new(50.0) // Default cell size
    }
}

impl SpatialGrid {
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            cells: HashMap::new(),
            entity_positions: HashMap::new(),
        }
    }

    /// Converts world position to grid coordinates
    fn world_to_grid(&self, pos: Vec2) -> GridCoord {
        GridCoord {
            x: (pos.x / self.cell_size).floor() as i32,
            y: (pos.y / self.cell_size).floor() as i32,
        }
    }

    /// Queries entities within a radius
    pub fn query_radius(&self, center: Vec2, radius: f32) -> Vec<Entity> {
        let mut results = HashSet::new();

        // Calculate grid bounds
        let min_coord = self.world_to_grid(center - Vec2::splat(radius));
        let max_coord = self.world_to_grid(center + Vec2::splat(radius));

        // Check all cells in range
        for x in min_coord.x..=max_coord.x {
            for y in min_coord.y..=max_coord.y {
                let coord = GridCoord { x, y };
                if let Some(entities) = self.cells.get(&coord) {
                    results.extend(entities);
                }
            }
        }

        results.into_iter().collect()
    }

    /// Queries entities within a radius with distance filtering
    pub fn query_radius_filtered(
        &self,
        center: Vec2,
        radius: f32,
        positions: &Query<&Position>,
    ) -> Vec<(Entity, f32)> {
        let mut results = Vec::new();
        let radius_squared = radius * radius;

        // Calculate grid bounds
        let min_coord = self.world_to_grid(center - Vec2::splat(radius));
        let max_coord = self.world_to_grid(center + Vec2::splat(radius));

        // Check all cells in range
        for x in min_coord.x..=max_coord.x {
            for y in min_coord.y..=max_coord.y {
                let coord = GridCoord { x, y };
                if let Some(entities) = self.cells.get(&coord) {
                    for &entity in entities {
                        if let Ok(pos) = positions.get(entity) {
                            let dist_squared = (pos.0 - center).length_squared();
                            if dist_squared <= radius_squared {
                                results.push((entity, dist_squared.sqrt()));
                            }
                        }
                    }
                }
            }
        }

        results
    }

    /// Clears the grid
    pub fn clear(&mut self) {
        self.cells.clear();
        self.entity_positions.clear();
    }
}

/// System to update spatial grid based on entity positions
fn update_spatial_grid(
    mut grid: ResMut<SpatialGrid>,
    query: Query<(Entity, &Position), Or<(Changed<Position>, Added<Position>)>>,
    mut removed: RemovedComponents<Position>,
) {
    // Remove despawned entities
    for entity in removed.read() {
        if let Some(old_coord) = grid.entity_positions.remove(&entity) {
            if let Some(cell) = grid.cells.get_mut(&old_coord) {
                cell.remove(&entity);
                if cell.is_empty() {
                    grid.cells.remove(&old_coord);
                }
            }
        }
    }

    // Update moved entities
    for (entity, pos) in query.iter() {
        let new_coord = grid.world_to_grid(pos.0);

        // Remove from old cell if moved
        if let Some(&old_coord) = grid.entity_positions.get(&entity) {
            if old_coord != new_coord {
                if let Some(cell) = grid.cells.get_mut(&old_coord) {
                    cell.remove(&entity);
                    if cell.is_empty() {
                        grid.cells.remove(&old_coord);
                    }
                }
            } else {
                continue; // No movement
            }
        }

        // Add to new cell
        grid.cells.entry(new_coord).or_default().insert(entity);
        grid.entity_positions.insert(entity, new_coord);
    }
}
