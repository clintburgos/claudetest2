//! Core simulation plugin with all gameplay systems

use crate::components::*;
use crate::plugins::{CreatureDiedEvent, DeathCause, ResourceConsumedEvent};
use bevy::prelude::*;

/// Plugin containing all core simulation systems
pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add system sets for proper ordering
            .configure_sets(
                Update,
                (
                    SimulationSet::Input,
                    SimulationSet::Decision,
                    SimulationSet::Movement,
                    SimulationSet::Interaction,
                    SimulationSet::Needs,
                    SimulationSet::Death,
                )
                    .chain(),
            )
            // Add systems
            .add_systems(
                Update,
                (
                    decision_system.in_set(SimulationSet::Decision),
                    movement_system.in_set(SimulationSet::Movement),
                    needs_update_system.in_set(SimulationSet::Needs),
                    consumption_system.in_set(SimulationSet::Interaction),
                    death_check_system.in_set(SimulationSet::Death),
                )
                    .run_if(|settings: Res<crate::plugins::SimulationSettings>| !settings.paused),
            );
    }
}

/// System sets for organizing simulation systems
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SimulationSet {
    Input,
    Decision,
    Movement,
    Interaction,
    Needs,
    Death,
}

/// System for creature decision making
fn decision_system(
    time: Res<Time>,
    mut creatures: Query<
        (
            Entity,
            &mut DecisionTimer,
            &mut CurrentTarget,
            &mut CreatureState,
            &Position,
            &Velocity,
            &Needs,
            &Health,
            &CreatureType,
        ),
        With<Creature>,
    >,
    spatial_grid: Res<crate::plugins::spatial::SpatialGrid>,
    all_creatures: Query<(Entity, &Position, &Health, &CreatureType), With<Creature>>,
    resources: Query<
        (Entity, &Position, &ResourceTypeComponent, &ResourceAmount),
        With<ResourceMarker>,
    >,
) {
    for (entity, mut timer, mut target, mut state, pos, _vel, needs, health, creature_type) in
        creatures.iter_mut()
    {
        timer.timer.tick(time.delta());
        timer.last_decision_time += time.delta_seconds();

        if !timer.timer.just_finished() {
            continue;
        }

        // Query nearby entities
        let nearby_entities = spatial_grid.query_radius(pos.0, 50.0);

        // Separate creatures and resources
        let mut nearby_creatures = Vec::new();
        let mut nearby_threats = Vec::new();
        let mut nearby_resources = Vec::new();

        for &nearby_entity in &nearby_entities {
            if nearby_entity == entity {
                continue; // Skip self
            }

            // Check if it's a creature
            if let Ok((creature_entity, creature_pos, creature_health, other_type)) =
                all_creatures.get(nearby_entity)
            {
                let distance = (pos.0 - creature_pos.0).length();

                // Check if it's a threat
                let is_threat = match (creature_type, other_type) {
                    (CreatureType::Herbivore, CreatureType::Carnivore) => true,
                    (CreatureType::Herbivore, CreatureType::Omnivore) => {
                        creature_health.current > health.current * 1.5
                    },
                    _ => false,
                };

                if is_threat {
                    nearby_threats.push((creature_entity, creature_pos.0, distance));
                } else {
                    nearby_creatures.push((
                        creature_entity,
                        creature_pos.0,
                        distance,
                        creature_health.current,
                    ));
                }
            }

            // Check if it's a resource
            if let Ok((resource_entity, resource_pos, resource_type, amount)) =
                resources.get(nearby_entity)
            {
                if !amount.is_depleted() {
                    let distance = (pos.0 - resource_pos.0).length();
                    nearby_resources.push((
                        resource_entity,
                        resource_pos.0,
                        resource_type.0,
                        distance,
                        amount.current,
                    ));
                }
            }
        }

        // Make decision based on needs and nearby entities
        let (need_type, urgency) = needs.most_urgent();

        // First priority: Flee from threats
        if !nearby_threats.is_empty() {
            // Find closest threat
            if let Some((_, threat_pos, _)) = nearby_threats
                .iter()
                .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal))
            {
                // Flee in opposite direction
                let flee_direction = (pos.0 - *threat_pos).normalize_or_zero();
                let flee_target = pos.0 + flee_direction * 100.0;
                *state = CreatureState::Moving {
                    target: flee_target,
                };
                *target = CurrentTarget::Position(flee_target);
                continue;
            }
        }

        // Second priority: Address critical needs
        if urgency > 0.7 {
            match need_type {
                crate::components::NeedType::Hunger => {
                    // Find nearest food
                    if let Some((food_entity, food_pos, _, distance, _)) = nearby_resources
                        .iter()
                        .filter(|(_, _, res_type, _, _)| {
                            *res_type == crate::simulation::ResourceType::Food
                        })
                        .min_by(|a, b| a.3.partial_cmp(&b.3).unwrap_or(std::cmp::Ordering::Equal))
                    {
                        if *distance < 2.0 {
                            *state = CreatureState::Eating;
                            *target = CurrentTarget::Entity(*food_entity);
                        } else {
                            *state = CreatureState::Moving { target: *food_pos };
                            *target = CurrentTarget::Position(*food_pos);
                        }
                        continue;
                    }
                },
                crate::components::NeedType::Thirst => {
                    // Find nearest water
                    if let Some((water_entity, water_pos, _, distance, _)) = nearby_resources
                        .iter()
                        .filter(|(_, _, res_type, _, _)| {
                            *res_type == crate::simulation::ResourceType::Water
                        })
                        .min_by(|a, b| a.3.partial_cmp(&b.3).unwrap_or(std::cmp::Ordering::Equal))
                    {
                        if *distance < 2.0 {
                            *state = CreatureState::Drinking;
                            *target = CurrentTarget::Entity(*water_entity);
                        } else {
                            *state = CreatureState::Moving { target: *water_pos };
                            *target = CurrentTarget::Position(*water_pos);
                        }
                        continue;
                    }
                },
                crate::components::NeedType::Energy => {
                    *state = CreatureState::Resting;
                    *target = CurrentTarget::None;
                    continue;
                },
            }
        }

        // Social behavior - move towards other creatures if lonely
        if needs.social > 0.6 && !nearby_creatures.is_empty() {
            if let Some((_, creature_pos, distance, _)) = nearby_creatures
                .iter()
                .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal))
            {
                if *distance > 10.0 {
                    *state = CreatureState::Moving {
                        target: *creature_pos,
                    };
                    *target = CurrentTarget::Position(*creature_pos);
                    continue;
                }
            }
        }

        // Default: wander if idle
        if matches!(*state, CreatureState::Idle) {
            let wander_target = pos.0
                + Vec2::new(
                    (pos.0.x * 12.345).sin() * 50.0,
                    (pos.0.y * 67.890).cos() * 50.0,
                );
            *state = CreatureState::Moving {
                target: wander_target,
            };
            *target = CurrentTarget::Position(wander_target);
        }

        // Reset decision timer
        timer.last_decision_time = 0.0;
    }
}

/// System for creature movement
fn movement_system(
    time: Res<Time>,
    mut creatures: Query<
        (
            &mut Position,
            &mut Velocity,
            &CurrentTarget,
            &MaxSpeed,
            &CreatureState,
        ),
        With<Creature>,
    >,
    target_positions: Query<&Position, Without<Creature>>,
) {
    let dt = time.delta_seconds();

    for (mut pos, mut vel, target, max_speed, state) in creatures.iter_mut() {
        // Skip if dead or stationary activity
        if matches!(
            state,
            CreatureState::Dead
                | CreatureState::Eating
                | CreatureState::Drinking
                | CreatureState::Resting
        ) {
            vel.0 = Vec2::ZERO;
            continue;
        }

        // Determine target position
        let target_pos = match target {
            CurrentTarget::Position(target_pos) => Some(*target_pos),
            CurrentTarget::Entity(entity) => {
                // Try to get position of target entity
                target_positions.get(*entity).ok().map(|p| p.0)
            },
            CurrentTarget::None => None,
        };

        // Move towards target
        if let Some(target_pos) = target_pos {
            let to_target = target_pos - pos.0;
            let distance = to_target.length();

            if distance > 1.0 {
                // Simple steering
                let desired_velocity = to_target.normalize() * max_speed.0;
                vel.0 = desired_velocity;

                // Update position
                pos.0 += vel.0 * dt;
            } else {
                vel.0 = Vec2::ZERO;
            }
        }
    }
}

/// System for updating creature needs
fn needs_update_system(
    time: Res<Time>,
    mut query: Query<(&mut Needs, &Size, &CreatureState), With<Creature>>,
) {
    let dt = time.delta_seconds();

    for (mut needs, size, state) in query.iter_mut() {
        // Metabolism rate based on size
        let metabolism = 1.0 / size.0.sqrt();

        // Update needs based on state
        match state {
            CreatureState::Resting => {
                needs.energy = (needs.energy - 0.1 * dt).max(0.0);
            },
            CreatureState::Eating => {
                needs.hunger = (needs.hunger - 0.5 * dt).max(0.0);
            },
            CreatureState::Drinking => {
                needs.thirst = (needs.thirst - 0.5 * dt).max(0.0);
            },
            _ => {
                // Normal need increase
                needs.hunger = (needs.hunger + 0.1 * metabolism * dt).min(1.0);
                needs.thirst = (needs.thirst + 0.15 * metabolism * dt).min(1.0);
                needs.energy = (needs.energy + 0.05 * dt).min(1.0);
            },
        }
    }
}

/// System for resource consumption
fn consumption_system(
    mut creatures: Query<(&Position, &mut Needs, &CreatureState), With<Creature>>,
    mut resources: Query<
        (&Position, &mut ResourceAmount, &ResourceTypeComponent),
        With<ResourceMarker>,
    >,
    _events: EventWriter<ResourceConsumedEvent>,
) {
    for (creature_pos, _needs, state) in creatures.iter_mut() {
        // Check if creature is consuming
        let consuming_type = match state {
            CreatureState::Eating => Some(crate::simulation::ResourceType::Food),
            CreatureState::Drinking => Some(crate::simulation::ResourceType::Water),
            _ => None,
        };

        if let Some(resource_type) = consuming_type {
            // Find nearby resource of correct type
            for (resource_pos, mut amount, res_type) in resources.iter_mut() {
                if res_type.0 != resource_type {
                    continue;
                }

                let distance = (creature_pos.0 - resource_pos.0).length();
                if distance < 2.0 && !amount.is_depleted() {
                    // Consume resource
                    let consumed = amount.consume(1.0);
                    if consumed > 0.0 {
                        // TODO: Get entity IDs for event
                        // events.send(ResourceConsumedEvent { ... });
                    }
                }
            }
        }
    }
}

/// System for checking creature death
fn death_check_system(
    mut commands: Commands,
    query: Query<(Entity, &Health, &Needs, &Age), With<Creature>>,
    mut events: EventWriter<CreatureDiedEvent>,
) {
    for (entity, health, needs, age) in query.iter() {
        let mut should_die = false;
        let mut cause = DeathCause::OldAge;

        if health.is_dead() {
            should_die = true;
        } else if needs.hunger >= 1.0 {
            should_die = true;
            cause = DeathCause::Starvation;
        } else if needs.thirst >= 1.0 {
            should_die = true;
            cause = DeathCause::Dehydration;
        } else if needs.energy >= 1.0 {
            should_die = true;
            cause = DeathCause::Exhaustion;
        } else if age.0 > 3600.0 {
            // 1 hour
            should_die = true;
            cause = DeathCause::OldAge;
        }

        if should_die {
            events.send(CreatureDiedEvent { entity, cause });
            commands.entity(entity).despawn();
        }
    }
}
