use crate::config::creature::DEFAULT_HEALTH;

#[derive(Debug, Clone)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self {
            current: max,
            max,
        }
    }
    
    pub fn full(max: f32) -> Self {
        Self::new(max)
    }
    
    pub fn damage(&mut self, amount: f32) {
        self.current = (self.current - amount).max(0.0);
    }
    
    pub fn heal(&mut self, amount: f32) {
        self.current = (self.current + amount).min(self.max);
    }
    
    pub fn set(&mut self, value: f32) {
        self.current = value.clamp(0.0, self.max);
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
        Self::new(DEFAULT_HEALTH)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_damage_and_heal() {
        let mut health = Health::new(100.0);
        
        health.damage(30.0);
        assert_eq!(health.current, 70.0);
        
        health.heal(20.0);
        assert_eq!(health.current, 90.0);
        
        health.heal(50.0);
        assert_eq!(health.current, 100.0); // Clamped to max
    }
    
    #[test]
    fn health_death() {
        let mut health = Health::new(100.0);
        
        assert!(!health.is_dead());
        
        health.damage(150.0);
        assert_eq!(health.current, 0.0);
        assert!(health.is_dead());
    }
    
    #[test]
    fn health_full() {
        let health = Health::full(150.0);
        assert_eq!(health.current, 150.0);
        assert_eq!(health.max, 150.0);
        assert!(health.is_full());
    }
    
    #[test]
    fn health_set() {
        let mut health = Health::new(100.0);
        
        health.set(50.0);
        assert_eq!(health.current, 50.0);
        
        health.set(-10.0);
        assert_eq!(health.current, 0.0); // Clamped to 0
        
        health.set(200.0);
        assert_eq!(health.current, 100.0); // Clamped to max
    }
    
    #[test]
    fn health_is_full() {
        let mut health = Health::new(100.0);
        assert!(health.is_full());
        
        health.damage(1.0);
        assert!(!health.is_full());
        
        health.heal(1.0);
        assert!(health.is_full());
    }
    
    #[test]
    fn health_percentage() {
        let mut health = Health::new(100.0);
        assert_eq!(health.percentage(), 1.0);
        
        health.damage(25.0);
        assert_eq!(health.percentage(), 0.75);
        
        health.damage(75.0);
        assert_eq!(health.percentage(), 0.0);
        
        // Test zero max health
        let zero_health = Health { current: 0.0, max: 0.0 };
        assert_eq!(zero_health.percentage(), 0.0);
    }
    
    #[test]
    fn health_default() {
        let health = Health::default();
        assert_eq!(health.current, 100.0);
        assert_eq!(health.max, 100.0);
        assert!(health.is_full());
    }
}