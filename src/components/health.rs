//! Health components

use bevy::prelude::*;

/// Health component for creatures
#[derive(Component, Debug, Clone)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }

    pub fn damage(&mut self, amount: f32) {
        self.current = (self.current - amount).max(0.0);
    }

    pub fn heal(&mut self, amount: f32) {
        self.current = (self.current + amount).min(self.max);
    }

    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }

    pub fn is_full(&self) -> bool {
        self.current >= self.max
    }

    pub fn percentage(&self) -> f32 {
        if self.max > 0.0 {
            self.current / self.max
        } else {
            0.0
        }
    }
}

impl Default for Health {
    fn default() -> Self {
        Self::new(100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_health_new() {
        let health = Health::new(150.0);
        assert_eq!(health.current, 150.0);
        assert_eq!(health.max, 150.0);
    }
    
    #[test]
    fn test_health_default() {
        let health = Health::default();
        assert_eq!(health.current, 100.0);
        assert_eq!(health.max, 100.0);
    }
    
    #[test]
    fn test_health_damage() {
        let mut health = Health::new(100.0);
        
        health.damage(30.0);
        assert_eq!(health.current, 70.0);
        assert_eq!(health.max, 100.0);
        
        // Test damage doesn't go below 0
        health.damage(100.0);
        assert_eq!(health.current, 0.0);
    }
    
    #[test]
    fn test_health_heal() {
        let mut health = Health {
            current: 50.0,
            max: 100.0,
        };
        
        health.heal(30.0);
        assert_eq!(health.current, 80.0);
        
        // Test heal doesn't exceed max
        health.heal(50.0);
        assert_eq!(health.current, 100.0);
    }
    
    #[test]
    fn test_health_is_dead() {
        let mut health = Health::new(100.0);
        assert!(!health.is_dead());
        
        health.current = 0.0;
        assert!(health.is_dead());
        
        health.current = -10.0;
        assert!(health.is_dead());
    }
    
    #[test]
    fn test_health_is_full() {
        let mut health = Health::new(100.0);
        assert!(health.is_full());
        
        health.damage(1.0);
        assert!(!health.is_full());
        
        health.heal(1.0);
        assert!(health.is_full());
    }
    
    #[test]
    fn test_health_percentage() {
        let mut health = Health::new(200.0);
        assert_eq!(health.percentage(), 1.0);
        
        health.current = 100.0;
        assert_eq!(health.percentage(), 0.5);
        
        health.current = 50.0;
        assert_eq!(health.percentage(), 0.25);
        
        health.current = 0.0;
        assert_eq!(health.percentage(), 0.0);
    }
    
    #[test]
    fn test_health_percentage_zero_max() {
        let health = Health {
            current: 0.0,
            max: 0.0,
        };
        assert_eq!(health.percentage(), 0.0);
    }
    
    #[test]
    fn test_health_clone() {
        let original = Health {
            current: 75.0,
            max: 100.0,
        };
        let cloned = original.clone();
        
        assert_eq!(cloned.current, original.current);
        assert_eq!(cloned.max, original.max);
    }
}
