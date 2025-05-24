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