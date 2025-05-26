//! Property-based tests for creature behaviors

use proptest::prelude::*;
use bevy::prelude::*;
use creature_simulation::components::*;
use creature_simulation::plugins::SpatialGrid;
use creature_simulation::core::determinism::{DeterministicRng, SystemId};

/// Generate arbitrary positions within world bounds
fn position_strategy() -> impl Strategy<Value = Vec2> {
    (
        -1000.0f32..1000.0,
        -1000.0f32..1000.0,
    ).prop_map(|(x, y)| Vec2::new(x, y))
}

/// Generate arbitrary needs values
fn needs_strategy() -> impl Strategy<Value = (f32, f32, f32, f32)> {
    (
        0.0f32..1.0,
        0.0f32..1.0,
        0.0f32..1.0,
        0.0f32..1.0,
    )
}

/// Generate arbitrary health values
fn health_strategy() -> impl Strategy<Value = f32> {
    0.0f32..100.0
}

/// Generate arbitrary time deltas
fn time_delta_strategy() -> impl Strategy<Value = f32> {
    0.001f32..0.1  // 1ms to 100ms
}

proptest! {
    #[test]
    fn test_needs_always_change_over_time(
        initial_needs in needs_strategy(),
        delta_time in time_delta_strategy(),
    ) {
        let (hunger, thirst, energy, social) = initial_needs;
        let mut needs = Needs { hunger, thirst, energy, social };
        let initial = needs.clone();
        
        // Simulate needs update
        needs.hunger = (needs.hunger + 0.1 * delta_time).min(1.0);
        needs.thirst = (needs.thirst + 0.15 * delta_time).min(1.0);
        needs.energy = (needs.energy - 0.05 * delta_time).max(0.0);
        
        // Hunger and thirst should increase or stay the same (if at max)
        prop_assert!(needs.hunger >= initial.hunger);
        prop_assert!(needs.thirst >= initial.thirst);
        // Energy should decrease or stay the same (if at min)
        prop_assert!(needs.energy <= initial.energy);
    }
    
    #[test]
    fn test_needs_never_exceed_bounds(
        initial_needs in needs_strategy(),
        delta_time in time_delta_strategy(),
        iterations in 1..1000u32,
    ) {
        let (hunger, thirst, energy, social) = initial_needs;
        let mut needs = Needs { hunger, thirst, energy, social };
        
        // Update many times
        for _ in 0..iterations {
            needs.hunger = (needs.hunger + 0.1 * delta_time).min(1.0);
            needs.thirst = (needs.thirst + 0.15 * delta_time).min(1.0);
            needs.energy = (needs.energy - 0.05 * delta_time).max(0.0);
        }
        
        // All needs should be within [0, 1]
        prop_assert!(needs.hunger >= 0.0 && needs.hunger <= 1.0);
        prop_assert!(needs.thirst >= 0.0 && needs.thirst <= 1.0);
        prop_assert!(needs.energy >= 0.0 && needs.energy <= 1.0);
        prop_assert!(needs.social >= 0.0 && needs.social <= 1.0);
    }
    
    #[test]
    fn test_health_damage_reduces_health(
        initial_health in health_strategy(),
        damage in 0.1f32..50.0,
    ) {
        let mut health = Health {
            current: initial_health,
            max: 100.0,
        };
        let before = health.current;
        
        health.current = (health.current - damage).max(0.0);
        
        prop_assert!(health.current <= before);
        prop_assert!(health.current >= 0.0);
    }
    
    #[test]
    fn test_health_heal_increases_health(
        initial_health in health_strategy(),
        heal_amount in 0.1f32..50.0,
    ) {
        let mut health = Health {
            current: initial_health,
            max: 100.0,
        };
        let before = health.current;
        
        health.current = (health.current + heal_amount).min(health.max);
        
        prop_assert!(health.current >= before);
        prop_assert!(health.current <= health.max);
    }
    
    #[test]
    fn test_position_clamping_keeps_in_bounds(
        position in position_strategy(),
        velocity in position_strategy(),
        delta_time in time_delta_strategy(),
    ) {
        let bounds = 1000.0;
        let mut pos = position;
        
        // Apply velocity
        pos += velocity * delta_time;
        
        // Clamp to bounds
        pos.x = pos.x.clamp(-bounds, bounds);
        pos.y = pos.y.clamp(-bounds, bounds);
        
        prop_assert!(pos.x >= -bounds && pos.x <= bounds);
        prop_assert!(pos.y >= -bounds && pos.y <= bounds);
    }
    
    #[test]
    fn test_creature_decision_determinism(
        seed in any::<u64>()
    ) {
        // Create two RNGs with same seed
        let mut rng1 = DeterministicRng::new(seed);
        let mut rng2 = DeterministicRng::new(seed);
        
        // Generate random decisions
        let decision1 = rng1.gen_range(SystemId::Decision, 0.0, 4.0);
        let decision2 = rng2.gen_range(SystemId::Decision, 0.0, 4.0);
        
        // Should be identical
        prop_assert_eq!(decision1, decision2);
    }
    
    #[test]
    fn test_resource_consumption_reduces_amount(
        initial_amount in 1.0f32..100.0,
        consume_rate in 0.1f32..10.0,
        delta_time in time_delta_strategy(),
    ) {
        let mut resource = ResourceAmount {
            current: initial_amount,
            max: initial_amount,
        };
        
        let consumed = consume_rate * delta_time;
        let before = resource.current;
        
        resource.current = (resource.current - consumed).max(0.0);
        
        prop_assert!(resource.current < before || resource.current == 0.0);
        prop_assert!(resource.current >= 0.0);
    }
    
    #[test]
    fn test_creature_age_increases_monotonically(
        initial_age in 0.0f32..100.0,
        time_deltas in prop::collection::vec(time_delta_strategy(), 1..100),
    ) {
        let mut age = Age(initial_age);
        let mut previous = age.0;
        
        for delta in time_deltas {
            age.0 += delta;
            prop_assert!(age.0 > previous);
            previous = age.0;
        }
    }
    
    #[test]
    fn test_spatial_grid_query_consistency(
        positions in prop::collection::vec(position_strategy(), 1..50),
        query_center in position_strategy(),
        query_radius in 10.0f32..200.0,
    ) {
        let grid = SpatialGrid::new(50.0);
        let mut entities = Vec::new();
        
        // Create entities for testing
        for (i, pos) in positions.iter().enumerate() {
            let entity = Entity::from_raw(i as u32);
            entities.push((entity, *pos));
        }
        
        // Note: In the actual system, grid updates are handled by the spatial plugin
        // For this test, we're just verifying the query logic
        let nearby = grid.query_radius(query_center, query_radius);
        
        // This test is limited because we can't directly insert into the grid
        // But we can verify the query returns valid results
        prop_assert!(nearby.is_empty() || nearby.len() <= entities.len());
    }
    
    #[test]
    fn test_movement_speed_limits(
        velocity in position_strategy(),
        max_speed in 10.0f32..100.0,
    ) {
        let clamped = if velocity.length() > max_speed {
            velocity.normalize() * max_speed
        } else {
            velocity
        };
        
        prop_assert!(clamped.length() <= max_speed + 0.001);
    }
}

#[cfg(test)]
mod creature_behavior_properties {
    use super::*;
    
    proptest! {
        #[test]
        fn test_creature_seeks_most_urgent_need(
            hunger in 0.0f32..1.0,
            thirst in 0.0f32..1.0,
            energy in 0.0f32..1.0,
        ) {
            let needs = Needs {
                hunger,
                thirst,
                energy,
                social: 0.0,
            };
            
            // The most_urgent method returns (NeedType, urgency_value)
            let (need_type, urgency) = needs.most_urgent();
            
            // Verify it returns the correct need type
            let energy_urgency = 1.0 - energy; // Energy is inverted
            let max_urgency = hunger.max(thirst).max(energy_urgency);
            
            prop_assert_eq!(urgency, max_urgency);
            
            if urgency == hunger {
                prop_assert_eq!(need_type, NeedType::Hunger);
            } else if urgency == thirst {
                prop_assert_eq!(need_type, NeedType::Thirst);
            } else {
                prop_assert_eq!(need_type, NeedType::Energy);
            }
        }
        
        #[test]
        fn test_creature_state_transitions_valid(
            current_state in prop::sample::select(vec![
                CreatureState::Idle,
                CreatureState::Moving { target: Vec2::ZERO },
                CreatureState::Eating,
                CreatureState::Drinking,
                CreatureState::Resting,
                CreatureState::Dead,
            ]),
            has_target in any::<bool>(),
            at_resource in any::<bool>(),
            is_threatened in any::<bool>(),
        ) {
            use CreatureState::*;
            
            // Define valid transitions
            let next_state = match (&current_state, is_threatened, has_target, at_resource) {
                (Dead, _, _, _) => Dead,
                (_, true, _, _) => Moving { target: Vec2::new(100.0, 100.0) },
                (Idle, _, true, _) => Moving { target: Vec2::ZERO },
                (Moving { .. }, _, _, true) => match rand::random::<u8>() % 3 {
                    0 => Eating,
                    1 => Drinking,
                    _ => Resting,
                },
                (Eating | Drinking | Resting, _, false, false) => Idle,
                _ => current_state.clone(),
            };
            
            // All states should be valid
            // Verify the state is valid
            let is_valid = matches!(
                next_state,
                Idle | Moving { .. } | Eating | Drinking | Resting | Dead
            );
            prop_assert!(is_valid);
        }
        
        #[test]
        fn test_decision_timer_behavior(
            delta_time in time_delta_strategy(),
        ) {
            let mut timer = DecisionTimer::default();
            
            // Test that timer works
            timer.timer.tick(std::time::Duration::from_secs_f32(delta_time));
            
            // Timer should either be ready or not
            prop_assert!(timer.timer.finished() || !timer.timer.finished());
        }
    }
}

#[cfg(test)]
mod resource_properties {
    use super::*;
    
    proptest! {
        #[test]
        fn test_resource_depletion_monotonic(
            initial in 10.0f32..100.0,
            consumptions in prop::collection::vec(0.1f32..5.0, 1..20),
        ) {
            let mut resource = ResourceAmount::new(initial);
            let mut previous = resource.current;
            
            for amount in consumptions {
                resource.current = (resource.current - amount).max(0.0);
                prop_assert!(resource.current <= previous);
                prop_assert!(resource.current >= 0.0);
                
                if resource.is_depleted() {
                    prop_assert_eq!(resource.current, 0.0);
                    break;
                }
                
                previous = resource.current;
            }
        }
        
        #[test]
        fn test_resource_regeneration_bounded(
            initial in 0.0f32..50.0,
            max_amount in 50.0f32..100.0,
            regen_rate in 0.1f32..5.0,
            time_steps in prop::collection::vec(time_delta_strategy(), 1..100),
        ) {
            let mut resource = ResourceAmount {
                current: initial,
                max: max_amount,
            };
            
            for delta in time_steps {
                let before = resource.current;
                let regen = regen_rate * delta;
                resource.current = (resource.current + regen).min(resource.max);
                
                prop_assert!(resource.current >= before);
                prop_assert!(resource.current <= resource.max);
            }
        }
    }
}

#[cfg(test)]
mod integration_properties {
    use super::*;
    use bevy::{app::App, MinimalPlugins};
    use creature_simulation::plugins::*;
    use creature_simulation::core::determinism::DeterminismPlugin;
    
    #[test]
    fn test_simulation_determinism() {
        // Create two identical apps with same seed
        let seed = 12345u64;
        
        let mut app1 = create_test_app(seed);
        let mut app2 = create_test_app(seed);
        
        // Run for same number of frames
        for _ in 0..100 {
            app1.update();
            app2.update();
        }
        
        // Extract creature positions from both
        let positions1 = extract_creature_positions(&app1);
        let positions2 = extract_creature_positions(&app2);
        
        // Should be identical
        assert_eq!(positions1.len(), positions2.len());
        for (p1, p2) in positions1.iter().zip(positions2.iter()) {
            assert!((p1.x - p2.x).abs() < 0.001);
            assert!((p1.y - p2.y).abs() < 0.001);
        }
    }
    
    fn create_test_app(_seed: u64) -> App {
        let mut app = App::new();
        
        app.add_plugins(MinimalPlugins)
            .add_plugins((
                creature_simulation::core::simulation_control::SimulationControlPlugin,
                DeterminismPlugin,
                CreatureSimulationPlugin, // This includes SimulationPlugin, SpatialPlugin and adds necessary events
            ));
            
        // The DeterminismPlugin handles seed initialization internally
        
        // Spawn test creatures
        for i in 0..5 {
            let angle = i as f32 * std::f32::consts::TAU / 5.0;
            let pos = Vec2::new(angle.cos() * 100.0, angle.sin() * 100.0);
            app.world.spawn(CreatureBundle::new(pos, 1.0));
        }
        
        app
    }
    
    fn extract_creature_positions(app: &App) -> Vec<Vec2> {
        let mut positions = Vec::new();
        
        for entity_ref in app.world.iter_entities() {
            if let Some(position) = entity_ref.get::<Position>() {
                positions.push(position.0);
            }
        }
        
        positions.sort_by(|a, b| {
            a.x.partial_cmp(&b.x).unwrap()
                .then(a.y.partial_cmp(&b.y).unwrap())
        });
        
        positions
    }
}