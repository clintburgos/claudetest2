# Critical Missing Systems

## 1. Determinism System

### Overview
Ensures simulation reproducibility for debugging and fairness. Critical for bug reports and testing.

```rust
pub struct DeterminismManager {
    seed: u64,
    frame_count: u64,
    rng_states: HashMap<SystemId, XorShiftRng>,
    checksum_history: RingBuffer<FrameChecksum>,
}

pub struct FrameChecksum {
    frame: u64,
    creature_checksum: u64,
    resource_checksum: u64,
    event_checksum: u64,
}

impl DeterminismManager {
    pub fn get_rng(&mut self, system: SystemId) -> &mut XorShiftRng {
        self.rng_states.entry(system)
            .or_insert_with(|| XorShiftRng::seed_from_u64(self.seed ^ system as u64))
    }
    
    pub fn verify_determinism(&self, other: &FrameChecksum) -> Result<(), DeterminismError> {
        if self.checksum_history.back() != Some(other) {
            return Err(DeterminismError::Desync {
                frame: self.frame_count,
                expected: *other,
                actual: *self.checksum_history.back().unwrap(),
            });
        }
        Ok(())
    }
}

// Enforce deterministic ordering
pub trait DeterministicSystem {
    fn update_deterministic(&mut self, world: &mut World, rng: &mut XorShiftRng);
    fn calculate_checksum(&self, world: &World) -> u64;
}
```

## 2. Error Recovery System

### Overview
Handles invalid states gracefully without crashing the simulation.

```rust
pub struct ErrorBoundary {
    recovery_strategies: HashMap<ErrorType, Box<dyn RecoveryStrategy>>,
    error_log: RingBuffer<ErrorEvent>,
    corruption_detector: CorruptionDetector,
}

pub enum ErrorType {
    CreatureStuck,
    InvalidPosition,
    ResourceNegative,
    PathfindingFailed,
    RelationshipCorrupted,
    MemoryExhausted,
}

pub trait RecoveryStrategy: Send + Sync {
    fn can_recover(&self, error: &SimulationError) -> bool;
    fn recover(&self, world: &mut World, error: &SimulationError) -> Result<(), FatalError>;
}

// Example recovery strategies
pub struct CreatureStuckRecovery;

impl RecoveryStrategy for CreatureStuckRecovery {
    fn recover(&self, world: &mut World, error: &SimulationError) -> Result<(), FatalError> {
        if let SimulationError::CreatureStuck { entity, position, duration } = error {
            // Try increasing search radius
            if *duration < 10.0 {
                world.get_mut::<PathfindingRequest>(*entity)
                    .map(|mut req| req.search_radius *= 1.5);
                return Ok(());
            }
            
            // Teleport to nearest valid position
            if *duration < 30.0 {
                let valid_pos = find_nearest_valid_position(*position, world);
                world.get_mut::<Position>(*entity)
                    .map(|mut pos| pos.0 = valid_pos);
                return Ok(());
            }
            
            // Last resort: respawn at home
            respawn_creature_at_home(*entity, world);
            Ok(())
        } else {
            Err(FatalError::UnexpectedError)
        }
    }
}

pub struct CorruptionDetector {
    invariants: Vec<Box<dyn Invariant>>,
}

pub trait Invariant: Send + Sync {
    fn check(&self, world: &World) -> Result<(), InvariantViolation>;
    fn repair(&self, world: &mut World, violation: &InvariantViolation) -> Result<(), FatalError>;
}
```

## 3. Built-in Profiling Infrastructure

### Overview
Performance monitoring that's always available, not just in debug builds.

```rust
pub struct ProfilerSystem {
    frame_profiler: FrameProfiler,
    system_profilers: HashMap<SystemId, SystemProfiler>,
    memory_profiler: MemoryProfiler,
    spike_detector: SpikeDetector,
}

pub struct FrameProfiler {
    frame_times: RingBuffer<FrameTime>,
    budget: Duration,
    warnings: Vec<PerformanceWarning>,
}

pub struct SystemProfiler {
    name: &'static str,
    timer: Instant,
    accumulated_time: Duration,
    call_count: u64,
    memory_allocated: usize,
    cache_stats: CacheStatistics,
}

pub struct SpikeDetector {
    baseline: Duration,
    spike_threshold: f32, // 1.5x baseline
    spike_callback: Box<dyn Fn(SpikeInfo)>,
}

// Easy-to-use macros
#[macro_export]
macro_rules! profile_scope {
    ($profiler:expr, $name:expr) => {
        let _guard = $profiler.scope($name);
    };
}

impl ProfilerSystem {
    pub fn begin_frame(&mut self) {
        self.frame_profiler.begin();
    }
    
    pub fn end_frame(&mut self) {
        let frame_time = self.frame_profiler.end();
        
        if frame_time > self.frame_profiler.budget {
            self.analyze_frame_spike(frame_time);
        }
        
        // Auto-report if consistently over budget
        if self.is_performance_degraded() {
            self.generate_performance_report();
        }
    }
    
    pub fn get_metrics(&self) -> PerformanceMetrics {
        PerformanceMetrics {
            avg_frame_time: self.frame_profiler.average(),
            worst_frame: self.frame_profiler.worst(),
            system_breakdown: self.get_system_breakdown(),
            memory_usage: self.memory_profiler.current_usage(),
            cache_efficiency: self.calculate_cache_efficiency(),
        }
    }
}
```

## 4. Debug Tooling

### Overview
Essential tools for understanding what's happening with 5000 creatures.

```rust
pub struct DebugSystem {
    creature_inspector: CreatureInspector,
    time_machine: TimeMachine,
    query_builder: DebugQueryBuilder,
    visualization: DebugVisualization,
}

pub struct CreatureInspector {
    watched_creatures: HashSet<Entity>,
    breakpoints: Vec<CreatureBreakpoint>,
    history_buffer: HashMap<Entity, CreatureHistory>,
}

pub struct CreatureBreakpoint {
    condition: BreakpointCondition,
    action: BreakpointAction,
    one_shot: bool,
}

pub enum BreakpointCondition {
    StateChange { from: Option<State>, to: Option<State> },
    HealthBelow(f32),
    StuckFor(Duration),
    Custom(Box<dyn Fn(&Creature) -> bool>),
}

pub struct TimeMachine {
    snapshots: RingBuffer<WorldSnapshot>,
    recording: bool,
    playback_speed: f32,
}

impl TimeMachine {
    pub fn record_frame(&mut self, world: &World) {
        if self.recording {
            self.snapshots.push(WorldSnapshot::capture(world));
        }
    }
    
    pub fn rewind(&mut self, frames: usize) -> Option<&WorldSnapshot> {
        self.snapshots.get(self.snapshots.len().saturating_sub(frames))
    }
    
    pub fn replay(&self, from: usize, to: usize) -> ReplayIterator {
        ReplayIterator::new(&self.snapshots, from, to)
    }
}

pub struct DebugVisualization {
    overlays: HashMap<OverlayType, bool>,
    custom_renders: Vec<Box<dyn DebugRender>>,
}

pub enum OverlayType {
    CreatureIds,
    HealthBars,
    Needs,
    Pathfinding,
    SocialConnections,
    GroupBoundaries,
    ResourceAvailability,
    ErrorLocations,
}
```

## 5. Performance Graceful Degradation

### Overview
Automatically reduces quality to maintain target framerate.

```rust
pub struct GracefulDegradation {
    target_frame_time: Duration,
    current_quality: QualityLevel,
    quality_controller: QualityController,
    metrics_window: RingBuffer<FrameMetrics>,
}

pub struct QualityController {
    levels: Vec<QualityLevel>,
    transition_thresholds: TransitionThresholds,
    minimum_quality: QualityLevel,
}

pub struct QualityLevel {
    name: &'static str,
    creature_limit: Option<usize>,
    update_frequencies: HashMap<SystemId, u32>,
    render_settings: RenderQuality,
    ai_complexity: AIComplexity,
}

impl GracefulDegradation {
    pub fn adjust_quality(&mut self, frame_time: Duration) {
        let avg_frame_time = self.metrics_window.average();
        
        if avg_frame_time > self.target_frame_time * 1.2 {
            // Degrade quality
            if let Some(lower) = self.quality_controller.get_lower_quality(self.current_quality) {
                log::warn!("Degrading quality to maintain performance: {} -> {}", 
                    self.current_quality.name, lower.name);
                self.apply_quality_level(lower);
            }
        } else if avg_frame_time < self.target_frame_time * 0.8 {
            // Try improving quality
            if let Some(higher) = self.quality_controller.get_higher_quality(self.current_quality) {
                self.test_quality_improvement(higher);
            }
        }
    }
    
    fn apply_quality_level(&mut self, level: QualityLevel) {
        // Reduce active creatures if needed
        if let Some(limit) = level.creature_limit {
            self.limit_active_creatures(limit);
        }
        
        // Adjust update frequencies
        for (system, frequency) in level.update_frequencies {
            self.set_system_update_frequency(system, frequency);
        }
        
        // Apply render settings
        self.apply_render_quality(level.render_settings);
        
        self.current_quality = level;
    }
}
```

## 6. Observation Goals System

### Overview
Even pure simulations need to give observers something to look for.

```rust
pub struct ObservationGoals {
    active_goals: Vec<ObservationGoal>,
    completed_goals: Vec<CompletedGoal>,
    discovery_tracker: DiscoveryTracker,
    statistics_tracker: StatisticsTracker,
}

pub struct ObservationGoal {
    id: GoalId,
    name: String,
    description: String,
    discovery_hints: Vec<String>,
    completion_criteria: CompletionCriteria,
    reward: GoalReward,
}

pub enum CompletionCriteria {
    ObserveEvent { event_type: EventType, count: u32 },
    TrackCreature { condition: CreatureCondition, duration: Duration },
    WitnessEmergence { phenomenon: EmergentPhenomenon },
    ReachStatistic { stat: StatisticType, value: f64 },
}

pub enum GoalReward {
    UnlockDataView(DataViewType),
    RevealInsight(InsightType),
    AchievementBadge(AchievementId),
    UnlockAdvancedTool(ToolType),
}

pub struct DiscoveryTracker {
    discoveries: HashMap<DiscoveryType, Discovery>,
    hints: HintSystem,
}

pub enum DiscoveryType {
    // Behavioral discoveries
    FirstGroupFormation,
    LeadershipEmergence,
    TerritorialBehavior,
    CooperativeHunting,
    ToolUse,
    
    // Social discoveries
    FriendshipBond,
    FamilyUnit,
    GroupMigration,
    ConflictResolution,
    
    // Evolutionary discoveries  
    TraitDivergence,
    SpeciesFormation,
    AdaptiveBehavior,
    ExtinctionEvent,
}

impl ObservationGoals {
    pub fn check_discoveries(&mut self, world: &World, events: &[Event]) {
        for event in events {
            if let Some(discovery) = self.analyze_for_discovery(event, world) {
                self.record_discovery(discovery);
            }
        }
        
        // Check for statistical discoveries
        if let Some(stat_discovery) = self.check_statistical_discoveries(world) {
            self.record_discovery(stat_discovery);
        }
    }
    
    pub fn get_active_hints(&self) -> Vec<DiscoveryHint> {
        self.discovery_tracker.hints.get_contextual_hints()
    }
}
```

Now, let me create the Phase 1 detailed design...