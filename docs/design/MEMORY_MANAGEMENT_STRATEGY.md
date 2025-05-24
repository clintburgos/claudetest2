# Memory Management Strategy Design

## Overview

A comprehensive memory management strategy for the creature simulation that ensures optimal performance with 1000+ creatures while maintaining smooth 60+ FPS gameplay. The system uses multiple techniques including object pooling, cache-friendly layouts, memory budgets, and intelligent allocation strategies.

## Memory Architecture

```rust
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct SimulationAllocator {
    system_allocator: System,
    memory_pools: MemoryPoolManager,
    allocation_tracker: AllocationTracker,
    memory_budget: MemoryBudget,
}

pub struct MemoryBudget {
    pub total_budget: usize,              // e.g., 4GB
    pub creature_budget: usize,           // 40% - 1.6GB  
    pub world_budget: usize,              // 20% - 800MB
    pub rendering_budget: usize,          // 20% - 800MB
    pub ui_budget: usize,                 // 10% - 400MB
    pub audio_budget: usize,              // 5% - 200MB
    pub reserve_budget: usize,            // 5% - 200MB
}

pub struct AllocationTracker {
    pub current_usage: AtomicUsize,
    pub peak_usage: AtomicUsize,
    pub allocation_count: AtomicUsize,
    pub category_usage: HashMap<MemoryCategory, AtomicUsize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryCategory {
    Creatures,
    Components,
    Behaviors,
    World,
    Rendering,
    UI,
    Audio,
    Temporary,
}
```

### Object Pool System

```rust
pub struct MemoryPoolManager {
    pools: HashMap<TypeId, Box<dyn MemoryPool>>,
    pool_configs: HashMap<TypeId, PoolConfig>,
}

pub struct PoolConfig {
    pub initial_capacity: usize,
    pub max_capacity: usize,
    pub growth_factor: f32,
    pub shrink_threshold: f32,
    pub alignment: usize,
}

pub trait MemoryPool: Send + Sync {
    fn allocate(&mut self) -> Option<*mut u8>;
    fn deallocate(&mut self, ptr: *mut u8);
    fn clear(&mut self);
    fn shrink_to_fit(&mut self);
    fn usage_stats(&self) -> PoolStats;
}

// Generic pool implementation
pub struct TypedPool<T> {
    free_list: Vec<*mut T>,
    allocated: HashSet<*mut T>,
    memory_blocks: Vec<MemoryBlock<T>>,
    config: PoolConfig,
}

pub struct MemoryBlock<T> {
    data: Vec<T>,
    free_mask: BitVec,
    block_size: usize,
}

impl<T: Default> TypedPool<T> {
    pub fn new(config: PoolConfig) -> Self {
        let mut pool = Self {
            free_list: Vec::with_capacity(config.initial_capacity),
            allocated: HashSet::new(),
            memory_blocks: Vec::new(),
            config,
        };
        
        pool.grow(config.initial_capacity);
        pool
    }
    
    fn grow(&mut self, additional: usize) {
        let block_size = (additional.max(1024) + 63) & !63; // Align to 64
        let mut block = MemoryBlock {
            data: Vec::with_capacity(block_size),
            free_mask: BitVec::with_capacity(block_size),
            block_size,
        };
        
        // Initialize block
        for i in 0..block_size {
            block.data.push(T::default());
            block.free_mask.push(true);
            
            let ptr = &block.data[i] as *const T as *mut T;
            self.free_list.push(ptr);
        }
        
        self.memory_blocks.push(block);
    }
    
    pub fn allocate(&mut self) -> Option<&'static mut T> {
        if self.free_list.is_empty() {
            let current_capacity = self.allocated.len() + self.free_list.len();
            if current_capacity >= self.config.max_capacity {
                return None;
            }
            
            let growth = (current_capacity as f32 * self.config.growth_factor) as usize;
            self.grow(growth.min(self.config.max_capacity - current_capacity));
        }
        
        self.free_list.pop().map(|ptr| {
            self.allocated.insert(ptr);
            unsafe { &mut *ptr }
        })
    }
    
    pub fn deallocate(&mut self, item: &mut T) {
        let ptr = item as *mut T;
        if self.allocated.remove(&ptr) {
            *item = T::default(); // Reset to default state
            self.free_list.push(ptr);
            
            // Check if we should shrink
            let usage_ratio = self.allocated.len() as f32 / 
                            (self.allocated.len() + self.free_list.len()) as f32;
            
            if usage_ratio < self.config.shrink_threshold {
                self.shrink_to_fit();
            }
        }
    }
}
```

### Component Memory Layout

```rust
// Cache-friendly component storage
pub struct ComponentStorage<T> {
    dense: Vec<T>,                    // Tightly packed component data
    sparse: Vec<Option<usize>>,       // Entity ID to dense index mapping
    entities: Vec<EntityId>,          // Dense index to entity ID
    generation: u32,                  // For detecting stale references
}

// Hot/Cold data separation
pub struct CreatureData {
    // Hot data - accessed every frame (64 bytes)
    pub hot: CreatureHotData,
    
    // Warm data - accessed frequently (256 bytes)
    pub warm: *mut CreatureWarmData,
    
    // Cold data - accessed rarely (unbounded)
    pub cold: *mut CreatureColdData,
}

#[repr(C, align(64))] // Cache line aligned
pub struct CreatureHotData {
    pub position: Vec3,               // 12 bytes
    pub velocity: Vec3,               // 12 bytes
    pub health: f32,                  // 4 bytes
    pub energy: f32,                  // 4 bytes
    pub state: CreatureState,         // 4 bytes
    pub flags: CreatureFlags,         // 4 bytes
    pub spatial_hash: u32,            // 4 bytes
    pub _padding: [u8; 20],          // Pad to 64 bytes
}

pub struct CreatureWarmData {
    pub movement: MovementComponent,  // 64 bytes
    pub sensory: SensoryComponent,    // 48 bytes
    pub social: SocialComponent,      // 64 bytes
    pub emotional: EmotionalComponent, // 32 bytes
    pub decision: DecisionComponent,  // 48 bytes
}

pub struct CreatureColdData {
    pub genetics: Genetics,
    pub memories: MemoryStorage,
    pub relationships: RelationshipMap,
    pub learned_behaviors: Vec<LearnedBehavior>,
    pub biography: CreatureBiography,
}
```

### Memory-Efficient Collections

```rust
// Custom allocator-aware collections
pub struct PooledVec<T> {
    data: Vec<T, PoolAllocator<T>>,
    pool: Arc<TypedPool<T>>,
}

// Compressed storage for sparse data
pub struct CompressedSparseArray<T> {
    chunks: HashMap<ChunkId, CompressedChunk<T>>,
    chunk_size: usize,
    compression: CompressionStrategy,
}

pub struct CompressedChunk<T> {
    data: CompressedData,
    indices: BitVec,
    decompression_cache: Option<Vec<T>>,
    last_access: Instant,
}

// Ring buffer for temporal data
pub struct RingBuffer<T> {
    buffer: Vec<T>,
    head: usize,
    tail: usize,
    capacity: usize,
}

impl<T: Clone> RingBuffer<T> {
    pub fn push(&mut self, item: T) {
        if self.is_full() {
            // Overwrite oldest
            self.tail = (self.tail + 1) % self.capacity;
        }
        
        self.buffer[self.head] = item;
        self.head = (self.head + 1) % self.capacity;
    }
    
    pub fn get_recent(&self, count: usize) -> Vec<&T> {
        let mut result = Vec::with_capacity(count.min(self.len()));
        let mut idx = (self.head + self.capacity - 1) % self.capacity;
        
        for _ in 0..count.min(self.len()) {
            result.push(&self.buffer[idx]);
            idx = (idx + self.capacity - 1) % self.capacity;
        }
        
        result
    }
}
```

### Hierarchical Memory Management

```rust
pub struct HierarchicalMemoryManager {
    global_heap: GlobalHeap,
    thread_local_heaps: ThreadLocalHeaps,
    arena_allocators: ArenaAllocators,
    stack_allocators: StackAllocators,
}

// Arena allocator for frame-temporary allocations
pub struct ArenaAllocator {
    memory: Vec<u8>,
    offset: AtomicUsize,
    generation: AtomicU32,
}

impl ArenaAllocator {
    pub fn allocate(&self, layout: Layout) -> Option<*mut u8> {
        let size = layout.size();
        let align = layout.align();
        
        loop {
            let current = self.offset.load(Ordering::Relaxed);
            let aligned = (current + align - 1) & !(align - 1);
            let new_offset = aligned + size;
            
            if new_offset > self.memory.len() {
                return None;
            }
            
            if self.offset.compare_exchange_weak(
                current,
                new_offset,
                Ordering::Release,
                Ordering::Relaxed,
            ).is_ok() {
                return Some(unsafe { self.memory.as_ptr().add(aligned) as *mut u8 });
            }
        }
    }
    
    pub fn reset(&mut self) {
        self.offset.store(0, Ordering::Release);
        self.generation.fetch_add(1, Ordering::Release);
    }
}

// Stack allocator for nested temporary allocations
pub struct StackAllocator {
    memory: Vec<u8>,
    markers: Vec<StackMarker>,
    current_offset: usize,
}

pub struct StackMarker {
    offset: usize,
    debug_info: &'static str,
}

impl StackAllocator {
    pub fn push_marker(&mut self, debug_info: &'static str) -> StackMarker {
        let marker = StackMarker {
            offset: self.current_offset,
            debug_info,
        };
        self.markers.push(marker.clone());
        marker
    }
    
    pub fn pop_marker(&mut self, marker: StackMarker) {
        assert_eq!(self.markers.last().unwrap().offset, marker.offset);
        self.current_offset = marker.offset;
        self.markers.pop();
    }
}
```

### Memory Pressure Handling

```rust
pub struct MemoryPressureManager {
    thresholds: PressureThresholds,
    handlers: Vec<Box<dyn PressureHandler>>,
    current_pressure: AtomicU8,
}

pub struct PressureThresholds {
    pub low: f32,      // 0.6 - 60% memory usage
    pub medium: f32,   // 0.8 - 80% memory usage  
    pub high: f32,     // 0.9 - 90% memory usage
    pub critical: f32, // 0.95 - 95% memory usage
}

pub trait PressureHandler: Send + Sync {
    fn handle_pressure(&mut self, level: PressureLevel) -> BytesFreed;
    fn priority(&self) -> u32;
}

// Example pressure handlers
pub struct TextureCachePressureHandler {
    texture_cache: Arc<Mutex<TextureCache>>,
}

impl PressureHandler for TextureCachePressureHandler {
    fn handle_pressure(&mut self, level: PressureLevel) -> BytesFreed {
        let mut cache = self.texture_cache.lock().unwrap();
        
        match level {
            PressureLevel::Low => {
                // Evict textures not used in last 60 seconds
                cache.evict_older_than(Duration::from_secs(60))
            }
            PressureLevel::Medium => {
                // Evict textures not used in last 10 seconds
                cache.evict_older_than(Duration::from_secs(10))
            }
            PressureLevel::High => {
                // Keep only textures used this frame
                cache.evict_all_but_active()
            }
            PressureLevel::Critical => {
                // Drop quality of active textures
                cache.reduce_quality(0.5) + cache.evict_all_inactive()
            }
        }
    }
    
    fn priority(&self) -> u32 {
        1 // Lower priority - evict textures before creature data
    }
}

pub struct CreatureMemoryPressureHandler {
    creature_storage: Arc<Mutex<CreatureStorage>>,
}

impl PressureHandler for CreatureMemoryPressureHandler {
    fn handle_pressure(&mut self, level: PressureLevel) -> BytesFreed {
        let mut storage = self.creature_storage.lock().unwrap();
        
        match level {
            PressureLevel::Low => {
                // Compress cold data
                storage.compress_cold_data()
            }
            PressureLevel::Medium => {
                // Move warm data to cold storage for distant creatures
                storage.demote_distant_warm_data(100.0)
            }
            PressureLevel::High => {
                // Aggressively compress all non-essential data
                storage.compress_all_cold_data() +
                storage.reduce_memory_history(10) // Keep only 10 recent memories
            }
            PressureLevel::Critical => {
                // Emergency measures - simplify distant creatures
                storage.simplify_distant_creatures(200.0)
            }
        }
    }
    
    fn priority(&self) -> u32 {
        10 // Higher priority - preserve creature data when possible
    }
}
```

### Memory Profiling & Debugging

```rust
pub struct MemoryProfiler {
    samples: RingBuffer<MemorySample>,
    allocation_sites: HashMap<String, AllocationSite>,
    profiling_enabled: AtomicBool,
}

pub struct MemorySample {
    timestamp: Instant,
    total_allocated: usize,
    category_breakdown: HashMap<MemoryCategory, usize>,
    largest_allocations: Vec<AllocationInfo>,
}

pub struct AllocationSite {
    location: String,
    count: usize,
    total_size: usize,
    peak_size: usize,
    call_stack: Vec<String>,
}

impl MemoryProfiler {
    pub fn record_allocation(&mut self, size: usize, category: MemoryCategory, location: String) {
        if !self.profiling_enabled.load(Ordering::Relaxed) {
            return;
        }
        
        let site = self.allocation_sites.entry(location.clone())
            .or_insert_with(|| AllocationSite {
                location,
                count: 0,
                total_size: 0,
                peak_size: 0,
                call_stack: backtrace::Backtrace::new()
                    .frames()
                    .iter()
                    .take(10)
                    .map(|f| format!("{:?}", f))
                    .collect(),
            });
        
        site.count += 1;
        site.total_size += size;
        site.peak_size = site.peak_size.max(site.total_size);
    }
    
    pub fn generate_report(&self) -> MemoryReport {
        let mut report = MemoryReport::default();
        
        // Top allocators by size
        let mut sites: Vec<_> = self.allocation_sites.values().collect();
        sites.sort_by_key(|s| s.total_size);
        sites.reverse();
        
        report.top_allocators = sites.iter()
            .take(20)
            .map(|s| AllocationSiteReport {
                location: s.location.clone(),
                total_size: s.total_size,
                allocation_count: s.count,
                average_size: s.total_size / s.count.max(1),
            })
            .collect();
        
        // Memory usage over time
        report.usage_timeline = self.samples.get_recent(1000)
            .into_iter()
            .map(|s| UsagePoint {
                timestamp: s.timestamp,
                total_mb: s.total_allocated as f32 / 1024.0 / 1024.0,
            })
            .collect();
        
        report
    }
}

// Debug allocator wrapper
#[cfg(debug_assertions)]
pub struct DebugAllocator<A: GlobalAlloc> {
    inner: A,
    tracker: Arc<Mutex<AllocationTracker>>,
}

#[cfg(debug_assertions)]
unsafe impl<A: GlobalAlloc> GlobalAlloc for DebugAllocator<A> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = self.inner.alloc(layout);
        
        if !ptr.is_null() {
            let mut tracker = self.tracker.lock().unwrap();
            tracker.track_allocation(ptr, layout.size());
        }
        
        ptr
    }
    
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut tracker = self.tracker.lock().unwrap();
        tracker.track_deallocation(ptr, layout.size());
        
        self.inner.dealloc(ptr, layout);
    }
}
```

### Optimization Strategies

```rust
// Memory layout optimization
#[repr(C)]
pub struct OptimizedCreatureLayout {
    // Group by access pattern
    pub simulation_data: SimulationData,    // 128 bytes - accessed every frame
    pub ai_data: AIData,                    // 256 bytes - accessed every AI tick
    pub rendering_data: RenderingData,      // 64 bytes - accessed every render
    pub persistence_data: *mut PersistenceData, // Cold - accessed on save/load
}

// SIMD-friendly layouts
#[repr(C, align(32))]
pub struct SimdPositions {
    pub x: [f32; 8], // 8 creatures' X coordinates
    pub y: [f32; 8], // 8 creatures' Y coordinates  
    pub z: [f32; 8], // 8 creatures' Z coordinates
}

// Bit packing for flags
#[derive(Clone, Copy)]
pub struct CreatureFlags {
    data: u32,
}

impl CreatureFlags {
    pub const ALIVE: u32 = 1 << 0;
    pub const MOVING: u32 = 1 << 1;
    pub const EATING: u32 = 1 << 2;
    pub const SLEEPING: u32 = 1 << 3;
    pub const IN_COMBAT: u32 = 1 << 4;
    pub const PREGNANT: u32 = 1 << 5;
    pub const DISEASED: u32 = 1 << 6;
    pub const SELECTED: u32 = 1 << 7;
    
    pub fn set(&mut self, flag: u32) {
        self.data |= flag;
    }
    
    pub fn clear(&mut self, flag: u32) {
        self.data &= !flag;
    }
    
    pub fn is_set(&self, flag: u32) -> bool {
        self.data & flag != 0
    }
}

// String interning for repeated strings
pub struct StringInterner {
    strings: HashMap<String, InternedString>,
    storage: Vec<String>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct InternedString(u32);

impl StringInterner {
    pub fn intern(&mut self, s: &str) -> InternedString {
        if let Some(&interned) = self.strings.get(s) {
            return interned;
        }
        
        let id = self.storage.len() as u32;
        self.storage.push(s.to_string());
        let interned = InternedString(id);
        self.strings.insert(s.to_string(), interned);
        interned
    }
    
    pub fn get(&self, interned: InternedString) -> &str {
        &self.storage[interned.0 as usize]
    }
}
```

## Memory Budgets and Limits

```rust
pub struct MemoryLimits {
    // Per-creature limits
    pub max_memories_per_creature: usize,          // 100
    pub max_relationships_per_creature: usize,     // 50
    pub max_learned_behaviors: usize,              // 20
    
    // System limits
    pub max_active_creatures: usize,               // 2000
    pub max_cached_paths: usize,                   // 1000
    pub max_pooled_objects: usize,                 // 10000
    
    // Collection size limits
    pub max_spatial_hash_entries: usize,           // 100000
    pub max_event_queue_size: usize,               // 10000
    pub max_render_commands: usize,                // 50000
}
```

## Integration with ECS

```rust
// Memory-efficient ECS integration
pub struct MemoryEfficientWorld {
    entities: EntityStorage,
    components: ComponentRegistry,
    systems: SystemScheduler,
    memory_manager: MemoryManager,
}

impl MemoryEfficientWorld {
    pub fn spawn_entity(&mut self) -> EntityBuilder {
        let entity = self.entities.create();
        EntityBuilder {
            entity,
            world: self,
            components: Vec::with_capacity(8), // Pre-allocate common size
        }
    }
    
    pub fn despawn_entity(&mut self, entity: Entity) {
        // Return components to pools
        for component_type in self.components.get_component_types(entity) {
            if let Some(pool) = self.memory_manager.get_pool(component_type) {
                let component_ptr = self.components.remove_raw(entity, component_type);
                pool.deallocate(component_ptr);
            }
        }
        
        self.entities.destroy(entity);
    }
}
```

## Performance Metrics

```rust
pub struct MemoryPerformanceMetrics {
    pub allocation_rate: f32,         // Allocations per second
    pub deallocation_rate: f32,       // Deallocations per second
    pub fragmentation: f32,           // 0.0 = no fragmentation
    pub cache_hit_rate: f32,          // Pool allocation hits
    pub gc_pause_time: Duration,      // Time spent in memory management
    pub memory_bandwidth: f32,        // GB/s
}
```

This comprehensive memory management strategy ensures the simulation can handle thousands of creatures efficiently while maintaining consistent performance and gracefully handling memory pressure situations.