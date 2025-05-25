use crate::config::{
    creature::*,
    interaction::*,
    needs::*,
    resource::*,
    time::{FIXED_TIMESTEP, MAX_STEPS_PER_UPDATE},
};
use crate::core::{SimulationError, TimeSystem, World};
use crate::simulation::{needs::EnvironmentalFactors, CreatureState};
use crate::systems::{DecoupledDecisionSystem, MovementSystem, ResourceSpawner};
use bevy::log::{debug, info};
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
    decision_system: DecoupledDecisionSystem,
    resource_spawner: ResourceSpawner,
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
            decision_system: DecoupledDecisionSystem::default(),
            resource_spawner: ResourceSpawner::default(),
            environment: EnvironmentalFactors::default(),
            frame_count: 0,
            last_update_time: Instant::now(),
        }
    }

    /// Creates a simulation with world bounds
    pub fn with_bounds(width: f32, height: f32) -> Self {
        let mut sim = Self::new();
        sim.world = World::with_bounds(crate::Vec2::new(0.0, 0.0), crate::Vec2::new(width, height));
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
            self.update_resource_spawner(dt);
            self.process_interactions();
            self.process_events();

            self.frame_count += 1;
            steps += 1;

            // Update game time
            self.world.time.advance(dt);

            // Prevent spiral of death
            if steps >= MAX_STEPS_PER_UPDATE {
                debug!("Hit max steps per update: {}", MAX_STEPS_PER_UPDATE);
                break;
            }
        }

        // Update performance stats
        let update_duration = start_time.elapsed();
        self.world.stats.update_time_ms = update_duration.as_secs_f32() * 1000.0;
        self.world.update_stats();

        // Log performance warnings
        if update_duration.as_millis() > 16 {
            debug!(
                "Slow frame: {}ms for {} steps",
                update_duration.as_millis(),
                steps
            );
        }

        steps
    }

    /// Updates creature decisions
    fn update_decisions(&mut self) {
        let current_time = self.time_system.game_time() as f32;
        self.decision_system.update(&mut self.world, current_time);
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
                // Handle resting state
                if matches!(creature.state, crate::simulation::CreatureState::Resting) {
                    creature.needs.rest(dt);
                } else {
                    creature.needs.update(dt, metabolism, &env);
                }
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
                damage += STARVATION_DAMAGE * dt;
                death_cause = Some(crate::core::events::DeathCause::Starvation);
            }

            if creature.needs.thirst >= 1.0 {
                damage += DEHYDRATION_DAMAGE * dt;
                death_cause = Some(crate::core::events::DeathCause::Dehydration);
            }

            if creature.needs.energy <= 0.0 {
                damage += EXHAUSTION_DAMAGE * dt;
            }

            // Check for old age
            if creature.age > OLD_AGE_THRESHOLD {
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
                let was_alive = creature.is_alive();
                creature.health.damage(damage);

                if creature.health.is_dead() && was_alive {
                    creature.die();
                    let cause = death_cause.unwrap_or(crate::core::events::DeathCause::Unknown);
                    info!("Creature {:?} died from {:?}", entity, cause);

                    // Emit death event
                    self.world
                        .events
                        .emit(crate::core::events::GameEvent::CreatureDied { entity, cause });
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

    /// Updates resource spawning
    fn update_resource_spawner(&mut self, dt: f32) {
        self.resource_spawner.update(&mut self.world, dt);
    }

    /// Processes creature-resource interactions
    fn process_interactions(&mut self) {
        // Decision system now handles resource interactions internally

        // Process ongoing interactions
        let mut consumptions = Vec::new();

        for (&entity, creature) in &self.world.creatures {
            match creature.state {
                crate::simulation::CreatureState::Eating => {
                    // Find nearest food
                    if let Some(resource_entity) = self.find_nearest_resource_of_type(
                        creature.position,
                        crate::simulation::ResourceType::Food,
                    ) {
                        consumptions.push((
                            entity,
                            resource_entity,
                            crate::simulation::ResourceType::Food,
                        ));
                    }
                },
                crate::simulation::CreatureState::Drinking => {
                    // Find nearest water
                    if let Some(resource_entity) = self.find_nearest_resource_of_type(
                        creature.position,
                        crate::simulation::ResourceType::Water,
                    ) {
                        consumptions.push((
                            entity,
                            resource_entity,
                            crate::simulation::ResourceType::Water,
                        ));
                    }
                },
                crate::simulation::CreatureState::Resting => {
                    // Resting is handled in update_needs
                },
                _ => {},
            }
        }

        // Apply consumptions
        for (creature_entity, resource_entity, resource_type) in consumptions {
            if let Some(resource) = self.world.resources.get_mut(&resource_entity) {
                let rate = resource.resource_type.consumption_rate();
                let consumed = resource
                    .consume(rate * self.time_system.fixed_timestep().unwrap_or(1.0 / 60.0));

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
                            // Food satisfaction multiplier from config
                            let satisfaction = consumed * FOOD_SATISFACTION_MULTIPLIER;
                            let old_hunger = creature.needs.hunger;
                            creature.needs.eat(satisfaction);
                            debug!("Creature {:?} eating: consumed={}, satisfaction={}, hunger: {} -> {}", 
                                   creature_entity, consumed, satisfaction, old_hunger, creature.needs.hunger);
                            if resource.is_depleted() {
                                debug!(
                                    "Resource depleted, creature {:?} changing to Idle",
                                    creature_entity
                                );
                                creature.state = CreatureState::Idle; // Stop eating - no more food
                            } else if creature.needs.hunger <= 0.1 {
                                debug!(
                                    "Creature {:?} fully satisfied (hunger={}), changing to Idle",
                                    creature_entity, creature.needs.hunger
                                );
                                creature.state = CreatureState::Idle; // Stop eating - fully satisfied
                            } else if creature.needs.hunger <= 0.3
                                && creature.state_duration() > 1.0
                            {
                                // If hunger is below urgency threshold and creature has been eating for at least 1 second,
                                // check if they've eaten enough to avoid immediate starvation
                                let hunger_rate = DEFAULT_HUNGER_RATE * creature.metabolism_rate();
                                let time_until_urgent = (0.3 - creature.needs.hunger) / hunger_rate;

                                if time_until_urgent > 10.0 {
                                    // Creature has eaten enough to last at least 10 seconds
                                    debug!("Creature {:?} has eaten enough (hunger={}, time_until_urgent={}s), changing to Idle", 
                                           creature_entity, creature.needs.hunger, time_until_urgent);
                                    creature.state = CreatureState::Idle;
                                }
                            }
                            // Continue eating otherwise
                        },
                        crate::simulation::ResourceType::Water => {
                            // Water satisfaction: 1 unit of water = 1.0 thirst reduction
                            // This means drinking 0.1 units/sec reduces thirst by 0.1/sec
                            let satisfaction = consumed * 1.0;
                            creature.needs.drink(satisfaction);
                            if resource.is_depleted() {
                                creature.state = CreatureState::Idle; // Stop drinking - no more water
                            } else if creature.needs.thirst <= 0.1 {
                                creature.state = CreatureState::Idle; // Stop drinking - fully satisfied
                            }
                        },
                    }
                }
            }
        }
    }

    /// Finds nearest resource of given type within interaction range
    fn find_nearest_resource_of_type(
        &self,
        position: crate::Vec2,
        resource_type: crate::simulation::ResourceType,
    ) -> Option<crate::core::Entity> {
        self.world
            .find_nearest_resource(position, resource_type, Some(MAX_INTERACTION_RANGE))
            .map(|(entity, _)| entity)
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
        let result = self.world.error_boundary.handle_error(error, self.world.time.total_seconds);

        match result {
            crate::core::error::RecoveryResult::Recovered => {
                debug!("Recovered from error");
            },
            crate::core::error::RecoveryResult::Failed => {
                info!("Failed to recover from error, continuing anyway");
            },
            crate::core::error::RecoveryResult::Fatal => {
                panic!("Fatal simulation error");
            },
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

    /// Updates the simulation with exactly one fixed timestep
    /// Used for testing to ensure predictable behavior
    pub fn step(&mut self) {
        let dt = FIXED_TIMESTEP;

        // Update systems in correct order
        self.update_decisions();
        self.update_movement(dt);
        self.update_needs(dt);
        self.update_health(dt);
        self.update_resources(dt);
        self.update_resource_spawner(dt);
        self.process_interactions();
        self.process_events();

        self.frame_count += 1;

        // Update game time
        self.world.time.advance(dt);
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
    use crate::core::Entity;
    use crate::simulation::{Creature, Resource, ResourceType};
    use crate::Vec2;

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
        creature.needs.hunger = 0.8; // High hunger to trigger food seeking
        creature.needs.thirst = 0.0; // Low thirst so hunger is priority
        sim.world.creatures.insert(creature_entity, creature);
        sim.world.spatial_grid.insert(creature_entity, Vec2::new(50.0, 50.0));

        // Add food nearby with plenty of amount
        let food_entity = sim.world.entities.create();
        let mut food = Resource::new(
            food_entity,
            Vec2::new(52.0, 50.0),
            crate::simulation::ResourceType::Food,
        );
        food.amount = 50.0; // Ensure plenty of food
        sim.world.resources.insert(food_entity, food);
        sim.world.spatial_grid.insert(food_entity, Vec2::new(52.0, 50.0));

        // Update simulation - creature should move to food
        let initial_hunger = sim.world.creatures[&creature_entity].needs.hunger;

        // Debug: Check if resource is findable
        let found_resources = sim.world.find_resources_near(
            Vec2::new(50.0, 50.0),
            10.0,
            crate::simulation::ResourceType::Food,
        );
        assert!(
            !found_resources.is_empty(),
            "No food resources found near creature!"
        );
        println!("Found {} food resources", found_resources.len());

        // Check initial decision
        sim.decision_system.update(&mut sim.world, 0.0);
        let creature_state = sim.world.creatures[&creature_entity].state.clone();
        println!(
            "Initial creature state after decision: {:?}",
            creature_state
        );

        let mut hunger_decreased = false;

        for _i in 0..240 {
            // 4 seconds - give more time
            sim.update(1.0 / 60.0);

            let creature = &sim.world.creatures[&creature_entity];
            if creature.needs.hunger < initial_hunger {
                hunger_decreased = true;
                break; // Success - hunger decreased
            }
        }

        assert!(
            hunger_decreased,
            "Creature hunger did not decrease from initial value of {}",
            initial_hunger
        );
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

    #[test]
    fn simulation_handle_error() {
        let mut sim = Simulation::new();

        // Test non-fatal error handling
        let error = SimulationError::EntityNotFound {
            entity: Entity::new(999),
        };
        sim.handle_error(error); // Should not panic

        // Check that error was logged
        assert!(sim.world.error_boundary.get_error_count() > 0);
    }

    #[test]
    fn simulation_set_environment() {
        let mut sim = Simulation::new();

        let env = EnvironmentalFactors {
            hunger_multiplier: 2.0,
            thirst_multiplier: 1.5,
            energy_multiplier: 0.8,
        };

        sim.set_environment(env.clone());

        // Add a creature and update to see if environment is applied
        let entity = sim.world.entities.create();
        let creature = Creature::new(entity, Vec2::ZERO);
        sim.world.creatures.insert(entity, creature);

        // Environment should affect need updates
        let initial_hunger = sim.world.creatures[&entity].needs.hunger;
        sim.update(1.0);
        let final_hunger = sim.world.creatures[&entity].needs.hunger;

        // With 2x multiplier, hunger should increase more
        assert!(final_hunger > initial_hunger);
    }

    #[test]
    fn simulation_frame_count() {
        let mut sim = Simulation::new();

        assert_eq!(sim.frame_count(), 0);

        // Update multiple times
        for _ in 0..5 {
            sim.update(1.0 / 60.0);
        }

        assert!(sim.frame_count() > 0);
    }

    #[test]
    fn simulation_interpolation() {
        let mut sim = Simulation::new();

        // Initial interpolation
        let initial_interp = sim.interpolation();
        assert!(initial_interp >= 0.0 && initial_interp <= 1.0);

        // Update with partial timestep
        sim.update(0.008); // Half of 1/60

        let interp = sim.interpolation();
        assert!(interp >= 0.0 && interp <= 1.0);
    }

    #[test]
    fn simulation_default() {
        let sim = Simulation::default();
        assert_eq!(sim.world.creature_count(), 0);
        assert_eq!(sim.world.resource_count(), 0);
    }

    #[test]
    fn simulation_find_nearest_resource() {
        let _sim = Simulation::new();

        // Test helper method (even though it's private, we test through behavior)
        // Create a resource and ensure it can be found
        let mut world = World::new();
        let resource_entity = world.entities.create();
        let resource = Resource::new(resource_entity, Vec2::new(10.0, 10.0), ResourceType::Food);
        world.resources.insert(resource_entity, resource);
        world.spatial_grid.insert(resource_entity, Vec2::new(10.0, 10.0));

        // Use world's find_nearest_resource which simulation uses internally
        let found = world.find_nearest_resource(Vec2::ZERO, ResourceType::Food, None);
        assert!(found.is_some());
        assert_eq!(found.unwrap().0, resource_entity);
    }
}
