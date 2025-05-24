# Memory Architecture

This document consolidates all memory-related design decisions, including management strategies, persistence, and performance optimizations.

## Overview

The simulation uses a multi-tiered memory architecture designed to support 5000+ creatures at 60+ FPS while maintaining under 2GB memory usage. Key strategies include object pooling, cache-friendly layouts, and intelligent persistence.

## Memory Budget

Total budget: 2GB allocated as follows:
- **Creatures**: 40% (800MB) - Entity data, components, behaviors
- **World**: 20% (400MB) - Terrain, resources, spatial indices
- **Rendering**: 20% (400MB) - Sprites, animations, particle effects
- **UI**: 10% (200MB) - Interface, data visualization
- **Audio**: 5% (100MB) - Sound effects, ambient audio
- **Reserve**: 5% (100MB) - Temporary allocations, spikes

## Core Architecture

### Memory Pool System

```rust
pub struct MemoryPoolManager {
    creature_pool: TypedPool<Creature>,
    component_pools: ComponentPoolSet,
    behavior_pool: TypedPool<BehaviorNode>,
    particle_pool: TypedPool<Particle>,
}

impl TypedPool<T> {
    pub fn acquire(&mut self) -> PoolHandle<T> {
        // O(1) allocation from pre-allocated pool
        if let Some(index) = self.free_list.pop() {
            PoolHandle::new(index)
        } else {
            self.grow()
        }
    }
}
```

### Component Storage

Components use Structure of Arrays (SoA) for cache efficiency:

```rust
pub struct ComponentStorage {
    // Hot data (accessed every frame)
    positions: Vec<Position>,
    velocities: Vec<Velocity>,
    health: Vec<Health>,
    
    // Cold data (accessed occasionally)
    genetics: Vec<Genetics>,
    memories: Vec<Memory>,
    relationships: Vec<Relationships>,
}
```

### Memory Persistence

Save/load system with memory-mapped files for performance:

```rust
pub struct SaveSystem {
    save_path: PathBuf,
    compression: CompressionLevel,
    buffer_pool: BufferPool,
}

impl SaveSystem {
    pub async fn save_world(&self, world: &World) -> Result<SaveHandle> {
        let snapshot = WorldSnapshot::capture(world)?;
        let compressed = self.compress(snapshot)?;
        self.write_atomic(compressed).await
    }
}
```

## Performance Optimizations

### Cache-Friendly Access Patterns

```rust
// Good: Sequential access
for i in 0..creatures.len() {
    positions[i].x += velocities[i].x * dt;
}

// Bad: Random access
for &id in active_creatures {
    creatures[id].position += creatures[id].velocity * dt;
}
```

### Memory Pressure Handling

```rust
impl MemoryPressureHandler {
    pub fn handle_pressure(&mut self, level: PressureLevel) {
        match level {
            PressureLevel::Low => self.aggressive_caching(),
            PressureLevel::Medium => self.reduce_detail_levels(),
            PressureLevel::High => self.emergency_cleanup(),
            PressureLevel::Critical => self.pause_and_gc(),
        }
    }
}
```

### Spatial Indexing

Hierarchical spatial hash for O(log n) neighbor queries:

```rust
pub struct SpatialIndex {
    cells: HashMap<CellId, Vec<EntityId>>,
    hierarchy: QuadTree,
    dirty_entities: HashSet<EntityId>,
}
```

## Memory Lifecycle

1. **Startup**: Pre-allocate pools based on world size
2. **Runtime**: Use pools for all dynamic allocations
3. **Pressure**: Gradually reduce quality when approaching limits
4. **Cleanup**: Defragment during quiet periods
5. **Shutdown**: Serialize state for next session

## Integration Points

- **LOD System**: Reduces memory for distant creatures
- **Save System**: Chunks data for incremental saves
- **Rendering**: Shares vertex buffers for similar creatures
- **AI**: Caches decision trees and reuses behavior nodes

## Monitoring

```rust
pub struct MemoryMonitor {
    pub current_usage: AtomicUsize,
    pub peak_usage: AtomicUsize,
    pub allocation_rate: AtomicUsize,
    pub fragmentation: AtomicF32,
}
```

Access memory stats via debug UI (F3) or performance dashboard.

## Best Practices

1. **Always use pools** for frequently created/destroyed objects
2. **Prefer SoA** over AoS for component storage
3. **Batch operations** to improve cache utilization
4. **Monitor memory** pressure and respond proactively
5. **Test with max creatures** to ensure budgets are respected

See [Performance Guide](PERFORMANCE.md) for related optimization strategies.