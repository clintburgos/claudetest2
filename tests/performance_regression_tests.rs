//! Performance regression tests to ensure consistent performance across changes

use creature_simulation::prelude::*;
use creature_simulation::core::simulation_control::SimulationControl;
use creature_simulation::core::determinism::DeterministicRng;
use creature_simulation::plugins::{SpatialPlugin, CreatureSpawnedEvent, CreatureDiedEvent, ResourceConsumedEvent, ResourceDepletedEvent};
use bevy::app::{App, AppExit, ScheduleRunnerPlugin};
use bevy::asset::AssetPlugin;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, DiagnosticsStore};
use bevy::ecs::system::Resource;
use bevy::render::texture::ImagePlugin;
use bevy::time::Time;
use std::time::{Duration, Instant};

/// Performance metrics collected during tests
#[derive(Default, Debug, Clone)]
struct PerformanceMetrics {
    frame_times: Vec<f32>,
    fps_samples: Vec<f32>,
    start_time: Option<Instant>,
    end_time: Option<Instant>,
}

impl PerformanceMetrics {
    fn add_frame_time(&mut self, time: f32) {
        self.frame_times.push(time);
    }
    
    fn add_fps(&mut self, fps: f32) {
        self.fps_samples.push(fps);
    }
    
    fn average_fps(&self) -> f32 {
        if self.fps_samples.is_empty() {
            0.0
        } else {
            self.fps_samples.iter().sum::<f32>() / self.fps_samples.len() as f32
        }
    }
    
    fn percentile_frame_time(&self, percentile: f32) -> f32 {
        if self.frame_times.is_empty() {
            return 0.0;
        }
        
        let mut sorted = self.frame_times.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let index = ((percentile / 100.0) * sorted.len() as f32) as usize;
        sorted[index.min(sorted.len() - 1)]
    }
    
    fn max_frame_time(&self) -> f32 {
        self.frame_times.iter().copied().fold(0.0, f32::max)
    }
}

/// Resource to control performance tests
#[derive(Resource)]
struct PerformanceTestController {
    test_duration: Duration,
    warmup_duration: Duration,
    creature_count: usize,
    metrics: PerformanceMetrics,
    warmup_complete: bool,
    test_start: Option<Instant>,
}

impl Default for PerformanceTestController {
    fn default() -> Self {
        Self {
            test_duration: Duration::from_secs(30),
            warmup_duration: Duration::from_secs(5),
            creature_count: 500,
            metrics: PerformanceMetrics::default(),
            warmup_complete: false,
            test_start: None,
        }
    }
}

/// Performance regression thresholds
struct PerformanceThresholds {
    min_average_fps: f32,
    max_frame_time_ms: f32,
    max_95th_percentile_ms: f32,
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            min_average_fps: 55.0,      // Must maintain at least 55 FPS average
            max_frame_time_ms: 20.0,     // No frame should exceed 20ms
            max_95th_percentile_ms: 18.0, // 95% of frames under 18ms
        }
    }
}

fn setup_performance_test(
    mut commands: Commands,
    mut controller: ResMut<PerformanceTestController>,
) {
    // Spawn test creatures
    let spawn_radius = 800.0;
    
    for i in 0..controller.creature_count {
        let angle = (i as f32 / controller.creature_count as f32) * std::f32::consts::TAU;
        let distance = (i as f32 / controller.creature_count as f32) * spawn_radius;
        let x = angle.cos() * distance;
        let y = angle.sin() * distance;
        
        commands.spawn(CreatureBundle::new(Vec2::new(x, y), 1.0));
    }
    
    // Spawn resources
    for i in 0..50 {
        let angle = (i as f32 / 50.0) * std::f32::consts::TAU;
        let x = angle.cos() * 300.0;
        let y = angle.sin() * 300.0;
        
        if i % 2 == 0 {
            commands.spawn(ResourceBundle::new(Vec2::new(x, y), ResourceType::Food, 50.0));
        } else {
            commands.spawn(ResourceBundle::new(Vec2::new(x, y), ResourceType::Water, 50.0));
        }
    }
    
    controller.test_start = Some(Instant::now());
    controller.metrics.start_time = Some(Instant::now());
    
    info!("Performance test started with {} creatures", controller.creature_count);
}

fn collect_performance_metrics(
    mut controller: ResMut<PerformanceTestController>,
    diagnostics: Res<DiagnosticsStore>,
    _time: Res<Time>,
    mut exit: EventWriter<AppExit>,
) {
    let elapsed = controller.test_start
        .map(|start| start.elapsed())
        .unwrap_or_default();
    
    // Warmup phase
    if !controller.warmup_complete {
        if elapsed >= controller.warmup_duration {
            controller.warmup_complete = true;
            controller.metrics = PerformanceMetrics::default();
            info!("Warmup complete, starting performance measurement");
        }
        return;
    }
    
    // Collect metrics
    if let Some(fps_diagnostic) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(fps) = fps_diagnostic.smoothed() {
            controller.metrics.add_fps(fps as f32);
        }
    }
    
    if let Some(frame_time_diagnostic) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FRAME_TIME) {
        if let Some(frame_time) = frame_time_diagnostic.smoothed() {
            controller.metrics.add_frame_time(frame_time as f32);
        }
    }
    
    // Check if test is complete
    if elapsed >= controller.warmup_duration + controller.test_duration {
        controller.metrics.end_time = Some(Instant::now());
        
        // Print results
        let avg_fps = controller.metrics.average_fps();
        let max_frame_time = controller.metrics.max_frame_time();
        let p95_frame_time = controller.metrics.percentile_frame_time(95.0);
        
        println!("\n=== Performance Test Results ===");
        println!("Creatures: {}", controller.creature_count);
        println!("Test Duration: {:?}", controller.test_duration);
        println!("Average FPS: {:.1}", avg_fps);
        println!("Max Frame Time: {:.2}ms", max_frame_time);
        println!("95th Percentile Frame Time: {:.2}ms", p95_frame_time);
        println!("Total Frames: {}", controller.metrics.frame_times.len());
        
        exit.send(AppExit);
    }
}

/// Verify performance meets regression thresholds
fn verify_performance_thresholds(metrics: &PerformanceMetrics) -> Result<(), String> {
    let thresholds = PerformanceThresholds::default();
    let mut errors = Vec::new();
    
    let avg_fps = metrics.average_fps();
    if avg_fps < thresholds.min_average_fps {
        errors.push(format!(
            "Average FPS {:.1} below threshold {:.1}",
            avg_fps, thresholds.min_average_fps
        ));
    }
    
    let max_frame_time = metrics.max_frame_time();
    if max_frame_time > thresholds.max_frame_time_ms {
        errors.push(format!(
            "Max frame time {:.2}ms exceeds threshold {:.2}ms",
            max_frame_time, thresholds.max_frame_time_ms
        ));
    }
    
    let p95_frame_time = metrics.percentile_frame_time(95.0);
    if p95_frame_time > thresholds.max_95th_percentile_ms {
        errors.push(format!(
            "95th percentile frame time {:.2}ms exceeds threshold {:.2}ms",
            p95_frame_time, thresholds.max_95th_percentile_ms
        ));
    }
    
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

#[test]
fn test_performance_regression_500_creatures() {
    let mut app = App::new();
    
    // Use MinimalPlugins for headless testing
    app.add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(
        Duration::from_secs_f64(1.0 / 60.0)
    )))
    .add_plugins((
        AssetPlugin::default(),
        ImagePlugin::default(),
        SpatialPlugin,
        SimulationPlugin,
        // Skip rendering plugins for headless test
        // RenderingPlugin,
        // CameraPlugin,
        // SelectionPlugin,
        // UiEguiPlugin,
        // DebugPlugin,
    ))
    .add_plugins(FrameTimeDiagnosticsPlugin)
    .init_resource::<PerformanceTestController>()
    .init_resource::<SimulationControl>()
    .insert_resource(DeterministicRng::new(12345))
    // Add required events
    .add_event::<CreatureSpawnedEvent>()
    .add_event::<CreatureDiedEvent>()
    .add_event::<ResourceConsumedEvent>()
    .add_event::<ResourceDepletedEvent>()
    .add_systems(Startup, setup_performance_test)
    .add_systems(Update, collect_performance_metrics);
    
    // Run for limited time
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(40) {
        app.update();
        
        if app.world.get_resource::<Events<AppExit>>().map(|e| !e.is_empty()).unwrap_or(false) {
            break;
        }
    }
    
    // Verify performance
    let controller = app.world.resource::<PerformanceTestController>();
    match verify_performance_thresholds(&controller.metrics) {
        Ok(()) => println!("\n✅ Performance regression test PASSED"),
        Err(e) => panic!("\n❌ Performance regression test FAILED:\n{}", e),
    }
}

#[test]
fn test_performance_scaling() {
    let creature_counts = vec![100, 250, 500, 750];
    let mut results = Vec::new();
    
    for count in creature_counts {
        let mut app = App::new();
        
        // Use MinimalPlugins for headless testing
        app.add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(
            Duration::from_secs_f64(1.0 / 60.0)
        )))
        .add_plugins((
            AssetPlugin::default(),
            ImagePlugin::default(),
            SpatialPlugin,
            SimulationPlugin,
            FrameTimeDiagnosticsPlugin,
        ))
        .insert_resource(PerformanceTestController {
            creature_count: count,
            test_duration: Duration::from_secs(10),
            warmup_duration: Duration::from_secs(2),
            ..default()
        })
        .init_resource::<SimulationControl>()
        .insert_resource(DeterministicRng::new(12345))
        // Add required events
        .add_event::<CreatureSpawnedEvent>()
        .add_event::<CreatureDiedEvent>()
        .add_event::<ResourceConsumedEvent>()
        .add_event::<ResourceDepletedEvent>()
        .add_systems(Startup, setup_performance_test)
        .add_systems(Update, collect_performance_metrics);
        
        // Run test
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(15) {
            app.update();
            if app.world.get_resource::<Events<AppExit>>().map(|e| !e.is_empty()).unwrap_or(false) {
                break;
            }
        }
        
        // Extract metrics
        let state = app.world.resource::<PerformanceTestController>();
        let metrics = &state.metrics;
        
        if !metrics.fps_samples.is_empty() {
            let avg_fps = metrics.fps_samples.iter().sum::<f32>() / metrics.fps_samples.len() as f32;
            let min_fps = metrics.fps_samples.iter().fold(f32::MAX, |a, &b| a.min(b));
            
            let mut sorted_times = metrics.frame_times.clone();
            sorted_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let p95 = if !sorted_times.is_empty() {
                sorted_times[sorted_times.len() * 95 / 100]
            } else {
                0.0
            };
            
            println!("{}\t{:.1}\t{:.1}\t{:.2}ms", count, avg_fps, min_fps, p95);
            results.push((count, avg_fps));
        }
    }
    
    // Print scaling results
    println!("\n=== Performance Scaling Results ===");
    for (count, fps) in &results {
        println!("{} creatures: {:.1} FPS", count, fps);
    }
    
    // Verify scaling is reasonable (FPS shouldn't drop too dramatically)
    for i in 1..results.len() {
        let (prev_count, prev_fps): (usize, f32) = results[i - 1];
        let (curr_count, curr_fps): (usize, f32) = results[i];
        
        let count_ratio = curr_count as f32 / prev_count as f32;
        let fps_ratio = curr_fps / prev_fps;
        
        // FPS should not drop more than linearly with creature count
        if fps_ratio < (1.0 / count_ratio) * 0.8 {
            panic!(
                "Performance scaling issue: {} -> {} creatures caused FPS drop from {:.1} to {:.1}",
                prev_count, curr_count, prev_fps, curr_fps
            );
        }
    }
    
    println!("✅ Performance scaling test PASSED");
}

#[test]
fn test_performance_stress_spikes() {
    let mut app = App::new();
    
    // Use MinimalPlugins for headless testing
    app.add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(
        Duration::from_secs_f64(1.0 / 60.0)
    )))
    .add_plugins((
        AssetPlugin::default(),
        ImagePlugin::default(),
        SpatialPlugin,
        SimulationPlugin,
        FrameTimeDiagnosticsPlugin,
    ))
    .insert_resource(PerformanceTestController {
        creature_count: 300,
        test_duration: Duration::from_secs(15),
        ..default()
    })
    .init_resource::<SimulationControl>()
    .insert_resource(DeterministicRng::new(12345))
    // Add required events
    .add_event::<CreatureSpawnedEvent>()
    .add_event::<CreatureDiedEvent>()
    .add_event::<ResourceConsumedEvent>()
    .add_event::<ResourceDepletedEvent>()
    .add_systems(Startup, setup_performance_test);
    
    // Custom system to create stress spikes
    let mut spike_timer = 0.0;
    app.add_systems(Update, move |mut commands: Commands, time: Res<Time>| {
        spike_timer += time.delta_seconds();
        
        // Every 3 seconds, spawn 50 additional creatures
        if spike_timer > 3.0 {
            spike_timer = 0.0;
            
            for i in 0..50 {
                let angle = i as f32 * 0.1;
                let x = angle.cos() * 500.0;
                let y = angle.sin() * 500.0;
                commands.spawn(CreatureBundle::new(Vec2::new(x, y), 1.0));
            }
            
            info!("Stress spike: spawned 50 additional creatures");
        }
    });
    
    app.add_systems(Update, collect_performance_metrics);
    
    // Run test
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(20) {
        app.update();
        if app.world.get_resource::<Events<AppExit>>().map(|e| !e.is_empty()).unwrap_or(false) {
            break;
        }
    }
    
    // Verify no catastrophic frame drops
    let controller = app.world.resource::<PerformanceTestController>();
    let max_frame_time = controller.metrics.max_frame_time();
    
    if max_frame_time > 50.0 {
        panic!(
            "Stress test failed: frame time spike of {:.2}ms detected",
            max_frame_time
        );
    }
    
    println!("\n✅ Performance stress test PASSED");
    println!("Max frame time during stress: {:.2}ms", max_frame_time);
}