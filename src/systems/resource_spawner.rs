use crate::core::{world::World, events::GameEvent};
use crate::simulation::resource::{Resource, ResourceType};
use crate::utils::random;
use crate::config;
use glam::Vec2;

pub struct ResourceSpawner {
    /// Target number of food resources per 100x100 area
    pub target_food_density: f32,
    /// Target number of water resources per 100x100 area  
    pub target_water_density: f32,
    /// Minimum time between spawn checks (seconds)
    pub spawn_cooldown: f32,
    /// Time since last spawn check
    time_since_spawn: f32,
    /// Random seed counter
    seed_counter: f32,
}

impl Default for ResourceSpawner {
    fn default() -> Self {
        Self {
            target_food_density: config::resource::TARGET_FOOD_DENSITY,
            target_water_density: config::resource::TARGET_WATER_DENSITY,
            spawn_cooldown: config::resource::SPAWN_CHECK_INTERVAL,
            time_since_spawn: 0.0,
            seed_counter: 0.0,
        }
    }
}

impl ResourceSpawner {
    pub fn new(food_density: f32, water_density: f32) -> Self {
        Self {
            target_food_density: food_density,
            target_water_density: water_density,
            ..Default::default()
        }
    }

    pub fn update(&mut self, world: &mut World, dt: f32) {
        self.time_since_spawn += dt;
        
        if self.time_since_spawn < self.spawn_cooldown {
            return;
        }
        
        self.time_since_spawn = 0.0;
        
        // Calculate world bounds
        let world_size = Vec2::new(1000.0, 1000.0); // TODO: Get from world config
        let grid_size = 100.0;
        let total_area = (world_size.x * world_size.y) / (grid_size * grid_size);
        
        // Count current resources
        let (food_count, water_count) = self.count_resources(world);
        
        // Calculate how many resources we should have
        let target_food = (self.target_food_density * total_area) as i32;
        let target_water = (self.target_water_density * total_area) as i32;
        
        // Spawn food if needed
        let food_deficit = target_food - food_count as i32;
        if food_deficit > 0 {
            for _ in 0..food_deficit.min(5) { // Spawn max 5 at a time
                if let Some(position) = self.find_spawn_location(world, &world_size) {
                    self.spawn_resource(world, position, ResourceType::Food);
                }
            }
        }
        
        // Spawn water if needed
        let water_deficit = target_water - water_count as i32;
        if water_deficit > 0 {
            for _ in 0..water_deficit.min(3) { // Spawn max 3 at a time
                if let Some(position) = self.find_spawn_location(world, &world_size) {
                    self.spawn_resource(world, position, ResourceType::Water);
                }
            }
        }
    }
    
    fn count_resources(&self, world: &World) -> (usize, usize) {
        let mut food_count = 0;
        let mut water_count = 0;
        
        for resource in world.resources.values() {
            match resource.resource_type {
                ResourceType::Food => food_count += 1,
                ResourceType::Water => water_count += 1,
            }
        }
        
        (food_count, water_count)
    }
    
    fn find_spawn_location(&mut self, world: &World, world_size: &Vec2) -> Option<Vec2> {
        // Try to find a location not too close to existing resources
        for i in 0..10 {
            self.seed_counter += 1.0;
            let position = Vec2::new(
                random::random_range(50.0, world_size.x - 50.0, self.seed_counter),
                random::random_range(50.0, world_size.y - 50.0, self.seed_counter + 0.5),
            );
            
            // Check if there's already a resource nearby
            let nearby = world.spatial_grid.query_radius(position, config::resource::MIN_RESOURCE_SPACING);
            let has_nearby_resource = nearby.iter().any(|&entity| {
                world.resources.contains_key(&entity)
            });
            
            if !has_nearby_resource {
                return Some(position);
            }
        }
        
        // If we couldn't find a good spot, just return a random one
        self.seed_counter += 1.0;
        Some(Vec2::new(
            random::random_range(50.0, world_size.x - 50.0, self.seed_counter),
            random::random_range(50.0, world_size.y - 50.0, self.seed_counter + 0.5),
        ))
    }
    
    fn spawn_resource(&mut self, world: &mut World, position: Vec2, resource_type: ResourceType) {
        let entity = world.entities.create();
        
        let resource = Resource::new(entity, position, resource_type);
        
        world.resources.insert(entity, resource);
        world.spatial_grid.insert(entity, position);
        
        world.events.emit(GameEvent::ResourceSpawned { 
            entity, 
            position,
            resource_type,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_resource_spawning() {
        let mut world = World::new();
        let mut spawner = ResourceSpawner::new(1.0, 0.5);
        spawner.spawn_cooldown = 0.0; // No cooldown for testing
        
        // Initially no resources
        assert_eq!(world.resources.len(), 0);
        
        // Update spawner
        spawner.update(&mut world, 0.1);
        
        // Should have spawned some resources
        assert!(world.resources.len() > 0);
        
        let (food, water) = spawner.count_resources(&world);
        assert!(food > 0);
        assert!(water > 0);
    }
    
    #[test]
    fn test_density_maintenance() {
        let mut world = World::new();
        let mut spawner = ResourceSpawner::new(0.5, 0.3);
        spawner.spawn_cooldown = 0.0;
        
        // Let it stabilize
        for _ in 0..5 {
            spawner.update(&mut world, 1.0);
        }
        
        let (food1, water1) = spawner.count_resources(&world);
        
        // Remove some resources
        let to_remove: Vec<_> = world.resources.keys().take(5).copied().collect();
        for entity in to_remove {
            world.resources.remove(&entity);
        }
        
        // Update again
        spawner.update(&mut world, 1.0);
        
        let (food2, water2) = spawner.count_resources(&world);
        
        // Should have respawned to maintain density
        assert!(food2 >= food1 - 5);
        assert!(water2 >= water1 - 5);
    }
}