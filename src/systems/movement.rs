use crate::config::movement::*;
use crate::core::World;
use crate::simulation::{Creature, CreatureState};
use crate::Vec2;
use bevy::log::debug;

/// Movement system responsible for updating creature positions
///
/// Handles:
/// - Position updates based on velocity
/// - Pathfinding towards targets
/// - Collision with world bounds
/// - Arrival at destinations
pub struct MovementSystem {
    /// Minimum distance to consider target reached
    arrival_threshold: f32,
    /// Maximum movement iterations per update
    max_iterations: usize,
}

impl MovementSystem {
    /// Creates a new movement system
    pub fn new() -> Self {
        Self {
            arrival_threshold: ARRIVAL_THRESHOLD,
            max_iterations: 10,
        }
    }

    /// Updates all creature positions for the given time step
    pub fn update(&mut self, world: &mut World, dt: f32) {
        // Collect movement updates to avoid borrowing issues
        let mut updates = Vec::new();
        let mut arrivals = Vec::new();

        // First pass: calculate new positions
        for (&entity, creature) in &world.creatures {
            if !creature.is_alive() {
                continue;
            }

            match creature.state {
                CreatureState::Moving { target } => {
                    let direction = target - creature.position;
                    let distance = direction.length();

                    if distance <= self.arrival_threshold {
                        // Arrived at destination
                        arrivals.push(entity);
                    } else {
                        // Calculate velocity towards target
                        let mut creature_mut = creature.clone();
                        let speed = creature_mut.movement_speed();
                        let max_step = speed * dt;

                        if distance <= max_step {
                            // Will arrive this frame
                            updates.push((entity, target, Vec2::ZERO));
                            arrivals.push(entity);
                        } else {
                            // Move towards target
                            let velocity = (direction / distance) * speed;
                            let new_position = creature.position + velocity * dt;
                            updates.push((entity, new_position, velocity));
                        }
                    }
                },
                _ => {
                    // Not moving, ensure velocity is zero
                    if creature.velocity != Vec2::ZERO {
                        updates.push((entity, creature.position, Vec2::ZERO));
                    }
                },
            }
        }

        // Apply world bounds if they exist
        if let Some(bounds) = world.bounds.as_ref() {
            for (_, position, _) in &mut updates {
                *position = bounds.clamp(*position);
            }
        }

        // Second pass: apply updates
        for (entity, new_position, velocity) in updates {
            if let Some(creature) = world.creatures.get_mut(&entity) {
                if creature.update_position(new_position) {
                    creature.velocity = velocity;
                    world.spatial_grid.insert(entity, new_position);
                }
            }
        }

        // Third pass: handle arrivals
        for entity in arrivals {
            if let Some(creature) = world.creatures.get_mut(&entity) {
                debug!("Creature {:?} arrived at destination", entity);
                creature.stop_moving();
            }
        }
    }

    /// Finds a valid path from start to goal
    ///
    /// For Phase 1, this is a simple straight-line path.
    /// Returns None if no valid path exists.
    pub fn find_path(&self, start: Vec2, goal: Vec2, world: &World) -> Option<Vec2> {
        // Phase 1: Simple validation only
        if !start.is_finite() || !goal.is_finite() {
            return None;
        }

        // Check world bounds
        if let Some(bounds) = &world.bounds {
            if !bounds.contains(goal) {
                // Goal is outside world, clamp it
                return Some(bounds.clamp(goal));
            }
        }

        // Direct path for now
        Some(goal)
    }

    /// Steers a creature towards a target position
    pub fn steer_towards(&self, creature: &Creature, target: Vec2, max_speed: f32) -> Vec2 {
        let desired = target - creature.position;
        let distance = desired.length();

        if distance > 0.0 {
            // Scale to max speed
            let desired = (desired / distance) * max_speed;

            // Simple steering: desired velocity minus current velocity
            let steer = desired - creature.velocity;

            // Limit steering force for smoother movement
            let max_force = max_speed * STEERING_FORCE_RATIO;
            if steer.length() > max_force {
                (steer / steer.length()) * max_force
            } else {
                steer
            }
        } else {
            Vec2::ZERO
        }
    }

    /// Checks if a position is valid for movement
    pub fn is_valid_position(&self, position: Vec2, world: &World) -> bool {
        if !position.is_finite() {
            return false;
        }

        if let Some(bounds) = &world.bounds {
            bounds.contains(position)
        } else {
            true
        }
    }
}

impl Default for MovementSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Entity, WorldBounds};
    use crate::simulation::Creature;

    fn create_test_world() -> World {
        let mut world = World::new();
        world.bounds = Some(WorldBounds {
            min: Vec2::new(0.0, 0.0),
            max: Vec2::new(100.0, 100.0),
        });
        world
    }

    #[test]
    fn movement_system_basic() {
        let mut world = create_test_world();
        let mut movement = MovementSystem::new();

        let entity = world.entities.create();
        let mut creature = Creature::new(entity, Vec2::new(10.0, 10.0));
        creature.start_moving(Vec2::new(20.0, 10.0));

        world.creatures.insert(entity, creature);
        world.spatial_grid.insert(entity, Vec2::new(10.0, 10.0));

        // Update movement
        movement.update(&mut world, 1.0);

        // Creature should have moved
        let creature = &world.creatures[&entity];
        assert!(creature.position.x > 10.0);
        assert_eq!(creature.position.y, 10.0);
    }

    #[test]
    fn movement_arrival() {
        let mut world = create_test_world();
        let mut movement = MovementSystem::new();

        let entity = world.entities.create();
        let mut creature = Creature::new(entity, Vec2::new(10.0, 10.0));
        creature.start_moving(Vec2::new(11.0, 10.0)); // Very close target

        world.creatures.insert(entity, creature);
        world.spatial_grid.insert(entity, Vec2::new(10.0, 10.0));

        // Update movement
        movement.update(&mut world, 1.0);

        // Creature should have arrived and stopped
        let creature = &world.creatures[&entity];
        assert_eq!(creature.state, CreatureState::Idle);
        assert_eq!(creature.velocity, Vec2::ZERO);
    }

    #[test]
    fn movement_world_bounds() {
        let mut world = create_test_world();
        let mut movement = MovementSystem::new();

        let entity = world.entities.create();
        let mut creature = Creature::new(entity, Vec2::new(90.0, 50.0));
        creature.start_moving(Vec2::new(200.0, 50.0)); // Outside bounds

        world.creatures.insert(entity, creature);
        world.spatial_grid.insert(entity, Vec2::new(90.0, 50.0));

        // Update movement several times
        for _ in 0..10 {
            movement.update(&mut world, 1.0);
        }

        // Creature should be clamped to world bounds
        let creature = &world.creatures[&entity];
        assert!(creature.position.x <= 100.0);
    }

    #[test]
    fn movement_dead_creature() {
        let mut world = create_test_world();
        let mut movement = MovementSystem::new();

        let entity = world.entities.create();
        let mut creature = Creature::new(entity, Vec2::new(10.0, 10.0));
        creature.start_moving(Vec2::new(20.0, 10.0));
        creature.die();

        world.creatures.insert(entity, creature);
        let initial_pos = Vec2::new(10.0, 10.0);
        world.spatial_grid.insert(entity, initial_pos);

        // Update movement
        movement.update(&mut world, 1.0);

        // Dead creature should not move
        let creature = &world.creatures[&entity];
        assert_eq!(creature.position, initial_pos);
    }

    #[test]
    fn movement_path_finding() {
        let world = create_test_world();
        let movement = MovementSystem::new();

        // Valid path
        let path = movement.find_path(Vec2::new(10.0, 10.0), Vec2::new(50.0, 50.0), &world);
        assert_eq!(path, Some(Vec2::new(50.0, 50.0)));

        // Goal outside bounds - should be clamped
        let path = movement.find_path(Vec2::new(10.0, 10.0), Vec2::new(150.0, 50.0), &world);
        assert_eq!(path, Some(Vec2::new(100.0, 50.0)));

        // Invalid positions
        let path = movement.find_path(Vec2::new(f32::NAN, 10.0), Vec2::new(50.0, 50.0), &world);
        assert_eq!(path, None);
    }

    #[test]
    fn movement_steering() {
        let movement = MovementSystem::new();
        let creature = Creature::new(Entity::new(1), Vec2::new(0.0, 0.0));

        // Steer towards target
        let steer = movement.steer_towards(&creature, Vec2::new(10.0, 0.0), 5.0);
        assert!(steer.x > 0.0);
        assert_eq!(steer.y, 0.0);

        // Already at target
        let steer = movement.steer_towards(&creature, Vec2::new(0.0, 0.0), 5.0);
        assert_eq!(steer, Vec2::ZERO);
    }
}
