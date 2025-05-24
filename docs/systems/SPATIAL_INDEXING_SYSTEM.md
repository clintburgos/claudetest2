# Unified Spatial Indexing System

## Overview

The spatial indexing system provides efficient O(log n) spatial queries across all game systems. It uses a hierarchical grid with dynamic subdivision to handle varying creature densities while maintaining consistent performance.

## Architecture

### Core Spatial Index

```rust
pub struct SpatialIndex {
    // Fixed-size top-level grid
    grid: HashMap<GridCell, CellData>,
    
    // Configuration
    cell_size: f32,              // 20.0 world units (unified)
    subdivision_threshold: u32,   // 50 entities per cell
    max_subdivision_depth: u8,    // 3 levels max
    
    // Optimization
    entity_positions: HashMap<Entity, Vec3>,
    dirty_cells: HashSet<GridCell>,
    update_buffer: Vec<(Entity, Vec3, Vec3)>, // entity, old_pos, new_pos
}

pub struct CellData {
    entities: SmallVec<[Entity; 32]>,
    subdivision: Option<Box<SubGrid>>,
    bounds: AABB,
    last_update: u32,
}

pub struct SubGrid {
    cells: [CellData; 4], // 2x2 subdivision
    depth: u8,
}
```

### Grid Coordinate System

```rust
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct GridCell {
    x: i32,
    y: i32,
    z: i32, // For future 3D support, currently always 0
}

impl GridCell {
    pub fn from_world_pos(pos: Vec3, cell_size: f32) -> Self {
        Self {
            x: (pos.x / cell_size).floor() as i32,
            y: (pos.y / cell_size).floor() as i32,
            z: 0,
        }
    }
    
    pub fn neighbors(&self) -> [GridCell; 8] {
        [
            GridCell { x: self.x - 1, y: self.y - 1, z: 0 },
            GridCell { x: self.x,     y: self.y - 1, z: 0 },
            GridCell { x: self.x + 1, y: self.y - 1, z: 0 },
            GridCell { x: self.x - 1, y: self.y,     z: 0 },
            GridCell { x: self.x + 1, y: self.y,     z: 0 },
            GridCell { x: self.x - 1, y: self.y + 1, z: 0 },
            GridCell { x: self.x,     y: self.y + 1, z: 0 },
            GridCell { x: self.x + 1, y: self.y + 1, z: 0 },
        ]
    }
}
```

## Query Operations

### Range Query

Find all entities within a radius:

```rust
pub struct RangeQuery {
    center: Vec3,
    radius: f32,
    filter: Option<EntityFilter>,
    max_results: Option<usize>,
}

impl SpatialIndex {
    pub fn query_range(&self, query: &RangeQuery) -> Vec<(Entity, f32)> {
        let mut results = Vec::new();
        let radius_sq = query.radius * query.radius;
        
        // Find cells that overlap query sphere
        let min_cell = GridCell::from_world_pos(
            query.center - Vec3::splat(query.radius), 
            self.cell_size
        );
        let max_cell = GridCell::from_world_pos(
            query.center + Vec3::splat(query.radius), 
            self.cell_size
        );
        
        // Check each potentially overlapping cell
        for x in min_cell.x..=max_cell.x {
            for y in min_cell.y..=max_cell.y {
                let cell = GridCell { x, y, z: 0 };
                
                if let Some(data) = self.grid.get(&cell) {
                    self.query_cell(data, query, radius_sq, &mut results);
                }
                
                if let Some(max) = query.max_results {
                    if results.len() >= max {
                        results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
                        results.truncate(max);
                        return results;
                    }
                }
            }
        }
        
        results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        results
    }
    
    fn query_cell(&self, cell: &CellData, query: &RangeQuery, 
                  radius_sq: f32, results: &mut Vec<(Entity, f32)>) {
        // Check subdivision if present
        if let Some(sub) = &cell.subdivision {
            for subcell in &sub.cells {
                if subcell.bounds.intersects_sphere(query.center, query.radius) {
                    self.query_cell(subcell, query, radius_sq, results);
                }
            }
            return;
        }
        
        // Check entities in cell
        for &entity in &cell.entities {
            if let Some(filter) = &query.filter {
                if !filter.matches(entity) {
                    continue;
                }
            }
            
            if let Some(pos) = self.entity_positions.get(&entity) {
                let dist_sq = (*pos - query.center).length_squared();
                if dist_sq <= radius_sq {
                    results.push((entity, dist_sq.sqrt()));
                }
            }
        }
    }
}
```

### K-Nearest Neighbors

Find the K nearest entities:

```rust
pub struct KNearestQuery {
    center: Vec3,
    k: usize,
    filter: Option<EntityFilter>,
    max_distance: Option<f32>,
}

impl SpatialIndex {
    pub fn query_k_nearest(&self, query: &KNearestQuery) -> Vec<(Entity, f32)> {
        // Use expanding ring search for efficiency
        let mut search_radius = self.cell_size;
        let mut results = BinaryHeap::new();
        
        loop {
            let range_query = RangeQuery {
                center: query.center,
                radius: search_radius,
                filter: query.filter.clone(),
                max_results: None,
            };
            
            let candidates = self.query_range(&range_query);
            
            for (entity, dist) in candidates {
                if let Some(max_dist) = query.max_distance {
                    if dist > max_dist {
                        continue;
                    }
                }
                
                results.push(Reverse((OrderedFloat(dist), entity)));
                if results.len() > query.k {
                    results.pop();
                }
            }
            
            // Check if we have enough results or searched far enough
            if results.len() >= query.k || search_radius > 1000.0 {
                break;
            }
            
            search_radius *= 2.0;
        }
        
        results.into_sorted_vec()
            .into_iter()
            .map(|Reverse((dist, entity))| (entity, dist.0))
            .collect()
    }
}
```

### Ray Query

Find entities along a ray:

```rust
pub struct RayQuery {
    origin: Vec3,
    direction: Vec3,
    max_distance: f32,
    filter: Option<EntityFilter>,
}

impl SpatialIndex {
    pub fn query_ray(&self, query: &RayQuery) -> Vec<(Entity, f32)> {
        let mut results = Vec::new();
        
        // DDA algorithm for ray-grid traversal
        let ray = Ray {
            origin: query.origin,
            direction: query.direction.normalize(),
        };
        
        let mut t = 0.0;
        while t < query.max_distance {
            let pos = ray.origin + ray.direction * t;
            let cell = GridCell::from_world_pos(pos, self.cell_size);
            
            if let Some(data) = self.grid.get(&cell) {
                self.check_ray_cell(data, &ray, query, &mut results);
            }
            
            // Step to next cell boundary
            t += self.cell_size * 0.5; // Conservative step
        }
        
        results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        results
    }
}
```

## Update Operations

### Batched Position Updates

```rust
impl SpatialIndex {
    pub fn update_positions(&mut self, updates: Vec<(Entity, Vec3)>) {
        // Batch updates for efficiency
        self.update_buffer.clear();
        
        for (entity, new_pos) in updates {
            if let Some(&old_pos) = self.entity_positions.get(&entity) {
                if old_pos != new_pos {
                    self.update_buffer.push((entity, old_pos, new_pos));
                }
            }
        }
        
        // Process removals
        for &(entity, old_pos, _) in &self.update_buffer {
            let old_cell = GridCell::from_world_pos(old_pos, self.cell_size);
            if let Some(data) = self.grid.get_mut(&old_cell) {
                data.entities.retain(|&e| e != entity);
                self.dirty_cells.insert(old_cell);
            }
        }
        
        // Process additions
        for &(entity, _, new_pos) in &self.update_buffer {
            let new_cell = GridCell::from_world_pos(new_pos, self.cell_size);
            let data = self.grid.entry(new_cell).or_insert_with(|| CellData {
                entities: SmallVec::new(),
                subdivision: None,
                bounds: AABB::from_center_size(
                    new_cell.to_world_pos(self.cell_size),
                    Vec3::splat(self.cell_size)
                ),
                last_update: 0,
            });
            
            data.entities.push(entity);
            self.entity_positions.insert(entity, new_pos);
            self.dirty_cells.insert(new_cell);
        }
        
        // Check for subdivision needs
        for cell in self.dirty_cells.drain() {
            self.check_subdivision(&cell);
        }
    }
}
```

### Dynamic Subdivision

```rust
impl SpatialIndex {
    fn check_subdivision(&mut self, cell: &GridCell) {
        let data = match self.grid.get_mut(cell) {
            Some(d) => d,
            None => return,
        };
        
        // Check if subdivision needed
        if data.entities.len() > self.subdivision_threshold as usize {
            if data.subdivision.is_none() {
                self.subdivide_cell(cell);
            }
        } else if data.entities.len() < (self.subdivision_threshold / 4) as usize {
            // Collapse subdivision if too few entities
            data.subdivision = None;
        }
    }
    
    fn subdivide_cell(&mut self, cell: &GridCell) {
        let data = match self.grid.get_mut(cell) {
            Some(d) => d,
            None => return,
        };
        
        // Create 2x2 subdivision
        let sub_size = self.cell_size / 2.0;
        let base_pos = cell.to_world_pos(self.cell_size);
        
        let mut subgrid = SubGrid {
            cells: Default::default(),
            depth: 1,
        };
        
        // Initialize subcells
        for i in 0..4 {
            let offset = Vec3::new(
                (i % 2) as f32 * sub_size,
                (i / 2) as f32 * sub_size,
                0.0
            );
            
            subgrid.cells[i] = CellData {
                entities: SmallVec::new(),
                subdivision: None,
                bounds: AABB::from_center_size(
                    base_pos + offset + Vec3::splat(sub_size / 2.0),
                    Vec3::splat(sub_size)
                ),
                last_update: 0,
            };
        }
        
        // Redistribute entities
        for &entity in &data.entities {
            if let Some(&pos) = self.entity_positions.get(&entity) {
                let local = pos - base_pos;
                let idx = ((local.x >= sub_size) as usize) + 
                         ((local.y >= sub_size) as usize) * 2;
                subgrid.cells[idx].entities.push(entity);
            }
        }
        
        data.subdivision = Some(Box::new(subgrid));
        data.entities.clear();
    }
}
```

## Integration with Systems

### Movement System Integration

```rust
pub struct MovementSpatialAdapter {
    spatial_index: Arc<RwLock<SpatialIndex>>,
}

impl MovementSpatialAdapter {
    pub fn update_creature_positions(&self, movements: &[(Entity, Vec3, Vec3)]) {
        let updates: Vec<(Entity, Vec3)> = movements
            .iter()
            .map(|&(entity, _, new_pos)| (entity, new_pos))
            .collect();
            
        self.spatial_index.write().unwrap().update_positions(updates);
    }
    
    pub fn find_obstacles(&self, from: Vec3, to: Vec3) -> Vec<Entity> {
        let query = RayQuery {
            origin: from,
            direction: to - from,
            max_distance: (to - from).length(),
            filter: Some(EntityFilter::HasComponent::<Obstacle>),
        };
        
        self.spatial_index.read().unwrap()
            .query_ray(&query)
            .into_iter()
            .map(|(e, _)| e)
            .collect()
    }
}
```

### Social System Integration

```rust
pub struct SocialSpatialAdapter {
    spatial_index: Arc<RwLock<SpatialIndex>>,
}

impl SocialSpatialAdapter {
    pub fn find_nearby_creatures(&self, creature: Entity, radius: f32) -> Vec<Entity> {
        let pos = self.get_creature_position(creature);
        
        let query = RangeQuery {
            center: pos,
            radius,
            filter: Some(EntityFilter::HasComponent::<Creature>),
            max_results: Some(50), // Limit for performance
        };
        
        self.spatial_index.read().unwrap()
            .query_range(&query)
            .into_iter()
            .map(|(e, _)| e)
            .filter(|&e| e != creature)
            .collect()
    }
}
```

### Resource System Integration

```rust
pub struct ResourceSpatialAdapter {
    spatial_index: Arc<RwLock<SpatialIndex>>,
}

impl ResourceSpatialAdapter {
    pub fn find_nearest_resource(&self, pos: Vec3, resource_type: ResourceType) -> Option<(Entity, f32)> {
        let query = KNearestQuery {
            center: pos,
            k: 1,
            filter: Some(EntityFilter::And(vec![
                EntityFilter::HasComponent::<Resource>,
                EntityFilter::Custom(Box::new(move |e| {
                    // Check resource type matches
                    true // Simplified
                })),
            ])),
            max_distance: Some(100.0),
        };
        
        self.spatial_index.read().unwrap()
            .query_k_nearest(&query)
            .into_iter()
            .next()
    }
}
```

## Performance Optimizations

### Cache-Friendly Layout

```rust
// Keep hot data together
pub struct SpatialCache {
    // Frequently accessed together
    recent_queries: RingBuffer<QueryResult>,
    hot_cells: Vec<GridCell>,
    entity_cache: FxHashMap<Entity, CachedPosition>,
}

pub struct CachedPosition {
    position: Vec3,
    cell: GridCell,
    last_update: u32,
}
```

### Parallel Updates

```rust
use rayon::prelude::*;

impl SpatialIndex {
    pub fn parallel_update(&mut self, updates: Vec<(Entity, Vec3)>) {
        // Group updates by cell
        let mut cell_updates: HashMap<GridCell, Vec<(Entity, Vec3)>> = HashMap::new();
        
        for (entity, new_pos) in updates {
            let cell = GridCell::from_world_pos(new_pos, self.cell_size);
            cell_updates.entry(cell).or_default().push((entity, new_pos));
        }
        
        // Process each cell in parallel
        let results: Vec<_> = cell_updates
            .par_iter()
            .map(|(cell, updates)| {
                // Process updates for this cell
                (*cell, updates.clone())
            })
            .collect();
            
        // Apply results
        for (cell, updates) in results {
            // Update cell data
        }
    }
}
```

### Query Caching

```rust
pub struct QueryCache {
    range_cache: LruCache<RangeQueryKey, Vec<(Entity, f32)>>,
    knn_cache: LruCache<KNearestQueryKey, Vec<(Entity, f32)>>,
    cache_frame: u32,
}

impl QueryCache {
    pub fn get_or_compute_range(&mut self, 
                               index: &SpatialIndex, 
                               query: &RangeQuery,
                               current_frame: u32) -> Vec<(Entity, f32)> {
        // Invalidate old cache entries
        if current_frame > self.cache_frame + 5 {
            self.range_cache.clear();
            self.cache_frame = current_frame;
        }
        
        let key = RangeQueryKey::from(query);
        
        if let Some(result) = self.range_cache.get(&key) {
            return result.clone();
        }
        
        let result = index.query_range(query);
        self.range_cache.put(key, result.clone());
        result
    }
}
```

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_insertion_and_query() {
        let mut index = SpatialIndex::new(20.0);
        
        // Insert entities
        let updates = vec![
            (Entity::from_raw(1), Vec3::new(10.0, 10.0, 0.0)),
            (Entity::from_raw(2), Vec3::new(30.0, 10.0, 0.0)),
            (Entity::from_raw(3), Vec3::new(10.0, 30.0, 0.0)),
        ];
        
        index.update_positions(updates);
        
        // Query range
        let query = RangeQuery {
            center: Vec3::new(15.0, 15.0, 0.0),
            radius: 20.0,
            filter: None,
            max_results: None,
        };
        
        let results = index.query_range(&query);
        assert_eq!(results.len(), 2); // Should find entities 1 and 3
    }
    
    #[test]
    fn test_subdivision() {
        let mut index = SpatialIndex::new(20.0);
        index.subdivision_threshold = 5; // Low threshold for testing
        
        // Insert many entities in same cell
        let updates: Vec<_> = (0..10)
            .map(|i| (Entity::from_raw(i), Vec3::new(5.0 + i as f32 * 0.1, 5.0, 0.0)))
            .collect();
            
        index.update_positions(updates);
        
        // Check that subdivision occurred
        let cell = GridCell::from_world_pos(Vec3::new(5.0, 5.0, 0.0), 20.0);
        assert!(index.grid[&cell].subdivision.is_some());
    }
    
    #[bench]
    fn bench_range_query_1000_entities(b: &mut Bencher) {
        let mut index = SpatialIndex::new(20.0);
        
        // Insert 1000 entities
        let updates: Vec<_> = (0..1000)
            .map(|i| {
                let x = (i % 100) as f32 * 10.0;
                let y = (i / 100) as f32 * 10.0;
                (Entity::from_raw(i), Vec3::new(x, y, 0.0))
            })
            .collect();
            
        index.update_positions(updates);
        
        b.iter(|| {
            let query = RangeQuery {
                center: Vec3::new(500.0, 500.0, 0.0),
                radius: 50.0,
                filter: None,
                max_results: None,
            };
            
            test::black_box(index.query_range(&query));
        });
    }
}
```

This unified spatial indexing system provides consistent O(log n) performance across all game systems while handling varying entity densities through dynamic subdivision.