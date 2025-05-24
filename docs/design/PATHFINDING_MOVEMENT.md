# Pathfinding & Movement System Design

## Overview

The pathfinding and movement system enables creatures to navigate the world efficiently while avoiding obstacles, managing group movement, and adapting to different terrain types. The system balances performance with realistic movement patterns.

## Pathfinding Algorithm

### Hierarchical Pathfinding (HPA*)
We use Hierarchical Pathfinding A* for optimal performance with large worlds:

```rust
struct PathfindingSystem {
    // Low-res grid for long-distance planning
    cluster_graph: ClusterGraph,
    cluster_size: usize, // 16x16 tiles per cluster
    
    // High-res grid for local navigation
    detail_grid: Grid<TerrainCost>,
    
    // Cache for frequent paths
    path_cache: LruCache<(TilePos, TilePos), Path>,
}
```

### Algorithm Layers

1. **Cluster Level** - Find general route between regions
2. **Detail Level** - Refine path within clusters
3. **Local Avoidance** - Real-time steering around creatures

```rust
impl PathfindingSystem {
    fn find_path(&mut self, start: TilePos, goal: TilePos) -> Option<Path> {
        // Check cache first
        if let Some(cached) = self.path_cache.get(&(start, goal)) {
            return Some(cached.clone());
        }
        
        // Find cluster path
        let start_cluster = self.tile_to_cluster(start);
        let goal_cluster = self.tile_to_cluster(goal);
        
        let cluster_path = self.find_cluster_path(start_cluster, goal_cluster)?;
        
        // Refine path through each cluster
        let mut detailed_path = Vec::new();
        let mut current = start;
        
        for cluster in cluster_path {
            let cluster_goal = if cluster == goal_cluster {
                goal
            } else {
                self.get_cluster_exit(cluster, &cluster_path)
            };
            
            let segment = self.find_detailed_path(current, cluster_goal)?;
            detailed_path.extend(segment);
            current = cluster_goal;
        }
        
        let path = Path::new(detailed_path);
        self.path_cache.put((start, goal), path.clone());
        
        Some(path)
    }
}
```

## Movement Cost Calculation

### Terrain Costs
```rust
#[derive(Debug, Clone)]
struct TerrainCost {
    base_cost: f32,
    modifier: TerrainModifier,
}

enum TerrainModifier {
    Normal,
    Difficult(f32), // Multiplier > 1.0
    Impassable,
    Water { swim_cost: f32 },
    Slope { gradient: f32 }, // 0.0 to 1.0
}

fn calculate_move_cost(
    from: TilePos,
    to: TilePos,
    terrain: &TerrainGrid,
    creature: &Creature
) -> Option<f32> {
    let terrain_type = terrain.get(to)?;
    
    let base_cost = match terrain_type.modifier {
        TerrainModifier::Normal => 1.0,
        TerrainModifier::Difficult(mult) => mult,
        TerrainModifier::Impassable => return None,
        TerrainModifier::Water { swim_cost } => {
            if creature.can_swim() {
                swim_cost
            } else {
                return None;
            }
        },
        TerrainModifier::Slope { gradient } => {
            1.0 + gradient * 2.0 // Steep slopes cost more
        }
    };
    
    // Diagonal movement costs more
    let distance = if from.x != to.x && from.y != to.y {
        SQRT_2
    } else {
        1.0
    };
    
    Some(base_cost * distance * creature.movement_modifier())
}
```

## Obstacle Avoidance

### Dynamic Obstacle System
```rust
struct DynamicObstacle {
    position: Vec2,
    velocity: Vec2,
    radius: f32,
    obstacle_type: ObstacleType,
}

enum ObstacleType {
    Creature(CreatureId),
    Temporary(Duration), // Falling trees, etc.
    Environmental, // Rocks, buildings
}
```

### Velocity Obstacles (VO) Algorithm
```rust
fn calculate_avoidance_velocity(
    creature: &Creature,
    obstacles: &[DynamicObstacle],
    desired_velocity: Vec2,
    max_speed: f32,
) -> Vec2 {
    let mut velocity_obstacles = Vec::new();
    
    for obstacle in obstacles {
        if let Some(vo) = compute_velocity_obstacle(creature, obstacle) {
            velocity_obstacles.push(vo);
        }
    }
    
    // Find best velocity outside all VOs
    if velocity_obstacles.is_empty() {
        return desired_velocity;
    }
    
    // Sample velocities in a circle
    let samples = 32;
    let mut best_velocity = Vec2::ZERO;
    let mut best_score = f32::NEG_INFINITY;
    
    for i in 0..samples {
        let angle = (i as f32 / samples as f32) * TAU;
        let velocity = Vec2::from_angle(angle) * max_speed;
        
        if !is_inside_any_vo(&velocity, &velocity_obstacles) {
            let score = velocity.dot(desired_velocity);
            if score > best_score {
                best_score = score;
                best_velocity = velocity;
            }
        }
    }
    
    best_velocity
}
```

## Group Movement

### Flocking Behavior
```rust
struct FlockingParams {
    separation_weight: f32, // 1.5
    alignment_weight: f32,  // 1.0
    cohesion_weight: f32,   // 1.0
    
    perception_radius: f32,    // 10.0 tiles
    separation_radius: f32,    // 2.0 tiles
    
    max_neighbors: usize, // 10
}

fn calculate_flocking_force(
    creature: &Creature,
    neighbors: &[&Creature],
    params: &FlockingParams,
) -> Vec2 {
    let mut separation = Vec2::ZERO;
    let mut alignment = Vec2::ZERO;
    let mut cohesion = Vec2::ZERO;
    let mut neighbor_count = 0;
    
    for neighbor in neighbors.iter().take(params.max_neighbors) {
        let offset = creature.position - neighbor.position;
        let distance = offset.length();
        
        if distance < params.separation_radius && distance > 0.0 {
            // Separation: move away from close neighbors
            separation += offset.normalize() / distance;
        }
        
        if distance < params.perception_radius {
            // Alignment: match neighbor velocities
            alignment += neighbor.velocity;
            
            // Cohesion: move toward group center
            cohesion += neighbor.position;
            neighbor_count += 1;
        }
    }
    
    if neighbor_count > 0 {
        alignment /= neighbor_count as f32;
        cohesion = (cohesion / neighbor_count as f32) - creature.position;
    }
    
    separation * params.separation_weight +
    alignment * params.alignment_weight +
    cohesion * params.cohesion_weight
}
```

### Formation Movement
```rust
enum Formation {
    Line { spacing: f32 },
    Column { spacing: f32 },
    Wedge { angle: f32, spacing: f32 },
    Circle { radius: f32 },
    Scatter { min_distance: f32 },
}

struct GroupMovement {
    leader: CreatureId,
    formation: Formation,
    members: Vec<CreatureId>,
}

fn calculate_formation_position(
    index: usize,
    formation: &Formation,
    leader_pos: Vec2,
    leader_facing: Vec2,
) -> Vec2 {
    match formation {
        Formation::Line { spacing } => {
            let right = Vec2::new(-leader_facing.y, leader_facing.x);
            let offset = (index as f32 - (members.len() as f32 / 2.0)) * spacing;
            leader_pos + right * offset
        },
        Formation::Wedge { angle, spacing } => {
            let row = (index as f32).sqrt() as usize;
            let col = index - (row * row);
            let half_angle = angle / 2.0;
            
            let offset_angle = -half_angle + (col as f32 / row as f32) * angle;
            let offset_dist = row as f32 * spacing;
            
            leader_pos - leader_facing * offset_dist +
                Vec2::from_angle(offset_angle) * offset_dist
        },
        // ... other formations
    }
}
```

## Movement Smoothing

### Path Smoothing
```rust
fn smooth_path(path: Vec<TilePos>, terrain: &TerrainGrid) -> Vec<Vec2> {
    if path.len() < 3 {
        return path.iter().map(|p| p.to_world()).collect();
    }
    
    let mut smoothed = vec![path[0].to_world()];
    let mut current = 0;
    
    while current < path.len() - 1 {
        let mut farthest = current + 1;
        
        // Find farthest visible point
        for i in (current + 1)..path.len() {
            if has_line_of_sight(path[current], path[i], terrain) {
                farthest = i;
            } else {
                break;
            }
        }
        
        smoothed.push(path[farthest].to_world());
        current = farthest;
    }
    
    // Apply Catmull-Rom spline for extra smoothness
    catmull_rom_spline(&smoothed, 4)
}
```

### Velocity Smoothing
```rust
fn smooth_velocity(
    current_velocity: Vec2,
    desired_velocity: Vec2,
    max_acceleration: f32,
    delta_time: f32,
) -> Vec2 {
    let velocity_change = desired_velocity - current_velocity;
    let max_change = max_acceleration * delta_time;
    
    if velocity_change.length() <= max_change {
        desired_velocity
    } else {
        current_velocity + velocity_change.normalize() * max_change
    }
}
```

## Performance Optimizations

### Path Caching
```rust
struct PathCache {
    cache: LruCache<(TilePos, TilePos), Path>,
    validity_map: HashMap<PathId, PathValidity>,
}

struct PathValidity {
    path_id: PathId,
    last_validated: Instant,
    obstacles_version: u64,
}

impl PathCache {
    fn get_path(&mut self, start: TilePos, goal: TilePos) -> Option<Path> {
        if let Some(path) = self.cache.get(&(start, goal)) {
            if self.is_path_valid(path) {
                return Some(path.clone());
            }
        }
        None
    }
}
```

### LOD System for Pathfinding
```rust
enum PathfindingLOD {
    Full,      // Complete pathfinding
    Simplified, // Use cached or approximate paths
    Direct,    // Straight line with basic avoidance
}

fn get_pathfinding_lod(creature: &Creature, camera: &Camera) -> PathfindingLOD {
    let distance = (creature.position - camera.center).length();
    
    match distance {
        0.0..=100.0 => PathfindingLOD::Full,
        100.0..=500.0 => PathfindingLOD::Simplified,
        _ => PathfindingLOD::Direct,
    }
}
```

### Batch Processing
```rust
struct PathfindingQueue {
    requests: VecDeque<PathRequest>,
    max_per_frame: usize, // 10
    priority_threshold: f32,
}

fn process_pathfinding_queue(
    queue: &mut PathfindingQueue,
    pathfinder: &mut PathfindingSystem,
    frame_time_budget: Duration,
) {
    let start_time = Instant::now();
    let mut processed = 0;
    
    while let Some(request) = queue.requests.pop_front() {
        if start_time.elapsed() > frame_time_budget {
            // Re-queue for next frame
            queue.requests.push_front(request);
            break;
        }
        
        if processed >= queue.max_per_frame && request.priority < queue.priority_threshold {
            queue.requests.push_back(request);
            break;
        }
        
        if let Some(path) = pathfinder.find_path(request.start, request.goal) {
            request.callback.send(path);
        }
        
        processed += 1;
    }
}
```

## Navigation Mesh Integration

### NavMesh Generation
```rust
struct NavigationMesh {
    polygons: Vec<NavPolygon>,
    connections: HashMap<PolygonId, Vec<PolygonId>>,
    detail_level: f32,
}

fn generate_navmesh(terrain: &TerrainGrid, obstacle_map: &ObstacleMap) -> NavigationMesh {
    // 1. Create walkable area bitmap
    let walkable = create_walkable_bitmap(terrain);
    
    // 2. Simplify to polygons using marching squares
    let contours = extract_contours(&walkable);
    
    // 3. Triangulate and merge into convex polygons
    let polygons = triangulate_and_merge(contours);
    
    // 4. Build connection graph
    let connections = build_polygon_connections(&polygons);
    
    NavigationMesh {
        polygons,
        connections,
        detail_level: 1.0,
    }
}
```

## Terrain Traversal

### Slope Handling
```rust
fn calculate_slope_penalty(
    from_elevation: f32,
    to_elevation: f32,
    distance: f32,
    creature: &Creature,
) -> f32 {
    let slope = (to_elevation - from_elevation).abs() / distance;
    
    match creature.mobility_type {
        MobilityType::Terrestrial => {
            match slope {
                0.0..=0.2 => 1.0,   // Flat
                0.2..=0.4 => 1.5,   // Moderate
                0.4..=0.6 => 3.0,   // Steep
                _ => f32::INFINITY, // Too steep
            }
        },
        MobilityType::Climber => {
            1.0 + slope * 0.5 // Climbers handle slopes better
        },
        MobilityType::Flying => {
            1.0 // No slope penalty for flyers
        },
    }
}
```