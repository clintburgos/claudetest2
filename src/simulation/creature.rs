use crate::Vec2;
use crate::core::Entity;
use crate::simulation::{Health, Needs};
use crate::config::creature::*;
use log::debug;

/// Represents a living creature in the simulation
/// 
/// Creatures have position, health, needs, and behavior state.
/// They age over time and their size affects metabolism and movement.
#[derive(Debug, Clone)]
pub struct Creature {
    pub id: Entity,
    pub position: Vec2,
    pub velocity: Vec2,
    pub health: Health,
    pub needs: Needs,
    pub state: CreatureState,
    pub age: f32,
    pub size: f32,
    /// Cached movement speed to avoid recalculation
    cached_speed: Option<f32>,
    /// Time spent in current state
    state_duration: f32,
}

/// Possible states for creature behavior
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CreatureState {
    /// Not performing any specific action
    Idle,
    /// Moving towards a target position
    Moving { target: Vec2 },
    /// Consuming food resource
    Eating,
    /// Consuming water resource
    Drinking,
    /// Recovering energy
    Resting,
    /// No longer alive
    Dead,
}

impl Creature {
    /// Creates a new creature at the given position
    /// 
    /// # Arguments
    /// * `id` - Unique entity identifier
    /// * `position` - Starting world position
    pub fn new(id: Entity, position: Vec2) -> Self {
        assert!(position.is_finite(), "Creature position must be finite");
        
        Self {
            id,
            position,
            velocity: Vec2::ZERO,
            health: Health::new(100.0),
            needs: Needs::new(),
            state: CreatureState::Idle,
            age: 0.0,
            size: 1.0,
            cached_speed: None,
            state_duration: 0.0,
        }
    }
    
    /// Updates creature age and state duration
    /// 
    /// # Arguments
    /// * `dt` - Time elapsed in seconds
    pub fn update_age(&mut self, dt: f32) {
        self.age += dt;
        self.state_duration += dt;
    }
    
    /// Checks if the creature is still alive
    pub fn is_alive(&self) -> bool {
        !matches!(self.state, CreatureState::Dead) && !self.health.is_dead()
    }
    
    /// Marks the creature as dead and stops all movement
    pub fn die(&mut self) {
        self.set_state(CreatureState::Dead);
        self.velocity = Vec2::ZERO;
    }
    
    /// Starts moving towards a target position
    pub fn start_moving(&mut self, target: Vec2) {
        assert!(target.is_finite(), "Movement target must be finite");
        if self.is_alive() {
            self.set_state(CreatureState::Moving { target });
        }
    }
    
    /// Stops current movement
    pub fn stop_moving(&mut self) {
        if matches!(self.state, CreatureState::Moving { .. }) {
            self.set_state(CreatureState::Idle);
            self.velocity = Vec2::ZERO;
        }
    }
    
    /// Starts eating behavior
    pub fn start_eating(&mut self) {
        if self.is_alive() {
            self.set_state(CreatureState::Eating);
            self.velocity = Vec2::ZERO;
        }
    }
    
    /// Starts drinking behavior
    pub fn start_drinking(&mut self) {
        if self.is_alive() {
            self.set_state(CreatureState::Drinking);
            self.velocity = Vec2::ZERO;
        }
    }
    
    /// Starts resting behavior
    pub fn start_resting(&mut self) {
        if self.is_alive() {
            self.set_state(CreatureState::Resting);
            self.velocity = Vec2::ZERO;
        }
    }
    
    /// Updates creature state and clears speed cache if needed
    fn set_state(&mut self, new_state: CreatureState) {
        // Use discriminant comparison to properly check if states are different
        if std::mem::discriminant(&self.state) != std::mem::discriminant(&new_state) {
            debug!("Creature {:?} state: {:?} -> {:?}", self.id, self.state, new_state);
            self.state = new_state;
            self.state_duration = 0.0;
            self.cached_speed = None; // Invalidate cache on state change
        } else if self.state != new_state {
            // States have same type but different data (e.g., different target positions)
            self.state = new_state;
        }
    }
    
    /// Calculates metabolism rate based on size
    /// 
    /// Larger creatures have slower metabolism
    pub fn metabolism_rate(&self) -> f32 {
        1.0 / self.size.sqrt()
    }
    
    /// Returns movement speed, using cache when possible
    /// 
    /// Speed is affected by size and energy level
    pub fn movement_speed(&mut self) -> f32 {
        if let Some(speed) = self.cached_speed {
            return speed;
        }
        
        // Calculate and cache speed
        let base_speed = BASE_SPEED;
        let size_modifier = (2.0 - self.size).max(0.5);
        let energy_modifier = (0.2 + self.needs.energy * 0.8).max(0.2);
        let speed = base_speed * size_modifier * energy_modifier;
        
        self.cached_speed = Some(speed);
        speed
    }
    
    /// Updates position with validation
    /// 
    /// # Arguments
    /// * `new_position` - New world position
    /// 
    /// # Returns
    /// `true` if position was updated, `false` if invalid
    pub fn update_position(&mut self, new_position: Vec2) -> bool {
        if new_position.is_finite() {
            self.position = new_position;
            true
        } else {
            debug!("Creature {:?} rejected invalid position {:?}", self.id, new_position);
            false
        }
    }
    
    /// Returns how long the creature has been in its current state
    pub fn state_duration(&self) -> f32 {
        self.state_duration
    }
}

impl Default for Creature {
    fn default() -> Self {
        Self::new(Entity::new(0), Vec2::ZERO)
    }
}

/// Builder pattern for creating creatures with specific attributes
pub struct CreatureBuilder {
    id: Entity,
    position: Vec2,
    size: f32,
    health: f32,
}

impl CreatureBuilder {
    /// Creates a new creature builder
    pub fn new(id: Entity, position: Vec2) -> Self {
        Self {
            id,
            position,
            size: 1.0,
            health: 100.0,
        }
    }
    
    /// Sets the creature's size
    pub fn with_size(mut self, size: f32) -> Self {
        self.size = size.max(0.1);
        self
    }
    
    /// Sets the creature's max health
    pub fn with_health(mut self, health: f32) -> Self {
        self.health = health.max(1.0);
        self
    }
    
    /// Builds the creature
    pub fn build(self) -> Creature {
        let mut creature = Creature::new(self.id, self.position);
        creature.size = self.size;
        creature.health = Health::new(self.health);
        creature
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creature_creation() {
        let creature = Creature::new(Entity::new(1), Vec2::new(10.0, 20.0));
        assert_eq!(creature.position, Vec2::new(10.0, 20.0));
        assert!(creature.is_alive());
        assert_eq!(creature.state, CreatureState::Idle);
    }
    
    #[test]
    fn creature_death() {
        let mut creature = Creature::new(Entity::new(1), Vec2::ZERO);
        
        creature.die();
        assert!(!creature.is_alive());
        assert_eq!(creature.state, CreatureState::Dead);
    }
    
    #[test]
    fn creature_state_transitions() {
        let mut creature = Creature::new(Entity::new(1), Vec2::ZERO);
        
        creature.start_moving(Vec2::new(10.0, 10.0));
        assert!(matches!(creature.state, CreatureState::Moving { .. }));
        
        creature.start_eating();
        assert_eq!(creature.state, CreatureState::Eating);
        assert_eq!(creature.velocity, Vec2::ZERO);
    }
    
    #[test]
    fn creature_update_age() {
        let mut creature = Creature::new(Entity::new(1), Vec2::ZERO);
        assert_eq!(creature.age, 0.0);
        
        creature.update_age(5.0);
        assert_eq!(creature.age, 5.0);
        
        creature.update_age(3.0);
        assert_eq!(creature.age, 8.0);
    }
    
    #[test]
    fn creature_stop_moving() {
        let mut creature = Creature::new(Entity::new(1), Vec2::ZERO);
        creature.velocity = Vec2::new(5.0, 5.0);
        creature.state = CreatureState::Moving { target: Vec2::new(10.0, 10.0) };
        
        creature.stop_moving();
        assert_eq!(creature.state, CreatureState::Idle);
        assert_eq!(creature.velocity, Vec2::ZERO);
        
        // Should not affect non-moving states
        creature.state = CreatureState::Eating;
        creature.stop_moving();
        assert_eq!(creature.state, CreatureState::Eating);
    }
    
    #[test]
    fn creature_start_activities() {
        let mut creature = Creature::new(Entity::new(1), Vec2::ZERO);
        
        // Test drinking
        creature.start_drinking();
        assert_eq!(creature.state, CreatureState::Drinking);
        assert_eq!(creature.velocity, Vec2::ZERO);
        
        // Test resting
        creature.start_resting();
        assert_eq!(creature.state, CreatureState::Resting);
        assert_eq!(creature.velocity, Vec2::ZERO);
        
        // Dead creatures can't start activities
        creature.die();
        creature.start_eating();
        assert_eq!(creature.state, CreatureState::Dead);
    }
    
    #[test]
    fn creature_metabolism_rate() {
        let mut creature = Creature::new(Entity::new(1), Vec2::ZERO);
        
        creature.size = 1.0;
        assert_eq!(creature.metabolism_rate(), 1.0);
        
        creature.size = 4.0;
        assert_eq!(creature.metabolism_rate(), 0.5); // 1/sqrt(4) = 0.5
        
        creature.size = 0.25;
        assert_eq!(creature.metabolism_rate(), 2.0); // 1/sqrt(0.25) = 2.0
    }
    
    #[test]
    fn creature_movement_speed() {
        let mut creature = Creature::new(Entity::new(1), Vec2::ZERO);
        
        // Default creature (size=1, full energy)
        let base_speed = creature.movement_speed();
        assert!(base_speed > 0.0);
        
        // Speed should be cached
        let cached_speed = creature.movement_speed();
        assert_eq!(base_speed, cached_speed);
        
        // Larger creature moves slower
        creature.size = 2.0;
        creature.cached_speed = None; // Clear cache
        let large_speed = creature.movement_speed();
        assert!(large_speed < base_speed);
        
        // Low energy reduces speed
        creature.size = 1.0;
        creature.needs.energy = 0.0;
        creature.cached_speed = None; // Clear cache
        let tired_speed = creature.movement_speed();
        assert!(tired_speed < base_speed);
        assert!(tired_speed >= 2.0); // Minimum speed (10 * 0.2)
    }
    
    #[test]
    fn creature_state_tracking() {
        let mut creature = Creature::new(Entity::new(1), Vec2::ZERO);
        
        assert_eq!(creature.state_duration(), 0.0);
        
        creature.update_age(5.0);
        assert_eq!(creature.state_duration(), 5.0);
        
        creature.start_moving(Vec2::new(10.0, 10.0));
        assert_eq!(creature.state_duration(), 0.0); // Reset on state change
    }
    
    #[test]
    fn creature_position_validation() {
        let mut creature = Creature::new(Entity::new(1), Vec2::ZERO);
        
        // Valid position update
        assert!(creature.update_position(Vec2::new(10.0, 20.0)));
        assert_eq!(creature.position, Vec2::new(10.0, 20.0));
        
        // Invalid position rejected
        assert!(!creature.update_position(Vec2::new(f32::NAN, 0.0)));
        assert_eq!(creature.position, Vec2::new(10.0, 20.0)); // Unchanged
    }
    
    #[test]
    fn creature_builder() {
        let creature = CreatureBuilder::new(Entity::new(1), Vec2::new(5.0, 5.0))
            .with_size(2.0)
            .with_health(150.0)
            .build();
        
        assert_eq!(creature.id, Entity::new(1));
        assert_eq!(creature.position, Vec2::new(5.0, 5.0));
        assert_eq!(creature.size, 2.0);
        assert_eq!(creature.health.max, 150.0);
        assert_eq!(creature.health.current, 150.0);
    }
    
    #[test]
    fn creature_default() {
        let creature = Creature::default();
        assert_eq!(creature.id, Entity::new(0));
        assert_eq!(creature.position, Vec2::ZERO);
        assert_eq!(creature.state, CreatureState::Idle);
        assert!(creature.is_alive());
    }
}