# Resource System Design

## Overview

The resource system governs how food, water, and other consumables spawn, deplete, and regenerate throughout the world. This system directly impacts creature survival, migration patterns, and population dynamics.

## Resource Types

### Basic Resources
1. **Food** - Berries, nuts, grass, fruit
2. **Water** - Springs, pools, streams
3. **Shelter** - Caves, trees, rock formations

### Resource Properties
```rust
struct Resource {
    resource_type: ResourceType,
    current_amount: f32,
    max_amount: f32,
    quality: f32, // 0.0-1.0, affects satisfaction gain
    regeneration_rate: f32, // units per hour
    depletion_rate: f32, // units per consumption
    seasonal_modifier: f32, // 0.5-2.0 multiplier
}
```

## Spawning Algorithms

### Initial World Generation
Resources spawn during world generation based on biome type and terrain features:

```rust
fn calculate_resource_density(biome: &Biome, terrain: &TerrainTile) -> f32 {
    let base_density = match biome {
        Biome::Forest => 0.8,
        Biome::Grassland => 0.6,
        Biome::Desert => 0.2,
        Biome::Tundra => 0.3,
        Biome::Mountain => 0.4,
        Biome::Swamp => 0.5,
        Biome::RainForest => 0.9,
        Biome::Savanna => 0.5,
    };
    
    // Modify by terrain features
    let terrain_modifier = match terrain.elevation {
        0.0..=0.3 => 1.2, // Low areas collect resources
        0.3..=0.7 => 1.0, // Normal
        0.7..=1.0 => 0.7, // High areas sparse
        _ => 1.0,
    };
    
    base_density * terrain_modifier * terrain.moisture
}
```

### Resource Distribution
- **Clustering**: Resources spawn in clusters using Poisson disk sampling
- **Minimum Distance**: 2-5 tiles between same resource type
- **Maximum Cluster Size**: 3-12 based on biome richness

```rust
fn spawn_resource_cluster(
    center: Vec2,
    resource_type: ResourceType,
    biome: &Biome,
    world: &mut World
) {
    let cluster_size = rand_range(3, biome.max_cluster_size());
    let spread = rand_range(1.0, 3.0);
    
    for _ in 0..cluster_size {
        let offset = Vec2::new(
            rand_range(-spread, spread),
            rand_range(-spread, spread)
        );
        
        let position = center + offset;
        if is_valid_spawn_location(&position, &world) {
            spawn_resource(position, resource_type, biome.resource_quality());
        }
    }
}
```

## Depletion & Regeneration

### Consumption Mechanics
```rust
fn consume_resource(resource: &mut Resource, amount: f32) -> f32 {
    let consumed = amount.min(resource.current_amount);
    resource.current_amount -= consumed;
    
    // Start regeneration timer if depleted
    if resource.current_amount <= 0.0 {
        resource.start_regeneration();
    }
    
    consumed * resource.quality
}
```

### Regeneration Rules
1. **Immediate**: Water sources regenerate continuously
2. **Delayed**: Food sources require depletion before regenerating
3. **Seasonal**: Growth rates affected by current season

```rust
fn update_regeneration(resource: &mut Resource, delta_time: f32, season: Season) {
    if resource.is_regenerating {
        let seasonal_modifier = match season {
            Season::Spring => 1.5,
            Season::Summer => 1.2,
            Season::Autumn => 0.8,
            Season::Winter => 0.3,
        };
        
        let regen_amount = resource.regeneration_rate 
            * delta_time 
            * seasonal_modifier;
            
        resource.current_amount = (resource.current_amount + regen_amount)
            .min(resource.max_amount);
    }
}
```

### Dynamic Spawning
New resources spawn based on:
- **Carrying Capacity**: Each chunk has maximum resource count
- **Proximity**: New resources spawn near existing ones
- **Creature Deaths**: Deceased creatures become food resources

## Energy Costs

### Movement Costs
```rust
const BASE_MOVEMENT_COST: f32 = 0.1; // per tile

fn calculate_movement_cost(
    creature: &Creature,
    terrain: &TerrainTile,
    weather: &Weather
) -> f32 {
    let terrain_modifier = match terrain.terrain_type {
        TerrainType::Flat => 1.0,
        TerrainType::Hills => 1.5,
        TerrainType::Mountains => 2.0,
        TerrainType::Water => 3.0, // Swimming
        TerrainType::Swamp => 2.5,
    };
    
    let weather_modifier = match weather {
        Weather::Clear => 1.0,
        Weather::Rain => 1.2,
        Weather::Storm => 1.5,
        Weather::Snow => 1.8,
    };
    
    let age_modifier = match creature.life_stage {
        LifeStage::Infant => 0.5, // Carried, no cost
        LifeStage::Child => 1.2,
        LifeStage::Young => 1.0,
        LifeStage::Adult => 1.0,
        LifeStage::Elder => 1.4,
    };
    
    BASE_MOVEMENT_COST * terrain_modifier * weather_modifier * age_modifier
}
```

### Action Costs
```rust
const ACTION_COSTS: &[(Action, f32)] = &[
    (Action::Eat, 0.5),
    (Action::Drink, 0.3),
    (Action::Sleep, -2.0), // Negative = energy gain
    (Action::Reproduce, 20.0),
    (Action::Fight, 5.0),
    (Action::Communicate, 1.0),
    (Action::Forage, 2.0),
    (Action::Build, 3.0),
];
```

## Biome-Specific Resources

### Forest
- **Abundant**: Berries, nuts, mushrooms
- **Moderate**: Fresh water springs
- **Scarce**: Open shelter
- **Regeneration**: Fast (24-48 hours)

### Desert
- **Abundant**: Cacti (water + food)
- **Moderate**: Shade spots
- **Scarce**: Water sources
- **Regeneration**: Slow (96-144 hours)

### Tundra
- **Abundant**: Snow (water when melted)
- **Moderate**: Hardy plants, moss
- **Scarce**: Edible food
- **Regeneration**: Very slow (144-240 hours)

### Grassland
- **Abundant**: Grass, seeds
- **Moderate**: Water holes
- **Scarce**: Shelter
- **Regeneration**: Moderate (48-72 hours)

## Performance Considerations

### Spatial Indexing for Resources
```rust
struct ResourceGrid {
    cells: Vec<Vec<HashSet<ResourceId>>>,
    cell_size: f32,
}

impl ResourceGrid {
    fn get_resources_in_radius(&self, position: Vec2, radius: f32) -> Vec<ResourceId> {
        let min_cell = self.world_to_cell(position - Vec2::splat(radius));
        let max_cell = self.world_to_cell(position + Vec2::splat(radius));
        
        let mut resources = Vec::new();
        for x in min_cell.x..=max_cell.x {
            for y in min_cell.y..=max_cell.y {
                if let Some(cell) = self.get_cell(x, y) {
                    resources.extend(cell.iter());
                }
            }
        }
        resources
    }
}
```

### Update Batching
- Resources update in batches of 100
- Stagger updates across frames
- Skip updates for resources outside view distance

## Balancing Parameters

```rust
struct BiomeBalance {
    // Resources per 100x100 tile area
    min_resources: u32,
    max_resources: u32,
    
    // Regeneration time in hours
    food_regen_time: f32,
    water_regen_time: f32,
    
    // Quality ranges
    min_quality: f32,
    max_quality: f32,
    
    // Carrying capacity per creature
    creatures_per_resource: f32,
}

const BIOME_BALANCE: &[(Biome, BiomeBalance)] = &[
    (Biome::Forest, BiomeBalance {
        min_resources: 50,
        max_resources: 100,
        food_regen_time: 24.0,
        water_regen_time: 12.0,
        min_quality: 0.7,
        max_quality: 1.0,
        creatures_per_resource: 2.0,
    }),
    // ... other biomes
];
```

## Migration Triggers

Resources influence creature migration through:

1. **Depletion Threshold**: Migrate when local resources < 30%
2. **Competition Index**: Creatures per resource > threshold
3. **Seasonal Shifts**: Follow resource abundance patterns
4. **Memory Integration**: Remember productive areas

```rust
fn calculate_migration_pressure(
    creature: &Creature,
    local_resources: &[Resource],
    local_creatures: usize
) -> f32 {
    let resource_score = local_resources.iter()
        .map(|r| r.current_amount / r.max_amount)
        .sum::<f32>() / local_resources.len() as f32;
        
    let competition_score = local_creatures as f32 / 
        (local_resources.len() as f32 * CREATURES_PER_RESOURCE);
        
    let hunger_modifier = (100.0 - creature.hunger) / 100.0;
    
    (1.0 - resource_score) * competition_score * hunger_modifier
}
```