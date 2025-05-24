# Time Scaling System

Time scaling allows players to control the simulation speed from real-time to extreme fast-forward, enabling both detailed observation and long-term evolution viewing.

## Time Scale Settings

### Available Speeds

| Speed | Multiplier | Primary Use | Update Strategy |
|-------|------------|-------------|-----------------|
| Pause | 0x | Detailed observation | No updates |
| Normal | 1x | Real-time watching | Full simulation |
| Fast | 5x | Active gameplay | Full simulation |
| Faster | 20x | Day/night cycles | Reduced animations |
| Very Fast | 100x | Generation viewing | Simplified behaviors |
| Ultra Fast | 500x | Long-term evolution | Statistical only |
| Maximum | 1000x | Geological time | Pure statistics |

## Scaling Strategies

### 1x - Normal Speed
- Full physics simulation
- All animations play
- Complete AI decision trees
- Real-time creature vocalizations
- Detailed particle effects

### 5x - Fast Speed
- Full simulation maintained
- Animation frame skipping
- Grouped UI updates
- Batched sound effects

### 20x - Faster Speed
- Simplified pathfinding (direct routes)
- Basic animation states only
- Reduced decision complexity
- Statistical combat resolution
- Batched creature updates

### 100x - Very Fast
- Movement becomes teleportation
- No animations (static sprites)
- Needs-based AI only
- Birth/death events only
- Major events logged

### 500x - Ultra Fast
- Population-level simulation
- Statistical breeding
- Resource consumption models
- Climate patterns accelerated
- Evolution tracking

### 1000x - Maximum Speed
- Pure statistical simulation
- Population dynamics only
- Genetic drift modeling
- Biome succession
- Mass extinction events

## System-Specific Scaling

### Movement System
```
1x: Physics-based movement with collision
5x: Simplified physics, grouped updates
20x: Direct movement, no physics
100x+: Teleportation between points
```

### AI Decision System
```
1x: Full behavior trees, all goals evaluated
5x: Priority goals only
20x: Basic needs only
100x+: Statistical behavior distribution
```

### Animation System
```
1x: Full skeletal animation
5x: Keyframe reduction
20x: State-based sprites only
100x+: No animation
```

### Reproduction System
```
1x: Full courtship, mating, pregnancy
5x: Shortened rituals
20x: Quick mating checks
100x+: Statistical reproduction rates
```

## Performance Optimizations

### Frame Skipping
At higher speeds, not every simulation tick needs rendering:
- 1-5x: Render every frame
- 20x: Render every 4th frame
- 100x: Render every 20th frame
- 500x+: Render once per second

### Update Batching
Group similar operations:
- Creature updates processed in parallel
- Spatial queries cached longer
- UI updates consolidated

### LOD Integration
Time scaling multiplies with distance-based LOD:
- Near + fast time = medium LOD
- Far + fast time = minimal LOD
- Far + ultra fast = statistical only

## Time Perception

### Day/Night Cycle
- 1x: 24 minute full cycle
- 5x: ~5 minute cycle
- 20x: ~1 minute cycle
- 100x: Rapid strobing (smoothed)

### Seasons
- 1x: 96 minutes per season
- 20x: ~5 minutes per season
- 100x: ~1 minute per season
- 1000x: Seconds per season

### Generations
- 1x: Hours per generation
- 100x: Minutes per generation
- 1000x: Seconds per generation

## Player Experience

### Smooth Transitions
- Speed changes interpolate over 0.5 seconds
- Creature positions smoothly interpolated
- Sound effects fade appropriately
- UI adapts to show relevant info

### Information Display
Different UI at different speeds:
- 1-5x: Individual creature details
- 20-100x: Population statistics
- 500x+: Evolution graphs, genetic drift

### Event Notifications
Important events shown regardless of speed:
- Births and deaths (aggregated at high speed)
- Environmental changes
- Evolution milestones
- Population crashes

## Technical Implementation

### Fixed vs Variable Timestep
- 1-20x: Fixed timestep with interpolation
- 100x+: Variable timestep with maximum delta

### Update Scheduling
Systems update at different rates:
```
Core (always): Position, health, age
Frequent: Hunger, thirst, energy
Periodic: Social bonds, learning
Rare: Genetic mutations, culture
```

### State Synchronization
At high speeds, ensure consistency:
- Atomic updates for related systems
- Event queues for order-dependent operations
- Rollback for constraint violations