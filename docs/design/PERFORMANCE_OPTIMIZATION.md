# Performance Optimization Guide

## Overview
This document outlines aggressive performance optimization strategies to ensure smooth 60+ FPS with 1000+ creatures, complex AI, and rich animations on desktop hardware.

## Core Performance Philosophy

### Data-Oriented Design
- **Cache-Friendly Layouts**: Keep hot data together
- **Minimize Indirection**: Direct access over pointer chasing
- **SIMD-Friendly Structures**: Align data for vectorization

### Update Only What's Necessary
- **Visibility Culling**: Skip off-screen entities
- **LOD Systems**: Reduce fidelity with distance
- **Importance-Based Updates**: Prioritize visible/important creatures

## Architecture for Performance

### 1. Component Layout Optimization

```rust
// BAD: Random memory access
#[derive(Component)]
struct CreatureData {
    position: Vec3,      // 12 bytes
    name: String,        // 24 bytes - heap allocated!
    age: f32,           // 4 bytes
    velocity: Vec3,      // 12 bytes
    health: f32,        // 4 bytes
    // Total: 56+ bytes, poor cache usage
}

// GOOD: Hot/Cold data separation
#[derive(Component)]
struct Position(Vec2);  // 8 bytes, tightly packed

#[derive(Component)]
struct Velocity(Vec2);  // 8 bytes, tightly packed

#[derive(Component)]
struct Health(f32);     // 4 bytes

#[derive(Component)]
struct CreatureMetadata {  // Cold data, accessed rarely
    name: String,
    birth_time: f64,
    parent_ids: [Option<Entity>; 2],
}
```

### 2. Spatial Indexing System

```rust
use bevy::prelude::*;

#[derive(Resource)]
struct SpatialIndex {
    // Hierarchical spatial hash grid
    cells: HashMap<CellCoord, Vec<Entity>>,
    cell_size: f32,
    
    // Dirty tracking
    moved_entities: HashSet<Entity>,
}

impl SpatialIndex {
    fn query_radius(&self, center: Vec2, radius: f32) -> Vec<Entity> {
        let min_cell = self.world_to_cell(center - Vec2::splat(radius));
        let max_cell = self.world_to_cell(center + Vec2::splat(radius));
        
        let mut results = Vec::with_capacity(32);  // Pre-allocate
        
        for x in min_cell.x..=max_cell.x {
            for y in min_cell.y..=max_cell.y {
                if let Some(entities) = self.cells.get(&CellCoord { x, y }) {
                    results.extend(entities);
                }
            }
        }
        
        results
    }
}
```

### 3. Parallel System Design

```rust
// Use Bevy's automatic parallelization
fn update_creature_needs(
    mut query: Query<(&mut Needs, &Position), Without<Sleeping>>,
    time: Res<SimulationTime>,
) {
    // This will automatically run in parallel
    query.par_iter_mut().for_each_mut(|(mut needs, pos)| {
        // Update logic - keep it simple and side-effect free
        needs.hunger -= HUNGER_DECAY_RATE * time.delta;
        needs.thirst -= THIRST_DECAY_RATE * time.delta;
    });
}

// Explicit parallelization for complex operations
use rayon::prelude::*;

fn calculate_genetics_batch(
    creatures: Vec<(Entity, DNA)>,
    commands: &mut Commands,
) {
    let offspring: Vec<_> = creatures
        .par_chunks(2)
        .filter_map(|parents| {
            if parents.len() == 2 {
                Some(DNA::crossover(&parents[0].1, &parents[1].1))
            } else {
                None
            }
        })
        .collect();
    
    // Single-threaded spawn after parallel computation
    for dna in offspring {
        spawn_creature(commands, dna);
    }
}
```
### 4. Level of Detail (LOD) System

```rust
#[derive(Component)]
struct LODLevel(u8);

const LOD_DISTANCES: [f32; 4] = [50.0, 200.0, 500.0, 1000.0];

fn update_lod_levels(
    camera: Query<&Transform, With<Camera>>,
    mut creatures: Query<(&Transform, &mut LODLevel)>,
) {
    let camera_pos = camera.single().translation.truncate();
    
    for (transform, mut lod) in &mut creatures {
        let distance = camera_pos.distance(transform.translation.truncate());
        
        let new_lod = LOD_DISTANCES
            .iter()
            .position(|&d| distance < d)
            .unwrap_or(LOD_DISTANCES.len()) as u8;
            
        if lod.0 != new_lod {
            lod.0 = new_lod;
        }
    }
}

// LOD-aware animation system
fn animate_creatures(
    mut query: Query<(&mut AnimationState, &LODLevel)>,
    time: Res<Time>,
) {
    for (mut anim, lod) in &mut query {
        match lod.0 {
            0 => anim.update_full(time.delta_seconds()),      // Full animation
            1 => anim.update_simple(time.delta_seconds()),    // Reduced keyframes
            2 => anim.update_minimal(time.delta_seconds()),   // Very basic
            _ => {} // No animation
        }
    }
}
```

### 5. Update Scheduling

```rust
#[derive(Resource)]
struct UpdateScheduler {
    frame_budget_ms: f32,  // e.g., 8ms for 120fps headroom
    priorities: Vec<UpdatePriority>,
}

#[derive(Clone)]
struct UpdatePriority {
    system: SystemId,
    importance: f32,
    last_run: f64,
    frequency: f32,  // How often this needs to run
}

fn intelligent_update_scheduler(
    mut scheduler: ResMut<UpdateScheduler>,
    time: Res<Time>,
    diagnostics: Res<Diagnostics>,
) {
    let frame_time = diagnostics.get_measurement(FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(|d| d.average())
        .unwrap_or(16.0);
    
    // Sort by priority score
    scheduler.priorities.sort_by(|a, b| {
        let a_score = a.importance * (time.elapsed_seconds_f64() - a.last_run);
        let b_score = b.importance * (time.elapsed_seconds_f64() - b.last_run);
        b_score.partial_cmp(&a_score).unwrap()
    });
    
    // Run systems until budget exhausted
    let mut budget_remaining = scheduler.frame_budget_ms;
    for priority in &mut scheduler.priorities {
        if budget_remaining <= 0.0 { break; }
        
        if time.elapsed_seconds_f64() - priority.last_run >= priority.frequency {
            // Run system (would need actual implementation)
            priority.last_run = time.elapsed_seconds_f64();
            budget_remaining -= 1.0; // Estimated cost
        }
    }
}
```
### 6. Efficient Rendering Pipeline

```rust
// Instanced rendering for creatures
#[derive(Component)]
struct InstancedSprite {
    texture_index: u32,
    tint: Color,
}

fn prepare_instanced_rendering(
    creatures: Query<(&Position, &InstancedSprite, &LODLevel)>,
    mut instance_buffer: ResMut<InstanceBuffer>,
) {
    // Clear previous frame
    instance_buffer.clear();
    
    // Group by texture for better batching
    let mut instances_by_texture: HashMap<u32, Vec<InstanceData>> = HashMap::new();
    
    for (pos, sprite, lod) in &creatures {
        if lod.0 > 3 { continue; } // Don't render distant creatures
        
        let instance = InstanceData {
            position: pos.0.extend(0.0),
            color: sprite.tint,
            scale: 1.0,
        };
        
        instances_by_texture
            .entry(sprite.texture_index)
            .or_insert_with(Vec::new)
            .push(instance);
    }
    
    // Submit batches
    for (texture_id, instances) in instances_by_texture {
        instance_buffer.submit_batch(texture_id, instances);
    }
}
```

### 7. Memory Pool Management

```rust
use std::mem::MaybeUninit;

// Pre-allocated pools for frequent allocations
struct CreaturePool {
    creatures: Vec<MaybeUninit<CreatureData>>,
    free_indices: Vec<usize>,
}

impl CreaturePool {
    fn new(capacity: usize) -> Self {
        Self {
            creatures: Vec::with_capacity(capacity),
            free_indices: (0..capacity).collect(),
        }
    }
    
    fn allocate(&mut self) -> Option<&mut CreatureData> {
        self.free_indices.pop().map(|idx| {
            unsafe {
                self.creatures[idx].assume_init_mut()
            }
        })
    }
    
    fn deallocate(&mut self, creature: &mut CreatureData) {
        let idx = unsafe {
            (creature as *mut _ as usize - self.creatures.as_ptr() as usize) 
                / std::mem::size_of::<CreatureData>()
        };
        self.free_indices.push(idx);
    }
}
```

### 8. AI Decision Optimization

```rust
// Decision tree with caching
#[derive(Component)]
struct DecisionCache {
    last_decision: Decision,
    valid_until: f64,
    decision_tree_state: u32,  // Compact state representation
}

fn make_decisions(
    mut creatures: Query<(
        &Needs,
        &Position,
        &mut DecisionCache,
        &mut CurrentAction,
    )>,
    spatial_index: Res<SpatialIndex>,
    time: Res<Time>,
) {
    for (needs, pos, mut cache, mut action) in &mut creatures {
        // Check cache first
        if time.elapsed_seconds_f64() < cache.valid_until {
            continue;
        }
        
        // Compact decision logic
        let decision = match cache.decision_tree_state {
            // Extremely hungry - find food fast
            s if s & 0x01 != 0 => {
                let food = find_nearest_food_cached(&spatial_index, pos.0);
                Decision::MoveTo(food)
            }
            // Thirsty
            s if s & 0x02 != 0 => {
                let water = find_nearest_water_cached(&spatial_index, pos.0);
                Decision::MoveTo(water)
            }
            // Social need
            s if s & 0x04 != 0 => {
                Decision::Socialize
            }
            _ => Decision::Wander,
        };
        
        cache.last_decision = decision.clone();
        cache.valid_until = time.elapsed_seconds_f64() + 0.5; // Cache for 0.5s
        *action = decision.into();
    }
}
```
### 9. World Generation Optimization

```rust
// Chunk generation with caching
#[derive(Resource)]
struct ChunkCache {
    generated_chunks: HashMap<ChunkCoord, Arc<ChunkData>>,
    generation_queue: VecDeque<ChunkCoord>,
    worker_thread: Option<std::thread::JoinHandle<()>>,
}

fn async_chunk_generation(
    mut cache: ResMut<ChunkCache>,
    camera: Query<&Transform, With<Camera>>,
) {
    let camera_chunk = world_to_chunk(camera.single().translation.truncate());
    
    // Queue nearby chunks for generation
    for dx in -LOAD_DISTANCE..=LOAD_DISTANCE {
        for dy in -LOAD_DISTANCE..=LOAD_DISTANCE {
            let coord = ChunkCoord {
                x: camera_chunk.x + dx,
                y: camera_chunk.y + dy,
            };
            
            if !cache.generated_chunks.contains_key(&coord) {
                cache.generation_queue.push_back(coord);
            }
        }
    }
    
    // Process queue in background thread
    if cache.worker_thread.is_none() && !cache.generation_queue.is_empty() {
        let queue = cache.generation_queue.clone();
        cache.worker_thread = Some(std::thread::spawn(move || {
            // Generate chunks using SIMD-optimized noise
            generate_chunks_batch(queue)
        }));
    }
}
```

### 10. Conversation System Optimization

```rust
// Text generation with pooling
struct ConversationPool {
    templates: Vec<&'static str>,
    buffer: String,  // Reusable string buffer
}

impl ConversationPool {
    fn generate_conversation(&mut self, topic: ConversationTopic) -> &str {
        self.buffer.clear();
        
        // Use pre-compiled templates instead of dynamic generation
        match topic {
            ConversationTopic::Food => {
                self.buffer.push_str(self.templates[0]);
            }
            ConversationTopic::Weather => {
                self.buffer.push_str(self.templates[1]);
            }
            _ => {
                self.buffer.push_str(self.templates[2]);
            }
        }
        
        &self.buffer
    }
}
```

## Performance Targets & Metrics

### Target Performance
- **1000 creatures**: 60+ FPS
- **5000 creatures**: 30+ FPS
- **Frame time**: < 16ms (ideally < 10ms)
- **Memory usage**: < 2GB for full simulation

### Key Metrics to Monitor
```rust
fn setup_performance_monitoring(app: &mut App) {
    app
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(EntityCountDiagnosticsPlugin::default())
        .add_plugins(SystemInformationDiagnosticsPlugin::default())
        .add_system(performance_logger);
}

fn performance_logger(
    diagnostics: Res<Diagnostics>,
    creature_count: Query<&Creature>,
) {
    if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(value) = fps.smoothed() {
            if value < 30.0 {
                warn!("Low FPS: {:.2} with {} creatures", value, creature_count.iter().count());
            }
        }
    }
}
```
## Profiling & Benchmarking

### Profiling Tools
```rust
// Use tracy for detailed profiling
use tracing_tracy;

#[tracing::instrument]
fn expensive_system() {
    // System code
}

// Custom timing
use std::time::Instant;

fn benchmark_system(mut commands: Commands) {
    let start = Instant::now();
    
    // Do work
    
    let duration = start.elapsed();
    if duration.as_millis() > 2 {
        warn!("System took {}ms", duration.as_millis());
    }
}
```

### Benchmarking Critical Paths
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_pathfinding(c: &mut Criterion) {
    c.bench_function("find nearest food", |b| {
        let spatial_index = create_test_spatial_index();
        let position = Vec2::new(100.0, 100.0);
        
        b.iter(|| {
            find_nearest_food(&spatial_index, black_box(position))
        });
    });
}
```

## Specific Optimizations

### Genetics System
```rust
// Bitpacked genes for cache efficiency
#[derive(Component, Clone, Copy)]
struct CompactDNA {
    genes: [u64; 4],  // 256 bits total
}

impl CompactDNA {
    fn get_trait(&self, index: usize) -> f32 {
        let word = index / 16;
        let bit_offset = (index % 16) * 4;
        let bits = (self.genes[word] >> bit_offset) & 0xF;
        bits as f32 / 15.0  // Normalize to 0.0-1.0
    }
}
```

### Animation System
```rust
// Animation compression
#[derive(Component)]
struct CompressedAnimation {
    keyframes: Vec<u16>,  // Quantized positions
    current_frame: f32,
    speed: f32,
}

fn decompress_position(compressed: u16) -> Vec2 {
    let x = (compressed >> 8) as f32 / 255.0;
    let y = (compressed & 0xFF) as f32 / 255.0;
    Vec2::new(x * WORLD_SIZE, y * WORLD_SIZE)
}
```

## Performance Checklist

### Data Layout
- [x] Components under 64 bytes
- [x] Hot/cold data separation  
- [x] Minimize heap allocations
- [x] Use SOA where beneficial

### Systems
- [x] Parallel queries with par_iter_mut
- [x] Query filters to reduce work
- [x] Cached spatial queries
- [x] LOD for all expensive operations

### Rendering
- [x] Instanced rendering
- [x] Frustum culling
- [x] Texture atlasing
- [x] Minimal draw calls

### Memory
- [x] Object pooling
- [x] Pre-allocated buffers
- [x] Reuse allocations
- [x] Compact data structures

### Time Scaling
- [x] Skip frames at high speed
- [x] Reduce fidelity at high speed
- [x] Batch updates
- [x] Priority-based updates

## Configuration for Maximum Performance

```toml
# Cargo.toml
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
strip = true

[profile.release.package."*"]
opt-level = 3

# Enable CPU features
[build]
rustflags = ["-C", "target-cpu=native"]
```

## Final Notes

1. **Profile First**: Never optimize without data
2. **Test at Scale**: Always test with 1000+ creatures
3. **Monitor Continuously**: Add performance regression tests
4. **Iterate**: Performance is an ongoing process

With these optimizations, the simulation should maintain 60+ FPS with 1000 creatures on modern desktop hardware, with headroom for rich animations and complex behaviors.

---
*Last Updated: 2024-12-XX*