# UI State Management Design

## Overview

The UI state management system handles all interface state, data updates, panel persistence, and performance optimization for the user interface. This ensures a responsive, consistent experience across different views and zoom levels.

## State Architecture

### Core State Structure
```rust
struct UIState {
    // View state
    camera: CameraState,
    selected_creatures: HashSet<CreatureId>,
    followed_creature: Option<CreatureId>,
    
    // Panel states
    panels: HashMap<PanelId, PanelState>,
    layout: UILayout,
    
    // Data caches
    statistics_cache: StatisticsCache,
    graph_data_cache: GraphDataCache,
    
    // Update scheduling
    update_scheduler: UpdateScheduler,
    
    // User preferences
    preferences: UIPreferences,
}

#[derive(Clone, Serialize, Deserialize)]
struct PanelState {
    id: PanelId,
    visible: bool,
    position: Vec2,
    size: Vec2,
    collapsed: bool,
    pinned: bool,
    opacity: f32,
    
    // Panel-specific state
    content_state: PanelContentState,
    
    // Update frequency
    update_rate: UpdateRate,
    last_update: Instant,
}

#[derive(Clone, Serialize, Deserialize)]
enum PanelContentState {
    Overview {
        stat_visibility: HashMap<StatType, bool>,
        sort_order: SortOrder,
    },
    Population {
        selected_species: Option<SpeciesId>,
        graph_type: PopulationGraphType,
        time_window: TimeWindow,
    },
    Genetics {
        trait_filter: Vec<TraitType>,
        comparison_mode: bool,
        selected_lineages: Vec<LineageId>,
    },
    Trends {
        tracked_metrics: Vec<MetricType>,
        prediction_enabled: bool,
        smoothing_level: f32,
    },
    CreatureDetail {
        creature_id: CreatureId,
        tab_selection: DetailTab,
        memory_filter: MemoryFilter,
    },
}
```

### Camera State Management
```rust
struct CameraState {
    position: Vec2,
    zoom_level: f32,
    target_position: Option<Vec2>,
    target_zoom: Option<f32>,
    
    // Smooth transitions
    transition_speed: f32,
    zoom_speed: f32,
    
    // Constraints
    bounds: Rectangle,
    min_zoom: f32,
    max_zoom: f32,
    
    // Follow mode
    follow_mode: FollowMode,
    follow_offset: Vec2,
}

enum FollowMode {
    None,
    Creature { id: CreatureId, smooth: bool },
    Group { id: GroupId, auto_zoom: bool },
    Region { center: Vec2, radius: f32 },
}

impl CameraState {
    fn update(&mut self, delta_time: f32) {
        // Smooth position transitions
        if let Some(target) = self.target_position {
            let diff = target - self.position;
            let move_speed = self.transition_speed * delta_time;
            
            if diff.length() > move_speed {
                self.position += diff.normalize() * move_speed;
            } else {
                self.position = target;
                self.target_position = None;
            }
        }
        
        // Smooth zoom transitions
        if let Some(target_zoom) = self.target_zoom {
            let zoom_diff = target_zoom - self.zoom_level;
            let zoom_change = self.zoom_speed * delta_time;
            
            if zoom_diff.abs() > zoom_change {
                self.zoom_level += zoom_diff.signum() * zoom_change;
            } else {
                self.zoom_level = target_zoom;
                self.target_zoom = None;
            }
        }
        
        // Update follow mode
        match &self.follow_mode {
            FollowMode::Creature { id, smooth } => {
                if let Some(creature) = get_creature(*id) {
                    let target = creature.position + self.follow_offset;
                    
                    if *smooth {
                        self.target_position = Some(target);
                    } else {
                        self.position = target;
                    }
                }
            },
            FollowMode::Group { id, auto_zoom } => {
                if let Some(group) = get_group(*id) {
                    let (center, radius) = group.get_bounding_circle();
                    self.target_position = Some(center);
                    
                    if *auto_zoom {
                        let desired_zoom = self.calculate_zoom_for_radius(radius);
                        self.target_zoom = Some(desired_zoom);
                    }
                }
            },
            _ => {},
        }
        
        // Apply constraints
        self.apply_bounds();
    }
}
```

## Update Scheduling

### Update Rate Management
```rust
#[derive(Clone, Copy)]
enum UpdateRate {
    Realtime,         // Every frame
    High,            // 10 Hz
    Medium,          // 2 Hz
    Low,             // 0.5 Hz
    OnDemand,        // Only when data changes
}

struct UpdateScheduler {
    scheduled_updates: BinaryHeap<ScheduledUpdate>,
    update_budgets: HashMap<UpdatePriority, Duration>,
    frame_time_target: Duration,
}

struct ScheduledUpdate {
    panel_id: PanelId,
    priority: UpdatePriority,
    next_update: Instant,
    update_fn: Box<dyn Fn(&mut PanelState, &World) + Send>,
}

impl UpdateScheduler {
    fn schedule_updates(&mut self, current_time: Instant, panels: &HashMap<PanelId, PanelState>) {
        for (id, panel) in panels {
            if !panel.visible && !panel.pinned {
                continue; // Skip hidden panels
            }
            
            let next_update = panel.last_update + self.get_update_interval(panel.update_rate);
            
            if next_update <= current_time {
                self.scheduled_updates.push(ScheduledUpdate {
                    panel_id: *id,
                    priority: self.calculate_priority(panel),
                    next_update,
                    update_fn: self.create_update_fn(*id),
                });
            }
        }
    }
    
    fn process_updates(
        &mut self,
        panels: &mut HashMap<PanelId, PanelState>,
        world: &World,
        time_budget: Duration,
    ) {
        let start_time = Instant::now();
        
        while let Some(update) = self.scheduled_updates.pop() {
            if start_time.elapsed() > time_budget {
                // Re-queue remaining updates
                self.scheduled_updates.push(update);
                break;
            }
            
            if let Some(panel) = panels.get_mut(&update.panel_id) {
                (update.update_fn)(panel, world);
                panel.last_update = Instant::now();
            }
        }
    }
    
    fn calculate_priority(&self, panel: &PanelState) -> UpdatePriority {
        // Visible panels have higher priority
        let visibility_score = if panel.visible { 1.0 } else { 0.3 };
        
        // Frequently accessed panels have higher priority
        let access_score = panel.get_access_frequency();
        
        // Critical data panels have higher priority
        let criticality_score = match panel.id {
            PanelId::Overview => 0.9,
            PanelId::Alerts => 1.0,
            _ => 0.5,
        };
        
        UpdatePriority::from_score(
            visibility_score * 0.4 + 
            access_score * 0.3 + 
            criticality_score * 0.3
        )
    }
}
```

## Data Caching

### Statistics Cache
```rust
struct StatisticsCache {
    global_stats: CachedValue<GlobalStatistics>,
    species_stats: HashMap<SpeciesId, CachedValue<SpeciesStatistics>>,
    biome_stats: HashMap<BiomeType, CachedValue<BiomeStatistics>>,
    
    // Incremental updates
    update_queue: VecDeque<StatUpdate>,
    batch_size: usize,
}

struct CachedValue<T> {
    value: T,
    timestamp: Instant,
    validity_duration: Duration,
    dirty: bool,
}

impl<T> CachedValue<T> {
    fn is_valid(&self) -> bool {
        !self.dirty && self.timestamp.elapsed() < self.validity_duration
    }
    
    fn invalidate(&mut self) {
        self.dirty = true;
    }
    
    fn update(&mut self, new_value: T) {
        self.value = new_value;
        self.timestamp = Instant::now();
        self.dirty = false;
    }
}

impl StatisticsCache {
    fn get_global_stats(&mut self, world: &World) -> &GlobalStatistics {
        if !self.global_stats.is_valid() {
            let stats = self.calculate_global_stats(world);
            self.global_stats.update(stats);
        }
        
        &self.global_stats.value
    }
    
    fn process_incremental_updates(&mut self, world: &World) {
        let batch_end = self.update_queue.len().min(self.batch_size);
        
        for _ in 0..batch_end {
            if let Some(update) = self.update_queue.pop_front() {
                match update {
                    StatUpdate::CreatureBorn { species } => {
                        self.global_stats.value.total_births += 1;
                        self.species_stats.get_mut(&species)
                            .map(|s| s.value.population += 1);
                    },
                    StatUpdate::CreatureDied { species, age } => {
                        self.global_stats.value.total_deaths += 1;
                        self.species_stats.get_mut(&species)
                            .map(|s| {
                                s.value.population -= 1;
                                s.value.avg_lifespan = 
                                    (s.value.avg_lifespan * s.value.total_lived as f32 + age) /
                                    (s.value.total_lived + 1) as f32;
                                s.value.total_lived += 1;
                            });
                    },
                    // ... other update types
                }
            }
        }
    }
}
```

### Graph Data Cache
```rust
struct GraphDataCache {
    time_series_data: HashMap<MetricType, TimeSeriesData>,
    histogram_data: HashMap<MetricType, HistogramData>,
    correlation_data: HashMap<(MetricType, MetricType), CorrelationData>,
    
    // Ring buffers for efficient updates
    data_buffers: HashMap<MetricType, RingBuffer<f32>>,
    buffer_size: usize,
    
    // Downsampling for performance
    downsample_levels: Vec<DownsampleLevel>,
}

struct TimeSeriesData {
    timestamps: Vec<SimTime>,
    values: Vec<f32>,
    
    // Pre-calculated for rendering
    min_value: f32,
    max_value: f32,
    average: f32,
    trend: TrendDirection,
    
    // Smoothed versions
    smoothed_values: Option<Vec<f32>>,
    smoothing_window: usize,
}

struct DownsampleLevel {
    factor: usize, // e.g., 10 = keep every 10th point
    time_window: Duration,
    data: HashMap<MetricType, Vec<f32>>,
}

impl GraphDataCache {
    fn add_data_point(&mut self, metric: MetricType, value: f32, timestamp: SimTime) {
        // Add to ring buffer
        self.data_buffers.entry(metric)
            .or_insert_with(|| RingBuffer::new(self.buffer_size))
            .push(value);
        
        // Update time series
        if let Some(series) = self.time_series_data.get_mut(&metric) {
            series.timestamps.push(timestamp);
            series.values.push(value);
            
            // Update statistics
            series.min_value = series.min_value.min(value);
            series.max_value = series.max_value.max(value);
            
            // Maintain window size
            if series.values.len() > MAX_SERIES_LENGTH {
                series.timestamps.remove(0);
                series.values.remove(0);
            }
            
            // Update smoothed values if needed
            if series.smoothed_values.is_some() {
                self.update_smoothed_values(series);
            }
        }
        
        // Update downsampled versions
        self.update_downsampled_data(metric, value, timestamp);
    }
    
    fn get_visible_data(
        &self,
        metric: MetricType,
        time_window: &TimeWindow,
        pixels_available: usize,
    ) -> &[f32] {
        // Choose appropriate downsample level based on pixels
        let points_needed = self.estimate_points_in_window(metric, time_window);
        
        if points_needed <= pixels_available * 2 {
            // Use full resolution
            &self.time_series_data[&metric].values
        } else {
            // Use downsampled data
            let level = self.select_downsample_level(points_needed, pixels_available);
            &self.downsample_levels[level].data[&metric]
        }
    }
}
```

## Panel Persistence

### Layout Persistence
```rust
#[derive(Serialize, Deserialize)]
struct UILayout {
    version: u32,
    panels: HashMap<PanelId, PanelLayout>,
    workspace_name: String,
    
    // Responsive breakpoints
    breakpoints: Vec<LayoutBreakpoint>,
    current_breakpoint: usize,
}

#[derive(Serialize, Deserialize)]
struct PanelLayout {
    anchor: AnchorPoint,
    offset: Vec2,
    size: Vec2,
    
    // Responsive behavior
    min_size: Vec2,
    max_size: Option<Vec2>,
    priority: u32, // For auto-hiding
    
    // Docking
    docked_to: Option<PanelId>,
    dock_side: Option<DockSide>,
}

impl UILayout {
    fn save_to_disk(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
    
    fn load_from_disk(path: &Path) -> Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let mut layout: UILayout = serde_json::from_str(&json)?;
        
        // Migrate if needed
        layout.migrate_to_current_version();
        
        Ok(layout)
    }
    
    fn apply_responsive_layout(&mut self, screen_size: Vec2) {
        let new_breakpoint = self.find_breakpoint(screen_size);
        
        if new_breakpoint != self.current_breakpoint {
            self.current_breakpoint = new_breakpoint;
            
            // Adjust panel sizes and positions
            for (panel_id, layout) in &mut self.panels {
                self.adjust_panel_for_breakpoint(panel_id, layout, new_breakpoint);
            }
        }
    }
}
```

### State Restoration
```rust
struct UIStateRestoration {
    autosave_interval: Duration,
    last_save: Instant,
    state_history: VecDeque<UIStateSnapshot>,
    max_history: usize,
}

#[derive(Clone, Serialize, Deserialize)]
struct UIStateSnapshot {
    timestamp: DateTime<Utc>,
    camera_state: CameraState,
    panel_states: HashMap<PanelId, PanelContentState>,
    selections: HashSet<CreatureId>,
    
    // Don't save cached data, only settings
    preferences: UIPreferences,
}

impl UIStateRestoration {
    fn create_snapshot(&self, ui_state: &UIState) -> UIStateSnapshot {
        UIStateSnapshot {
            timestamp: Utc::now(),
            camera_state: ui_state.camera.clone(),
            panel_states: ui_state.panels.iter()
                .map(|(id, panel)| (*id, panel.content_state.clone()))
                .collect(),
            selections: ui_state.selected_creatures.clone(),
            preferences: ui_state.preferences.clone(),
        }
    }
    
    fn restore_snapshot(&self, ui_state: &mut UIState, snapshot: UIStateSnapshot) {
        ui_state.camera = snapshot.camera_state;
        ui_state.selected_creatures = snapshot.selections;
        ui_state.preferences = snapshot.preferences;
        
        // Restore panel content states
        for (panel_id, content_state) in snapshot.panel_states {
            if let Some(panel) = ui_state.panels.get_mut(&panel_id) {
                panel.content_state = content_state;
            }
        }
        
        // Mark caches as dirty
        ui_state.invalidate_all_caches();
    }
}
```

## Performance Optimization

### Update Batching
```rust
struct UpdateBatcher {
    pending_updates: HashMap<ComponentType, Vec<UpdateCommand>>,
    batch_thresholds: HashMap<ComponentType, usize>,
    flush_interval: Duration,
    last_flush: Instant,
}

enum UpdateCommand {
    StatIncrement { stat: StatType, delta: i32 },
    DataPoint { metric: MetricType, value: f32 },
    SelectionChange { added: Vec<CreatureId>, removed: Vec<CreatureId> },
    PanelRefresh { panel_id: PanelId },
}

impl UpdateBatcher {
    fn queue_update(&mut self, component: ComponentType, command: UpdateCommand) {
        self.pending_updates
            .entry(component)
            .or_insert_with(Vec::new)
            .push(command);
        
        // Check if we should flush this component
        if let Some(threshold) = self.batch_thresholds.get(&component) {
            if self.pending_updates[&component].len() >= *threshold {
                self.flush_component(component);
            }
        }
    }
    
    fn flush_if_needed(&mut self) {
        if self.last_flush.elapsed() >= self.flush_interval {
            self.flush_all();
        }
    }
    
    fn flush_component(&mut self, component: ComponentType) {
        if let Some(updates) = self.pending_updates.remove(&component) {
            match component {
                ComponentType::Statistics => {
                    self.apply_stat_updates(updates);
                },
                ComponentType::GraphData => {
                    self.apply_graph_updates(updates);
                },
                // ... other components
            }
        }
    }
}
```

### Render Optimization
```rust
struct UIRenderOptimizer {
    dirty_regions: Vec<Rectangle>,
    static_cache: HashMap<PanelId, CachedRender>,
    frame_time_budget: Duration,
    quality_settings: RenderQuality,
}

struct CachedRender {
    texture: Texture,
    last_rendered: Instant,
    dependencies: HashSet<DataDependency>,
}

impl UIRenderOptimizer {
    fn mark_dirty(&mut self, region: Rectangle) {
        // Merge overlapping regions
        let mut merged = false;
        for existing in &mut self.dirty_regions {
            if existing.intersects(&region) {
                *existing = existing.union(&region);
                merged = true;
                break;
            }
        }
        
        if !merged {
            self.dirty_regions.push(region);
        }
    }
    
    fn render_frame(&mut self, ui_state: &UIState, renderer: &mut Renderer) {
        let frame_start = Instant::now();
        
        // Sort panels by z-order
        let mut panels: Vec<_> = ui_state.panels.values().collect();
        panels.sort_by_key(|p| p.z_order);
        
        for panel in panels {
            if !panel.visible {
                continue;
            }
            
            // Check if panel needs redraw
            if self.should_redraw_panel(panel) {
                self.render_panel(panel, ui_state, renderer);
                
                // Check frame budget
                if frame_start.elapsed() > self.frame_time_budget {
                    // Defer remaining panels
                    self.mark_dirty(panel.get_bounds());
                    break;
                }
            } else if let Some(cached) = self.static_cache.get(&panel.id) {
                // Use cached render
                renderer.draw_texture(&cached.texture, panel.position);
            }
        }
        
        self.dirty_regions.clear();
    }
}
```

## Data Update Frequencies

### Update Rate Configuration
```rust
const UPDATE_RATES: &[(DataType, UpdateRate, Duration)] = &[
    // Critical - realtime
    (DataType::CreaturePosition, UpdateRate::Realtime, Duration::ZERO),
    (DataType::Selection, UpdateRate::Realtime, Duration::ZERO),
    
    // High frequency - 10Hz
    (DataType::CreatureStats, UpdateRate::High, Duration::from_millis(100)),
    (DataType::ResourceLevels, UpdateRate::High, Duration::from_millis(100)),
    
    // Medium frequency - 2Hz
    (DataType::PopulationStats, UpdateRate::Medium, Duration::from_millis(500)),
    (DataType::BiomeConditions, UpdateRate::Medium, Duration::from_millis(500)),
    
    // Low frequency - 0.5Hz
    (DataType::GlobalStatistics, UpdateRate::Low, Duration::from_secs(2)),
    (DataType::EvolutionaryTrends, UpdateRate::Low, Duration::from_secs(2)),
    
    // On demand
    (DataType::MemoryDetails, UpdateRate::OnDemand, Duration::MAX),
    (DataType::GeneticAnalysis, UpdateRate::OnDemand, Duration::MAX),
];

struct DataUpdateManager {
    update_timers: HashMap<DataType, Instant>,
    update_callbacks: HashMap<DataType, Box<dyn Fn(&World) -> bool>>,
    
    // Adaptive rates based on activity
    activity_monitor: ActivityMonitor,
    rate_multipliers: HashMap<DataType, f32>,
}

impl DataUpdateManager {
    fn should_update(&self, data_type: DataType) -> bool {
        let base_rate = self.get_base_rate(data_type);
        let multiplier = self.rate_multipliers.get(&data_type).unwrap_or(&1.0);
        let adjusted_interval = base_rate.mul_f32(1.0 / multiplier);
        
        self.update_timers.get(&data_type)
            .map(|last| last.elapsed() >= adjusted_interval)
            .unwrap_or(true)
    }
    
    fn adapt_update_rates(&mut self, ui_state: &UIState) {
        // Increase rates for visible data
        for (panel_id, panel) in &ui_state.panels {
            if panel.visible {
                for data_type in panel.get_data_dependencies() {
                    self.rate_multipliers.insert(data_type, 1.5);
                }
            }
        }
        
        // Decrease rates for hidden data
        for data_type in DataType::all() {
            if !self.is_data_visible(data_type, ui_state) {
                self.rate_multipliers.insert(data_type, 0.5);
            }
        }
        
        // Boost rates during high activity
        if self.activity_monitor.get_activity_level() > 0.8 {
            for multiplier in self.rate_multipliers.values_mut() {
                *multiplier *= 1.2;
            }
        }
    }
}
```