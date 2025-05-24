# Biome Resource Balance Design

## Overview

The biome resource balance system ensures each biome has appropriate resource distribution, spawn rates, and carrying capacities to support realistic creature populations while maintaining gameplay balance and encouraging migration patterns.

## Biome Characteristics

### Resource Distribution Matrix
```rust
struct BiomeResourceProfile {
    biome_type: BiomeType,
    
    // Resource abundance (0.0-1.0)
    food_abundance: f32,
    water_abundance: f32,
    shelter_abundance: f32,
    
    // Resource types and weights
    food_types: Vec<(FoodType, f32)>,
    water_sources: Vec<(WaterSource, f32)>,
    shelter_types: Vec<(ShelterType, f32)>,
    
    // Seasonal variations
    seasonal_modifiers: [SeasonalModifier; 4],
    
    // Spatial distribution
    resource_clustering: f32, // 0.0 = uniform, 1.0 = highly clustered
    edge_density: f32, // Resources at biome edges
}

const BIOME_PROFILES: &[BiomeResourceProfile] = &[
    BiomeResourceProfile {
        biome_type: BiomeType::Forest,
        food_abundance: 0.85,
        water_abundance: 0.75,
        shelter_abundance: 0.90,
        food_types: vec![
            (FoodType::Berries, 0.35),
            (FoodType::Nuts, 0.30),
            (FoodType::Mushrooms, 0.20),
            (FoodType::Roots, 0.15),
        ],
        water_sources: vec![
            (WaterSource::Stream, 0.50),
            (WaterSource::Spring, 0.30),
            (WaterSource::Puddle, 0.20),
        ],
        shelter_types: vec![
            (ShelterType::TreeHollow, 0.40),
            (ShelterType::UnderBrush, 0.35),
            (ShelterType::Cave, 0.25),
        ],
        seasonal_modifiers: [
            SeasonalModifier { food: 1.2, water: 1.1, shelter: 1.0 }, // Spring
            SeasonalModifier { food: 1.0, water: 0.9, shelter: 1.0 }, // Summer
            SeasonalModifier { food: 1.3, water: 1.0, shelter: 0.9 }, // Autumn
            SeasonalModifier { food: 0.4, water: 0.8, shelter: 0.8 }, // Winter
        ],
        resource_clustering: 0.6,
        edge_density: 0.7,
    },
    BiomeResourceProfile {
        biome_type: BiomeType::Desert,
        food_abundance: 0.25,
        water_abundance: 0.15,
        shelter_abundance: 0.60,
        food_types: vec![
            (FoodType::Cactus, 0.50),
            (FoodType::Seeds, 0.30),
            (FoodType::Insects, 0.20),
        ],
        water_sources: vec![
            (WaterSource::Oasis, 0.70),
            (WaterSource::Underground, 0.30),
        ],
        shelter_types: vec![
            (ShelterType::RockFormation, 0.60),
            (ShelterType::Burrow, 0.40),
        ],
        seasonal_modifiers: [
            SeasonalModifier { food: 0.8, water: 0.6, shelter: 1.0 }, // Spring
            SeasonalModifier { food: 0.5, water: 0.3, shelter: 1.1 }, // Summer
            SeasonalModifier { food: 0.9, water: 0.7, shelter: 1.0 }, // Autumn
            SeasonalModifier { food: 0.7, water: 0.8, shelter: 0.9 }, // Winter
        ],
        resource_clustering: 0.9, // Highly clustered around oases
        edge_density: 0.3,
    },
    // ... other biomes
];
```

### Carrying Capacity Calculation
```rust
struct CarryingCapacity {
    base_capacity: f32,
    current_load: f32,
    pressure_factors: PressureFactors,
}

struct PressureFactors {
    resource_depletion: f32,
    competition_index: f32,
    predation_pressure: f32,
    disease_prevalence: f32,
}

impl CarryingCapacity {
    fn calculate_for_biome(
        biome: &Biome,
        area: f32,
        season: Season,
    ) -> f32 {
        let profile = get_biome_profile(biome.biome_type);
        
        // Base capacity from resources
        let food_capacity = profile.food_abundance * area * FOOD_TO_CREATURES_RATIO;
        let water_capacity = profile.water_abundance * area * WATER_TO_CREATURES_RATIO;
        let shelter_capacity = profile.shelter_abundance * area * SHELTER_TO_CREATURES_RATIO;
        
        // Take minimum as bottleneck
        let base = food_capacity.min(water_capacity).min(shelter_capacity);
        
        // Apply seasonal modifier
        let seasonal = profile.seasonal_modifiers[season as usize];
        let seasonal_factor = (seasonal.food + seasonal.water + seasonal.shelter) / 3.0;
        
        base * seasonal_factor
    }
    
    fn get_population_pressure(&self) -> f32 {
        (self.current_load / self.base_capacity).clamp(0.0, 2.0)
    }
    
    fn update_pressure_factors(
        &mut self,
        creatures: &[Creature],
        resources: &[Resource],
    ) {
        // Resource depletion
        self.pressure_factors.resource_depletion = 
            1.0 - (resources.iter()
                .map(|r| r.current_amount / r.max_amount)
                .sum::<f32>() / resources.len() as f32);
        
        // Competition index
        let avg_creatures_per_resource = creatures.len() as f32 / resources.len() as f32;
        self.pressure_factors.competition_index = 
            (avg_creatures_per_resource / IDEAL_CREATURES_PER_RESOURCE).min(2.0);
        
        // Update current load
        self.current_load = creatures.len() as f32;
    }
}
```

## Spawn Rate Algorithms

### Dynamic Spawn Rates
```rust
struct ResourceSpawnSystem {
    spawn_timers: HashMap<ChunkId, SpawnTimer>,
    spawn_queue: BinaryHeap<ScheduledSpawn>,
    environmental_factors: EnvironmentalFactors,
}

struct SpawnTimer {
    last_spawn: SimTime,
    spawn_pressure: f32,
    local_depletion: f32,
}

struct ScheduledSpawn {
    time: SimTime,
    location: Vec2,
    resource_type: ResourceType,
    amount: f32,
}

impl ResourceSpawnSystem {
    fn calculate_spawn_rate(
        &self,
        biome: &Biome,
        local_resources: &[Resource],
        local_creatures: usize,
    ) -> f32 {
        let profile = get_biome_profile(biome.biome_type);
        
        // Base spawn rate
        let base_rate = match biome.biome_type {
            BiomeType::RainForest => 2.0, // Resources per hour per chunk
            BiomeType::Forest => 1.5,
            BiomeType::Grassland => 1.2,
            BiomeType::Savanna => 0.8,
            BiomeType::Desert => 0.3,
            BiomeType::Tundra => 0.4,
            BiomeType::Mountain => 0.6,
            BiomeType::Swamp => 1.0,
        };
        
        // Depletion factor - spawn more when depleted
        let total_capacity: f32 = local_resources.iter()
            .map(|r| r.max_amount)
            .sum();
        let current_amount: f32 = local_resources.iter()
            .map(|r| r.current_amount)
            .sum();
        let depletion_factor = if total_capacity > 0.0 {
            1.0 + (1.0 - current_amount / total_capacity)
        } else {
            2.0 // High spawn rate if no resources
        };
        
        // Competition factor - spawn less with high competition
        let competition_factor = 1.0 / (1.0 + local_creatures as f32 / 10.0);
        
        // Environmental factors
        let env_factor = self.environmental_factors.get_multiplier(biome);
        
        base_rate * depletion_factor * competition_factor * env_factor
    }
    
    fn spawn_resources(
        &mut self,
        chunk: &Chunk,
        spawn_count: usize,
        world: &mut World,
    ) {
        let profile = get_biome_profile(chunk.biome_type);
        
        for _ in 0..spawn_count {
            // Select resource type based on weights
            let resource_type = self.select_resource_type(profile);
            
            // Find spawn location
            if let Some(location) = self.find_spawn_location(chunk, &resource_type) {
                let quality = self.calculate_resource_quality(
                    chunk.biome_type,
                    &location,
                    world.get_season(),
                );
                
                let amount = self.calculate_spawn_amount(&resource_type, quality);
                
                world.spawn_resource(Resource {
                    resource_type,
                    position: location,
                    current_amount: amount,
                    max_amount: amount,
                    quality,
                    regeneration_rate: profile.get_regen_rate(&resource_type),
                });
            }
        }
    }
    
    fn find_spawn_location(
        &self,
        chunk: &Chunk,
        resource_type: &ResourceType,
    ) -> Option<Vec2> {
        let mut attempts = 0;
        let max_attempts = 20;
        
        while attempts < max_attempts {
            let candidate = chunk.get_random_position();
            
            // Check terrain suitability
            if !self.is_suitable_terrain(&candidate, resource_type) {
                attempts += 1;
                continue;
            }
            
            // Check minimum distance from similar resources
            let min_distance = self.get_min_spawn_distance(resource_type);
            if chunk.has_resource_within_distance(&candidate, min_distance) {
                attempts += 1;
                continue;
            }
            
            return Some(candidate);
        }
        
        None
    }
}
```

### Resource Quality Distribution
```rust
struct QualityDistribution {
    biome_type: BiomeType,
    base_quality: f32,
    variance: f32,
    
    // Factors affecting quality
    elevation_curve: Curve,
    moisture_curve: Curve,
    temperature_curve: Curve,
}

impl QualityDistribution {
    fn sample_quality(
        &self,
        location: &TerrainPoint,
        season: Season,
    ) -> f32 {
        // Base quality with random variance
        let base = self.base_quality + 
            rand_normal(0.0, self.variance).clamp(-0.3, 0.3);
        
        // Environmental modifiers
        let elevation_mod = self.elevation_curve.evaluate(location.elevation);
        let moisture_mod = self.moisture_curve.evaluate(location.moisture);
        let temp_mod = self.temperature_curve.evaluate(location.temperature);
        
        // Seasonal modifier
        let seasonal_mod = match (self.biome_type, season) {
            (BiomeType::Forest, Season::Autumn) => 1.2, // Abundant harvest
            (BiomeType::Desert, Season::Summer) => 0.6, // Harsh conditions
            (BiomeType::Tundra, Season::Winter) => 0.4, // Extreme scarcity
            _ => 1.0,
        };
        
        (base * elevation_mod * moisture_mod * temp_mod * seasonal_mod)
            .clamp(0.1, 1.0)
    }
}
```

## Migration Patterns

### Migration Triggers
```rust
struct MigrationAnalyzer {
    pressure_threshold: f32,
    resource_memory_weight: f32,
    seasonal_pattern_weight: f32,
}

impl MigrationAnalyzer {
    fn analyze_migration_need(
        &self,
        current_biome: &Biome,
        creature: &Creature,
        local_conditions: &LocalConditions,
    ) -> Option<MigrationDecision> {
        let pressure = self.calculate_total_pressure(local_conditions);
        
        if pressure > self.pressure_threshold {
            // Find better biomes
            let candidates = self.find_candidate_biomes(
                current_biome,
                creature,
                local_conditions,
            );
            
            if let Some(best) = self.select_best_destination(candidates, creature) {
                return Some(MigrationDecision {
                    destination_biome: best.biome_type,
                    direction: best.direction,
                    urgency: pressure,
                    reason: self.determine_primary_reason(local_conditions),
                });
            }
        }
        
        None
    }
    
    fn calculate_total_pressure(&self, conditions: &LocalConditions) -> f32 {
        let resource_pressure = (100.0 - conditions.resource_availability) / 100.0;
        let competition_pressure = conditions.creature_density / 
            conditions.carrying_capacity;
        let environmental_pressure = conditions.environmental_stress;
        
        (resource_pressure * 0.4 + 
         competition_pressure * 0.4 + 
         environmental_pressure * 0.2).min(1.0)
    }
    
    fn find_candidate_biomes(
        &self,
        current: &Biome,
        creature: &Creature,
        conditions: &LocalConditions,
    ) -> Vec<BiomeCandidate> {
        let mut candidates = Vec::new();
        
        // Check adjacent biomes
        for (direction, adjacent) in current.get_adjacent_biomes() {
            let suitability = self.evaluate_biome_suitability(
                adjacent,
                creature,
                conditions.season,
            );
            
            // Consider memory of past visits
            let memory_bonus = creature.memory_system
                .recall_biome_quality(adjacent.biome_type)
                .unwrap_or(0.0) * self.resource_memory_weight;
            
            // Consider seasonal patterns
            let seasonal_bonus = self.get_seasonal_migration_bonus(
                current.biome_type,
                adjacent.biome_type,
                conditions.season,
            ) * self.seasonal_pattern_weight;
            
            let total_score = suitability + memory_bonus + seasonal_bonus;
            
            if total_score > conditions.resource_availability {
                candidates.push(BiomeCandidate {
                    biome_type: adjacent.biome_type,
                    direction,
                    score: total_score,
                    distance: adjacent.distance_to_edge,
                });
            }
        }
        
        candidates
    }
}
```

### Seasonal Resource Patterns
```rust
struct SeasonalResourcePattern {
    biome_type: BiomeType,
    season_transitions: [SeasonTransition; 4],
}

struct SeasonTransition {
    from_season: Season,
    to_season: Season,
    
    // Resource changes
    food_change: f32, // Multiplier
    water_change: f32,
    
    // Migration triggers
    triggers_migration: bool,
    migration_directions: Vec<CompassDirection>,
    
    // Special events
    special_spawns: Vec<SpecialSpawn>,
}

struct SpecialSpawn {
    resource_type: ResourceType,
    spawn_multiplier: f32,
    duration: Duration,
    locations: SpawnPattern,
}

const SEASONAL_PATTERNS: &[SeasonalResourcePattern] = &[
    SeasonalResourcePattern {
        biome_type: BiomeType::Grassland,
        season_transitions: [
            SeasonTransition {
                from_season: Season::Winter,
                to_season: Season::Spring,
                food_change: 2.5, // Spring growth burst
                water_change: 1.5, // Snow melt
                triggers_migration: true,
                migration_directions: vec![CompassDirection::North],
                special_spawns: vec![
                    SpecialSpawn {
                        resource_type: ResourceType::SpringFlowers,
                        spawn_multiplier: 5.0,
                        duration: Duration::days(30),
                        locations: SpawnPattern::Scattered,
                    },
                ],
            },
            // ... other transitions
        ],
    },
    // ... other biomes
];
```

## Balance Parameters

### Resource Regeneration
```rust
struct RegenerationConfig {
    biome_type: BiomeType,
    base_rates: HashMap<ResourceType, f32>,
    
    // Modifiers
    depletion_acceleration: f32, // Faster regen when depleted
    crowding_penalty: f32, // Slower regen when crowded
    seasonal_modifiers: [f32; 4],
    
    // Limits
    max_regeneration_rate: f32,
    regeneration_delay: Duration, // After full depletion
}

impl RegenerationConfig {
    fn calculate_regen_rate(
        &self,
        resource: &Resource,
        local_creatures: usize,
        season: Season,
    ) -> f32 {
        let base = self.base_rates.get(&resource.resource_type)
            .unwrap_or(&1.0);
        
        // Depletion bonus
        let depletion_ratio = 1.0 - (resource.current_amount / resource.max_amount);
        let depletion_bonus = 1.0 + (depletion_ratio * self.depletion_acceleration);
        
        // Crowding penalty
        let crowding_factor = 1.0 / (1.0 + local_creatures as f32 * self.crowding_penalty);
        
        // Seasonal modifier
        let seasonal = self.seasonal_modifiers[season as usize];
        
        let total_rate = base * depletion_bonus * crowding_factor * seasonal;
        
        total_rate.min(self.max_regeneration_rate)
    }
}
```

### Competition Dynamics
```rust
struct CompetitionModel {
    resource_access_radius: f32,
    dominance_factor: f32,
    group_advantage: f32,
}

impl CompetitionModel {
    fn allocate_resources(
        &self,
        resource: &Resource,
        competing_creatures: &[Creature],
    ) -> HashMap<CreatureId, f32> {
        let mut allocations = HashMap::new();
        
        // Calculate competition scores
        let mut scores: Vec<(CreatureId, f32)> = competing_creatures.iter()
            .map(|c| {
                let mut score = c.size * self.dominance_factor;
                
                // Group bonus
                if c.is_in_group() {
                    score *= 1.0 + self.group_advantage;
                }
                
                // Health/energy factor
                score *= c.health * c.energy / 100.0;
                
                (c.id, score)
            })
            .collect();
        
        // Sort by competition score
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Allocate resources
        let mut remaining = resource.current_amount;
        let total_score: f32 = scores.iter().map(|(_, s)| s).sum();
        
        for (creature_id, score) in scores {
            if remaining <= 0.0 {
                break;
            }
            
            let share = (score / total_score) * resource.current_amount;
            let allocated = share.min(remaining);
            
            allocations.insert(creature_id, allocated);
            remaining -= allocated;
        }
        
        allocations
    }
}
```

### Biome Interaction Effects
```rust
struct BiomeInteractionSystem {
    edge_effects: HashMap<(BiomeType, BiomeType), EdgeEffect>,
    transition_zones: Vec<TransitionZone>,
}

struct EdgeEffect {
    resource_bonus: f32, // Extra resources at edges
    biodiversity_multiplier: f32,
    special_resources: Vec<ResourceType>,
}

struct TransitionZone {
    from_biome: BiomeType,
    to_biome: BiomeType,
    width: f32,
    
    // Gradient functions
    resource_gradient: Gradient,
    difficulty_gradient: Gradient,
}

impl BiomeInteractionSystem {
    fn get_edge_resources(
        &self,
        position: Vec2,
        biome1: BiomeType,
        biome2: BiomeType,
    ) -> Vec<Resource> {
        let mut resources = Vec::new();
        
        if let Some(edge_effect) = self.edge_effects.get(&(biome1, biome2)) {
            // Spawn special edge resources
            for resource_type in &edge_effect.special_resources {
                let amount = BASE_AMOUNT * edge_effect.resource_bonus;
                resources.push(Resource::new(*resource_type, position, amount));
            }
        }
        
        resources
    }
    
    fn calculate_transition_properties(
        &self,
        position: Vec2,
        transition: &TransitionZone,
    ) -> TransitionProperties {
        let distance_from_edge = self.distance_to_biome_edge(position, transition);
        let normalized_distance = distance_from_edge / transition.width;
        
        TransitionProperties {
            resource_multiplier: transition.resource_gradient.evaluate(normalized_distance),
            movement_difficulty: transition.difficulty_gradient.evaluate(normalized_distance),
            visibility: 1.0 - normalized_distance * 0.3, // Reduced visibility in transitions
        }
    }
}
```

### Population Balance Targets
```rust
struct PopulationTargets {
    biome_targets: HashMap<BiomeType, BiomePopulationTarget>,
    global_target: GlobalPopulationTarget,
}

struct BiomePopulationTarget {
    min_density: f32, // Creatures per 100 tiles
    optimal_density: f32,
    max_density: f32,
    
    // Species diversity
    min_species: usize,
    optimal_species_distribution: HashMap<SpeciesType, f32>,
}

struct GlobalPopulationTarget {
    total_min: usize,
    total_optimal: usize,
    total_max: usize,
    
    // Distribution goals
    biome_distribution: HashMap<BiomeType, f32>, // Percentage
    migration_rate: f32, // Percentage migrating per season
}

impl PopulationTargets {
    fn evaluate_balance(&self, world: &World) -> BalanceReport {
        let mut report = BalanceReport::new();
        
        // Check each biome
        for (biome_type, target) in &self.biome_targets {
            let biome_stats = world.get_biome_statistics(*biome_type);
            let density = biome_stats.creature_count as f32 / biome_stats.area * 100.0;
            
            if density < target.min_density {
                report.add_issue(BalanceIssue::Underpopulated(*biome_type));
            } else if density > target.max_density {
                report.add_issue(BalanceIssue::Overpopulated(*biome_type));
            }
            
            // Check species diversity
            if biome_stats.species_count < target.min_species {
                report.add_issue(BalanceIssue::LowDiversity(*biome_type));
            }
        }
        
        // Check global balance
        let total_population = world.get_total_creature_count();
        if total_population < self.global_target.total_min {
            report.add_issue(BalanceIssue::GlobalUnderpopulation);
        }
        
        report
    }
}
```