//! Error boundary system for graceful error recovery

use bevy::prelude::*;
use std::collections::HashMap;
use std::fmt;
use std::time::{Duration, Instant};

/// Error types that can occur in the simulation
#[derive(Debug, Clone, PartialEq)]
pub enum SimulationError {
    CreatureStuck {
        entity: Entity,
        duration: f32,
    },
    InvalidPosition {
        entity: Entity,
        position: Vec2,
    },
    ResourceNegative {
        entity: Entity,
        resource: String,
        value: f32,
    },
    PathfindingFailed {
        entity: Entity,
        target: Vec2,
    },
    ComponentMissing {
        entity: Entity,
        component: &'static str,
    },
    SystemPanic {
        system: &'static str,
        message: String,
    },
}

impl fmt::Display for SimulationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SimulationError::CreatureStuck { entity, duration } => {
                write!(f, "Creature {:?} stuck for {:.1}s", entity, duration)
            },
            SimulationError::InvalidPosition { entity, position } => {
                write!(f, "Invalid position {:?} for entity {:?}", position, entity)
            },
            SimulationError::ResourceNegative {
                entity,
                resource,
                value,
            } => {
                write!(
                    f,
                    "Negative {} ({}) for entity {:?}",
                    resource, value, entity
                )
            },
            SimulationError::PathfindingFailed { entity, target } => {
                write!(f, "Pathfinding failed for {:?} to {:?}", entity, target)
            },
            SimulationError::ComponentMissing { entity, component } => {
                write!(f, "Missing component {} for entity {:?}", component, entity)
            },
            SimulationError::SystemPanic { system, message } => {
                write!(f, "System {} panicked: {}", system, message)
            },
        }
    }
}

/// Recovery strategies for different error types
pub trait RecoveryStrategy: Send + Sync {
    fn can_recover(&self, error: &SimulationError) -> bool;
    fn recover(&self, world: &mut World, error: &SimulationError) -> Result<(), String>;
}

/// Recovery for stuck creatures
pub struct CreatureStuckRecovery;

impl RecoveryStrategy for CreatureStuckRecovery {
    fn can_recover(&self, error: &SimulationError) -> bool {
        matches!(error, SimulationError::CreatureStuck { .. })
    }

    fn recover(&self, world: &mut World, error: &SimulationError) -> Result<(), String> {
        if let SimulationError::CreatureStuck {
            entity,
            duration: _,
        } = error
        {
            // Try to teleport creature to a nearby valid position
            if let Some(position) = world.get::<crate::components::Position>(*entity) {
                let current_pos = position.0;

                // Find nearby valid position
                let offsets = [
                    Vec2::new(50.0, 0.0),
                    Vec2::new(-50.0, 0.0),
                    Vec2::new(0.0, 50.0),
                    Vec2::new(0.0, -50.0),
                ];

                for offset in &offsets {
                    let new_pos = current_pos + *offset;
                    // Check if position is valid (simplified check)
                    if new_pos.x.abs() < 1000.0 && new_pos.y.abs() < 1000.0 {
                        if let Some(mut position) =
                            world.get_mut::<crate::components::Position>(*entity)
                        {
                            position.0 = new_pos;
                            info!("Recovered stuck creature {:?} by teleporting", entity);
                            return Ok(());
                        }
                    }
                }
            }

            Err("Failed to find valid recovery position".to_string())
        } else {
            Err("Wrong error type".to_string())
        }
    }
}

/// Recovery for invalid positions
pub struct InvalidPositionRecovery;

impl RecoveryStrategy for InvalidPositionRecovery {
    fn can_recover(&self, error: &SimulationError) -> bool {
        matches!(error, SimulationError::InvalidPosition { .. })
    }

    fn recover(&self, world: &mut World, error: &SimulationError) -> Result<(), String> {
        if let SimulationError::InvalidPosition { entity, position } = error {
            // Clamp position to valid bounds
            let bounds = 1000.0;
            let clamped = Vec2::new(
                position.x.clamp(-bounds, bounds),
                position.y.clamp(-bounds, bounds),
            );

            if let Some(mut pos) = world.get_mut::<crate::components::Position>(*entity) {
                pos.0 = clamped;
                info!(
                    "Recovered invalid position for {:?}: {:?} -> {:?}",
                    entity, position, clamped
                );
                Ok(())
            } else {
                Err("Failed to access position component".to_string())
            }
        } else {
            Err("Wrong error type".to_string())
        }
    }
}

/// Recovery for negative resources
pub struct NegativeResourceRecovery;

impl RecoveryStrategy for NegativeResourceRecovery {
    fn can_recover(&self, error: &SimulationError) -> bool {
        matches!(error, SimulationError::ResourceNegative { .. })
    }

    fn recover(&self, world: &mut World, error: &SimulationError) -> Result<(), String> {
        if let SimulationError::ResourceNegative {
            entity, resource, ..
        } = error
        {
            // Reset resource to minimum valid value
            match resource.as_str() {
                "health" => {
                    if let Some(mut health) = world.get_mut::<crate::components::Health>(*entity) {
                        health.current = 1.0; // Minimum alive health
                        info!("Reset health to minimum for {:?}", entity);
                        Ok(())
                    } else {
                        Err("Failed to access health component".to_string())
                    }
                },
                "hunger" | "thirst" | "energy" => {
                    if let Some(mut needs) = world.get_mut::<crate::components::Needs>(*entity) {
                        match resource.as_str() {
                            "hunger" => needs.hunger = 0.0,
                            "thirst" => needs.thirst = 0.0,
                            "energy" => needs.energy = 0.0,
                            _ => unreachable!(),
                        }
                        info!("Reset {} to 0 for {:?}", resource, entity);
                        Ok(())
                    } else {
                        Err("Failed to access needs component".to_string())
                    }
                },
                _ => Err(format!("Unknown resource type: {}", resource)),
            }
        } else {
            Err("Wrong error type".to_string())
        }
    }
}

/// Circuit breaker state for error handling
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    /// Normal operation
    Closed,
    /// Errors detected, still attempting recovery
    Open { 
        since: Instant,
        error_count: usize,
    },
    /// Too many errors, recovery disabled temporarily
    HalfOpen {
        since: Instant,
        test_until: Instant,
    },
}

/// Tracks persistent error patterns
#[derive(Default)]
pub struct ErrorPattern {
    /// Count of similar errors per entity
    entity_errors: HashMap<Entity, Vec<(SimulationError, Instant)>>,
    /// Count of errors per system
    system_errors: HashMap<&'static str, Vec<Instant>>,
}

impl ErrorPattern {
    fn record_error(&mut self, error: &SimulationError) {
        let now = Instant::now();
        
        // Track by entity
        match error {
            SimulationError::CreatureStuck { entity, .. }
            | SimulationError::InvalidPosition { entity, .. }
            | SimulationError::ResourceNegative { entity, .. }
            | SimulationError::PathfindingFailed { entity, .. }
            | SimulationError::ComponentMissing { entity, .. } => {
                self.entity_errors
                    .entry(*entity)
                    .or_default()
                    .push((error.clone(), now));
            }
            SimulationError::SystemPanic { system, .. } => {
                self.system_errors
                    .entry(system)
                    .or_default()
                    .push(now);
            }
        }
        
        // Clean old entries (keep last 5 minutes)
        let cutoff = now - Duration::from_secs(300);
        self.entity_errors.retain(|_, errors| {
            errors.retain(|(_, time)| *time > cutoff);
            !errors.is_empty()
        });
        self.system_errors.retain(|_, times| {
            times.retain(|time| *time > cutoff);
            !times.is_empty()
        });
    }
    
    fn is_persistent_error(&self, entity: Entity, threshold: usize) -> bool {
        self.entity_errors
            .get(&entity)
            .map(|errors| errors.len() >= threshold)
            .unwrap_or(false)
    }
    
    fn get_entity_error_rate(&self, entity: Entity, window: Duration) -> f32 {
        let now = Instant::now();
        let cutoff = now - window;
        
        if let Some(errors) = self.entity_errors.get(&entity) {
            let recent_count = errors.iter()
                .filter(|(_, time)| *time > cutoff)
                .count();
            recent_count as f32 / window.as_secs_f32()
        } else {
            0.0
        }
    }
}

/// Main error boundary resource
#[derive(Resource)]
pub struct ErrorBoundary {
    strategies: HashMap<String, Box<dyn RecoveryStrategy>>,
    error_log: Vec<(SimulationError, std::time::Instant)>,
    max_errors: usize,
    error_window: std::time::Duration,
    /// Circuit breaker state
    circuit_state: CircuitState,
    /// Circuit breaker configuration
    circuit_failure_threshold: usize,
    circuit_reset_timeout: Duration,
    circuit_half_open_duration: Duration,
    /// Error pattern detection
    error_patterns: ErrorPattern,
    /// Entities marked as problematic
    quarantined_entities: HashMap<Entity, Instant>,
}

impl Default for ErrorBoundary {
    fn default() -> Self {
        let mut strategies: HashMap<String, Box<dyn RecoveryStrategy>> = HashMap::new();
        strategies.insert(
            "creature_stuck".to_string(),
            Box::new(CreatureStuckRecovery),
        );
        strategies.insert(
            "invalid_position".to_string(),
            Box::new(InvalidPositionRecovery),
        );
        strategies.insert(
            "negative_resource".to_string(),
            Box::new(NegativeResourceRecovery),
        );

        Self {
            strategies,
            error_log: Vec::new(),
            max_errors: 100,
            error_window: std::time::Duration::from_secs(60),
            circuit_state: CircuitState::Closed,
            circuit_failure_threshold: 10,
            circuit_reset_timeout: Duration::from_secs(30),
            circuit_half_open_duration: Duration::from_secs(10),
            error_patterns: ErrorPattern::default(),
            quarantined_entities: HashMap::new(),
        }
    }
}

impl ErrorBoundary {
    pub fn handle_error(
        &mut self,
        error: SimulationError,
        world: &mut World,
    ) -> Result<(), String> {
        let now = Instant::now();
        
        // Update circuit breaker state
        self.update_circuit_state(now);
        
        // Check if circuit is open (but allow recording the error first)
        let should_block = match &self.circuit_state {
            CircuitState::Open { error_count, .. } => *error_count >= self.circuit_failure_threshold,
            _ => false,
        };
        
        // Record error pattern
        self.error_patterns.record_error(&error);
        
        // Check for persistent errors on entities
        if let Some(entity) = self.get_error_entity(&error) {
            if self.error_patterns.is_persistent_error(entity, 5) {
                self.quarantine_entity(entity);
                warn!("Entity {:?} quarantined due to persistent errors", entity);
            }
        }
        
        // Log the error
        self.error_log.push((error.clone(), now));

        // Clean up old errors
        let cutoff = now - self.error_window;
        self.error_log.retain(|(_, time)| *time > cutoff);

        // Check if we're getting too many errors
        if self.error_log.len() > self.max_errors {
            self.trip_circuit_breaker(now);
            return Err(format!(
                "Too many errors: {} in {:?}",
                self.error_log.len(),
                self.error_window
            ));
        }
        
        // Block recovery if circuit is open
        if should_block {
            warn!("Circuit breaker open, skipping recovery for: {}", error);
            return Err("Circuit breaker is open".to_string());
        }

        // Try to recover if entity is not quarantined
        if let Some(entity) = self.get_error_entity(&error) {
            if self.is_entity_quarantined(entity) {
                return Err(format!("Entity {:?} is quarantined", entity));
            }
        }
        
        // Attempt recovery
        let mut recovery_attempted = false;
        let mut recovery_succeeded = false;
        let mut recovery_errors = Vec::new();
        
        for strategy in self.strategies.values() {
            if strategy.can_recover(&error) {
                recovery_attempted = true;
                match strategy.recover(world, &error) {
                    Ok(()) => {
                        debug!("Successfully recovered from: {}", error);
                        recovery_succeeded = true;
                        break;
                    },
                    Err(e) => {
                        warn!("Recovery failed for {}: {}", error, e);
                        recovery_errors.push(e);
                    },
                }
            }
        }
        
        if recovery_succeeded {
            self.record_successful_recovery();
            Ok(())
        } else if !recovery_attempted {
            // Don't count as a failure if no recovery was attempted
            Err(format!("No recovery strategy for: {}", error))
        } else {
            // Only record failed recovery if we actually tried
            self.record_failed_recovery(now);
            Err("All recovery strategies failed".to_string())
        }
    }

    pub fn recent_error_count(&self) -> usize {
        self.error_log.len()
    }

    pub fn add_strategy(&mut self, name: String, strategy: Box<dyn RecoveryStrategy>) {
        self.strategies.insert(name, strategy);
    }
    
    fn update_circuit_state(&mut self, now: Instant) {
        match &self.circuit_state {
            CircuitState::Open { since, .. } => {
                if now.duration_since(*since) > self.circuit_reset_timeout {
                    self.circuit_state = CircuitState::HalfOpen {
                        since: now,
                        test_until: now + self.circuit_half_open_duration,
                    };
                    info!("Circuit breaker entering half-open state");
                }
            }
            CircuitState::HalfOpen { test_until, .. } => {
                if now > *test_until {
                    self.circuit_state = CircuitState::Closed;
                    info!("Circuit breaker closed after successful test period");
                }
            }
            _ => {}
        }
    }
    
    fn trip_circuit_breaker(&mut self, now: Instant) {
        let error_count = self.error_log.len();
        self.circuit_state = CircuitState::Open { since: now, error_count };
        warn!("Circuit breaker tripped with {} errors", error_count);
    }
    
    fn record_successful_recovery(&mut self) {
        if let CircuitState::HalfOpen { .. } = self.circuit_state {
            // Don't immediately close, wait for test period
        }
    }
    
    fn record_failed_recovery(&mut self, now: Instant) {
        match &mut self.circuit_state {
            CircuitState::Closed => {
                self.circuit_state = CircuitState::Open { since: now, error_count: 1 };
            }
            CircuitState::Open { error_count, .. } => {
                *error_count += 1;
            }
            CircuitState::HalfOpen { .. } => {
                // Failed during test, reopen circuit
                self.trip_circuit_breaker(now);
            }
        }
    }
    
    fn get_error_entity(&self, error: &SimulationError) -> Option<Entity> {
        match error {
            SimulationError::CreatureStuck { entity, .. }
            | SimulationError::InvalidPosition { entity, .. }
            | SimulationError::ResourceNegative { entity, .. }
            | SimulationError::PathfindingFailed { entity, .. }
            | SimulationError::ComponentMissing { entity, .. } => Some(*entity),
            _ => None,
        }
    }
    
    fn quarantine_entity(&mut self, entity: Entity) {
        self.quarantined_entities.insert(entity, Instant::now());
    }
    
    fn is_entity_quarantined(&self, entity: Entity) -> bool {
        if let Some(quarantine_time) = self.quarantined_entities.get(&entity) {
            // Quarantine for 2 minutes
            Instant::now().duration_since(*quarantine_time) < Duration::from_secs(120)
        } else {
            false
        }
    }
    
    pub fn clean_quarantine(&mut self) {
        let now = Instant::now();
        self.quarantined_entities.retain(|_, time| {
            now.duration_since(*time) < Duration::from_secs(120)
        });
    }
    
    pub fn get_circuit_state(&self) -> &CircuitState {
        &self.circuit_state
    }
    
    pub fn get_error_rate(&self, entity: Entity, window: Duration) -> f32 {
        self.error_patterns.get_entity_error_rate(entity, window)
    }
    
    pub fn get_quarantined_entities(&self) -> Vec<Entity> {
        self.quarantined_entities.keys().copied().collect()
    }
}

/// Plugin for error boundary system
pub struct ErrorBoundaryPlugin;

impl Plugin for ErrorBoundaryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ErrorBoundary>();
    }
}

/// Helper macro for safe component access
#[macro_export]
macro_rules! safe_component_access {
    ($world:expr, $entity:expr, $component:ty, $error_boundary:expr) => {
        match $world.get::<$component>($entity) {
            Some(component) => Ok(component),
            None => {
                let error = $crate::core::error_boundary::SimulationError::ComponentMissing {
                    entity: $entity,
                    component: std::any::type_name::<$component>(),
                };
                $error_boundary.handle_error(error, $world)?;
                Err("Component missing after recovery")
            },
        }
    };
}

/// Helper for validating positions
pub fn validate_position(
    entity: Entity,
    position: Vec2,
    error_boundary: &mut ErrorBoundary,
    world: &mut World,
) -> Result<Vec2, String> {
    let bounds = 1000.0;
    if position.x.abs() > bounds
        || position.y.abs() > bounds
        || position.x.is_nan()
        || position.y.is_nan()
    {
        let error = SimulationError::InvalidPosition { entity, position };
        error_boundary.handle_error(error, world)?;
        // Return the corrected position
        if let Some(pos) = world.get::<crate::components::Position>(entity) {
            Ok(pos.0)
        } else {
            Err("Failed to get corrected position".to_string())
        }
    } else {
        Ok(position)
    }
}

/// Helper for validating resources
pub fn validate_resource(
    entity: Entity,
    resource_name: &str,
    value: f32,
    error_boundary: &mut ErrorBoundary,
    world: &mut World,
) -> Result<f32, String> {
    if value < 0.0 || value.is_nan() {
        let error = SimulationError::ResourceNegative {
            entity,
            resource: resource_name.to_string(),
            value,
        };
        error_boundary.handle_error(error, world)?;
        Ok(0.0) // Return safe minimum
    } else {
        Ok(value)
    }
}

#[cfg(test)]
mod tests;
