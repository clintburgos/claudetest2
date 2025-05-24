# Creature System

## Table of Contents
1. [Core Design](#core-design)
2. [Lifecycle System](#lifecycle-system)
3. [Traits & Genetics](#traits--genetics)
4. [Behaviors & AI](#behaviors--ai)

---

## Core Design

### Overview

Creatures are the heart of the simulation - autonomous entities with needs, personalities, relationships, and the ability to learn and adapt. Each creature is unique, shaped by genetics, experiences, and social interactions.

### Core Attributes

```rust
pub struct Creature {
    // Identity
    pub id: EntityId,
    pub name: String,
    pub species: Species,
    
    // Physical state
    pub position: Vec3,
    pub health: f32,        // 0-100
    pub energy: f32,        // 0-100
    pub size: f32,          // 0.5-2.0x species base
    
    // Needs (0-100, higher = more urgent)
    pub hunger: f32,
    pub thirst: f32,
    pub social_need: f32,
    pub safety_need: f32,
    
    // Mental state
    pub happiness: f32,     // -100 to 100
    pub stress: f32,        // 0-100
    pub curiosity: f32,     // 0-100
    
    // Genetics & traits
    pub genetics: Genetics,
    pub personality: Personality,
    pub skills: HashMap<SkillType, f32>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Species {
    Herbivore,
    Carnivore,
    Omnivore,
}

impl Species {
    pub fn base_stats(&self) -> BaseStats {
        match self {
            Species::Herbivore => BaseStats {
                size: 1.0,
                speed: 8.0,
                strength: 3.0,
                intelligence: 5.0,
                lifespan: 365.0,
            },
            Species::Carnivore => BaseStats {
                size: 1.5,
                speed: 10.0,
                strength: 8.0,
                intelligence: 7.0,
                lifespan: 300.0,
            },
            Species::Omnivore => BaseStats {
                size: 1.2,
                speed: 7.0,
                strength: 5.0,
                intelligence: 9.0,
                lifespan: 400.0,
            },
        }
    }
}
```

### Personality System

```rust
pub struct Personality {
    // Big Five personality traits (0.0-1.0)
    pub openness: f32,          // Curiosity, creativity
    pub conscientiousness: f32, // Organization, persistence  
    pub extraversion: f32,      // Sociability, energy
    pub agreeableness: f32,     // Cooperation, trust
    pub neuroticism: f32,       // Emotional stability (inverse)
    
    // Derived behavioral tendencies
    pub cached_tendencies: BehavioralTendencies,
}

pub struct BehavioralTendencies {
    pub exploration_drive: f32,
    pub social_comfort: f32,
    pub risk_tolerance: f32,
    pub learning_rate: f32,
    pub stress_resilience: f32,
}

impl Personality {
    pub fn calculate_tendencies(&mut self) {
        self.cached_tendencies = BehavioralTendencies {
            exploration_drive: self.openness * 0.7 + self.extraversion * 0.3,
            social_comfort: self.extraversion * 0.6 + self.agreeableness * 0.4,
            risk_tolerance: (1.0 - self.neuroticism) * 0.5 + self.openness * 0.5,
            learning_rate: self.openness * 0.5 + self.conscientiousness * 0.5,
            stress_resilience: (1.0 - self.neuroticism) * 0.8 + self.conscientiousness * 0.2,
        };
    }
}
```

---

## Lifecycle System

### Life Stages

```rust
pub enum LifeStage {
    Baby { 
        age: f32,           // 0-10% of lifespan
        parent_ids: Vec<EntityId>,
        dependency: f32,    // 1.0 = fully dependent
    },
    Child { 
        age: f32,           // 10-25% of lifespan
        growth_rate: f32,
        learning_bonus: f32,
    },
    Adult { 
        age: f32,           // 25-75% of lifespan
        reproductive_maturity: f32,
        peak_performance: bool,
    },
    Elder { 
        age: f32,           // 75-100% of lifespan
        wisdom_bonus: f32,
        physical_decline: f32,
    },
}

impl LifeStage {
    pub fn from_age(age: f32, lifespan: f32) -> Self {
        let life_percentage = age / lifespan;
        
        match life_percentage {
            p if p < 0.1 => LifeStage::Baby { 
                age,
                parent_ids: Vec::new(),
                dependency: 1.0 - p * 10.0,
            },
            p if p < 0.25 => LifeStage::Child {
                age,
                growth_rate: 1.5,
                learning_bonus: 0.5,
            },
            p if p < 0.75 => LifeStage::Adult {
                age,
                reproductive_maturity: (p - 0.25) * 2.0,
                peak_performance: p < 0.5,
            },
            _ => LifeStage::Elder {
                age,
                wisdom_bonus: (life_percentage - 0.75) * 2.0,
                physical_decline: (life_percentage - 0.75) * 4.0,
            },
        }
    }
    
    pub fn get_modifiers(&self) -> LifeStageModifiers {
        match self {
            LifeStage::Baby { dependency, .. } => LifeStageModifiers {
                speed_mult: 0.5,
                strength_mult: 0.2,
                learning_mult: 2.0,
                social_need_mult: 2.0,
                independence: 1.0 - dependency,
            },
            LifeStage::Child { learning_bonus, .. } => LifeStageModifiers {
                speed_mult: 0.8,
                strength_mult: 0.6,
                learning_mult: 1.0 + learning_bonus,
                social_need_mult: 1.5,
                independence: 0.8,
            },
            LifeStage::Adult { peak_performance, .. } => LifeStageModifiers {
                speed_mult: if *peak_performance { 1.2 } else { 1.0 },
                strength_mult: if *peak_performance { 1.2 } else { 1.0 },
                learning_mult: 1.0,
                social_need_mult: 1.0,
                independence: 1.0,
            },
            LifeStage::Elder { physical_decline, wisdom_bonus, .. } => LifeStageModifiers {
                speed_mult: 1.0 - physical_decline * 0.5,
                strength_mult: 1.0 - physical_decline * 0.6,
                learning_mult: 0.7,
                social_need_mult: 1.2,
                independence: 1.0,
            },
        }
    }
}
```

### Growth & Development

```rust
pub struct GrowthSystem {
    growth_curves: HashMap<Species, GrowthCurve>,
    skill_development: SkillDevelopment,
}

impl GrowthSystem {
    pub fn update_creature_growth(
        &mut self,
        creature: &mut Creature,
        delta_time: f32,
    ) {
        match creature.life_stage {
            LifeStage::Baby { .. } | LifeStage::Child { .. } => {
                // Physical growth
                let growth_rate = self.calculate_growth_rate(creature);
                creature.size += growth_rate * delta_time;
                creature.size = creature.size.min(creature.genetics.max_size);
                
                // Skill development through play
                if creature.is_playing() {
                    self.develop_skills_through_play(creature, delta_time);
                }
            }
            LifeStage::Adult { .. } => {
                // Maintain skills through use
                self.maintain_skills(creature, delta_time);
            }
            LifeStage::Elder { .. } => {
                // Gradual physical decline
                creature.physical_stats.strength *= 0.9999_f32.powf(delta_time);
                creature.physical_stats.speed *= 0.9998_f32.powf(delta_time);
                
                // But increased wisdom
                creature.mental_stats.wisdom += 0.01 * delta_time;
            }
        }
    }
    
    fn develop_skills_through_play(
        &mut self,
        creature: &mut Creature,
        delta_time: f32,
    ) {
        let learning_rate = creature.personality.cached_tendencies.learning_rate;
        
        // Random skill development during play
        let skill = match rand::gen_range(0..4) {
            0 => SkillType::Foraging,
            1 => SkillType::SocialInteraction,
            2 => SkillType::ProblemSolving,
            _ => SkillType::PhysicalCoordination,
        };
        
        let current = creature.skills.get(&skill).copied().unwrap_or(0.0);
        let new_value = (current + learning_rate * 0.1 * delta_time).min(1.0);
        creature.skills.insert(skill, new_value);
    }
}
```

### Aging & Death

```rust
pub struct AgingSystem {
    age_effects: HashMap<AgeCategory, AgeEffects>,
    death_probability: DeathProbabilityCurve,
}

impl AgingSystem {
    pub fn update_aging(
        &mut self,
        creature: &mut Creature,
        delta_time: f32,
        environment: &Environment,
    ) -> Option<DeathCause> {
        // Age creature
        creature.age += delta_time;
        
        // Update life stage
        creature.life_stage = LifeStage::from_age(
            creature.age,
            creature.species.base_stats().lifespan * creature.genetics.lifespan_modifier
        );
        
        // Apply age-related changes
        self.apply_age_effects(creature);
        
        // Check for natural death
        if self.should_die_of_old_age(creature, environment) {
            return Some(DeathCause::OldAge);
        }
        
        // Check for other death causes
        if creature.health <= 0.0 {
            return Some(DeathCause::Injury);
        }
        
        if creature.hunger >= 100.0 {
            return Some(DeathCause::Starvation);
        }
        
        if creature.thirst >= 100.0 {
            return Some(DeathCause::Dehydration);
        }
        
        None
    }
    
    fn should_die_of_old_age(&self, creature: &Creature, environment: &Environment) -> bool {
        let max_age = creature.species.base_stats().lifespan * 
                     creature.genetics.lifespan_modifier;
        let age_ratio = creature.age / max_age;
        
        // Increasing probability after 80% of lifespan
        if age_ratio > 0.8 {
            let death_chance = self.death_probability.evaluate(age_ratio);
            
            // Environmental factors
            let stress_modifier = 1.0 + creature.stress * 0.01;
            let health_modifier = 2.0 - (creature.health / 100.0);
            
            let final_chance = death_chance * stress_modifier * health_modifier;
            
            rand::random::<f32>() < final_chance * (1.0 / 86400.0) // Per second
        } else {
            false
        }
    }
}
```

---

## Traits & Genetics

### Genetic System

```rust
pub struct Genetics {
    // Physical traits
    pub size_genes: GenePair<SizeAllele>,
    pub speed_genes: GenePair<SpeedAllele>,
    pub strength_genes: GenePair<StrengthAllele>,
    pub color_genes: GenePair<ColorAllele>,
    
    // Mental traits
    pub intelligence_genes: GenePair<IntelligenceAllele>,
    pub personality_genes: Vec<PersonalityGene>,
    
    // Health traits
    pub lifespan_modifier: f32,
    pub disease_resistance: f32,
    pub fertility: f32,
    
    // Unique identifier
    pub genome_id: u64,
    pub generation: u32,
}

pub struct GenePair<T> {
    pub allele1: T,
    pub allele2: T,
    pub dominance: Dominance,
}

pub enum Dominance {
    Allele1Dominant,
    Allele2Dominant,
    Codominant,
    Incomplete,
}

impl Genetics {
    pub fn inherit(parent1: &Genetics, parent2: &Genetics) -> Self {
        let mut rng = rand::thread_rng();
        
        // Mendelian inheritance for discrete traits
        let size_genes = GenePair {
            allele1: if rng.gen_bool(0.5) { parent1.size_genes.allele1 } 
                    else { parent1.size_genes.allele2 },
            allele2: if rng.gen_bool(0.5) { parent2.size_genes.allele1 } 
                    else { parent2.size_genes.allele2 },
            dominance: determine_dominance(),
        };
        
        // Combine personality genes with possible mutations
        let mut personality_genes = Vec::new();
        for i in 0..5 {
            let gene = if rng.gen_bool(0.5) {
                parent1.personality_genes.get(i).cloned()
            } else {
                parent2.personality_genes.get(i).cloned()
            }.unwrap_or_else(|| PersonalityGene::random());
            
            // Small chance of mutation
            if rng.gen_bool(0.01) {
                personality_genes.push(gene.mutate());
            } else {
                personality_genes.push(gene);
            }
        }
        
        // Continuous traits (average with variation)
        let lifespan_modifier = (parent1.lifespan_modifier + parent2.lifespan_modifier) / 2.0
            + rng.gen_range(-0.1..0.1);
        
        Genetics {
            size_genes,
            // ... other genes
            personality_genes,
            lifespan_modifier: lifespan_modifier.clamp(0.5, 2.0),
            disease_resistance: inherit_continuous_trait(
                parent1.disease_resistance,
                parent2.disease_resistance,
                0.1
            ),
            fertility: inherit_continuous_trait(
                parent1.fertility,
                parent2.fertility,
                0.1
            ),
            genome_id: generate_genome_id(),
            generation: parent1.generation.max(parent2.generation) + 1,
        }
    }
    
    pub fn express_phenotype(&self) -> Phenotype {
        Phenotype {
            size: self.express_size(),
            speed: self.express_speed(),
            strength: self.express_strength(),
            color: self.express_color(),
            intelligence: self.express_intelligence(),
            personality: self.express_personality(),
        }
    }
}
```

### Learned Behaviors

```rust
pub struct LearnedBehaviors {
    behaviors: HashMap<BehaviorType, LearnedBehavior>,
    teaching_ability: f32,
}

pub struct LearnedBehavior {
    pub behavior_type: BehaviorType,
    pub proficiency: f32,
    pub learned_from: LearningSource,
    pub practice_count: u32,
    pub last_used: f64,
}

pub enum LearningSource {
    Instinct,
    ParentTaught,
    ObservedPeer(EntityId),
    SelfDiscovered,
    GroupCulture,
}

impl LearnedBehaviors {
    pub fn learn_behavior(
        &mut self,
        behavior: BehaviorType,
        source: LearningSource,
        initial_proficiency: f32,
    ) {
        self.behaviors.insert(behavior, LearnedBehavior {
            behavior_type: behavior,
            proficiency: initial_proficiency,
            learned_from: source,
            practice_count: 0,
            last_used: current_time(),
        });
    }
    
    pub fn practice_behavior(&mut self, behavior: BehaviorType, success: bool) {
        if let Some(learned) = self.behaviors.get_mut(&behavior) {
            learned.practice_count += 1;
            learned.last_used = current_time();
            
            // Improve proficiency with practice
            if success {
                learned.proficiency = (learned.proficiency + 0.05).min(1.0);
            } else {
                learned.proficiency = (learned.proficiency + 0.01).min(1.0);
            }
        }
    }
    
    pub fn can_teach(&self, behavior: BehaviorType) -> bool {
        self.behaviors.get(&behavior)
            .map(|b| b.proficiency > 0.7 && self.teaching_ability > 0.5)
            .unwrap_or(false)
    }
}
```

---

## Behaviors & AI

### Behavior Tree System

```rust
pub enum Behavior {
    // Movement
    Idle,
    Wander { radius: f32 },
    MoveTo { target: Vec3, speed: f32 },
    Flee { from: Vec3, speed: f32 },
    Follow { target: EntityId, distance: f32 },
    
    // Survival
    SearchFood,
    Eat { food_source: EntityId },
    SearchWater,
    Drink { water_source: EntityId },
    Rest,
    SeekShelter,
    
    // Social
    Socialize { with: EntityId },
    CourtshipDisplay { target: EntityId },
    PlayWith { playmate: EntityId },
    Groom { target: EntityId },
    
    // Parenting
    ProtectOffspring { children: Vec<EntityId> },
    FeedOffspring { child: EntityId },
    TeachSkill { student: EntityId, skill: SkillType },
    
    // Complex
    Hunt { prey: EntityId, strategy: HuntingStrategy },
    GatherResources { resource_type: ResourceType },
    BuildNest { location: Vec3, progress: f32 },
}

pub struct BehaviorTree {
    root: Box<dyn BehaviorNode>,
    blackboard: Blackboard,
}

pub trait BehaviorNode: Send + Sync {
    fn tick(&mut self, creature: &mut Creature, world: &World, dt: f32) -> NodeStatus;
}

pub enum NodeStatus {
    Success,
    Failure,
    Running,
}

// Example composite nodes
pub struct Sequence {
    children: Vec<Box<dyn BehaviorNode>>,
    current_child: usize,
}

impl BehaviorNode for Sequence {
    fn tick(&mut self, creature: &mut Creature, world: &World, dt: f32) -> NodeStatus {
        while self.current_child < self.children.len() {
            match self.children[self.current_child].tick(creature, world, dt) {
                NodeStatus::Success => self.current_child += 1,
                NodeStatus::Failure => {
                    self.current_child = 0;
                    return NodeStatus::Failure;
                }
                NodeStatus::Running => return NodeStatus::Running,
            }
        }
        self.current_child = 0;
        NodeStatus::Success
    }
}
```

### Need-Based Behaviors

```rust
pub struct NeedSystem {
    need_curves: HashMap<NeedType, NeedCurve>,
    need_behaviors: HashMap<NeedType, Box<dyn NeedBehavior>>,
}

pub trait NeedBehavior: Send + Sync {
    fn get_behavior(&self, need_level: f32, creature: &Creature) -> Option<Behavior>;
    fn get_priority(&self, need_level: f32) -> f32;
}

pub struct HungerBehavior;

impl NeedBehavior for HungerBehavior {
    fn get_behavior(&self, need_level: f32, creature: &Creature) -> Option<Behavior> {
        match need_level {
            level if level > 80.0 => {
                // Desperate - eat anything
                Some(Behavior::SearchFood)
            }
            level if level > 50.0 => {
                // Hungry - look for preferred food
                if let Some(food) = creature.memory.recall_food_location() {
                    Some(Behavior::MoveTo { 
                        target: food.position,
                        speed: 1.0,
                    })
                } else {
                    Some(Behavior::SearchFood)
                }
            }
            _ => None,
        }
    }
    
    fn get_priority(&self, need_level: f32) -> f32{
        // Exponential urgency as hunger increases
        (need_level / 100.0).powf(2.0)
    }
}
```

### Emotional States

```rust
pub struct EmotionalState {
    pub current_emotion: Emotion,
    pub intensity: f32,
    pub duration: f32,
    pub triggers: Vec<EmotionalTrigger>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Emotion {
    Happy,
    Sad,
    Angry,
    Fearful,
    Curious,
    Loving,
    Playful,
    Stressed,
    Content,
}

pub enum EmotionalTrigger {
    NeedSatisfied(NeedType),
    NeedCritical(NeedType),
    SocialInteraction { positive: bool },
    EnvironmentalThreat,
    Discovery,
    Loss(EntityId),
    Birth(EntityId),
}

impl EmotionalState {
    pub fn update(&mut self, triggers: &[EmotionalTrigger], personality: &Personality, dt: f32) {
        // Process new triggers
        for trigger in triggers {
            let (emotion, intensity) = self.process_trigger(trigger, personality);
            
            if intensity > self.intensity {
                self.current_emotion = emotion;
                self.intensity = intensity;
                self.duration = 0.0;
            }
        }
        
        // Decay emotion over time
        self.duration += dt;
        self.intensity *= 0.99_f32.powf(dt);
        
        // Return to baseline
        if self.intensity < 0.1 {
            self.current_emotion = Emotion::Content;
            self.intensity = 0.3;
        }
    }
    
    fn process_trigger(
        &self,
        trigger: &EmotionalTrigger,
        personality: &Personality,
    ) -> (Emotion, f32) {
        match trigger {
            EmotionalTrigger::NeedSatisfied(_) => {
                (Emotion::Happy, 0.7 + personality.extraversion * 0.3)
            }
            EmotionalTrigger::NeedCritical(_) => {
                (Emotion::Stressed, 0.8 + personality.neuroticism * 0.2)
            }
            EmotionalTrigger::SocialInteraction { positive: true } => {
                (Emotion::Happy, 0.6 + personality.extraversion * 0.4)
            }
            EmotionalTrigger::Discovery => {
                (Emotion::Curious, 0.5 + personality.openness * 0.5)
            }
            _ => (Emotion::Content, 0.3),
        }
    }
}
```

### Memory System

```rust
pub struct Memory {
    short_term: VecDeque<MemoryItem>,
    long_term: HashMap<MemoryType, Vec<MemoryItem>>,
    spatial_memory: SpatialMemory,
    social_memory: SocialMemory,
    capacity: usize,
}

pub struct MemoryItem {
    pub memory_type: MemoryType,
    pub content: MemoryContent,
    pub timestamp: f64,
    pub importance: f32,
    pub recall_count: u32,
}

pub enum MemoryContent {
    Location { position: Vec3, place_type: PlaceType },
    Event { event_type: EventType, participants: Vec<EntityId> },
    Creature { id: EntityId, relationship: Relationship },
    Danger { threat_type: ThreatType, location: Vec3 },
    Success { action: Behavior, outcome: Outcome },
}

impl Memory {
    pub fn store(&mut self, content: MemoryContent, importance: f32) {
        let memory = MemoryItem {
            memory_type: content.get_type(),
            content,
            timestamp: current_time(),
            importance,
            recall_count: 0,
        };
        
        // Add to short-term memory
        self.short_term.push_back(memory.clone());
        if self.short_term.len() > 10 {
            // Move to long-term if important enough
            if let Some(old_memory) = self.short_term.pop_front() {
                if old_memory.importance > 0.5 {
                    self.consolidate_to_long_term(old_memory);
                }
            }
        }
    }
    
    pub fn recall(&mut self, memory_type: MemoryType) -> Option<&MemoryContent> {
        // Check short-term first
        for memory in &mut self.short_term {
            if memory.memory_type == memory_type {
                memory.recall_count += 1;
                return Some(&memory.content);
            }
        }
        
        // Then long-term
        if let Some(memories) = self.long_term.get_mut(&memory_type) {
            if let Some(memory) = memories.last_mut() {
                memory.recall_count += 1;
                memory.importance *= 1.1; // Strengthen recalled memories
                return Some(&memory.content);
            }
        }
        
        None
    }
    
    pub fn forget_old_memories(&mut self) {
        let current_time = current_time();
        
        for memories in self.long_term.values_mut() {
            memories.retain(|memory| {
                let age = current_time - memory.timestamp;
                let forget_threshold = 86400.0 / (memory.importance * (memory.recall_count as f32 + 1.0));
                age < forget_threshold
            });
        }
    }
}
```