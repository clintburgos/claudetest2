use crate::Vec2;
use crate::core::Entity;
use crate::simulation::{Health, Needs};

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
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CreatureState {
    Idle,
    Moving { target: Vec2 },
    Eating,
    Drinking,
    Resting,
    Dead,
}

impl Creature {
    pub fn new(id: Entity, position: Vec2) -> Self {
        Self {
            id,
            position,
            velocity: Vec2::ZERO,
            health: Health::new(100.0),
            needs: Needs::new(),
            state: CreatureState::Idle,
            age: 0.0,
            size: 1.0,
        }
    }
    
    pub fn update_age(&mut self, dt: f32) {
        self.age += dt;
    }
    
    pub fn is_alive(&self) -> bool {
        !matches!(self.state, CreatureState::Dead) && !self.health.is_dead()
    }
    
    pub fn die(&mut self) {
        self.state = CreatureState::Dead;
        self.velocity = Vec2::ZERO;
    }
    
    pub fn start_moving(&mut self, target: Vec2) {
        if self.is_alive() {
            self.state = CreatureState::Moving { target };
        }
    }
    
    pub fn stop_moving(&mut self) {
        if matches!(self.state, CreatureState::Moving { .. }) {
            self.state = CreatureState::Idle;
            self.velocity = Vec2::ZERO;
        }
    }
    
    pub fn start_eating(&mut self) {
        if self.is_alive() {
            self.state = CreatureState::Eating;
            self.velocity = Vec2::ZERO;
        }
    }
    
    pub fn start_drinking(&mut self) {
        if self.is_alive() {
            self.state = CreatureState::Drinking;
            self.velocity = Vec2::ZERO;
        }
    }
    
    pub fn start_resting(&mut self) {
        if self.is_alive() {
            self.state = CreatureState::Resting;
            self.velocity = Vec2::ZERO;
        }
    }
    
    pub fn metabolism_rate(&self) -> f32 {
        // Larger creatures have slower metabolism
        1.0 / self.size.sqrt()
    }
    
    pub fn movement_speed(&self) -> f32 {
        // Base speed modified by size and energy
        let base_speed = 10.0;
        let size_modifier = (2.0 - self.size).max(0.5);
        let energy_modifier = (0.2 + self.needs.energy * 0.8).max(0.2);
        
        base_speed * size_modifier * energy_modifier
    }
}

impl Default for Creature {
    fn default() -> Self {
        Self::new(Entity::new(0), Vec2::ZERO)
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
}