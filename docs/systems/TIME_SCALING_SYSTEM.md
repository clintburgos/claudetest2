# Unified Time Scaling Architecture

## Overview

The time scaling system allows the simulation to run at different speeds from pause (0x) to generational time (1000x), maintaining accuracy at lower speeds and using statistical approximations at higher speeds.

## Time Scale Levels

| Speed | Multiplier | Method | Target FPS | Accuracy |
|-------|------------|--------|------------|----------|
| Pause | 0x | No updates | 144+ | N/A |
| Slow | 0.1x - 0.9x | Full simulation | 120+ | 100% |
| Normal | 1x | Full simulation | 90+ | 100% |
| Fast | 2x - 10x | Full simulation | 60+ | 100% |
| Very Fast | 11x - 100x | Reduced accuracy | 60+ | 95% |
| Ultra Fast | 101x - 500x | Batched updates | 60+ | 85% |
| Generational | 501x - 1000x | Statistical only | 60+ | 70% |

## Core Time System

```rust
pub struct TimeManager {
    // Current state
    game_time: f64,              // Total simulated time in seconds
    real_time: f64,              // Total real time elapsed
    time_scale: f32,             // Current time multiplier
    
    // Time accumulation
    accumulator: f64,            // Unprocessed time
    fixed_timestep: f64,         // 1/60 second
    max_substeps: u32,           // Prevent spiral of death
    
    // Performance
    target_fps: f32,
    current_fps: f32,
    frame_budget: Duration,
    
    // Time scaling mode
    scaling_mode: TimeScalingMode,
    statistical_threshold: f32,   // When to switch to statistical
}

#[derive(Debug, Clone, Copy)]
pub enum TimeScalingMode {
    Realtime,      // 1:1 simulation
    FastForward,   // Accelerated but accurate
    Batched,       // Update in batches
    Statistical,   // Pure statistics
}

impl TimeManager {
    pub fn update(&mut self, real_dt: f64) {
        // Add to accumulator based on time scale
        self.accumulator += real_dt * self.time_scale as f64;
        
        // Determine scaling mode
        self.scaling_mode = self.determine_scaling_mode();
        
        match self.scaling_mode {
            TimeScalingMode::Realtime => self.update_realtime(),
            TimeScalingMode::FastForward => self.update_fast_forward(),
            TimeScalingMode::Batched => self.update_batched(),
            TimeScalingMode::Statistical => self.update_statistical(),
        }
        
        self.real_time += real_dt;
    }
    
    fn determine_scaling_mode(&self) -> TimeScalingMode {
        match self.time_scale {
            s if s == 0.0 => TimeScalingMode::Realtime, // Paused
            s if s <= 10.0 => TimeScalingMode::Realtime,
            s if s <= 100.0 => TimeScalingMode::FastForward,
            s if s <= 500.0 => TimeScalingMode::Batched,
            _ => TimeScalingMode::Statistical,
        }
    }
}
```

## Scaling Strategies

### Realtime Mode (0x - 10x)

Full simulation with normal fixed timestep:

```rust
impl TimeManager {
    fn update_realtime(&mut self) {
        let mut substeps = 0;
        
        while self.accumulator >= self.fixed_timestep && substeps < self.max_substeps {
            self.game_time += self.fixed_timestep;
            self.accumulator -= self.fixed_timestep;
            substeps += 1;
            
            // Update all systems normally
            self.dispatch_update(self.fixed_timestep);
        }
        
        // Interpolation remainder
        let alpha = self.accumulator / self.fixed_timestep;
        self.dispatch_interpolation(alpha as f32);
    }
}
```

### Fast Forward Mode (11x - 100x)

Maintain accuracy with larger timesteps:

```rust
impl TimeManager {
    fn update_fast_forward(&mut self) {
        // Use adaptive timestep
        let scaled_timestep = self.fixed_timestep * (self.time_scale / 10.0).min(10.0) as f64;
        let mut substeps = 0;
        
        while self.accumulator >= scaled_timestep && substeps < self.max_substeps {
            self.game_time += scaled_timestep;
            self.accumulator -= scaled_timestep;
            substeps += 1;
            
            // Update with scaled timestep
            self.dispatch_scaled_update(scaled_timestep);
        }
    }
}
```

### Batched Mode (101x - 500x)

Process multiple updates in batches:

```rust
pub struct BatchedUpdate {
    time_span: f64,
    update_count: u32,
    systems: BatchedSystems,
}

impl TimeManager {
    fn update_batched(&mut self) {
        let batch_size = (self.time_scale / 100.0) as u32;
        let batch_timestep = self.fixed_timestep * batch_size as f64;
        
        if self.accumulator >= batch_timestep {
            let batch = BatchedUpdate {
                time_span: batch_timestep,
                update_count: batch_size,
                systems: self.prepare_batch_systems(),
            };
            
            self.execute_batch(batch);
            self.game_time += batch_timestep;
            self.accumulator -= batch_timestep;
        }
    }
}

pub struct BatchedSystems {
    // Aggregate updates
    movement: MovementBatch,
    needs: NeedsBatch,
    social: SocialBatch,
    resources: ResourceBatch,
}

pub struct MovementBatch {
    // Instead of updating each frame, calculate final positions
    pub fn execute(&mut self, entities: &[Entity], time_span: f64) {
        for &entity in entities {
            let velocity = self.get_velocity(entity);
            let start_pos = self.get_position(entity);
            let end_pos = start_pos + velocity * time_span as f32;
            
            // Check for major obstacles only
            if self.path_clear(start_pos, end_pos) {
                self.set_position(entity, end_pos);
            }
        }
    }
}
```

### Statistical Mode (501x - 1000x)

Pure statistical simulation:

```rust
pub struct StatisticalSimulation {
    population_model: PopulationModel,
    resource_model: ResourceModel,
    genetics_model: GeneticsModel,
    social_model: SocialModel,
}

impl TimeManager {
    fn update_statistical(&mut self) {
        let time_delta = self.accumulator.min(3600.0); // Max 1 hour per update
        
        let stats = StatisticalUpdate {
            time_span: time_delta,
            current_state: self.gather_aggregate_state(),
        };
        
        // Run statistical models
        let results = self.statistical_sim.update(stats);
        
        // Apply results back to entities
        self.apply_statistical_results(results);
        
        self.game_time += time_delta;
        self.accumulator -= time_delta;
    }
}

pub struct PopulationModel {
    birth_rate: f32,
    death_rate: f32,
    
    pub fn update(&mut self, population: u32, time: f64) -> PopulationChange {
        let births = (population as f32 * self.birth_rate * time as f32) as u32;
        let deaths = (population as f32 * self.death_rate * time as f32) as u32;
        
        PopulationChange {
            births,
            deaths,
            migrations: 0,
        }
    }
}
```

## System-Specific Time Scaling

### Movement System Scaling

```rust
pub trait TimeScalable {
    fn update_normal(&mut self, dt: f64);
    fn update_batched(&mut self, time_span: f64, steps: u32);
    fn update_statistical(&mut self, time_span: f64);
}

impl TimeScalable for MovementSystem {
    fn update_normal(&mut self, dt: f64) {
        // Standard per-frame movement
        for (entity, velocity, mut position) in self.query() {
            position.0 += velocity.0 * dt as f32;
        }
    }
    
    fn update_batched(&mut self, time_span: f64, steps: u32) {
        // Calculate final positions, skip intermediate
        for (entity, velocity, mut position) in self.query() {
            let displacement = velocity.0 * time_span as f32;
            
            // Simplified collision check
            if self.can_move_to(position.0 + displacement) {
                position.0 += displacement;
            }
        }
    }
    
    fn update_statistical(&mut self, time_span: f64) {
        // Group creatures by destination
        let migrations = self.calculate_migrations(time_span);
        
        // Teleport groups to destinations
        for migration in migrations {
            self.apply_migration(migration);
        }
    }
}
```

### Needs System Scaling

```rust
impl TimeScalable for NeedsSystem {
    fn update_normal(&mut self, dt: f64) {
        for (entity, mut needs) in self.query() {
            needs.hunger += needs.metabolism * dt as f32;
            needs.thirst += needs.thirst_rate * dt as f32;
            needs.energy -= needs.activity_level * dt as f32;
        }
    }
    
    fn update_batched(&mut self, time_span: f64, steps: u32) {
        for (entity, mut needs) in self.query() {
            // Apply full time span at once
            needs.hunger += needs.metabolism * time_span as f32;
            needs.thirst += needs.thirst_rate * time_span as f32;
            needs.energy -= needs.activity_level * time_span as f32;
            
            // Check for critical events
            if needs.hunger > STARVATION_THRESHOLD {
                self.queue_death(entity, DeathCause::Starvation);
            }
        }
    }
    
    fn update_statistical(&mut self, time_span: f64) {
        // Statistical death rates
        let starvation_rate = 0.01; // 1% per hour without food
        let dehydration_rate = 0.02; // 2% per hour without water
        
        let hours = time_span / 3600.0;
        let starvation_deaths = (self.hungry_count() as f64 * starvation_rate * hours) as u32;
        
        // Apply deaths randomly
        self.apply_random_deaths(DeathCause::Starvation, starvation_deaths);
    }
}
```

### Conversation System Scaling

```rust
impl TimeScalable for ConversationSystem {
    fn update_normal(&mut self, dt: f64) {
        // Full conversation simulation
        for conversation in self.active_conversations() {
            conversation.update(dt);
        }
    }
    
    fn update_batched(&mut self, time_span: f64, steps: u32) {
        // Complete conversations instantly
        for conversation in self.active_conversations() {
            let outcome = conversation.simulate_outcome(time_span);
            self.apply_outcome(conversation.participants, outcome);
        }
    }
    
    fn update_statistical(&mut self, time_span: f64) {
        // Statistical relationship changes
        let interaction_rate = 0.1; // Interactions per hour
        let interactions = (self.creature_count() as f64 * interaction_rate * time_span / 3600.0) as u32;
        
        for _ in 0..interactions {
            let (a, b) = self.random_pair();
            let relationship_delta = self.calculate_relationship_change(a, b);
            self.update_relationship(a, b, relationship_delta);
        }
    }
}
```

## Time-Aware Events

```rust
pub enum TimeScaledEvent {
    // Instant events (happen regardless of time scale)
    Instant(Event),
    
    // Delayed events (respect time scale)
    Delayed {
        event: Event,
        delay: f64,
        scheduled_time: f64,
    },
    
    // Periodic events
    Periodic {
        event: Event,
        period: f64,
        last_triggered: f64,
    },
    
    // Statistical events (only in statistical mode)
    Statistical {
        event: Event,
        probability_per_hour: f32,
    },
}

pub struct TimeScaledEventQueue {
    instant_queue: VecDeque<Event>,
    delayed_queue: BinaryHeap<DelayedEvent>,
    periodic_events: Vec<PeriodicEvent>,
    statistical_events: Vec<StatisticalEvent>,
}

impl TimeScaledEventQueue {
    pub fn update(&mut self, game_time: f64, time_span: f64, mode: TimeScalingMode) {
        // Always process instant events
        while let Some(event) = self.instant_queue.pop_front() {
            self.dispatch(event);
        }
        
        // Process delayed events
        while let Some(delayed) = self.delayed_queue.peek() {
            if delayed.scheduled_time <= game_time {
                let event = self.delayed_queue.pop().unwrap().event;
                self.dispatch(event);
            } else {
                break;
            }
        }
        
        // Check periodic events
        for periodic in &mut self.periodic_events {
            if game_time - periodic.last_triggered >= periodic.period {
                self.dispatch(periodic.event.clone());
                periodic.last_triggered = game_time;
            }
        }
        
        // Statistical events only in statistical mode
        if matches!(mode, TimeScalingMode::Statistical) {
            for stat_event in &self.statistical_events {
                let hours = time_span / 3600.0;
                let probability = stat_event.probability_per_hour * hours as f32;
                
                if rand::random::<f32>() < probability {
                    self.dispatch(stat_event.event.clone());
                }
            }
        }
    }
}
```

## Performance Optimization

### Frame Skipping

```rust
pub struct FrameSkipper {
    systems: Vec<Box<dyn System>>,
    skip_patterns: HashMap<TypeId, SkipPattern>,
}

pub struct SkipPattern {
    base_interval: u32,    // Frames between updates at 1x
    scale_factor: f32,     // How much to increase with time scale
    min_interval: u32,     // Never skip less than this
    max_interval: u32,     // Never skip more than this
}

impl FrameSkipper {
    pub fn should_update(&self, system_id: TypeId, frame: u32, time_scale: f32) -> bool {
        let pattern = &self.skip_patterns[&system_id];
        let interval = (pattern.base_interval as f32 * (1.0 + time_scale * pattern.scale_factor))
            .max(pattern.min_interval as f32)
            .min(pattern.max_interval as f32) as u32;
            
        frame % interval == 0
    }
}
```

### Time Compression

```rust
pub struct TimeCompressor {
    compression_level: CompressionLevel,
    update_groups: Vec<UpdateGroup>,
}

pub enum CompressionLevel {
    None,        // Update everything
    Light,       // Skip cosmetic updates
    Medium,      // Skip non-critical systems
    Heavy,       // Only critical systems
    Statistical, // Pure statistics
}

pub struct UpdateGroup {
    systems: Vec<SystemId>,
    priority: Priority,
    can_skip: bool,
    can_batch: bool,
}

impl TimeCompressor {
    pub fn get_active_systems(&self) -> Vec<SystemId> {
        match self.compression_level {
            CompressionLevel::None => self.all_systems(),
            CompressionLevel::Light => self.non_cosmetic_systems(),
            CompressionLevel::Medium => self.critical_systems(),
            CompressionLevel::Heavy => self.minimal_systems(),
            CompressionLevel::Statistical => vec![SystemId::Statistical],
        }
    }
}
```

## Debug and Monitoring

```rust
pub struct TimeScalingDebug {
    show_time_info: bool,
    show_compression: bool,
    show_skip_patterns: bool,
    log_statistical: bool,
}

impl TimeScalingDebug {
    pub fn render_overlay(&self, time_manager: &TimeManager) {
        if self.show_time_info {
            ui.label(format!("Game Time: {:.1}s", time_manager.game_time));
            ui.label(format!("Time Scale: {:.1}x", time_manager.time_scale));
            ui.label(format!("Mode: {:?}", time_manager.scaling_mode));
            ui.label(format!("FPS: {:.1}", time_manager.current_fps));
        }
        
        if self.show_compression {
            ui.label(format!("Compression: {:?}", self.compression_level));
            ui.label(format!("Active Systems: {}", self.active_system_count));
            ui.label(format!("Skipped Updates: {}", self.skipped_count));
        }
    }
}
```

## Testing Time Scaling

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_accumulation() {
        let mut time_mgr = TimeManager::new();
        time_mgr.set_time_scale(10.0);
        
        // Simulate 1 second of real time
        time_mgr.update(1.0);
        
        // Should have 10 seconds of game time
        assert!((time_mgr.game_time - 10.0).abs() < 0.01);
    }
    
    #[test]
    fn test_statistical_accuracy() {
        let mut world = create_test_world_with_creatures(1000);
        let initial_pop = world.creature_count();
        
        // Run at 1000x for 1 game hour
        world.set_time_scale(1000.0);
        world.update(3.6); // 3.6 real seconds = 1 game hour
        
        // Check population is within expected range
        let final_pop = world.creature_count();
        let expected_deaths = (initial_pop as f32 * 0.01) as u32; // 1% death rate
        
        assert!((initial_pop - final_pop) as i32 - expected_deaths as i32 < 50);
    }
}
```

This unified time scaling system ensures the simulation can run at any speed while maintaining reasonable accuracy and performance.