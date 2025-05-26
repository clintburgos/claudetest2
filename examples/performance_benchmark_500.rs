//! Performance benchmark for verifying 500 creatures at 60 FPS
//! 
//! Run with: cargo run --release --example performance_benchmark_500

use bevy::prelude::*;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::window::{PresentMode, WindowMode};
use creature_simulation::prelude::*;
use creature_simulation::plugins::*;
use std::time::{Duration, Instant};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Performance Benchmark - 500 Creatures".to_string(),
                resolution: (1280.0, 720.0).into(),
                present_mode: PresentMode::AutoVsync,
                mode: WindowMode::Windowed,
                ..default()
            }),
            ..default()
        }))
        .add_plugins((
            FrameTimeDiagnosticsPlugin,
            LogDiagnosticsPlugin::default(),
        ))
        .add_plugins((
            CreatureSimulationPlugin,
            SpatialPlugin,
            SpawnPlugin,
            SimulationPlugin,
            RenderingPlugin,
            CameraPlugin,
            SelectionPlugin,
            UiEguiPlugin,
            DebugPlugin,
        ))
        .insert_resource(BenchmarkState::default())
        .add_systems(Startup, setup_benchmark)
        .add_systems(Update, (
            benchmark_monitor,
            spawn_benchmark_creatures,
        ))
        .run();
}

#[derive(Resource, Default)]
struct BenchmarkState {
    start_time: Option<Instant>,
    warmup_complete: bool,
    test_duration: Duration,
    creatures_spawned: usize,
    target_creatures: usize,
    
    // Performance metrics
    frame_count: u32,
    total_frame_time: f64,
    min_fps: f32,
    max_fps: f32,
    below_60_count: u32,
}

impl BenchmarkState {
    fn new() -> Self {
        Self {
            test_duration: Duration::from_secs(60), // 1 minute test
            target_creatures: 500,
            min_fps: f32::INFINITY,
            max_fps: 0.0,
            ..default()
        }
    }
}

fn setup_benchmark(
    mut commands: Commands,
    mut benchmark: ResMut<BenchmarkState>,
) {
    info!("=== PERFORMANCE BENCHMARK: 500 CREATURES AT 60 FPS ===");
    info!("Target: {} creatures", benchmark.target_creatures);
    info!("Test duration: {} seconds", benchmark.test_duration.as_secs());
    info!("Warming up...");
    
    *benchmark = BenchmarkState::new();
    
    // Spawn initial resources
    let resource_count = 200;
    for i in 0..resource_count {
        let angle = (i as f32 / resource_count as f32) * std::f32::consts::TAU;
        let radius = 200.0 + (i as f32 % 50.0) * 4.0;
        let pos = Vec2::new(angle.cos() * radius, angle.sin() * radius);
        
        commands.spawn((
            Position(pos),
            ResourceMarker,
            ResourceTypeComponent(if i % 2 == 0 { 
                creature_simulation::components::ResourceType::Food 
            } else { 
                creature_simulation::components::ResourceType::Water 
            }),
            ResourceAmount::new(100.0),
        ));
    }
}

fn spawn_benchmark_creatures(
    mut commands: Commands,
    mut benchmark: ResMut<BenchmarkState>,
    _time: Res<Time>,
) {
    // Spawn creatures gradually to avoid startup spike
    if benchmark.creatures_spawned < benchmark.target_creatures {
        let spawn_rate = 10; // creatures per frame
        let to_spawn = (benchmark.target_creatures - benchmark.creatures_spawned).min(spawn_rate);
        
        for i in 0..to_spawn {
            let angle = (benchmark.creatures_spawned as f32 / benchmark.target_creatures as f32) * std::f32::consts::TAU;
            let radius = 100.0 + (i as f32 * 10.0);
            let pos = Vec2::new(angle.cos() * radius, angle.sin() * radius);
            
            let creature_type = match benchmark.creatures_spawned % 3 {
                0 => CreatureType::Herbivore,
                1 => CreatureType::Carnivore,
                _ => CreatureType::Omnivore,
            };
            
            commands.spawn((
                Creature,
                creature_type,
                Position(pos),
                Velocity(Vec2::ZERO),
                Health::new(100.0),
                Needs::default(),
                Size(1.0 + (i as f32 * 0.1)),
                MaxSpeed(50.0),
                Age(0.0),
                CreatureState::Idle,
                DecisionTimer::default(),
                CurrentTarget::None,
            ));
            
            benchmark.creatures_spawned += 1;
        }
        
        if benchmark.creatures_spawned >= benchmark.target_creatures {
            info!("All {} creatures spawned. Starting benchmark...", benchmark.target_creatures);
            benchmark.start_time = Some(Instant::now());
        }
    }
}

fn benchmark_monitor(
    mut benchmark: ResMut<BenchmarkState>,
    diagnostics: Res<DiagnosticsStore>,
    time: Res<Time>,
    creature_query: Query<&Creature>,
    mut app_exit_events: EventWriter<bevy::app::AppExit>,
) {
    // Wait for warmup
    if !benchmark.warmup_complete {
        if time.elapsed_seconds() > 5.0 {
            benchmark.warmup_complete = true;
            info!("Warmup complete. Benchmark starting...");
        }
        return;
    }
    
    // Check if benchmark is complete
    if let Some(start_time) = benchmark.start_time {
        if start_time.elapsed() >= benchmark.test_duration {
            // Print results
            let avg_fps = if benchmark.frame_count > 0 {
                (benchmark.frame_count as f64) / benchmark.total_frame_time
            } else {
                0.0
            };
            
            let below_60_percentage = (benchmark.below_60_count as f32 / benchmark.frame_count as f32) * 100.0;
            
            info!("=== BENCHMARK RESULTS ===");
            info!("Creatures: {}", creature_query.iter().count());
            info!("Average FPS: {:.1}", avg_fps);
            info!("Min FPS: {:.1}", benchmark.min_fps);
            info!("Max FPS: {:.1}", benchmark.max_fps);
            info!("Frames below 60 FPS: {} ({:.1}%)", benchmark.below_60_count, below_60_percentage);
            
            if avg_fps >= 60.0 && below_60_percentage < 5.0 {
                info!("✅ PASS: Achieved 60+ FPS with 500 creatures!");
            } else {
                warn!("❌ FAIL: Did not maintain 60 FPS with 500 creatures");
            }
            
            app_exit_events.send(bevy::app::AppExit);
            return;
        }
    }
    
    // Record FPS metrics
    if let Some(fps_diagnostic) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(fps_value) = fps_diagnostic.smoothed() {
            benchmark.frame_count += 1;
            benchmark.total_frame_time += time.delta_seconds_f64();
            
            benchmark.min_fps = benchmark.min_fps.min(fps_value as f32);
            benchmark.max_fps = benchmark.max_fps.max(fps_value as f32);
            
            if fps_value < 60.0 {
                benchmark.below_60_count += 1;
            }
            
            // Log every 5 seconds
            if benchmark.frame_count % 300 == 0 {
                info!("Current FPS: {:.1}, Creatures: {}", fps_value, creature_query.iter().count());
            }
        }
    }
}