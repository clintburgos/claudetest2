# Debugging Guide

## Overview
This guide provides practical debugging strategies for common issues in the creature simulation using our logging system.

## Common Debugging Scenarios

### 1. Creature Behaving Unexpectedly

**Symptoms**: A creature isn't eating when hungry, ignoring mates, or making poor decisions.

**Debugging Approach**:
```rust
// Enable detailed logging for specific creature
LogConfig::set_creature_verbosity(creature_id, Level::TRACE);

// In your debug UI, add tracking
debug_overlay.track_creature(creature_id);

// Add temporary debug spans
let span = info_span!(
    "debug_creature",
    creature.id = %creature_id,
    issue = "not_eating_when_hungry"
);
```

**What to look for in logs**:
- Need values at decision time
- Available actions considered
- Utility calculations for each action
- External influences affecting decisions
- Recent conversation effects

### 2. Population Crashes

**Symptoms**: Sudden population decline, mass extinctions.

**Debugging Approach**:
```rust
// Enable population analytics
LogConfig::enable_population_tracking(true);

// Add death cause analysis
impl DeathLogger {
    fn analyze_deaths(&self, window: Duration) -> DeathAnalysis {
        let recent_deaths = self.deaths_in_window(window);
        
        warn!(
            target: "population.crash",
            total_deaths = recent_deaths.len(),
            by_cause = ?self.group_by_cause(&recent_deaths),
            "Analyzing population crash"
        );
        
        DeathAnalysis {
            primary_cause: self.most_common_cause(&recent_deaths),
            age_distribution: self.age_histogram(&recent_deaths),
            location_clusters: self.find_death_clusters(&recent_deaths),
        }
    }
}
```

**Investigation queries**:
```python
# In log analysis tool
def investigate_crash(log_file, crash_time):
    # Look for resource depletion
    resources = extract_events(log_file, "resource.depleted", 
                              time_range=(crash_time - 60, crash_time))
    
    # Check for cascade effects
    deaths = extract_events(log_file, "creature.death",
                           time_range=(crash_time - 30, crash_time + 30))
    
    # Analyze genetic diversity before crash
    genetics = extract_metrics(log_file, "population.genetics",
                              time_range=(crash_time - 300, crash_time))
```

### 3. Performance Degradation

**Symptoms**: FPS drops, stuttering, increasing frame times.

**Debugging Approach**:
```rust
// Enable system profiling
performance_monitor.set_profiling_level(ProfilingLevel::Detailed);

// Add automatic performance snapshots
if frame_time > target_frame_time * 2.0 {
    error!(
        target: "performance.spike",
        frame_time_ms = frame_time * 1000.0,
        snapshot = ?self.capture_performance_snapshot(),
        "Severe frame spike detected"
    );
}

// Track hot paths
#[instrument(skip_all, fields(creature_count = creatures.len()))]
fn update_all_creatures(&mut self, creatures: &mut [Creature], dt: f32) {
    // Implementation
}
```

**Performance analysis**:
```rust
// Find bottlenecks
let bottlenecks = performance_monitor.find_slowest_systems(5);
for (system, avg_time) in bottlenecks {
    warn!(
        "System '{}' averaging {:.2}ms per frame",
        system, avg_time * 1000.0
    );
}
```

### 4. Emergent Behavior Investigation

**Symptoms**: Unexpected group formations, cultural developments, evolution patterns.

**Debugging Approach**:
```rust
// Pattern detection system
impl PatternDetector {
    fn detect_emergent_behavior(&mut self, world: &World) {
        // Check for group formations
        if let Some(groups) = self.detect_groups(world) {
            for group in groups {
                info!(
                    target: "emergent.social",
                    group_size = group.members.len(),
                    leader = ?group.leader_traits(),
                    cohesion = group.calculate_cohesion(),
                    "New group formation detected"
                );
            }
        }
        
        // Check for behavioral convergence
        if let Some(convergence) = self.detect_behavioral_convergence(world) {
            warn!(
                target: "emergent.culture",
                behavior = ?convergence.behavior_type,
                adoption_rate = convergence.adoption_rate,
                "Cultural behavior emerging"
            );
        }
    }
}
```

### 5. Genetic Anomalies

**Symptoms**: Traits not inheriting correctly, impossible mutations, genetic drift.

**Debugging Approach**:
```rust
// Detailed reproduction logging
impl GeneticLogger {
    fn log_reproduction(&mut self, parent1: &Creature, parent2: &Creature, 
                       offspring: &Creature) {
        let inheritance_analysis = self.analyze_inheritance(
            parent1, parent2, offspring
        );
        
        debug!(
            target: "genetics.reproduction",
            parent1.genes = ?parent1.dna.summary(),
            parent2.genes = ?parent2.dna.summary(),
            offspring.genes = ?offspring.dna.summary(),
            mutations = ?inheritance_analysis.mutations,
            unexpected_traits = ?inheritance_analysis.anomalies,
            "Reproduction event"
        );
        
        if !inheritance_analysis.anomalies.is_empty() {
            error!(
                target: "genetics.anomaly",
                anomalies = ?inheritance_analysis.anomalies,
                "Genetic anomaly detected!"
            );
        }
    }
}
```

## Debug Commands

Add console commands for live debugging:

```rust
pub enum DebugCommand {
    TrackCreature(CreatureId),
    LogLevel(String, Level),
    DumpCreatureState(CreatureId),
    StartReplay(Duration),
    AnalyzePopulation,
    ProfileSystem(String),
    InjectEvent(SimulationEvent),
}

impl DebugConsole {
    fn execute_command(&mut self, cmd: DebugCommand, world: &mut World) {
        match cmd {
            DebugCommand::DumpCreatureState(id) => {
                if let Some(creature) = world.get_creature(id) {
                    println!("{}", serde_json::to_string_pretty(creature).unwrap());
                }
            },
            DebugCommand::ProfileSystem(system_name) => {
                self.profiling_target = Some(system_name);
                info!("Profiling {} for next 1000 frames", system_name);
            },
            // ... other commands
        }
    }
}
```

## Log Analysis Patterns

### Finding Cause and Effect

```python
def trace_creature_death(log_file, creature_id, death_time):
    """Trace back what led to a creature's death."""
    
    # Get creature's last hour of life
    creature_logs = extract_creature_logs(
        log_file, creature_id, 
        time_range=(death_time - 3600, death_time)
    )
    
    # Find critical events
    critical_events = []
    
    # Check for starvation pattern
    hunger_trend = extract_need_trend(creature_logs, 'hunger')
    if hunger_trend.is_increasing():
        # Find when creature last ate
        last_meal = find_last_action(creature_logs, 'eat')
        critical_events.append(('last_meal', last_meal))
        
        # Check why it didn't eat
        failed_eat_attempts = find_failed_actions(creature_logs, 'seek_food')
        for attempt in failed_eat_attempts:
            print(f"Failed to find food at {attempt.time}: {attempt.reason}")
    
    return critical_events
```

### Performance Correlation

```python
def correlate_performance_with_population(log_file):
    """Find correlation between creature count and frame time."""
    
    perf_logs = extract_performance_logs(log_file)
    pop_logs = extract_population_logs(log_file)
    
    # Align by timestamp and analyze
    correlation = calculate_correlation(
        perf_logs['frame_time'],
        pop_logs['creature_count']
    )
    
    print(f"Correlation: {correlation}")
    
    # Find specific bottlenecks
    spike_times = find_performance_spikes(perf_logs)
    for spike_time in spike_times:
        world_state = reconstruct_world_state(log_file, spike_time)
        print(f"At spike {spike_time}:")
        print(f"  Creatures: {world_state.creature_count}")
        print(f"  Active conversations: {world_state.active_conversations}")
```

## Debugging Workflow

1. **Reproduce the Issue**
   - Save the world seed
   - Record the approximate time
   - Note any user actions

2. **Enable Targeted Logging**
   ```rust
   // For specific issue types
   match issue_type {
       IssueType::CreatureBehavior => {
           config.system_levels.insert("creature", Level::TRACE);
           config.system_levels.insert("ai", Level::DEBUG);
       },
       IssueType::Performance => {
           config.enable_tracy = true;
           config.system_levels.insert("performance", Level::DEBUG);
       },
       IssueType::PopulationDynamics => {
           config.system_levels.insert("population", Level::DEBUG);
           config.system_levels.insert("genetics", Level::DEBUG);
       },
   }
   ```

3. **Collect Data**
   - Run until issue reproduces
   - Use replay system if needed
   - Export relevant logs

4. **Analyze**
   - Use log analysis scripts
   - Look for patterns
   - Correlate events

5. **Form Hypothesis**
   - Based on log evidence
   - Create minimal test case
   - Add specific assertions

6. **Fix and Verify**
   - Implement fix
   - Verify with logs
   - Add regression test

---
*Last Updated: 2024-01-XX*
