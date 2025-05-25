#[derive(Debug, Clone)]
pub struct Needs {
    pub hunger: f32,    // 0.0 = full, 1.0 = starving
    pub thirst: f32,    // 0.0 = hydrated, 1.0 = dehydrated  
    pub energy: f32,    // 0.0 = exhausted, 1.0 = full energy
}

impl Needs {
    pub fn new() -> Self {
        Self {
            hunger: 0.3,
            thirst: 0.3,
            energy: 0.8,
        }
    }
    
    pub fn update(&mut self, dt: f32, metabolism_multiplier: f32) {
        // Base rates per second
        const HUNGER_RATE: f32 = 0.01;
        const THIRST_RATE: f32 = 0.015;
        const ENERGY_DRAIN_RATE: f32 = 0.008;
        
        self.hunger += HUNGER_RATE * dt * metabolism_multiplier;
        self.thirst += THIRST_RATE * dt * metabolism_multiplier;
        self.energy -= ENERGY_DRAIN_RATE * dt * metabolism_multiplier;
        
        self.clamp();
    }
    
    pub fn eat(&mut self, amount: f32) {
        self.hunger -= amount;
        self.clamp();
    }
    
    pub fn drink(&mut self, amount: f32) {
        self.thirst -= amount;
        self.clamp();
    }
    
    pub fn rest(&mut self, amount: f32) {
        self.energy += amount;
        self.clamp();
    }
    
    pub fn is_critical(&self) -> bool {
        self.hunger >= 0.9 || self.thirst >= 0.9 || self.energy <= 0.1
    }
    
    pub fn most_urgent(&self) -> NeedType {
        // Inverted energy since low energy is bad
        let energy_urgency = 1.0 - self.energy;
        
        if self.thirst >= self.hunger && self.thirst >= energy_urgency {
            NeedType::Thirst
        } else if self.hunger >= energy_urgency {
            NeedType::Hunger
        } else {
            NeedType::Energy
        }
    }
    
    pub fn get_urgency(&self, need_type: NeedType) -> f32 {
        match need_type {
            NeedType::Hunger => self.hunger,
            NeedType::Thirst => self.thirst,
            NeedType::Energy => 1.0 - self.energy,
        }
    }
    
    fn clamp(&mut self) {
        self.hunger = self.hunger.clamp(0.0, 1.0);
        self.thirst = self.thirst.clamp(0.0, 1.0);
        self.energy = self.energy.clamp(0.0, 1.0);
    }
}

impl Default for Needs {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NeedType {
    Hunger,
    Thirst,
    Energy,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn needs_update() {
        let mut needs = Needs::new();
        let initial_hunger = needs.hunger;
        
        needs.update(1.0, 1.0); // 1 second at normal metabolism
        
        assert!(needs.hunger > initial_hunger);
        assert!(needs.thirst > 0.3);
        assert!(needs.energy < 0.8);
    }
    
    #[test]
    fn needs_satisfaction() {
        let mut needs = Needs::new();
        needs.hunger = 0.8;
        needs.thirst = 0.8;
        needs.energy = 0.2;
        
        needs.eat(0.5);
        assert_eq!(needs.hunger, 0.3);
        
        needs.drink(0.5);
        assert_eq!(needs.thirst, 0.3);
        
        needs.rest(0.5);
        assert_eq!(needs.energy, 0.7);
    }
    
    #[test]
    fn needs_urgency() {
        let mut needs = Needs::new();
        needs.hunger = 0.8;
        needs.thirst = 0.3;
        needs.energy = 0.9;
        
        assert_eq!(needs.most_urgent(), NeedType::Hunger);
        
        needs.thirst = 0.9;
        assert_eq!(needs.most_urgent(), NeedType::Thirst);
        
        needs.energy = 0.05;
        assert_eq!(needs.most_urgent(), NeedType::Energy);
    }
    
    #[test]
    fn needs_is_critical() {
        let mut needs = Needs::new();
        assert!(!needs.is_critical());
        
        needs.hunger = 0.9;
        assert!(needs.is_critical());
        
        needs.hunger = 0.5;
        needs.thirst = 0.95;
        assert!(needs.is_critical());
        
        needs.thirst = 0.5;
        needs.energy = 0.05;
        assert!(needs.is_critical());
    }
    
    #[test]
    fn needs_get_urgency() {
        let mut needs = Needs::new();
        needs.hunger = 0.6;
        needs.thirst = 0.4;
        needs.energy = 0.3; // Low energy = high urgency
        
        assert_eq!(needs.get_urgency(NeedType::Hunger), 0.6);
        assert_eq!(needs.get_urgency(NeedType::Thirst), 0.4);
        assert_eq!(needs.get_urgency(NeedType::Energy), 0.7); // 1.0 - 0.3
    }
    
    #[test]
    fn needs_clamping() {
        let mut needs = Needs::new();
        
        // Test over-eating
        needs.hunger = 0.1;
        needs.eat(0.5);
        assert_eq!(needs.hunger, 0.0); // Clamped to 0
        
        // Test starvation
        needs.hunger = 0.9;
        needs.update(100.0, 1.0); // Long time
        assert_eq!(needs.hunger, 1.0); // Clamped to 1
        
        // Test energy recovery
        needs.energy = 0.8;
        needs.rest(0.5);
        assert_eq!(needs.energy, 1.0); // Clamped to 1
    }
    
    #[test]
    fn needs_default() {
        let needs = Needs::default();
        assert_eq!(needs.hunger, 0.3);
        assert_eq!(needs.thirst, 0.3);
        assert_eq!(needs.energy, 0.8);
    }
}