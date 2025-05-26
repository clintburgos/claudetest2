//! Decoupled Decision System for creature AI with pure functions and caching.
//!
//! This module implements Phase 1.2 of the improvement plan, separating
//! decision logic from state access for better testability and performance.

use crate::config::decision::*;
use crate::core::Entity;
use crate::simulation::{needs::NeedType, CreatureState, ResourceType};
use crate::Vec2;
use parking_lot::RwLock;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::Hasher;

/// Decision context containing all information needed for decision-making.
/// This struct is passed to pure decision functions, decoupling them from world state.
#[derive(Clone, Debug)]
pub struct DecisionContext {
    pub entity: Entity,
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

        if self.energy < LOW_ENERGY_THRESHOLD && (1.0 - self.energy) > highest_urgency {
            most_urgent = NeedType::Energy;
            highest_urgency = 1.0 - self.energy;
        }

        (most_urgent, highest_urgency)
    }
}

/// Information about nearby resources
#[derive(Clone, Debug)]
pub struct ResourceInfo {
    pub entity: Entity,
    pub position: Vec2,
    pub resource_type: ResourceType,
    pub amount: f32,
    pub distance: f32,
}

/// Information about nearby creatures
#[derive(Clone, Debug)]
pub struct CreatureInfo {
    pub entity: Entity,
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
        resource: Entity,
        resource_type: ResourceType,
    },
    Rest {
        duration: f32,
    },
    Socialize {
        target: Entity,
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
    #[allow(dead_code)]
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
            // Cache entries expire after configured time
            if current_time - cached.timestamp < CACHE_EXPIRY_TIME {
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
        // Use proper hashing to avoid collisions
        let mut hasher = DefaultHasher::new();

        // Hash position with full precision
        hasher.write_u32(context.position.x.to_bits());
        hasher.write_u32(context.position.y.to_bits());

        // Hash needs with reasonable precision (2 decimal places)
        hasher.write_u32((context.needs.hunger * 100.0) as u32);
        hasher.write_u32((context.needs.thirst * 100.0) as u32);
        hasher.write_u32((context.needs.energy * 100.0) as u32);

        // Hash creature state
        hasher.write_u8(match context.state {
            CreatureState::Idle => 0,
            CreatureState::Moving { .. } => 1,
            CreatureState::Eating => 2,
            CreatureState::Drinking => 3,
            CreatureState::Resting => 4,
            CreatureState::Dead => 5,
        });

        // Hash resource availability
        hasher.write_usize(context.nearby_resources.len());
        for resource in &context.nearby_resources {
            hasher.write_u32(resource.entity.id());
        }

        hasher.finish()
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

        // If already eating/drinking and still have that need, continue
        match context.state {
            CreatureState::Eating if context.needs.hunger > 0.1 => {
                // Continue eating if still hungry
                return Decision::Idle;
            },
            CreatureState::Drinking if context.needs.thirst > 0.1 => {
                // Continue drinking if still thirsty
                return Decision::Idle;
            },
            _ => {},
        }

        // Check urgent needs
        let (most_urgent_need, urgency) = context.needs.most_urgent();

        if urgency > HIGH_URGENCY_THRESHOLD {
            if let Some(need_decision) = address_urgent_need(context, most_urgent_need) {
                return need_decision;
            }
        }

        // Social interactions if no urgent needs
        if context.needs.social > SOCIAL_NEED_THRESHOLD {
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
            .min_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap_or(std::cmp::Ordering::Equal))
        {
            if nearest_threat.threat_level > HIGH_THREAT_LEVEL
                && nearest_threat.distance < THREAT_PROXIMITY
            {
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
            NeedType::Energy => Some(Decision::Rest {
                duration: DEFAULT_REST_DURATION,
            }),
        }
    }

    fn find_food_decision(context: &DecisionContext) -> Option<Decision> {
        let food_sources: Vec<_> = context
            .nearby_resources
            .iter()
            .filter(|r| r.resource_type == ResourceType::Food)
            .collect();

        if let Some(best_food) = find_best_resource(&food_sources, context) {
            // Check if we're close enough to consume
            if best_food.distance <= INTERACTION_DISTANCE {
                Some(Decision::Consume {
                    resource: best_food.entity,
                    resource_type: ResourceType::Food,
                })
            } else {
                // Move to the food first
                Some(Decision::Move {
                    target: best_food.position,
                    urgency: context.needs.hunger,
                })
            }
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
            // Check if we're close enough to consume
            if best_water.distance <= INTERACTION_DISTANCE {
                Some(Decision::Consume {
                    resource: best_water.entity,
                    resource_type: ResourceType::Water,
                })
            } else {
                // Move to the water first
                Some(Decision::Move {
                    target: best_water.position,
                    urgency: context.needs.thirst,
                })
            }
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
            .filter(|c| {
                c.relationship == Relationship::Friendly && c.distance < SOCIAL_INTERACTION_DISTANCE
            })
            .min_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap_or(std::cmp::Ordering::Equal))
            .map(|creature| Decision::Socialize {
                target: creature.entity,
            })
    }

    fn find_best_resource<'a>(
        resources: &[&'a ResourceInfo],
        _context: &DecisionContext,
    ) -> Option<&'a ResourceInfo> {
        resources
            .iter()
            .min_by(|a, b| {
                // Score based on distance and amount
                let score_a = a.distance / a.amount.max(MIN_RESOURCE_AMOUNT);
                let score_b = b.distance / b.amount.max(MIN_RESOURCE_AMOUNT);
                score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .copied()
    }

    fn calculate_wander_target(current_pos: Vec2) -> Vec2 {
        // Simple wander logic - in practice would use proper RNG
        // This creates a pseudo-random angle based on position:
        // 1. Combine x and y coordinates with different weights (0.1, 0.2) to create variation
        // 2. Take sin() to map to [-1, 1] range
        // 3. Multiply by TAU (2π) to get full circle range [−2π, 2π]
        // Note: This is deterministic - same position always yields same wander target
        // This prevents creatures from getting stuck but makes behavior predictable
        let angle = (current_pos.x * 0.1 + current_pos.y * 0.2).sin() * std::f32::consts::TAU;

        // Convert angle to unit vector and scale by wander distance
        // cos(angle) gives x component, sin(angle) gives y component of direction
        let offset = Vec2::new(angle.cos(), angle.sin()) * WANDER_DISTANCE;
        current_pos + offset
    }

    fn calculate_search_target(current_pos: Vec2, resource_type: ResourceType) -> Vec2 {
        // Different search patterns for different resources
        let angle_offset = match resource_type {
            ResourceType::Food => 0.0,
            ResourceType::Water => std::f32::consts::PI,
        };

        let angle = (current_pos.x * 0.15).sin() * std::f32::consts::TAU + angle_offset;
        let offset = Vec2::new(angle.cos(), angle.sin()) * SEARCH_DISTANCE;
        current_pos + offset
    }
}

/// Gather decision contexts from the world state
/// This function extracts all necessary information for decision-making from the world
/// without passing the world reference to decision functions (pure function pattern)
pub fn gather_decision_contexts(world: &crate::core::World) -> Vec<DecisionContext> {
    let mut contexts = Vec::new();

    // Iterate through all creatures in the world
    for (entity, creature) in world.creatures.iter() {
        // Skip dead creatures to avoid unnecessary processing
        if !creature.is_alive() {
            continue;
        }

        let position = creature.position;

        // Gather nearby entities using spatial grid for O(log n) performance
        // The spatial grid divides the world into cells for efficient proximity queries
        let nearby_entities = world.spatial_grid.query_radius(position, NEARBY_RESOURCE_RADIUS);
        let mut nearby_resources = Vec::new();

        // Filter nearby entities to find resources and calculate distances
        for nearby_entity in nearby_entities {
            if let Some(resource) = world.resources.get(&nearby_entity) {
                // Calculate Euclidean distance for accurate proximity assessment
                let distance = (resource.position - position).length();
                nearby_resources.push(ResourceInfo {
                    entity: nearby_entity,
                    position: resource.position,
                    resource_type: resource.resource_type,
                    amount: resource.amount,
                    distance,
                });
            }
        }

        // Build the decision context with all relevant creature state
        // This decouples decision-making from direct world access
        let context = DecisionContext {
            entity: *entity,
            position,
            velocity: creature.velocity,
            state: creature.state.clone(),
            needs: NeedState {
                hunger: creature.needs.hunger,
                thirst: creature.needs.thirst,
                energy: creature.needs.energy,
                social: 0.0, // Not implemented yet
            },
            // Normalize health to 0-1 range for consistent decision-making
            health: creature.health.current / creature.health.max,
            energy: creature.needs.energy, // Note: This duplicates needs.energy - possible redundancy
            nearby_resources,
            nearby_creatures: vec![], // TODO: Implement creature detection
            nearby_threats: vec![],   // TODO: Implement threat detection
            time_since_last_decision: 0.0, // TODO: Track this for smarter caching
        };

        contexts.push(context);
    }

    contexts
}

/// Execute decisions in the world
pub fn execute_decisions(decisions: Vec<(Entity, Decision)>, world: &mut crate::core::World) {
    for (entity, decision) in decisions {
        if let Some(creature) = world.creatures.get_mut(&entity) {
            match decision {
                Decision::Move { target, urgency: _ } => {
                    creature.start_moving(target);
                },
                Decision::Consume {
                    resource: _,
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
                    let flee_target = creature.position + direction * FLEE_DISTANCE;
                    creature.start_moving(flee_target);
                },
                Decision::Idle => {
                    // Do nothing
                },
            }
        }
    }
}

/// The main decoupled decision system
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

impl DecoupledDecisionSystem {
    /// Update the decision system
    pub fn update(&mut self, world: &mut crate::core::World, current_time: f32) {
        // Only update at the configured interval
        if current_time - self.last_update < self.decision_interval {
            return;
        }

        self.last_update = current_time;

        // Gather contexts
        let contexts = gather_decision_contexts(world);

        // Make decisions (can be parallelized with rayon)
        let decisions: Vec<_> = contexts
            .into_iter()
            .filter_map(|context| {
                // Check cache first
                if let Some(cached) = self.cache.get(&context, current_time) {
                    return Some((context.entity, cached));
                }

                // Make new decision
                let decision = decision_functions::make_decision(&context);

                // Cache the result
                self.cache.insert(&context, decision.clone(), current_time);

                Some((context.entity, decision))
            })
            .collect();

        // Execute decisions
        execute_decisions(decisions, world);
    }
}

/// For backwards compatibility
pub struct DecoupledDecisionPlugin;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pure_decision_making() {
        let context = DecisionContext {
            entity: Entity::new(1),
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
                entity: Entity::new(2),
                position: Vec2::new(60.0, 50.0),
                resource_type: ResourceType::Food,
                amount: 50.0,
                distance: 1.5, // Within INTERACTION_DISTANCE (2.0)
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
            _ => panic!("Test failed: Expected Consume decision for hungry creature near food, but got {:?}", decision),
        }
    }

    #[test]
    fn test_threat_avoidance() {
        let context = DecisionContext {
            entity: Entity::new(1),
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
            _ => panic!(
                "Test failed: Expected Flee decision when near threat, but got {:?}",
                decision
            ),
        }
    }

    #[test]
    fn test_decision_caching() {
        let cache = DecisionCache::new(10);
        let context = DecisionContext {
            entity: Entity::new(1),
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
            entity: Entity::new(1),
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
                entity: Entity::new(2),
                position: Vec2::new(60.0, 50.0),
                resource_type: ResourceType::Water,
                amount: 50.0,
                distance: 1.5, // Within INTERACTION_DISTANCE (2.0)
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
            _ => panic!(
                "Test failed: Expected Consume water decision for thirsty creature, but got {:?}",
                decision
            ),
        }
    }

    #[test]
    fn test_social_interaction() {
        let context = DecisionContext {
            entity: Entity::new(1),
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
                entity: Entity::new(2),
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
            _ => panic!("Test failed: Expected Socialize decision for lonely creature near friend, but got {:?}", decision),
        }
    }

    #[test]
    fn test_rest_decision() {
        let context = DecisionContext {
            entity: Entity::new(1),
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
            _ => panic!(
                "Test failed: Expected Rest decision for tired creature, but got {:?}",
                decision
            ),
        }
    }

    #[test]
    fn test_wander_decision() {
        let context = DecisionContext {
            entity: Entity::new(1),
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
            _ => panic!("Test failed: Expected Move or Idle for creature with no urgent needs, but got {:?}", decision),
        }
    }

    #[test]
    fn test_search_decision() {
        let context = DecisionContext {
            entity: Entity::new(1),
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
            _ => panic!(
                "Test failed: Expected Move decision to search for food, but got {:?}",
                decision
            ),
        }
    }

    #[test]
    fn test_empty_nearby_lists() {
        let context = DecisionContext {
            entity: Entity::new(1),
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
            _ => panic!(
                "Test failed: Expected Rest decision for exhausted creature, but got {:?}",
                decision
            ),
        }
    }

    #[test]
    fn test_decision_cache_hash_collision() {
        let cache = DecisionCache::new(10);

        // Create two different contexts
        let context1 = DecisionContext {
            entity: Entity::new(1),
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
            _ => panic!("Test failed: Wrong decision cached. Expected second decision to match first, but got different results"),
        }
    }
}
