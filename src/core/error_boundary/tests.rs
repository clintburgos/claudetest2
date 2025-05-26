//! Tests for the error boundary system

#[cfg(test)]
mod tests {
    use crate::core::error_boundary::*;
    use bevy::prelude::{Vec2, World};

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

        let entity = world.spawn((crate::components::Position(Vec2::new(0.0, 0.0)),)).id();

        // Add many errors quickly
        for i in 0..6 {
            let error = SimulationError::InvalidPosition {
                entity,
                position: Vec2::new(i as f32, i as f32),
            };

            let result = boundary.handle_error(error, &mut world);

            if i < 5 {
                assert!(result.is_ok());
            } else {
                // Should fail on 6th error
                assert!(result.is_err());
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
}
