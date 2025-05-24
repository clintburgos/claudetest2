# Unified LOD (Level of Detail) System

## Overview

The LOD system dynamically adjusts simulation and rendering detail based on camera distance, creature importance, and performance constraints. It ensures smooth performance with thousands of creatures by intelligently reducing computational overhead for distant or off-screen entities.

## LOD Levels

### Level Definitions

| LOD Level | Distance Range | Visible | Update Rate | Features |
|-----------|---------------|---------|-------------|-----------|
| **LOD0 - Full** | 0-50 units | Yes | Every frame | All systems active, full animation |
| **LOD1 - High** | 50-200 units | Yes | Every 2 frames | Simplified animation, full AI |
| **LOD2 - Medium** | 200-500 units | Yes | Every 4 frames | Basic animation, reduced AI |
| **LOD3 - Low** | 500-1000 units | Yes | Every 8 frames | No animation, simple AI |
| **LOD4 - Minimal** | 1000+ units | Maybe | Every 16 frames | Statistical only |
| **LOD5 - Culled** | Off-screen | No | Every 32 frames | Minimal updates |

### Importance Modifiers

Certain creatures get preferential LOD treatment:

```rust
pub enum ImportanceLevel {
    Critical,    // Never below LOD1 (selected, in conversation)
    High,        // +1 LOD level (group leaders, rare genetics)
    Normal,      // Standard LOD calculation
    Low,         // -1 LOD level (sleeping, stationary)
}

pub struct LODModifiers {
    // Conditions that affect LOD
    is_selected: bool,              // +2 levels
    is_in_conversation: bool,       // +2 levels
    is_group_leader: bool,          // +1 level
    has_rare_traits: bool,          // +1 level
    is_near_selected: bool,         // +1 level
    is_player_following: bool,      // Never below LOD1
    time_since_last_viewed: f32,    // Gradual degradation
}
```

## System-Specific LOD Behavior

### Movement System LOD

```rust
pub struct MovementLOD {
    pub update_rate: UpdateRate,
    pub pathfinding_quality: PathfindingQuality,
    pub collision_detection: CollisionDetail,
}

impl MovementLOD {
    pub fn for_level(level: LODLevel) -> Self {
        match level {
            LODLevel::Full => Self {
                update_rate: UpdateRate::EveryFrame,
                pathfinding_quality: PathfindingQuality::Full,
                collision_detection: CollisionDetail::Precise,
            },
            LODLevel::High => Self {
                update_rate: UpdateRate::EveryNFrames(2),
                pathfinding_quality: PathfindingQuality::Full,
                collision_detection: CollisionDetail::Precise,
            },
            LODLevel::Medium => Self {
                update_rate: UpdateRate::EveryNFrames(4),
                pathfinding_quality: PathfindingQuality::Simplified,
                collision_detection: CollisionDetail::Approximate,
            },
            LODLevel::Low => Self {
                update_rate: UpdateRate::EveryNFrames(8),
                pathfinding_quality: PathfindingQuality::Direct,
                collision_detection: CollisionDetail::None,
            },
            LODLevel::Minimal => Self {
                update_rate: UpdateRate::EveryNFrames(16),
                pathfinding_quality: PathfindingQuality::Teleport,
                collision_detection: CollisionDetail::None,
            },
            LODLevel::Culled => Self {
                update_rate: UpdateRate::EveryNFrames(32),
                pathfinding_quality: PathfindingQuality::Statistical,
                collision_detection: CollisionDetail::None,
            },
        }
    }
}
```

### Decision System LOD

```rust
pub struct DecisionLOD {
    pub think_rate: f32,              // Hz
    pub consideration_count: usize,   // Max options to evaluate
    pub memory_updates: bool,         // Update long-term memory
    pub planning_depth: usize,        // How far ahead to plan
}

impl DecisionLOD {
    pub fn for_level(level: LODLevel) -> Self {
        match level {
            LODLevel::Full => Self {
                think_rate: 10.0,
                consideration_count: 20,
                memory_updates: true,
                planning_depth: 5,
            },
            LODLevel::High => Self {
                think_rate: 5.0,
                consideration_count: 15,
                memory_updates: true,
                planning_depth: 3,
            },
            LODLevel::Medium => Self {
                think_rate: 2.0,
                consideration_count: 10,
                memory_updates: true,
                planning_depth: 2,
            },
            LODLevel::Low => Self {
                think_rate: 0.5,
                consideration_count: 5,
                memory_updates: false,
                planning_depth: 1,
            },
            LODLevel::Minimal => Self {
                think_rate: 0.1,
                consideration_count: 3,
                memory_updates: false,
                planning_depth: 0,
            },
            LODLevel::Culled => Self {
                think_rate: 0.03,
                consideration_count: 1,
                memory_updates: false,
                planning_depth: 0,
            },
        }
    }
}
```

### Animation System LOD

```rust
pub struct AnimationLOD {
    pub skeletal_animation: bool,
    pub blend_transitions: bool,
    pub facial_expressions: bool,
    pub particle_effects: bool,
    pub shadow_casting: bool,
    pub texture_resolution: TextureRes,
}

impl AnimationLOD {
    pub fn for_level(level: LODLevel) -> Self {
        match level {
            LODLevel::Full => Self {
                skeletal_animation: true,
                blend_transitions: true,
                facial_expressions: true,
                particle_effects: true,
                shadow_casting: true,
                texture_resolution: TextureRes::Full,
            },
            LODLevel::High => Self {
                skeletal_animation: true,
                blend_transitions: true,
                facial_expressions: true,
                particle_effects: true,
                shadow_casting: true,
                texture_resolution: TextureRes::Full,
            },
            LODLevel::Medium => Self {
                skeletal_animation: true,
                blend_transitions: false,
                facial_expressions: false,
                particle_effects: true,
                shadow_casting: false,
                texture_resolution: TextureRes::Half,
            },
            LODLevel::Low => Self {
                skeletal_animation: false,
                blend_transitions: false,
                facial_expressions: false,
                particle_effects: false,
                shadow_casting: false,
                texture_resolution: TextureRes::Quarter,
            },
            _ => Self {
                skeletal_animation: false,
                blend_transitions: false,
                facial_expressions: false,
                particle_effects: false,
                shadow_casting: false,
                texture_resolution: TextureRes::Billboard,
            },
        }
    }
}
```

### Social System LOD

```rust
pub struct SocialLOD {
    pub relationship_updates: bool,
    pub group_dynamics: bool,
    pub conversation_detail: ConversationDetail,
    pub memory_formation: bool,
}

impl SocialLOD {
    pub fn for_level(level: LODLevel) -> Self {
        match level {
            LODLevel::Full => Self {
                relationship_updates: true,
                group_dynamics: true,
                conversation_detail: ConversationDetail::Full,
                memory_formation: true,
            },
            LODLevel::High => Self {
                relationship_updates: true,
                group_dynamics: true,
                conversation_detail: ConversationDetail::Full,
                memory_formation: true,
            },
            LODLevel::Medium => Self {
                relationship_updates: true,
                group_dynamics: true,
                conversation_detail: ConversationDetail::Simplified,
                memory_formation: true,
            },
            LODLevel::Low => Self {
                relationship_updates: true,
                group_dynamics: false,
                conversation_detail: ConversationDetail::Statistical,
                memory_formation: false,
            },
            _ => Self {
                relationship_updates: false,
                group_dynamics: false,
                conversation_detail: ConversationDetail::None,
                memory_formation: false,
            },
        }
    }
}
```

## LOD Manager

### Core LOD Manager

```rust
pub struct LODManager {
    // Configuration
    camera_position: Vec3,
    camera_forward: Vec3,
    screen_bounds: Rect,
    performance_mode: PerformanceMode,
    
    // Per-entity LOD data
    entity_lods: HashMap<Entity, LODData>,
    
    // Performance tracking
    frame_time_budget: Duration,
    current_frame_time: Duration,
    lod_distribution: [u32; 6], // Count per LOD level
    
    // Optimization
    update_queue: BinaryHeap<LODUpdate>,
    spatial_index: Arc<RwLock<SpatialIndex>>,
}

pub struct LODData {
    pub level: LODLevel,
    pub importance: ImportanceLevel,
    pub modifiers: LODModifiers,
    pub last_update: Instant,
    pub distance_to_camera: f32,
    pub screen_space_size: f32,
}

pub struct LODUpdate {
    entity: Entity,
    priority: f32,
    next_update: Instant,
}
```

### LOD Calculation

```rust
impl LODManager {
    pub fn calculate_lod(&self, entity: Entity, pos: Vec3) -> LODLevel {
        // Base calculation from distance
        let distance = (pos - self.camera_position).length();
        let base_lod = self.distance_to_lod(distance);
        
        // Check if on screen
        let screen_pos = self.world_to_screen(pos);
        let on_screen = self.screen_bounds.contains(screen_pos);
        
        // Get importance modifiers
        let data = self.entity_lods.get(&entity);
        let importance = data.map(|d| d.importance).unwrap_or(ImportanceLevel::Normal);
        let modifiers = data.map(|d| &d.modifiers).unwrap_or(&LODModifiers::default());
        
        // Calculate final LOD
        let mut lod_value = base_lod as i32;
        
        // Apply importance
        match importance {
            ImportanceLevel::Critical => lod_value = 1.max(lod_value),
            ImportanceLevel::High => lod_value -= 1,
            ImportanceLevel::Low => lod_value += 1,
            _ => {}
        }
        
        // Apply modifiers
        if modifiers.is_selected { lod_value -= 2; }
        if modifiers.is_in_conversation { lod_value -= 2; }
        if modifiers.is_group_leader { lod_value -= 1; }
        if modifiers.has_rare_traits { lod_value -= 1; }
        if modifiers.is_near_selected { lod_value -= 1; }
        
        // Off-screen penalty
        if !on_screen { lod_value += 1; }
        
        // Performance mode adjustments
        match self.performance_mode {
            PerformanceMode::Quality => lod_value -= 1,
            PerformanceMode::Balanced => {},
            PerformanceMode::Performance => lod_value += 1,
        }
        
        // Clamp to valid range
        let final_lod = (lod_value.max(0).min(5)) as u8;
        LODLevel::from_u8(final_lod)
    }
    
    fn distance_to_lod(&self, distance: f32) -> u8 {
        if distance < 50.0 { 0 }
        else if distance < 200.0 { 1 }
        else if distance < 500.0 { 2 }
        else if distance < 1000.0 { 3 }
        else if distance < 2000.0 { 4 }
        else { 5 }
    }
}
```

### Dynamic LOD Adjustment

```rust
impl LODManager {
    pub fn update_lods(&mut self, dt: Duration) {
        self.current_frame_time += dt;
        
        // Check if we're over budget
        if self.current_frame_time > self.frame_time_budget {
            self.decrease_lod_quality();
        } else if self.current_frame_time < self.frame_time_budget * 0.8 {
            self.increase_lod_quality();
        }
        
        // Process LOD update queue
        let now = Instant::now();
        while let Some(update) = self.update_queue.peek() {
            if update.next_update > now { break; }
            
            let update = self.update_queue.pop().unwrap();
            self.update_entity_lod(update.entity);
        }
    }
    
    fn decrease_lod_quality(&mut self) {
        // Find entities to downgrade
        let downgrades: Vec<Entity> = self.entity_lods
            .iter()
            .filter(|(_, data)| {
                data.level != LODLevel::Culled && 
                data.importance != ImportanceLevel::Critical
            })
            .sorted_by_key(|(_, data)| OrderedFloat(data.distance_to_camera))
            .rev()
            .take(100)
            .map(|(e, _)| *e)
            .collect();
            
        for entity in downgrades {
            if let Some(data) = self.entity_lods.get_mut(&entity) {
                data.level = data.level.downgrade();
            }
        }
    }
}
```

## LOD Transitions

### Smooth Transitions

```rust
pub struct LODTransition {
    entity: Entity,
    from_lod: LODLevel,
    to_lod: LODLevel,
    progress: f32,
    duration: f32,
}

impl LODTransition {
    pub fn update(&mut self, dt: f32) -> bool {
        self.progress += dt / self.duration;
        self.progress >= 1.0
    }
    
    pub fn get_blend_factor(&self) -> f32 {
        // Smooth step function
        let t = self.progress.clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }
}
```

### Hysteresis

Prevent LOD flickering:

```rust
pub struct LODHysteresis {
    upgrade_threshold: f32,
    downgrade_threshold: f32,
    time_threshold: Duration,
}

impl LODHysteresis {
    pub fn should_change_lod(&self, 
                           current: LODLevel, 
                           target: LODLevel,
                           time_in_current: Duration) -> bool {
        if time_in_current < self.time_threshold {
            return false; // Too soon to change
        }
        
        match current.cmp(&target) {
            Ordering::Less => true,  // Always allow upgrades
            Ordering::Greater => {
                // Only downgrade if significantly over threshold
                let current_dist = current.max_distance();
                let target_dist = target.max_distance();
                (target_dist - current_dist) > self.downgrade_threshold
            },
            Ordering::Equal => false,
        }
    }
}
```

## Performance Monitoring

### LOD Statistics

```rust
pub struct LODStats {
    pub distribution: [u32; 6],          // Creatures per LOD level
    pub average_lod: f32,                // Average LOD across all creatures
    pub transition_count: u32,           // LOD changes this frame
    pub performance_adjustments: i32,    // Auto-adjustments made
}

impl LODManager {
    pub fn get_stats(&self) -> LODStats {
        let mut stats = LODStats::default();
        
        for (_, data) in &self.entity_lods {
            stats.distribution[data.level as usize] += 1;
        }
        
        let total = stats.distribution.iter().sum::<u32>() as f32;
        stats.average_lod = stats.distribution
            .iter()
            .enumerate()
            .map(|(i, &count)| i as f32 * count as f32)
            .sum::<f32>() / total;
            
        stats
    }
}
```

### Debug Visualization

```rust
pub struct LODDebugRenderer {
    pub show_lod_levels: bool,
    pub show_transitions: bool,
    pub show_importance: bool,
    pub color_by_lod: bool,
}

impl LODDebugRenderer {
    pub fn render(&self, entity: Entity, lod_data: &LODData) {
        if self.show_lod_levels {
            // Draw LOD level text above creature
            self.draw_text_billboard(
                entity,
                &format!("LOD{}", lod_data.level as u8),
                self.lod_to_color(lod_data.level),
            );
        }
        
        if self.color_by_lod {
            // Tint creature by LOD level
            self.set_entity_tint(entity, self.lod_to_color(lod_data.level));
        }
    }
    
    fn lod_to_color(&self, lod: LODLevel) -> Color {
        match lod {
            LODLevel::Full => Color::GREEN,
            LODLevel::High => Color::YELLOW_GREEN,
            LODLevel::Medium => Color::YELLOW,
            LODLevel::Low => Color::ORANGE,
            LODLevel::Minimal => Color::RED,
            LODLevel::Culled => Color::DARK_RED,
        }
    }
}
```

## Integration Example

```rust
// In the main game loop
pub fn update_creature_systems(world: &mut World, dt: Duration) {
    let lod_manager = world.get_resource::<LODManager>().unwrap();
    
    // Update each system with appropriate LOD
    for (entity, transform, lod_data) in query_creatures(world) {
        let movement_lod = MovementLOD::for_level(lod_data.level);
        let decision_lod = DecisionLOD::for_level(lod_data.level);
        let animation_lod = AnimationLOD::for_level(lod_data.level);
        
        // Update systems based on LOD settings
        if should_update(movement_lod.update_rate, entity) {
            update_movement(entity, &movement_lod);
        }
        
        if should_think(decision_lod.think_rate, entity, dt) {
            update_decisions(entity, &decision_lod);
        }
        
        if lod_data.level <= LODLevel::Low {
            update_animation(entity, &animation_lod);
        }
    }
}
```

This unified LOD system ensures consistent performance scaling across all game systems while maintaining visual quality where it matters most.