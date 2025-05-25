//! Configuration constants and structures for the creature simulation.
//! 
//! This module centralizes all game balance values, system constants, and
//! configuration parameters to make tuning and testing easier.

use crate::Vec2;

/// Entity system configuration constants
pub mod entity {
    /// Initial capacity for entity ID allocation
    pub const INITIAL_CAPACITY: usize = 1000;
    
    /// Initial capacity for recycled ID storage
    pub const RECYCLED_CAPACITY: usize = 100;
    
    /// Threshold before entity ID overflow (with safety margin)
    pub const ID_OVERFLOW_THRESHOLD: u32 = u32::MAX - 10000;
}

/// Time system configuration constants
pub mod time {
    /// Default fixed timestep for simulation (60 FPS)
    pub const FIXED_TIMESTEP: f32 = 1.0 / 60.0;
    
    /// Maximum delta time to prevent spiral of death
    pub const MAX_DELTA: f32 = 0.1;
    
    /// Maximum time scale for fast-forward (Phase 1 limit)
    pub const MAX_TIME_SCALE: f32 = 10.0;
    
    /// Maximum simulation steps per update to prevent freezing
    pub const MAX_STEPS_PER_UPDATE: u32 = 10;
}

/// Spatial grid configuration constants
pub mod spatial {
    /// Default spatial grid cell size in world units
    pub const DEFAULT_CELL_SIZE: f32 = 50.0;
    
    /// Initial capacity for cell HashMap
    pub const CELL_CAPACITY: usize = 100;
    
    /// Initial capacity for entity position tracking
    pub const ENTITY_CAPACITY: usize = 1000;
    
    /// Query buffer pre-allocation size
    pub const QUERY_BUFFER_CAPACITY: usize = 50;
}

/// Error handling configuration
pub mod error {
    /// Maximum number of errors to keep in the error log
    pub const MAX_LOG_SIZE: usize = 1000;
}

/// Creature behavior and stats configuration
pub mod creature {
    /// Base movement speed for creatures
    pub const BASE_SPEED: f32 = 10.0;
    
    /// Default maximum health
    pub const DEFAULT_HEALTH: f32 = 100.0;
    
    /// Age threshold for death by old age (in seconds)
    pub const OLD_AGE_THRESHOLD: f32 = 300.0;
}

/// Need system configuration
pub mod needs {
    /// Default hunger depletion rate per second
    pub const DEFAULT_HUNGER_RATE: f32 = 0.1;
    
    /// Default thirst depletion rate per second
    pub const DEFAULT_THIRST_RATE: f32 = 0.15;
    
    /// Default energy depletion rate per second
    pub const DEFAULT_ENERGY_RATE: f32 = 0.05;
    
    /// Critical threshold for urgent needs (90% depleted)
    pub const CRITICAL_THRESHOLD: f32 = 0.9;
    
    /// Low energy threshold (10% remaining)
    pub const LOW_ENERGY_THRESHOLD: f32 = 0.1;
    
    /// Damage per second when starving
    pub const STARVATION_DAMAGE: f32 = 10.0;
    
    /// Damage per second when dehydrated
    pub const DEHYDRATION_DAMAGE: f32 = 15.0;
    
    /// Damage per second when exhausted
    pub const EXHAUSTION_DAMAGE: f32 = 5.0;
}

/// Resource system configuration
pub mod resource {
    /// Default amount of food in a food resource
    pub const DEFAULT_FOOD_AMOUNT: f32 = 50.0;
    
    /// Default amount of water in a water resource
    pub const DEFAULT_WATER_AMOUNT: f32 = 100.0;
    
    /// Food consumption rate per second
    pub const FOOD_CONSUMPTION_RATE: f32 = 1.0;
    
    /// Water consumption rate per second
    pub const WATER_CONSUMPTION_RATE: f32 = 2.0;
    
    /// Food regeneration rate per second
    pub const FOOD_REGENERATION_RATE: f32 = 0.1;
    
    /// Water regeneration rate per second
    pub const WATER_REGENERATION_RATE: f32 = 0.5;
    
    /// How much a creature's hunger is satisfied per unit of food
    pub const FOOD_SATISFACTION_MULTIPLIER: f32 = 2.0;
}

/// AI decision-making configuration
pub mod decision {
    /// Search radius for finding resources
    pub const SEARCH_RADIUS: f32 = 50.0;
    
    /// Urgency threshold for taking action
    pub const URGENCY_THRESHOLD: f32 = 0.3;
    
    /// Distance threshold for resource interaction
    pub const INTERACTION_DISTANCE: f32 = 2.0;
}

/// Movement system configuration
pub mod movement {
    /// Distance threshold for arrival detection
    pub const ARRIVAL_THRESHOLD: f32 = 1.0;
    
    /// Steering force as percentage of max speed
    pub const STEERING_FORCE_RATIO: f32 = 0.1;
}

/// Interaction system configuration
pub mod interaction {
    /// Maximum range for creature interactions
    pub const MAX_INTERACTION_RANGE: f32 = 3.0;
}

/// Complete game configuration structure
#[derive(Debug, Clone)]
pub struct GameConfig {
    /// Creature-specific configuration
    pub creature: CreatureConfig,
    
    /// World configuration
    pub world: WorldConfig,
    
    /// Need system rates
    pub needs: NeedRates,
    
    /// Resource configuration
    pub resources: ResourceConfig,
    
    /// AI decision parameters
    pub ai: AIConfig,
    
    /// Movement parameters
    pub movement: MovementConfig,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            creature: CreatureConfig::default(),
            world: WorldConfig::default(),
            needs: NeedRates::default(),
            resources: ResourceConfig::default(),
            ai: AIConfig::default(),
            movement: MovementConfig::default(),
        }
    }
}

/// Creature-specific configuration
#[derive(Debug, Clone)]
pub struct CreatureConfig {
    pub base_speed: f32,
    pub default_health: f32,
    pub old_age_threshold: f32,
}

impl Default for CreatureConfig {
    fn default() -> Self {
        Self {
            base_speed: creature::BASE_SPEED,
            default_health: creature::DEFAULT_HEALTH,
            old_age_threshold: creature::OLD_AGE_THRESHOLD,
        }
    }
}

/// World configuration
#[derive(Debug, Clone)]
pub struct WorldConfig {
    pub bounds: Option<(Vec2, Vec2)>,
    pub spatial_cell_size: f32,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            bounds: None,
            spatial_cell_size: spatial::DEFAULT_CELL_SIZE,
        }
    }
}

/// Need system depletion rates
#[derive(Debug, Clone)]
pub struct NeedRates {
    pub hunger_rate: f32,
    pub thirst_rate: f32,
    pub energy_rate: f32,
    pub critical_threshold: f32,
    pub low_energy_threshold: f32,
    pub starvation_damage: f32,
    pub dehydration_damage: f32,
    pub exhaustion_damage: f32,
}

impl Default for NeedRates {
    fn default() -> Self {
        Self {
            hunger_rate: needs::DEFAULT_HUNGER_RATE,
            thirst_rate: needs::DEFAULT_THIRST_RATE,
            energy_rate: needs::DEFAULT_ENERGY_RATE,
            critical_threshold: needs::CRITICAL_THRESHOLD,
            low_energy_threshold: needs::LOW_ENERGY_THRESHOLD,
            starvation_damage: needs::STARVATION_DAMAGE,
            dehydration_damage: needs::DEHYDRATION_DAMAGE,
            exhaustion_damage: needs::EXHAUSTION_DAMAGE,
        }
    }
}

/// Resource system configuration
#[derive(Debug, Clone)]
pub struct ResourceConfig {
    pub default_food_amount: f32,
    pub default_water_amount: f32,
    pub food_consumption_rate: f32,
    pub water_consumption_rate: f32,
    pub food_regeneration_rate: f32,
    pub water_regeneration_rate: f32,
    pub food_satisfaction_multiplier: f32,
}

impl Default for ResourceConfig {
    fn default() -> Self {
        Self {
            default_food_amount: resource::DEFAULT_FOOD_AMOUNT,
            default_water_amount: resource::DEFAULT_WATER_AMOUNT,
            food_consumption_rate: resource::FOOD_CONSUMPTION_RATE,
            water_consumption_rate: resource::WATER_CONSUMPTION_RATE,
            food_regeneration_rate: resource::FOOD_REGENERATION_RATE,
            water_regeneration_rate: resource::WATER_REGENERATION_RATE,
            food_satisfaction_multiplier: resource::FOOD_SATISFACTION_MULTIPLIER,
        }
    }
}

/// AI decision-making configuration
#[derive(Debug, Clone)]
pub struct AIConfig {
    pub search_radius: f32,
    pub urgency_threshold: f32,
    pub interaction_distance: f32,
}

impl Default for AIConfig {
    fn default() -> Self {
        Self {
            search_radius: decision::SEARCH_RADIUS,
            urgency_threshold: decision::URGENCY_THRESHOLD,
            interaction_distance: decision::INTERACTION_DISTANCE,
        }
    }
}

/// Movement system configuration
#[derive(Debug, Clone)]
pub struct MovementConfig {
    pub arrival_threshold: f32,
    pub steering_force_ratio: f32,
}

impl Default for MovementConfig {
    fn default() -> Self {
        Self {
            arrival_threshold: movement::ARRIVAL_THRESHOLD,
            steering_force_ratio: movement::STEERING_FORCE_RATIO,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn game_config_default() {
        let config = GameConfig::default();
        
        // Verify all sub-configs are properly initialized
        assert_eq!(config.creature.base_speed, creature::BASE_SPEED);
        assert_eq!(config.world.spatial_cell_size, spatial::DEFAULT_CELL_SIZE);
        assert_eq!(config.needs.hunger_rate, needs::DEFAULT_HUNGER_RATE);
        assert_eq!(config.resources.default_food_amount, resource::DEFAULT_FOOD_AMOUNT);
        assert_eq!(config.ai.search_radius, decision::SEARCH_RADIUS);
        assert_eq!(config.movement.arrival_threshold, movement::ARRIVAL_THRESHOLD);
    }
    
    #[test]
    fn creature_config_default() {
        let config = CreatureConfig::default();
        assert_eq!(config.base_speed, creature::BASE_SPEED);
        assert_eq!(config.default_health, creature::DEFAULT_HEALTH);
        assert_eq!(config.old_age_threshold, creature::OLD_AGE_THRESHOLD);
    }
    
    #[test]
    fn world_config_default() {
        let config = WorldConfig::default();
        assert!(config.bounds.is_none());
        assert_eq!(config.spatial_cell_size, spatial::DEFAULT_CELL_SIZE);
    }
    
    #[test]
    fn need_rates_default() {
        let rates = NeedRates::default();
        assert_eq!(rates.hunger_rate, needs::DEFAULT_HUNGER_RATE);
        assert_eq!(rates.thirst_rate, needs::DEFAULT_THIRST_RATE);
        assert_eq!(rates.energy_rate, needs::DEFAULT_ENERGY_RATE);
        assert_eq!(rates.critical_threshold, needs::CRITICAL_THRESHOLD);
        assert_eq!(rates.low_energy_threshold, needs::LOW_ENERGY_THRESHOLD);
        assert_eq!(rates.starvation_damage, needs::STARVATION_DAMAGE);
        assert_eq!(rates.dehydration_damage, needs::DEHYDRATION_DAMAGE);
        assert_eq!(rates.exhaustion_damage, needs::EXHAUSTION_DAMAGE);
    }
    
    #[test]
    fn resource_config_default() {
        let config = ResourceConfig::default();
        assert_eq!(config.default_food_amount, resource::DEFAULT_FOOD_AMOUNT);
        assert_eq!(config.default_water_amount, resource::DEFAULT_WATER_AMOUNT);
        assert_eq!(config.food_consumption_rate, resource::FOOD_CONSUMPTION_RATE);
        assert_eq!(config.water_consumption_rate, resource::WATER_CONSUMPTION_RATE);
        assert_eq!(config.food_regeneration_rate, resource::FOOD_REGENERATION_RATE);
        assert_eq!(config.water_regeneration_rate, resource::WATER_REGENERATION_RATE);
        assert_eq!(config.food_satisfaction_multiplier, resource::FOOD_SATISFACTION_MULTIPLIER);
    }
    
    #[test]
    fn ai_config_default() {
        let config = AIConfig::default();
        assert_eq!(config.search_radius, decision::SEARCH_RADIUS);
        assert_eq!(config.urgency_threshold, decision::URGENCY_THRESHOLD);
        assert_eq!(config.interaction_distance, decision::INTERACTION_DISTANCE);
    }
    
    #[test]
    fn movement_config_default() {
        let config = MovementConfig::default();
        assert_eq!(config.arrival_threshold, movement::ARRIVAL_THRESHOLD);
        assert_eq!(config.steering_force_ratio, movement::STEERING_FORCE_RATIO);
    }
    
    #[test]
    fn constants_are_positive() {
        // Entity constants
        assert!(entity::INITIAL_CAPACITY > 0);
        assert!(entity::RECYCLED_CAPACITY > 0);
        assert!(entity::ID_OVERFLOW_THRESHOLD > 0);
        
        // Time constants
        assert!(time::FIXED_TIMESTEP > 0.0);
        assert!(time::MAX_DELTA > 0.0);
        assert!(time::MAX_TIME_SCALE > 0.0);
        assert!(time::MAX_STEPS_PER_UPDATE > 0);
        
        // Spatial constants
        assert!(spatial::DEFAULT_CELL_SIZE > 0.0);
        assert!(spatial::CELL_CAPACITY > 0);
        assert!(spatial::ENTITY_CAPACITY > 0);
        assert!(spatial::QUERY_BUFFER_CAPACITY > 0);
        
        // Creature constants
        assert!(creature::BASE_SPEED > 0.0);
        assert!(creature::DEFAULT_HEALTH > 0.0);
        assert!(creature::OLD_AGE_THRESHOLD > 0.0);
        
        // Resource constants
        assert!(resource::DEFAULT_FOOD_AMOUNT > 0.0);
        assert!(resource::DEFAULT_WATER_AMOUNT > 0.0);
    }
    
    #[test]
    fn config_clone() {
        let config = GameConfig::default();
        let cloned = config.clone();
        
        assert_eq!(config.creature.base_speed, cloned.creature.base_speed);
        assert_eq!(config.needs.hunger_rate, cloned.needs.hunger_rate);
    }
}