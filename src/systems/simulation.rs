use crate::core::{World, TimeSystem, SimulationError};
use crate::systems::{MovementSystem, DecisionSystem};
use crate::simulation::{CreatureState, needs::EnvironmentalFactors};
use log::{debug, info};
use std::time::Instant;

/// Main simulation orchestrator that manages all systems
/// 
/// Responsible for:
/// - System update order
/// - Fixed timestep integration
/// - Performance monitoring
/// - Error recovery
pub struct Simulation {
    pub world: World,
    pub time_system: TimeSystem,
    movement_system: MovementSystem,
    decision_system: DecisionSystem,
    /// Environmental factors for the simulation
    environment: EnvironmentalFactors,
    /// Frame counter for debugging
    frame_count: u64,
    /// Performance tracking
    last_update_time: Instant,
}

impl Simulation {
    /// Creates a new simulation instance
    pub fn new() -> Self {
        Self {
            world: World::new(),
            time_system: TimeSystem::new(),
            movement_system: MovementSystem::new(),
            decision_system: DecisionSystem::new(),
            environment: EnvironmentalFactors::default(),
            frame_count: 0,
            last_update_time: Instant::now(),
        }
    }
    
    /// Creates a simulation with world bounds
    pub fn with_bounds(width: f32, height: f32) -> Self {
        let mut sim = Self::new();
        sim.world = World::with_bounds(
            crate::Vec2::new(0.0, 0.0),
            crate::Vec2::new(width, height)
        );
        sim
    }
    
    /// Main update function - call this each frame
    /// 
    /// # Arguments
    /// * `real_dt` - Real time elapsed since last update in seconds
    /// 
    /// # Returns
    /// Number of fixed timesteps performed
    pub fn update(&mut self, real_dt: f32) -> u32 {
        let start_time = Instant::now();
        let mut steps = 0;
        const MAX_STEPS_PER_UPDATE: u32 = 10; // Prevent spiral of death
        
        // Fixed timestep update loop
        while let Some(dt) = self.time_system.fixed_update(real_dt) {
            // Skip invariant checking for now to avoid borrow issues
            // In a real implementation, this would be done differently
            
            // Update systems in correct order
            self.update_decisions();
            self.update_movement(dt);
            self.update_needs(dt);
            self.update_health(dt);
            self.update_resources(dt);
            self.process_interactions();
            self.process_events();
            
            self.frame_count += 1;
            steps += 1;
            
            // Update game time
            self.world.time.advance(dt);
            
            // Prevent spiral of death
            if steps >= MAX_STEPS_PER_UPDATE {
                break;
            }
        }
        
        // Update performance stats
        let update_duration = start_time.elapsed();
        self.world.stats.update_time_ms = update_duration.as_secs_f32() * 1000.0;
        self.world.update_stats();
        
        // Log performance warnings
        if update_duration.as_millis() > 16 {
            debug!("Slow frame: {}ms for {} steps", update_duration.as_millis(), steps);
        }
        
        steps
    }
    
    /// Updates creature decisions
    fn update_decisions(&mut self) {
        self.decision_system.update(&mut self.world);
    }
    
    /// Updates creature movement
    fn update_movement(&mut self, dt: f32) {
        self.movement_system.update(&mut self.world, dt);
    }
    
    /// Updates creature needs
    fn update_needs(&mut self, dt: f32) {
        let env = self.environment.clone();
        let mut updates = Vec::new();
        
        for (&entity, creature) in &self.world.creatures {
            if creature.is_alive() {
                updates.push((entity, creature.metabolism_rate()));
            }
        }
        
        for (entity, metabolism) in updates {
            if let Some(creature) = self.world.creatures.get_mut(&entity) {
                creature.needs.update(dt, metabolism, &env);
                creature.update_age(dt);
            }
        }
    }
    
    /// Updates creature health based on needs
    fn update_health(&mut self, dt: f32) {
        let mut health_changes = Vec::new();
        
        for (&entity, creature) in &self.world.creatures {
            if !creature.is_alive() {
                continue;
            }
            
            // Calculate health changes based on critical needs
            let mut damage = 0.0;
            let mut death_cause = None;
            
            if creature.needs.hunger >= 1.0 {
                damage += 10.0 * dt; // Starvation damage
                death_cause = Some(crate::core::events::DeathCause::Starvation);
            }
            
            if creature.needs.thirst >= 1.0 {
                damage += 15.0 * dt; // Dehydration damage (faster)
                death_cause = Some(crate::core::events::DeathCause::Dehydration);
            }
            
            if creature.needs.energy <= 0.0 {
                damage += 5.0 * dt; // Exhaustion damage
            }
            
            // Check for old age (creatures die after ~5 minutes at 60 FPS)
            if creature.age > 300.0 {
                damage += 1.0 * dt;
                death_cause = death_cause.or(Some(crate::core::events::DeathCause::OldAge));
            }
            
            if damage > 0.0 {
                health_changes.push((entity, damage, death_cause));
            }
        }
        
        // Apply health changes
        for (entity, damage, death_cause) in health_changes {
            if let Some(creature) = self.world.creatures.get_mut(&entity) {
                creature.health.damage(damage);
                
                if creature.health.is_dead() && creature.is_alive() {
                    creature.die();
                    let cause = death_cause.unwrap_or(crate::core::events::DeathCause::Unknown);
                    info!("Creature {:?} died from {:?}", entity, cause);
                    
                    // Emit death event
                    self.world.events.emit(crate::core::events::GameEvent::CreatureDied {
                        entity,
                        cause,
                    });
                }
            }
        }
    }
    
    /// Updates resource regeneration
    fn update_resources(&mut self, dt: f32) {
        for resource in self.world.resources.values_mut() {
            resource.regenerate(dt);
        }
    }
    
    /// Processes creature-resource interactions
    fn process_interactions(&mut self) {
        self.decision_system.check_resource_interaction(&mut self.world);
        
        // Process ongoing interactions
        let mut consumptions = Vec::new();
        
        for (&entity, creature) in &self.world.creatures {
            match creature.state {
                crate::simulation::CreatureState::Eating => {
                    // Find nearest food
                    if let Some(resource_entity) = self.find_nearest_resource_of_type(
                        creature.position,
                        crate::simulation::ResourceType::Food
                    ) {
                        consumptions.push((entity, resource_entity, crate::simulation::ResourceType::Food));
                    }
                }
                crate::simulation::CreatureState::Drinking => {
                    // Find nearest water
                    if let Some(resource_entity) = self.find_nearest_resource_of_type(
                        creature.position,
                        crate::simulation::ResourceType::Water
                    ) {
                        consumptions.push((entity, resource_entity, crate::simulation::ResourceType::Water));
                    }
                }
                crate::simulation::CreatureState::Resting => {
                    // Resting is handled in update_needs
                }
                _ => {}
            }
        }
        
        // Apply consumptions
        for (creature_entity, resource_entity, resource_type) in consumptions {
            if let Some(resource) = self.world.resources.get_mut(&resource_entity) {
                let rate = resource.resource_type.consumption_rate();
                let consumed = resource.consume(rate * self.time_system.fixed_timestep().unwrap_or(1.0/60.0));
                
                if consumed > 0.0 {
                    // Emit consumption event
                    self.world.events.emit(crate::core::events::GameEvent::ResourceConsumed {
                        creature: creature_entity,
                        resource: resource_entity,
                        amount: consumed,
                    });
                    
                    // Check if resource is depleted
                    if resource.is_depleted() {
                        self.world.events.emit(crate::core::events::GameEvent::ResourceDepleted {
                            entity: resource_entity,
                        });
                    }
                }
                
                if let Some(creature) = self.world.creatures.get_mut(&creature_entity) {
                    match resource_type {
                        crate::simulation::ResourceType::Food => {
                            // Food satisfaction: 1 unit of food = 2.0 hunger reduction
                            // This means eating 0.05 units/sec reduces hunger by 0.1/sec
                            let satisfaction = consumed * 2.0;
                            creature.needs.eat(satisfaction);
                            if resource.is_depleted() || creature.needs.hunger <= 0.1 {
                                creature.state = CreatureState::Idle; // Stop eating
                            }
                        }
                        crate::simulation::ResourceType::Water => {
                            // Water satisfaction: 1 unit of water = 1.0 thirst reduction
                            // This means drinking 0.1 units/sec reduces thirst by 0.1/sec  
                            let satisfaction = consumed * 1.0;
                            creature.needs.drink(satisfaction);
                            if resource.is_depleted() || creature.needs.thirst <= 0.1 {
                                creature.state = CreatureState::Idle; // Stop drinking
                            }
                        }
                    }
                }
            }
        }
    }
    
    /// Finds nearest resource of given type (helper)
    fn find_nearest_resource_of_type(
        &self,
        position: crate::Vec2,
        resource_type: crate::simulation::ResourceType
    ) -> Option<crate::core::Entity> {
        let mut nearest = None;
        let mut min_distance = f32::MAX;
        
        for (&entity, resource) in &self.world.resources {
            if resource.resource_type == resource_type && !resource.is_depleted() {
                let distance = (resource.position - position).length();
                if distance < min_distance && distance < 3.0 { // Interaction range
                    min_distance = distance;
                    nearest = Some(entity);
                }
            }
        }
        
        nearest
    }
    
    /// Processes events generated during the frame
    fn process_events(&mut self) {
        let mut event_count = 0;
        self.world.events.process(|event| {
            event_count += 1;
            debug!("Event: {:?}", event);
            // In Phase 1, we just log events
            // Future phases would handle them appropriately
        });
        
        self.world.stats.events_processed += event_count;
    }
    
    /// Handles simulation errors
    fn handle_error(&mut self, error: SimulationError) {
        let result = self.world.error_boundary.handle_error(
            error,
            self.world.time.total_seconds
        );
        
        match result {
            crate::core::error::RecoveryResult::Recovered => {
                debug!("Recovered from error");
            }
            crate::core::error::RecoveryResult::Failed => {
                info!("Failed to recover from error, continuing anyway");
            }
            crate::core::error::RecoveryResult::Fatal => {
                panic!("Fatal simulation error");
            }
        }
    }
    
    /// Sets environmental factors
    pub fn set_environment(&mut self, environment: EnvironmentalFactors) {
        self.environment = environment;
    }
    
    /// Returns current frame count
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }
    
    /// Returns interpolation factor for smooth rendering
    pub fn interpolation(&self) -> f32 {
        self.time_system.interpolation()
    }
}

impl Default for Simulation {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Vec2;
    use crate::simulation::{Creature, Resource, ResourceType};
    
    #[test]
    fn simulation_basic_update() {
        let mut sim = Simulation::with_bounds(100.0, 100.0);
        
        // Add a creature
        let entity = sim.world.entities.create();
        let creature = Creature::new(entity, Vec2::new(50.0, 50.0));
        sim.world.creatures.insert(entity, creature);
        sim.world.spatial_grid.insert(entity, Vec2::new(50.0, 50.0));
        
        // Update simulation
        let steps = sim.update(1.0 / 60.0);
        assert!(steps > 0);
        assert_eq!(sim.frame_count(), steps as u64);
    }
    
    #[test]
    fn simulation_creature_needs() {
        let mut sim = Simulation::with_bounds(100.0, 100.0);
        
        // Add a creature
        let entity = sim.world.entities.create();
        let creature = Creature::new(entity, Vec2::new(50.0, 50.0));
        let initial_hunger = creature.needs.hunger;
        sim.world.creatures.insert(entity, creature);
        
        // Update for 1 second
        for _ in 0..60 {
            sim.update(1.0 / 60.0);
        }
        
        // Needs should have increased
        let creature = &sim.world.creatures[&entity];
        assert!(creature.needs.hunger > initial_hunger);
    }
    
    #[test]
    fn simulation_resource_interaction() {
        let mut sim = Simulation::with_bounds(100.0, 100.0);
        
        // Add hungry creature
        let creature_entity = sim.world.entities.create();
        let mut creature = Creature::new(creature_entity, Vec2::new(50.0, 50.0));
        creature.needs.hunger = 0.8;
        sim.world.creatures.insert(creature_entity, creature);
        sim.world.spatial_grid.insert(creature_entity, Vec2::new(50.0, 50.0));
        
        // Add food nearby
        let food_entity = sim.world.entities.create();
        let food = Resource::new(
            food_entity,
            Vec2::new(52.0, 50.0),
            crate::simulation::ResourceType::Food
        );
        sim.world.resources.insert(food_entity, food);
        sim.world.spatial_grid.insert(food_entity, Vec2::new(52.0, 50.0));
        
        // Update simulation - creature should move to food
        for _ in 0..120 { // 2 seconds
            sim.update(1.0 / 60.0);
        }
        
        // Check that creature's hunger decreased
        let creature = &sim.world.creatures[&creature_entity];
        assert!(creature.needs.hunger < 0.8);
    }
    
    #[test]
    fn simulation_performance_stats() {
        let mut sim = Simulation::with_bounds(100.0, 100.0);
        
        // Add multiple creatures
        for i in 0..10 {
            let entity = sim.world.entities.create();
            let creature = Creature::new(entity, Vec2::new(i as f32 * 10.0, 50.0));
            sim.world.creatures.insert(entity, creature);
            sim.world.spatial_grid.insert(entity, Vec2::new(i as f32 * 10.0, 50.0));
        }
        
        sim.update(1.0 / 60.0);
        
        assert_eq!(sim.world.stats.creature_count, 10);
        assert!(sim.world.stats.update_time_ms >= 0.0);
    }
}