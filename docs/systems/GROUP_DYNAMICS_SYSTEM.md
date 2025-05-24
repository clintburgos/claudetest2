# Group Dynamics and Formation Rules

## Overview

The group dynamics system governs how creatures form, maintain, and dissolve social groups. It handles leadership, roles, collective behavior, and group decision-making while maintaining individual autonomy.

## Group Formation

### Formation Triggers

```rust
pub enum GroupFormationTrigger {
    // Safety in numbers
    PredatorThreat {
        threat_level: f32,
        min_members: u32,
    },
    
    // Resource efficiency  
    ResourceSharing {
        resource_type: ResourceType,
        abundance_threshold: f32,
    },
    
    // Social bonding
    HighRelationships {
        min_relationship_strength: f32,
        min_connections: u32,
    },
    
    // Migration
    SeasonalMovement {
        destination: BiomeType,
        season: Season,
    },
    
    // Reproduction
    MatingGroup {
        gender_ratio: (u32, u32),
        min_genetic_diversity: f32,
    },
    
    // Cultural
    SharedKnowledge {
        knowledge_type: KnowledgeType,
        min_overlap: f32,
    },
}

pub struct GroupFormationRules {
    pub min_size: u32,
    pub max_size: u32,
    pub formation_radius: f32,
    pub compatibility_threshold: f32,
    pub triggers: Vec<GroupFormationTrigger>,
}

impl GroupFormationRules {
    pub fn can_form_group(&self, candidates: &[Entity], world: &World) -> Option<GroupProposal> {
        if candidates.len() < self.min_size as usize {
            return None;
        }
        
        // Check triggers
        let active_trigger = self.triggers.iter()
            .find(|trigger| trigger.is_active(candidates, world))?;
            
        // Check compatibility
        let compatibility_matrix = self.calculate_compatibility(candidates, world);
        let avg_compatibility = compatibility_matrix.average();
        
        if avg_compatibility < self.compatibility_threshold {
            return None;
        }
        
        // Select leader
        let leader = self.select_leader(candidates, world, &active_trigger);
        
        // Filter final members
        let members = self.filter_members(candidates, leader, world);
        
        if members.len() >= self.min_size as usize {
            Some(GroupProposal {
                leader,
                members,
                group_type: active_trigger.group_type(),
                formation_reason: active_trigger.clone(),
            })
        } else {
            None
        }
    }
}
```

### Compatibility Calculation

```rust
pub struct CompatibilityCalculator {
    weights: CompatibilityWeights,
}

pub struct CompatibilityWeights {
    pub genetics: f32,
    pub personality: f32,
    pub relationships: f32,
    pub goals: f32,
    pub culture: f32,
}

impl CompatibilityCalculator {
    pub fn calculate(&self, a: Entity, b: Entity, world: &World) -> f32 {
        let mut score = 0.0;
        
        // Genetic compatibility
        if let (Some(gen_a), Some(gen_b)) = (
            world.get::<Genetics>(a),
            world.get::<Genetics>(b)
        ) {
            score += self.genetic_compatibility(gen_a, gen_b) * self.weights.genetics;
        }
        
        // Personality compatibility
        if let (Some(pers_a), Some(pers_b)) = (
            world.get::<Personality>(a),
            world.get::<Personality>(b)
        ) {
            score += self.personality_compatibility(pers_a, pers_b) * self.weights.personality;
        }
        
        // Existing relationships
        if let Some(social) = world.get::<SocialComponent>(a) {
            if let Some(relationship) = social.get_relationship(b) {
                score += relationship.strength * self.weights.relationships;
            }
        }
        
        // Shared goals
        score += self.shared_goals_score(a, b, world) * self.weights.goals;
        
        // Cultural similarity
        score += self.cultural_similarity(a, b, world) * self.weights.culture;
        
        score.clamp(0.0, 1.0)
    }
    
    fn personality_compatibility(&self, a: &Personality, b: &Personality) -> f32 {
        // Complementary traits
        let leadership_balance = 1.0 - (a.leadership - b.leadership).abs();
        let aggression_similarity = 1.0 - (a.aggression - b.aggression).abs();
        let social_match = 1.0 - (a.sociability - b.sociability).abs();
        
        (leadership_balance + aggression_similarity + social_match) / 3.0
    }
}
```

## Group Structure

### Group Hierarchy

```rust
pub struct Group {
    pub id: GroupId,
    pub group_type: GroupType,
    pub formation_time: f32,
    
    // Members
    pub leader: Entity,
    pub members: HashSet<Entity>,
    pub roles: HashMap<Entity, GroupRole>,
    
    // Structure
    pub hierarchy: GroupHierarchy,
    pub subgroups: Vec<SubGroup>,
    
    // State
    pub cohesion: f32,
    pub stability: f32,
    pub shared_resources: ResourcePool,
    pub collective_knowledge: KnowledgeBase,
    
    // Behavior
    pub decision_style: DecisionStyle,
    pub movement_formation: MovementFormation,
    pub conflict_resolution: ConflictResolution,
}

#[derive(Debug, Clone)]
pub enum GroupType {
    Family,
    Pack,
    Herd,
    Flock,
    Tribe,
    Coalition,
    MatingGroup,
    MigrationGroup,
}

#[derive(Debug, Clone)]
pub enum GroupRole {
    Leader,
    Scout,
    Protector,
    Forager,
    Caregiver,
    Teacher,
    Member,
}

pub struct GroupHierarchy {
    pub ranks: Vec<HierarchyLevel>,
    pub succession_rules: SuccessionRules,
}

pub struct HierarchyLevel {
    pub rank: u32,
    pub members: Vec<Entity>,
    pub privileges: Vec<Privilege>,
    pub responsibilities: Vec<Responsibility>,
}
```

### Leadership

```rust
pub struct LeadershipSystem {
    pub selection_method: LeaderSelectionMethod,
    pub leadership_traits: LeadershipTraits,
    pub succession_rules: SuccessionRules,
}

pub enum LeaderSelectionMethod {
    // Trait-based
    Strongest,
    Smartest,
    MostExperienced,
    MostSocial,
    
    // Consensus-based
    Democratic,
    Elders,
    
    // Combat-based
    Challenge,
    Tournament,
    
    // Inheritance
    Hereditary,
    Appointed,
}

pub struct LeadershipTraits {
    pub min_age: f32,
    pub required_traits: Vec<(TraitType, f32)>,
    pub experience_requirements: Vec<ExperienceType>,
    pub social_requirements: Option<SocialRequirements>,
}

impl LeadershipSystem {
    pub fn select_leader(&self, candidates: &[Entity], world: &World) -> Entity {
        match self.selection_method {
            LeaderSelectionMethod::Strongest => {
                candidates.iter()
                    .max_by_key(|&&e| {
                        world.get::<Strength>(e)
                            .map(|s| OrderedFloat(s.value))
                            .unwrap_or(OrderedFloat(0.0))
                    })
                    .copied()
                    .unwrap()
            }
            LeaderSelectionMethod::Democratic => {
                self.hold_election(candidates, world)
            }
            LeaderSelectionMethod::Challenge => {
                self.resolve_challenges(candidates, world)
            }
            // ... other methods
        }
    }
    
    fn hold_election(&self, candidates: &[Entity], world: &World) -> Entity {
        let mut votes: HashMap<Entity, u32> = HashMap::new();
        
        for &voter in candidates {
            if let Some(choice) = self.get_vote(voter, candidates, world) {
                *votes.entry(choice).or_default() += 1;
            }
        }
        
        votes.into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(entity, _)| entity)
            .unwrap()
    }
}
```

## Group Behaviors

### Collective Decision Making

```rust
pub enum DecisionStyle {
    // Leader decides
    Autocratic,
    
    // Leader consults then decides
    Consultative,
    
    // Majority vote
    Democratic { quorum: f32 },
    
    // Full agreement needed
    Consensus,
    
    // Weighted by rank/experience
    Weighted { weight_fn: WeightFunction },
}

pub struct GroupDecision {
    pub decision_type: DecisionType,
    pub proposer: Entity,
    pub options: Vec<DecisionOption>,
    pub votes: HashMap<Entity, usize>,
    pub deadline: f32,
}

pub enum DecisionType {
    Movement { destination: Vec3 },
    Resource { action: ResourceAction },
    Combat { target: Entity, strategy: CombatStrategy },
    Social { action: SocialAction },
    Dissolution,
}

impl Group {
    pub fn make_decision(&mut self, decision: GroupDecision, world: &World) -> DecisionOutcome {
        match self.decision_style {
            DecisionStyle::Autocratic => {
                let leader_choice = self.get_leader_preference(&decision, world);
                DecisionOutcome::Decided(leader_choice)
            }
            DecisionStyle::Democratic { quorum } => {
                let participation = decision.votes.len() as f32 / self.members.len() as f32;
                if participation >= quorum {
                    let winner = decision.get_majority();
                    DecisionOutcome::Decided(winner)
                } else {
                    DecisionOutcome::InsufficientParticipation
                }
            }
            DecisionStyle::Consensus => {
                if decision.has_unanimous_agreement() {
                    DecisionOutcome::Decided(decision.options[0].clone())
                } else {
                    DecisionOutcome::NoConsensus
                }
            }
            // ... other styles
        }
    }
}
```

### Movement Formation

```rust
pub enum MovementFormation {
    // Protective formations
    Circle { radius: f32 },
    Wedge { angle: f32 },
    Line { spacing: f32 },
    
    // Efficient formations
    Diamond,
    Square,
    Column { width: u32 },
    
    // Natural formations
    Flock { separation: f32, cohesion: f32, alignment: f32 },
    School { neighbor_distance: f32 },
    Herd { spread: f32 },
    
    // Tactical formations
    Escort { protected: Vec<Entity> },
    Ambush { positions: Vec<Vec3> },
}

pub struct FormationManager {
    pub formation: MovementFormation,
    pub positions: HashMap<Entity, FormationPosition>,
    pub flexibility: f32,
}

pub struct FormationPosition {
    pub ideal_offset: Vec3,
    pub role: FormationRole,
    pub priority: u32,
}

impl FormationManager {
    pub fn calculate_positions(&self, group: &Group, leader_pos: Vec3, leader_dir: Vec3) -> HashMap<Entity, Vec3> {
        let mut positions = HashMap::new();
        
        match &self.formation {
            MovementFormation::Wedge { angle } => {
                let mut row = 0;
                let mut position_in_row = 0;
                
                for (i, &member) in group.members.iter().enumerate() {
                    if member == group.leader {
                        positions.insert(member, leader_pos);
                        continue;
                    }
                    
                    let lateral_offset = (position_in_row as f32 - row as f32 / 2.0) * 2.0;
                    let forward_offset = -(row as f32) * 3.0;
                    
                    let offset = leader_dir * forward_offset + 
                               leader_dir.cross(Vec3::Y).normalize() * lateral_offset;
                    
                    positions.insert(member, leader_pos + offset);
                    
                    position_in_row += 1;
                    if position_in_row > row {
                        row += 1;
                        position_in_row = 0;
                    }
                }
            }
            // ... other formations
        }
        
        positions
    }
}
```

### Group Cohesion

```rust
pub struct CohesionSystem {
    pub base_cohesion: f32,
    pub factors: CohesionFactors,
    pub thresholds: CohesionThresholds,
}

pub struct CohesionFactors {
    pub shared_experiences: f32,
    pub successful_cooperation: f32,
    pub internal_conflict: f32,
    pub external_threats: f32,
    pub resource_abundance: f32,
    pub leadership_quality: f32,
}

pub struct CohesionThresholds {
    pub dissolution: f32,      // Below this, group dissolves
    pub unstable: f32,         // Members may leave
    pub stable: f32,           // Normal functioning
    pub strong: f32,           // Bonus cooperation
}

impl CohesionSystem {
    pub fn update_cohesion(&mut self, group: &mut Group, world: &World, dt: f32) {
        let mut cohesion_delta = 0.0;
        
        // Positive factors
        cohesion_delta += self.calculate_shared_experience_bonus(group, world);
        cohesion_delta += self.calculate_success_bonus(group);
        cohesion_delta += self.calculate_threat_bonus(group, world);
        
        // Negative factors
        cohesion_delta -= self.calculate_conflict_penalty(group);
        cohesion_delta -= self.calculate_resource_stress(group, world);
        cohesion_delta -= self.calculate_size_penalty(group);
        
        // Apply leader influence
        let leader_modifier = self.calculate_leader_influence(group, world);
        cohesion_delta *= leader_modifier;
        
        // Update cohesion
        group.cohesion += cohesion_delta * dt;
        group.cohesion = group.cohesion.clamp(0.0, 1.0);
        
        // Check thresholds
        if group.cohesion < self.thresholds.dissolution {
            self.trigger_dissolution(group);
        } else if group.cohesion < self.thresholds.unstable {
            self.check_member_loyalty(group, world);
        }
    }
}
```

## Group Dissolution

```rust
pub enum DissolutionReason {
    LowCohesion,
    LeaderDeath,
    ResourceDepletion,
    InternalConflict,
    ExternalPressure,
    GoalAchieved,
    MergerWithLargerGroup,
}

pub struct DissolutionHandler {
    pub dissolution_rules: Vec<DissolutionRule>,
    pub aftermath_handlers: HashMap<DissolutionReason, Box<dyn AftermathHandler>>,
}

pub struct DissolutionRule {
    pub condition: DissolutionCondition,
    pub reason: DissolutionReason,
    pub can_prevent: bool,
}

pub enum DissolutionCondition {
    CohesionBelow(f32),
    SizeBelow(u32),
    LeaderMissing,
    MajorityVote,
    TimeElapsed(f32),
}

impl DissolutionHandler {
    pub fn check_dissolution(&self, group: &Group, world: &World) -> Option<DissolutionReason> {
        for rule in &self.dissolution_rules {
            if rule.condition.is_met(group, world) {
                if rule.can_prevent && self.attempt_prevention(group, world) {
                    continue;
                }
                return Some(rule.reason.clone());
            }
        }
        None
    }
    
    pub fn dissolve_group(&mut self, group: Group, reason: DissolutionReason, world: &mut World) {
        // Record dissolution event
        world.send_event(GroupEvent::Dissolved {
            group_id: group.id,
            reason: reason.clone(),
            final_members: group.members.clone(),
        });
        
        // Handle aftermath
        if let Some(handler) = self.aftermath_handlers.get(&reason) {
            handler.handle_aftermath(group, world);
        }
        
        // Default aftermath
        self.default_aftermath(group, world);
    }
    
    fn default_aftermath(&self, group: Group, world: &mut World) {
        // Distribute shared resources
        if !group.shared_resources.is_empty() {
            let share = group.shared_resources.total() / group.members.len() as f32;
            for &member in &group.members {
                if let Some(mut inventory) = world.get_mut::<Inventory>(member) {
                    inventory.add_resources(share);
                }
            }
        }
        
        // Update relationships
        for &member_a in &group.members {
            for &member_b in &group.members {
                if member_a != member_b {
                    if let Some(mut social) = world.get_mut::<SocialComponent>(member_a) {
                        social.update_relationship(member_b, RelationshipChange::GroupDissolved);
                    }
                }
            }
        }
        
        // Clear group references
        for &member in &group.members {
            if let Some(mut creature) = world.get_mut::<Creature>(member) {
                creature.group = None;
            }
        }
    }
}
```

## Subgroups and Factions

```rust
pub struct SubGroup {
    pub id: SubGroupId,
    pub parent_group: GroupId,
    pub members: HashSet<Entity>,
    pub purpose: SubGroupPurpose,
    pub autonomy_level: f32,
}

pub enum SubGroupPurpose {
    Foraging,
    Scouting,
    Defense,
    Hunting,
    Childcare,
    Faction { ideology: FactionIdeology },
}

pub struct FactionSystem {
    pub faction_formation_threshold: f32,
    pub conflict_resolution: ConflictResolution,
}

pub enum FactionIdeology {
    Conservative,  // Maintain traditions
    Progressive,   // Embrace change
    Aggressive,    // Expand territory
    Peaceful,      // Avoid conflict
    Isolationist,  // Minimize outside contact
    Cooperative,   // Maximize alliances
}

impl Group {
    pub fn check_faction_formation(&self, world: &World) -> Option<Vec<SubGroup>> {
        let ideological_clusters = self.cluster_by_ideology(world);
        
        let factions: Vec<SubGroup> = ideological_clusters
            .into_iter()
            .filter(|(_, members)| {
                members.len() >= 3 && 
                members.len() as f32 / self.members.len() as f32 > 0.2
            })
            .map(|(ideology, members)| SubGroup {
                id: SubGroupId::new(),
                parent_group: self.id,
                members: members.into_iter().collect(),
                purpose: SubGroupPurpose::Faction { ideology },
                autonomy_level: 0.5,
            })
            .collect();
            
        if !factions.is_empty() {
            Some(factions)
        } else {
            None
        }
    }
}
```

## Integration Example

```rust
pub fn update_group_dynamics(
    mut groups: Query<&mut Group>,
    creatures: Query<(&Creature, &Position, &SocialComponent)>,
    spatial_index: Res<SpatialIndex>,
    time: Res<Time>,
    mut events: EventWriter<GroupEvent>,
) {
    // Check for new group formation
    for (entity, pos, social) in creatures.iter() {
        if social.group.is_none() && social.desires_group() {
            let nearby = spatial_index.query_range(pos.0, 20.0);
            let candidates: Vec<_> = nearby.into_iter()
                .filter(|&e| {
                    creatures.get(e)
                        .map(|(c, _, _)| c.group.is_none())
                        .unwrap_or(false)
                })
                .collect();
                
            if let Some(proposal) = GROUP_FORMATION_RULES.can_form_group(&candidates, &world) {
                create_group(proposal, &mut events);
            }
        }
    }
    
    // Update existing groups
    for mut group in groups.iter_mut() {
        // Update cohesion
        COHESION_SYSTEM.update_cohesion(&mut group, &world, time.delta_seconds());
        
        // Check dissolution
        if let Some(reason) = DISSOLUTION_HANDLER.check_dissolution(&group, &world) {
            events.send(GroupEvent::Dissolving {
                group_id: group.id,
                reason,
            });
        }
        
        // Update formations if moving
        if group.is_moving() {
            let positions = group.formation_manager.calculate_positions(
                &group,
                group.get_leader_position(),
                group.get_movement_direction()
            );
            
            // Apply positions with some flexibility
            for (member, target_pos) in positions {
                if let Ok((_, mut pos, _)) = creatures.get_mut(member) {
                    pos.0 = pos.0.lerp(target_pos, 0.1);
                }
            }
        }
    }
}
```

This system creates rich, dynamic group behaviors with realistic formation, maintenance, and dissolution mechanics.