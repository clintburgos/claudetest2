//! Simulation configuration resource

use bevy::prelude::*;

/// Global simulation configuration
#[derive(Resource, Debug, Clone)]
pub struct SimulationConfig {
    /// Base movement speed for creatures
    pub creature_base_speed: f32,
    
    /// Whether debug mode is enabled
    pub debug_mode: bool,
    
    /// Maximum number of creatures
    pub max_creatures: usize,
    
    /// World size (radius)
    pub world_radius: f32,
    
    /// Resource spawn rate (per second)
    pub resource_spawn_rate: f32,
    
    /// Maximum resources in world
    pub max_resources: usize,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            creature_base_speed: 50.0,
            debug_mode: false,
            max_creatures: 1000,
            world_radius: 1000.0,
            resource_spawn_rate: 0.5,
            max_resources: 200,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = SimulationConfig::default();
        assert_eq!(config.creature_base_speed, 50.0);
        assert!(!config.debug_mode);
        assert_eq!(config.max_creatures, 1000);
    }
}