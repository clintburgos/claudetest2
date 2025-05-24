# Creature Lifecycle Design

## Overview

The creature lifecycle system manages all stages of a creature's life from birth to death, including growth, aging, abilities, and behavioral changes. This system creates realistic population dynamics and generational gameplay.

## Life Stages

### Stage Definitions
```rust
#[derive(Clone, Debug, PartialEq)]
enum LifeStage {
    Infant,  // 0-10% of lifespan
    Child,   // 10-20% of lifespan
    Young,   // 20-40% of lifespan
    Adult,   // 40-80% of lifespan
    Elder,   // 80-100% of lifespan
}

struct LifeStageConfig {
    stage: LifeStage,
    age_range: (f32, f32), // Percentage of max lifespan
    size_multiplier: f32,
    speed_multiplier: f32,
    strength_multiplier: f32,
    learning_rate: f32,
    metabolism_rate: f32,
    social_priority: f32,
    reproduction_ability: bool,
}

const LIFE_STAGES: &[LifeStageConfig] = &[
    LifeStageConfig {
        stage: LifeStage::Infant,
        age_range: (0.0, 0.1),
        size_multiplier: 0.3,
        speed_multiplier: 0.0, // Carried by parents
        strength_multiplier: 0.1,
        learning_rate: 3.0, // Fast learning
        metabolism_rate: 0.5, // Low energy needs
        social_priority: 2.0, // High need for care
        reproduction_ability: false,
    },
    LifeStageConfig {
        stage: LifeStage::Child,
        age_range: (0.1, 0.2),
        size_multiplier: 0.5,
        speed_multiplier: 0.7,
        strength_multiplier: 0.4,
        learning_rate: 2.0,
        metabolism_rate: 0.8,
        social_priority: 1.5,
        reproduction_ability: false,
    },
    LifeStageConfig {
        stage: LifeStage::Young,
        age_range: (0.2, 0.4),
        size_multiplier: 0.8,
        speed_multiplier: 1.2, // Peak agility
        strength_multiplier: 0.8,
        learning_rate: 1.5,
        metabolism_rate: 1.2, // High energy
        social_priority: 1.0,
        reproduction_ability: true,
    },
    LifeStageConfig {
        stage: LifeStage::Adult,
        age_range: (0.4, 0.8),
        size_multiplier: 1.0,
        speed_multiplier: 1.0,
        strength_multiplier: 1.0, // Peak strength
        learning_rate: 1.0,
        metabolism_rate: 1.0,
        social_priority: 0.8,
        reproduction_ability: true,
    },
    LifeStageConfig {
        stage: LifeStage::Elder,
        age_range: (0.8, 1.0),
        size_multiplier: 0.95,
        speed_multiplier: 0.6,
        strength_multiplier: 0.6,
        learning_rate: 0.5,
        metabolism_rate: 0.7,
        social_priority: 1.2, // Seek comfort
        reproduction_ability: false,
    },
];
```

### Age Calculation
```rust
struct CreatureAge {
    current_age: f32, // In simulation hours
    max_lifespan: f32, // Genetically determined
    life_stage: LifeStage,
    stage_progress: f32, // 0.0-1.0 within current stage
}

impl CreatureAge {
    fn update(&mut self, delta_time: f32, health_modifier: f32) {
        // Age faster when unhealthy
        let aging_rate = 1.0 + (1.0 - health_modifier) * 0.5;
        self.current_age += delta_time * aging_rate;
        
        // Update life stage
        let age_percentage = self.current_age / self.max_lifespan;
        self.life_stage = self.calculate_life_stage(age_percentage);
        self.stage_progress = self.calculate_stage_progress(age_percentage);
    }
    
    fn calculate_life_stage(&self, age_percentage: f32) -> LifeStage {
        for config in LIFE_STAGES {
            if age_percentage >= config.age_range.0 && 
               age_percentage < config.age_range.1 {
                return config.stage.clone();
            }
        }
        LifeStage::Elder // Default for very old
    }
    
    fn calculate_stage_progress(&self, age_percentage: f32) -> f32 {
        let config = self.get_current_stage_config();
        let range = config.age_range.1 - config.age_range.0;
        let progress = age_percentage - config.age_range.0;
        (progress / range).clamp(0.0, 1.0)
    }
}
```

## Growth System

### Physical Development
```rust
struct PhysicalGrowth {
    base_size: f32, // Genetically determined adult size
    current_size: f32,
    growth_rate: f32,
    nutrition_history: CircularBuffer<f32>, // Affects final size
}

impl PhysicalGrowth {
    fn update(&mut self, life_stage: &LifeStage, nutrition: f32, delta_time: f32) {
        self.nutrition_history.push(nutrition);
        
        let stage_config = get_life_stage_config(life_stage);
        let target_size = self.base_size * stage_config.size_multiplier;
        
        // Nutrition affects growth rate and final size
        let avg_nutrition = self.nutrition_history.average();
        let nutrition_modifier = 0.8 + (avg_nutrition * 0.4); // 0.8-1.2x
        
        // Smooth growth towards target
        let growth_speed = self.growth_rate * nutrition_modifier * delta_time;
        self.current_size = lerp(
            self.current_size,
            target_size * nutrition_modifier,
            growth_speed
        );
    }
    
    fn get_stunting_factor(&self) -> f32 {
        // Poor nutrition during growth causes permanent stunting
        let critical_period_nutrition = self.nutrition_history
            .iter()
            .take(self.nutrition_history.len() / 3) // First third of life
            .sum::<f32>() / (self.nutrition_history.len() / 3) as f32;
            
        0.7 + (critical_period_nutrition * 0.3) // 70-100% of potential
    }
}
```

### Ability Development
```rust
struct AbilityDevelopment {
    abilities: HashMap<AbilityType, AbilityState>,
    learning_experiences: HashMap<AbilityType, f32>,
}

#[derive(Clone)]
struct AbilityState {
    unlocked: bool,
    proficiency: f32, // 0.0-1.0
    last_used: Option<SimTime>,
    use_count: u32,
}

enum AbilityType {
    // Basic abilities
    Walking,
    Running,
    Swimming,
    Climbing,
    
    // Social abilities
    Communication,
    Teaching,
    Leadership,
    
    // Survival abilities
    Foraging,
    Hunting,
    Building,
    
    // Advanced abilities
    ToolUse,
    ProblemSolving,
    Planning,
}

impl AbilityDevelopment {
    fn unlock_ability(&mut self, ability: AbilityType, life_stage: &LifeStage) -> bool {
        let requirements = match ability {
            AbilityType::Walking => {
                matches!(life_stage, LifeStage::Child | LifeStage::Young | 
                        LifeStage::Adult | LifeStage::Elder)
            },
            AbilityType::Running => {
                matches!(life_stage, LifeStage::Young | LifeStage::Adult) &&
                self.has_ability(AbilityType::Walking)
            },
            AbilityType::Communication => {
                !matches!(life_stage, LifeStage::Infant)
            },
            AbilityType::Teaching => {
                matches!(life_stage, LifeStage::Adult | LifeStage::Elder) &&
                self.get_proficiency(AbilityType::Communication) > 0.7
            },
            // ... other abilities
        };
        
        if requirements {
            self.abilities.entry(ability)
                .or_insert(AbilityState {
                    unlocked: true,
                    proficiency: 0.1,
                    last_used: None,
                    use_count: 0,
                });
            true
        } else {
            false
        }
    }
    
    fn improve_ability(
        &mut self,
        ability: AbilityType,
        learning_rate: f32,
        success: bool
    ) {
        if let Some(state) = self.abilities.get_mut(&ability) {
            let improvement = if success {
                0.01 * learning_rate
            } else {
                0.001 * learning_rate // Learn from failures too
            };
            
            state.proficiency = (state.proficiency + improvement).min(1.0);
            state.use_count += 1;
            state.last_used = Some(current_time());
            
            // Track learning for related abilities
            self.learning_experiences.entry(ability)
                .and_modify(|e| *e += improvement)
                .or_insert(improvement);
        }
    }
}
```

## Behavioral Changes

### Age-Based Behavior
```rust
struct AgeBehaviorModifiers {
    exploration_desire: f32,
    risk_tolerance: f32,
    social_need: f32,
    teaching_tendency: f32,
    routine_preference: f32,
    energy_conservation: f32,
}

impl AgeBehaviorModifiers {
    fn from_life_stage(stage: &LifeStage, stage_progress: f32) -> Self {
        match stage {
            LifeStage::Infant => AgeBehaviorModifiers {
                exploration_desire: 0.1,
                risk_tolerance: 0.0, // Parents protect
                social_need: 1.0,    // Constant care needed
                teaching_tendency: 0.0,
                routine_preference: 0.0,
                energy_conservation: 0.9,
            },
            LifeStage::Child => AgeBehaviorModifiers {
                exploration_desire: 0.8 + stage_progress * 0.2,
                risk_tolerance: 0.3,
                social_need: 0.8,
                teaching_tendency: 0.0,
                routine_preference: 0.2,
                energy_conservation: 0.3,
            },
            LifeStage::Young => AgeBehaviorModifiers {
                exploration_desire: 1.0,
                risk_tolerance: 0.7 - stage_progress * 0.2,
                social_need: 0.6,
                teaching_tendency: 0.2,
                routine_preference: 0.3,
                energy_conservation: 0.2,
            },
            LifeStage::Adult => AgeBehaviorModifiers {
                exploration_desire: 0.5,
                risk_tolerance: 0.3,
                social_need: 0.5,
                teaching_tendency: 0.8,
                routine_preference: 0.6,
                energy_conservation: 0.4,
            },
            LifeStage::Elder => AgeBehaviorModifiers {
                exploration_desire: 0.2,
                risk_tolerance: 0.1,
                social_need: 0.7, // Seek comfort in others
                teaching_tendency: 1.0, // Pass on knowledge
                routine_preference: 0.9,
                energy_conservation: 0.8,
            },
        }
    }
}
```

### Decision Weight Modifications
```rust
impl DecisionSystem {
    fn apply_age_modifiers(
        &mut self,
        base_weights: &DecisionWeights,
        age_modifiers: &AgeBehaviorModifiers,
    ) -> DecisionWeights {
        DecisionWeights {
            explore: base_weights.explore * age_modifiers.exploration_desire,
            eat: base_weights.eat * (1.0 + age_modifiers.energy_conservation),
            socialize: base_weights.socialize * age_modifiers.social_need,
            rest: base_weights.rest * (1.0 + age_modifiers.energy_conservation * 0.5),
            flee: base_weights.flee * (2.0 - age_modifiers.risk_tolerance),
            reproduce: base_weights.reproduce * self.get_reproduction_modifier(),
            teach: base_weights.teach * age_modifiers.teaching_tendency,
            learn: base_weights.learn * self.get_learning_modifier(),
        }
    }
}
```

## Dependency System

### Infant Care
```rust
struct InfantCare {
    infant_id: CreatureId,
    caregivers: Vec<CreatureId>, // Usually parents
    care_quality: f32,
    last_fed: SimTime,
    last_cleaned: SimTime,
    last_comforted: SimTime,
}

impl InfantCare {
    fn update_care_needs(&mut self, delta_time: f32) -> Vec<CareNeed> {
        let mut needs = Vec::new();
        
        let current_time = current_time();
        
        if current_time - self.last_fed > Duration::hours(2) {
            needs.push(CareNeed::Feeding);
        }
        
        if current_time - self.last_cleaned > Duration::hours(4) {
            needs.push(CareNeed::Cleaning);
        }
        
        if current_time - self.last_comforted > Duration::hours(1) {
            needs.push(CareNeed::Comfort);
        }
        
        // Update care quality based on met needs
        self.care_quality = 1.0 - (needs.len() as f32 * 0.3);
        
        needs
    }
    
    fn apply_care(&mut self, care_type: CareNeed, caregiver: CreatureId) {
        match care_type {
            CareNeed::Feeding => {
                self.last_fed = current_time();
            },
            CareNeed::Cleaning => {
                self.last_cleaned = current_time();
            },
            CareNeed::Comfort => {
                self.last_comforted = current_time();
            },
        }
        
        // Strengthen bond with caregiver
        self.strengthen_caregiver_bond(caregiver);
    }
}
```

### Elder Care
```rust
struct ElderCare {
    elder_id: CreatureId,
    mobility_level: f32, // 0.0-1.0
    cognitive_decline: f32, // 0.0-1.0
    care_providers: HashSet<CreatureId>,
    assistance_needs: Vec<AssistanceType>,
}

enum AssistanceType {
    Mobility,
    Feeding,
    SocialInteraction,
    KnowledgePreservation,
}

impl ElderCare {
    fn calculate_assistance_needs(&mut self, health: f32, age_progress: f32) {
        self.assistance_needs.clear();
        
        // Mobility declines with age and health
        self.mobility_level = health * (1.0 - age_progress * 0.7);
        if self.mobility_level < 0.5 {
            self.assistance_needs.push(AssistanceType::Mobility);
        }
        
        // Cognitive decline is more random
        if age_progress > 0.7 && rand::random::<f32>() < 0.3 {
            self.cognitive_decline += 0.01;
        }
        
        // Social needs increase with isolation
        if self.care_providers.is_empty() {
            self.assistance_needs.push(AssistanceType::SocialInteraction);
        }
        
        // Knowledge preservation becomes important
        if age_progress > 0.5 {
            self.assistance_needs.push(AssistanceType::KnowledgePreservation);
        }
    }
}
```

## Death System

### Natural Death
```rust
struct MortalitySystem {
    base_mortality_curve: MortalityCurve,
    health_modifier: f32,
    genetic_longevity: f32,
}

struct MortalityCurve {
    // Gompertz-Makeham mortality model
    alpha: f32, // Base mortality
    beta: f32,  // Aging rate
    lambda: f32, // Environmental hazard
}

impl MortalitySystem {
    fn calculate_death_probability(
        &self,
        age: f32,
        max_lifespan: f32,
        health: f32,
        stress: f32,
    ) -> f32 {
        let age_fraction = age / max_lifespan;
        
        // Gompertz function for age-related mortality
        let age_mortality = self.base_mortality_curve.alpha * 
            (self.base_mortality_curve.beta.powf(age_fraction) - 1.0);
        
        // Health impacts
        let health_penalty = (1.0 - health).powf(2.0) * 0.5;
        
        // Stress impacts
        let stress_penalty = stress * 0.1;
        
        // Environmental hazard (constant)
        let environmental = self.base_mortality_curve.lambda;
        
        // Genetic modifier
        let genetic_modifier = 2.0 - self.genetic_longevity;
        
        let total_probability = (age_mortality + health_penalty + 
            stress_penalty + environmental) * genetic_modifier;
            
        total_probability.clamp(0.0, 1.0)
    }
    
    fn check_death(&self, creature: &Creature, delta_time: f32) -> bool {
        let death_prob = self.calculate_death_probability(
            creature.age.current_age,
            creature.age.max_lifespan,
            creature.health,
            creature.stress,
        );
        
        // Convert to per-frame probability
        let frame_prob = 1.0 - (1.0 - death_prob).powf(delta_time / 3600.0);
        
        rand::random::<f32>() < frame_prob
    }
}
```

### Death Handling
```rust
enum DeathCause {
    OldAge,
    Starvation,
    Dehydration,
    Predation,
    Disease,
    Accident,
}

struct DeathEvent {
    creature_id: CreatureId,
    cause: DeathCause,
    location: Vec2,
    time: SimTime,
    age_at_death: f32,
    offspring_count: u32,
    legacy_knowledge: Vec<ConceptId>,
}

impl CreatureLifecycle {
    fn handle_death(&mut self, creature: &Creature, cause: DeathCause) {
        let death_event = DeathEvent {
            creature_id: creature.id,
            cause,
            location: creature.position,
            time: current_time(),
            age_at_death: creature.age.current_age,
            offspring_count: creature.offspring.len() as u32,
            legacy_knowledge: creature.get_teachable_knowledge(),
        };
        
        // Notify related creatures
        self.notify_death(&death_event);
        
        // Create food resource from remains
        self.create_remains_resource(&death_event);
        
        // Preserve cultural knowledge
        self.preserve_knowledge(&death_event);
        
        // Update statistics
        self.update_mortality_stats(&death_event);
    }
    
    fn notify_death(&self, event: &DeathEvent) {
        // Notify family members
        for relative in self.get_relatives(event.creature_id) {
            relative.add_memory(Memory::Loss {
                creature_id: event.creature_id,
                relationship: self.get_relationship_type(relative.id, event.creature_id),
                grief_level: self.calculate_grief(relative, event.creature_id),
            });
        }
        
        // Notify nearby creatures
        for creature in self.get_nearby_creatures(event.location, 50.0) {
            creature.add_memory(Memory::Witnessed {
                event_type: EventType::Death,
                location: event.location,
                emotional_impact: 0.3,
            });
        }
    }
}
```

## Lifecycle Transitions

### Stage Transition Events
```rust
struct LifecycleTransition {
    from_stage: LifeStage,
    to_stage: LifeStage,
    creature_id: CreatureId,
    unlocked_abilities: Vec<AbilityType>,
    lost_abilities: Vec<AbilityType>,
}

impl CreatureLifecycle {
    fn handle_stage_transition(
        &mut self,
        creature: &mut Creature,
        from: LifeStage,
        to: LifeStage,
    ) {
        match (from, to) {
            (LifeStage::Infant, LifeStage::Child) => {
                // Start independent movement
                creature.unlock_ability(AbilityType::Walking);
                creature.set_movement_mode(MovementMode::Independent);
                
                // Begin social learning
                creature.enable_learning_from_others();
            },
            (LifeStage::Child, LifeStage::Young) => {
                // Physical maturation
                creature.unlock_ability(AbilityType::Running);
                creature.increase_stat_caps();
                
                // Social independence
                creature.reduce_parental_dependency();
            },
            (LifeStage::Young, LifeStage::Adult) => {
                // Peak capabilities
                creature.unlock_ability(AbilityType::Teaching);
                creature.enable_reproduction();
                
                // Leadership potential
                if creature.has_leadership_traits() {
                    creature.unlock_ability(AbilityType::Leadership);
                }
            },
            (LifeStage::Adult, LifeStage::Elder) => {
                // Physical decline
                creature.reduce_physical_stats();
                creature.disable_reproduction();
                
                // Wisdom increase
                creature.increase_teaching_effectiveness();
                creature.prioritize_knowledge_transfer();
            },
            _ => {},
        }
        
        // Notify others of transition
        self.broadcast_transition(creature.id, from, to);
    }
}
```