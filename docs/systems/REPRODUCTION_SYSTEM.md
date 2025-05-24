# Reproduction & Mating System Design

## Overview

The reproduction system governs how creatures find mates, produce offspring, and pass on genetic traits. It integrates with the genetics system, social dynamics, and creature lifecycle to create emergent breeding behaviors and evolutionary pressures.

## Core Mechanics

### Reproductive Readiness

```rust
pub struct ReproductiveState {
    pub maturity: f32,              // 0.0 = juvenile, 1.0 = fully mature
    pub fertility: f32,             // Current fertility level (0.0-1.0)
    pub readiness: f32,             // Desire to mate (0.0-1.0)
    pub gestation_progress: f32,    // 0.0 = not pregnant, 1.0 = ready to give birth
    pub cooldown_remaining: f32,    // Time until can reproduce again
    pub offspring_count: u32,       // Lifetime offspring produced
    pub mate_preferences: MatePreferences,
}

pub struct MatePreferences {
    pub preferred_traits: Vec<(GeneticTrait, f32)>, // Trait and weight
    pub min_fitness_threshold: f32,
    pub preferred_age_range: (f32, f32),
    pub territory_requirement: bool,
    pub resource_requirement: f32,
}

// Fertility cycles
impl ReproductiveSystem {
    fn update_fertility(&mut self, creature: &mut Creature, delta_time: f32) {
        if creature.age < creature.species.maturity_age {
            creature.reproductive_state.maturity = 
                creature.age / creature.species.maturity_age;
            return;
        }
        
        // Seasonal fertility for some species
        let seasonal_modifier = if creature.species.seasonal_breeder {
            let season_factor = match self.current_season {
                Season::Spring => 2.0,
                Season::Summer => 1.5,
                Season::Fall => 0.5,
                Season::Winter => 0.1,
            };
            season_factor
        } else {
            1.0
        };
        
        // Health and nutrition affect fertility
        let health_modifier = creature.physical_stats.health.powf(2.0);
        let nutrition_modifier = (creature.physical_stats.energy / 50.0).clamp(0.0, 1.0);
        
        // Stress reduces fertility
        let stress_modifier = 1.0 - creature.emotional_state.stress;
        
        creature.reproductive_state.fertility = 
            (seasonal_modifier * health_modifier * nutrition_modifier * stress_modifier)
            .clamp(0.0, 1.0);
    }
}
```

### Mate Selection Process

```rust
pub struct MateSelectionSystem {
    pub selection_radius: f32,
    pub evaluation_weights: EvaluationWeights,
}

pub struct EvaluationWeights {
    pub genetic_compatibility: f32,
    pub fitness_score: f32,
    pub resource_provision: f32,
    pub social_status: f32,
    pub visual_display: f32,
}

impl MateSelectionSystem {
    fn find_potential_mates(
        &self,
        creature: &Creature,
        all_creatures: &[Creature],
        spatial_index: &SpatialIndex,
    ) -> Vec<EntityId> {
        let nearby = spatial_index.query_radius(creature.position, self.selection_radius);
        
        nearby.into_iter()
            .filter(|&id| {
                let other = &all_creatures[id];
                self.is_compatible_mate(creature, other)
            })
            .collect()
    }
    
    fn is_compatible_mate(&self, seeker: &Creature, potential: &Creature) -> bool {
        // Basic compatibility checks
        seeker.species == potential.species
            && potential.reproductive_state.maturity >= 1.0
            && potential.reproductive_state.fertility > 0.3
            && potential.reproductive_state.cooldown_remaining <= 0.0
            && !self.are_closely_related(seeker, potential)
    }
    
    fn evaluate_mate(&self, seeker: &Creature, potential: &Creature) -> f32 {
        let mut score = 0.0;
        
        // Genetic compatibility (avoid inbreeding, seek diversity)
        let genetic_score = self.calculate_genetic_compatibility(
            &seeker.genetics,
            &potential.genetics
        );
        score += genetic_score * self.evaluation_weights.genetic_compatibility;
        
        // Physical fitness
        let fitness = potential.physical_stats.health 
            * potential.physical_stats.strength
            * (1.0 - potential.age / potential.species.lifespan);
        score += fitness * self.evaluation_weights.fitness_score;
        
        // Resource provision ability
        let resources = potential.inventory.total_food() as f32 / 100.0;
        score += resources * self.evaluation_weights.resource_provision;
        
        // Social status
        let status = potential.social_state.reputation / 100.0;
        score += status * self.evaluation_weights.social_status;
        
        // Visual display (if applicable)
        if seeker.species.has_mating_display {
            let display_score = self.evaluate_display(potential);
            score += display_score * self.evaluation_weights.visual_display;
        }
        
        score
    }
    
    fn calculate_genetic_compatibility(
        &self,
        genes1: &Genetics,
        genes2: &Genetics,
    ) -> f32 {
        // Avoid inbreeding
        let relatedness = self.calculate_relatedness(genes1, genes2);
        let inbreeding_penalty = (1.0 - relatedness).powf(2.0);
        
        // Seek complementary traits
        let complementarity = self.calculate_trait_complementarity(genes1, genes2);
        
        inbreeding_penalty * complementarity
    }
}
```

### Courtship Behaviors

```rust
pub enum CourtshipBehavior {
    VocalDisplay {
        call_type: MatingCallType,
        duration: f32,
        intensity: f32,
    },
    VisualDisplay {
        display_type: DisplayType,
        movement_pattern: MovementPattern,
        color_intensity: f32,
    },
    GiftGiving {
        resource_type: ResourceType,
        quantity: u32,
    },
    NestBuilding {
        progress: f32,
        quality: f32,
        location: Vec3,
    },
    Dancing {
        dance_type: DanceType,
        synchronization: f32,
        duration: f32,
    },
}

pub struct CourtshipSystem {
    active_courtships: HashMap<(EntityId, EntityId), CourtshipState>,
}

pub struct CourtshipState {
    initiator: EntityId,
    recipient: EntityId,
    behaviors_performed: Vec<CourtshipBehavior>,
    recipient_interest: f32,
    duration: f32,
    success_threshold: f32,
}

impl CourtshipSystem {
    fn initiate_courtship(
        &mut self,
        initiator: EntityId,
        recipient: EntityId,
        species: &Species,
    ) {
        let state = CourtshipState {
            initiator,
            recipient,
            behaviors_performed: Vec::new(),
            recipient_interest: 0.3, // Base interest
            duration: 0.0,
            success_threshold: species.courtship_difficulty,
        };
        
        self.active_courtships.insert((initiator, recipient), state);
    }
    
    fn perform_courtship_behavior(
        &mut self,
        creature: &mut Creature,
        behavior: CourtshipBehavior,
        partner_id: EntityId,
    ) {
        let key = (creature.id, partner_id);
        if let Some(courtship) = self.active_courtships.get_mut(&key) {
            // Energy cost for display
            let energy_cost = match &behavior {
                CourtshipBehavior::VocalDisplay { intensity, .. } => 5.0 * intensity,
                CourtshipBehavior::VisualDisplay { .. } => 10.0,
                CourtshipBehavior::GiftGiving { quantity, .. } => *quantity as f32 * 2.0,
                CourtshipBehavior::NestBuilding { .. } => 15.0,
                CourtshipBehavior::Dancing { duration, .. } => 8.0 * duration,
            };
            
            creature.physical_stats.energy -= energy_cost;
            courtship.behaviors_performed.push(behavior);
        }
    }
    
    fn evaluate_courtship_response(
        &mut self,
        courtship: &mut CourtshipState,
        recipient: &Creature,
        initiator: &Creature,
    ) -> CourtshipResponse {
        // Calculate interest based on displays
        let display_quality = self.calculate_display_quality(
            &courtship.behaviors_performed,
            recipient.mate_preferences
        );
        
        courtship.recipient_interest += display_quality * 0.1;
        courtship.recipient_interest *= recipient.emotional_state.mood; // Mood affects receptiveness
        
        if courtship.recipient_interest >= courtship.success_threshold {
            CourtshipResponse::Accept
        } else if courtship.duration > 300.0 || courtship.recipient_interest < 0.1 {
            CourtshipResponse::Reject
        } else {
            CourtshipResponse::Continue
        }
    }
}
```

### Mating Process

```rust
pub struct MatingSystem {
    mating_events: Vec<MatingEvent>,
}

pub struct MatingEvent {
    parent1: EntityId,
    parent2: EntityId,
    location: Vec3,
    time: f32,
    offspring_count: u32,
}

impl MatingSystem {
    fn initiate_mating(
        &mut self,
        parent1: &mut Creature,
        parent2: &mut Creature,
    ) -> Result<MatingEvent, MatingError> {
        // Verify both are ready
        if parent1.reproductive_state.cooldown_remaining > 0.0 {
            return Err(MatingError::OnCooldown);
        }
        
        // Energy cost
        let energy_cost = 20.0;
        if parent1.physical_stats.energy < energy_cost {
            return Err(MatingError::InsufficientEnergy);
        }
        
        parent1.physical_stats.energy -= energy_cost;
        parent2.physical_stats.energy -= energy_cost;
        
        // Determine offspring count
        let base_offspring = parent1.species.offspring_range;
        let fertility_modifier = (parent1.reproductive_state.fertility 
            + parent2.reproductive_state.fertility) / 2.0;
        let offspring_count = (base_offspring.0 + 
            rand::random::<f32>() * (base_offspring.1 - base_offspring.0) as f32 
            * fertility_modifier) as u32;
        
        // Create mating event
        let event = MatingEvent {
            parent1: parent1.id,
            parent2: parent2.id,
            location: parent1.position,
            time: 0.0,
            offspring_count,
        };
        
        // Update reproductive states
        parent1.reproductive_state.cooldown_remaining = parent1.species.mating_cooldown;
        parent2.reproductive_state.cooldown_remaining = parent2.species.mating_cooldown;
        
        // If species has gestation, start pregnancy
        if parent1.species.gestation_period > 0.0 {
            let pregnant_parent = if parent1.species.pregnancy_carrier == PregnancyCarrier::Female 
                && parent1.genetics.sex == Sex::Female {
                parent1
            } else {
                parent2
            };
            
            pregnant_parent.reproductive_state.gestation_progress = 0.0;
        }
        
        Ok(event)
    }
}
```

### Pregnancy & Gestation

```rust
pub struct PregnancySystem {
    active_pregnancies: HashMap<EntityId, Pregnancy>,
}

pub struct Pregnancy {
    parent_ids: (EntityId, EntityId),
    offspring_genetics: Vec<Genetics>,
    gestation_start: f32,
    complications: Vec<PregnancyComplication>,
    nest_location: Option<Vec3>,
}

pub enum PregnancyComplication {
    NutritionalDeficiency,
    Stress,
    Environmental,
    Genetic,
}

impl PregnancySystem {
    fn update_pregnancy(
        &mut self,
        creature: &mut Creature,
        pregnancy: &mut Pregnancy,
        delta_time: f32,
    ) {
        let gestation_period = creature.species.gestation_period;
        creature.reproductive_state.gestation_progress += 
            delta_time / gestation_period;
        
        // Increased nutritional needs
        creature.physical_stats.hunger_rate *= 1.5;
        
        // Check for complications
        if creature.physical_stats.energy < 30.0 {
            pregnancy.complications.push(PregnancyComplication::NutritionalDeficiency);
        }
        
        if creature.emotional_state.stress > 0.7 {
            pregnancy.complications.push(PregnancyComplication::Stress);
        }
        
        // Movement penalties late in pregnancy
        if creature.reproductive_state.gestation_progress > 0.7 {
            creature.physical_stats.speed *= 0.7;
        }
        
        // Trigger birth
        if creature.reproductive_state.gestation_progress >= 1.0 {
            self.trigger_birth(creature, pregnancy);
        }
    }
    
    fn prepare_nest(&mut self, creature: &mut Creature) -> Option<Vec3> {
        // Find suitable nesting location
        let search_radius = 50.0;
        let candidate_spots = self.find_nesting_spots(
            creature.position,
            search_radius,
            &creature.species.nesting_preferences
        );
        
        if let Some(best_spot) = candidate_spots.first() {
            // Gather nesting materials
            creature.current_behavior = Behavior::GatherNestingMaterials {
                target_location: *best_spot,
                progress: 0.0,
            };
            Some(*best_spot)
        } else {
            None
        }
    }
}
```

### Birth & Offspring Care

```rust
pub struct OffspringCareSystem {
    parent_child_bonds: HashMap<EntityId, Vec<EntityId>>,
    care_behaviors: HashMap<EntityId, CareState>,
}

pub struct CareState {
    offspring_ids: Vec<EntityId>,
    care_duration_remaining: f32,
    feeding_schedule: f32,
    protection_radius: f32,
    teaching_progress: HashMap<SkillType, f32>,
}

impl OffspringCareSystem {
    fn give_birth(
        &mut self,
        parent: &mut Creature,
        pregnancy: &Pregnancy,
        world: &mut World,
    ) -> Vec<EntityId> {
        let mut offspring_ids = Vec::new();
        
        for genetics in &pregnancy.offspring_genetics {
            // Create baby creature
            let baby = Creature {
                id: world.next_entity_id(),
                species: parent.species.clone(),
                genetics: genetics.clone(),
                age: 0.0,
                size: parent.species.baby_size,
                position: parent.position + Vec3::random_in_sphere(2.0),
                physical_stats: PhysicalStats {
                    health: 0.7, // Babies start somewhat fragile
                    energy: 50.0,
                    speed: parent.species.baby_speed,
                    strength: 0.1,
                    ..Default::default()
                },
                ..Default::default()
            };
            
            offspring_ids.push(baby.id);
            world.spawn_creature(baby);
        }
        
        // Reset parent reproductive state
        parent.reproductive_state.gestation_progress = 0.0;
        parent.reproductive_state.offspring_count += offspring_ids.len() as u32;
        
        // Initialize parental care
        if parent.species.parental_care_duration > 0.0 {
            self.care_behaviors.insert(parent.id, CareState {
                offspring_ids: offspring_ids.clone(),
                care_duration_remaining: parent.species.parental_care_duration,
                feeding_schedule: parent.species.feeding_interval,
                protection_radius: 20.0,
                teaching_progress: HashMap::new(),
            });
        }
        
        offspring_ids
    }
    
    fn perform_parental_care(
        &mut self,
        parent: &mut Creature,
        care_state: &mut CareState,
        offspring: &mut [Creature],
        delta_time: f32,
    ) {
        care_state.care_duration_remaining -= delta_time;
        
        // Feeding behavior
        care_state.feeding_schedule -= delta_time;
        if care_state.feeding_schedule <= 0.0 {
            self.feed_offspring(parent, offspring);
            care_state.feeding_schedule = parent.species.feeding_interval;
        }
        
        // Protection behavior
        for baby in offspring {
            let distance = (baby.position - parent.position).length();
            if distance > care_state.protection_radius {
                // Parent moves toward straying offspring
                parent.current_behavior = Behavior::RetrieveOffspring {
                    target_id: baby.id,
                };
            }
        }
        
        // Teaching behaviors (for species with cultural transmission)
        if parent.species.teaches_offspring {
            self.teach_offspring(parent, offspring, &mut care_state.teaching_progress);
        }
        
        // End care when duration expires or offspring mature
        if care_state.care_duration_remaining <= 0.0 {
            self.end_parental_care(parent.id);
        }
    }
    
    fn teach_offspring(
        &mut self,
        parent: &Creature,
        offspring: &mut [Creature],
        teaching_progress: &mut HashMap<SkillType, f32>,
    ) {
        for skill in &parent.learned_behaviors {
            let progress = teaching_progress.entry(skill.skill_type).or_insert(0.0);
            
            // Teaching success based on parent skill and offspring intelligence
            for baby in offspring {
                let learning_rate = baby.cognitive_stats.intelligence * 0.01;
                *progress += learning_rate;
                
                if *progress >= 1.0 && !baby.has_skill(skill.skill_type) {
                    baby.learn_skill(skill.skill_type, SkillSource::ParentTaught);
                }
            }
        }
    }
}
```

### Genetic Inheritance

```rust
pub struct GeneticInheritanceSystem {
    mutation_rate: f32,
    crossover_points: usize,
}

impl GeneticInheritanceSystem {
    fn combine_genetics(
        &self,
        parent1: &Genetics,
        parent2: &Genetics,
    ) -> Genetics {
        let mut offspring = Genetics::default();
        
        // Mendelian inheritance for discrete traits
        offspring.size_genes = self.mendel_inherit(&parent1.size_genes, &parent2.size_genes);
        offspring.color_genes = self.mendel_inherit(&parent1.color_genes, &parent2.color_genes);
        offspring.behavior_genes = self.mendel_inherit(&parent1.behavior_genes, &parent2.behavior_genes);
        
        // Polygenic inheritance for continuous traits
        offspring.speed_modifier = self.polygenic_inherit(
            parent1.speed_modifier,
            parent2.speed_modifier,
            0.1 // variance
        );
        
        offspring.intelligence_modifier = self.polygenic_inherit(
            parent1.intelligence_modifier,
            parent2.intelligence_modifier,
            0.15
        );
        
        // Sex determination
        offspring.sex = if rand::random::<bool>() {
            Sex::Male
        } else {
            Sex::Female
        };
        
        // Apply mutations
        self.apply_mutations(&mut offspring);
        
        // Calculate fitness
        offspring.fitness = self.calculate_fitness(&offspring);
        
        offspring
    }
    
    fn mendel_inherit<T: Clone>(&self, allele1: &(T, T), allele2: &(T, T)) -> (T, T) {
        let first = if rand::random::<bool>() {
            allele1.0.clone()
        } else {
            allele1.1.clone()
        };
        
        let second = if rand::random::<bool>() {
            allele2.0.clone()
        } else {
            allele2.1.clone()
        };
        
        (first, second)
    }
    
    fn apply_mutations(&self, genetics: &mut Genetics) {
        if rand::random::<f32>() < self.mutation_rate {
            // Apply random mutation
            match rand::random::<u32>() % 5 {
                0 => genetics.speed_modifier *= 1.0 + (rand::random::<f32>() - 0.5) * 0.2,
                1 => genetics.size_modifier *= 1.0 + (rand::random::<f32>() - 0.5) * 0.2,
                2 => genetics.intelligence_modifier *= 1.0 + (rand::random::<f32>() - 0.5) * 0.2,
                3 => self.mutate_color(&mut genetics.color_genes),
                4 => self.mutate_behavior(&mut genetics.behavior_genes),
                _ => {}
            }
        }
    }
}
```

### Reproductive Strategies

```rust
pub enum ReproductiveStrategy {
    RSelection {
        // Many offspring, little parental care
        offspring_per_mating: (u32, u32),
        maturation_speed: f32,
        parental_investment: f32,
    },
    KSelection {
        // Few offspring, high parental care
        offspring_per_mating: (u32, u32),
        maturation_speed: f32,
        parental_investment: f32,
        teaching_behaviors: bool,
    },
    Tournament {
        // Males compete for females
        combat_intensity: f32,
        display_importance: f32,
        harem_size: Option<u32>,
    },
    PairBonding {
        // Monogamous pairs
        bond_duration: f32,
        shared_parenting: bool,
        bond_strength_threshold: f32,
    },
    Cooperative {
        // Group raises offspring
        helper_contribution: f32,
        communal_nesting: bool,
        alloparenting: bool,
    },
}

impl Species {
    fn get_reproductive_strategy(&self) -> ReproductiveStrategy {
        match self.base_type {
            CreatureType::Herbivore => ReproductiveStrategy::RSelection {
                offspring_per_mating: (3, 6),
                maturation_speed: 2.0,
                parental_investment: 0.3,
            },
            CreatureType::Carnivore => ReproductiveStrategy::KSelection {
                offspring_per_mating: (1, 3),
                maturation_speed: 0.5,
                parental_investment: 0.8,
                teaching_behaviors: true,
            },
            CreatureType::Omnivore => ReproductiveStrategy::PairBonding {
                bond_duration: 365.0,
                shared_parenting: true,
                bond_strength_threshold: 0.7,
            },
        }
    }
}
```

## Integration Points

### With Genetics System
- Trait inheritance and mutation
- Genetic compatibility calculations
- Fitness evaluation

### With Social System
- Mate competition and selection
- Pair bonding and loyalty
- Cooperative breeding

### With Movement System
- Courtship displays and dances
- Nest site selection
- Offspring following behavior

### With Decision System
- Reproductive priorities vs survival
- Mate choice decisions
- Parental care allocation

### With Emotion System
- Love and attachment formation
- Jealousy and mate guarding
- Parental bonding

## Performance Considerations

- Mate search uses spatial indexing for O(log n) queries
- Genetic calculations are cached and updated only on changes
- Courtship behaviors use behavior trees for efficient decision making
- Pregnancy updates are throttled to once per second
- Offspring are spawned in batches to reduce instantiation overhead

## UI/UX Elements

- Heart particles during courtship
- Special colors/glows for pregnant creatures
- Family tree visualization
- Mating season indicators
- Offspring count displays
- Genetic trait previews for potential pairings