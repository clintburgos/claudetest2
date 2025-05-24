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