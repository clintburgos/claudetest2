# Disease & Health System Design

## Overview

A comprehensive health system that models diseases, injuries, immunity, and overall creature wellness. The system simulates disease transmission, recovery, and the development of immunity, creating realistic population dynamics and evolutionary pressures.

## Core Health Components

```rust
pub struct HealthSystem {
    pub current_health: f32,      // 0.0 = dead, 100.0 = perfect health
    pub max_health: f32,          // Can be modified by genetics/age
    pub vitality: f32,            // Overall life force, decreases with age
    pub injuries: Vec<Injury>,
    pub diseases: Vec<Disease>,
    pub immunities: HashMap<DiseaseType, Immunity>,
    pub health_modifiers: Vec<HealthModifier>,
    pub recovery_rate: f32,       // Base healing speed
}

pub struct Injury {
    pub injury_type: InjuryType,
    pub severity: f32,            // 0.0 = minor, 1.0 = critical
    pub location: BodyPart,
    pub healing_progress: f32,
    pub infected: bool,
    pub impairments: Vec<Impairment>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InjuryType {
    Laceration,
    Fracture,
    Sprain,
    Bruise,
    Bite,
    Scratch,
    Burn,
    Frostbite,
    Exhaustion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BodyPart {
    Head,
    Torso,
    LeftLeg,
    RightLeg,
    LeftArm,
    RightArm,
    Tail,
}

pub struct Impairment {
    pub impairment_type: ImpairmentType,
    pub severity: f32,
    pub affected_stats: Vec<(StatType, f32)>, // Stat and multiplier
}

pub enum ImpairmentType {
    ReducedMobility,
    ReducedVision,
    ReducedHearing,
    ReducedStrength,
    ReducedCognition,
    Pain,
}
```

### Disease System

```rust
pub struct Disease {
    pub disease_type: DiseaseType,
    pub strain: DiseaseStrain,
    pub stage: DiseaseStage,
    pub severity: f32,
    pub duration: f32,
    pub contagious: bool,
    pub transmission_rate: f32,
    pub symptoms: Vec<Symptom>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DiseaseType {
    pub name: String,
    pub category: DiseaseCategory,
    pub base_severity: f32,
    pub base_duration: f32,
    pub lethality: f32,
}

pub enum DiseaseCategory {
    Viral,
    Bacterial,
    Fungal,
    Parasitic,
    Genetic,
    Environmental,
}

pub struct DiseaseStrain {
    pub id: u64,
    pub mutations: Vec<Mutation>,
    pub virulence: f32,
    pub resistance: HashMap<Treatment, f32>,
}

pub enum DiseaseStage {
    Incubation {
        progress: f32,
        duration: f32,
    },
    Symptomatic {
        intensity: f32,
    },
    Critical {
        organ_failure_risk: f32,
    },
    Recovery {
        progress: f32,
    },
    Chronic {
        flare_up_chance: f32,
    },
}

pub struct Symptom {
    pub symptom_type: SymptomType,
    pub severity: f32,
    pub visible: bool,
}

pub enum SymptomType {
    Fever,
    Coughing,
    Sneezing,
    Fatigue,
    Nausea,
    Rash,
    Swelling,
    Paralysis,
    Delirium,
    Bleeding,
}
```

### Disease Transmission

```rust
pub struct TransmissionSystem {
    pub transmission_routes: HashMap<DiseaseType, Vec<TransmissionRoute>>,
    pub environmental_factors: EnvironmentalFactors,
    pub quarantine_zones: Vec<QuarantineZone>,
}

pub enum TransmissionRoute {
    Airborne {
        range: f32,
        particle_lifetime: f32,
    },
    Contact {
        direct: bool,
        surface_persistence: f32,
    },
    Fluid {
        fluid_type: FluidType,
    },
    Vector {
        vector_species: String,
        vector_population: f32,
    },
    Foodborne {
        contamination_chance: f32,
    },
    Waterborne {
        water_source_contamination: f32,
    },
}

impl TransmissionSystem {
    pub fn calculate_transmission_probability(
        &self,
        disease: &Disease,
        carrier: &Creature,
        potential_host: &Creature,
        distance: f32,
        interaction_type: InteractionType,
    ) -> f32 {
        let base_rate = disease.transmission_rate;
        
        // Distance factor
        let distance_factor = match self.get_primary_route(disease) {
            TransmissionRoute::Airborne { range, .. } => {
                if distance <= range {
                    1.0 - (distance / range).powf(2.0)
                } else {
                    0.0
                }
            }
            TransmissionRoute::Contact { .. } => {
                if distance < 1.0 { 1.0 } else { 0.0 }
            }
            _ => 0.5,
        };
        
        // Carrier infectiousness
        let carrier_factor = match disease.stage {
            DiseaseStage::Incubation { .. } => 0.2,
            DiseaseStage::Symptomatic { intensity } => intensity,
            DiseaseStage::Critical { .. } => 0.8,
            DiseaseStage::Recovery { progress } => 1.0 - progress,
            DiseaseStage::Chronic { .. } => 0.3,
        };
        
        // Host susceptibility
        let immunity = potential_host.health_system.immunities
            .get(&disease.disease_type)
            .map(|i| i.protection_level)
            .unwrap_or(0.0);
        let susceptibility = (1.0 - immunity) * 
            (1.0 - potential_host.health_system.current_health / 100.0);
        
        // Environmental factors
        let env_factor = self.calculate_environmental_factor(
            disease,
            &self.environmental_factors
        );
        
        // Interaction modifier
        let interaction_modifier = match interaction_type {
            InteractionType::Grooming => 2.0,
            InteractionType::Fighting => 1.5,
            InteractionType::Mating => 3.0,
            InteractionType::Feeding => 1.2,
            InteractionType::Proximity => 1.0,
        };
        
        base_rate * distance_factor * carrier_factor * 
        susceptibility * env_factor * interaction_modifier
    }
    
    pub fn attempt_transmission(
        &mut self,
        disease: &Disease,
        carrier: &Creature,
        potential_host: &mut Creature,
        probability: f32,
    ) -> bool {
        if rand::random::<f32>() < probability {
            // Check if host already has this disease
            if potential_host.health_system.diseases.iter()
                .any(|d| d.disease_type == disease.disease_type) {
                return false;
            }
            
            // Create new infection
            let mut new_infection = disease.clone();
            new_infection.stage = DiseaseStage::Incubation {
                progress: 0.0,
                duration: disease.disease_type.base_duration * 0.2,
            };
            
            // Possible mutation
            if rand::random::<f32>() < 0.01 {
                new_infection.strain.mutate();
            }
            
            potential_host.health_system.diseases.push(new_infection);
            true
        } else {
            false
        }
    }
}
```

### Immunity System

```rust
pub struct Immunity {
    pub disease_type: DiseaseType,
    pub protection_level: f32,    // 0.0 = no protection, 1.0 = full immunity
    pub duration: ImmunityDuration,
    pub antibody_count: f32,
    pub memory_cells: f32,
}

pub enum ImmunityDuration {
    Permanent,
    Temporary { remaining: f32 },
    Maternal { remaining: f32 },
}

pub struct ImmuneResponse {
    pub response_strength: f32,
    pub response_speed: f32,
    pub cytokine_storm_risk: f32,
}

impl HealthSystem {
    pub fn develop_immunity(
        &mut self,
        disease_type: &DiseaseType,
        exposure_severity: f32,
    ) {
        let existing = self.immunities.entry(disease_type.clone())
            .or_insert(Immunity {
                disease_type: disease_type.clone(),
                protection_level: 0.0,
                duration: ImmunityDuration::Temporary { remaining: 365.0 },
                antibody_count: 0.0,
                memory_cells: 0.0,
            });
        
        // Increase immunity based on exposure
        let immunity_gain = exposure_severity * 0.3;
        existing.protection_level = (existing.protection_level + immunity_gain).min(1.0);
        existing.antibody_count += exposure_severity * 100.0;
        existing.memory_cells += exposure_severity * 50.0;
        
        // Upgrade to permanent immunity if strong enough
        if existing.protection_level > 0.8 && existing.memory_cells > 200.0 {
            existing.duration = ImmunityDuration::Permanent;
        }
    }
    
    pub fn inherit_maternal_immunity(
        &mut self,
        mother: &Creature,
        inheritance_factor: f32,
    ) {
        for (disease_type, immunity) in &mother.health_system.immunities {
            if immunity.protection_level > 0.3 {
                self.immunities.insert(disease_type.clone(), Immunity {
                    disease_type: disease_type.clone(),
                    protection_level: immunity.protection_level * inheritance_factor,
                    duration: ImmunityDuration::Maternal { remaining: 60.0 },
                    antibody_count: immunity.antibody_count * 0.5,
                    memory_cells: 0.0, // No memory cells inherited
                });
            }
        }
    }
}
```

### Treatment & Recovery

```rust
pub struct TreatmentSystem {
    pub available_treatments: HashMap<DiseaseType, Vec<Treatment>>,
    pub medicinal_plants: HashMap<PlantType, MedicinalProperties>,
    pub learned_remedies: HashMap<CreatureSpecies, Vec<LearnedRemedy>>,
}

pub struct Treatment {
    pub treatment_type: TreatmentType,
    pub effectiveness: f32,
    pub side_effects: Vec<SideEffect>,
    pub resource_cost: ResourceRequirement,
}

pub enum TreatmentType {
    Rest,
    Hydration,
    Nutrition(NutrientType),
    MedicinalPlant(PlantType),
    SocialGrooming,
    Isolation,
    TemperatureRegulation,
}

pub struct MedicinalProperties {
    pub plant_type: PlantType,
    pub active_compounds: Vec<Compound>,
    pub treats: Vec<SymptomType>,
    pub toxicity: f32,
    pub optimal_dose: f32,
}

pub struct LearnedRemedy {
    pub disease_symptoms: Vec<SymptomType>,
    pub treatment_sequence: Vec<TreatmentType>,
    pub success_rate: f32,
    pub cultural_transmission: bool,
}

impl TreatmentSystem {
    pub fn apply_treatment(
        &self,
        creature: &mut Creature,
        treatment: &Treatment,
    ) -> TreatmentResult {
        let mut result = TreatmentResult::default();
        
        // Apply treatment effects
        for disease in &mut creature.health_system.diseases {
            let effectiveness = self.calculate_treatment_effectiveness(
                disease,
                treatment,
                &creature.health_system
            );
            
            match &mut disease.stage {
                DiseaseStage::Symptomatic { intensity } => {
                    *intensity *= 1.0 - effectiveness;
                    result.symptom_relief = effectiveness;
                }
                DiseaseStage::Critical { organ_failure_risk } => {
                    *organ_failure_risk *= 1.0 - effectiveness * 0.5;
                    result.critical_improvement = effectiveness * 0.5;
                }
                _ => {}
            }
            
            disease.duration *= 1.0 - effectiveness * 0.3;
        }
        
        // Apply side effects
        for side_effect in &treatment.side_effects {
            self.apply_side_effect(creature, side_effect);
        }
        
        result
    }
    
    pub fn self_medicate(
        &self,
        creature: &mut Creature,
        symptoms: &[Symptom],
    ) -> Option<TreatmentType> {
        // Check if creature has learned remedies
        if let Some(remedies) = self.learned_remedies.get(&creature.species) {
            for remedy in remedies {
                if remedy.matches_symptoms(symptoms) {
                    return Some(remedy.treatment_sequence[0].clone());
                }
            }
        }
        
        // Instinctive behaviors
        if symptoms.iter().any(|s| matches!(s.symptom_type, SymptomType::Fever)) {
            return Some(TreatmentType::TemperatureRegulation);
        }
        
        if symptoms.iter().any(|s| matches!(s.symptom_type, SymptomType::Nausea)) {
            return Some(TreatmentType::MedicinalPlant(PlantType::Ginger));
        }
        
        None
    }
}
```

### Environmental Health Factors

```rust
pub struct EnvironmentalHealthFactors {
    pub air_quality: f32,
    pub water_quality: f32,
    pub temperature_stress: f32,
    pub population_density: f32,
    pub sanitation: f32,
    pub food_availability: f32,
    pub predator_stress: f32,
}

impl EnvironmentalHealthFactors {
    pub fn calculate_health_impact(&self, creature: &Creature) -> HealthModifier {
        let mut modifier = HealthModifier::default();
        
        // Air quality impacts
        if self.air_quality < 0.5 {
            modifier.respiratory_efficiency *= self.air_quality * 2.0;
            modifier.disease_susceptibility *= 1.5 - self.air_quality;
        }
        
        // Water quality impacts
        if self.water_quality < 0.7 {
            modifier.digestion_efficiency *= self.water_quality;
            modifier.toxin_buildup += (1.0 - self.water_quality) * 0.1;
        }
        
        // Temperature stress
        let temp_tolerance = creature.get_temperature_tolerance();
        if self.temperature_stress > temp_tolerance {
            modifier.energy_drain += (self.temperature_stress - temp_tolerance) * 10.0;
            modifier.immune_suppression += (self.temperature_stress - temp_tolerance) * 0.2;
        }
        
        // Overcrowding stress
        if self.population_density > 0.8 {
            modifier.stress_level += (self.population_density - 0.8) * 2.0;
            modifier.disease_transmission_risk *= 1.0 + self.population_density;
        }
        
        modifier
    }
}
```

### Epidemiology System

```rust
pub struct EpidemiologySystem {
    pub disease_registry: HashMap<DiseaseType, DiseaseStatistics>,
    pub outbreak_tracker: OutbreakTracker,
    pub strain_evolution: StrainEvolutionTracker,
}

pub struct DiseaseStatistics {
    pub total_cases: u64,
    pub active_cases: u32,
    pub recovered: u64,
    pub deaths: u64,
    pub r0: f32, // Basic reproduction number
    pub mutation_rate: f32,
    pub geographic_spread: HashMap<BiomeType, u32>,
}

pub struct OutbreakTracker {
    pub active_outbreaks: Vec<Outbreak>,
    pub historical_outbreaks: Vec<HistoricalOutbreak>,
    pub early_warning_system: EarlyWarningSystem,
}

pub struct Outbreak {
    pub disease: DiseaseType,
    pub patient_zero: Option<EntityId>,
    pub start_time: f64,
    pub epicenter: Vec3,
    pub affected_creatures: HashSet<EntityId>,
    pub transmission_chain: TransmissionChain,
    pub containment_measures: Vec<ContainmentMeasure>,
}

impl EpidemiologySystem {
    pub fn track_transmission(
        &mut self,
        disease: &DiseaseType,
        carrier: EntityId,
        infected: EntityId,
        location: Vec3,
        time: f64,
    ) {
        if let Some(outbreak) = self.find_active_outbreak(disease) {
            outbreak.transmission_chain.add_link(carrier, infected, location, time);
            outbreak.affected_creatures.insert(infected);
        } else {
            // Start new outbreak
            self.start_outbreak(disease.clone(), infected, location, time);
        }
        
        // Update R0
        self.update_reproduction_number(disease);
    }
    
    pub fn predict_spread(
        &self,
        outbreak: &Outbreak,
        population: &Population,
        time_horizon: f64,
    ) -> SpreadPrediction {
        let current_infected = outbreak.affected_creatures.len() as f32;
        let susceptible = population.get_susceptible_count(&outbreak.disease);
        let r0 = self.disease_registry[&outbreak.disease].r0;
        
        // SIR model
        let mut s = susceptible as f32;
        let mut i = current_infected;
        let mut r = 0.0;
        let beta = r0 / outbreak.disease.base_duration;
        let gamma = 1.0 / outbreak.disease.base_duration;
        
        let mut predictions = Vec::new();
        let dt = 0.1;
        let steps = (time_horizon / dt) as usize;
        
        for _ in 0..steps {
            let ds = -beta * s * i / population.total as f32;
            let di = beta * s * i / population.total as f32 - gamma * i;
            let dr = gamma * i;
            
            s += ds * dt;
            i += di * dt;
            r += dr * dt;
            
            predictions.push(PredictionPoint {
                infected: i,
                susceptible: s,
                recovered: r,
            });
        }
        
        SpreadPrediction {
            peak_infections: predictions.iter().map(|p| p.infected).max(),
            time_to_peak: predictions.iter().position(|p| p.infected == peak_infections),
            total_affected: r,
        }
    }
}
```

### Health UI Visualization

```rust
pub struct HealthVisualization {
    pub health_bars: bool,
    pub disease_indicators: bool,
    pub symptom_particles: bool,
    pub immunity_auras: bool,
    pub outbreak_heatmap: bool,
}

impl HealthVisualization {
    pub fn render_creature_health(&self, creature: &Creature) -> HealthDisplay {
        let mut display = HealthDisplay::default();
        
        // Health bar color
        display.bar_color = match creature.health_system.current_health {
            h if h > 80.0 => Color::GREEN,
            h if h > 50.0 => Color::YELLOW,
            h if h > 20.0 => Color::ORANGE,
            _ => Color::RED,
        };
        
        // Disease indicators
        for disease in &creature.health_system.diseases {
            display.status_icons.push(self.get_disease_icon(disease));
            
            // Symptom particles
            if self.symptom_particles {
                for symptom in &disease.symptoms {
                    if symptom.visible {
                        display.particles.push(self.create_symptom_particle(symptom));
                    }
                }
            }
        }
        
        // Immunity aura
        if self.immunity_auras && creature.has_strong_immunity() {
            display.aura = Some(AuraEffect {
                color: Color::rgba(0.2, 0.8, 1.0, 0.3),
                radius: 2.0,
                pulse_rate: 1.0,
            });
        }
        
        display
    }
    
    pub fn render_outbreak_heatmap(
        &self,
        outbreaks: &[Outbreak],
        world_bounds: &Bounds,
    ) -> HeatmapData {
        let mut heatmap = HeatmapData::new(world_bounds, 10.0); // 10m grid cells
        
        for outbreak in outbreaks {
            for &creature_id in &outbreak.affected_creatures {
                if let Some(position) = self.get_creature_position(creature_id) {
                    heatmap.add_heat_point(position, 1.0, 20.0); // 20m radius
                }
            }
        }
        
        heatmap
    }
}
```

## Integration Points

### With Genetics System
- Genetic disease susceptibility
- Inherited immunities
- Evolution of disease resistance

### With Social System
- Disease spread through social contact
- Quarantine behaviors
- Care for sick individuals

### With Resource System
- Medicinal plant gathering
- Nutritional health impacts
- Energy costs of illness

### With Decision System
- Self-medication behaviors
- Avoidance of sick individuals
- Risk assessment for interactions

## Performance Considerations

- Disease checks only run for creatures in proximity
- Immunity calculations are cached and updated on changes
- Outbreak tracking uses spatial indexing
- Symptom effects are LOD-optimized
- Epidemiology predictions run asynchronously

## Balance Parameters

```rust
pub struct DiseaseBalanceConfig {
    // Transmission rates
    pub base_transmission_rate: f32,         // 0.1
    pub airborne_multiplier: f32,            // 2.0
    pub contact_multiplier: f32,             // 3.0
    
    // Disease severity
    pub lethality_range: (f32, f32),         // (0.01, 0.5)
    pub duration_range: (f32, f32),          // (3.0, 30.0) days
    pub mutation_chance: f32,                // 0.001
    
    // Recovery rates
    pub base_recovery_rate: f32,             // 0.1 per day
    pub treatment_effectiveness: f32,         // 0.3
    pub immunity_development_rate: f32,       // 0.2
    
    // Population impacts
    pub max_simultaneous_diseases: u32,      // 3
    pub epidemic_threshold: f32,             // 0.1 (10% infected)
    pub herd_immunity_threshold: f32,        // 0.7 (70% immune)
}