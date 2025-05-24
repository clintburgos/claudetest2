# Performance-First Architecture

## Core Philosophy
**Speed is a feature.** This simulation prioritizes performance to enable rich behaviors and beautiful animations while maintaining 60+ FPS with 1000+ creatures.

## Key Strategies

### 1. Data-Oriented Design
- Components are kept under 64 bytes for cache efficiency
- Hot data (Position, Velocity) separated from cold data (Name, History)
- Struct-of-Arrays layout where beneficial

### 2. Aggressive Culling
- **Frustum Culling**: Don't process off-screen entities
- **LOD System**: 5 levels of detail based on distance
- **Importance-Based Updates**: Visible creatures get priority

### 3. Spatial Optimization
- **Hierarchical Spatial Hash**: O(1) average lookup
- **Chunk System**: Only load/process nearby world sections
- **Query Caching**: Reuse expensive spatial queries

### 4. Parallel Processing
- Bevy's automatic system parallelization
- Rayon for complex batch operations
- SIMD-friendly data layouts

### 5. Smart Time Scaling
```
At 1x speed:  Full simulation
At 5x speed:  Reduce animation quality
At 20x speed: Simplify AI decisions
At 100x speed: Batch updates, skip frames
At 1000x speed: Statistical simulation
```

## Performance Targets

| Creatures | Target FPS | Frame Budget |
|-----------|------------|--------------|
| 100 | 144 | 6.9ms |
| 500 | 120 | 8.3ms |
| 1000 | 60 | 16.6ms |
| 5000 | 30 | 33.3ms |

## Architecture Impact

### System Design
- Systems are designed to early-exit
- Queries use filters aggressively
- Updates are scheduled by importance

### Memory Layout
- Pre-allocated pools for creatures
- Reusable buffers for strings/calculations
- Compact representations (bit-packed genes)

### Rendering Pipeline
- Instanced rendering for all creatures
- Texture atlasing to reduce draw calls
- Dynamic batching based on creature type

## Measurement & Monitoring

Every system includes performance tracking:
```rust
#[tracing::instrument]
fn system_name() {
    // Automatic performance tracking
}
```

## Development Principles

1. **Profile before optimizing** - But design for performance from the start
2. **Test at scale** - Always test with 1000+ creatures
3. **Monitor regressions** - Performance tests in CI
4. **Iterate based on data** - Use Tracy/perf for real metrics

## The Result

With this performance-first approach, the simulation can handle:
- **Rich creature behaviors** without simplified AI
- **Beautiful animations** without static sprites  
- **Complex social interactions** without fake relationships
- **Detailed worlds** without pop-in or loading screens
- **Smooth time scaling** from 1x to 1000x

**Performance enables features, not the other way around.**

---
*Detailed optimizations: [PERFORMANCE_OPTIMIZATION.md](./PERFORMANCE_OPTIMIZATION.md)*
*Quick reference: [PERFORMANCE_QUICK_REF.md](./PERFORMANCE_QUICK_REF.md)*