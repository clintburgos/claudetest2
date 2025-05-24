# Resource System Taxonomy

## Overview

This document provides a comprehensive taxonomy of all resources in the simulation, their properties, interactions, and regeneration mechanics. Resources form the foundation of creature survival and ecosystem dynamics.

## Resource Classification

### Primary Resource Categories

```rust
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum ResourceCategory {
    Sustenance,    // Food and water
    Materials,     // Building/tool materials
    Territory,     // Spatial resources
    Knowledge,     // Information resources
    Social,        // Relationship-based resources
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum ResourceType {
    // Sustenance
    Water,
    Fruit,
    Vegetables,
    Meat,
    Seeds,
    Nectar,
    
    // Materials
    Wood,
    Stone,
    Fiber,
    Clay,
    Bone,
    Shell,
    
    // Special
    MedicinalPlant,
    Shelter,
    NestingSite,
}

pub struct ResourceDefinition {
    pub resource_type: ResourceType,
    pub category: ResourceCategory,
    pub base_properties: ResourceProperties,
    pub availability: AvailabilityPattern,
    pub extraction: ExtractionRequirements,
    pub storage: StorageProperties,
}

pub struct ResourceProperties {
    pub nutrition_value: f32,
    pub hydration_value: f32,
    pub energy_cost: f32,
    pub durability: f32,
    pub weight: f32,
    pub volume: f32,
    pub decay_rate: f32,
}
```

### Sustenance Resources

#### Water Sources

```rust
pub enum WaterSource {
    River {
        flow_rate: f32,
        purity: f32,
        seasonal_variation: SeasonalPattern,
    },
    Lake {
        size: f32,
        depth: f32,
        evaporation_rate: f32,
    },
    Spring {
        output_rate: f32,
        temperature: f32,
        mineral_content: f32,
    },
    RainCollection {
        catchment_area: f32,
        storage_capacity: f32,
    },
    Dew {
        formation_rate: f32,
        collection_difficulty: f32,
    },
}

pub struct WaterProperties {
    pub base: ResourceProperties,
    pub contamination_risk: f32,
    pub freezing_point: f32,
    pub evaporation_rate: f32,
}

impl WaterSource {
    pub fn get_availability(&self, weather: &Weather, season: Season) -> f32 {
        match self {
            WaterSource::River { flow_rate, seasonal_variation, .. } => {
                flow_rate * seasonal_variation.get_multiplier(season)
            }
            WaterSource::RainCollection { catchment_area, .. } => {
                match weather.precipitation {
                    Precipitation::Rain(intensity) => catchment_area * intensity,
                    _ => 0.0,
                }
            }
            // ... other sources
        }
    }
}
```

#### Food Resources

```rust
pub struct FoodResource {
    pub food_type: FoodType,
    pub nutrition_profile: NutritionProfile,
    pub ripeness_cycle: Option<RipenessCycle>,
    pub preparation_required: PreparationMethod,
    pub toxicity: Option<Toxicity>,
}

pub enum FoodType {
    Fruit {
        plant_species: PlantSpecies,
        seeds_present: bool,
        juice_content: f32,
    },
    Vegetation {
        plant_part: PlantPart,
        fiber_content: f32,
        digestibility: f32,
    },
    Prey {
        animal_type: AnimalType,
        meat_yield: f32,
        hunting_difficulty: f32,
    },
    Carrion {
        decay_stage: DecayStage,
        contamination_risk: f32,
    },
    Eggs {
        species: Species,
        clutch_size: u32,
        incubation_stage: f32,
    },
    Insects {
        swarm_size: u32,
        protein_content: f32,
        catching_method: CatchingMethod,
    },
}

pub struct NutritionProfile {
    pub calories: f32,
    pub proteins: f32,
    pub fats: f32,
    pub carbohydrates: f32,
    pub vitamins: HashMap<VitaminType, f32>,
    pub minerals: HashMap<MineralType, f32>,
}

pub struct RipenessCycle {
    pub stages: Vec<RipenessStage>,
    pub current_stage: usize,
    pub temperature_dependent: bool,
}

pub enum RipenessStage {
    Unripe { days_to_next: f32, edible: bool },
    Ripe { optimal_window: f32, peak_nutrition: f32 },
    Overripe { decay_rate: f32, reduced_nutrition: f32 },
    Rotten { toxicity: f32 },
}
```

### Material Resources

```rust
pub struct MaterialResource {
    pub material_type: MaterialType,
    pub mechanical_properties: MechanicalProperties,
    pub workability: Workability,
    pub combinations: Vec<MaterialCombination>,
}

pub enum MaterialType {
    Wood {
        tree_species: TreeSpecies,
        hardness: f32,
        grain_pattern: GrainPattern,
    },
    Stone {
        rock_type: RockType,
        fracture_pattern: FracturePattern,
        tool_quality: f32,
    },
    Fiber {
        plant_source: PlantSpecies,
        tensile_strength: f32,
        flexibility: f32,
    },
    Clay {
        plasticity: f32,
        drying_time: f32,
        fired_strength: f32,
    },
    Bone {
        animal_source: AnimalType,
        density: f32,
        marrow_present: bool,
    },
}

pub struct MechanicalProperties {
    pub hardness: f32,
    pub brittleness: f32,
    pub elasticity: f32,
    pub density: f32,
    pub thermal_resistance: f32,
}

pub struct Workability {
    pub required_tool: Option<ToolType>,
    pub skill_requirement: f32,
    pub processing_time: f32,
    pub failure_rate: f32,
}

pub struct MaterialCombination {
    pub materials: Vec<MaterialType>,
    pub result: CraftedItem,
    pub binding_agent: Option<MaterialType>,
    pub process: CraftingProcess,
}
```

### Territory Resources

```rust
pub struct TerritoryResource {
    pub location: Vec3,
    pub area: f32,
    pub territory_type: TerritoryType,
    pub features: Vec<TerritoryFeature>,
    pub capacity: CreatureCapacity,
}

pub enum TerritoryType {
    NestingSite {
        shelter_quality: f32,
        predator_protection: f32,
        material_availability: f32,
    },
    FeedingGround {
        food_density: f32,
        food_types: Vec<FoodType>,
        regeneration_rate: f32,
    },
    WaterAccess {
        water_sources: Vec<WaterSource>,
        bank_accessibility: f32,
        flood_risk: f32,
    },
    SocialGathering {
        visibility: f32,
        acoustic_properties: f32,
        capacity: u32,
    },
}

pub struct TerritoryFeature {
    pub feature_type: FeatureType,
    pub quality: f32,
    pub seasonal_availability: SeasonalPattern,
}

pub enum FeatureType {
    Shelter,
    Vantage,
    Concealment,
    ThermalRegulation,
    ResourceCache,
}
```

## Resource Distribution

### Spatial Distribution

```rust
pub struct ResourceDistribution {
    pub distribution_type: DistributionType,
    pub density_map: DensityMap,
    pub clustering: ClusteringPattern,
    pub biome_preferences: HashMap<BiomeType, f32>,
}

pub enum DistributionType {
    Uniform {
        density: f32,
        variation: f32,
    },
    Clustered {
        cluster_size: f32,
        cluster_density: f32,
        spacing: f32,
    },
    Gradient {
        center: Vec3,
        falloff: FalloffFunction,
    },
    Patchy {
        patch_size_range: (f32, f32),
        patch_density: f32,
        connectivity: f32,
    },
}

pub struct ClusteringPattern {
    pub attraction_distance: f32,
    pub repulsion_distance: f32,
    pub max_cluster_size: u32,
}

impl ResourceDistribution {
    pub fn generate_points(&self, area: &AABB, rng: &mut StdRng) -> Vec<ResourceSpawn> {
        match &self.distribution_type {
            DistributionType::Clustered { cluster_size, cluster_density, spacing } => {
                let mut spawns = Vec::new();
                let cluster_count = (area.volume() * cluster_density / cluster_size) as u32;
                
                for _ in 0..cluster_count {
                    let center = area.random_point(rng);
                    let points_in_cluster = (*cluster_size * rng.gen::<f32>()) as u32;
                    
                    for _ in 0..points_in_cluster {
                        let offset = Vec3::new(
                            rng.gen::<f32>() * spacing - spacing / 2.0,
                            0.0,
                            rng.gen::<f32>() * spacing - spacing / 2.0,
                        );
                        
                        spawns.push(ResourceSpawn {
                            position: center + offset,
                            amount: self.calculate_amount(center + offset),
                        });
                    }
                }
                
                spawns
            }
            // ... other distribution types
        }
    }
}
```

### Temporal Availability

```rust
pub struct TemporalAvailability {
    pub seasonal_pattern: SeasonalPattern,
    pub daily_cycle: Option<DailyCycle>,
    pub weather_modifiers: WeatherModifiers,
    pub lunar_influence: Option<LunarInfluence>,
}

pub struct SeasonalPattern {
    pub availability_curve: Curve,
    pub peak_season: Season,
    pub abundance_multiplier: HashMap<Season, f32>,
}

pub struct DailyCycle {
    pub availability_hours: Vec<(f32, f32)>, // (start_hour, end_hour)
    pub peak_time: f32,
    pub night_availability: f32,
}

pub struct WeatherModifiers {
    pub rain_effect: f32,
    pub temperature_curve: Curve,
    pub wind_effect: f32,
    pub extreme_weather_impact: HashMap<ExtremeWeather, f32>,
}

impl TemporalAvailability {
    pub fn get_current_multiplier(&self, time: &GameTime, weather: &Weather) -> f32 {
        let mut multiplier = 1.0;
        
        // Seasonal effect
        multiplier *= self.seasonal_pattern.get_multiplier(time.season);
        
        // Daily cycle effect
        if let Some(daily) = &self.daily_cycle {
            multiplier *= daily.get_multiplier(time.hour);
        }
        
        // Weather effect
        multiplier *= self.weather_modifiers.get_multiplier(weather);
        
        // Lunar effect
        if let Some(lunar) = &self.lunar_influence {
            multiplier *= lunar.get_multiplier(time.lunar_phase);
        }
        
        multiplier.clamp(0.0, 2.0)
    }
}
```

## Resource Regeneration

### Regeneration Mechanics

```rust
pub struct RegenerationSystem {
    pub base_rate: f32,
    pub growth_model: GrowthModel,
    pub limiting_factors: Vec<LimitingFactor>,
    pub disturbance_recovery: DisturbanceRecovery,
}

pub enum GrowthModel {
    Linear {
        rate: f32,
    },
    Logistic {
        carrying_capacity: f32,
        growth_rate: f32,
    },
    Exponential {
        rate: f32,
        cap: f32,
    },
    Seasonal {
        rates: HashMap<Season, f32>,
        dormant_season: Option<Season>,
    },
}

pub enum LimitingFactor {
    Water {
        requirement: f32,
        drought_tolerance: f32,
    },
    Nutrients {
        soil_quality_needed: f32,
        depletion_rate: f32,
    },
    Space {
        min_spacing: f32,
        competition_radius: f32,
    },
    Temperature {
        optimal_range: (f32, f32),
        tolerance_range: (f32, f32),
    },
}

pub struct DisturbanceRecovery {
    pub recovery_stages: Vec<RecoveryStage>,
    pub setback_events: HashMap<DisturbanceType, f32>,
}

pub struct RecoveryStage {
    pub name: String,
    pub duration: f32,
    pub growth_multiplier: f32,
    pub resource_quality: f32,
}

impl RegenerationSystem {
    pub fn update(&mut self, resource: &mut Resource, environment: &Environment, dt: f32) {
        let growth_rate = self.calculate_growth_rate(resource, environment);
        
        match &self.growth_model {
            GrowthModel::Logistic { carrying_capacity, .. } => {
                let population_ratio = resource.amount as f32 / carrying_capacity;
                let actual_growth = growth_rate * (1.0 - population_ratio);
                resource.amount += (actual_growth * dt) as u32;
            }
            // ... other models
        }
        
        // Apply limiting factors
        for factor in &self.limiting_factors {
            resource.amount = (resource.amount as f32 * factor.get_multiplier(environment)) as u32;
        }
    }
}
```

### Resource Depletion

```rust
pub struct DepletionMechanics {
    pub extraction_impact: ExtractionImpact,
    pub overexploitation_threshold: f32,
    pub recovery_penalty: RecoveryPenalty,
}

pub enum ExtractionImpact {
    Sustainable {
        max_harvest_rate: f32,
    },
    Damaging {
        damage_threshold: f32,
        recovery_time: f32,
    },
    Destructive {
        destruction_chance: f32,
        regeneration_blocked: Duration,
    },
}

pub struct RecoveryPenalty {
    pub penalty_curve: Curve,
    pub duration: f32,
    pub compounds_with_repeated_damage: bool,
}

impl DepletionMechanics {
    pub fn apply_extraction(&mut self, resource: &mut Resource, amount: f32) -> ExtractionResult {
        let extraction_ratio = amount / resource.amount as f32;
        
        match &self.extraction_impact {
            ExtractionImpact::Sustainable { max_harvest_rate } => {
                if extraction_ratio <= *max_harvest_rate {
                    resource.amount -= amount as u32;
                    ExtractionResult::Success { amount, damage: 0.0 }
                } else {
                    let allowed = resource.amount as f32 * max_harvest_rate;
                    resource.amount -= allowed as u32;
                    ExtractionResult::Partial { 
                        amount: allowed, 
                        requested: amount,
                        damage: 0.0,
                    }
                }
            }
            ExtractionImpact::Damaging { damage_threshold, recovery_time } => {
                if extraction_ratio > *damage_threshold {
                    resource.damaged = true;
                    resource.recovery_timer = *recovery_time;
                    resource.growth_multiplier *= 0.5;
                }
                resource.amount -= amount.min(resource.amount as f32) as u32;
                ExtractionResult::Success { 
                    amount, 
                    damage: (extraction_ratio - damage_threshold).max(0.0),
                }
            }
            // ... destructive case
        }
    }
}
```

## Resource Interactions

### Competition

```rust
pub struct ResourceCompetition {
    pub competition_type: CompetitionType,
    pub interference_model: InterferenceModel,
    pub dominance_factors: Vec<DominanceFactor>,
}

pub enum CompetitionType {
    Exploitative,  // Depleting resource faster
    Interference,  // Preventing access
    Territorial,   // Exclusive control
    Scramble,      // First-come-first-served
}

pub enum InterferenceModel {
    None,
    Linear { strength: f32 },
    ThresholdBased { threshold: u32, effect: f32 },
    Hierarchical { rank_effect: Curve },
}

pub enum DominanceFactor {
    Size,
    Aggression,
    GroupSize,
    TerritoryHolder,
    ArrivalOrder,
}
```

### Resource Cascades

```rust
pub struct ResourceCascade {
    pub trigger_resource: ResourceType,
    pub affected_resources: Vec<(ResourceType, EffectType)>,
    pub propagation_delay: f32,
}

pub enum EffectType {
    Increase { multiplier: f32 },
    Decrease { multiplier: f32 },
    Transformation { into: ResourceType },
    Availability { modifier: f32 },
}

impl ResourceCascade {
    pub fn propagate(&self, trigger_amount: f32, resources: &mut ResourceMap) {
        for (resource_type, effect) in &self.affected_resources {
            if let Some(resource) = resources.get_mut(resource_type) {
                match effect {
                    EffectType::Increase { multiplier } => {
                        resource.amount = (resource.amount as f32 * multiplier) as u32;
                    }
                    EffectType::Transformation { into } => {
                        let transformed = resource.amount;
                        resource.amount = 0;
                        if let Some(target) = resources.get_mut(into) {
                            target.amount += transformed;
                        }
                    }
                    // ... other effects
                }
            }
        }
    }
}
```

## Integration Example

```rust
pub fn spawn_resources(
    world: &mut World,
    biome_map: &BiomeMap,
    season: Season,
) {
    for (biome_type, areas) in biome_map.iter() {
        let resource_types = RESOURCE_REGISTRY.get_biome_resources(*biome_type);
        
        for resource_type in resource_types {
            let definition = RESOURCE_REGISTRY.get_definition(resource_type);
            let distribution = definition.get_distribution_for_biome(*biome_type);
            
            for area in areas {
                let spawn_points = distribution.generate_points(area, &mut thread_rng());
                
                for spawn in spawn_points {
                    let temporal_multiplier = definition.availability
                        .get_current_multiplier(&world.time, &world.weather);
                        
                    if temporal_multiplier > 0.0 {
                        world.spawn_resource(Resource {
                            resource_type,
                            position: spawn.position,
                            amount: (spawn.amount as f32 * temporal_multiplier) as u32,
                            quality: calculate_quality(spawn.position, biome_type),
                            regeneration: RegenerationState::default(),
                        });
                    }
                }
            }
        }
    }
}

pub fn update_resources(
    mut resources: Query<(&mut Resource, &Position)>,
    environment: Res<Environment>,
    time: Res<Time>,
) {
    for (mut resource, position) in resources.iter_mut() {
        let definition = RESOURCE_REGISTRY.get_definition(resource.resource_type);
        
        // Update regeneration
        if resource.amount < definition.carrying_capacity {
            definition.regeneration_system.update(
                &mut resource,
                &environment,
                time.delta_seconds()
            );
        }
        
        // Apply environmental effects
        let env_effects = environment.get_effects_at(position.0);
        for effect in env_effects {
            effect.apply_to_resource(&mut resource);
        }
        
        // Check depletion
        if resource.amount == 0 && resource.can_despawn() {
            commands.entity(entity).despawn();
        }
    }
}
```

This comprehensive resource taxonomy provides the foundation for complex ecological interactions and creature survival strategies.