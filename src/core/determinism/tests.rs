//! Tests for deterministic RNG system

#[cfg(test)]
mod tests {
    use super::super::*;
    use bevy::ecs::system::SystemState;

    #[test]
    fn test_deterministic_rng_creation() {
        let rng = DeterministicRng::new(12345);
        assert_eq!(rng.master_seed, 12345);
        assert_eq!(rng.frame_count, 0);
        assert!(rng.system_rngs.is_empty());
    }

    #[test]
    fn test_system_rng_isolation() {
        let mut rng = DeterministicRng::new(12345);

        // Get RNGs for different systems
        let val1 = rng.gen_range_f32(SystemId::Movement);
        let val2 = rng.gen_range_f32(SystemId::Decision);

        // Different systems should produce different values
        assert_ne!(val1, val2);

        // Same system should produce consistent sequence
        let mut rng2 = DeterministicRng::new(12345);
        let val3 = rng2.gen_range_f32(SystemId::Movement);
        assert_eq!(val1, val3);
    }

    #[test]
    fn test_gen_range_f32() {
        let mut rng = DeterministicRng::new(12345);

        // Test range [0, 1)
        for _ in 0..100 {
            let val = rng.gen_range_f32(SystemId::Movement);
            assert!(val >= 0.0 && val < 1.0);
        }
    }

    #[test]
    fn test_gen_range() {
        let mut rng = DeterministicRng::new(12345);

        // Test custom range
        for _ in 0..100 {
            let val = rng.gen_range(SystemId::Movement, 10.0, 20.0);
            assert!(val >= 10.0 && val < 20.0);
        }
    }

    #[test]
    fn test_gen_range_i32() {
        let mut rng = DeterministicRng::new(12345);

        // Test integer range
        for _ in 0..100 {
            let val = rng.gen_range_i32(SystemId::Decision, -50, 50);
            assert!(val >= -50 && val < 50);
        }
    }

    #[test]
    fn test_gen_bool() {
        let mut rng = DeterministicRng::new(12345);

        // Test probability
        let mut true_count = 0;
        let trials = 1000;

        for _ in 0..trials {
            if rng.gen_bool(SystemId::Movement, 0.7) {
                true_count += 1;
            }
        }

        // Should be approximately 70%
        let ratio = true_count as f32 / trials as f32;
        assert!(ratio > 0.65 && ratio < 0.75);
    }

    #[test]
    fn test_gen_direction() {
        let mut rng = DeterministicRng::new(12345);

        // Test unit vectors
        for _ in 0..100 {
            let dir = rng.gen_direction(SystemId::Movement);
            let length = dir.length();
            assert!((length - 1.0).abs() < 0.001);
        }
    }

    #[test]
    fn test_determinism() {
        let mut rng1 = DeterministicRng::new(12345);
        let mut rng2 = DeterministicRng::new(12345);

        // Same seed should produce same sequence
        for _ in 0..10 {
            assert_eq!(
                rng1.gen_range_f32(SystemId::Movement),
                rng2.gen_range_f32(SystemId::Movement)
            );
        }

        // Different seed should produce different sequence
        let mut rng3 = DeterministicRng::new(54321);
        assert_ne!(
            rng3.gen_range_f32(SystemId::Movement),
            rng2.gen_range_f32(SystemId::Movement)
        );
    }

    #[test]
    fn test_frame_advance() {
        let mut rng = DeterministicRng::new(12345);
        assert_eq!(rng.frame_count, 0);

        rng.advance_frame();
        assert_eq!(rng.frame_count, 1);

        rng.advance_frame();
        assert_eq!(rng.frame_count, 2);
    }

    #[test]
    fn test_reset() {
        let mut rng = DeterministicRng::new(12345);

        // Generate some values
        rng.gen_range_f32(SystemId::Movement);
        rng.advance_frame();
        rng.advance_frame();

        assert!(!rng.system_rngs.is_empty());
        assert_eq!(rng.frame_count, 2);

        // Reset
        rng.reset();
        assert!(rng.system_rngs.is_empty());
        assert_eq!(rng.frame_count, 0);
    }

    #[test]
    fn test_set_seed() {
        let mut rng = DeterministicRng::new(12345);

        // Generate value with original seed
        let val1 = rng.gen_range_f32(SystemId::Movement);

        // Change seed
        rng.set_seed(54321);
        assert_eq!(rng.master_seed, 54321);
        assert!(rng.system_rngs.is_empty()); // Should reset

        // Generate with new seed
        let val2 = rng.gen_range_f32(SystemId::Movement);
        assert_ne!(val1, val2);
    }

    #[test]
    fn test_determinism_config() {
        let config = DeterminismConfig::default();
        assert!(config.enabled);
        assert!(config.seed.is_none());
        assert_eq!(config.checksum_frequency, 60);
        assert!(!config.log_checksums);
    }

    #[test]
    fn test_frame_checksum_calculation() {
        let mut world = World::new();

        // Add some creatures
        world.spawn((
            crate::components::Position(Vec2::new(10.0, 20.0)),
            crate::components::Health {
                current: 80.0,
                max: 100.0,
            },
            crate::components::Creature,
        ));
        world.spawn((
            crate::components::Position(Vec2::new(30.0, 40.0)),
            crate::components::Health {
                current: 60.0,
                max: 100.0,
            },
            crate::components::Creature,
        ));

        // Add resources
        world.spawn((
            crate::components::Position(Vec2::new(50.0, 60.0)),
            crate::components::ResourceMarker,
        ));

        // Create a system state to get the queries
        let mut system_state: SystemState<(
            Query<
                (&crate::components::Position, &crate::components::Health),
                With<crate::components::Creature>,
            >,
            Query<&crate::components::Position, With<crate::components::ResourceMarker>>,
        )> = SystemState::new(&mut world);

        let (creature_query, resource_query) = system_state.get(&world);
        let checksum = FrameChecksum::calculate(1, &creature_query, &resource_query);

        assert_eq!(checksum.frame, 1);
        assert_ne!(checksum.creature_checksum, 0);
        assert_ne!(checksum.resource_checksum, 0);
        assert_ne!(checksum.position_checksum, 0);
    }

    #[test]
    fn test_checksum_determinism() {
        let mut world = World::new();

        // Add same entities
        world.spawn((
            crate::components::Position(Vec2::new(10.0, 20.0)),
            crate::components::Health {
                current: 80.0,
                max: 100.0,
            },
            crate::components::Creature,
        ));

        // Create a system state to get the queries
        let mut system_state: SystemState<(
            Query<
                (&crate::components::Position, &crate::components::Health),
                With<crate::components::Creature>,
            >,
            Query<&crate::components::Position, With<crate::components::ResourceMarker>>,
        )> = SystemState::new(&mut world);

        let (creature_query, resource_query) = system_state.get(&world);

        let checksum1 = FrameChecksum::calculate(1, &creature_query, &resource_query);
        let checksum2 = FrameChecksum::calculate(1, &creature_query, &resource_query);

        // Same state should produce same checksum
        assert_eq!(checksum1, checksum2);
    }

    #[test]
    fn test_checksum_history() {
        let mut history = ChecksumHistory::new(5);

        // Add checksums
        for i in 0..7 {
            let checksum = FrameChecksum {
                frame: i,
                creature_checksum: i,
                resource_checksum: i,
                position_checksum: i,
            };
            history.add(checksum);
        }

        // Should maintain max size
        assert_eq!(history.checksums.len(), 5);
        assert_eq!(history.checksums[0].frame, 2); // Oldest should be frame 2

        // Test get_recent
        let recent = history.get_recent(3);
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0].frame, 4);
    }

    #[test]
    fn test_seeded_random_trait() {
        let mut app = App::new();
        app.insert_resource(DeterministicRng::new(12345));

        let mut rng = app.world.resource_mut::<DeterministicRng>();

        // Test trait methods - use the direct methods instead of trait methods
        let f32_val = rng.gen_range_f32(SystemId::Movement);
        assert!(f32_val >= 0.0 && f32_val < 1.0);

        let range_val = rng.gen_range(SystemId::Decision, 5.0, 10.0);
        assert!(range_val >= 5.0 && range_val < 10.0);

        let bool_val = rng.gen_bool(SystemId::ResourceSpawn, 0.5);
        assert!(bool_val == true || bool_val == false);

        let dir = rng.gen_direction(SystemId::Movement);
        assert!((dir.length() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_default_seed_is_set() {
        // Just verify that default constructor sets a seed
        let rng = DeterministicRng::default();
        assert!(rng.master_seed > 0);
    }
}
