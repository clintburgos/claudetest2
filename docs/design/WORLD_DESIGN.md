# World Design Specification

## Overview
The creature simulation takes place in a procedurally generated isometric world with diverse biomes, animated environments, and realistic transitions between regions.

## World Generation

### Procedural Generation System
- **Seed-based**: Each world has a unique seed for reproducibility
- **Multi-layer Noise**: 
  - Elevation (Perlin noise, octaves: 6, persistence: 0.5)
  - Temperature (Perlin noise, octaves: 4, persistence: 0.6)
  - Moisture (Perlin noise, octaves: 5, persistence: 0.55)
  - Vegetation density (derived from moisture + temperature)

### Biome Determination
Biomes are determined by the intersection of:
1. **Elevation**: Sea level to mountain peaks
2. **Temperature**: Arctic to tropical
3. **Moisture**: Desert to rainforest

## Biome Types

### 1. Grassland Plains
- **Characteristics**:
  - Moderate temperature and moisture
  - Rolling hills with tall grass
  - Scattered wildflowers
  - Occasional lone trees
- **Tile Variations**: 
  - Short grass (5 variants)
  - Tall grass (5 variants)
  - Flower patches (8 types)
  - Rock formations (3 types)
- **Animations**:
  - Grass sways in wind (sine wave displacement)
  - Flowers bob gently
  - Butterflies float between flowers
  - Wind ripples across grass

### 2. Dense Forest
- **Characteristics**:
  - High moisture, moderate temperature
  - Canopy layer creates shadows
  - Thick undergrowth
  - Fallen logs and mushrooms
- **Tile Variations**:
  - Tree types (oak, birch, pine - 4 variants each)
  - Undergrowth (ferns, bushes - 6 variants)
  - Forest floor (leaves, moss - 8 variants)
  - Clearings (3 types)
- **Animations**:
  - Tree branches sway (hierarchical animation)
  - Leaves fall occasionally
  - Dappled sunlight shifts
  - Small animals scurry in undergrowth

### 3. Desert
- **Characteristics**:
  - High temperature, low moisture
  - Sand dunes and rock formations
  - Cacti and hardy plants
  - Oasis rare spawn points
- **Tile Variations**:
  - Sand (6 patterns)
  - Rock formations (5 types)
  - Cacti (4 species, 3 sizes each)
  - Desert plants (6 types)
- **Animations**:
  - Heat shimmer effect
  - Sand particles blow
  - Cacti flowers open/close with day cycle
  - Tumbleweeds roll by
### 4. Tundra
- **Characteristics**:
  - Low temperature, moderate moisture
  - Snow-covered ground
  - Sparse vegetation
  - Ice formations
- **Tile Variations**:
  - Snow (5 patterns, 3 depths)
  - Ice patches (4 types)
  - Tundra plants (4 types)
  - Rock outcrops (3 types)
- **Animations**:
  - Snow particles drift
  - Ice crystals sparkle
  - Aurora borealis at night
  - Frost forms on objects

### 5. Tropical Rainforest
- **Characteristics**:
  - High temperature, high moisture
  - Multi-layer canopy
  - Extreme biodiversity
  - Rivers and waterfalls
- **Tile Variations**:
  - Giant trees (6 types)
  - Vines and lianas (8 patterns)
  - Tropical plants (12 types)
  - Water features (streams, pools)
- **Animations**:
  - Rain drips from leaves
  - Mist rises from ground
  - Exotic birds fly between trees
  - Vines sway slightly

### 6. Wetlands/Swamp
- **Characteristics**:
  - Variable temperature, high moisture
  - Standing water with islands
  - Mangroves and reeds
  - Rich wildlife sounds
- **Tile Variations**:
  - Water (murky, clear - 4 variants)
  - Lily pads (3 types)
  - Reeds and cattails (5 types)
  - Mud banks (4 patterns)
- **Animations**:
  - Water ripples
  - Fireflies at dusk
  - Fog rolls across water
  - Dragonflies hover

### 7. Mountain
- **Characteristics**:
  - High elevation, variable temperature
  - Rocky terrain
  - Alpine meadows
  - Snow caps at peaks
- **Tile Variations**:
  - Rock faces (8 types)
  - Alpine grass (4 types)
  - Mountain flowers (6 types)
  - Snow patches (variable coverage)
- **Animations**:
  - Clouds pass by peaks
  - Small rockfalls
  - Eagles soar
  - Snow blows off peaks
### 8. Beach/Coastal
- **Characteristics**:
  - Transition between land and water
  - Sandy shores
  - Tidal pools
  - Coastal vegetation
- **Tile Variations**:
  - Sand (wet/dry - 6 patterns)
  - Rocks and shells (8 types)
  - Beach grass (4 types)
  - Tidal pools (3 types)
- **Animations**:
  - Waves lap shore
  - Seabirds fly
  - Crabs scuttle
  - Foam bubbles pop

## Biome Transitions

### Transition System
- **Gradient Zones**: 3-5 tiles wide
- **Blend Factors**: Based on distance to biome center
- **Mixed Tiles**: Combine elements from adjacent biomes

### Transition Rules
1. **Temperature Gradients**:
   - Desert → Grassland: Sparse grass increases
   - Grassland → Tundra: Grass yellows, snow patches appear
   - Forest → Rainforest: Trees become denser, larger

2. **Moisture Gradients**:
   - Desert → Wetlands: Dry riverbed → stream → swamp
   - Grassland → Forest: Trees gradually increase
   - Beach → Any: Sand gives way to biome-specific ground

3. **Elevation Transitions**:
   - Any → Mountain: Vegetation becomes sparser
   - Mountain → Tundra: Snow line appears
   - Beach → Ocean: Depth gradients visible

## Tile System

### Isometric Tile Specifications
- **Base Size**: 64x32 pixels (or 128x64 for high-res)
- **Height Levels**: 8 levels (0-7)
- **Tile Layers**:
  1. Base terrain
  2. Decoration layer (rocks, plants)
  3. Animated elements
  4. Particle effects

### Tile Rendering
```
Rendering Order (back to front):
1. Ground tiles (elevation-sorted)
2. Ground decorations
3. Tall objects (trees, rocks)
4. Creatures
5. Weather/particle effects
6. UI elements
```
## Environmental Animations

### Wind System
- **Global Wind**: Direction and strength affect all biomes
- **Local Variations**: Biome-specific wind patterns
- **Effects**:
  - Grass displacement: `offset = sin(time + x * frequency) * windStrength`
  - Tree sway: Hierarchical bone animation
  - Particle direction: Leaves, sand, snow follow wind
  - Creature fur/features react to wind

### Day/Night Cycle
- **Duration**: 10-15 minutes real-time per day
- **Lighting Changes**:
  - Dawn: Orange/pink hues, long shadows
  - Day: Bright, short shadows
  - Dusk: Purple/orange sky, long shadows
  - Night: Blue tint, reduced visibility
- **Biome-Specific**:
  - Desert: Extreme temperature changes
  - Rainforest: Morning mist
  - Tundra: Aurora at night
  - Wetlands: Fireflies at dusk

### Weather System
- **Weather Types**:
  - Clear (60% chance)
  - Cloudy (20% chance)
  - Rain (15% chance - not in desert)
  - Snow (5% chance - cold biomes only)
  - Storms (rare events)
- **Weather Effects**:
  - Rain: Puddles form, plants droop, visibility reduced
  - Snow: Accumulates on surfaces, footprints visible
  - Wind: Increases during storms
  - Lightning: Rare, affects creature behavior

## Resource Distribution

### Food Sources
- **Grassland**: Berries, seeds, insects
- **Forest**: Nuts, mushrooms, fruits
- **Desert**: Cacti fruit, roots, insects
- **Tundra**: Lichen, berries (seasonal)
- **Rainforest**: Abundant fruits, nectar
- **Wetlands**: Fish, aquatic plants
- **Mountain**: Hardy plants, birds' eggs
- **Beach**: Shellfish, seaweed

### Water Sources
- **Natural**: Rivers, lakes, oasis, rain puddles
- **Biome-Specific**:
  - Desert: Rare oasis, morning dew
  - Wetlands: Abundant but may be brackish
  - Mountain: Fresh springs
  - Tundra: Snow (requires energy to melt)
### Shelter Locations
- **Natural**: Caves, tree hollows, rock overhangs
- **Biome-Specific**:
  - Forest: Dense canopy areas
  - Desert: Rock formations, shade
  - Mountain: Caves, crevices
  - Beach: Driftwood shelters

## Performance Optimization

### Level of Detail (LOD)
- **Near** (< 10 tiles): Full animation, all particles
- **Medium** (10-25 tiles): Reduced animation frequency
- **Far** (25-50 tiles): Static sprites, no particles
- **Distant** (> 50 tiles): Simplified colors, fog effect

### Animation Optimization
- **Staggered Updates**: Animations offset by tile position
- **Culling**: Only animate visible tiles
- **Shared Animations**: Multiple tiles reference same animation
- **Quality Settings**:
  - Low: 25% animations active
  - Medium: 50% animations active
  - High: 75% animations active
  - Ultra: 100% animations active

### Chunk System
- **Chunk Size**: 16x16 tiles
- **Active Radius**: 3 chunks around camera
- **Loading**: Progressive, priority based on camera movement
- **Memory**: Unload distant chunks, cache recent

## Biome Effects on Creatures

### Temperature Effects
- **Hot** (Desert, Tropical): Increased thirst, seek shade
- **Cold** (Tundra, Mountain): Increased hunger, huddle together
- **Moderate**: Optimal comfort, normal behavior

### Terrain Effects
- **Movement Speed**:
  - Grassland: 100% (baseline)
  - Forest: 80% (obstacles)
  - Desert Sand: 70% (soft ground)
  - Wetlands: 60% (water/mud)
  - Mountain: 50-90% (elevation-based)
  - Snow: 65% (deep snow)

### Visibility
- **Clear**: Grassland, Desert (day), Beach
- **Moderate**: Mountain, Tundra
- **Limited**: Forest, Rainforest, Wetlands
- **Weather-Affected**: All biomes during rain/snow

### Resource Availability
Affects creature behavior and population density:
- **Abundant** (Rainforest, Wetlands): Higher population
- **Moderate** (Grassland, Forest): Balanced ecosystem
- **Scarce** (Desert, Tundra): Lower population, nomadic behavior

---
*Last Updated: 2024-12-XX*
*See also: [UI_DESIGN.md](./UI_DESIGN.md) for visual implementation details*