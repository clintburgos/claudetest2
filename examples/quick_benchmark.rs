//! Quick headless benchmark to verify 500 creature performance

use bevy::app::ScheduleRunnerPlugin;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use creature_simulation::{components::*, plugins::*};
use std::time::{Duration, Instant};

fn main() {
    App::new()
        // Minimal plugins for headless benchmarking
        .add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(
            Duration::from_secs_f64(1.0 / 60.0),
        )))
        .add_plugins(FrameTimeDiagnosticsPlugin)
        // Our simulation plugins
        .add_plugins(CreatureSimulationPlugin)
        // Benchmark systems
        .add_systems(Startup, spawn_benchmark_entities)
        .add_systems(Update, monitor_performance)
        .insert_resource(BenchmarkData::default())
        .run();
}

#[derive(Resource, Default)]
struct BenchmarkData {
    start_time: Option<Instant>,
    frame_count: u32,
    warmup_frames: u32,
    measurement_frames: Vec<f32>,
}

fn spawn_benchmark_entities(mut commands: Commands, mut benchmark: ResMut<BenchmarkData>) {
    println!("Spawning 500 creatures for benchmark...");

    // Spawn 500 creatures
    for i in 0..500 {
        let angle = (i as f32 / 500.0) * std::f32::consts::TAU;
        let radius = 200.0 + (i as f32 % 10.0) * 20.0;
        let position = Vec2::new(angle.cos() * radius, angle.sin() * radius);

        let creature_type = match i % 3 {
            0 => CreatureType::Herbivore,
            1 => CreatureType::Carnivore,
            _ => CreatureType::Omnivore,
        };

        commands.spawn(CreatureBundle {
            creature: Creature,
            creature_type,
            position: Position(position),
            velocity: Velocity(Vec2::ZERO),
            health: Health::new(100.0),
            needs: Needs {
                hunger: 0.3 + (i as f32 % 5.0) * 0.1,
                thirst: 0.2 + (i as f32 % 7.0) * 0.1,
                energy: 0.8,
                social: 0.5,
            },
            state: CreatureState::Idle,
            age: Age(0.0),
            size: Size(1.0),
            genetics: Genetics::default(),
            max_speed: MaxSpeed(50.0),
            decision_timer: DecisionTimer::default(),
            current_target: CurrentTarget::None,
        });
    }

    // Spawn 100 resources
    for i in 0..100 {
        let x = (i % 10) as f32 * 100.0 - 450.0;
        let y = (i / 10) as f32 * 100.0 - 450.0;

        commands.spawn(ResourceBundle {
            resource: ResourceMarker,
            position: Position(Vec2::new(x, y)),
            resource_type: ResourceTypeComponent(if i % 2 == 0 {
                ResourceType::Food
            } else {
                ResourceType::Water
            }),
            amount: ResourceAmount::new(100.0),
        });
    }

    benchmark.start_time = Some(Instant::now());
    println!("Benchmark started. Warming up...");
}

fn monitor_performance(
    mut benchmark: ResMut<BenchmarkData>,
    time: Res<Time>,
    _diagnostics: Res<DiagnosticsStore>,
    mut app_exit: EventWriter<bevy::app::AppExit>,
) {
    if benchmark.start_time.is_none() {
        return;
    }

    let frame_time = time.delta_seconds();
    let fps = 1.0 / frame_time;

    benchmark.frame_count += 1;

    // Warmup for 60 frames
    if benchmark.frame_count <= 60 {
        benchmark.warmup_frames = benchmark.frame_count;
        if benchmark.frame_count == 60 {
            println!("Warmup complete. Starting measurement...");
        }
        return;
    }

    // Measure for 300 frames after warmup
    benchmark.measurement_frames.push(fps);

    // Print progress every 60 frames
    if benchmark.measurement_frames.len() % 60 == 0 {
        println!("Measured {} frames...", benchmark.measurement_frames.len());
    }

    // Complete after 300 measurement frames
    if benchmark.measurement_frames.len() >= 300 {
        let avg_fps = benchmark.measurement_frames.iter().sum::<f32>()
            / benchmark.measurement_frames.len() as f32;
        let min_fps = benchmark.measurement_frames.iter().fold(f32::MAX, |a, &b| a.min(b));
        let max_fps = benchmark.measurement_frames.iter().fold(0.0f32, |a, &b| a.max(b));

        // Calculate percentiles
        let mut sorted = benchmark.measurement_frames.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let p95 = sorted[(sorted.len() as f32 * 0.95) as usize];
        let p99 = sorted[(sorted.len() as f32 * 0.99) as usize];

        println!("\n=== BENCHMARK RESULTS ===");
        println!("500 Creatures + 100 Resources");
        println!(
            "Measured {} frames after {} warmup frames",
            benchmark.measurement_frames.len(),
            benchmark.warmup_frames
        );
        println!("\nFPS Statistics:");
        println!("  Average: {:.1} FPS", avg_fps);
        println!("  Minimum: {:.1} FPS", min_fps);
        println!("  Maximum: {:.1} FPS", max_fps);
        println!("  95th percentile: {:.1} FPS", p95);
        println!("  99th percentile: {:.1} FPS", p99);
        println!("\nTarget: 60 FPS");
        println!(
            "Result: {}",
            if avg_fps >= 60.0 { "PASS ✓" } else { "FAIL ✗" }
        );

        app_exit.send(bevy::app::AppExit);
    }
}
