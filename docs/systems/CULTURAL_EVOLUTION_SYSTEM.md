# Cultural Evolution Mechanics

## Overview

The cultural evolution system simulates the emergence, transmission, and evolution of behaviors, traditions, and knowledge across creature populations. Culture evolves through social learning, innovation, and environmental pressures.

## Cultural Components

### Cultural Traits

```rust
pub struct Culture {
    pub id: CultureId,
    pub origin_group: Option<GroupId>,
    pub age: f32,
    
    // Behavioral patterns
    pub behaviors: HashMap<BehaviorType, CulturalBehavior>,
    
    // Knowledge and skills
    pub techniques: HashMap<TechniqueType, Technique>,
    pub knowledge: KnowledgeBase,
    
    // Social customs
    pub social_norms: Vec<SocialNorm>,
    pub rituals: Vec<Ritual>,
    pub taboos: Vec<Taboo>,
    
    // Communication
    pub language_patterns: LanguageVariant,
    pub signals: HashMap<SignalType, CulturalSignal>,
    
    // Material culture
    pub tool_traditions: Vec<ToolTradition>,
    pub aesthetic_preferences: AestheticStyle,
}

#[derive(Debug, Clone)]
pub struct CulturalBehavior {
    pub behavior_type: BehaviorType,
    pub variant: BehaviorVariant,
    pub efficiency: f32,
    pub complexity: f32,
    pub transmission_fidelity: f32,
}

pub enum BehaviorType {
    Foraging,
    Hunting,
    ToolUse,
    Communication,
    Mating,
    Childcare,
    Defense,
    Construction,
}

pub enum BehaviorVariant {
    // Foraging variants
    ForagingPattern { sequence: Vec<ForageStep> },
    
    // Hunting variants  
    HuntingStrategy { tactics: Vec<HuntTactic> },
    
    // Tool use variants
    ToolTechnique { steps: Vec<ToolStep>, tool_type: ToolType },
    
    // Communication variants
    CallSequence { pattern: Vec<CallType>, meaning: CommunicationIntent },
}
```

### Knowledge Systems

```rust
pub struct KnowledgeBase {
    pub facts: HashMap<FactType, CulturalFact>,
    pub causal_models: Vec<CausalModel>,
    pub predictions: Vec<EnvironmentalPrediction>,
    pub myths: Vec<Myth>,
}

pub struct CulturalFact {
    pub fact_type: FactType,
    pub content: FactContent,
    pub confidence: f32,
    pub source: KnowledgeSource,
    pub age: f32,
}

pub enum FactType {
    ResourceLocation,
    DangerZone,
    SeasonalPattern,
    AnimalBehavior,
    PlantProperty,
    ToolFunction,
}

pub struct CausalModel {
    pub cause: ObservableEvent,
    pub effect: ObservableEvent,
    pub confidence: f32,
    pub observations: u32,
}

pub struct Myth {
    pub narrative: MythNarrative,
    pub purpose: MythPurpose,
    pub belief_strength: f32,
}

pub enum MythPurpose {
    ExplainNatural,    // Why thunder happens
    SocialCohesion,    // Shared origin story
    BehaviorGuide,     // Why we don't go there
    StatusJustification, // Why leader leads
}
```

### Social Norms

```rust
pub struct SocialNorm {
    pub id: NormId,
    pub norm_type: NormType,
    pub behavior_rules: Vec<BehaviorRule>,
    pub enforcement: EnforcementMechanism,
    pub adherence_rate: f32,
}

pub enum NormType {
    FoodSharing,
    MatingRights,
    TerritoryRespect,
    ElderCare,
    ConflictResolution,
    ResourceUse,
}

pub struct BehaviorRule {
    pub context: NormContext,
    pub expected_behavior: ExpectedBehavior,
    pub flexibility: f32,
}

pub enum EnforcementMechanism {
    Ostracism { duration: f32 },
    ReputationLoss { amount: f32 },
    ResourceDenial,
    PhysicalPunishment,
    RitualShaming,
}

pub struct Ritual {
    pub id: RitualId,
    pub ritual_type: RitualType,
    pub participants: ParticipantRequirement,
    pub steps: Vec<RitualStep>,
    pub frequency: RitualFrequency,
    pub significance: f32,
}

pub enum RitualType {
    Greeting,
    Mating,
    GroupBonding,
    SeasonalCelebration,
    Mourning,
    ComingOfAge,
    Leadership,
}
```

## Cultural Transmission

### Learning Mechanisms

```rust
pub struct CulturalTransmission {
    pub learning_modes: Vec<LearningMode>,
    pub transmission_biases: TransmissionBiases,
    pub innovation_rate: f32,
}

pub enum LearningMode {
    // Vertical: parent to offspring
    Vertical {
        fidelity: f32,
        selective: bool,
    },
    
    // Horizontal: peer to peer
    Horizontal {
        network_size: u32,
        preference: PeerPreference,
    },
    
    // Oblique: older to younger (non-parent)
    Oblique {
        teacher_selection: TeacherSelection,
        respect_factor: f32,
    },
}

pub struct TransmissionBiases {
    pub conformity_bias: f32,      // Copy the majority
    pub prestige_bias: f32,        // Copy successful individuals
    pub content_bias: f32,         // Prefer certain content types
    pub frequency_bias: f32,       // Copy common behaviors
}

pub enum TeacherSelection {
    MostSuccessful,
    Eldest,
    MostKnowledgeable,
    HighestStatus,
    Random,
}

impl CulturalTransmission {
    pub fn transmit_behavior(
        &self,
        teacher: Entity,
        learner: Entity,
        behavior: &CulturalBehavior,
        world: &World,
    ) -> Result<CulturalBehavior, TransmissionError> {
        let mode = self.select_learning_mode(teacher, learner, world);
        
        let mut transmitted = behavior.clone();
        
        // Apply transmission fidelity
        let fidelity = self.calculate_fidelity(teacher, learner, behavior, world);
        
        if rand::random::<f32>() > fidelity {
            // Transmission error occurred
            transmitted = self.introduce_variation(transmitted);
        }
        
        // Apply biases
        transmitted = self.apply_biases(transmitted, teacher, learner, world);
        
        // Check for innovation
        if rand::random::<f32>() < self.innovation_rate {
            transmitted = self.innovate(transmitted, learner, world);
        }
        
        Ok(transmitted)
    }
    
    fn introduce_variation(&self, behavior: CulturalBehavior) -> CulturalBehavior {
        let mut varied = behavior.clone();
        
        match &mut varied.variant {
            BehaviorVariant::ForagingPattern { sequence } => {
                // Randomly modify sequence
                if rand::random::<f32>() < 0.5 && !sequence.is_empty() {
                    let idx = rand::random::<usize>() % sequence.len();
                    sequence[idx] = sequence[idx].mutate();
                }
            }
            // ... other variants
        }
        
        // Recalculate efficiency after variation
        varied.efficiency *= 0.8 + rand::random::<f32>() * 0.4;
        
        varied
    }
}
```

### Cultural Diffusion

```rust
pub struct CulturalDiffusion {
    pub diffusion_rate: f32,
    pub interaction_threshold: f32,
    pub adoption_factors: AdoptionFactors,
}

pub struct AdoptionFactors {
    pub perceived_benefit: f32,
    pub compatibility: f32,
    pub complexity_penalty: f32,
    pub social_pressure: f32,
}

impl CulturalDiffusion {
    pub fn evaluate_adoption(
        &self,
        creature: Entity,
        cultural_trait: &CulturalTrait,
        source_culture: &Culture,
        world: &World,
    ) -> AdoptionDecision {
        let creature_culture = world.get::<CreatureCulture>(creature).unwrap();
        
        // Calculate adoption probability
        let benefit = self.calculate_perceived_benefit(cultural_trait, creature, world);
        let compatibility = self.calculate_compatibility(
            cultural_trait,
            &creature_culture.culture,
        );
        let complexity_cost = cultural_trait.complexity * self.adoption_factors.complexity_penalty;
        let social_influence = self.calculate_social_pressure(creature, source_culture, world);
        
        let adoption_score = 
            benefit * self.adoption_factors.perceived_benefit +
            compatibility * self.adoption_factors.compatibility +
            social_influence * self.adoption_factors.social_pressure -
            complexity_cost;
            
        if adoption_score > self.interaction_threshold {
            AdoptionDecision::Adopt {
                probability: adoption_score.clamp(0.0, 1.0),
                modifications: self.suggest_modifications(cultural_trait, creature_culture),
            }
        } else {
            AdoptionDecision::Reject {
                reason: self.identify_rejection_reason(adoption_score, compatibility, benefit),
            }
        }
    }
}
```

## Cultural Innovation

```rust
pub struct InnovationSystem {
    pub innovation_triggers: Vec<InnovationTrigger>,
    pub creativity_factors: CreativityFactors,
    pub combination_rules: CombinationRules,
}

pub enum InnovationTrigger {
    EnvironmentalChallenge {
        challenge_type: ChallengeType,
        severity: f32,
    },
    ResourceScarcity {
        resource: ResourceType,
        scarcity_level: f32,
    },
    SocialCompetition {
        competition_type: CompetitionType,
        intensity: f32,
    },
    RandomCreativity {
        base_chance: f32,
    },
    CulturalCombination {
        culture_a: CultureId,
        culture_b: CultureId,
    },
}

pub struct CreativityFactors {
    pub intelligence_weight: f32,
    pub experience_weight: f32,
    pub youth_bonus: f32,
    pub stress_modifier: Curve,
}

impl InnovationSystem {
    pub fn generate_innovation(
        &self,
        innovator: Entity,
        trigger: &InnovationTrigger,
        world: &World,
    ) -> Option<Innovation> {
        let creativity_score = self.calculate_creativity(innovator, world);
        
        if rand::random::<f32>() > creativity_score {
            return None;
        }
        
        match trigger {
            InnovationTrigger::EnvironmentalChallenge { challenge_type, .. } => {
                self.innovate_solution(innovator, challenge_type, world)
            }
            InnovationTrigger::CulturalCombination { culture_a, culture_b } => {
                self.combine_cultures(culture_a, culture_b, innovator, world)
            }
            // ... other triggers
        }
    }
    
    fn innovate_solution(
        &self,
        innovator: Entity,
        challenge: &ChallengeType,
        world: &World,
    ) -> Option<Innovation> {
        let existing_knowledge = self.gather_relevant_knowledge(innovator, challenge, world);
        
        // Attempt to combine existing elements in new ways
        let combination = self.combination_rules.generate_combination(
            existing_knowledge,
            challenge,
        );
        
        if let Some(new_behavior) = combination {
            Some(Innovation {
                innovator,
                innovation_type: InnovationType::Behavioral(new_behavior),
                timestamp: world.time(),
                success_probability: self.estimate_success(new_behavior, challenge),
            })
        } else {
            None
        }
    }
}
```

## Cultural Selection

```rust
pub struct CulturalSelection {
    pub selection_pressures: Vec<SelectionPressure>,
    pub fitness_calculator: CulturalFitnessCalculator,
}

pub enum SelectionPressure {
    Environmental {
        factor: EnvironmentalFactor,
        strength: f32,
    },
    Social {
        factor: SocialFactor,
        strength: f32,
    },
    Efficiency {
        resource: ResourceType,
        importance: f32,
    },
    Reproductive {
        trait_attractiveness: f32,
    },
}

pub struct CulturalFitnessCalculator {
    pub survival_weight: f32,
    pub reproduction_weight: f32,
    pub social_weight: f32,
    pub efficiency_weight: f32,
}

impl CulturalFitnessCalculator {
    pub fn calculate_fitness(
        &self,
        creature: Entity,
        culture: &Culture,
        world: &World,
    ) -> f32 {
        let survival = self.calculate_survival_benefit(creature, culture, world);
        let reproduction = self.calculate_reproductive_benefit(creature, culture, world);
        let social = self.calculate_social_benefit(creature, culture, world);
        let efficiency = self.calculate_efficiency_benefit(creature, culture, world);
        
        survival * self.survival_weight +
        reproduction * self.reproduction_weight +
        social * self.social_weight +
        efficiency * self.efficiency_weight
    }
    
    fn calculate_survival_benefit(&self, creature: Entity, culture: &Culture, world: &World) -> f32 {
        let mut benefit = 0.0;
        
        // Tool use improves survival
        for tool_tradition in &culture.tool_traditions {
            benefit += tool_tradition.effectiveness * 0.1;
        }
        
        // Knowledge improves survival
        for (fact_type, fact) in &culture.knowledge.facts {
            match fact_type {
                FactType::DangerZone => benefit += 0.2,
                FactType::ResourceLocation => benefit += 0.15,
                _ => benefit += 0.05,
            }
        }
        
        benefit.clamp(0.0, 2.0)
    }
}
```

## Cultural Persistence

```rust
pub struct CulturalMemory {
    pub storage_medium: StorageMedium,
    pub retention_rate: f32,
    pub corruption_rate: f32,
}

pub enum StorageMedium {
    Individual {
        capacity: u32,
        recall_accuracy: f32,
    },
    Collective {
        redundancy: u32,
        consensus_mechanism: ConsensusMechanism,
    },
    Environmental {
        markers: Vec<EnvironmentalMarker>,
        durability: f32,
    },
}

pub struct EnvironmentalMarker {
    pub marker_type: MarkerType,
    pub location: Vec3,
    pub information: EncodedInformation,
    pub age: f32,
}

pub enum MarkerType {
    ScentTrail,
    VisualMark,
    ModifiedTerrain,
    ToolCache,
    SymbolicRepresentation,
}
```

## Cultural Groups

```rust
pub struct CulturalGroup {
    pub id: CulturalGroupId,
    pub members: HashSet<Entity>,
    pub core_culture: Culture,
    pub variations: HashMap<Entity, CulturalVariation>,
    pub influence_radius: f32,
}

pub struct CulturalVariation {
    pub base_adherence: f32,
    pub personal_innovations: Vec<Innovation>,
    pub borrowed_traits: HashMap<CultureId, Vec<CulturalTrait>>,
}

impl CulturalGroup {
    pub fn update_cultural_cohesion(&mut self, world: &World) {
        // Calculate cultural similarity matrix
        let similarity_matrix = self.calculate_similarity_matrix();
        
        // Identify core vs peripheral members
        let (core_members, peripheral_members) = self.partition_by_similarity(&similarity_matrix);
        
        // Update core culture based on majority practices
        self.update_core_culture(&core_members, world);
        
        // Check for cultural splits
        if let Some(split_groups) = self.check_for_schism(&similarity_matrix) {
            self.handle_cultural_split(split_groups, world);
        }
    }
}
```

## Observable Cultural Phenomena

```rust
pub struct CulturalPhenomena {
    pub traditions: Vec<Tradition>,
    pub fads: Vec<Fad>,
    pub taboos: Vec<Taboo>,
    pub innovations: Vec<Innovation>,
}

pub struct Tradition {
    pub id: TraditionId,
    pub behavior_pattern: BehaviorPattern,
    pub participants: ParticipationRequirement,
    pub frequency: Frequency,
    pub age: f32,
    pub stability: f32,
}

pub struct Fad {
    pub id: FadId,
    pub behavior: CulturalBehavior,
    pub adoption_curve: AdoptionCurve,
    pub peak_popularity: f32,
    pub current_phase: FadPhase,
}

pub enum FadPhase {
    Emerging,
    Growing,
    Peak,
    Declining,
    Extinct,
}

pub struct Taboo {
    pub id: TabooId,
    pub prohibited_behavior: BehaviorType,
    pub violation_consequences: Vec<Consequence>,
    pub strength: f32,
    pub origin_story: Option<Myth>,
}
```

## Integration Example

```rust
pub fn update_cultural_evolution(
    mut creatures: Query<(&mut CreatureCulture, &SocialComponent, &Intelligence)>,
    groups: Query<&Group>,
    environment: Res<Environment>,
    time: Res<Time>,
    mut events: EventWriter<CulturalEvent>,
) {
    // Process cultural transmission within groups
    for group in groups.iter() {
        let members: Vec<_> = group.members.iter().copied().collect();
        
        for i in 0..members.len() {
            for j in i+1..members.len() {
                let (teacher, learner) = (members[i], members[j]);
                
                if let (Ok((teacher_culture, _, _)), Ok((mut learner_culture, _, intelligence))) = 
                    (creatures.get(teacher), creatures.get_mut(learner)) 
                {
                    // Attempt cultural transmission
                    for behavior in teacher_culture.culture.behaviors.values() {
                        if rand::random::<f32>() < calculate_transmission_chance(
                            teacher,
                            learner,
                            behavior,
                            &world
                        ) {
                            match TRANSMISSION_SYSTEM.transmit_behavior(
                                teacher,
                                learner,
                                behavior,
                                &world
                            ) {
                                Ok(transmitted) => {
                                    learner_culture.adopt_behavior(transmitted);
                                    events.send(CulturalEvent::BehaviorTransmitted {
                                        from: teacher,
                                        to: learner,
                                        behavior: transmitted,
                                    });
                                }
                                Err(_) => {}
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Check for innovations
    for (mut culture, social, intelligence) in creatures.iter_mut() {
        let innovation_chance = calculate_innovation_chance(
            intelligence.value,
            culture.experience,
            &environment,
        );
        
        if rand::random::<f32>() < innovation_chance {
            if let Some(innovation) = INNOVATION_SYSTEM.generate_innovation(
                entity,
                &identify_trigger(&environment),
                &world
            ) {
                culture.add_innovation(innovation.clone());
                events.send(CulturalEvent::InnovationCreated {
                    innovator: entity,
                    innovation,
                });
            }
        }
    }
    
    // Update cultural fitness
    for (culture, _, _) in creatures.iter() {
        let fitness = CULTURAL_FITNESS.calculate_fitness(
            entity,
            &culture.culture,
            &world
        );
        
        // Store fitness for selection pressures
        culture.current_fitness = fitness;
    }
}
```

This cultural evolution system creates emergent traditions, knowledge systems, and behavioral variations that spread and evolve through creature populations.