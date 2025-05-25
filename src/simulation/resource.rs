use crate::Vec2;
use crate::core::Entity;
use crate::config::resource::*;

#[derive(Debug, Clone)]
pub struct Resource {
    pub id: Entity,
    pub position: Vec2,
    pub resource_type: ResourceType,
    pub amount: f32,
    pub max_amount: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResourceType {
    Food,
    Water,
}

impl ResourceType {
    pub fn regeneration_rate(&self) -> f32 {
        match self {
            ResourceType::Food => FOOD_REGENERATION_RATE,
            ResourceType::Water => WATER_REGENERATION_RATE,
        }
    }
    
    pub fn consumption_rate(&self) -> f32 {
        match self {
            ResourceType::Food => FOOD_CONSUMPTION_RATE,
            ResourceType::Water => WATER_CONSUMPTION_RATE,
        }
    }
    
    pub fn color(&self) -> [f32; 4] {
        match self {
            ResourceType::Food => [0.2, 0.8, 0.2, 1.0], // Green
            ResourceType::Water => [0.2, 0.6, 1.0, 1.0], // Blue
        }
    }
    
    pub fn default_max_amount(&self) -> f32 {
        match self {
            ResourceType::Food => DEFAULT_FOOD_AMOUNT,
            ResourceType::Water => DEFAULT_WATER_AMOUNT,
        }
    }
}

impl Resource {
    pub fn new(id: Entity, position: Vec2, resource_type: ResourceType) -> Self {
        let max_amount = resource_type.default_max_amount();
        Self {
            id,
            position,
            resource_type,
            amount: max_amount,
            max_amount,
        }
    }
    
    pub fn with_amount(mut self, amount: f32) -> Self {
        self.amount = amount.clamp(0.0, self.max_amount);
        self
    }
    
    pub fn consume(&mut self, amount: f32) -> f32 {
        let consumed = amount.min(self.amount);
        self.amount -= consumed;
        consumed
    }
    
    pub fn regenerate(&mut self, dt: f32) {
        if self.amount < self.max_amount {
            let regen = self.resource_type.regeneration_rate() * dt;
            self.amount = (self.amount + regen).min(self.max_amount);
        }
    }
    
    pub fn is_depleted(&self) -> bool {
        self.amount <= 0.0
    }
    
    pub fn is_full(&self) -> bool {
        self.amount >= self.max_amount
    }
    
    pub fn percentage(&self) -> f32 {
        if self.max_amount > 0.0 {
            self.amount / self.max_amount
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_creation() {
        let resource = Resource::new(
            Entity::new(1),
            Vec2::new(10.0, 20.0),
            ResourceType::Food
        );
        
        assert_eq!(resource.amount, DEFAULT_FOOD_AMOUNT);
        assert_eq!(resource.max_amount, DEFAULT_FOOD_AMOUNT);
        assert!(!resource.is_depleted());
    }
    
    #[test]
    fn resource_consumption() {
        let mut resource = Resource::new(
            Entity::new(1),
            Vec2::ZERO,
            ResourceType::Water
        );
        
        let consumed = resource.consume(50.0);
        assert_eq!(consumed, 50.0);
        assert_eq!(resource.amount, DEFAULT_WATER_AMOUNT - 50.0);
        
        let consumed = resource.consume(200.0);
        assert_eq!(consumed, DEFAULT_WATER_AMOUNT - 50.0);
        assert_eq!(resource.amount, 0.0);
        assert!(resource.is_depleted());
    }
    
    #[test]
    fn resource_regeneration() {
        let mut resource = Resource::new(
            Entity::new(1),
            Vec2::ZERO,
            ResourceType::Food
        ).with_amount(40.0); // Start below max to allow regeneration
        
        resource.regenerate(10.0); // 10 seconds
        let expected = 40.0 + FOOD_REGENERATION_RATE * 10.0;
        assert!((resource.amount - expected).abs() < 0.01, 
            "Expected {}, got {}", expected, resource.amount);
        
        resource.amount = resource.max_amount;
        resource.regenerate(10.0);
        assert_eq!(resource.amount, resource.max_amount); // No regen when full
    }
    
    #[test]
    fn resource_type_properties() {
        // Test regeneration rates
        assert_eq!(ResourceType::Food.regeneration_rate(), FOOD_REGENERATION_RATE);
        assert_eq!(ResourceType::Water.regeneration_rate(), WATER_REGENERATION_RATE);
        
        // Test consumption rates
        assert_eq!(ResourceType::Food.consumption_rate(), FOOD_CONSUMPTION_RATE);
        assert_eq!(ResourceType::Water.consumption_rate(), WATER_CONSUMPTION_RATE);
        
        // Test colors
        assert_eq!(ResourceType::Food.color(), [0.2, 0.8, 0.2, 1.0]);
        assert_eq!(ResourceType::Water.color(), [0.2, 0.6, 1.0, 1.0]);
        
        // Test default amounts
        assert_eq!(ResourceType::Food.default_max_amount(), DEFAULT_FOOD_AMOUNT);
        assert_eq!(ResourceType::Water.default_max_amount(), DEFAULT_WATER_AMOUNT);
    }
    
    #[test]
    fn resource_with_amount() {
        let resource = Resource::new(
            Entity::new(1),
            Vec2::ZERO,
            ResourceType::Food
        ).with_amount(25.0);
        
        assert_eq!(resource.amount, 25.0);
        assert_eq!(resource.max_amount, DEFAULT_FOOD_AMOUNT);
        
        // Test clamping
        let resource2 = Resource::new(
            Entity::new(2),
            Vec2::ZERO,
            ResourceType::Water
        ).with_amount(300.0);
        
        assert_eq!(resource2.amount, DEFAULT_WATER_AMOUNT); // Clamped to max
    }
    
    #[test]
    fn resource_is_full() {
        let mut resource = Resource::new(
            Entity::new(1),
            Vec2::ZERO,
            ResourceType::Food
        );
        
        assert!(resource.is_full());
        
        resource.consume(1.0);
        assert!(!resource.is_full());
        
        resource.amount = resource.max_amount;
        assert!(resource.is_full());
    }
    
    #[test]
    fn resource_percentage() {
        let mut resource = Resource::new(
            Entity::new(1),
            Vec2::ZERO,
            ResourceType::Water
        );
        
        assert_eq!(resource.percentage(), 1.0);
        
        resource.consume(DEFAULT_WATER_AMOUNT / 2.0);
        assert_eq!(resource.percentage(), 0.5); // Half consumed
        
        resource.consume(100.0);
        assert_eq!(resource.percentage(), 0.0);
        
        // Test zero max amount
        resource.max_amount = 0.0;
        assert_eq!(resource.percentage(), 0.0);
    }
}