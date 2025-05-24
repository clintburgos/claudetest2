use crate::Vec2;
use crate::core::Entity;
use ahash::AHashMap as HashMap;
use ahash::AHashSet as HashSet;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct GridCoord {
    pub x: i32,
    pub y: i32,
}

impl GridCoord {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

pub struct SpatialGrid {
    cell_size: f32,
    cells: HashMap<GridCoord, Vec<Entity>>,
    pub(crate) entity_positions: HashMap<Entity, (GridCoord, Vec2)>,
}

impl SpatialGrid {
    pub fn new(cell_size: f32) -> Self {
        assert!(cell_size > 0.0, "Cell size must be positive");
        
        Self {
            cell_size,
            cells: HashMap::new(),
            entity_positions: HashMap::new(),
        }
    }
    
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
    
    pub fn get_position(&self, entity: Entity) -> Option<Vec2> {
        self.entity_positions.get(&entity).map(|(_, pos)| *pos)
    }
    
    pub fn query_radius(&self, center: Vec2, radius: f32) -> Vec<Entity> {
        let mut results = Vec::new();
        let mut seen = HashSet::new();
        
        let radius_squared = radius * radius;
        let min_coord = self.world_to_grid(center - Vec2::splat(radius));
        let max_coord = self.world_to_grid(center + Vec2::splat(radius));
        
        for x in min_coord.x..=max_coord.x {
            for y in min_coord.y..=max_coord.y {
                if let Some(entities) = self.cells.get(&GridCoord { x, y }) {
                    for &entity in entities {
                        if seen.insert(entity) {
                            if let Some((_, pos)) = self.entity_positions.get(&entity) {
                                let dist_squared = (*pos - center).length_squared();
                                if dist_squared <= radius_squared {
                                    results.push(entity);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        results
    }
    
    pub fn query_rect(&self, min: Vec2, max: Vec2) -> Vec<Entity> {
        let mut results = Vec::new();
        let mut seen = HashSet::new();
        
        let min_coord = self.world_to_grid(min);
        let max_coord = self.world_to_grid(max);
        
        for x in min_coord.x..=max_coord.x {
            for y in min_coord.y..=max_coord.y {
                if let Some(entities) = self.cells.get(&GridCoord { x, y }) {
                    for &entity in entities {
                        if seen.insert(entity) {
                            if let Some((_, pos)) = self.entity_positions.get(&entity) {
                                if pos.x >= min.x && pos.x <= max.x && 
                                   pos.y >= min.y && pos.y <= max.y {
                                    results.push(entity);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        results
    }
    
    pub fn query_cell(&self, coord: GridCoord) -> &[Entity] {
        self.cells.get(&coord).map(|v| v.as_slice()).unwrap_or(&[])
    }
    
    pub fn clear(&mut self) {
        self.cells.clear();
        self.entity_positions.clear();
    }
    
    pub fn entity_count(&self) -> usize {
        self.entity_positions.len()
    }
    
    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }
    
    fn world_to_grid(&self, pos: Vec2) -> GridCoord {
        GridCoord {
            x: (pos.x / self.cell_size).floor() as i32,
            y: (pos.y / self.cell_size).floor() as i32,
        }
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
    fn spatial_grid_remove() {
        let mut grid = SpatialGrid::new(10.0);
        let entity = Entity::new(1);
        let pos = Vec2::new(5.0, 5.0);
        
        grid.insert(entity, pos);
        assert_eq!(grid.remove(entity), Some(pos));
        assert_eq!(grid.entity_count(), 0);
        assert_eq!(grid.cell_count(), 0);
    }
}