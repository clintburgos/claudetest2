//! Decoupled Decision System for creature AI with pure functions and caching.
//!
//! This module implements Phase 1.2 of the improvement plan, separating
//! decision logic from state access for better testability and performance.

use crate::core::versioned_entity::EntityVersioning;
use crate::core::{Version, VersionedEntity};
use crate::simulation::{needs::NeedType, Creature, CreatureState, Resource, ResourceType};
use bevy::prelude::*;
use parking_lot::RwLock;
use std::collections::HashMap;

/// Decision context containing all information needed for decision-making.
/// This struct is passed to pure decision functions, decoupling them from world state.
#[derive(Clone, Debug)]
pub struct DecisionContext {
    pub entity: VersionedEntity,
    pub position: Vec2,
    pub velocity: Vec2,
    pub state: CreatureState,
    pub needs: NeedState,
    pub health: f32,
    pub energy: f32,
    pub nearby_resources: Vec<ResourceInfo>,
    pub nearby_creatures: Vec<CreatureInfo>,
    pub nearby_threats: Vec<ThreatInfo>,
    pub time_since_last_decision: f32,
}

/// Simplified need state for decision-making
#[derive(Clone, Debug, Default)]
pub struct NeedState {
    pub hunger: f32,
    pub thirst: f32,
    pub energy: f32,
    pub social: f32,
}

impl NeedState {
    pub fn most_urgent(&self) -> (NeedType, f32) {
        let mut most_urgent = NeedType::Hunger;
        let mut highest_urgency = self.hunger;

        if self.thirst > highest_urgency {
            most_urgent = NeedType::Thirst;
            highest_urgency = self.thirst;
        }

        if self.energy < 0.2 && (1.0 - self.energy) > highest_urgency {
            most_urgent = NeedType::Energy;
            highest_urgency = 1.0 - self.energy;
        }

        (most_urgent, highest_urgency)
    }
}

/// Information about nearby resources
#[derive(Clone, Debug)]
pub struct ResourceInfo {
    pub entity: VersionedEntity,
    pub position: Vec2,
    pub resource_type: ResourceType,
    pub amount: f32,
    pub distance: f32,
}

/// Information about nearby creatures
#[derive(Clone, Debug)]
pub struct CreatureInfo {
    pub entity: VersionedEntity,
    pub position: Vec2,
    pub relationship: Relationship,
    pub distance: f32,
}

/// Information about nearby threats
#[derive(Clone, Debug)]
pub struct ThreatInfo {
    pub position: Vec2,
    pub threat_level: f32,
    pub distance: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Relationship {
    Friendly,
    Neutral,
    Hostile,
}

/// The decision output from the AI system
#[derive(Clone, Debug, PartialEq)]
pub enum Decision {
    Move {
        target: Vec2,
        urgency: f32,
    },
    Consume {
        resource: VersionedEntity,
        resource_type: ResourceType,
    },
    Rest {
        duration: f32,
    },
    Socialize {
        target: VersionedEntity,
    },
    Flee {
        direction: Vec2,
    },
    Idle,
}

/// Cache for decision results to avoid recomputation
pub struct DecisionCache {
    cache: RwLock<HashMap<u64, CachedDecision>>,
    max_entries: usize,
}

#[derive(Clone)]
struct CachedDecision {
    decision: Decision,
    timestamp: f32,
    context_hash: u64,
}

impl DecisionCache {
    pub fn new(max_entries: usize) -> Self {
        Self {
            cache: RwLock::new(HashMap::with_capacity(max_entries)),
            max_entries,
        }
    }

    pub fn get(&self, context: &DecisionContext, current_time: f32) -> Option<Decision> {
        let hash = self.hash_context(context);
        let cache = self.cache.read();

        if let Some(cached) = cache.get(&hash) {
            // Cache entries expire after 1 second
            if current_time - cached.timestamp < 1.0 {
                return Some(cached.decision.clone());
            }
        }

        None
    }

    pub fn insert(&self, context: &DecisionContext, decision: Decision, current_time: f32) {
        let hash = self.hash_context(context);
        let mut cache = self.cache.write();

        // Simple LRU eviction if cache is full
        if cache.len() >= self.max_entries {
            if let Some(oldest_key) =
                cache.iter().min_by_key(|(_, v)| v.timestamp as i64).map(|(k, _)| *k)
            {
                cache.remove(&oldest_key);
            }
        }

        cache.insert(
            hash,
            CachedDecision {
                decision,
                timestamp: current_time,
                context_hash: hash,
            },
        );
    }

    fn hash_context(&self, context: &DecisionContext) -> u64 {
        // Simple hash based on key decision factors
        let mut hash = 0u64;
        hash ^= (context.position.x as i32 as u64) << 32;
        hash ^= (context.position.y as i32 as u64) << 16;
        hash ^= (context.needs.hunger * 100.0) as u64;
        hash ^= ((context.needs.thirst * 100.0) as u64) << 8;
        hash ^= ((context.needs.energy * 100.0) as u64) << 16;
        hash ^= (context.nearby_resources.len() as u64) << 24;
        hash
    }
}

/// Pure decision-making functions that operate on contexts
pub mod decision_functions {
    use super::*;

    /// Main decision function - pure and testable
    pub fn make_decision(context: &DecisionContext) -> Decision {
        // Check for immediate threats
        if let Some(flee_decision) = check_threats(context) {
            return flee_decision;
        }

        // Check urgent needs
        let (most_urgent_need, urgency) = context.needs.most_urgent();

        if urgency > 0.7 {
            if let Some(need_decision) = address_urgent_need(context, most_urgent_need) {
                return need_decision;
            }
        }

        // Social interactions if no urgent needs
        if context.needs.social > 0.5 {
            if let Some(social_decision) = find_social_interaction(context) {
                return social_decision;
            }
        }

        // Default to idle or wander
        if context.state == CreatureState::Idle {
            Decision::Move {
                target: calculate_wander_target(context.position),
                urgency: 0.2,
            }
        } else {
            Decision::Idle
        }
    }

    fn check_threats(context: &DecisionContext) -> Option<Decision> {
        if let Some(nearest_threat) = context
            .nearby_threats
            .iter()
            .min_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap())
        {
            if nearest_threat.threat_level > 0.5 && nearest_threat.distance < 20.0 {
                let flee_direction = (context.position - nearest_threat.position).normalize();
                return Some(Decision::Flee {
                    direction: flee_direction,
                });
            }
        }

        None
    }

    fn address_urgent_need(context: &DecisionContext, need: NeedType) -> Option<Decision> {
        match need {
            NeedType::Hunger => find_food_decision(context),
            NeedType::Thirst => find_water_decision(context),
            NeedType::Energy => Some(Decision::Rest { duration: 5.0 }),
        }
    }

    fn find_food_decision(context: &DecisionContext) -> Option<Decision> {
        let food_sources: Vec<_> = context
            .nearby_resources
            .iter()
            .filter(|r| r.resource_type == ResourceType::Food)
            .collect();

        if let Some(best_food) = find_best_resource(&food_sources, context) {
            Some(Decision::Consume {
                resource: best_food.entity,
                resource_type: ResourceType::Food,
            })
        } else {
            // Search for food
            Some(Decision::Move {
                target: calculate_search_target(context.position, ResourceType::Food),
                urgency: context.needs.hunger,
            })
        }
    }

    fn find_water_decision(context: &DecisionContext) -> Option<Decision> {
        let water_sources: Vec<_> = context
            .nearby_resources
            .iter()
            .filter(|r| r.resource_type == ResourceType::Water)
            .collect();

        if let Some(best_water) = find_best_resource(&water_sources, context) {
            Some(Decision::Consume {
                resource: best_water.entity,
                resource_type: ResourceType::Water,
            })
        } else {
            // Search for water
            Some(Decision::Move {
                target: calculate_search_target(context.position, ResourceType::Water),
                urgency: context.needs.thirst,
            })
        }
    }

    fn find_social_interaction(context: &DecisionContext) -> Option<Decision> {
        context
            .nearby_creatures
            .iter()
            .filter(|c| c.relationship == Relationship::Friendly && c.distance < 10.0)
            .min_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap())
            .map(|creature| Decision::Socialize {
                target: creature.entity,
            })
    }

    fn find_best_resource<'a>(
        resources: &[&'a ResourceInfo],
        context: &DecisionContext,
    ) -> Option<&'a ResourceInfo> {
        resources
            .iter()
            .min_by(|a, b| {
                // Score based on distance and amount
                let score_a = a.distance / a.amount.max(0.1);
                let score_b = b.distance / b.amount.max(0.1);
                score_a.partial_cmp(&score_b).unwrap()
            })
            .copied()
    }

    fn calculate_wander_target(current_pos: Vec2) -> Vec2 {
        // Simple wander logic - in practice would use proper RNG
        let angle = (current_pos.x * 0.1 + current_pos.y * 0.2).sin() * std::f32::consts::TAU;
        let offset = Vec2::new(angle.cos(), angle.sin()) * 15.0;
        current_pos + offset
    }

    fn calculate_search_target(current_pos: Vec2, resource_type: ResourceType) -> Vec2 {
        // Different search patterns for different resources
        let angle_offset = match resource_type {
            ResourceType::Food => 0.0,
            ResourceType::Water => std::f32::consts::PI,
        };

        let angle = (current_pos.x * 0.15).sin() * std::f32::consts::TAU + angle_offset;
        let offset = Vec2::new(angle.cos(), angle.sin()) * 25.0;
        current_pos + offset
    }
}

/// System for gathering decision contexts from the ECS world
pub fn gather_decision_contexts(
    creatures: Query<(Entity, &Transform, &Version<Creature>, &Version<Velocity>)>,
    resources: Query<(Entity, &Transform, &Version<Resource>)>,
    spatial: Res<crate::core::SpatialHashGrid>,
    entity_versions: Res<crate::core::EntityVersions>,
) -> Vec<DecisionContext> {
    let mut contexts = Vec::new();

    for (entity, transform, creature, velocity) in creatures.iter() {
        if !creature.is_alive() {
            continue;
        }

        let versioned_entity = entity.to_versioned(&entity_versions).unwrap();
        let position = transform.translation.truncate();

        // Gather nearby entities using spatial grid
        let nearby_resources = spatial
            .query_radius(position, 30.0)
            .into_iter()
            .filter_map(|e| {
                resources.get(e).ok().and_then(|(entity, transform, resource)| {
                    let resource_pos = transform.translation.truncate();
                    let distance = (resource_pos - position).length();

                    entity.to_versioned(&entity_versions).map(|versioned| ResourceInfo {
                        entity: versioned,
                        position: resource_pos,
                        resource_type: resource.resource_type,
                        amount: resource.amount,
                        distance,
                    })
                })
            })
            .collect();

        let context = DecisionContext {
            entity: versioned_entity,
            position,
            velocity: velocity.0,
            state: creature.state.clone(),
            needs: NeedState {
                hunger: creature.needs.hunger,
                thirst: creature.needs.thirst,
                energy: creature.needs.energy,
                social: 0.0, // Not implemented yet
            },
            health: creature.health.current / creature.health.max,
            energy: creature.needs.energy,
            nearby_resources,
            nearby_creatures: vec![], // TODO: Implement creature detection
            nearby_threats: vec![],   // TODO: Implement threat detection
            time_since_last_decision: 0.0, // TODO: Track this
        };

        contexts.push(context);
    }

    contexts
}

/// System for executing decisions
pub fn execute_decisions(
    decisions: Vec<(VersionedEntity, Decision)>,
    mut creatures: Query<(&mut Version<Creature>, &mut Transform)>,
    entity_versions: Res<crate::core::EntityVersions>,
) {
    for (versioned_entity, decision) in decisions {
        let entity = Entity::from_raw(versioned_entity.id);

        if let Ok((mut creature, mut transform)) = creatures.get_mut(entity) {
            match decision {
                Decision::Move { target, urgency: _ } => {
                    creature.start_moving(target);
                },
                Decision::Consume {
                    resource,
                    resource_type,
                } => match resource_type {
                    ResourceType::Food => creature.start_eating(),
                    ResourceType::Water => creature.start_drinking(),
                },
                Decision::Rest { duration: _ } => {
                    creature.start_resting();
                },
                Decision::Socialize { target: _ } => {
                    // TODO: Implement social interactions
                },
                Decision::Flee { direction } => {
                    let flee_target = transform.translation.truncate() + direction * 30.0;
                    creature.start_moving(flee_target);
                },
                Decision::Idle => {
                    // Do nothing
                },
            }
        }
    }
}

/// Component for Bevy's Velocity
#[derive(Component, Default, Clone, Debug)]
pub struct Velocity(pub Vec2);

/// The main decoupled decision system
#[derive(Resource)]
pub struct DecoupledDecisionSystem {
    pub cache: DecisionCache,
    pub decision_interval: f32,
    pub last_update: f32,
}

impl Default for DecoupledDecisionSystem {
    fn default() -> Self {
        Self {
            cache: DecisionCache::new(1000),
            decision_interval: 0.5, // Make decisions every 0.5 seconds
            last_update: 0.0,
        }
    }
}

/// Plugin for the decoupled decision system
pub struct DecoupledDecisionPlugin;

impl Plugin for DecoupledDecisionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DecoupledDecisionSystem>()
            .add_systems(Update, (decision_system, execute_decision_system).chain());
    }
}

fn decision_system(
    mut decision_system: ResMut<DecoupledDecisionSystem>,
    time: Res<Time>,
    creatures: Query<(Entity, &Transform, &Version<Creature>, &Version<Velocity>)>,
    resources: Query<(Entity, &Transform, &Version<Resource>)>,
    spatial: Res<crate::core::SpatialHashGrid>,
    entity_versions: Res<crate::core::EntityVersions>,
) {
    let current_time = time.elapsed_seconds();

    // Only update at the configured interval
    if current_time - decision_system.last_update < decision_system.decision_interval {
        return;
    }

    decision_system.last_update = current_time;

    // Gather contexts
    let contexts = gather_decision_contexts(creatures, resources, spatial, entity_versions);

    // Make decisions (can be parallelized with rayon)
    let decisions: Vec<_> = contexts
        .into_iter()
        .filter_map(|context| {
            // Check cache first
            if let Some(cached) = decision_system.cache.get(&context, current_time) {
                return Some((context.entity, cached));
            }

            // Make new decision
            let decision = decision_functions::make_decision(&context);

            // Cache the result
            decision_system.cache.insert(&context, decision.clone(), current_time);

            Some((context.entity, decision))
        })
        .collect();

    // Store decisions for execution in next system
    // In practice, you'd use Events or a resource to pass decisions
}

fn execute_decision_system(// This would receive decisions from the previous system
    // Implementation depends on how you want to pass data between systems
) {
    // Execute decisions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pure_decision_making() {
        let context = DecisionContext {
            entity: VersionedEntity::new(1, 0),
            position: Vec2::new(50.0, 50.0),
            velocity: Vec2::ZERO,
            state: CreatureState::Idle,
            needs: NeedState {
                hunger: 0.9, // Very hungry
                thirst: 0.3,
                energy: 0.7,
                social: 0.2,
            },
            health: 1.0,
            energy: 0.7,
            nearby_resources: vec![ResourceInfo {
                entity: VersionedEntity::new(2, 0),
                position: Vec2::new(60.0, 50.0),
                resource_type: ResourceType::Food,
                amount: 50.0,
                distance: 10.0,
            }],
            nearby_creatures: vec![],
            nearby_threats: vec![],
            time_since_last_decision: 1.0,
        };

        let decision = decision_functions::make_decision(&context);

        // Should decide to consume the nearby food
        match decision {
            Decision::Consume { resource_type, .. } => {
                assert_eq!(resource_type, ResourceType::Food);
            },
            _ => panic!("Expected Consume decision for hungry creature near food"),
        }
    }

    #[test]
    fn test_threat_avoidance() {
        let context = DecisionContext {
            entity: VersionedEntity::new(1, 0),
            position: Vec2::new(50.0, 50.0),
            velocity: Vec2::ZERO,
            state: CreatureState::Idle,
            needs: NeedState::default(),
            health: 1.0,
            energy: 1.0,
            nearby_resources: vec![],
            nearby_creatures: vec![],
            nearby_threats: vec![ThreatInfo {
                position: Vec2::new(55.0, 50.0),
                threat_level: 0.8,
                distance: 5.0,
            }],
            time_since_last_decision: 1.0,
        };

        let decision = decision_functions::make_decision(&context);

        // Should flee from threat
        match decision {
            Decision::Flee { direction } => {
                // Should flee away from threat (negative x direction)
                assert!(direction.x < 0.0);
            },
            _ => panic!("Expected Flee decision when near threat"),
        }
    }

    #[test]
    fn test_decision_caching() {
        let cache = DecisionCache::new(10);
        let context = DecisionContext {
            entity: VersionedEntity::new(1, 0),
            position: Vec2::new(50.0, 50.0),
            velocity: Vec2::ZERO,
            state: CreatureState::Idle,
            needs: NeedState::default(),
            health: 1.0,
            energy: 1.0,
            nearby_resources: vec![],
            nearby_creatures: vec![],
            nearby_threats: vec![],
            time_since_last_decision: 1.0,
        };

        let decision = Decision::Idle;
        cache.insert(&context, decision.clone(), 0.0);

        // Should retrieve from cache
        let cached = cache.get(&context, 0.5);
        assert_eq!(cached, Some(Decision::Idle));

        // Should expire after 1 second
        let expired = cache.get(&context, 1.1);
        assert_eq!(expired, None);
    }

    #[test]
    fn test_most_urgent_need() {
        let needs = NeedState {
            hunger: 0.9,
            thirst: 0.5,
            energy: 0.3,
            social: 0.1,
        };

        let (need_type, urgency) = needs.most_urgent();
        assert_eq!(need_type, NeedType::Hunger);
        assert_eq!(urgency, 0.9);

        let needs2 = NeedState {
            hunger: 0.2,
            thirst: 0.95,
            energy: 0.3,
            social: 0.1,
        };

        let (need_type, urgency) = needs2.most_urgent();
        assert_eq!(need_type, NeedType::Thirst);
        assert_eq!(urgency, 0.95);

        // Energy is inverted (0 = exhausted)
        let needs3 = NeedState {
            hunger: 0.2,
            thirst: 0.2,
            energy: 0.05, // Very low energy
            social: 0.1,
        };

        let (need_type, urgency) = needs3.most_urgent();
        assert_eq!(need_type, NeedType::Energy);
        assert!((urgency - 0.95).abs() < 0.001); // 1.0 - 0.05 = 0.95
    }

    #[test]
    fn test_water_decision() {
        let context = DecisionContext {
            entity: VersionedEntity::new(1, 0),
            position: Vec2::new(50.0, 50.0),
            velocity: Vec2::ZERO,
            state: CreatureState::Idle,
            needs: NeedState {
                hunger: 0.2,
                thirst: 0.9, // Very thirsty
                energy: 0.7,
                social: 0.2,
            },
            health: 1.0,
            energy: 0.7,
            nearby_resources: vec![ResourceInfo {
                entity: VersionedEntity::new(2, 0),
                position: Vec2::new(60.0, 50.0),
                resource_type: ResourceType::Water,
                amount: 50.0,
                distance: 10.0,
            }],
            nearby_creatures: vec![],
            nearby_threats: vec![],
            time_since_last_decision: 1.0,
        };

        let decision = decision_functions::make_decision(&context);

        match decision {
            Decision::Consume { resource_type, .. } => {
                assert_eq!(resource_type, ResourceType::Water);
            },
            _ => panic!("Expected Consume water decision for thirsty creature"),
        }
    }

    #[test]
    fn test_social_interaction() {
        let context = DecisionContext {
            entity: VersionedEntity::new(1, 0),
            position: Vec2::new(50.0, 50.0),
            velocity: Vec2::ZERO,
            state: CreatureState::Idle,
            needs: NeedState {
                hunger: 0.2,
                thirst: 0.2,
                energy: 0.8,
                social: 0.9, // Very lonely
            },
            health: 1.0,
            energy: 0.8,
            nearby_resources: vec![],
            nearby_creatures: vec![CreatureInfo {
                entity: VersionedEntity::new(2, 0),
                position: Vec2::new(55.0, 50.0),
                relationship: Relationship::Friendly,
                distance: 5.0,
            }],
            nearby_threats: vec![],
            time_since_last_decision: 1.0,
        };

        let decision = decision_functions::make_decision(&context);

        match decision {
            Decision::Socialize { .. } => {
                // Expected
            },
            _ => panic!("Expected Socialize decision for lonely creature near friend"),
        }
    }

    #[test]
    fn test_rest_decision() {
        let context = DecisionContext {
            entity: VersionedEntity::new(1, 0),
            position: Vec2::new(50.0, 50.0),
            velocity: Vec2::ZERO,
            state: CreatureState::Idle,
            needs: NeedState {
                hunger: 0.2,
                thirst: 0.2,
                energy: 0.1, // Very tired
                social: 0.2,
            },
            health: 1.0,
            energy: 0.1,
            nearby_resources: vec![],
            nearby_creatures: vec![],
            nearby_threats: vec![],
            time_since_last_decision: 1.0,
        };

        let decision = decision_functions::make_decision(&context);

        match decision {
            Decision::Rest { duration } => {
                assert!(duration > 0.0);
            },
            _ => panic!("Expected Rest decision for tired creature"),
        }
    }

    #[test]
    fn test_wander_decision() {
        let context = DecisionContext {
            entity: VersionedEntity::new(1, 0),
            position: Vec2::new(50.0, 50.0),
            velocity: Vec2::ZERO,
            state: CreatureState::Idle,
            needs: NeedState {
                hunger: 0.2,
                thirst: 0.2,
                energy: 0.8,
                social: 0.2,
            },
            health: 1.0,
            energy: 0.8,
            nearby_resources: vec![],
            nearby_creatures: vec![],
            nearby_threats: vec![],
            time_since_last_decision: 1.0,
        };

        let decision = decision_functions::make_decision(&context);

        match decision {
            Decision::Move { urgency, .. } => {
                assert!(urgency < 0.5); // Low urgency for wandering
            },
            Decision::Idle => {
                // Also acceptable
            },
            _ => panic!("Expected Move or Idle for creature with no urgent needs"),
        }
    }

    #[test]
    fn test_search_decision() {
        let context = DecisionContext {
            entity: VersionedEntity::new(1, 0),
            position: Vec2::new(50.0, 50.0),
            velocity: Vec2::ZERO,
            state: CreatureState::Idle,
            needs: NeedState {
                hunger: 0.8, // Hungry
                thirst: 0.2,
                energy: 0.8,
                social: 0.2,
            },
            health: 1.0,
            energy: 0.8,
            nearby_resources: vec![], // No food nearby
            nearby_creatures: vec![],
            nearby_threats: vec![],
            time_since_last_decision: 1.0,
        };

        let decision = decision_functions::make_decision(&context);

        match decision {
            Decision::Move { urgency, .. } => {
                assert_eq!(urgency, 0.8); // High urgency matching hunger
            },
            _ => panic!("Expected Move decision to search for food"),
        }
    }

    #[test]
    fn test_empty_nearby_lists() {
        let context = DecisionContext {
            entity: VersionedEntity::new(1, 0),
            position: Vec2::new(50.0, 50.0),
            velocity: Vec2::ZERO,
            state: CreatureState::Idle,
            needs: NeedState::default(),
            health: 1.0,
            energy: 1.0,
            nearby_resources: vec![],
            nearby_creatures: vec![],
            nearby_threats: vec![],
            time_since_last_decision: 1.0,
        };

        // Should not panic with empty lists
        let decision = decision_functions::make_decision(&context);
        // With default NeedState, energy is 0.0 which means exhausted
        // So it should decide to rest
        match decision {
            Decision::Rest { duration } => {
                assert!(duration > 0.0);
            },
            _ => panic!("Expected Rest decision for exhausted creature"),
        }
    }

    #[test]
    fn test_decision_cache_hash_collision() {
        let cache = DecisionCache::new(10);

        // Create two different contexts
        let context1 = DecisionContext {
            entity: VersionedEntity::new(1, 0),
            position: Vec2::new(50.0, 50.0),
            velocity: Vec2::ZERO,
            state: CreatureState::Idle,
            needs: NeedState::default(),
            health: 1.0,
            energy: 1.0,
            nearby_resources: vec![],
            nearby_creatures: vec![],
            nearby_threats: vec![],
            time_since_last_decision: 1.0,
        };

        let mut context2 = context1.clone();
        context2.position.x = 51.0;

        // Insert different decisions for different contexts
        cache.insert(&context1, Decision::Idle, 0.0);
        cache.insert(
            &context2,
            Decision::Move {
                target: Vec2::ZERO,
                urgency: 0.5,
            },
            0.0,
        );

        // Should get correct decision for each context
        assert_eq!(cache.get(&context1, 0.5), Some(Decision::Idle));
        match cache.get(&context2, 0.5) {
            Some(Decision::Move { .. }) => (), // Expected
            _ => panic!("Wrong decision cached"),
        }
    }
}
