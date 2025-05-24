# Performance Targets and Constraints

This document establishes the definitive performance targets for the creature simulation project, resolving inconsistencies across documentation.

## Core Performance Requirements

### Creature Population Targets

| Population Size | Performance Target | LOD Level | Update Frequency |
|----------------|-------------------|-----------|------------------|
| 0-100          | 144+ FPS         | Full      | Every frame      |
| 100-500        | 120+ FPS         | Full      | Every frame      |
| 500-1000       | 90+ FPS          | High      | Every frame      |
| 1000-2500      | 60+ FPS          | Medium    | Every 2 frames   |
| 2500-5000      | 60+ FPS          | Low       | Every 4 frames   |
| 5000+          | 30+ FPS          | Minimal   | Statistical      |

**Definitive Target**: Support up to **5000 active creatures** at 60+ FPS on mid-range hardware (GTX 1060/RX 580 class GPU, 6-core CPU, 16GB RAM).

### Memory Budget Allocation

Total Memory Budget: **2GB** (leaving 2GB for OS and other applications on 4GB systems)

| Component | Allocation | Per-Creature (5000) | Notes |
|-----------|------------|-------------------|--------|
| Creatures | 1000MB (50%) | 200KB | All creature data including components |
| World | 400MB (20%) | - | Terrain, resources, spatial indices |
| UI/Rendering | 300MB (15%) | - | Textures, meshes, UI state |
| Systems | 200MB (10%) | - | Caches, buffers, working memory |
| Reserve | 100MB (5%) | - | Dynamic allocations, spikes |

### Per-Creature Memory Breakdown (200KB)

```
Core Components (80KB):
- Transform: 48 bytes
- Genetics: 2KB
- Health/Needs: 1KB
- Memory (compressed): 16KB
- Relationships: 8KB
- Current State: 4KB
- Misc Components: ~50KB

Dynamic Allocations (120KB):
- Active Conversations: 40KB (when engaged)
- Pathfinding Cache: 20KB (when moving)
- Animation State: 10KB
- Inventory/Tools: 10KB
- Group Membership: 10KB
- Buffer/Overhead: 30KB
```

### Rendering Performance

| Zoom Level | Visible Creatures | Target FPS | Rendering Strategy |
|------------|------------------|------------|-------------------|
| Close (1-50) | 50 | 144+ | Full detail, all effects |
| Medium (50-200) | 200 | 120+ | Reduced particles, simplified shadows |
| Far (200-500) | 500 | 90+ | Batched rendering, no shadows |
| Overview (500+) | All | 60+ | Instanced dots/icons |

### System Update Frequencies

| System | Close LOD | Medium LOD | Far LOD | Minimal LOD |
|--------|-----------|------------|---------|-------------|
| Movement | Every frame | Every frame | Every 2 frames | Every 8 frames |
| Decision Making | 10Hz | 5Hz | 2Hz | 0.5Hz |
| Conversations | Real-time | 5Hz | 1Hz | Statistical |
| Needs Update | 2Hz | 1Hz | 0.5Hz | 0.1Hz |
| Social Updates | 1Hz | 0.5Hz | 0.2Hz | Statistical |
| Animation | 60Hz | 30Hz | 15Hz | None |

### Time Scaling Performance

| Speed | Simulation Method | Target Performance |
|-------|------------------|-------------------|
| Pause (0x) | No updates | 144+ FPS |
| Slow (0.5x) | Full simulation | 120+ FPS |
| Normal (1x) | Full simulation | 90+ FPS |
| Fast (10x) | Full simulation | 60+ FPS |
| Very Fast (100x) | Reduced accuracy | 60+ FPS |
| Generational (1000x) | Statistical only | 60+ FPS |

### Spatial Performance Constraints

- Spatial grid cell size: **20.0 world units** (unified across all systems)
- Maximum creatures per cell: **50** (before subdivision)
- Query performance: **O(log n)** for range queries
- Update performance: **O(1)** for position changes

### Minimum Hardware Requirements

**Minimum Spec** (1000 creatures at 30 FPS):
- CPU: 4-core, 2.5GHz
- GPU: GTX 750 Ti / RX 460
- RAM: 8GB
- Storage: 2GB free

**Recommended Spec** (5000 creatures at 60 FPS):
- CPU: 6-core, 3.0GHz
- GPU: GTX 1060 / RX 580
- RAM: 16GB
- Storage: 4GB free

**Ideal Spec** (10000+ creatures at 60 FPS):
- CPU: 8-core, 3.5GHz
- GPU: RTX 3060 / RX 6600
- RAM: 32GB
- Storage: 8GB free

### Performance Monitoring

Built-in performance metrics track:
- Frame time breakdown (update/render/UI)
- Memory usage by system
- Entity count by LOD level
- Spatial index efficiency
- Cache hit rates

Performance targets are validated through automated benchmarks in CI/CD pipeline.