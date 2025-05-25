use crate::config::needs::*;

/// Configuration for need update rates
#[derive(Debug, Clone)]
pub struct NeedRates {
    pub hunger_per_second: f32,
    pub thirst_per_second: f32,
    pub energy_drain_per_second: f32,
    pub energy_recovery_per_second: f32,
}

impl Default for NeedRates {
    fn default() -> Self {
        Self {
            hunger_per_second: DEFAULT_HUNGER_RATE,
            thirst_per_second: DEFAULT_THIRST_RATE,
            energy_drain_per_second: DEFAULT_ENERGY_RATE,
            energy_recovery_per_second: DEFAULT_ENERGY_RATE * 10.0, // Recovery is faster
        }
    }
}

/// Represents a creature's basic needs
/// 
/// All need values range from 0.0 (satisfied) to 1.0 (critical),
/// except energy which is inverted: 0.0 (exhausted) to 1.0 (full).
#[derive(Debug, Clone)]
pub struct Needs {
    /// Hunger level: 0.0 = full, 1.0 = starving
    pub hunger: f32,
    /// Thirst level: 0.0 = hydrated, 1.0 = dehydrated
    pub thirst: f32,
    /// Energy level: 0.0 = exhausted, 1.0 = full energy
    pub energy: f32,
    /// Configuration for update rates
    rates: NeedRates,
}

impl Needs {
    /// Creates new needs with default starting values
    pub fn new() -> Self {
        Self {
            hunger: 0.3,
            thirst: 0.3,
            energy: 0.8,
            rates: NeedRates::default(),
        }
    }
    
    /// Creates needs with custom configuration
    pub fn with_rates(rates: NeedRates) -> Self {
        Self {
            hunger: 0.3,
            thirst: 0.3,
            energy: 0.8,
            rates,
        }
    }
    
    /// Updates needs based on time and environmental factors
    /// 
    /// # Arguments
    /// * `dt` - Time elapsed in seconds
    /// * `metabolism_multiplier` - Metabolism rate multiplier (e.g., from creature size)
    /// * `env_factors` - Environmental factors affecting needs
    pub fn update(&mut self, dt: f32, metabolism_multiplier: f32, env_factors: &EnvironmentalFactors) {
        // Apply base rates with metabolism
        let hunger_rate = self.rates.hunger_per_second * metabolism_multiplier * env_factors.hunger_multiplier;
        let thirst_rate = self.rates.thirst_per_second * metabolism_multiplier * env_factors.thirst_multiplier;
        let energy_rate = self.rates.energy_drain_per_second * metabolism_multiplier * env_factors.energy_multiplier;
        
        self.hunger += hunger_rate * dt;
        self.thirst += thirst_rate * dt;
        self.energy -= energy_rate * dt;
        
        self.clamp();
    }
    
    /// Simple update without environmental factors (backwards compatibility)
    pub fn update_simple(&mut self, dt: f32, metabolism_multiplier: f32) {
        self.update(dt, metabolism_multiplier, &EnvironmentalFactors::default());
    }
    
    /// Satisfies hunger need
    /// 
    /// # Arguments
    /// * `amount` - Amount to reduce hunger (0.0 to 1.0)
    /// 
    /// # Returns
    /// Actual amount consumed after clamping
    pub fn eat(&mut self, amount: f32) -> f32 {
        let old_hunger = self.hunger;
        self.hunger -= amount;
        self.clamp();
        old_hunger - self.hunger
    }
    
    /// Satisfies thirst need
    /// 
    /// # Arguments
    /// * `amount` - Amount to reduce thirst (0.0 to 1.0)
    /// 
    /// # Returns
    /// Actual amount consumed after clamping
    pub fn drink(&mut self, amount: f32) -> f32 {
        let old_thirst = self.thirst;
        self.thirst -= amount;
        self.clamp();
        old_thirst - self.thirst
    }
    
    /// Recovers energy through rest
    /// 
    /// # Arguments
    /// * `dt` - Time spent resting in seconds
    /// 
    /// # Returns
    /// Actual energy recovered after clamping
    pub fn rest(&mut self, dt: f32) -> f32 {
        let old_energy = self.energy;
        self.energy += self.rates.energy_recovery_per_second * dt;
        self.clamp();
        self.energy - old_energy
    }
    
    /// Checks if any need is at critical level
    pub fn is_critical(&self) -> bool {
        self.hunger >= CRITICAL_THRESHOLD || 
        self.thirst >= CRITICAL_THRESHOLD || 
        self.energy <= LOW_ENERGY_THRESHOLD
    }
    
    /// Returns detailed critical status for each need
    pub fn critical_status(&self) -> CriticalStatus {
        CriticalStatus {
            hunger_critical: self.hunger >= CRITICAL_THRESHOLD,
            thirst_critical: self.thirst >= CRITICAL_THRESHOLD,
            energy_critical: self.energy <= LOW_ENERGY_THRESHOLD,
        }
    }
    
    /// Determines which need is most urgent
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
    
    /// Gets the urgency level for a specific need
    /// 
    /// # Returns
    /// Value from 0.0 (not urgent) to 1.0 (critical)
    pub fn get_urgency(&self, need_type: NeedType) -> f32 {
        match need_type {
            NeedType::Hunger => self.hunger,
            NeedType::Thirst => self.thirst,
            NeedType::Energy => 1.0 - self.energy,
        }
    }
    
    /// Sets custom update rates
    pub fn set_rates(&mut self, rates: NeedRates) {
        self.rates = rates;
    }
    
    /// Clamps all needs to valid ranges
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

/// Types of creature needs
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NeedType {
    Hunger,
    Thirst,
    Energy,
}

/// Environmental factors affecting need rates
#[derive(Debug, Clone)]
pub struct EnvironmentalFactors {
    /// Multiplier for hunger rate (e.g., cold increases hunger)
    pub hunger_multiplier: f32,
    /// Multiplier for thirst rate (e.g., heat increases thirst)
    pub thirst_multiplier: f32,
    /// Multiplier for energy drain (e.g., difficult terrain)
    pub energy_multiplier: f32,
}

impl Default for EnvironmentalFactors {
    fn default() -> Self {
        Self {
            hunger_multiplier: 1.0,
            thirst_multiplier: 1.0,
            energy_multiplier: 1.0,
        }
    }
}

/// Detailed critical status for needs
#[derive(Debug, Clone)]
pub struct CriticalStatus {
    pub hunger_critical: bool,
    pub thirst_critical: bool,
    pub energy_critical: bool,
}

impl CriticalStatus {
    /// Returns true if any need is critical
    pub fn any_critical(&self) -> bool {
        self.hunger_critical || self.thirst_critical || self.energy_critical
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn needs_update() {
        let mut needs = Needs::new();
        let initial_hunger = needs.hunger;
        
        needs.update_simple(1.0, 1.0); // 1 second at normal metabolism
        
        assert!(needs.hunger > initial_hunger);
        assert!(needs.thirst > 0.3);
        assert!(needs.energy < 0.8);
    }
    
    #[test]
    fn needs_update_with_environment() {
        let mut needs = Needs::new();
        let env = EnvironmentalFactors {
            hunger_multiplier: 2.0, // Double hunger rate
            thirst_multiplier: 0.5, // Half thirst rate
            energy_multiplier: 1.5, // 1.5x energy drain
        };
        
        let initial_hunger = needs.hunger;
        let initial_thirst = needs.thirst;
        let initial_energy = needs.energy;
        
        needs.update(1.0, 1.0, &env);
        
        // Hunger should increase more (2x the default rate)
        let expected_hunger_increase = DEFAULT_HUNGER_RATE * 1.0 * 2.0; // dt=1.0, multiplier=2.0
        assert!((needs.hunger - initial_hunger - expected_hunger_increase).abs() < 0.001);
        
        // Thirst should increase less (0.5x the default rate)
        let expected_thirst_increase = DEFAULT_THIRST_RATE * 1.0 * 0.5; // dt=1.0, multiplier=0.5
        assert!((needs.thirst - initial_thirst - expected_thirst_increase).abs() < 0.001);
        
        // Energy should decrease more (1.5x the default rate)
        let expected_energy_decrease = DEFAULT_ENERGY_RATE * 1.0 * 1.5; // dt=1.0, multiplier=1.5
        assert!((initial_energy - needs.energy - expected_energy_decrease).abs() < 0.001);
    }
    
    #[test]
    fn needs_satisfaction() {
        let mut needs = Needs::new();
        needs.hunger = 0.8;
        needs.thirst = 0.8;
        needs.energy = 0.2;
        
        let consumed = needs.eat(0.5);
        assert_eq!(needs.hunger, 0.3);
        assert_eq!(consumed, 0.5);
        
        let drunk = needs.drink(0.5);
        assert_eq!(needs.thirst, 0.3);
        assert_eq!(drunk, 0.5);
        
        let recovered = needs.rest(10.0); // 10 seconds of rest
        assert!(recovered > 0.0);
        assert!(needs.energy > 0.2);
    }
    
    #[test]
    fn needs_satisfaction_feedback() {
        let mut needs = Needs::new();
        needs.hunger = 0.1;
        
        // Try to eat more than needed
        let consumed = needs.eat(0.5);
        assert_eq!(consumed, 0.1); // Only consumed what was needed
        assert_eq!(needs.hunger, 0.0);
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
        needs.update_simple(100.0, 1.0); // Long time
        assert_eq!(needs.hunger, 1.0); // Clamped to 1
        
        // Test energy recovery
        needs.energy = 0.8;
        needs.rest(10.0);
        assert_eq!(needs.energy, 1.0); // Clamped to 1
    }
    
    #[test]
    fn critical_status() {
        let mut needs = Needs::new();
        
        let status = needs.critical_status();
        assert!(!status.any_critical());
        
        needs.hunger = 0.95;
        let status = needs.critical_status();
        assert!(status.hunger_critical);
        assert!(status.any_critical());
        
        needs.energy = 0.05;
        let status = needs.critical_status();
        assert!(status.energy_critical);
        assert!(status.any_critical());
    }
    
    #[test]
    fn custom_rates() {
        let rates = NeedRates {
            hunger_per_second: 0.02,
            thirst_per_second: 0.03,
            energy_drain_per_second: 0.01,
            energy_recovery_per_second: 0.1,
        };
        
        let mut needs = Needs::with_rates(rates);
        let initial_hunger = needs.hunger;
        
        needs.update_simple(1.0, 1.0);
        
        // Should use custom rates
        assert!((needs.hunger - initial_hunger - 0.02).abs() < 0.001);
    }
    
    #[test]
    fn needs_default() {
        let needs = Needs::default();
        assert_eq!(needs.hunger, 0.3);
        assert_eq!(needs.thirst, 0.3);
        assert_eq!(needs.energy, 0.8);
    }
}