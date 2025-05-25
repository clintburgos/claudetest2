//! Error handling and recovery system for simulation robustness.
//!
//! This module provides comprehensive error handling with recovery
//! strategies to keep the simulation running even when issues occur.
//! Rather than panicking on errors, the system attempts recovery
//! and logs issues for debugging.
//!
//! # Philosophy
//! - Errors are expected in complex simulations
//! - Recovery is preferable to crashes
//! - All errors are logged for debugging
//! - Fatal errors can still halt execution if needed

use crate::config::error::*;
use crate::core::Entity;
use crate::Vec2;
use bevy::log::error;
use std::collections::VecDeque;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum SimulationError {
    #[error("Creature {entity:?} stuck at position {position:?} for {duration:.1}s")]
    CreatureStuck {
        entity: Entity,
        position: Vec2,
        duration: f32,
    },

    #[error("Invalid position {position:?} for entity {entity:?}")]
    InvalidPosition { entity: Entity, position: Vec2 },

    #[error("Resource {entity:?} has negative amount: {amount}")]
    ResourceNegative { entity: Entity, amount: f32 },

    #[error("Pathfinding failed for entity {entity:?} from {from:?} to {to:?}")]
    PathfindingFailed {
        entity: Entity,
        from: Vec2,
        to: Vec2,
    },

    #[error("Entity {entity:?} not found")]
    EntityNotFound { entity: Entity },

    #[error("System invariant violated: {message}")]
    InvariantViolation { message: String },
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
            max_log_size: MAX_LOG_SIZE,
            recovery_enabled: true,
            panic_on_fatal: false,
        }
    }

    pub fn handle_error(&mut self, error: SimulationError, game_time: f64) -> RecoveryResult {
        // Log the error
        error!("Simulation error at time {:.2}: {:?}", game_time, error);

        // Attempt recovery if enabled
        let recovered = if self.recovery_enabled { self.attempt_recovery(&error) } else { false };

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
            },
            SimulationError::InvalidPosition { .. } => {
                // Can attempt to reset to valid position
                true
            },
            SimulationError::ResourceNegative { .. } => {
                // Can clamp to zero
                true
            },
            SimulationError::PathfindingFailed { .. } => {
                // Non-critical, can continue
                true
            },
            SimulationError::EntityNotFound { .. } => {
                // Usually recoverable by ignoring the operation
                true
            },
            SimulationError::InvariantViolation { .. } => {
                // Depends on the specific invariant
                false
            },
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::World;

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

    #[test]
    fn error_boundary_get_recent_errors() {
        let mut boundary = ErrorBoundary::new();

        for i in 0..5 {
            let error = SimulationError::EntityNotFound {
                entity: Entity::new(i),
            };
            boundary.handle_error(error, i as f64);
        }

        let recent = boundary.get_recent_errors(3);
        assert_eq!(recent.len(), 3);

        // Most recent should be first
        if let SimulationError::EntityNotFound { entity } = &recent[0].error {
            assert_eq!(entity.id(), 4);
        }
    }

    #[test]
    fn error_boundary_clear_log() {
        let mut boundary = ErrorBoundary::new();

        let error = SimulationError::EntityNotFound {
            entity: Entity::new(1),
        };
        boundary.handle_error(error, 0.0);

        assert_eq!(boundary.get_error_count(), 1);
        boundary.clear_log();
        assert_eq!(boundary.get_error_count(), 0);
    }

    #[test]
    fn error_boundary_recovery_settings() {
        let mut boundary = ErrorBoundary::new();

        // Disable recovery
        boundary.set_recovery_enabled(false);
        let error = SimulationError::CreatureStuck {
            entity: Entity::new(1),
            position: Vec2::new(0.0, 0.0),
            duration: 5.0, // Should normally recover
        };

        let result = boundary.handle_error(error, 0.0);
        assert_eq!(result, RecoveryResult::Failed);

        // Check panic setting (we won't test actual panic)
        boundary.set_panic_on_fatal(true);
        assert!(boundary.panic_on_fatal);
    }

    #[test]
    fn error_boundary_default() {
        let boundary = ErrorBoundary::default();
        assert_eq!(boundary.get_error_count(), 0);
        assert!(boundary.recovery_enabled);
        assert!(!boundary.panic_on_fatal);
    }

    #[test]
    fn error_boundary_check_invariants() {
        let mut boundary = ErrorBoundary::new();
        let mut world = World::new();

        // Valid state should pass
        assert!(boundary.check_invariants(&world).is_ok());

        // Add entity to spatial grid
        let entity = world.entities.create();
        world.spatial_grid.insert(entity, Vec2::new(0.0, 0.0));

        // Still valid
        assert!(boundary.check_invariants(&world).is_ok());

        // Destroy entity but don't remove from spatial grid
        world.entities.destroy(entity);

        // Should detect invariant violation
        let result = boundary.check_invariants(&world);
        assert!(result.is_err());
        if let Err(SimulationError::InvariantViolation { message }) = result {
            assert!(message.contains("Dead entity"));
        }
    }

    #[test]
    fn spatial_grid_iterators() {
        let mut grid = crate::core::SpatialGrid::new(10.0);
        let e1 = Entity::new(1);
        let e2 = Entity::new(2);
        let pos1 = Vec2::new(5.0, 5.0);
        let pos2 = Vec2::new(15.0, 15.0);

        grid.insert(e1, pos1);
        grid.insert(e2, pos2);

        // Test iter_entities
        let entities: Vec<Entity> = grid.iter_entities().collect();
        assert_eq!(entities.len(), 2);
        assert!(entities.contains(&e1));
        assert!(entities.contains(&e2));

        // Test iter_positions
        let positions: Vec<(Entity, Vec2)> = grid.iter_positions().collect();
        assert_eq!(positions.len(), 2);
        assert!(positions.contains(&(e1, pos1)));
        assert!(positions.contains(&(e2, pos2)));
    }

    #[test]
    fn error_types_recovery() {
        let mut boundary = ErrorBoundary::new();

        // Test all error types
        let errors = vec![
            (
                SimulationError::CreatureStuck {
                    entity: Entity::new(1),
                    position: Vec2::ZERO,
                    duration: 5.0,
                },
                true,
            ), // Should recover
            (
                SimulationError::InvalidPosition {
                    entity: Entity::new(2),
                    position: Vec2::new(f32::NAN, 0.0),
                },
                true,
            ), // Should recover
            (
                SimulationError::ResourceNegative {
                    entity: Entity::new(3),
                    amount: -10.0,
                },
                true,
            ), // Should recover
            (
                SimulationError::PathfindingFailed {
                    entity: Entity::new(4),
                    from: Vec2::ZERO,
                    to: Vec2::new(100.0, 100.0),
                },
                true,
            ), // Should recover
            (
                SimulationError::EntityNotFound {
                    entity: Entity::new(5),
                },
                true,
            ), // Should recover
            (
                SimulationError::InvariantViolation {
                    message: "Test violation".to_string(),
                },
                false,
            ), // Should NOT recover
        ];

        for (error, should_recover) in errors {
            let result = boundary.handle_error(error.clone(), 0.0);
            if should_recover {
                assert_eq!(result, RecoveryResult::Recovered);
            } else {
                assert_eq!(result, RecoveryResult::Fatal);
            }
        }
    }

    #[test]
    fn error_boundary_panic_on_fatal() {
        let mut boundary = ErrorBoundary::new();
        boundary.set_panic_on_fatal(true);

        // Non-fatal error should not panic
        let non_fatal = SimulationError::EntityNotFound {
            entity: Entity::new(1),
        };
        let result = boundary.handle_error(non_fatal, 0.0);
        assert_eq!(result, RecoveryResult::Recovered);
    }

    #[test]
    #[should_panic(expected = "Fatal simulation error")]
    fn error_boundary_panic_on_fatal_enabled() {
        let mut boundary = ErrorBoundary::new();
        boundary.set_panic_on_fatal(true);

        // Fatal error should panic
        let fatal = SimulationError::InvariantViolation {
            message: "Test".to_string(),
        };
        let _ = boundary.handle_error(fatal, 0.0);
    }

    #[test]
    fn error_boundary_recovery_disabled() {
        let mut boundary = ErrorBoundary::new();
        boundary.set_recovery_enabled(false);

        // Even recoverable errors should fail when recovery is disabled
        let error = SimulationError::EntityNotFound {
            entity: Entity::new(1),
        };
        let result = boundary.handle_error(error, 0.0);
        assert_eq!(result, RecoveryResult::Failed);
    }
}
