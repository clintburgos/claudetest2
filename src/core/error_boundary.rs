//! Error boundary system for graceful error recovery

use bevy::prelude::*;
use std::collections::HashMap;
use std::fmt;

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

/// Main error boundary resource
#[derive(Resource)]
pub struct ErrorBoundary {
    strategies: HashMap<String, Box<dyn RecoveryStrategy>>,
    error_log: Vec<(SimulationError, std::time::Instant)>,
    max_errors: usize,
    error_window: std::time::Duration,
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
        }
    }
}

impl ErrorBoundary {
    pub fn handle_error(
        &mut self,
        error: SimulationError,
        world: &mut World,
    ) -> Result<(), String> {
        // Log the error
        self.error_log.push((error.clone(), std::time::Instant::now()));

        // Clean up old errors
        let cutoff = std::time::Instant::now() - self.error_window;
        self.error_log.retain(|(_, time)| *time > cutoff);

        // Check if we're getting too many errors
        if self.error_log.len() > self.max_errors {
            return Err(format!(
                "Too many errors: {} in {:?}",
                self.error_log.len(),
                self.error_window
            ));
        }

        // Try to recover
        for strategy in self.strategies.values() {
            if strategy.can_recover(&error) {
                match strategy.recover(world, &error) {
                    Ok(()) => {
                        debug!("Successfully recovered from: {}", error);
                        return Ok(());
                    },
                    Err(e) => {
                        warn!("Recovery failed for {}: {}", error, e);
                    },
                }
            }
        }

        Err(format!("No recovery strategy for: {}", error))
    }

    pub fn recent_error_count(&self) -> usize {
        self.error_log.len()
    }

    pub fn add_strategy(&mut self, name: String, strategy: Box<dyn RecoveryStrategy>) {
        self.strategies.insert(name, strategy);
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
