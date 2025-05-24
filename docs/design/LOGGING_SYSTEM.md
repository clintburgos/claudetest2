# Logging & Debugging Strategy

## Overview
This document outlines the comprehensive logging system for debugging and analyzing the creature simulation, from individual decisions to population-wide emergent behaviors.

## Logging Architecture

### Core Framework
Using the `tracing` ecosystem for structured, contextual logging:

```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json", "fmt", "ansi"] }
tracing-appender = "0.2"
tracing-timing = "0.6"  # For performance metrics
tracing-tracy = "0.10"  # For profiling integration

# Serialization for structured logs
serde_json = "1.0"
```

### Log Levels & Categories

```rust
// Log levels by system
pub enum LogCategory {
    CreatureDecision,    // Individual AI decisions
    CreatureBiology,     // Needs, health, energy
    Genetics,           // Reproduction, mutations
    Social,             // Conversations, relationships
    WorldEvents,        // Resources, environment
    Performance,        // Frame times, system costs
    Population,         // Demographics, statistics
    Emergent,          // Detected patterns
}

// Structured logging macros
macro_rules! creature_log {
    ($level:expr, $creature_id:expr, $($arg:tt)*) => {
        tracing::event!(
            target: "creature",
            $level,
            creature.id = $creature_id,
            $($arg)*
        )
    };
}
```

## Logging Systems

### 1. Creature Decision Logging

Track every decision a creature makes with full context:

```rust
#[derive(Debug, Serialize)]
pub struct DecisionLog {
    creature_id: CreatureId,
    timestamp: f64,
    location: Position,
    needs: NeedsSnapshot,
    available_actions: Vec<ActionOption>,
    chosen_action: Action,
    utility_scores: HashMap<Action, f32>,
    influence_factors: Vec<InfluenceFactor>,
}

impl Creature {
    fn make_decision(&mut self, world: &World) -> Action {
        let span = info_span!(
            "creature_decision",
            creature.id = %self.id,
            creature.age = %self.age,
            creature.generation = %self.generation
        );
        let _enter = span.enter();
        
        // Calculate utilities
        let options = self.evaluate_actions(world);
        
        // Log decision process
        if log_enabled!(Level::DEBUG) {
            let decision_log = DecisionLog {
                creature_id: self.id,
                timestamp: world.time(),
                location: self.position,
                needs: self.needs.snapshot(),
                available_actions: options.clone(),
                chosen_action: best_action,
                utility_scores: utilities.clone(),
                influence_factors: self.recent_influences.clone(),
            };
            
            debug!(
                decision = ?decision_log,
                "Creature made decision"
            );
        }
        
        best_action
    }
}
```

### 2. Event Logging System

Capture significant events for later analysis:

```rust
#[derive(Debug, Serialize, Clone)]
pub enum SimulationEvent {
    Birth {
        creature_id: CreatureId,
        parent_ids: (CreatureId, CreatureId),
        mutations: Vec<Mutation>,
        location: Position,
    },
    Death {
        creature_id: CreatureId,
        cause: DeathCause,
        age: f32,
        offspring_count: u32,
    },
    Conversation {
        participants: (CreatureId, CreatureId),
        concept: ConceptType,
        influence_applied: f32,
        trust_change: f32,
    },
    GroupFormed {
        leader_id: CreatureId,
        members: Vec<CreatureId>,
    },
    ResourceDepleted {
        resource_type: ResourceType,
        location: Position,
    },
    PopulationMilestone {
        total_population: usize,
        milestone_type: MilestoneType,
    },
}

pub struct EventLogger {
    events: RingBuffer<SimulationEvent>,
    event_counts: HashMap<String, u64>,
    interesting_patterns: Vec<Pattern>,
}

impl EventLogger {
    pub fn log_event(&mut self, event: SimulationEvent) {
        let event_type = event.type_name();
        
        info!(
            target: "simulation.events",
            event = ?event,
            event_type = %event_type,
            world_time = %self.world_time,
            "Simulation event occurred"
        );
        
        self.events.push(event.clone());
        *self.event_counts.entry(event_type).or_insert(0) += 1;
        
        // Check for interesting patterns
        if let Some(pattern) = self.detect_pattern(&event) {
            warn!(
                target: "simulation.emergent",
                pattern = ?pattern,
                "Interesting pattern detected!"
            );
            self.interesting_patterns.push(pattern);
        }
    }
}
```

### 3. Performance Monitoring

Track system performance with minimal overhead:

```rust
pub struct PerformanceMonitor {
    frame_times: RingBuffer<f32>,
    system_timings: HashMap<String, RingBuffer<f32>>,
    memory_usage: RingBuffer<MemorySnapshot>,
}

impl PerformanceMonitor {
    pub fn measure_system<F, R>(&mut self, system_name: &str, f: F) -> R 
    where 
        F: FnOnce() -> R 
    {
        let span = trace_span!("system", name = %system_name);
        let _guard = span.enter();
        
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed();
        
        self.system_timings
            .entry(system_name.to_string())
            .or_insert_with(|| RingBuffer::new(1000))
            .push(duration.as_secs_f32());
        
        if duration.as_millis() > 5 {
            warn!(
                target: "performance",
                system = %system_name,
                duration_ms = %duration.as_millis(),
                "System took longer than expected"
            );
        }
        
        result
    }
}
```

### 4. Replay System

Record enough information to recreate interesting scenarios:

```rust
#[derive(Serialize, Deserialize)]
pub struct ReplayFrame {
    tick: u64,
    world_seed: u64,
    creature_snapshots: Vec<CreatureSnapshot>,
    resource_positions: Vec<(ResourceType, Position)>,
    events_this_frame: Vec<SimulationEvent>,
    random_seed: u64,
}

pub struct ReplayRecorder {
    frames: VecDeque<ReplayFrame>,
    recording_mode: RecordingMode,
    interesting_creature_ids: HashSet<CreatureId>,
}

impl ReplayRecorder {
    pub fn mark_interesting(&mut self, creature_id: CreatureId, reason: &str) {
        info!(
            target: "replay",
            creature.id = %creature_id,
            reason = %reason,
            "Marking creature as interesting for replay"
        );
        
        self.interesting_creature_ids.insert(creature_id);
        self.recording_mode = RecordingMode::Detailed;
    }
    
    pub fn should_record_frame(&self, tick: u64) -> bool {
        match self.recording_mode {
            RecordingMode::Off => false,
            RecordingMode::Sparse => tick % 60 == 0, // Every second at 60fps
            RecordingMode::Normal => tick % 10 == 0,
            RecordingMode::Detailed => true,
        }
    }
}
```

### 5. Debug Visualization

Real-time visualization of logs in the UI:

```rust
pub struct DebugOverlay {
    log_buffer: RingBuffer<LogEntry>,
    filters: LogFilters,
    selected_creature: Option<CreatureId>,
    show_categories: HashSet<LogCategory>,
}

impl DebugOverlay {
    pub fn render(&mut self, ui: &mut egui::Ui, world: &World) {
        egui::Window::new("Debug Logs")
            .scroll2([false, true])
            .show(ui.ctx(), |ui| {
                // Filter controls
                ui.horizontal(|ui| {
                    ui.label("Categories:");
                    for category in LogCategory::all() {
                        let mut enabled = self.show_categories.contains(&category);
                        if ui.checkbox(&format!("{:?}", category), &mut enabled) {
                            if enabled {
                                self.show_categories.insert(category);
                            } else {
                                self.show_categories.remove(&category);
                            }
                        }
                    }
                });
                
                // Creature selector
                ui.horizontal(|ui| {
                    ui.label("Track Creature:");
                    egui::ComboBox::from_label("")
                        .selected_text(format!("{:?}", self.selected_creature))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.selected_creature, None, "All");
                            for creature_id in world.all_creatures() {
                                ui.selectable_value(
                                    &mut self.selected_creature,
                                    Some(creature_id),
                                    format!("Creature {}", creature_id)
                                );
                            }
                        });
                });
                
                // Log display
                ui.separator();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for entry in self.filtered_logs() {
                        ui.colored_label(
                            entry.level_color(),
                            format!("[{:.2}] {}", entry.timestamp, entry.message)
                        );
                    }
                });
            });
    }
}
```

## Configuration System

### Log Configuration

```rust
#[derive(Debug, Deserialize, Clone)]
pub struct LogConfig {
    // Output settings
    pub console_level: Level,
    pub file_level: Level,
    pub log_directory: PathBuf,
    pub max_log_size_mb: u64,
    pub max_log_files: u32,
    
    // Per-system verbosity
    pub system_levels: HashMap<String, Level>,
    
    // Performance settings
    pub enable_tracy: bool,
    pub metrics_interval_ms: u64,
    pub event_buffer_size: usize,
    
    // Replay settings
    pub replay_mode: RecordingMode,
    pub auto_mark_interesting: bool,
    pub interesting_patterns: Vec<PatternTrigger>,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            console_level: Level::INFO,
            file_level: Level::DEBUG,
            log_directory: PathBuf::from("logs"),
            max_log_size_mb: 100,
            max_log_files: 10,
            system_levels: HashMap::new(),
            enable_tracy: cfg!(debug_assertions),
            metrics_interval_ms: 1000,
            event_buffer_size: 10000,
            replay_mode: RecordingMode::Normal,
            auto_mark_interesting: true,
            interesting_patterns: vec![
                PatternTrigger::MassExtinction,
                PatternTrigger::RapidEvolution,
                PatternTrigger::SocialRevolution,
            ],
        }
    }
}
```

### Environment Variables

```bash
# Set via environment for debugging
export CREATURE_SIM_LOG=debug
export CREATURE_SIM_LOG_CREATURE=trace  # Detailed creature logs
export CREATURE_SIM_LOG_PERF=warn       # Only performance warnings
export CREATURE_SIM_TRACY=1             # Enable Tracy profiler
```

## Usage Examples

### Debugging Creature Behavior

```rust
// In creature decision making
creature_log!(
    Level::DEBUG,
    self.id,
    needs = ?self.needs,
    "Evaluating action options"
);

for (action, utility) in &utilities {
    creature_log!(
        Level::TRACE,
        self.id,
        action = ?action,
        utility = utility,
        "Calculated utility score"
    );
}

creature_log!(
    Level::INFO,
    self.id,
    chosen_action = ?best_action,
    utility = best_utility,
    "Creature chose action"
);
```

### Tracking Population Dynamics

```rust
// In population manager
if population.size() % 100 == 0 {
    info!(
        target: "population",
        size = population.size(),
        births_this_generation = stats.births,
        deaths_this_generation = stats.deaths,
        average_lifespan = stats.avg_lifespan,
        "Population milestone reached"
    );
}

if let Some(anomaly) = detect_population_anomaly(&stats) {
    warn!(
        target: "population.anomaly",
        anomaly = ?anomaly,
        "Unusual population dynamics detected"
    );
}
```

### Performance Debugging

```rust
// In main game loop
performance.measure_system("creature_updates", || {
    for creature in &mut creatures {
        creature.update(dt);
    }
});

performance.measure_system("social_interactions", || {
    social_system.process_conversations(&mut creatures);
});

// Automatic warnings for slow frames
if frame_time > target_frame_time * 1.5 {
    warn!(
        target: "performance.frame",
        frame_time_ms = frame_time * 1000.0,
        creature_count = creatures.len(),
        "Frame took longer than expected"
    );
}
```

## Analysis Tools

### Log Analysis Scripts

Create a `tools/` directory with analysis scripts:

```python
# tools/analyze_logs.py
import json
import pandas as pd
from datetime import datetime

def analyze_creature_decisions(log_file):
    """Analyze creature decision patterns from logs."""
    decisions = []
    
    with open(log_file, 'r') as f:
        for line in f:
            log = json.loads(line)
            if log.get('target') == 'creature.decision':
                decisions.append({
                    'creature_id': log['creature.id'],
                    'time': log['timestamp'],
                    'action': log['chosen_action'],
                    'hunger': log['needs']['hunger'],
                    'social': log['needs']['social'],
                })
    
    df = pd.DataFrame(decisions)
    
    # Analyze patterns
    print("Decision frequency by need levels:")
    print(df.groupby(['action', pd.cut(df['hunger'], bins=5)])
          .size().unstack(fill_value=0))

def find_interesting_behaviors(log_file):
    """Identify unusual patterns in creature behavior."""
    events = []
    
    with open(log_file, 'r') as f:
        for line in f:
            log = json.loads(line)
            if 'pattern' in log:
                events.append(log)
    
    return events
```

### Real-time Log Viewer

```rust
// In UI system
pub struct LogViewer {
    search_query: String,
    time_range: (f64, f64),
    level_filter: Level,
    following_creature: Option<CreatureId>,
}

impl LogViewer {
    pub fn update(&mut self, logs: &LogBuffer) {
        let filtered = logs.iter()
            .filter(|log| log.level >= self.level_filter)
            .filter(|log| {
                if let Some(id) = self.following_creature {
                    log.creature_id == Some(id)
                } else {
                    true
                }
            })
            .filter(|log| {
                self.search_query.is_empty() || 
                log.message.contains(&self.search_query)
            });
        
        // Update display with filtered logs
    }
}
```

## Best Practices

### 1. Contextual Logging
Always include relevant context:
```rust
// Good
info!(
    creature.id = %self.id,
    creature.age = %self.age,
    action = ?action,
    needs = ?self.needs.snapshot(),
    "Creature took action"
);

// Bad
info!("Action taken");
```

### 2. Appropriate Log Levels
- **TRACE**: Detailed decision calculations, every update
- **DEBUG**: Significant decisions, state changes
- **INFO**: Major events (birth, death, milestones)
- **WARN**: Unusual patterns, performance issues
- **ERROR**: System failures, data corruption

### 3. Structured Data
Use structured fields for analysis:
```rust
// Good - structured for analysis
debug!(
    target: "genetics",
    parent1.id = %parent1_id,
    parent2.id = %parent2_id,
    mutations.count = mutations.len(),
    offspring.traits = ?offspring_traits,
    "Reproduction completed"
);
```

### 4. Performance Considerations
- Use log levels to control verbosity
- Defer expensive formatting with closures
- Use sampling for high-frequency events
- Batch log writes

```rust
// Only format if logging is enabled
if log_enabled!(Level::DEBUG) {
    let expensive_debug_info = calculate_debug_info();
    debug!(info = ?expensive_debug_info, "Debug details");
}
```

## Integration with Testing

### Test Assertions on Logs

```rust
#[test]
fn creature_logs_decision_making() {
    let (subscriber, handle) = capture_logs();
    
    let mut creature = TestCreature::hungry();
    let world = TestWorld::with_food();
    
    creature.make_decision(&world);
    
    // Verify expected logs were created
    let logs = handle.logs();
    assert!(logs.iter().any(|log| {
        log.target == "creature.decision" &&
        log.message.contains("chose action")
    }));
}
```

### Performance Regression Tests

```rust
#[test]
fn performance_stays_within_bounds() {
    let mut world = create_large_world();
    let mut monitor = PerformanceMonitor::new();
    
    for _ in 0..100 {
        monitor.measure_system("update", || {
            world.update(0.016);
        });
    }
    
    let avg_frame_time = monitor.average_frame_time();
    assert!(
        avg_frame_time < 0.016,
        "Average frame time {} exceeds target",
        avg_frame_time
    );
}
```

---
*Last Updated: 2024-01-XX*
