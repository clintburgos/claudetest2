# Combat & Conflict Resolution System Design

## Overview

A nuanced combat system that emphasizes realistic animal behavior, with options for both lethal and non-lethal conflict resolution. The system models territorial disputes, resource competition, predator-prey dynamics, and social dominance while avoiding gratuitous violence.

## Core Combat Components

```rust
pub struct CombatSystem {
    pub active_conflicts: HashMap<ConflictId, Conflict>,
    pub combat_resolver: CombatResolver,
    pub escape_system: EscapeSystem,
    pub injury_calculator: InjuryCalculator,
    pub threat_assessment: ThreatAssessment,
}

pub struct Conflict {
    pub id: ConflictId,
    pub participants: Vec<Combatant>,
    pub conflict_type: ConflictType,
    pub stakes: ConflictStakes,
    pub location: Vec3,
    pub duration: f32,
    pub intensity: f32,
    pub resolution: Option<ConflictResolution>,
}

pub struct Combatant {
    pub creature_id: EntityId,
    pub role: CombatRole,
    pub stance: CombatStance,
    pub stamina: f32,
    pub morale: f32,
    pub injuries_inflicted: Vec<EntityId>,
    pub injuries_received: Vec<Injury>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConflictType {
    Predation,
    TerritorialDispute,
    ResourceCompetition,
    MateCompetition,
    DominanceChallenge,
    DefenseOfYoung,
    PlayFighting,
}

#[derive(Debug, Clone)]
pub enum ConflictStakes {
    Life,                           // Predator-prey
    Territory(TerritoryBounds),     // Territorial dispute
    Resource(ResourceType, u32),    // Food, water, etc.
    MatingRights(EntityId),         // Competition for mate
    SocialRank(u32),              // Dominance hierarchy
    Protection(Vec<EntityId>),      // Protecting offspring/group
}

pub enum CombatStance {
    Aggressive,
    Defensive,
    Evasive,
    Intimidating,
    Submissive,
}
```

### Threat Assessment

```rust
pub struct ThreatAssessment {
    pub assessment_factors: AssessmentFactors,
    pub flight_threshold: f32,
    pub fight_threshold: f32,
}

pub struct ThreatProfile {
    pub threat_level: f32,          // 0.0 = harmless, 1.0 = lethal
    pub confidence: f32,            // 0.0 = certain defeat, 1.0 = certain victory
    pub escape_probability: f32,
    pub victory_probability: f32,
    pub injury_risk: f32,
    pub recommended_response: ThreatResponse,
}

pub enum ThreatResponse {
    Fight,
    Flee,
    Intimidate,
    Submit,
    PlayDead,
    CallForHelp,
    Negotiate,
}

impl ThreatAssessment {
    pub fn assess_threat(
        &self,
        assessor: &Creature,
        opponent: &Creature,
        context: &ConflictContext,
    ) -> ThreatProfile {
        let mut profile = ThreatProfile::default();
        
        // Size comparison
        let size_ratio = opponent.physical_stats.size / assessor.physical_stats.size;
        let size_factor = size_ratio.powf(2.0); // Square for more impact
        
        // Strength and health
        let strength_ratio = opponent.physical_stats.strength / assessor.physical_stats.strength;
        let health_ratio = opponent.health_system.current_health / assessor.health_system.current_health;
        
        // Experience and age
        let experience_factor = self.calculate_experience_factor(assessor, opponent);
        
        // Group support
        let ally_support = self.count_nearby_allies(assessor, &context.nearby_creatures);
        let enemy_support = self.count_nearby_allies(opponent, &context.nearby_creatures);
        let support_ratio = (ally_support + 1.0) / (enemy_support + 1.0);
        
        // Environmental advantages
        let terrain_advantage = self.calculate_terrain_advantage(
            assessor,
            opponent,
            &context.terrain
        );
        
        // Calculate threat level
        profile.threat_level = (size_factor * 0.3 + 
                               strength_ratio * 0.3 + 
                               health_ratio * 0.2 + 
                               experience_factor * 0.2)
                               .clamp(0.0, 1.0);
        
        // Calculate confidence
        profile.confidence = (support_ratio * 0.3 + 
                             terrain_advantage * 0.2 + 
                             assessor.emotional_state.confidence * 0.3 +
                             (1.0 / size_factor) * 0.2)
                             .clamp(0.0, 1.0);
        
        // Calculate probabilities
        profile.escape_probability = self.calculate_escape_chance(assessor, opponent, context);
        profile.victory_probability = profile.confidence * (1.0 - profile.threat_level);
        profile.injury_risk = profile.threat_level * (1.0 - profile.confidence);
        
        // Determine recommended response
        profile.recommended_response = self.determine_response(
            &profile,
            assessor,
            &context.conflict_type
        );
        
        profile
    }
    
    fn determine_response(
        &self,
        profile: &ThreatProfile,
        creature: &Creature,
        conflict_type: &ConflictType,
    ) -> ThreatResponse {
        // Special cases
        if matches!(conflict_type, ConflictType::DefenseOfYoung) {
            return ThreatResponse::Fight; // Always fight to protect young
        }
        
        if creature.physical_stats.energy < 10.0 {
            return ThreatResponse::Submit; // Too exhausted to fight or flee
        }
        
        // Normal threat response
        if profile.threat_level > self.flight_threshold && profile.escape_probability > 0.5 {
            ThreatResponse::Flee
        } else if profile.victory_probability > self.fight_threshold {
            ThreatResponse::Fight
        } else if profile.confidence > 0.6 && profile.threat_level < 0.7 {
            ThreatResponse::Intimidate
        } else if profile.threat_level > 0.8 {
            ThreatResponse::Submit
        } else {
            ThreatResponse::Negotiate
        }
    }
}
```

### Combat Resolution

```rust
pub struct CombatResolver {
    pub combat_rules: CombatRules,
    pub action_calculator: ActionCalculator,
    pub damage_system: DamageSystem,
}

pub struct CombatRound {
    pub round_number: u32,
    pub actions: Vec<CombatAction>,
    pub outcomes: Vec<ActionOutcome>,
    pub state_changes: Vec<StateChange>,
}

pub struct CombatAction {
    pub actor: EntityId,
    pub action_type: ActionType,
    pub target: Option<EntityId>,
    pub success_chance: f32,
    pub stamina_cost: f32,
}

pub enum ActionType {
    // Offensive
    Bite { aim: BodyPart },
    Claw { swipe_type: SwipeType },
    Charge { momentum: f32 },
    Grapple,
    
    // Defensive
    Block,
    Dodge { direction: Vec3 },
    CounterAttack,
    
    // Psychological
    Intimidate { display_type: ThreatDisplay },
    Feint,
    Submit,
    
    // Movement
    Retreat { distance: f32 },
    Circle { direction: CircleDirection },
    CloseDistance,
}

impl CombatResolver {
    pub fn resolve_round(
        &mut self,
        conflict: &mut Conflict,
        creatures: &mut HashMap<EntityId, Creature>,
    ) -> CombatRound {
        let mut round = CombatRound {
            round_number: conflict.duration as u32,
            actions: Vec::new(),
            outcomes: Vec::new(),
            state_changes: Vec::new(),
        };
        
        // Determine action order (based on speed and initiative)
        let action_order = self.determine_action_order(&conflict.participants, creatures);
        
        // Each combatant chooses and executes action
        for &combatant_id in &action_order {
            let combatant = conflict.get_combatant_mut(combatant_id);
            let creature = &creatures[&combatant_id];
            
            // AI decision making
            let action = self.choose_combat_action(
                combatant,
                creature,
                conflict,
                &round.actions
            );
            
            // Calculate success
            let success_chance = self.calculate_action_success(
                &action,
                creature,
                conflict,
                creatures
            );
            
            // Execute action
            let outcome = self.execute_action(
                action.clone(),
                success_chance,
                conflict,
                creatures
            );
            
            round.actions.push(action);
            round.outcomes.push(outcome);
        }
        
        // Update conflict state
        self.update_conflict_state(conflict, &round, creatures);
        
        // Check for resolution
        if let Some(resolution) = self.check_resolution(conflict, creatures) {
            conflict.resolution = Some(resolution);
        }
        
        round
    }
    
    fn choose_combat_action(
        &self,
        combatant: &Combatant,
        creature: &Creature,
        conflict: &Conflict,
        previous_actions: &[CombatAction],
    ) -> CombatAction {
        // Get valid actions based on creature abilities
        let valid_actions = self.get_valid_actions(creature, combatant);
        
        // Score each action
        let mut action_scores = Vec::new();
        for action in valid_actions {
            let score = self.score_action(
                &action,
                combatant,
                creature,
                conflict,
                previous_actions
            );
            action_scores.push((action, score));
        }
        
        // Choose best action with some randomness
        self.select_action_probabilistic(action_scores)
    }
    
    fn execute_action(
        &mut self,
        action: CombatAction,
        success_chance: f32,
        conflict: &mut Conflict,
        creatures: &mut HashMap<EntityId, Creature>,
    ) -> ActionOutcome {
        let success = rand::random::<f32>() < success_chance;
        
        match action.action_type {
            ActionType::Bite { aim } => {
                if success {
                    let damage = self.damage_system.calculate_bite_damage(
                        &creatures[&action.actor],
                        &creatures[&action.target.unwrap()],
                        aim
                    );
                    
                    self.apply_damage(action.target.unwrap(), damage, creatures);
                    
                    ActionOutcome::Hit { 
                        damage: damage.total_damage(),
                        critical: damage.is_critical,
                    }
                } else {
                    ActionOutcome::Miss
                }
            }
            
            ActionType::Intimidate { display_type } => {
                if success {
                    let intimidation = self.calculate_intimidation_effect(
                        &creatures[&action.actor],
                        display_type
                    );
                    
                    for combatant in &mut conflict.participants {
                        if combatant.creature_id != action.actor {
                            combatant.morale *= 1.0 - intimidation;
                        }
                    }
                    
                    ActionOutcome::Intimidated { 
                        morale_damage: intimidation 
                    }
                } else {
                    ActionOutcome::Ineffective
                }
            }
            
            ActionType::Submit => {
                let combatant = conflict.get_combatant_mut(action.actor);
                combatant.stance = CombatStance::Submissive;
                
                ActionOutcome::Submitted
            }
            
            _ => ActionOutcome::Other
        }
    }
}
```

### Non-Lethal Resolution

```rust
pub struct NonLethalResolution {
    pub intimidation_system: IntimidationSystem,
    pub submission_signals: SubmissionSignals,
    pub ritual_combat: RitualCombatRules,
}

pub struct IntimidationSystem {
    pub display_types: HashMap<Species, Vec<ThreatDisplay>>,
    pub effectiveness_calculator: EffectivenessCalculator,
}

pub enum ThreatDisplay {
    VocalThreat {
        volume: f32,
        frequency: f32,
        duration: f32,
    },
    VisualDisplay {
        size_increase: f32,        // Puffing up, raising fur
        color_intensity: f32,      // Flushing, color changes
        movement_pattern: String,
    },
    ScentMarking {
        intensity: f32,
        pheromone_type: PheromoneType,
    },
    MockCharge {
        speed: f32,
        stop_distance: f32,
    },
}

impl IntimidationSystem {
    pub fn perform_threat_display(
        &self,
        creature: &mut Creature,
        display: &ThreatDisplay,
    ) -> IntimidationEffect {
        let base_intimidation = match display {
            ThreatDisplay::VocalThreat { volume, .. } => {
                creature.size * volume * creature.social_state.reputation / 100.0
            }
            ThreatDisplay::VisualDisplay { size_increase, .. } => {
                creature.size * size_increase * creature.physical_stats.health / 100.0
            }
            _ => 0.5,
        };
        
        // Energy cost
        creature.physical_stats.energy -= 5.0;
        
        IntimidationEffect {
            fear_induced: base_intimidation,
            flee_chance: base_intimidation * 0.5,
            submit_chance: base_intimidation * 0.3,
        }
    }
}

pub struct RitualCombatRules {
    pub ritual_types: HashMap<ConflictType, RitualType>,
    pub victory_conditions: HashMap<RitualType, VictoryCondition>,
    pub injury_limits: InjuryLimits,
}

pub enum RitualType {
    PushingMatch,
    WrestlingBout,
    DisplayContest,
    ChaseAndEscape,
    MockBattle,
}

pub enum VictoryCondition {
    PinOpponent { duration: f32 },
    PushOutOfArea { boundary: Circle },
    ForceSubmission,
    OutlastOpponent,
    ImpressionPoints { threshold: f32 },
}

impl RitualCombatRules {
    pub fn is_ritual_combat(&self, conflict: &Conflict) -> bool {
        match conflict.conflict_type {
            ConflictType::MateCompetition => true,
            ConflictType::DominanceChallenge => true,
            ConflictType::PlayFighting => true,
            ConflictType::TerritorialDispute => {
                // Ritual if same species
                conflict.participants.iter()
                    .all(|p| p.species == conflict.participants[0].species)
            }
            _ => false,
        }
    }
    
    pub fn check_ritual_victory(
        &self,
        ritual_type: &RitualType,
        conflict: &Conflict,
    ) -> Option<EntityId> {
        match self.victory_conditions.get(ritual_type) {
            Some(VictoryCondition::PinOpponent { duration }) => {
                // Check if anyone has been pinned long enough
                for combatant in &conflict.participants {
                    if combatant.pinned_duration >= *duration {
                        return Some(combatant.pinner);
                    }
                }
            }
            Some(VictoryCondition::ForceSubmission) => {
                // Check for submission
                for combatant in &conflict.participants {
                    if matches!(combatant.stance, CombatStance::Submissive) {
                        return conflict.get_opponent(combatant.creature_id);
                    }
                }
            }
            _ => {}
        }
        None
    }
}
```

### Escape & Evasion

```rust
pub struct EscapeSystem {
    pub escape_routes: EscapeRouteCalculator,
    pub pursuit_system: PursuitSystem,
    pub evasion_tactics: HashMap<Species, Vec<EvasionTactic>>,
}

pub struct EscapeAttempt {
    pub escapee: EntityId,
    pub pursuers: Vec<EntityId>,
    pub initial_distance: f32,
    pub escape_route: Vec<Vec3>,
    pub tactics_used: Vec<EvasionTactic>,
    pub success: Option<bool>,
}

pub enum EvasionTactic {
    ZigZag {
        amplitude: f32,
        frequency: f32,
    },
    TerrainUse {
        terrain_type: TerrainType,
        advantage: f32,
    },
    GroupScatter,
    Camouflage {
        effectiveness: f32,
    },
    WaterEscape,
    ClimbTree,
    Burrow,
    DistractionDisplay,
}

impl EscapeSystem {
    pub fn attempt_escape(
        &mut self,
        escapee: &mut Creature,
        pursuers: &[&Creature],
        terrain: &TerrainGrid,
    ) -> EscapeAttempt {
        let mut attempt = EscapeAttempt {
            escapee: escapee.id,
            pursuers: pursuers.iter().map(|p| p.id).collect(),
            initial_distance: self.calculate_min_distance(escapee, pursuers),
            escape_route: Vec::new(),
            tactics_used: Vec::new(),
            success: None,
        };
        
        // Calculate optimal escape route
        attempt.escape_route = self.escape_routes.calculate_route(
            escapee.position,
            pursuers.iter().map(|p| p.position).collect(),
            terrain,
            escapee.movement_capabilities()
        );
        
        // Select evasion tactics
        let available_tactics = self.evasion_tactics
            .get(&escapee.species)
            .cloned()
            .unwrap_or_default();
        
        for tactic in available_tactics {
            if self.can_use_tactic(escapee, &tactic, terrain) {
                attempt.tactics_used.push(tactic);
            }
        }
        
        attempt
    }
    
    pub fn update_pursuit(
        &mut self,
        pursuit: &mut EscapeAttempt,
        escapee: &mut Creature,
        pursuers: &mut [&mut Creature],
        delta_time: f32,
    ) {
        // Update escapee position
        if let Some(next_pos) = pursuit.escape_route.first() {
            let move_speed = self.calculate_escape_speed(escapee, &pursuit.tactics_used);
            escapee.move_toward(*next_pos, move_speed * delta_time);
            
            if escapee.position.distance(*next_pos) < 1.0 {
                pursuit.escape_route.remove(0);
            }
        }
        
        // Update pursuer positions
        for pursuer in pursuers {
            let chase_speed = self.calculate_chase_speed(pursuer, escapee);
            let predicted_pos = self.predict_intercept_point(
                pursuer.position,
                escapee.position,
                escapee.velocity,
                chase_speed
            );
            
            pursuer.move_toward(predicted_pos, chase_speed * delta_time);
        }
        
        // Check if escape successful
        let min_distance = self.calculate_min_distance(escapee, pursuers);
        if min_distance > 50.0 {
            pursuit.success = Some(true);
        } else if min_distance < 2.0 {
            pursuit.success = Some(false);
        }
    }
}
```

### Injury System Integration

```rust
pub struct CombatInjurySystem {
    pub injury_tables: HashMap<(AttackType, BodyPart), InjuryProbability>,
    pub severity_calculator: SeverityCalculator,
    pub scarring_system: ScarringSystem,
}

pub struct InjuryProbability {
    pub injury_types: Vec<(InjuryType, f32)>,
    pub base_severity: f32,
    pub bleed_chance: f32,
    pub infection_risk: f32,
}

pub struct CombatDamage {
    pub damage_type: DamageType,
    pub amount: f32,
    pub location: BodyPart,
    pub is_critical: bool,
    pub status_effects: Vec<StatusEffect>,
}

impl CombatInjurySystem {
    pub fn apply_combat_damage(
        &mut self,
        target: &mut Creature,
        damage: &CombatDamage,
    ) -> Vec<Injury> {
        let mut injuries = Vec::new();
        
        // Reduce health
        target.health_system.current_health -= damage.amount;
        
        // Determine injuries based on damage
        if damage.amount > 10.0 {
            let injury_prob = self.injury_tables
                .get(&(damage.damage_type.to_attack_type(), damage.location))
                .unwrap_or(&DEFAULT_INJURY_PROB);
            
            for (injury_type, chance) in &injury_prob.injury_types {
                if rand::random::<f32>() < *chance {
                    let severity = self.severity_calculator.calculate(
                        damage.amount,
                        damage.is_critical,
                        target.physical_stats.toughness
                    );
                    
                    injuries.push(Injury {
                        injury_type: *injury_type,
                        severity,
                        location: damage.location,
                        healing_progress: 0.0,
                        infected: false,
                        impairments: self.get_impairments(*injury_type, severity),
                    });
                }
            }
        }
        
        // Apply status effects
        for effect in &damage.status_effects {
            target.apply_status_effect(effect.clone());
        }
        
        // Add scars for severe injuries
        if damage.amount > 30.0 && rand::random::<f32>() < 0.5 {
            self.scarring_system.add_scar(target, damage.location, damage.damage_type);
        }
        
        injuries
    }
}
```

### Group Combat

```rust
pub struct GroupCombatSystem {
    pub formation_manager: FormationManager,
    pub coordination_calculator: CoordinationCalculator,
    pub morale_system: MoraleSystem,
}

pub struct GroupConflict {
    pub groups: Vec<CombatGroup>,
    pub battlefield: Battlefield,
    pub phase: BattlePhase,
    pub group_morale: HashMap<GroupId, f32>,
}

pub struct CombatGroup {
    pub id: GroupId,
    pub members: Vec<EntityId>,
    pub formation: Formation,
    pub leader: Option<EntityId>,
    pub tactics: GroupTactics,
    pub cohesion: f32,
}

pub enum Formation {
    Line { spacing: f32 },
    Circle { radius: f32 },
    Wedge { angle: f32 },
    Scatter,
    Protective { protected: Vec<EntityId> },
}

pub enum GroupTactics {
    Surround,
    Flanking,
    FocusFire { target: EntityId },
    HitAndRun,
    Defensive,
    Retreat,
}

impl GroupCombatSystem {
    pub fn coordinate_group_action(
        &mut self,
        group: &mut CombatGroup,
        enemies: &[Creature],
        battlefield: &Battlefield,
    ) -> Vec<(EntityId, CombatAction)> {
        let mut actions = Vec::new();
        
        // Leader decides tactics
        if let Some(leader_id) = group.leader {
            group.tactics = self.decide_group_tactics(
                leader_id,
                group,
                enemies,
                battlefield
            );
        }
        
        // Assign roles based on tactics
        let roles = self.assign_combat_roles(group, &group.tactics);
        
        // Generate coordinated actions
        for (member_id, role) in roles {
            let action = self.generate_role_action(
                member_id,
                role,
                &group.tactics,
                enemies
            );
            actions.push((member_id, action));
        }
        
        // Update formation
        self.formation_manager.update_formation(
            group,
            battlefield,
            enemies
        );
        
        actions
    }
    
    pub fn update_group_morale(
        &mut self,
        group: &CombatGroup,
        events: &[CombatEvent],
    ) -> f32 {
        let mut morale_change = 0.0;
        
        for event in events {
            match event {
                CombatEvent::AllyDefeated { .. } => morale_change -= 0.2,
                CombatEvent::EnemyDefeated { .. } => morale_change += 0.15,
                CombatEvent::LeaderFallen => morale_change -= 0.4,
                CombatEvent::Reinforcements => morale_change += 0.3,
                _ => {}
            }
        }
        
        // Group cohesion affects morale resilience
        morale_change *= 1.0 - group.cohesion * 0.5;
        
        morale_change
    }
}
```

## Combat AI

```rust
pub struct CombatAI {
    pub personality: CombatPersonality,
    pub skill_level: f32,
    pub learned_patterns: HashMap<EntityId, CombatPattern>,
}

pub struct CombatPersonality {
    pub aggression: f32,
    pub caution: f32,
    pub adaptability: f32,
    pub vindictiveness: f32,
}

impl CombatAI {
    pub fn decide_action(
        &mut self,
        state: &CombatState,
        opponent: &Creature,
    ) -> CombatDecision {
        // Learn from opponent's patterns
        if let Some(pattern) = self.learned_patterns.get(&opponent.id) {
            if pattern.confidence > 0.7 {
                return self.counter_pattern(pattern, state);
            }
        }
        
        // Personality-based decision
        let aggression_score = self.score_aggressive_action(state) * self.personality.aggression;
        let defensive_score = self.score_defensive_action(state) * self.personality.caution;
        let tactical_score = self.score_tactical_action(state) * self.skill_level;
        
        // Choose highest scoring approach
        if aggression_score > defensive_score && aggression_score > tactical_score {
            self.generate_aggressive_action(state)
        } else if defensive_score > tactical_score {
            self.generate_defensive_action(state)
        } else {
            self.generate_tactical_action(state)
        }
    }
}
```

## Integration Points

### With Movement System
- Combat movement and positioning
- Escape routes and pursuit paths
- Terrain advantages

### With Social System
- Group combat coordination
- Protection of group members
- Dominance hierarchy effects

### With Emotion System
- Fear and aggression levels
- Morale and confidence
- Trauma from combat

### With Health System
- Injury application
- Stamina management
- Long-term effects of combat

## Performance Considerations

- Combat calculations use predictive algorithms to reduce per-frame cost
- Group combat uses flocking for coordinated movement
- Injury calculations are deferred until combat ends
- AI pattern learning uses compressed representations
- Spatial queries for combat use the existing spatial index

## Balance Configuration

```rust
pub struct CombatBalance {
    // Damage scaling
    pub size_damage_multiplier: f32,      // 1.5
    pub critical_hit_chance: f32,         // 0.1
    pub critical_damage_multiplier: f32,  // 2.0
    
    // Stamina costs
    pub attack_stamina_cost: f32,         // 5.0
    pub dodge_stamina_cost: f32,          // 3.0
    pub block_stamina_cost: f32,          // 2.0
    
    // Resolution thresholds
    pub morale_break_threshold: f32,      // 0.2
    pub injury_retreat_threshold: f32,    // 0.5
    pub stamina_exhaustion: f32,          // 10.0
    
    // AI parameters
    pub learning_rate: f32,               // 0.1
    pub pattern_detection_threshold: u32,  // 3
    pub prediction_confidence: f32,        // 0.7
}