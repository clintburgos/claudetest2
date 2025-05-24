# Cache Management and Invalidation System

## Overview

The cache management system provides efficient caching strategies across all game systems, with automatic invalidation, memory pressure handling, and performance monitoring.

## Cache Architecture

### Core Cache Manager

```rust
pub struct CacheManager {
    // Cache registry
    caches: HashMap<CacheId, Box<dyn Cache>>,
    
    // Global settings
    total_memory_limit: usize,
    current_memory_usage: AtomicUsize,
    
    // Invalidation system
    invalidation_graph: InvalidationGraph,
    pending_invalidations: Mutex<Vec<InvalidationRequest>>,
    
    // Performance tracking
    hit_rates: HashMap<CacheId, HitRateTracker>,
    memory_pressure: MemoryPressure,
}

pub trait Cache: Send + Sync {
    fn id(&self) -> CacheId;
    fn memory_usage(&self) -> usize;
    fn clear(&mut self);
    fn evict_oldest(&mut self, count: usize);
    fn hit_rate(&self) -> f32;
    fn invalidate(&mut self, key: &dyn Any);
    fn shrink_to_fit(&mut self);
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum CacheId {
    Pathfinding,
    SpatialQueries,
    Animation,
    Conversation,
    Genetics,
    Rendering,
    UI,
    Custom(u32),
}
```

### Cache Types

#### LRU Cache

```rust
pub struct LruCache<K: Hash + Eq, V> {
    map: HashMap<K, CacheEntry<V>>,
    order: LinkedList<K>,
    capacity: usize,
    memory_usage: usize,
}

struct CacheEntry<V> {
    value: V,
    node: *mut Node<K>,
    size: usize,
    last_access: Instant,
}

impl<K: Hash + Eq + Clone, V> Cache for LruCache<K, V> {
    fn evict_oldest(&mut self, count: usize) {
        for _ in 0..count {
            if let Some(key) = self.order.pop_back() {
                if let Some(entry) = self.map.remove(&key) {
                    self.memory_usage -= entry.size;
                }
            }
        }
    }
}
```

#### Time-Based Cache

```rust
pub struct TimeBasedCache<K: Hash + Eq, V> {
    entries: HashMap<K, TimedEntry<V>>,
    expiry_heap: BinaryHeap<ExpiryEntry<K>>,
    default_ttl: Duration,
    max_entries: usize,
}

struct TimedEntry<V> {
    value: V,
    expires_at: Instant,
    size: usize,
}

impl<K: Hash + Eq + Clone, V> TimeBasedCache<K, V> {
    pub fn get(&mut self, key: &K) -> Option<&V> {
        self.remove_expired();
        
        self.entries.get(key).map(|entry| {
            if entry.expires_at > Instant::now() {
                Some(&entry.value)
            } else {
                None
            }
        }).flatten()
    }
    
    fn remove_expired(&mut self) {
        let now = Instant::now();
        
        while let Some(entry) = self.expiry_heap.peek() {
            if entry.expires_at <= now {
                let entry = self.expiry_heap.pop().unwrap();
                self.entries.remove(&entry.key);
            } else {
                break;
            }
        }
    }
}
```

#### Spatial Cache

```rust
pub struct SpatialCache {
    grid_cache: HashMap<GridCell, CellCache>,
    query_cache: LruCache<QueryKey, Vec<Entity>>,
    dirty_cells: HashSet<GridCell>,
    invalidation_radius: f32,
}

struct CellCache {
    entities: Vec<Entity>,
    last_update: u32,
    version: u32,
}

impl SpatialCache {
    pub fn invalidate_area(&mut self, center: Vec3, radius: f32) {
        let affected_cells = self.get_cells_in_radius(center, radius);
        
        for cell in affected_cells {
            self.dirty_cells.insert(cell);
            self.grid_cache.remove(&cell);
        }
        
        // Invalidate affected queries
        self.query_cache.clear(); // Simple approach, could be optimized
    }
}
```

## Invalidation System

### Invalidation Graph

```rust
pub struct InvalidationGraph {
    dependencies: HashMap<CacheId, Vec<CacheId>>,
    invalidation_rules: Vec<InvalidationRule>,
}

pub struct InvalidationRule {
    trigger: InvalidationTrigger,
    affected_caches: Vec<CacheId>,
    propagation: PropagationType,
}

pub enum InvalidationTrigger {
    ComponentChanged(TypeId),
    EntityMoved { old_pos: Vec3, new_pos: Vec3 },
    SystemUpdate(SystemId),
    TimeElapsed(Duration),
    MemoryPressure(MemoryPressure),
    Custom(Box<dyn Fn() -> bool>),
}

pub enum PropagationType {
    Immediate,
    Deferred,
    Cascading { max_depth: u32 },
    Selective { filter: Box<dyn Fn(&CacheId) -> bool> },
}

impl InvalidationGraph {
    pub fn propagate_invalidation(&self, source: CacheId) -> Vec<CacheId> {
        let mut to_invalidate = vec![source];
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(source);
        
        while let Some(cache_id) = queue.pop_front() {
            if !visited.insert(cache_id) {
                continue;
            }
            
            if let Some(deps) = self.dependencies.get(&cache_id) {
                for &dep in deps {
                    to_invalidate.push(dep);
                    queue.push_back(dep);
                }
            }
        }
        
        to_invalidate
    }
}
```

### Invalidation Strategies

```rust
pub enum InvalidationStrategy {
    // Invalidate entire cache
    Full,
    
    // Invalidate specific entries
    Selective {
        predicate: Box<dyn Fn(&dyn Any) -> bool>,
    },
    
    // Invalidate based on spatial proximity
    Spatial {
        center: Vec3,
        radius: f32,
    },
    
    // Invalidate entries older than threshold
    Age {
        max_age: Duration,
    },
    
    // Invalidate least recently used
    LRU {
        keep_count: usize,
    },
}

pub struct InvalidationRequest {
    cache_id: CacheId,
    strategy: InvalidationStrategy,
    priority: InvalidationPriority,
    timestamp: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum InvalidationPriority {
    Critical,  // Invalidate immediately
    High,      // Invalidate this frame
    Normal,    // Invalidate when convenient
    Low,       // Invalidate under memory pressure
}
```

## System-Specific Caches

### Pathfinding Cache

```rust
pub struct PathfindingCache {
    path_cache: LruCache<PathKey, Path>,
    navmesh_cache: HashMap<ChunkId, NavMesh>,
    obstacle_cache: SpatialCache,
    
    // Invalidation tracking
    moving_obstacles: HashMap<Entity, Vec3>,
    static_obstacles: HashSet<Entity>,
}

#[derive(Hash, Eq, PartialEq)]
struct PathKey {
    from: IVec3,  // Quantized position
    to: IVec3,    // Quantized position
    creature_type: CreatureType,
}

impl PathfindingCache {
    pub fn get_path(&mut self, from: Vec3, to: Vec3, creature_type: CreatureType) -> Option<&Path> {
        let key = PathKey {
            from: quantize_position(from),
            to: quantize_position(to),
            creature_type,
        };
        
        self.path_cache.get(&key)
    }
    
    pub fn invalidate_for_obstacle_movement(&mut self, entity: Entity, old_pos: Vec3, new_pos: Vec3) {
        // Invalidate paths that cross the old or new position
        let invalidation_bounds = AABB::from_points(&[old_pos, new_pos]).expanded(5.0);
        
        self.path_cache.retain(|key, path| {
            !path.intersects_bounds(&invalidation_bounds)
        });
    }
}
```

### Animation Cache

```rust
pub struct AnimationCache {
    // Sampled animation poses
    pose_cache: LruCache<PoseKey, AnimationPose>,
    
    // Blended animations
    blend_cache: LruCache<BlendKey, BlendedPose>,
    
    // Expression combinations
    expression_cache: HashMap<ExpressionKey, ExpressionData>,
    
    // Particle effect instances
    particle_cache: ObjectPool<ParticleEffect>,
}

#[derive(Hash, Eq, PartialEq)]
struct PoseKey {
    animation_id: AnimationId,
    time: OrderedFloat<f32>,
    lod_level: u8,
}

impl AnimationCache {
    pub fn get_or_sample(&mut self, animation: &Animation, time: f32, lod: u8) -> AnimationPose {
        let quantized_time = quantize_time(time, lod);
        let key = PoseKey {
            animation_id: animation.id,
            time: OrderedFloat(quantized_time),
            lod_level: lod,
        };
        
        if let Some(pose) = self.pose_cache.get(&key) {
            return pose.clone();
        }
        
        let pose = animation.sample(time);
        self.pose_cache.insert(key, pose.clone());
        pose
    }
}
```

### Conversation Cache

```rust
pub struct ConversationCache {
    // Response generation cache
    response_cache: LruCache<ResponseKey, ConversationResponse>,
    
    // Topic relevance cache
    topic_relevance: HashMap<(Entity, Topic), f32>,
    
    // Relationship context cache
    context_cache: TimeBasedCache<ContextKey, ConversationContext>,
    
    // Language pattern cache
    pattern_cache: HashMap<(Entity, Entity), LanguagePatterns>,
}

#[derive(Hash, Eq, PartialEq)]
struct ResponseKey {
    speaker: Entity,
    topic: Topic,
    emotion: Emotion,
    relationship_strength: OrderedFloat<f32>,
}

impl ConversationCache {
    pub fn invalidate_for_relationship_change(&mut self, a: Entity, b: Entity) {
        // Clear caches that depend on this relationship
        self.response_cache.retain(|key, _| {
            key.speaker != a && key.speaker != b
        });
        
        self.context_cache.clear(); // Could be more selective
        self.pattern_cache.remove(&(a, b));
        self.pattern_cache.remove(&(b, a));
    }
}
```

## Memory Management

### Memory Pressure Handling

```rust
pub struct MemoryPressureMonitor {
    thresholds: MemoryThresholds,
    current_usage: AtomicUsize,
    total_available: usize,
    pressure_level: AtomicU8,
}

#[derive(Debug, Clone, Copy)]
pub struct MemoryThresholds {
    low: f32,     // 0.6 - Start monitoring
    medium: f32,  // 0.75 - Start evicting
    high: f32,    // 0.85 - Aggressive eviction
    critical: f32, // 0.95 - Emergency measures
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MemoryPressure {
    None = 0,
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

impl CacheManager {
    pub fn handle_memory_pressure(&mut self) {
        let pressure = self.memory_pressure.get();
        
        match pressure {
            MemoryPressure::None => return,
            
            MemoryPressure::Low => {
                // Evict old entries from low-priority caches
                self.evict_by_priority(vec![CacheId::UI, CacheId::Animation], 0.1);
            }
            
            MemoryPressure::Medium => {
                // Evict from all caches proportionally
                self.evict_proportionally(0.2);
            }
            
            MemoryPressure::High => {
                // Aggressive eviction, keep only essential data
                self.evict_all_except_essential(0.5);
            }
            
            MemoryPressure::Critical => {
                // Clear all caches except critical paths
                self.emergency_clear();
            }
        }
    }
    
    fn evict_proportionally(&mut self, ratio: f32) {
        for (_, cache) in &mut self.caches {
            let current_size = cache.memory_usage();
            let to_free = (current_size as f32 * ratio) as usize;
            cache.evict_oldest(to_free / 1024); // Rough estimate
        }
    }
}
```

### Cache Warming

```rust
pub struct CacheWarmer {
    warming_queue: VecDeque<WarmingTask>,
    active_tasks: Vec<JoinHandle<()>>,
    thread_pool: ThreadPool,
}

pub enum WarmingTask {
    PathfindingGrid {
        area: AABB,
        resolution: u32,
    },
    AnimationPoses {
        animations: Vec<AnimationId>,
        lod_levels: Vec<u8>,
    },
    SpatialQueries {
        positions: Vec<Vec3>,
        radius: f32,
    },
}

impl CacheWarmer {
    pub fn warm_area_async(&mut self, center: Vec3, radius: f32) {
        // Queue warming tasks for the area
        self.warming_queue.push_back(WarmingTask::PathfindingGrid {
            area: AABB::from_center_size(center, Vec3::splat(radius * 2.0)),
            resolution: 10,
        });
        
        self.warming_queue.push_back(WarmingTask::SpatialQueries {
            positions: generate_grid_points(center, radius, 20),
            radius: 50.0,
        });
        
        self.process_queue();
    }
}
```

## Performance Monitoring

```rust
pub struct CacheMetrics {
    pub hit_rates: HashMap<CacheId, f32>,
    pub memory_usage: HashMap<CacheId, usize>,
    pub eviction_counts: HashMap<CacheId, u32>,
    pub invalidation_counts: HashMap<CacheId, u32>,
    pub average_entry_age: HashMap<CacheId, Duration>,
}

pub struct CacheProfiler {
    metrics: CacheMetrics,
    history: RingBuffer<CacheMetrics>,
    anomaly_detector: AnomalyDetector,
}

impl CacheProfiler {
    pub fn analyze(&self) -> CacheAnalysis {
        CacheAnalysis {
            underperforming_caches: self.find_low_hit_rate_caches(),
            memory_hogs: self.find_high_memory_caches(),
            thrashing_caches: self.find_thrashing_caches(),
            optimization_suggestions: self.generate_suggestions(),
        }
    }
    
    fn find_thrashing_caches(&self) -> Vec<CacheId> {
        self.metrics.eviction_counts
            .iter()
            .filter(|(_, &count)| count > 100) // High eviction rate
            .map(|(id, _)| *id)
            .collect()
    }
}
```

## Integration Example

```rust
pub fn setup_cache_system(world: &mut World) {
    let mut cache_manager = CacheManager::new(512 * 1024 * 1024); // 512MB limit
    
    // Register caches
    cache_manager.register(Box::new(PathfindingCache::new(10000)));
    cache_manager.register(Box::new(AnimationCache::new(5000)));
    cache_manager.register(Box::new(SpatialCache::new(20.0)));
    cache_manager.register(Box::new(ConversationCache::new(1000)));
    
    // Setup invalidation rules
    cache_manager.add_invalidation_rule(InvalidationRule {
        trigger: InvalidationTrigger::ComponentChanged(TypeId::of::<Position>()),
        affected_caches: vec![CacheId::Pathfinding, CacheId::SpatialQueries],
        propagation: PropagationType::Immediate,
    });
    
    cache_manager.add_invalidation_rule(InvalidationRule {
        trigger: InvalidationTrigger::TimeElapsed(Duration::from_secs(60)),
        affected_caches: vec![CacheId::Conversation],
        propagation: PropagationType::Deferred,
    });
    
    world.insert_resource(cache_manager);
}

// In game loop
pub fn update_caches(world: &mut World) {
    let mut cache_manager = world.get_resource_mut::<CacheManager>().unwrap();
    
    // Process invalidations
    cache_manager.process_invalidations();
    
    // Handle memory pressure
    cache_manager.handle_memory_pressure();
    
    // Warm caches for visible area
    if let Some(camera) = world.get_resource::<Camera>() {
        cache_manager.warm_area(camera.position, camera.view_distance);
    }
}
```

This cache management system provides efficient caching with smart invalidation and memory management across all game systems.