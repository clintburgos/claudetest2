use crate::Vec2;
use crate::core::Entity;
use std::collections::VecDeque;
use thiserror::Error;
use log::error;

#[derive(Error, Debug, Clone)]
pub enum SimulationError {
    #[error("Creature {entity:?} stuck at position {position:?} for {duration:.1}s")]
    CreatureStuck {
        entity: Entity,
        position: Vec2,
        duration: f32,
    },
    
    #[error("Invalid position {position:?} for entity {entity:?}")]
    InvalidPosition {
        entity: Entity,
        position: Vec2,
    },
    
    #[error("Resource {entity:?} has negative amount: {amount}")]
    ResourceNegative {
        entity: Entity,
        amount: f32,
    },
    
    #[error("Pathfinding failed for entity {entity:?} from {from:?} to {to:?}")]
    PathfindingFailed {
        entity: Entity,
        from: Vec2,
        to: Vec2,
    },
    
    #[error("Entity {entity:?} not found")]
    EntityNotFound {
        entity: Entity,
    },
    
    #[error("System invariant violated: {message}")]
    InvariantViolation {
        message: String,
    },
}

#[derive(Debug)]
pub struct ErrorEvent {
    pub error: SimulationError,
    pub timestamp: f64,
    pub recovered: bool,
}

pub struct ErrorBoundary {
    error_log: VecDeque<ErrorEvent>,
    max_log_size: usize,
    recovery_enabled: bool,
    panic_on_fatal: bool,
}

impl ErrorBoundary {
    pub fn new() -> Self {
        Self {
            error_log: VecDeque::new(),
            max_log_size: 1000,
            recovery_enabled: true,
            panic_on_fatal: false,
        }
    }
    
    pub fn handle_error(&mut self, error: SimulationError, game_time: f64) -> RecoveryResult {
        // Log the error
        error!("Simulation error at time {:.2}: {:?}", game_time, error);
        
        // Attempt recovery if enabled
        let recovered = if self.recovery_enabled {
            self.attempt_recovery(&error)
        } else {
            false
        };
        
        // Record in log
        self.record_error(ErrorEvent {
            error: error.clone(),
            timestamp: game_time,
            recovered,
        });
        
        if recovered {
            RecoveryResult::Recovered
        } else if self.is_fatal(&error) {
            if self.panic_on_fatal {
                panic!("Fatal simulation error: {:?}", error);
            }
            RecoveryResult::Fatal
        } else {
            RecoveryResult::Failed
        }
    }
    
    pub fn check_invariants(&mut self, world: &crate::core::World) -> Result<(), SimulationError> {
        // Check that all entities in spatial grid exist
        for entity in world.spatial_grid.iter_entities() {
            if !world.entities.is_alive(entity) {
                return Err(SimulationError::InvariantViolation {
                    message: format!("Dead entity {:?} still in spatial grid", entity),
                });
            }
        }
        
        // Check that no positions are NaN or infinite
        for (entity, position) in world.spatial_grid.iter_positions() {
            if !position.is_finite() {
                return Err(SimulationError::InvalidPosition { entity, position });
            }
        }
        
        Ok(())
    }
    
    fn attempt_recovery(&self, error: &SimulationError) -> bool {
        match error {
            SimulationError::CreatureStuck { duration, .. } => {
                // Can recover from short-term stuck situations
                *duration < 30.0
            }
            SimulationError::InvalidPosition { .. } => {
                // Can attempt to reset to valid position
                true
            }
            SimulationError::ResourceNegative { .. } => {
                // Can clamp to zero
                true
            }
            SimulationError::PathfindingFailed { .. } => {
                // Non-critical, can continue
                true
            }
            SimulationError::EntityNotFound { .. } => {
                // Usually recoverable by ignoring the operation
                true
            }
            SimulationError::InvariantViolation { .. } => {
                // Depends on the specific invariant
                false
            }
        }
    }
    
    fn is_fatal(&self, error: &SimulationError) -> bool {
        matches!(error, SimulationError::InvariantViolation { .. })
    }
    
    fn record_error(&mut self, event: ErrorEvent) {
        if self.error_log.len() >= self.max_log_size {
            self.error_log.pop_front();
        }
        self.error_log.push_back(event);
    }
    
    pub fn get_recent_errors(&self, count: usize) -> Vec<&ErrorEvent> {
        self.error_log.iter().rev().take(count).collect()
    }
    
    pub fn get_error_count(&self) -> usize {
        self.error_log.len()
    }
    
    pub fn get_recovery_rate(&self) -> f32 {
        if self.error_log.is_empty() {
            1.0
        } else {
            let recovered = self.error_log.iter().filter(|e| e.recovered).count();
            recovered as f32 / self.error_log.len() as f32
        }
    }
    
    pub fn clear_log(&mut self) {
        self.error_log.clear();
    }
    
    pub fn set_recovery_enabled(&mut self, enabled: bool) {
        self.recovery_enabled = enabled;
    }
    
    pub fn set_panic_on_fatal(&mut self, panic: bool) {
        self.panic_on_fatal = panic;
    }
}

impl Default for ErrorBoundary {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, PartialEq)]
pub enum RecoveryResult {
    Recovered,
    Failed,
    Fatal,
}

// Placeholder for spatial grid iterator methods
impl crate::core::SpatialGrid {
    pub fn iter_entities(&self) -> impl Iterator<Item = Entity> + '_ {
        self.entity_positions.keys().copied()
    }
    
    pub fn iter_positions(&self) -> impl Iterator<Item = (Entity, Vec2)> + '_ {
        self.entity_positions.iter().map(|(e, (_, pos))| (*e, *pos))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_boundary_recovery() {
        let mut boundary = ErrorBoundary::new();
        
        let error = SimulationError::CreatureStuck {
            entity: Entity::new(1),
            position: Vec2::new(0.0, 0.0),
            duration: 5.0,
        };
        
        let result = boundary.handle_error(error, 10.0);
        assert_eq!(result, RecoveryResult::Recovered);
        assert_eq!(boundary.get_error_count(), 1);
        assert_eq!(boundary.get_recovery_rate(), 1.0);
    }
    
    #[test]
    fn error_boundary_fatal() {
        let mut boundary = ErrorBoundary::new();
        
        let error = SimulationError::InvariantViolation {
            message: "Test violation".to_string(),
        };
        
        let result = boundary.handle_error(error, 10.0);
        assert_eq!(result, RecoveryResult::Fatal);
    }
    
    #[test]
    fn error_boundary_log_limit() {
        let mut boundary = ErrorBoundary::new();
        boundary.max_log_size = 5;
        
        for i in 0..10 {
            let error = SimulationError::EntityNotFound {
                entity: Entity::new(i),
            };
            boundary.handle_error(error, i as f64);
        }
        
        assert_eq!(boundary.get_error_count(), 5);
    }
}