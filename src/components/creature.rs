//! Creature-specific components

use bevy::prelude::*;

/// Marker component for creature entities
#[derive(Component, Debug, Default)]
pub struct Creature;

/// Age of the creature in seconds
#[derive(Component, Debug, Default)]
pub struct Age(pub f32);

/// Size of the creature (affects metabolism and speed)
#[derive(Component, Debug)]
pub struct Size(pub f32);

impl Default for Size {
    fn default() -> Self {
        Self(1.0)
    }
}

/// Current state of the creature
#[derive(Component, Debug, Clone, PartialEq)]
pub enum CreatureState {
    Idle,
    Moving { target: Vec2 },
    Eating,
    Drinking,
    Resting,
    Dead,
}

impl Default for CreatureState {
    fn default() -> Self {
        Self::Idle
    }
}

/// Tracks how long creature has been in current state
#[derive(Component, Debug, Default)]
pub struct StateDuration(pub f32);

/// Creature type for predator/prey dynamics
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub enum CreatureType {
    Herbivore,
    Carnivore,
    Omnivore,
}

impl Default for CreatureType {
    fn default() -> Self {
        Self::Herbivore
    }
}

/// Genetic traits that affect appearance and behavior
#[derive(Component, Debug, Clone)]
pub struct Genetics {
    /// Size gene (0.0-1.0, affects creature scale)
    pub size: f32,
    /// Color gene (0.0-1.0, affects hue variation)
    pub color: f32,
    /// Pattern gene (0.0-1.0, affects pattern type)
    pub pattern: f32,
    /// Speed gene (0.0-1.0, affects movement speed)
    pub speed: f32,
    /// Aggression gene (0.0-1.0, affects behavior)
    pub aggression: f32,
}

impl Default for Genetics {
    fn default() -> Self {
        Self {
            size: 0.5,
            color: 0.5,
            pattern: 0.5,
            speed: 0.5,
            aggression: 0.5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_creature_component_default() {
        let creature = Creature::default();
        // Creature is a marker component, just ensure it can be created
        let _ = creature;
    }
    
    #[test]
    fn test_age_component() {
        let age = Age(100.5);
        assert_eq!(age.0, 100.5);
        
        let default_age = Age::default();
        assert_eq!(default_age.0, 0.0);
    }
    
    #[test]
    fn test_size_component() {
        let size = Size(2.5);
        assert_eq!(size.0, 2.5);
        
        let default_size = Size::default();
        assert_eq!(default_size.0, 1.0);
    }
    
    #[test]
    fn test_creature_state_variants() {
        // Test all state variants
        let idle = CreatureState::Idle;
        assert_eq!(idle, CreatureState::Idle);
        
        let moving = CreatureState::Moving { target: Vec2::new(10.0, 20.0) };
        match moving {
            CreatureState::Moving { target } => {
                assert_eq!(target.x, 10.0);
                assert_eq!(target.y, 20.0);
            }
            _ => panic!("Expected Moving state"),
        }
        
        let eating = CreatureState::Eating;
        assert_eq!(eating, CreatureState::Eating);
        
        let drinking = CreatureState::Drinking;
        assert_eq!(drinking, CreatureState::Drinking);
        
        let resting = CreatureState::Resting;
        assert_eq!(resting, CreatureState::Resting);
        
        let dead = CreatureState::Dead;
        assert_eq!(dead, CreatureState::Dead);
    }
    
    #[test]
    fn test_creature_state_default() {
        let state = CreatureState::default();
        assert_eq!(state, CreatureState::Idle);
    }
    
    #[test]
    fn test_creature_state_equality() {
        assert_eq!(CreatureState::Idle, CreatureState::Idle);
        assert_ne!(CreatureState::Idle, CreatureState::Eating);
        assert_ne!(
            CreatureState::Moving { target: Vec2::new(1.0, 2.0) },
            CreatureState::Moving { target: Vec2::new(3.0, 4.0) }
        );
    }
    
    #[test]
    fn test_state_duration() {
        let duration = StateDuration(5.5);
        assert_eq!(duration.0, 5.5);
        
        let default_duration = StateDuration::default();
        assert_eq!(default_duration.0, 0.0);
    }
    
    #[test]
    fn test_creature_type_variants() {
        let herbivore = CreatureType::Herbivore;
        assert_eq!(herbivore, CreatureType::Herbivore);
        
        let carnivore = CreatureType::Carnivore;
        assert_eq!(carnivore, CreatureType::Carnivore);
        
        let omnivore = CreatureType::Omnivore;
        assert_eq!(omnivore, CreatureType::Omnivore);
    }
    
    #[test]
    fn test_creature_type_default() {
        let creature_type = CreatureType::default();
        assert_eq!(creature_type, CreatureType::Herbivore);
    }
    
    #[test]
    fn test_creature_type_copy() {
        let original = CreatureType::Carnivore;
        let copied = original;
        assert_eq!(original, copied);
    }
}
