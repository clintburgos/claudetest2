//! Tests for the error boundary system

#[cfg(test)]
mod tests {
    use crate::core::error_boundary::*;
    use bevy::prelude::{Vec2, World};
    use std::time::Duration;

    #[test]
    fn test_creature_stuck_recovery() {
        let mut world = World::new();
        let recovery = CreatureStuckRecovery;

        // Create a stuck creature
        let entity = world.spawn((crate::components::Position(Vec2::new(100.0, 100.0)),)).id();

        let error = SimulationError::CreatureStuck {
            entity,
            duration: 15.0,
        };

        // Test can_recover
        assert!(recovery.can_recover(&error));

        // Test recovery - should teleport creature
        let result = recovery.recover(&mut world, &error);
        assert!(result.is_ok());

        // Check position changed
        let new_pos = world.get::<crate::components::Position>(entity).unwrap();
        assert_ne!(new_pos.0, Vec2::new(100.0, 100.0));
    }

    #[test]
    fn test_invalid_position_recovery() {
        let mut world = World::new();
        let recovery = InvalidPositionRecovery;

        // Create entity with invalid position
        let entity = world.spawn((crate::components::Position(Vec2::new(2000.0, -3000.0)),)).id();

        let error = SimulationError::InvalidPosition {
            entity,
            position: Vec2::new(2000.0, -3000.0),
        };

        assert!(recovery.can_recover(&error));

        // Test recovery - should clamp position
        let result = recovery.recover(&mut world, &error);
        assert!(result.is_ok());

        // Check position is clamped
        let new_pos = world.get::<crate::components::Position>(entity).unwrap();
        assert!(new_pos.0.x <= 1000.0);
        assert!(new_pos.0.y >= -1000.0);
    }

    #[test]
    fn test_negative_resource_recovery() {
        let mut world = World::new();
        let recovery = NegativeResourceRecovery;

        // Test health recovery
        let entity = world
            .spawn((crate::components::Health {
                current: -10.0,
                max: 100.0,
            },))
            .id();

        let error = SimulationError::ResourceNegative {
            entity,
            resource: "health".to_string(),
            value: -10.0,
        };

        assert!(recovery.can_recover(&error));
        let result = recovery.recover(&mut world, &error);
        assert!(result.is_ok());

        let health = world.get::<crate::components::Health>(entity).unwrap();
        assert_eq!(health.current, 1.0); // Reset to minimum alive health

        // Test needs recovery
        let entity2 = world
            .spawn((crate::components::Needs {
                hunger: -0.5,
                thirst: 0.5,
                energy: 0.5,
                social: 0.5,
            },))
            .id();

        let error2 = SimulationError::ResourceNegative {
            entity: entity2,
            resource: "hunger".to_string(),
            value: -0.5,
        };

        let result2 = recovery.recover(&mut world, &error2);
        assert!(result2.is_ok());

        let needs = world.get::<crate::components::Needs>(entity2).unwrap();
        assert_eq!(needs.hunger, 0.0);
    }

    #[test]
    fn test_error_boundary_handle_error() {
        let mut world = World::new();
        let mut boundary = ErrorBoundary::default();

        // Create test entity
        let entity = world.spawn((crate::components::Position(Vec2::new(5000.0, 5000.0)),)).id();

        let error = SimulationError::InvalidPosition {
            entity,
            position: Vec2::new(5000.0, 5000.0),
        };

        // Test error handling
        let result = boundary.handle_error(error.clone(), &mut world);
        assert!(result.is_ok());

        // Check error was logged
        assert_eq!(boundary.recent_error_count(), 1);

        // Test error window cleanup
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Add another error
        let error2 = SimulationError::InvalidPosition {
            entity,
            position: Vec2::new(-5000.0, -5000.0),
        };
        boundary.handle_error(error2, &mut world).ok();
        assert_eq!(boundary.recent_error_count(), 2);
    }

    #[test]
    fn test_error_boundary_max_errors() {
        let mut world = World::new();
        let mut boundary = ErrorBoundary::default();
        boundary.max_errors = 5; // Set low for testing

        // Use different entities to avoid quarantine
        let entities: Vec<_> = (0..6)
            .map(|_| world.spawn((crate::components::Position(Vec2::new(0.0, 0.0)),)).id())
            .collect();

        // Add many errors quickly - use different entities to avoid quarantine
        for (i, &entity) in entities.iter().enumerate() {
            let error = SimulationError::InvalidPosition {
                entity,
                position: Vec2::new((i as f32 + 1.0) * 2000.0, (i as f32 + 1.0) * 2000.0),
            };

            let result = boundary.handle_error(error, &mut world);

            if i < 5 {
                // First 5 errors should be handled (and recovered)
                assert!(result.is_ok(), "Error {} should succeed, got: {:?}", i, result);
            } else {
                // Should fail on 6th error due to max errors
                assert!(result.is_err(), "Error {} should fail", i);
                assert!(result.unwrap_err().contains("Too many errors"));
            }
        }
    }

    #[test]
    fn test_validate_position() {
        let mut world = World::new();
        let mut boundary = ErrorBoundary::default();

        let entity = world.spawn((crate::components::Position(Vec2::new(0.0, 0.0)),)).id();

        // Test valid position
        let result = validate_position(entity, Vec2::new(500.0, 500.0), &mut boundary, &mut world);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Vec2::new(500.0, 500.0));

        // Test out of bounds
        let result =
            validate_position(entity, Vec2::new(2000.0, 2000.0), &mut boundary, &mut world);
        assert!(result.is_ok());
        let corrected = result.unwrap();
        assert!(corrected.x <= 1000.0);
        assert!(corrected.y <= 1000.0);

        // Test NaN
        let result = validate_position(entity, Vec2::new(f32::NAN, 0.0), &mut boundary, &mut world);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_resource() {
        let mut world = World::new();
        let mut boundary = ErrorBoundary::default();

        let entity = world
            .spawn((crate::components::Health {
                current: 50.0,
                max: 100.0,
            },))
            .id();

        // Test valid resource
        let result = validate_resource(entity, "health", 50.0, &mut boundary, &mut world);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 50.0);

        // Test negative resource
        let result = validate_resource(entity, "health", -10.0, &mut boundary, &mut world);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0.0); // Should return safe minimum

        // Test NaN resource
        let result = validate_resource(entity, "health", f32::NAN, &mut boundary, &mut world);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0.0);
    }

    #[test]
    fn test_wrong_error_type_recovery() {
        let _world = World::new();
        let recovery = CreatureStuckRecovery;

        // Test with wrong error type
        let error = SimulationError::InvalidPosition {
            entity: Entity::from_raw(0),
            position: Vec2::ZERO,
        };

        assert!(!recovery.can_recover(&error));
    }

    #[test]
    fn test_custom_recovery_strategy() {
        struct TestRecovery;

        impl RecoveryStrategy for TestRecovery {
            fn can_recover(&self, error: &SimulationError) -> bool {
                matches!(error, SimulationError::SystemPanic { .. })
            }

            fn recover(&self, _world: &mut World, _error: &SimulationError) -> Result<(), String> {
                Ok(())
            }
        }

        let mut boundary = ErrorBoundary::default();
        boundary.add_strategy("test".to_string(), Box::new(TestRecovery));

        let mut world = World::new();
        let error = SimulationError::SystemPanic {
            system: "test",
            message: "test panic".to_string(),
        };

        let result = boundary.handle_error(error, &mut world);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_circuit_breaker() {
        let mut world = World::new();
        let mut boundary = ErrorBoundary::default();
        boundary.circuit_failure_threshold = 3; // Low threshold for testing
        boundary.max_errors = 20; // Increase to avoid hitting max errors first
        
        // Add a custom recovery strategy that always fails
        struct FailingRecovery;
        impl RecoveryStrategy for FailingRecovery {
            fn can_recover(&self, error: &SimulationError) -> bool {
                matches!(error, SimulationError::PathfindingFailed { .. })
            }
            fn recover(&self, _world: &mut World, _error: &SimulationError) -> Result<(), String> {
                Err("Always fails".to_string())
            }
        }
        boundary.add_strategy("failing".to_string(), Box::new(FailingRecovery));
        
        let entity = world.spawn((crate::components::Position(Vec2::new(0.0, 0.0)),)).id();
        
        // Circuit should start closed
        assert!(matches!(boundary.get_circuit_state(), CircuitState::Closed));
        
        // Create errors that will fail recovery
        for i in 0..4 {
            let error = SimulationError::PathfindingFailed {
                entity,
                target: Vec2::new(i as f32 * 100.0, 0.0),
            };
            boundary.handle_error(error, &mut world).ok();
        }
        
        // Circuit should now be open due to failed recoveries
        match boundary.get_circuit_state() {
            CircuitState::Open { error_count, .. } => {
                assert!(*error_count >= boundary.circuit_failure_threshold);
            }
            _ => panic!("Circuit should be open, current state: {:?}", boundary.get_circuit_state()),
        }
        
        // Further errors should be rejected while circuit is open
        let error = SimulationError::PathfindingFailed {
            entity,
            target: Vec2::new(0.0, 0.0),
        };
        let result = boundary.handle_error(error, &mut world);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Circuit breaker"));
    }
    
    #[test]
    fn test_entity_quarantine() {
        let mut world = World::new();
        let mut boundary = ErrorBoundary::default();
        
        let entity = world.spawn((crate::components::Position(Vec2::new(0.0, 0.0)),)).id();
        
        // Generate multiple errors for the same entity
        for i in 0..6 {
            let error = SimulationError::CreatureStuck {
                entity,
                duration: i as f32,
            };
            boundary.handle_error(error, &mut world).ok();
        }
        
        // Entity should be quarantined
        assert!(boundary.is_entity_quarantined(entity));
        
        // Further errors from quarantined entity should be rejected
        let error = SimulationError::CreatureStuck {
            entity,
            duration: 10.0,
        };
        let result = boundary.handle_error(error, &mut world);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("quarantined"));
    }
    
    #[test]
    fn test_error_pattern_detection() {
        let mut world = World::new();
        let mut boundary = ErrorBoundary::default();
        
        let entity = world.spawn((crate::components::Position(Vec2::new(0.0, 0.0)),)).id();
        
        // Generate errors over time
        for _ in 0..3 {
            let error = SimulationError::InvalidPosition {
                entity,
                position: Vec2::new(2000.0, 2000.0),
            };
            boundary.handle_error(error, &mut world).ok();
        }
        
        // Check error rate
        let rate = boundary.get_error_rate(entity, Duration::from_secs(60));
        assert!(rate > 0.0);
    }
    
    #[test]
    fn test_quarantine_cleanup() {
        let mut boundary = ErrorBoundary::default();
        
        // Manually quarantine an entity
        let entity = Entity::from_raw(42);
        boundary.quarantine_entity(entity);
        
        assert!(boundary.is_entity_quarantined(entity));
        assert_eq!(boundary.get_quarantined_entities().len(), 1);
        
        // Clean should not remove recent quarantines
        boundary.clean_quarantine();
        assert!(boundary.is_entity_quarantined(entity));
    }
}
