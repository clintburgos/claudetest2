# Group Dynamics Implementation Design

## Overview

The group dynamics system enables creatures to form social groups, make collective decisions, establish hierarchies, and coordinate activities. This creates emergent social behaviors and complex inter-group interactions.

## Group Formation

### Formation Triggers
```rust
enum GroupFormationTrigger {
    MutualBenefit {
        shared_goal: Goal,
        min_participants: usize,
    },
    FamilyBond {
        relationship_threshold: f32,
    },
    SafetyInNumbers {
        threat_level: f32,
        proximity_radius: f32,
    },
    ResourceSharing {
        resource_scarcity: f32,
        cooperation_benefit: f32,
    },
    SocialAffinity {
        compatibility_score: f32,
        interaction_history: u32,
    },
}

struct GroupFormationSystem {
    potential_groups: HashMap<GroupSeed, Vec<CreatureId>>,
    formation_threshold: f32,
    max_group_size: usize,
}

impl GroupFormationSystem {
    fn evaluate_group_formation(
        &mut self,
        creatures: &[Creature],
        trigger: GroupFormationTrigger,
    ) -> Option<Group> {
        match trigger {
            GroupFormationTrigger::MutualBenefit { shared_goal, min_participants } => {
                let interested = self.find_creatures_with_goal(creatures, &shared_goal);
                
                if interested.len() >= min_participants {
                    let compatibility = self.calculate_group_compatibility(&interested);
                    
                    if compatibility > self.formation_threshold {
                        return Some(self.form_group(interested, GroupType::GoalOriented));
                    }
                }
            },
            GroupFormationTrigger::FamilyBond { relationship_threshold } => {
                let families = self.identify_family_clusters(creatures, relationship_threshold);
                
                for family in families {
                    if family.len() >= 2 {
                        return Some(self.form_group(family, GroupType::Family));
                    }
                }
            },
            GroupFormationTrigger::SafetyInNumbers { threat_level, proximity_radius } => {
                if threat_level > 0.7 {
                    let nearby = self.find_creatures_in_radius(creatures, proximity_radius);
                    
                    if nearby.len() >= 3 {
                        return Some(self.form_group(nearby, GroupType::Defensive));
                    }
                }
            },
            // ... other triggers
        }
        
        None
    }
    
    fn calculate_group_compatibility(&self, creatures: &[&Creature]) -> f32 {
        let mut total_compatibility = 0.0;
        let mut pair_count = 0;
        
        for i in 0..creatures.len() {
            for j in i+1..creatures.len() {
                let compatibility = self.calculate_pair_compatibility(
                    creatures[i],
                    creatures[j]
                );
                total_compatibility += compatibility;
                pair_count += 1;
            }
        }
        
        if pair_count > 0 {
            total_compatibility / pair_count as f32
        } else {
            0.0
        }
    }
}
```

### Group Structure
```rust
#[derive(Clone)]
struct Group {
    id: GroupId,
    group_type: GroupType,
    members: HashMap<CreatureId, GroupRole>,
    leader: Option<CreatureId>,
    formation_time: SimTime,
    cohesion: f32,
    shared_knowledge: HashSet<ConceptId>,
    group_decisions: VecDeque<GroupDecision>,
    dissolution_pressure: f32,
}

#[derive(Clone, PartialEq)]
enum GroupType {
    Family,
    Foraging,
    Defensive,
    Migratory,
    Social,
    GoalOriented,
}

#[derive(Clone)]
enum GroupRole {
    Leader,
    Scout,
    Protector,
    Forager,
    Caregiver,
    Member, // Default
}

impl Group {
    fn add_member(&mut self, creature_id: CreatureId, role: GroupRole) {
        self.members.insert(creature_id, role);
        self.recalculate_cohesion();
    }
    
    fn remove_member(&mut self, creature_id: CreatureId) -> bool {
        self.members.remove(&creature_id);
        
        // Check if group should dissolve
        if self.members.len() < 2 {
            return true; // Dissolve
        }
        
        // Replace leader if necessary
        if Some(creature_id) == self.leader {
            self.elect_new_leader();
        }
        
        self.recalculate_cohesion();
        false
    }
    
    fn recalculate_cohesion(&mut self) {
        // Cohesion based on relationships and shared experiences
        let relationship_sum: f32 = self.members.keys()
            .combinations(2)
            .map(|pair| get_relationship(pair[0], pair[1]))
            .sum();
            
        let pair_count = (self.members.len() * (self.members.len() - 1)) / 2;
        let avg_relationship = if pair_count > 0 {
            relationship_sum / pair_count as f32
        } else {
            0.0
        };
        
        let time_bonus = (self.get_age().as_hours() as f32 / 168.0).min(0.2); // Max 0.2 after a week
        
        self.cohesion = (avg_relationship + time_bonus).clamp(0.0, 1.0);
    }
}
```

## Leadership Selection

### Leadership Traits
```rust
struct LeadershipTraits {
    decisiveness: f32,
    empathy: f32,
    experience: f32,
    communication: f32,
    stress_tolerance: f32,
}

impl LeadershipTraits {
    fn from_creature(creature: &Creature) -> Self {
        LeadershipTraits {
            decisiveness: creature.personality.decisiveness,
            empathy: creature.personality.empathy,
            experience: (creature.age.current_age / creature.age.max_lifespan).min(1.0),
            communication: creature.get_ability_proficiency(AbilityType::Communication),
            stress_tolerance: 1.0 - creature.stress_sensitivity,
        }
    }
    
    fn calculate_leadership_score(&self, group_type: &GroupType) -> f32 {
        match group_type {
            GroupType::Family => {
                self.empathy * 0.4 + 
                self.experience * 0.3 + 
                self.communication * 0.3
            },
            GroupType::Defensive => {
                self.decisiveness * 0.4 + 
                self.stress_tolerance * 0.4 + 
                self.experience * 0.2
            },
            GroupType::Foraging => {
                self.experience * 0.4 + 
                self.communication * 0.3 + 
                self.decisiveness * 0.3
            },
            _ => {
                // Balanced scoring for other types
                (self.decisiveness + self.empathy + self.experience + 
                 self.communication + self.stress_tolerance) / 5.0
            }
        }
    }
}
```

### Leadership Election
```rust
impl Group {
    fn elect_new_leader(&mut self) {
        let mut candidates: Vec<(CreatureId, f32)> = Vec::new();
        
        for (member_id, role) in &self.members {
            let creature = get_creature(*member_id);
            let traits = LeadershipTraits::from_creature(&creature);
            let score = traits.calculate_leadership_score(&self.group_type);
            
            // Modify score based on current role
            let role_modifier = match role {
                GroupRole::Leader => 0.8, // Slight penalty for failed leaders
                GroupRole::Scout => 1.1,   // Scouts make good leaders
                GroupRole::Protector => 1.1,
                _ => 1.0,
            };
            
            candidates.push((*member_id, score * role_modifier));
        }
        
        // Sort by score
        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        if let Some((new_leader_id, _)) = candidates.first() {
            self.leader = Some(*new_leader_id);
            self.members.insert(*new_leader_id, GroupRole::Leader);
            
            // Notify group of leadership change
            self.broadcast_leadership_change(*new_leader_id);
        }
    }
    
    fn challenge_leadership(&mut self, challenger_id: CreatureId) -> bool {
        if let Some(current_leader_id) = self.leader {
            let challenger = get_creature(challenger_id);
            let leader = get_creature(current_leader_id);
            
            let challenger_score = LeadershipTraits::from_creature(&challenger)
                .calculate_leadership_score(&self.group_type);
            let leader_score = LeadershipTraits::from_creature(&leader)
                .calculate_leadership_score(&self.group_type);
            
            // Need significant advantage to overthrow
            if challenger_score > leader_score * 1.2 {
                self.leader = Some(challenger_id);
                self.members.insert(challenger_id, GroupRole::Leader);
                self.members.insert(current_leader_id, GroupRole::Member);
                
                // Cohesion penalty for leadership struggle
                self.cohesion *= 0.8;
                
                return true;
            }
        }
        
        false
    }
}
```

## Group Decision Making

### Collective Decision Process
```rust
#[derive(Clone)]
struct GroupDecision {
    decision_type: GroupDecisionType,
    options: Vec<DecisionOption>,
    votes: HashMap<CreatureId, usize>, // Member -> option index
    leader_preference: Option<usize>,
    decision_time: SimTime,
    urgency: f32,
}

#[derive(Clone)]
enum GroupDecisionType {
    Movement { destination: Vec2 },
    Activity { activity: GroupActivity },
    Resource { resource_location: Vec2 },
    Conflict { response: ConflictResponse },
    Dissolution,
}

#[derive(Clone)]
struct DecisionOption {
    description: String,
    proposer: CreatureId,
    benefit_estimate: f32,
    risk_estimate: f32,
}

impl Group {
    fn initiate_decision(
        &mut self,
        decision_type: GroupDecisionType,
        proposer: CreatureId,
        urgency: f32,
    ) {
        let decision = GroupDecision {
            decision_type,
            options: Vec::new(),
            votes: HashMap::new(),
            leader_preference: None,
            decision_time: current_time(),
            urgency,
        };
        
        self.group_decisions.push_back(decision);
        
        // Notify members
        self.broadcast_decision_needed();
    }
    
    fn vote_on_decision(&mut self, member_id: CreatureId, option_index: usize) {
        if let Some(decision) = self.group_decisions.back_mut() {
            decision.votes.insert(member_id, option_index);
            
            // Check if all have voted
            if decision.votes.len() == self.members.len() {
                self.finalize_decision();
            }
        }
    }
    
    fn finalize_decision(&mut self) {
        if let Some(mut decision) = self.group_decisions.pop_back() {
            // Count votes
            let mut vote_counts: HashMap<usize, usize> = HashMap::new();
            for (_, option_index) in &decision.votes {
                *vote_counts.entry(*option_index).or_insert(0) += 1;
            }
            
            // Find winner
            let winning_option = if let Some(leader_pref) = decision.leader_preference {
                // Leader influence based on cohesion
                let leader_weight = 1.0 + self.cohesion;
                *vote_counts.entry(leader_pref).or_insert(0) += leader_weight as usize;
                leader_pref
            } else {
                // Simple majority
                vote_counts.iter()
                    .max_by_key(|(_, count)| *count)
                    .map(|(option, _)| *option)
                    .unwrap_or(0)
            };
            
            // Execute decision
            self.execute_group_decision(&decision.decision_type, winning_option);
        }
    }
}
```

### Decision Execution
```rust
enum GroupActivity {
    Foraging { area: Vec2, radius: f32 },
    Moving { destination: Vec2, formation: Formation },
    Resting { location: Vec2 },
    Defending { threat: ThreatInfo },
    Socializing,
}

impl Group {
    fn execute_group_decision(
        &mut self,
        decision_type: &GroupDecisionType,
        chosen_option: usize,
    ) {
        match decision_type {
            GroupDecisionType::Movement { destination } => {
                self.initiate_group_movement(*destination);
            },
            GroupDecisionType::Activity { activity } => {
                self.set_group_activity(activity.clone());
            },
            GroupDecisionType::Dissolution => {
                if chosen_option == 1 { // Yes to dissolution
                    self.dissolution_pressure = 1.0;
                }
            },
            // ... other decision types
        }
        
        // Update cohesion based on decision consensus
        let consensus_rate = self.calculate_consensus_rate(chosen_option);
        self.cohesion = (self.cohesion * 0.9 + consensus_rate * 0.1).clamp(0.0, 1.0);
    }
    
    fn calculate_consensus_rate(&self, chosen_option: usize) -> f32 {
        if let Some(decision) = self.group_decisions.back() {
            let votes_for_winner = decision.votes.values()
                .filter(|&&v| v == chosen_option)
                .count();
                
            votes_for_winner as f32 / self.members.len() as f32
        } else {
            0.0
        }
    }
}
```

## Group Movement

### Formation Movement
```rust
struct GroupMovement {
    formation: Formation,
    destination: Option<Vec2>,
    movement_speed: f32,
    path: Option<Path>,
    stragglers: HashSet<CreatureId>,
}

impl GroupMovement {
    fn update_formation(
        &mut self,
        group: &Group,
        creatures: &mut HashMap<CreatureId, Creature>,
        delta_time: f32,
    ) {
        let leader_id = group.leader.unwrap_or(*group.members.keys().next().unwrap());
        let leader = &creatures[&leader_id];
        let leader_pos = leader.position;
        let leader_facing = leader.facing;
        
        // Calculate target positions for each member
        let mut member_targets: HashMap<CreatureId, Vec2> = HashMap::new();
        let members: Vec<_> = group.members.keys().collect();
        
        for (index, member_id) in members.iter().enumerate() {
            if **member_id == leader_id {
                continue; // Leader sets the pace
            }
            
            let target_pos = calculate_formation_position(
                index,
                &self.formation,
                leader_pos,
                leader_facing,
            );
            
            member_targets.insert(**member_id, target_pos);
        }
        
        // Move creatures toward formation positions
        for (member_id, target_pos) in member_targets {
            if let Some(creature) = creatures.get_mut(&member_id) {
                let to_target = target_pos - creature.position;
                let distance = to_target.length();
                
                if distance > FORMATION_TOLERANCE {
                    // Apply formation movement
                    let desired_velocity = to_target.normalize() * self.movement_speed;
                    creature.apply_group_movement(desired_velocity, delta_time);
                    
                    // Check for stragglers
                    if distance > STRAGGLER_DISTANCE {
                        self.stragglers.insert(member_id);
                    } else {
                        self.stragglers.remove(&member_id);
                    }
                }
            }
        }
        
        // Slow down if too many stragglers
        if self.stragglers.len() > group.members.len() / 3 {
            self.movement_speed *= 0.9;
        }
    }
}
```

## Group Size Management

### Splitting Mechanics
```rust
struct GroupSplitAnalyzer {
    max_size: usize,
    min_size: usize,
    split_threshold: f32,
}

impl GroupSplitAnalyzer {
    fn should_split(&self, group: &Group) -> Option<SplitStrategy> {
        // Size-based splitting
        if group.members.len() > self.max_size {
            return Some(SplitStrategy::SizeLimit);
        }
        
        // Cohesion-based splitting
        if group.cohesion < self.split_threshold {
            return Some(SplitStrategy::LowCohesion);
        }
        
        // Conflict-based splitting
        let factions = self.identify_factions(group);
        if factions.len() > 1 {
            return Some(SplitStrategy::Factions(factions));
        }
        
        None
    }
    
    fn execute_split(
        &self,
        group: &mut Group,
        strategy: SplitStrategy,
    ) -> Option<Group> {
        match strategy {
            SplitStrategy::SizeLimit => {
                // Split into two roughly equal groups
                let mut members: Vec<_> = group.members.keys().cloned().collect();
                members.shuffle(&mut thread_rng());
                
                let split_point = members.len() / 2;
                let new_members: HashMap<_, _> = members[split_point..]
                    .iter()
                    .map(|id| (*id, GroupRole::Member))
                    .collect();
                
                // Remove from original group
                for id in new_members.keys() {
                    group.members.remove(id);
                }
                
                // Create new group
                let mut new_group = Group::new(group.group_type.clone());
                new_group.members = new_members;
                new_group.elect_new_leader();
                
                Some(new_group)
            },
            SplitStrategy::Factions(factions) => {
                // Split along faction lines
                if factions.len() >= 2 {
                    let largest_faction = factions.iter()
                        .max_by_key(|f| f.len())
                        .unwrap();
                        
                    // Keep largest faction in original group
                    group.members.retain(|id, _| largest_faction.contains(id));
                    
                    // Create new group from second faction
                    let mut new_group = Group::new(group.group_type.clone());
                    new_group.members = factions[1].iter()
                        .map(|id| (*id, GroupRole::Member))
                        .collect();
                    new_group.elect_new_leader();
                    
                    Some(new_group)
                } else {
                    None
                }
            },
            _ => None,
        }
    }
}
```

### Merging Groups
```rust
impl GroupDynamicsSystem {
    fn evaluate_merge(
        &self,
        group1: &Group,
        group2: &Group,
    ) -> Option<MergeProposal> {
        // Check compatibility
        if group1.group_type != group2.group_type {
            return None;
        }
        
        // Check combined size
        if group1.members.len() + group2.members.len() > MAX_GROUP_SIZE {
            return None;
        }
        
        // Calculate merge benefit
        let relationship_score = self.calculate_inter_group_relationships(group1, group2);
        let resource_benefit = self.calculate_resource_sharing_benefit(group1, group2);
        let safety_benefit = self.calculate_safety_benefit(group1, group2);
        
        let total_benefit = relationship_score * 0.4 + 
                          resource_benefit * 0.3 + 
                          safety_benefit * 0.3;
                          
        if total_benefit > MERGE_THRESHOLD {
            Some(MergeProposal {
                benefit_score: total_benefit,
                leader_selection: self.propose_merged_leader(group1, group2),
                role_assignments: self.propose_role_redistribution(group1, group2),
            })
        } else {
            None
        }
    }
    
    fn execute_merge(
        &mut self,
        group1: &mut Group,
        group2: Group,
        proposal: MergeProposal,
    ) {
        // Combine members
        for (member_id, _) in group2.members {
            let new_role = proposal.role_assignments
                .get(&member_id)
                .cloned()
                .unwrap_or(GroupRole::Member);
            group1.add_member(member_id, new_role);
        }
        
        // Set new leader
        group1.leader = Some(proposal.leader_selection);
        
        // Combine knowledge
        group1.shared_knowledge.extend(group2.shared_knowledge);
        
        // Recalculate cohesion with penalty for recent merge
        group1.recalculate_cohesion();
        group1.cohesion *= 0.7; // Initial integration penalty
    }
}
```

## Inter-Group Dynamics

### Group Relationships
```rust
struct InterGroupRelation {
    group1_id: GroupId,
    group2_id: GroupId,
    relationship_type: GroupRelationType,
    intensity: f32,
    history: VecDeque<GroupInteraction>,
}

enum GroupRelationType {
    Alliance,
    Competition,
    Neutral,
    Hostile,
    Trade,
}

struct GroupInteraction {
    timestamp: SimTime,
    interaction_type: GroupInteractionType,
    outcome: InteractionOutcome,
    location: Vec2,
}

enum GroupInteractionType {
    ResourceCompetition,
    TerritorialDispute,
    CooperativeHunting,
    KnowledgeExchange,
    Conflict,
}
```

### Territory and Resources
```rust
struct GroupTerritory {
    group_id: GroupId,
    core_area: Circle,
    influence_areas: Vec<Circle>,
    resource_claims: HashSet<ResourceId>,
    patrol_routes: Vec<Path>,
}

impl GroupTerritory {
    fn calculate_overlap(&self, other: &GroupTerritory) -> f32 {
        let core_overlap = self.core_area.intersection_area(&other.core_area);
        let total_area = self.core_area.area();
        
        core_overlap / total_area
    }
    
    fn defend_territory(&self, intruder_pos: Vec2) -> DefenseResponse {
        let distance_to_core = self.core_area.distance_to_point(intruder_pos);
        
        match distance_to_core {
            d if d < 0.0 => DefenseResponse::Aggressive, // Inside core
            d if d < 50.0 => DefenseResponse::Warning,   // Near core
            _ => DefenseResponse::Monitor,               // In influence area
        }
    }
}
```