# Performance Guide

## Table of Contents
1. [Philosophy](#philosophy)
2. [Quick Reference](#quick-reference)
3. [Detailed Optimization Guide](#detailed-optimization-guide)

---

## Philosophy

**Speed is a feature.** This simulation prioritizes performance to enable rich behaviors and beautiful animations while maintaining 60+ FPS with 1000+ creatures.

### Core Principles

1. **Profile before optimizing** - But design for performance from the start
2. **Test at scale** - Always test with 1000+ creatures
3. **Monitor regressions** - Performance tests in CI
4. **Iterate based on data** - Use Tracy/perf for real metrics

### The Result

With this performance-first approach, the simulation can handle:
- **Rich creature behaviors** without simplified AI
- **Beautiful animations** without static sprites  
- **Complex social interactions** without fake relationships
- **Detailed worlds** without pop-in or loading screens
- **Smooth time scaling** from 1x to 1000x

**Performance enables features, not the other way around.**

---

## Quick Reference

### Critical Performance Rules

#### 1. **Update Only What's Visible**
```rust
// Always check visibility
if !in_camera_view(entity) { return; }
```

#### 2. **Use LOD Everywhere**
- Animations: Full → Simple → Static
- AI: Complex → Basic → Dormant  
- Rendering: Detailed → Simple → Hidden

#### 3. **Cache Expensive Calculations**
- Spatial queries: Cache for 0.5-1.0 seconds
- Pathfinding: Store results
- Decision making: Reuse recent decisions

#### 4. **Batch Operations**
- Group similar updates
- Process in parallel when possible
- Minimize system switches

### Component Size Guidelines

| Component | Max Size | Reason |
|-----------|----------|--------|
| Position | 8 bytes | Hot path data |
| Velocity | 8 bytes | Updated every frame |
| Health/Needs | 4-8 bytes | Frequently accessed |
| Metadata | No limit | Cold storage, rarely accessed |

### System Update Frequencies

| System | Frequency | LOD Impact |
|--------|-----------|------------|
| Movement | Every frame | Yes - reduce for distant |
| Needs | 10 Hz | No - always update |
| Decisions | 0.5-10 Hz | Yes - 10Hz near, 2Hz medium, 1Hz far, 0.5Hz distant |
| Genetics | On event | No |
| Animations | Every frame | Yes - heavily reduced |

### Memory Budgets

- **Per Creature**: < 1KB hot data
- **World Chunk**: < 64KB
- **Total RAM**: < 2GB target

### Frame Time Budgets (16ms total)

- Input: 1ms
- Simulation: 6ms
- Rendering: 6ms  
- UI: 2ms
- Buffer: 1ms

### Performance Targets

| Creatures | Target FPS | Frame Budget |
|-----------|------------|--------------|
| 100 | 144 | 6.9ms |
| 500 | 120 | 8.3ms |
| 1000 | 60 | 16.6ms |
| 5000 | 30 | 33.3ms |

### Red Flags 🚩

1. **Components > 64 bytes**
2. **Allocations in hot loops**
3. **Uncached spatial queries**
4. **No LOD on distant entities**
5. **String operations in systems**
6. **HashMap in hot paths**
7. **Unbounded growth**
8. **No frustum culling**

### Quick Wins 🎯

1. **Enable LTO**: 10-20% performance
2. **Use par_iter_mut**: Near-linear scaling
3. **Frustum culling**: 50%+ fewer entities
4. **LOD system**: 70%+ reduction in work
5. **Spatial indexing**: O(n²) → O(n log n)

### Profiling Commands

```bash
# CPU profiling
cargo build --release && perf record -g ./target/release/game
perf report

# Memory profiling  
valgrind --tool=massif ./target/release/game
ms_print massif.out.*

# Tracy real-time profiling
cargo run --release --features tracy
```

---

## Detailed Optimization Guide

### Key Strategies

#### 1. Data-Oriented Design
- Components are kept under 64 bytes for cache efficiency
- Hot data (Position, Velocity) separated from cold data (Name, History)
- Struct-of-Arrays layout where beneficial

#### 2. Aggressive Culling
- **Frustum Culling**: Don't process off-screen entities
- **LOD System**: 5 levels of detail based on distance
- **Importance-Based Updates**: Visible creatures get priority

#### 3. Spatial Optimization
- **Hierarchical Spatial Hash**: O(1) average lookup
- **Chunk System**: Only load/process nearby world sections
- **Query Caching**: Reuse expensive spatial queries

#### 4. Parallel Processing
- Bevy's automatic system parallelization
- Rayon for complex batch operations
- SIMD-friendly data layouts

#### 5. Smart Time Scaling
```
At 1x speed:  Full simulation
At 5x speed:  Reduce animation quality
At 20x speed: Simplify AI decisions
At 100x speed: Batch updates, skip frames
At 1000x speed: Statistical simulation
```

### Architecture Impact

#### System Design
- Systems are designed to early-exit
- Queries use filters aggressively
- Updates are scheduled by importance

#### Memory Layout
- Pre-allocated pools for creatures
- Reusable buffers for strings/calculations
- Compact representations (bit-packed genes)

#### Rendering Pipeline
- Instanced rendering for all creatures
- Texture atlasing to reduce draw calls
- Dynamic batching based on creature type

### Measurement & Monitoring

Every system includes performance tracking:
```rust
#[tracing::instrument]
fn system_name() {
    // Automatic performance tracking
}
```

### Spatial Indexing

The simulation uses a hierarchical spatial index for O(log n) proximity queries:

```rust
pub struct SpatialIndex {
    cells: HashMap<CellCoord, Vec<EntityId>>,
    cell_size: f32,
    dirty_entities: HashSet<EntityId>,
}

impl SpatialIndex {
    pub fn query_radius(&self, center: Vec3, radius: f32) -> Vec<EntityId> {
        let min_cell = self.world_to_cell(center - Vec3::splat(radius));
        let max_cell = self.world_to_cell(center + Vec3::splat(radius));
        
        let mut results = Vec::with_capacity(32);
        for x in min_cell.x..=max_cell.x {
            for y in min_cell.y..=max_cell.y {
                for z in min_cell.z..=max_cell.z {
                    if let Some(entities) = self.cells.get(&CellCoord { x, y, z }) {
                        results.extend(entities.iter().copied());
                    }
                }
            }
        }
        results
    }
}
```

### Component Storage Optimization

Components are organized by access patterns:

```rust
// Hot data - accessed every frame
pub struct Position(Vec3);
pub struct Velocity(Vec3);

// Warm data - accessed frequently
pub struct Health(f32);
pub struct Energy(f32);

// Cold data - accessed rarely
pub struct Biography {
    name: String,
    birth_date: f64,
    parents: (EntityId, EntityId),
    life_events: Vec<LifeEvent>,
}
```

### Level of Detail (LOD) System

```rust
pub enum LODLevel {
    Full = 0,      // < 50m
    Reduced = 1,   // 50-100m
    Simple = 2,    // 100-200m
    Minimal = 3,   // 200-500m
    Dormant = 4,   // > 500m
}

impl CreatureAI {
    pub fn update(&mut self, lod: LODLevel) {
        match lod {
            LODLevel::Full => {
                self.update_all_systems();
            }
            LODLevel::Reduced => {
                self.update_movement();
                self.update_basic_needs();
                if self.frames % 4 == 0 {
                    self.update_decisions();
                }
            }
            LODLevel::Simple => {
                if self.frames % 10 == 0 {
                    self.update_movement();
                    self.update_basic_needs();
                }
            }
            LODLevel::Minimal => {
                if self.frames % 60 == 0 {
                    self.update_basic_needs();
                }
            }
            LODLevel::Dormant => {
                // No updates
            }
        }
    }
}
```

### Memory Pooling

```rust
// See Memory Architecture document for pool implementation
```

Memory pooling is critical for performance. See [Memory Architecture](MEMORY_ARCHITECTURE.md#memory-pool-system) for the complete pool system implementation including:
- TypedPool for different object types
- Pool growth strategies
- Handle-based access patterns
- Memory budget integration

### Batch Processing

```rust
pub fn update_creature_needs(creatures: &mut [Creature]) {
    creatures.par_chunks_mut(64).for_each(|chunk| {
        for creature in chunk {
            creature.hunger = (creature.hunger + HUNGER_RATE * DELTA_TIME).min(100.0);
            creature.thirst = (creature.thirst + THIRST_RATE * DELTA_TIME).min(100.0);
            creature.energy = (creature.energy - ENERGY_DRAIN * DELTA_TIME).max(0.0);
        }
    });
}
```

### Time Scaling Optimizations

```rust
pub fn get_update_frequency(time_scale: f32, system: SystemType) -> f32 {
    match (time_scale, system) {
        (s, SystemType::Movement) if s <= 5.0 => 60.0,
        (s, SystemType::Movement) if s <= 20.0 => 30.0,
        (s, SystemType::Movement) => 10.0,
        
        (s, SystemType::AI) if s <= 10.0 => 10.0,
        (s, SystemType::AI) if s <= 100.0 => 2.0,
        (s, SystemType::AI) => 0.5,
        
        (s, SystemType::Animation) if s <= 2.0 => 60.0,
        (s, SystemType::Animation) if s <= 10.0 => 15.0,
        (s, SystemType::Animation) => 0.0, // Disable
        
        _ => 1.0,
    }
}
```

### Rendering Optimizations

```rust
pub struct InstancedRenderer {
    instance_buffers: HashMap<CreatureType, InstanceBuffer>,
    draw_calls: Vec<DrawCall>,
}

impl InstancedRenderer {
    pub fn render(&mut self, visible_creatures: &[CreatureRenderData]) {
        // Group by creature type
        for (creature_type, creatures) in visible_creatures.iter().group_by(|c| c.creature_type) {
            let buffer = self.instance_buffers.get_mut(&creature_type).unwrap();
            buffer.update(creatures);
            
            self.draw_calls.push(DrawCall {
                mesh: creature_type.mesh(),
                instances: buffer,
                material: creature_type.material(),
            });
        }
        
        // Submit all draw calls at once
        self.submit_draw_calls();
    }
}
```

### Debug Mode Considerations

```rust
#[cfg(debug_assertions)]
const SPATIAL_CELL_SIZE: f32 = 20.0;

#[cfg(not(debug_assertions))]
const SPATIAL_CELL_SIZE: f32 = 10.0;

// Remove expensive checks in release
#[cfg(debug_assertions)]
fn validate_creature_state(creature: &Creature) {
    assert!(creature.health >= 0.0 && creature.health <= 100.0);
    assert!(creature.position.is_finite());
}

#[cfg(not(debug_assertions))]
fn validate_creature_state(_: &Creature) {}
```

### Common Bottlenecks and Solutions

| Bottleneck | Solution |
|------------|----------|
| Spatial queries | Use spatial index, cache results |
| String allocations | Use string interning, pre-allocated buffers |
| Random access | Group data by access pattern |
| Lock contention | Use atomics, reduce critical sections |
| Cache misses | Keep hot data < 64 bytes, use SOA layout |
| Branch prediction | Sort entities by type before processing |

### Performance Testing

```rust
#[cfg(test)]
mod perf_tests {
    use criterion::{black_box, criterion_group, criterion_main, Criterion};
    
    fn bench_spatial_query(c: &mut Criterion) {
        let mut world = create_world_with_creatures(1000);
        
        c.bench_function("spatial_query_100m", |b| {
            b.iter(|| {
                black_box(world.query_radius(Vec3::ZERO, 100.0))
            })
        });
    }
    
    criterion_group!(benches, bench_spatial_query);
    criterion_main!(benches);
}
```