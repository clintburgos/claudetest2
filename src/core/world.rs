use crate::core::{Entity, EntityManager, SpatialGrid, EventBus, GameTime, ErrorBoundary};
use crate::simulation::{Creature, Resource};
use ahash::AHashMap as HashMap;

pub struct World {
    pub entities: EntityManager,
    pub creatures: HashMap<Entity, Creature>,
    pub resources: HashMap<Entity, Resource>,
    pub spatial_grid: SpatialGrid,
    pub events: EventBus,
    pub time: GameTime,
    pub error_boundary: ErrorBoundary,
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: EntityManager::new(),
            creatures: HashMap::new(),
            resources: HashMap::new(),
            spatial_grid: SpatialGrid::new(50.0), // 50-unit cells for ~10 creatures per cell
            events: EventBus::new(),
            time: GameTime::new(),
            error_boundary: ErrorBoundary::new(),
        }
    }
    
    pub fn with_cell_size(cell_size: f32) -> Self {
        let mut world = Self::new();
        world.spatial_grid = SpatialGrid::new(cell_size);
        world
    }
    
    pub fn creature_count(&self) -> usize {
        self.creatures.len()
    }
    
    pub fn resource_count(&self) -> usize {
        self.resources.len()
    }
    
    pub fn total_entity_count(&self) -> usize {
        self.entities.active_count()
    }
    
    pub fn clear(&mut self) {
        self.creatures.clear();
        self.resources.clear();
        self.spatial_grid.clear();
        self.events.clear();
        // Note: We don't clear entities as that would invalidate IDs
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
}