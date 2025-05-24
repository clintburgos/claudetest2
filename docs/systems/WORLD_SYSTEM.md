# World System

## Table of Contents
1. [World Generation](#world-generation)
2. [Biome Design](#biome-design)
3. [Resource Distribution](#resource-distribution)
4. [Environmental Systems](#environmental-systems)

---

## World Generation

### Procedural Generation System

The world is procedurally generated using layered noise functions to create realistic, diverse environments that support creature life.

```rust
pub struct WorldGenerator {
    seed: u64,
    size: IVec2,
    noise_layers: Vec<NoiseLayer>,
    biome_mapper: BiomeMapper,
}

pub struct NoiseLayer {
    noise_type: NoiseType,
    scale: f32,
    amplitude: f32,
    octaves: u32,
    persistence: f32,
    lacunarity: f32,
}

impl WorldGenerator {
    pub fn generate(&self) -> WorldData {
        let mut height_map = self.generate_height_map();
        let temperature_map = self.generate_temperature_map(&height_map);
        let moisture_map = self.generate_moisture_map(&height_map);
        
        // Generate biomes based on height, temperature, and moisture
        let biome_map = self.biome_mapper.map_biomes(
            &height_map,
            &temperature_map,
            &moisture_map,
        );
        
        // Place features
        let features = self.place_features(&biome_map);
        
        // Generate resource distribution
        let resources = self.distribute_resources(&biome_map);
        
        WorldData {
            seed: self.seed,
            size: self.size,
            height_map,
            biome_map,
            features,
            resources,
        }
    }
    
    fn generate_height_map(&self) -> HeightMap {
        let mut map = HeightMap::new(self.size);
        let mut rng = StdRng::seed_from_u64(self.seed);
        
        // Continental shelf
        let continental = PerlinNoise::new(&mut rng);
        
        // Mountain ranges  
        let mountains = RidgedNoise::new(&mut rng);
        
        // Local variation
        let detail = SimplexNoise::new(&mut rng);
        
        for y in 0..self.size.y {
            for x in 0..self.size.x {
                let pos = Vec2::new(x as f32, y as f32);
                
                // Combine noise layers
                let continental_height = continental.sample(pos * 0.001) * 0.6 + 0.4;
                let mountain_height = mountains.sample(pos * 0.003).max(0.0) * 0.4;
                let detail_height = detail.sample(pos * 0.01) * 0.1;
                
                let height = continental_height + mountain_height + detail_height;
                map.set(x, y, height);
            }
        }
        
        // Erosion pass
        self.apply_erosion(&mut map);
        
        map
    }
}
```

### Biome Mapping

```rust
pub struct BiomeMapper {
    biome_chart: BiomeChart,
    transition_width: f32,
}

impl BiomeMapper {
    pub fn map_biomes(
        &self,
        height: &HeightMap,
        temperature: &TemperatureMap,
        moisture: &MoistureMap,
    ) -> BiomeMap {
        let mut biome_map = BiomeMap::new(height.size());
        
        for y in 0..height.height() {
            for x in 0..height.width() {
                let h = height.get(x, y);
                let t = temperature.get(x, y);
                let m = moisture.get(x, y);
                
                // Determine base biome
                let biome = self.determine_biome(h, t, m);
                
                // Calculate transition weights to neighboring biomes
                let transitions = self.calculate_transitions(x, y, height, temperature, moisture);
                
                biome_map.set(x, y, BiomeCell {
                    primary_biome: biome,
                    transitions,
                });
            }
        }
        
        biome_map
    }
    
    fn determine_biome(&self, height: f32, temperature: f32, moisture: f32) -> BiomeType {
        // Water biomes
        if height < 0.15 {
            return BiomeType::DeepOcean;
        } else if height < 0.25 {
            return BiomeType::Ocean;
        } else if height < 0.35 {
            return BiomeType::Beach;
        }
        
        // Land biomes based on temperature and moisture
        match (temperature, moisture) {
            (t, m) if t < 0.2 => {
                if m < 0.3 { BiomeType::Tundra }
                else { BiomeType::Taiga }
            }
            (t, m) if t < 0.6 => {
                if m < 0.2 { BiomeType::Desert }
                else if m < 0.5 { BiomeType::Grassland }
                else if m < 0.7 { BiomeType::Forest }
                else { BiomeType::Swamp }
            }
            (t, m) => {
                if m < 0.3 { BiomeType::Desert }
                else if m < 0.6 { BiomeType::Savanna }
                else { BiomeType::Rainforest }
            }
        }
    }
}
```

---

## Biome Design

### Biome Types

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum BiomeType {
    // Aquatic
    DeepOcean,
    Ocean,
    Beach,
    
    // Cold
    Tundra,
    Taiga,
    
    // Temperate
    Forest,
    Grassland,
    
    // Hot
    Desert,
    Savanna,
    Rainforest,
    
    // Wetland
    Swamp,
    
    // Mountainous
    Mountain,
    Alpine,
}

impl BiomeType {
    pub fn properties(&self) -> BiomeProperties {
        match self {
            BiomeType::Forest => BiomeProperties {
                name: "Temperate Forest",
                base_color: Color::rgb(0.2, 0.5, 0.2),
                temperature_range: (10.0, 25.0),
                moisture_range: (0.4, 0.7),
                movement_speed_modifier: 0.8,
                visibility_range: 30.0,
                ambient_sound: "sounds/forest_ambience.ogg",
                spawn_rules: SpawnRules {
                    creature_density: 0.8,
                    preferred_species: vec![Species::Herbivore, Species::Omnivore],
                    plant_density: 0.9,
                    water_sources: 0.3,
                },
            },
            
            BiomeType::Desert => BiomeProperties {
                name: "Desert",
                base_color: Color::rgb(0.9, 0.8, 0.5),
                temperature_range: (20.0, 45.0),
                moisture_range: (0.0, 0.2),
                movement_speed_modifier: 0.9,
                visibility_range: 100.0,
                ambient_sound: "sounds/desert_wind.ogg",
                spawn_rules: SpawnRules {
                    creature_density: 0.2,
                    preferred_species: vec![Species::Carnivore],
                    plant_density: 0.1,
                    water_sources: 0.05,
                },
            },
            
            BiomeType::Swamp => BiomeProperties {
                name: "Swamp",
                base_color: Color::rgb(0.3, 0.4, 0.3),
                temperature_range: (15.0, 30.0),
                moisture_range: (0.7, 1.0),
                movement_speed_modifier: 0.5,
                visibility_range: 20.0,
                ambient_sound: "sounds/swamp_ambience.ogg",
                spawn_rules: SpawnRules {
                    creature_density: 0.6,
                    preferred_species: vec![Species::Omnivore],
                    plant_density: 1.0,
                    water_sources: 0.8,
                },
            },
            
            // ... other biomes
        }
    }
}
```

### Biome Features

```rust
pub struct BiomeFeatures {
    vegetation: Vec<VegetationType>,
    terrain_features: Vec<TerrainFeature>,
    hazards: Vec<EnvironmentalHazard>,
    special_resources: Vec<SpecialResource>,
}

pub enum VegetationType {
    // Trees
    OakTree { age: f32, size: f32 },
    PineTree { age: f32, size: f32 },
    PalmTree { age: f32, size: f32 },
    
    // Bushes
    BerryBush { berries: u32, ripeness: f32 },
    Shrub { density: f32 },
    
    // Ground cover
    Grass { height: f32, nutrition: f32 },
    Moss { coverage: f32 },
    Flowers { species: FlowerType, blooming: bool },
}

pub enum TerrainFeature {
    Rock { size: f32, climbable: bool },
    Cliff { height: f32, direction: Vec3 },
    Cave { depth: f32, occupied: bool },
    Stream { width: f32, flow_rate: f32 },
    Pond { radius: f32, depth: f32 },
}

impl BiomeFeatures {
    pub fn generate_for_biome(biome: BiomeType, area: Rect) -> Self {
        match biome {
            BiomeType::Forest => Self {
                vegetation: Self::generate_forest_vegetation(area),
                terrain_features: vec![
                    TerrainFeature::Stream { 
                        width: rand::gen_range(2.0..5.0),
                        flow_rate: 0.5,
                    },
                    TerrainFeature::Rock {
                        size: rand::gen_range(1.0..3.0),
                        climbable: true,
                    },
                ],
                hazards: vec![],
                special_resources: vec![
                    SpecialResource::MedicinalHerb,
                    SpecialResource::RareFlower,
                ],
            },
            
            BiomeType::Desert => Self {
                vegetation: vec![
                    VegetationType::Cactus { 
                        water_content: rand::gen_range(0.3..0.7) 
                    },
                ],
                terrain_features: vec![
                    TerrainFeature::Rock {
                        size: rand::gen_range(2.0..6.0),
                        climbable: false,
                    },
                    TerrainFeature::Oasis {
                        radius: rand::gen_range(5.0..15.0),
                    },
                ],
                hazards: vec![
                    EnvironmentalHazard::Sandstorm {
                        frequency: 0.1,
                        duration: 120.0,
                    },
                    EnvironmentalHazard::ExtremeHeat,
                ],
                special_resources: vec![
                    SpecialResource::CactusFlower,
                ],
            },
            
            // ... other biomes
        }
    }
}
```

---

## Resource Distribution

### Resource System

```rust
pub struct ResourceDistribution {
    resource_maps: HashMap<ResourceType, ResourceMap>,
    regeneration_rates: HashMap<ResourceType, f32>,
    seasonal_modifiers: HashMap<(Season, ResourceType), f32>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceType {
    // Food
    Berries,
    Nuts,
    Seeds,
    Grass,
    Leaves,
    Fruit,
    Insects,
    Fish,
    
    // Water
    FreshWater,
    StreamWater,
    
    // Materials
    Wood,
    Stone,
    Fiber,
    
    // Special
    MedicinalPlant,
    NestingMaterial,
}

pub struct ResourceNode {
    pub resource_type: ResourceType,
    pub position: Vec3,
    pub quantity: f32,
    pub max_quantity: f32,
    pub quality: f32,
    pub regeneration_rate: f32,
    pub last_harvested: Option<f64>,
}

impl ResourceDistribution {
    pub fn distribute_resources(
        &mut self,
        biome_map: &BiomeMap,
        world_size: IVec2,
    ) {
        for resource_type in ResourceType::all() {
            let distribution = self.calculate_distribution(resource_type, biome_map);
            
            let mut resource_map = ResourceMap::new(world_size);
            
            for (position, density) in distribution {
                if rand::random::<f32>() < density {
                    let node = self.create_resource_node(resource_type, position, biome_map);
                    resource_map.add_node(node);
                }
            }
            
            self.resource_maps.insert(resource_type, resource_map);
        }
    }
    
    fn calculate_distribution(
        &self,
        resource: ResourceType,
        biome_map: &BiomeMap,
    ) -> HashMap<Vec3, f32> {
        let mut distribution = HashMap::new();
        
        match resource {
            ResourceType::Berries => {
                // Berries prefer forest edges and clearings
                for (pos, biome) in biome_map.iter() {
                    let density = match biome.primary_biome {
                        BiomeType::Forest => 0.7,
                        BiomeType::Grassland => 0.4,
                        BiomeType::Taiga => 0.3,
                        _ => 0.0,
                    };
                    
                    // Increase density at biome transitions
                    let edge_bonus = if biome.is_edge() { 0.2 } else { 0.0 };
                    
                    distribution.insert(pos, density + edge_bonus);
                }
            }
            
            ResourceType::FreshWater => {
                // Water sources based on moisture and terrain
                for (pos, biome) in biome_map.iter() {
                    let base_chance = biome.moisture;
                    
                    // Lower areas more likely to have water
                    let height_modifier = 1.0 - biome.height.clamp(0.0, 1.0);
                    
                    distribution.insert(pos, base_chance * height_modifier);
                }
            }
            
            // ... other resources
        }
        
        distribution
    }
}
```

### Resource Balance

Each biome has carefully balanced resources to support different creature populations:

```rust
pub struct BiomeResourceBalance {
    pub carrying_capacity: CarryingCapacity,
    pub resource_ratios: ResourceRatios,
    pub seasonal_variation: SeasonalVariation,
}

pub struct CarryingCapacity {
    pub herbivores: f32,
    pub carnivores: f32,
    pub omnivores: f32,
}

impl BiomeResourceBalance {
    pub fn calculate_for_biome(biome: BiomeType) -> Self {
        match biome {
            BiomeType::Forest => Self {
                carrying_capacity: CarryingCapacity {
                    herbivores: 100.0,
                    carnivores: 20.0,
                    omnivores: 40.0,
                },
                resource_ratios: ResourceRatios {
                    plant_food: 0.6,
                    animal_food: 0.2,
                    water: 0.15,
                    shelter: 0.05,
                },
                seasonal_variation: SeasonalVariation {
                    spring: ResourceModifier { food: 1.2, water: 1.1 },
                    summer: ResourceModifier { food: 1.0, water: 0.9 },
                    autumn: ResourceModifier { food: 1.3, water: 1.0 },
                    winter: ResourceModifier { food: 0.4, water: 1.0 },
                },
            },
            
            BiomeType::Desert => Self {
                carrying_capacity: CarryingCapacity {
                    herbivores: 20.0,
                    carnivores: 10.0,
                    omnivores: 15.0,
                },
                resource_ratios: ResourceRatios {
                    plant_food: 0.2,
                    animal_food: 0.1,
                    water: 0.05,
                    shelter: 0.65,
                },
                seasonal_variation: SeasonalVariation {
                    spring: ResourceModifier { food: 1.1, water: 0.8 },
                    summer: ResourceModifier { food: 0.6, water: 0.3 },
                    autumn: ResourceModifier { food: 0.8, water: 0.5 },
                    winter: ResourceModifier { food: 0.9, water: 1.2 },
                },
            },
            
            // ... other biomes
        }
    }
    
    pub fn adjust_for_season(&self, season: Season) -> ResourceAvailability {
        let modifier = match season {
            Season::Spring => &self.seasonal_variation.spring,
            Season::Summer => &self.seasonal_variation.summer,
            Season::Autumn => &self.seasonal_variation.autumn,
            Season::Winter => &self.seasonal_variation.winter,
        };
        
        ResourceAvailability {
            food_multiplier: modifier.food,
            water_multiplier: modifier.water,
            shelter_quality: self.calculate_shelter_quality(season),
        }
    }
}
```

---

## Environmental Systems

### Weather System

```rust
pub struct WeatherSystem {
    current_weather: Weather,
    weather_forecast: VecDeque<Weather>,
    transition_progress: f32,
    regional_patterns: HashMap<BiomeType, WeatherPattern>,
}

#[derive(Clone)]
pub enum Weather {
    Clear { 
        temperature: f32,
        wind_speed: f32,
    },
    Cloudy {
        cloud_coverage: f32,
        temperature: f32,
    },
    Rain {
        intensity: f32,
        temperature: f32,
        duration: f32,
    },
    Storm {
        intensity: f32,
        lightning_frequency: f32,
        wind_speed: f32,
    },
    Snow {
        intensity: f32,
        accumulation: f32,
        temperature: f32,
    },
    Fog {
        density: f32,
        visibility: f32,
    },
}

impl WeatherSystem {
    pub fn update(&mut self, delta_time: f32, season: Season, biomes: &[BiomeType]) {
        // Progress weather transitions
        if let Some(next_weather) = self.weather_forecast.front() {
            self.transition_progress += delta_time / WEATHER_TRANSITION_TIME;
            
            if self.transition_progress >= 1.0 {
                self.current_weather = self.weather_forecast.pop_front().unwrap();
                self.transition_progress = 0.0;
            }
        }
        
        // Generate new weather if forecast is running low
        if self.weather_forecast.len() < 3 {
            self.generate_weather_forecast(season, biomes);
        }
    }
    
    fn generate_weather_forecast(&mut self, season: Season, biomes: &[BiomeType]) {
        let dominant_biome = self.find_dominant_biome(biomes);
        let pattern = &self.regional_patterns[&dominant_biome];
        
        for _ in 0..5 {
            let weather = pattern.generate_weather(season, &self.current_weather);
            self.weather_forecast.push_back(weather);
        }
    }
    
    pub fn get_weather_effects(&self) -> WeatherEffects {
        match &self.current_weather {
            Weather::Rain { intensity, .. } => WeatherEffects {
                movement_modifier: 1.0 - intensity * 0.2,
                visibility_modifier: 1.0 - intensity * 0.3,
                mood_modifier: -intensity * 0.1,
                water_availability: intensity * 2.0,
            },
            Weather::Storm { intensity, wind_speed, .. } => WeatherEffects {
                movement_modifier: 1.0 - intensity * 0.4,
                visibility_modifier: 1.0 - intensity * 0.6,
                mood_modifier: -intensity * 0.3,
                danger_level: intensity * 0.5,
            },
            Weather::Clear { temperature, .. } => WeatherEffects {
                movement_modifier: 1.0,
                visibility_modifier: 1.0,
                mood_modifier: 0.1,
                temperature_stress: (temperature - 20.0).abs() / 30.0,
            },
            // ... other weather types
        }
    }
}
```

### Day/Night Cycle

```rust
pub struct DayNightCycle {
    pub time_of_day: f32,  // 0.0 = midnight, 0.5 = noon
    pub day_length: f32,   // seconds per day
    pub season: Season,
}

impl DayNightCycle {
    pub fn update(&mut self, delta_time: f32) {
        self.time_of_day += delta_time / self.day_length;
        self.time_of_day = self.time_of_day.fract();
    }
    
    pub fn get_light_level(&self) -> f32 {
        // Simple sine wave for light level
        let sun_angle = self.time_of_day * std::f32::consts::TAU;
        let base_light = (sun_angle.sin() + 1.0) / 2.0;
        
        // Adjust for season (longer days in summer)
        let seasonal_modifier = match self.season {
            Season::Summer => 1.2,
            Season::Winter => 0.8,
            _ => 1.0,
        };
        
        (base_light * seasonal_modifier).clamp(0.1, 1.0)
    }
    
    pub fn get_temperature_modifier(&self) -> f32 {
        // Temperature peaks in afternoon
        let time_shifted = (self.time_of_day - 0.125).fract();
        let temp_curve = (time_shifted * std::f32::consts::TAU).sin();
        
        match self.season {
            Season::Summer => 20.0 + temp_curve * 15.0,
            Season::Winter => 0.0 + temp_curve * 10.0,
            Season::Spring | Season::Autumn => 15.0 + temp_curve * 10.0,
        }
    }
    
    pub fn is_night(&self) -> bool {
        self.time_of_day < 0.25 || self.time_of_day > 0.75
    }
}
```

### Seasonal Changes

```rust
pub struct SeasonalSystem {
    pub current_season: Season,
    pub season_progress: f32,
    pub year_length: f32,  // days per year
}

#[derive(Clone, Copy, PartialEq)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

impl SeasonalSystem {
    pub fn update(&mut self, delta_time: f32) {
        self.season_progress += delta_time / (self.year_length * 86400.0 / 4.0);
        
        if self.season_progress >= 1.0 {
            self.season_progress = 0.0;
            self.current_season = self.next_season();
            self.trigger_seasonal_changes();
        }
    }
    
    fn next_season(&self) -> Season {
        match self.current_season {
            Season::Spring => Season::Summer,
            Season::Summer => Season::Autumn,
            Season::Autumn => Season::Winter,
            Season::Winter => Season::Spring,
        }
    }
    
    fn trigger_seasonal_changes(&self) {
        match self.current_season {
            Season::Spring => {
                // Trigger plant growth
                // Increase animal reproduction
                // Melt snow
            }
            Season::Summer => {
                // Peak resource availability
                // Highest temperatures
            }
            Season::Autumn => {
                // Food abundance (harvest)
                // Preparation behaviors
                // Leaf color changes
            }
            Season::Winter => {
                // Resource scarcity
                // Migration triggers
                // Snow accumulation
            }
        }
    }
    
    pub fn get_seasonal_effects(&self) -> SeasonalEffects {
        match self.current_season {
            Season::Spring => SeasonalEffects {
                plant_growth_rate: 1.5,
                reproduction_modifier: 1.3,
                food_abundance: 0.8,
                temperature_range: (5.0, 20.0),
            },
            Season::Summer => SeasonalEffects {
                plant_growth_rate: 1.0,
                reproduction_modifier: 1.0,
                food_abundance: 1.0,
                temperature_range: (15.0, 35.0),
            },
            Season::Autumn => SeasonalEffects {
                plant_growth_rate: 0.5,
                reproduction_modifier: 0.7,
                food_abundance: 1.2,
                temperature_range: (5.0, 20.0),
            },
            Season::Winter => SeasonalEffects {
                plant_growth_rate: 0.0,
                reproduction_modifier: 0.2,
                food_abundance: 0.3,
                temperature_range: (-10.0, 10.0),
            },
        }
    }
}
```

### Natural Disasters

```rust
pub struct DisasterSystem {
    pub active_disasters: Vec<ActiveDisaster>,
    pub disaster_probability: HashMap<(BiomeType, Season), f32>,
}

pub enum DisasterType {
    Wildfire {
        spread_rate: f32,
        intensity: f32,
    },
    Flood {
        water_level: f32,
        flow_force: f32,
    },
    Drought {
        severity: f32,
        duration: f32,
    },
    Earthquake {
        magnitude: f32,
        aftershocks: u32,
    },
}

impl DisasterSystem {
    pub fn check_disaster_conditions(
        &self,
        biome: BiomeType,
        weather: &Weather,
        season: Season,
    ) -> Option<DisasterType> {
        let base_probability = self.disaster_probability
            .get(&(biome, season))
            .copied()
            .unwrap_or(0.01);
        
        // Adjust probability based on conditions
        let probability = match (biome, weather, season) {
            (BiomeType::Forest, Weather::Clear { temperature, .. }, Season::Summer) 
                if *temperature > 35.0 => {
                // High wildfire risk
                base_probability * 5.0
            }
            (_, Weather::Rain { intensity, .. }, _) if *intensity > 0.8 => {
                // Flood risk
                base_probability * 3.0
            }
            _ => base_probability,
        };
        
        if rand::random::<f32>() < probability {
            self.generate_disaster(biome, weather, season)
        } else {
            None
        }
    }
}
```