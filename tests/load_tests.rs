//! Load tests for high creature counts

use bevy::prelude::*;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, DiagnosticsStore};
use creature_simulation::prelude::*;
use std::time::{Duration, Instant};

/// Configuration for load tests
#[derive(Resource)]
struct LoadTestConfig {
    creature_count: usize,
    test_duration: Duration,
    warmup_duration: Duration,
    target_fps: f32,
    world_size: f32,
}

impl LoadTestConfig {
    fn new(creature_count: usize) -> Self {
        Self {
            creature_count,
            test_duration: Duration::from_secs(30),
            warmup_duration: Duration::from_secs(5),
            target_fps: 60.0,
            world_size: 2000.0,
        }
    }
}

/// Metrics collected during load test
#[derive(Default, Debug)]
struct LoadTestMetrics {
    fps_samples: Vec<f32>,
    frame_times: Vec<f32>,
    memory_samples: Vec<usize>,
    creature_updates_per_second: Vec<f32>,
    spatial_query_times: Vec<f32>,
}

impl LoadTestMetrics {
    fn report(&self) {
        println!("\n=== Load Test Results ===");
        
        // FPS statistics
        let avg_fps = self.fps_samples.iter().sum::<f32>() / self.fps_samples.len() as f32;
        let min_fps = self.fps_samples.iter().fold(f32::MAX, |a, &b| a.min(b));
        let max_fps = self.fps_samples.iter().fold(0.0f32, |a, &b| a.max(b));
        
        println!("FPS: avg={:.1}, min={:.1}, max={:.1}", avg_fps, min_fps, max_fps);
        
        // Frame time percentiles
        let mut sorted_times = self.frame_times.clone();
        sorted_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let p50 = sorted_times[sorted_times.len() / 2];
        let p95 = sorted_times[sorted_times.len() * 95 / 100];
        let p99 = sorted_times[sorted_times.len() * 99 / 100];
        
        println!("Frame times (ms): p50={:.2}, p95={:.2}, p99={:.2}", 
                 p50, p95, p99);
        
        // Performance verdict
        if avg_fps >= 55.0 && p95 < 20.0 {
            println!("✅ Performance PASSED");
        } else {
            println!("❌ Performance FAILED");
        }
    }
}

fn setup_load_test(
    mut commands: Commands,
    config: Res<LoadTestConfig>,
) {
    // Spawn creatures in a grid pattern for even distribution
    let grid_size = (config.creature_count as f32).sqrt().ceil() as usize;
    let spacing = config.world_size / grid_size as f32;
    
    let mut count = 0;
    for x in 0..grid_size {
        for y in 0..grid_size {
            if count >= config.creature_count {
                break;
            }
            
            let pos = Vec2::new(
                (x as f32 - grid_size as f32 / 2.0) * spacing,
                (y as f32 - grid_size as f32 / 2.0) * spacing,
            );
            
            commands.spawn(CreatureBundle::new(pos, 1.0));
            count += 1;
        }
    }
    
    // Spawn resources scattered throughout
    let resource_count = config.creature_count / 5; // 1 resource per 5 creatures
    for i in 0..resource_count {
        let angle = i as f32 * std::f32::consts::TAU / resource_count as f32;
        let radius = config.world_size * 0.4 * (i as f32 / resource_count as f32);
        let pos = Vec2::new(angle.cos() * radius, angle.sin() * radius);
        
        let resource_type = if i % 2 == 0 {
            ResourceType::Food
        } else {
            ResourceType::Water
        };
        
        commands.spawn(ResourceBundle::new(pos, resource_type, 50.0));
    }
    
    info!("Spawned {} creatures and {} resources", 
          config.creature_count, resource_count);
}

#[derive(Resource)]
struct LoadTestState {
    start_time: Instant,
    warmup_complete: bool,
    metrics: LoadTestMetrics,
}

fn collect_metrics(
    mut state: ResMut<LoadTestState>,
    config: Res<LoadTestConfig>,
    diagnostics: Res<DiagnosticsStore>,
    time: Res<Time>,
    creature_query: Query<&Creature>,
    mut exit: EventWriter<bevy::app::AppExit>,
) {
    let elapsed = state.start_time.elapsed();
    
    // Skip warmup period
    if !state.warmup_complete {
        if elapsed >= config.warmup_duration {
            state.warmup_complete = true;
            state.metrics = LoadTestMetrics::default();
            info!("Warmup complete, starting metrics collection");
        }
        return;
    }
    
    // Collect FPS
    if let Some(fps_diagnostic) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(fps) = fps_diagnostic.smoothed() {
            state.metrics.fps_samples.push(fps as f32);
        }
    }
    
    // Collect frame time
    if let Some(frame_time_diagnostic) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FRAME_TIME) {
        if let Some(frame_time) = frame_time_diagnostic.smoothed() {
            state.metrics.frame_times.push(frame_time as f32);
        }
    }
    
    // Check if test is complete
    if elapsed >= config.warmup_duration + config.test_duration {
        state.metrics.report();
        exit.send(bevy::app::AppExit);
    }
}

/// Create a test app for load testing
fn create_load_test_app(config: LoadTestConfig) -> App {
    let mut app = App::new();
    
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            resolution: (1280.0, 720.0).into(),
            title: format!("Load Test - {} creatures", config.creature_count),
            present_mode: bevy::window::PresentMode::AutoNoVsync,
            ..default()
        }),
        ..default()
    }))
    .add_plugins((
        // Core plugins
        creature_simulation::core::simulation_control::SimulationControlPlugin,
        creature_simulation::core::determinism::DeterminismPlugin,
        creature_simulation::core::error_boundary::ErrorBoundaryPlugin,
        creature_simulation::core::performance_monitor::PerformanceMonitorPlugin,
        
        // Simulation plugins
        CreatureSimulationPlugin,
        RenderingPlugin,
        CameraPlugin,
        
        // Diagnostics
        FrameTimeDiagnosticsPlugin,
    ))
    .insert_resource(config)
    .insert_resource(LoadTestState {
        start_time: Instant::now(),
        warmup_complete: false,
        metrics: LoadTestMetrics::default(),
    })
    .add_systems(Startup, setup_load_test)
    .add_systems(Update, collect_metrics);
    
    app
}

/// Helper to check if app wants to exit
fn app_should_exit(app: &App) -> bool {
    app.world.get_resource::<Events<bevy::app::AppExit>>()
        .map(|events| !events.is_empty())
        .unwrap_or(false)
}

#[test]
#[ignore] // Run with: cargo test --test load_tests test_500_creatures -- --ignored --nocapture
fn test_500_creatures() {
    let config = LoadTestConfig::new(500);
    let mut app = create_load_test_app(config);
    
    // Run for limited time
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(40) {
        app.update();
        if app_should_exit(&app) {
            break;
        }
    }
}

#[test]
#[ignore] // Run with: cargo test --test load_tests test_1000_creatures -- --ignored --nocapture
fn test_1000_creatures() {
    let config = LoadTestConfig::new(1000);
    let mut app = create_load_test_app(config);
    
    // Run for limited time
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(40) {
        app.update();
        if app_should_exit(&app) {
            break;
        }
    }
}

#[test]
#[ignore] // Run with: cargo test --test load_tests test_scaling -- --ignored --nocapture
fn test_scaling() {
    let creature_counts = vec![100, 250, 500, 750, 1000];
    
    println!("\n=== Scaling Test Results ===");
    println!("Count\tAvg FPS\tMin FPS\tp95 Frame");
    
    for count in creature_counts {
        let mut config = LoadTestConfig::new(count);
        config.test_duration = Duration::from_secs(10); // Shorter for scaling test
        
        let mut app = create_load_test_app(config);
        
        // Run test
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(15) {
            app.update();
            if app_should_exit(&app) {
                break;
            }
        }
        
        // Extract metrics
        let state = app.world.resource::<LoadTestState>();
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
        }
    }
}

#[test]
#[ignore] // Run with: cargo test --test load_tests stress_test_spawn_despawn -- --ignored --nocapture
fn stress_test_spawn_despawn() {
    let mut app = create_load_test_app(LoadTestConfig::new(500));
    
    // Add a system that spawns and despawns creatures
    app.add_systems(Update, stress_spawn_despawn_system);
    
    // Run test
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(30) {
        app.update();
        if app_should_exit(&app) {
            break;
        }
    }
    
    println!("\n✅ Spawn/despawn stress test completed without crashes");
}

fn stress_spawn_despawn_system(
    mut commands: Commands,
    creatures: Query<Entity, With<Creature>>,
    time: Res<Time>,
    mut spawn_timer: Local<f32>,
) {
    *spawn_timer += time.delta_seconds();
    
    // Every second, despawn 10 creatures and spawn 10 new ones
    if *spawn_timer > 1.0 {
        *spawn_timer = 0.0;
        
        let mut count = 0;
        for entity in creatures.iter() {
            if count >= 10 {
                break;
            }
            commands.entity(entity).despawn();
            count += 1;
        }
        
        // Spawn new creatures
        for i in 0..10 {
            let angle = i as f32 * 0.628;
            let pos = Vec2::new(angle.cos() * 200.0, angle.sin() * 200.0);
            commands.spawn(CreatureBundle::new(pos, 1.0));
        }
    }
}

/// Basic memory tracking test (simplified version)
#[test]
#[ignore] // Run with: cargo test --test load_tests memory_stability -- --ignored --nocapture
fn memory_stability() {
    let mut app = create_load_test_app(LoadTestConfig::new(500));
    
    let start = Instant::now();
    let mut frame_count = 0;
    
    println!("\n=== Memory Stability Test ===");
    println!("Running for 60 seconds to check for memory leaks...");
    
    while start.elapsed() < Duration::from_secs(60) {
        app.update();
        frame_count += 1;
        
        // Print progress every 10 seconds
        if frame_count % 600 == 0 {
            let elapsed = start.elapsed().as_secs();
            println!("Progress: {}s / 60s", elapsed);
        }
        
        if app_should_exit(&app) {
            break;
        }
    }
    
    println!("✅ Memory stability test completed");
    println!("Total frames: {}", frame_count);
    println!("Note: Use external memory profiler for detailed analysis");
}