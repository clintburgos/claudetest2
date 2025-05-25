use crate::Vec2;
use crate::core::{World, Entity};
use crate::simulation::{Creature, CreatureState, ResourceType};
use crate::simulation::needs::NeedType;
use log::debug;

/// Decision system responsible for creature AI and behavior selection
/// 
/// Handles:
/// - Need evaluation and prioritization
/// - Resource discovery and selection
/// - Behavior state transitions
/// - Basic decision-making logic
pub struct DecisionSystem {
    /// Maximum distance to search for resources
    search_radius: f32,
    /// Minimum need urgency to trigger action
    urgency_threshold: f32,
    /// Time between decision updates per creature
    decision_interval: f32,
}

/// Represents a potential decision target
#[derive(Debug, Clone)]
struct DecisionTarget {
    entity: Entity,
    position: Vec2,
    distance: f32,
    value: f32,
}

impl DecisionSystem {
    /// Creates a new decision system
    pub fn new() -> Self {
        Self {
            search_radius: 50.0,
            urgency_threshold: 0.3,
            decision_interval: 1.0, // Reconsider decisions every second
        }
    }
    
    /// Updates decisions for all creatures
    pub fn update(&mut self, world: &mut World) {
        // Collect creature decisions to avoid borrowing conflicts
        let mut decisions = Vec::new();
        
        for (&entity, creature) in &world.creatures {
            if !creature.is_alive() {
                continue;
            }
            
            // Skip if creature is busy with an action
            match creature.state {
                CreatureState::Eating | CreatureState::Drinking | CreatureState::Resting => {
                    continue;
                }
                _ => {}
            }
            
            // Make decision based on needs
            if let Some(decision) = self.make_decision(entity, creature, world) {
                decisions.push((entity, decision));
            }
        }
        
        // Apply decisions
        for (entity, decision) in decisions {
            self.apply_decision(entity, decision, world);
        }
    }
    
    /// Makes a decision for a single creature
    fn make_decision(&self, entity: Entity, creature: &Creature, world: &World) -> Option<Decision> {
        // Check if any needs are urgent
        let most_urgent = creature.needs.most_urgent();
        let urgency = creature.needs.get_urgency(most_urgent);
        
        if urgency < self.urgency_threshold {
            // No urgent needs, idle or wander
            return if creature.state == CreatureState::Idle {
                Some(Decision::Wander)
            } else {
                None
            };
        }
        
        debug!("Creature {:?} has urgent need: {:?} (urgency: {:.2})", 
               entity, most_urgent, urgency);
        
        // Find appropriate resource for the need
        match most_urgent {
            NeedType::Hunger => {
                if let Some(target) = self.find_nearest_resource(
                    creature.position, 
                    ResourceType::Food, 
                    world
                ) {
                    Some(Decision::GoToResource { 
                        resource: target.entity, 
                        position: target.position,
                        action: PlannedAction::Eat 
                    })
                } else {
                    Some(Decision::SearchForResource { 
                        resource_type: ResourceType::Food 
                    })
                }
            }
            NeedType::Thirst => {
                if let Some(target) = self.find_nearest_resource(
                    creature.position, 
                    ResourceType::Water, 
                    world
                ) {
                    Some(Decision::GoToResource { 
                        resource: target.entity, 
                        position: target.position,
                        action: PlannedAction::Drink 
                    })
                } else {
                    Some(Decision::SearchForResource { 
                        resource_type: ResourceType::Water 
                    })
                }
            }
            NeedType::Energy => {
                // Find safe spot to rest
                Some(Decision::Rest)
            }
        }
    }
    
    /// Finds the nearest resource of a given type
    fn find_nearest_resource(
        &self, 
        position: Vec2, 
        resource_type: ResourceType, 
        world: &World
    ) -> Option<DecisionTarget> {
        let mut best_target = None;
        let mut best_score = f32::MAX;
        
        // Query spatial grid for nearby entities
        // Note: In Phase 1, we'll iterate through all resources instead of using spatial query
        // to avoid mutable borrow issues. This is acceptable for 500 creatures.
        let mut candidates = Vec::new();
        
        for (&entity, resource) in &world.resources {
            if resource.resource_type == resource_type && !resource.is_depleted() {
                let distance = (resource.position - position).length();
                if distance <= self.search_radius {
                    candidates.push((entity, resource, distance));
                }
            }
        }
        
        for (entity, resource, distance) in candidates {
            // Score based on distance and resource amount
            let score = distance / resource.percentage();
            
            if score < best_score {
                best_score = score;
                best_target = Some(DecisionTarget {
                    entity,
                    position: resource.position,
                    distance,
                    value: resource.amount,
                });
            }
        }
        
        best_target
    }
    
    /// Applies a decision to a creature
    fn apply_decision(&self, entity: Entity, decision: Decision, world: &mut World) {
        // Extract position first to avoid borrow conflicts
        let creature_pos = world.creatures.get(&entity).map(|c| c.position);
        
        if let Some(pos) = creature_pos {
            match decision {
                Decision::GoToResource { position, .. } => {
                    debug!("Creature {:?} moving to resource at {:?}", entity, position);
                    if let Some(creature) = world.creatures.get_mut(&entity) {
                        creature.start_moving(position);
                    }
                }
                Decision::SearchForResource { resource_type } => {
                    debug!("Creature {:?} searching for {:?}", entity, resource_type);
                    // Move in a random direction to search
                    let search_dir = self.random_search_direction(pos, world);
                    if let Some(creature) = world.creatures.get_mut(&entity) {
                        creature.start_moving(creature.position + search_dir * 20.0);
                    }
                }
                Decision::Rest => {
                    debug!("Creature {:?} resting", entity);
                    if let Some(creature) = world.creatures.get_mut(&entity) {
                        creature.start_resting();
                    }
                }
                Decision::Wander => {
                    let wander_target = self.random_wander_target(pos, world);
                    if let Some(creature) = world.creatures.get_mut(&entity) {
                        if matches!(creature.state, CreatureState::Idle) {
                            creature.start_moving(wander_target);
                        }
                    }
                }
            }
        }
    }
    
    /// Generates a random search direction
    fn random_search_direction(&self, current_pos: Vec2, _world: &World) -> Vec2 {
        // For Phase 1, use a simple approach
        // In a full implementation, this would use the RNG from the world
        let angle = (current_pos.x + current_pos.y) * 0.1; // Pseudo-random based on position
        Vec2::new(angle.cos(), angle.sin())
    }
    
    /// Generates a random wander target
    fn random_wander_target(&self, current_pos: Vec2, world: &World) -> Vec2 {
        let offset = self.random_search_direction(current_pos, world) * 10.0;
        let target = current_pos + offset;
        
        // Clamp to world bounds if they exist
        if let Some(bounds) = &world.bounds {
            bounds.clamp(target)
        } else {
            target
        }
    }
    
    /// Checks if a creature is near a resource
    pub fn check_resource_interaction(&self, world: &mut World) {
        let interaction_distance = 2.0;
        let mut interactions = Vec::new();
        
        // Find creatures that are near resources
        for (&creature_entity, creature) in &world.creatures {
            if !creature.is_alive() {
                continue;
            }
            
            // Check if creature is in appropriate state
            let needed_resource = match creature.state {
                CreatureState::Moving { .. } => {
                    // Determine what resource we're looking for based on needs
                    match creature.needs.most_urgent() {
                        NeedType::Hunger => Some(ResourceType::Food),
                        NeedType::Thirst => Some(ResourceType::Water),
                        _ => None,
                    }
                }
                _ => None,
            };
            
            if let Some(resource_type) = needed_resource {
                // Check nearby resources
                // Note: Simple iteration for Phase 1
                let creature_pos = creature.position;
                
                for (&resource_entity, resource) in &world.resources {
                    if resource.resource_type == resource_type && !resource.is_depleted() {
                        let distance = (resource.position - creature_pos).length();
                        if distance <= interaction_distance {
                            interactions.push((creature_entity, resource_entity, resource_type));
                            break; // Only interact with one resource at a time
                        }
                    }
                }
            }
        }
        
        // Process interactions
        for (creature_entity, _resource_entity, resource_type) in interactions {
            if let Some(creature) = world.creatures.get_mut(&creature_entity) {
                match resource_type {
                    ResourceType::Food => creature.start_eating(),
                    ResourceType::Water => creature.start_drinking(),
                }
                debug!("Creature {:?} started interacting with {:?}", 
                       creature_entity, resource_type);
            }
        }
    }
}

impl Default for DecisionSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a decision made by the AI
#[derive(Debug, Clone)]
enum Decision {
    GoToResource {
        resource: Entity,
        position: Vec2,
        action: PlannedAction,
    },
    SearchForResource {
        resource_type: ResourceType,
    },
    Rest,
    Wander,
}

/// Planned action to execute when reaching a target
#[derive(Debug, Clone, Copy)]
enum PlannedAction {
    Eat,
    Drink,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::{Creature, Resource};
    
    fn create_test_world() -> World {
        World::with_bounds(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0))
    }
    
    #[test]
    fn decision_system_urgent_needs() {
        let mut world = create_test_world();
        let mut decision_system = DecisionSystem::new();
        
        // Create hungry creature
        let entity = world.entities.create();
        let mut creature = Creature::new(entity, Vec2::new(50.0, 50.0));
        creature.needs.hunger = 0.8; // Very hungry
        world.creatures.insert(entity, creature);
        
        // Create food resource nearby
        let food_entity = world.entities.create();
        let food = Resource::new(food_entity, Vec2::new(60.0, 50.0), ResourceType::Food);
        world.resources.insert(food_entity, food);
        world.spatial_grid.insert(food_entity, Vec2::new(60.0, 50.0));
        
        // Update decisions
        decision_system.update(&mut world);
        
        // Creature should be moving towards food
        let creature = &world.creatures[&entity];
        assert!(matches!(creature.state, CreatureState::Moving { .. }));
    }
    
    #[test]
    fn decision_system_no_urgent_needs() {
        let mut world = create_test_world();
        let mut decision_system = DecisionSystem::new();
        
        // Create satisfied creature
        let entity = world.entities.create();
        let creature = Creature::new(entity, Vec2::new(50.0, 50.0));
        world.creatures.insert(entity, creature);
        
        // Update decisions
        decision_system.update(&mut world);
        
        // Creature should wander
        let creature = &world.creatures[&entity];
        assert!(matches!(creature.state, CreatureState::Moving { .. }));
    }
    
    #[test]
    fn decision_system_rest_when_tired() {
        let mut world = create_test_world();
        let mut decision_system = DecisionSystem::new();
        
        // Create tired creature
        let entity = world.entities.create();
        let mut creature = Creature::new(entity, Vec2::new(50.0, 50.0));
        creature.needs.energy = 0.1; // Very tired
        world.creatures.insert(entity, creature);
        
        // Update decisions
        decision_system.update(&mut world);
        
        // Creature should rest
        let creature = &world.creatures[&entity];
        assert_eq!(creature.state, CreatureState::Resting);
    }
    
    #[test]
    fn decision_system_resource_interaction() {
        let mut world = create_test_world();
        let decision_system = DecisionSystem::new();
        
        // Create creature near food
        let entity = world.entities.create();
        let mut creature = Creature::new(entity, Vec2::new(50.0, 50.0));
        creature.needs.hunger = 0.8;
        creature.state = CreatureState::Moving { target: Vec2::new(51.0, 50.0) };
        world.creatures.insert(entity, creature);
        world.spatial_grid.insert(entity, Vec2::new(50.0, 50.0));
        
        // Create food very close
        let food_entity = world.entities.create();
        let food = Resource::new(food_entity, Vec2::new(51.0, 50.0), ResourceType::Food);
        world.resources.insert(food_entity, food);
        world.spatial_grid.insert(food_entity, Vec2::new(51.0, 50.0));
        
        // Check interactions
        decision_system.check_resource_interaction(&mut world);
        
        // Creature should start eating
        let creature = &world.creatures[&entity];
        assert_eq!(creature.state, CreatureState::Eating);
    }
    
    #[test]
    fn decision_system_find_nearest_resource() {
        let mut world = create_test_world();
        let decision_system = DecisionSystem::new();
        
        // Create multiple food resources
        let food1 = world.entities.create();
        world.resources.insert(food1, Resource::new(food1, Vec2::new(60.0, 50.0), ResourceType::Food));
        world.spatial_grid.insert(food1, Vec2::new(60.0, 50.0));
        
        let food2 = world.entities.create();
        world.resources.insert(food2, Resource::new(food2, Vec2::new(40.0, 50.0), ResourceType::Food));
        world.spatial_grid.insert(food2, Vec2::new(40.0, 50.0));
        
        // Find nearest from center
        let nearest = decision_system.find_nearest_resource(
            Vec2::new(50.0, 50.0),
            ResourceType::Food,
            &world
        );
        
        assert!(nearest.is_some());
        // Both are equidistant, so either is valid
        let target = nearest.unwrap();
        assert!(target.entity == food1 || target.entity == food2);
    }
}