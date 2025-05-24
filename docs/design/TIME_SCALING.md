# Time Scaling System

## Overview
The time scaling system allows users to observe the simulation at different temporal resolutions, from individual actions to evolutionary changes.

## Time Scale Levels

### 1. Action Time (1x)
- **Resolution**: Individual decisions/actions
- **Use Case**: Observe detailed behavior
- **Updates**: Every creature, every frame
- **Visualization**: Full detail, all animations

### 2. Day Cycle (60x)
- **Resolution**: Daily patterns
- **Use Case**: Resource cycles, sleep patterns
- **Updates**: Batch similar actions
- **Visualization**: Simplified movement, trails

### 3. Life Cycle (1000x) 
- **Resolution**: Birth to death
- **Use Case**: Individual development
- **Updates**: Major life events only
- **Visualization**: Growth stages, key moments

### 4. Generational (10000x)
- **Resolution**: Population generations
- **Use Case**: Evolution observation
- **Updates**: Birth/death statistics
- **Visualization**: Population graphs, trait distribution

### 5. Epoch (100000x)
- **Resolution**: Long-term changes
- **Use Case**: Species emergence
- **Updates**: Major population shifts
- **Visualization**: Heat maps, phylogenetic trees

## Implementation Strategy

### Level of Detail (LOD) System
```rust
enum LODLevel {
    Full,        // All systems active
    Simplified,  // Batch decisions
    Statistical, // Population-level only
    Historical,  // Major events only
}

fn determine_lod(time_scale: f32) -> LODLevel {
    match time_scale {
        0.0..=100.0 => LODLevel::Full,
        100.0..=5000.0 => LODLevel::Simplified,
        5000.0..=50000.0 => LODLevel::Statistical,
        _ => LODLevel::Historical
    }
}
```

### System Optimizations

#### At Simplified LOD:
- Group similar creatures
- Batch pathfinding
- Simplify conversations
- Aggregate needs updates

#### At Statistical LOD:
- Population-level simulation
- Probabilistic outcomes
- Trait distribution tracking
- Summary statistics only

#### At Historical LOD:
- Major event logging
- Evolutionary milestones
- Environmental changes
- Species emergence/extinction

## Smooth Transitions

### Interpolation Strategy
1. Detect scale change request
2. Snapshot current state
3. Calculate target state
4. Interpolate over 2-3 seconds
5. Adjust LOD progressively

### Visual Continuity
- Creature positions interpolated
- Population counts smoothly adjusted
- UI elements fade in/out
- Camera zoom automated

## Data Recording

### Event Logging
- Birth/death events
- Significant conversations
- Environmental changes
- Evolutionary milestones

### Compression Strategy
- Full detail last 1000 ticks
- Sampled data last 10000 ticks
- Summary statistics forever
- Key events always retained

---
*Last Updated: 2024-01-XX*
