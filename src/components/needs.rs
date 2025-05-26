//! Need system components

use bevy::prelude::*;

/// Creature needs (hunger, thirst, energy, social)
#[derive(Component, Debug, Clone)]
pub struct Needs {
    pub hunger: f32,
    pub thirst: f32,
    pub energy: f32,
    pub social: f32,
}

impl Needs {
    /// Returns the most urgent need type and its urgency
    pub fn most_urgent(&self) -> (NeedType, f32) {
        // For energy, low values are urgent (inverted)
        let energy_urgency = 1.0 - self.energy;
        let mut most_urgent = (NeedType::Energy, energy_urgency);

        if self.hunger > most_urgent.1 {
            most_urgent = (NeedType::Hunger, self.hunger);
        }
        if self.thirst > most_urgent.1 {
            most_urgent = (NeedType::Thirst, self.thirst);
        }

        most_urgent
    }

    /// Checks if any need is critical (>0.8)
    pub fn has_critical_need(&self) -> bool {
        self.hunger > 0.8 || self.thirst > 0.8 || self.energy < 0.2
    }
}

impl Default for Needs {
    fn default() -> Self {
        Self {
            hunger: 0.0,
            thirst: 0.0,
            energy: 1.0, // Start with full energy
            social: 0.0,
        }
    }
}

/// Type of need
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NeedType {
    Hunger,
    Thirst,
    Energy,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_needs_default() {
        let needs = Needs::default();
        assert_eq!(needs.hunger, 0.0);
        assert_eq!(needs.thirst, 0.0);
        assert_eq!(needs.energy, 1.0);
        assert_eq!(needs.social, 0.0);
    }
    
    #[test]
    fn test_most_urgent_hunger() {
        let needs = Needs {
            hunger: 0.9,
            thirst: 0.3,
            energy: 0.8, // Low urgency (0.2)
            social: 0.5,
        };
        
        let (need_type, urgency) = needs.most_urgent();
        assert_eq!(need_type, NeedType::Hunger);
        assert_eq!(urgency, 0.9);
    }
    
    #[test]
    fn test_most_urgent_thirst() {
        let needs = Needs {
            hunger: 0.4,
            thirst: 0.95,
            energy: 0.6, // Low urgency (0.4)
            social: 0.2,
        };
        
        let (need_type, urgency) = needs.most_urgent();
        assert_eq!(need_type, NeedType::Thirst);
        assert_eq!(urgency, 0.95);
    }
    
    #[test]
    fn test_most_urgent_energy() {
        let needs = Needs {
            hunger: 0.3,
            thirst: 0.4,
            energy: 0.1, // High urgency (0.9)
            social: 0.5,
        };
        
        let (need_type, urgency) = needs.most_urgent();
        assert_eq!(need_type, NeedType::Energy);
        assert_eq!(urgency, 0.9);
    }
    
    #[test]
    fn test_has_critical_need() {
        let mut needs = Needs::default();
        assert!(!needs.has_critical_need());
        
        // Test critical hunger
        needs.hunger = 0.85;
        assert!(needs.has_critical_need());
        
        // Test critical thirst
        needs.hunger = 0.5;
        needs.thirst = 0.9;
        assert!(needs.has_critical_need());
        
        // Test critical energy (low)
        needs.thirst = 0.5;
        needs.energy = 0.15;
        assert!(needs.has_critical_need());
        
        // Test no critical needs
        needs.hunger = 0.7;
        needs.thirst = 0.7;
        needs.energy = 0.3;
        assert!(!needs.has_critical_need());
    }
    
    #[test]
    fn test_needs_clone() {
        let original = Needs {
            hunger: 0.5,
            thirst: 0.6,
            energy: 0.7,
            social: 0.8,
        };
        let cloned = original.clone();
        
        assert_eq!(cloned.hunger, original.hunger);
        assert_eq!(cloned.thirst, original.thirst);
        assert_eq!(cloned.energy, original.energy);
        assert_eq!(cloned.social, original.social);
    }
    
    #[test]
    fn test_need_type_equality() {
        assert_eq!(NeedType::Hunger, NeedType::Hunger);
        assert_ne!(NeedType::Hunger, NeedType::Thirst);
        assert_ne!(NeedType::Thirst, NeedType::Energy);
    }
    
    #[test]
    fn test_need_type_copy() {
        let original = NeedType::Thirst;
        let copied = original;
        assert_eq!(original, copied);
    }
    
    #[test]
    fn test_edge_case_all_needs_equal() {
        let needs = Needs {
            hunger: 0.5,
            thirst: 0.5,
            energy: 0.5, // Urgency will be 0.5
            social: 0.5,
        };
        
        // When all are equal, energy should win due to order of checks
        let (need_type, urgency) = needs.most_urgent();
        assert_eq!(need_type, NeedType::Energy);
        assert_eq!(urgency, 0.5);
    }
    
    #[test]
    fn test_boundary_values() {
        let needs = Needs {
            hunger: 1.0,
            thirst: 0.0,
            energy: 0.0, // Urgency = 1.0
            social: 0.5,
        };
        
        // Both hunger and energy have urgency 1.0, energy checked first
        let (need_type, urgency) = needs.most_urgent();
        assert!(need_type == NeedType::Energy || need_type == NeedType::Hunger);
        assert_eq!(urgency, 1.0);
    }
}
