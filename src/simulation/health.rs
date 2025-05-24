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
        Self::new(100.0)
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
}