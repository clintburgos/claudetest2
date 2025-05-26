//! Performance monitoring system with automatic threshold detection

use bevy::prelude::*;
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

/// Performance metrics for a single frame
#[derive(Debug, Clone)]
pub struct FrameMetrics {
    pub frame_number: u64,
    pub total_time: Duration,
    pub system_times: HashMap<&'static str, Duration>,
    pub entity_count: usize,
    pub timestamp: Instant,
}

/// Performance threshold configuration
#[derive(Debug, Clone)]
pub struct PerformanceThresholds {
    pub target_fps: f32,
    pub warning_fps: f32,
    pub critical_fps: f32,
    pub spike_multiplier: f32,
    pub system_budget_ms: HashMap<&'static str, f32>,
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        let mut system_budgets = HashMap::new();
        // Budget in milliseconds for each system
        system_budgets.insert("movement", 2.0);
        system_budgets.insert("decision", 3.0);
        system_budgets.insert("spatial_update", 1.0);
        system_budgets.insert("rendering", 6.0);
        system_budgets.insert("ui", 2.0);

        Self {
            target_fps: 60.0,
            warning_fps: 45.0,
            critical_fps: 30.0,
            spike_multiplier: 1.5,
            system_budget_ms: system_budgets,
        }
    }
}

/// Performance warning types
#[derive(Debug, Clone)]
pub enum PerformanceWarning {
    LowFPS {
        current: f32,
        target: f32,
    },
    SystemOverBudget {
        system: &'static str,
        time_ms: f32,
        budget_ms: f32,
    },
    FrameSpike {
        frame: u64,
        time_ms: f32,
        baseline_ms: f32,
    },
    MemoryPressure {
        used_mb: f32,
        available_mb: f32,
    },
}

/// Main performance monitoring resource
#[derive(Resource)]
pub struct PerformanceMonitor {
    frame_history: VecDeque<FrameMetrics>,
    history_size: usize,
    thresholds: PerformanceThresholds,
    warnings: Vec<PerformanceWarning>,
    current_frame: u64,
    frame_start: Option<Instant>,
    system_timers: HashMap<&'static str, Instant>,
    baseline_frame_time: Option<Duration>,
    degradation_level: QualityLevel,
}

/// Quality levels for graceful degradation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum QualityLevel {
    Ultra = 4,
    #[default]
    High = 3,
    Medium = 2,
    Low = 1,
    Minimal = 0,
}

impl QualityLevel {
    /// Get update frequencies for this quality level
    pub fn get_update_frequencies(&self) -> crate::core::performance_config::UpdateFrequencies {
        use crate::core::performance_config::UpdateFrequencies;
        match self {
            QualityLevel::Ultra => UpdateFrequencies {
                movement_divisor: 1,
                decision_divisor: 1,
                needs_divisor: 2,
                render_divisor: 1,
            },
            QualityLevel::High => UpdateFrequencies {
                movement_divisor: 1,
                decision_divisor: 2,
                needs_divisor: 3,
                render_divisor: 1,
            },
            QualityLevel::Medium => UpdateFrequencies {
                movement_divisor: 1,
                decision_divisor: 3,
                needs_divisor: 4,
                render_divisor: 1,
            },
            QualityLevel::Low => UpdateFrequencies {
                movement_divisor: 2,
                decision_divisor: 4,
                needs_divisor: 6,
                render_divisor: 2,
            },
            QualityLevel::Minimal => UpdateFrequencies {
                movement_divisor: 2,
                decision_divisor: 6,
                needs_divisor: 10,
                render_divisor: 3,
            },
        }
    }
    
    /// Get LOD settings for this quality level
    pub fn get_lod_settings(&self) -> crate::core::performance_config::LodSettings {
        use crate::core::performance_config::LodSettings;
        match self {
            QualityLevel::Ultra => LodSettings {
                lod_distance: 500.0,
                distant_update_divisor: 2,
                cull_distance: 1000.0,
            },
            QualityLevel::High => LodSettings {
                lod_distance: 300.0,
                distant_update_divisor: 3,
                cull_distance: 800.0,
            },
            QualityLevel::Medium => LodSettings {
                lod_distance: 200.0,
                distant_update_divisor: 4,
                cull_distance: 600.0,
            },
            QualityLevel::Low => LodSettings {
                lod_distance: 150.0,
                distant_update_divisor: 6,
                cull_distance: 400.0,
            },
            QualityLevel::Minimal => LodSettings {
                lod_distance: 100.0,
                distant_update_divisor: 8,
                cull_distance: 300.0,
            },
        }
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self {
            frame_history: VecDeque::with_capacity(120), // 2 seconds at 60 FPS
            history_size: 120,
            thresholds: PerformanceThresholds::default(),
            warnings: Vec::new(),
            current_frame: 0,
            frame_start: None,
            system_timers: HashMap::new(),
            baseline_frame_time: None,
            degradation_level: QualityLevel::High,
        }
    }
}

impl PerformanceMonitor {
    /// Start timing a new frame
    pub fn begin_frame(&mut self) {
        self.frame_start = Some(Instant::now());
        self.current_frame += 1;
        self.system_timers.clear();
    }

    /// Start timing a specific system
    pub fn begin_system(&mut self, system_name: &'static str) {
        self.system_timers.insert(system_name, Instant::now());
    }

    /// End timing a specific system
    pub fn end_system(&mut self, system_name: &'static str) {
        if let Some(start) = self.system_timers.get(system_name) {
            let duration = start.elapsed();
            let ms = duration.as_secs_f32() * 1000.0;

            // Check if over budget
            if let Some(&budget) = self.thresholds.system_budget_ms.get(system_name) {
                if ms > budget {
                    self.warnings.push(PerformanceWarning::SystemOverBudget {
                        system: system_name,
                        time_ms: ms,
                        budget_ms: budget,
                    });
                }
            }
        }
    }

    /// End frame timing and analyze performance
    pub fn end_frame(&mut self, entity_count: usize) {
        if let Some(start) = self.frame_start.take() {
            let total_time = start.elapsed();

            // Create frame metrics
            let metrics = FrameMetrics {
                frame_number: self.current_frame,
                total_time,
                system_times: self
                    .system_timers
                    .iter()
                    .map(|(&name, &start)| (name, start.elapsed()))
                    .collect(),
                entity_count,
                timestamp: Instant::now(),
            };

            // Add to history
            if self.frame_history.len() >= self.history_size {
                self.frame_history.pop_front();
            }
            self.frame_history.push_back(metrics.clone());

            // Analyze performance
            self.analyze_frame(&metrics);

            // Update baseline
            if self.baseline_frame_time.is_none() && self.frame_history.len() > 30 {
                self.update_baseline();
            }
        }
    }

    /// Analyze frame performance and generate warnings
    fn analyze_frame(&mut self, metrics: &FrameMetrics) {
        let frame_ms = metrics.total_time.as_secs_f32() * 1000.0;
        let current_fps = 1000.0 / frame_ms;

        // Check FPS thresholds
        if current_fps < self.thresholds.critical_fps {
            self.warnings.push(PerformanceWarning::LowFPS {
                current: current_fps,
                target: self.thresholds.target_fps,
            });
        }

        // Check for frame spikes
        if let Some(baseline) = self.baseline_frame_time {
            let baseline_ms = baseline.as_secs_f32() * 1000.0;
            if frame_ms > baseline_ms * self.thresholds.spike_multiplier {
                self.warnings.push(PerformanceWarning::FrameSpike {
                    frame: metrics.frame_number,
                    time_ms: frame_ms,
                    baseline_ms,
                });
            }
        }
    }

    /// Update baseline frame time from recent history
    fn update_baseline(&mut self) {
        if self.frame_history.len() < 30 {
            return;
        }

        // Use median of recent frames as baseline
        let mut times: Vec<_> = self.frame_history.iter().map(|m| m.total_time).collect();
        times.sort();
        self.baseline_frame_time = Some(times[times.len() / 2]);
    }

    /// Get current performance statistics
    pub fn get_stats(&self) -> PerformanceStats {
        if self.frame_history.is_empty() {
            return PerformanceStats::default();
        }

        let recent_frames = 60; // Last second
        let start_idx = self.frame_history.len().saturating_sub(recent_frames);
        let recent: &[FrameMetrics] =
            &self.frame_history.iter().skip(start_idx).cloned().collect::<Vec<_>>();

        let total_ms: f32 = recent.iter().map(|m| m.total_time.as_secs_f32() * 1000.0).sum();
        let avg_ms = total_ms / recent.len() as f32;
        let avg_fps = 1000.0 / avg_ms;

        let min_fps = recent
            .iter()
            .map(|m| 1000.0 / (m.total_time.as_secs_f32() * 1000.0))
            .fold(f32::INFINITY, f32::min);

        let max_ms = recent.iter().map(|m| m.total_time.as_secs_f32() * 1000.0).fold(0.0, f32::max);

        PerformanceStats {
            avg_fps,
            min_fps,
            avg_frame_ms: avg_ms,
            max_frame_ms: max_ms,
            entity_count: recent.last().map(|m| m.entity_count).unwrap_or(0),
            quality_level: self.degradation_level,
            warning_count: self.warnings.len(),
        }
    }

    /// Get quality settings for current performance level
    pub fn get_quality_settings(&self) -> QualitySettings {
        match self.degradation_level {
            QualityLevel::Ultra => QualitySettings {
                max_creatures: None,
                update_frequency: 1,
                render_distance: 2000.0,
                enable_shadows: true,
                enable_particles: true,
                spatial_update_freq: 1,
            },
            QualityLevel::High => QualitySettings {
                max_creatures: Some(1000),
                update_frequency: 1,
                render_distance: 1500.0,
                enable_shadows: true,
                enable_particles: true,
                spatial_update_freq: 1,
            },
            QualityLevel::Medium => QualitySettings {
                max_creatures: Some(500),
                update_frequency: 1,
                render_distance: 1000.0,
                enable_shadows: false,
                enable_particles: true,
                spatial_update_freq: 2,
            },
            QualityLevel::Low => QualitySettings {
                max_creatures: Some(300),
                update_frequency: 2,
                render_distance: 750.0,
                enable_shadows: false,
                enable_particles: false,
                spatial_update_freq: 3,
            },
            QualityLevel::Minimal => QualitySettings {
                max_creatures: Some(150),
                update_frequency: 3,
                render_distance: 500.0,
                enable_shadows: false,
                enable_particles: false,
                spatial_update_freq: 5,
            },
        }
    }

    /// Adjust quality level based on performance
    pub fn adjust_quality(&mut self) {
        let stats = self.get_stats();

        // Degrade if consistently below warning FPS
        if stats.avg_fps < self.thresholds.warning_fps
            && self.degradation_level > QualityLevel::Minimal
        {
            self.degradation_level = match self.degradation_level {
                QualityLevel::Ultra => QualityLevel::High,
                QualityLevel::High => QualityLevel::Medium,
                QualityLevel::Medium => QualityLevel::Low,
                QualityLevel::Low => QualityLevel::Minimal,
                QualityLevel::Minimal => QualityLevel::Minimal,
            };
            info!(
                "Degrading quality to {:?} due to low FPS: {:.1}",
                self.degradation_level, stats.avg_fps
            );
        }

        // Improve if consistently above target FPS
        if stats.avg_fps > self.thresholds.target_fps * 1.2
            && self.degradation_level < QualityLevel::Ultra
        {
            self.degradation_level = match self.degradation_level {
                QualityLevel::Minimal => QualityLevel::Low,
                QualityLevel::Low => QualityLevel::Medium,
                QualityLevel::Medium => QualityLevel::High,
                QualityLevel::High => QualityLevel::Ultra,
                QualityLevel::Ultra => QualityLevel::Ultra,
            };
            info!(
                "Improving quality to {:?} with FPS: {:.1}",
                self.degradation_level, stats.avg_fps
            );
        }
    }

    /// Clear warnings (call after displaying them)
    pub fn clear_warnings(&mut self) {
        self.warnings.clear();
    }

    /// Get recent warnings
    pub fn get_warnings(&self) -> &[PerformanceWarning] {
        &self.warnings
    }
}

/// Performance statistics summary
#[derive(Debug, Default)]
pub struct PerformanceStats {
    pub avg_fps: f32,
    pub min_fps: f32,
    pub avg_frame_ms: f32,
    pub max_frame_ms: f32,
    pub entity_count: usize,
    pub quality_level: QualityLevel,
    pub warning_count: usize,
}

/// Quality settings for graceful degradation
#[derive(Debug, Clone)]
pub struct QualitySettings {
    pub max_creatures: Option<usize>,
    pub update_frequency: u32,
    pub render_distance: f32,
    pub enable_shadows: bool,
    pub enable_particles: bool,
    pub spatial_update_freq: u32,
}

/// Plugin for performance monitoring
pub struct PerformanceMonitorPlugin;

impl Plugin for PerformanceMonitorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PerformanceMonitor>()
            .add_systems(First, begin_frame_timing)
            .add_systems(Last, (end_frame_timing, apply_quality_settings, log_performance_warnings).chain());
    }
}

/// System to start frame timing
fn begin_frame_timing(mut monitor: ResMut<PerformanceMonitor>) {
    monitor.begin_frame();
}

/// System to end frame timing
fn end_frame_timing(
    mut monitor: ResMut<PerformanceMonitor>,
    creature_query: Query<Entity, With<crate::components::Creature>>,
) {
    let entity_count = creature_query.iter().count();
    monitor.end_frame(entity_count);

    // Adjust quality every 60 frames (1 second at 60 FPS)
    if monitor.current_frame % 60 == 0 {
        monitor.adjust_quality();
    }
}

/// System to apply quality settings based on performance
fn apply_quality_settings(
    monitor: Res<PerformanceMonitor>,
    mut perf_config: ResMut<crate::core::performance_config::PerformanceConfig>,
) {
    // Apply update frequencies from quality level
    perf_config.update_frequencies = monitor.degradation_level.get_update_frequencies();
    
    // Apply LOD settings from quality level
    perf_config.lod_settings = monitor.degradation_level.get_lod_settings();
}

/// System to log performance warnings
fn log_performance_warnings(
    mut monitor: ResMut<PerformanceMonitor>,
    time: Res<Time>,
    mut timer: Local<Timer>,
) {
    // Initialize timer
    if timer.duration() == Duration::ZERO {
        *timer = Timer::from_seconds(5.0, TimerMode::Repeating);
    }

    timer.tick(time.delta());
    if timer.just_finished() {
        let warnings = monitor.get_warnings();
        if !warnings.is_empty() {
            warn!("Performance warnings: {} issues detected", warnings.len());
            for warning in warnings.iter().take(5) {
                warn!("  {:?}", warning);
            }
            monitor.clear_warnings();
        }

        // Log current stats
        let stats = monitor.get_stats();
        info!(
            "Performance: {:.1} FPS (min: {:.1}), {} entities, quality: {:?}",
            stats.avg_fps, stats.min_fps, stats.entity_count, stats.quality_level
        );
    }
}

/// Helper macro for timing systems
#[macro_export]
macro_rules! profile_system {
    ($monitor:expr, $system_name:expr, $code:block) => {{
        $monitor.begin_system($system_name);
        let result = $code;
        $monitor.end_system($system_name);
        result
    }};
}

#[cfg(test)]
mod tests;
