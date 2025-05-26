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
