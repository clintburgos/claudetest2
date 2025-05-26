//! Benchmark example to verify performance with 500+ creatures

use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use creature_simulation::{components::*, plugins::*};
use std::time::{Duration, Instant};

fn main() {
    App::new()
        // Core Bevy plugins with window for monitoring
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Creature Simulation - 500 Creature Benchmark".to_string(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        // Diagnostics
        .add_plugins(FrameTimeDiagnosticsPlugin)
        // Third-party plugins
        .add_plugins(EguiPlugin)
        // Our custom plugins
        .add_plugins((
            CreatureSimulationPlugin,
            CameraPlugin,
            RenderingPlugin,
            SelectionPlugin,
            UiEguiPlugin,
            DebugPlugin,
        ))
        // Benchmark systems
        .add_systems(Startup, spawn_benchmark_entities)
        .add_systems(Update, (monitor_performance, check_benchmark_completion))
        .insert_resource(BenchmarkData::default())
        .run();
}

#[derive(Resource, Default)]
struct BenchmarkData {
    start_time: Option<Instant>,
    frame_count: u32,
    total_frame_time: Duration,
    min_fps: f32,
    max_fps: f32,
    creature_count: u32,
    resource_count: u32,
}

fn spawn_benchmark_entities(mut commands: Commands, mut benchmark: ResMut<BenchmarkData>) {
    info!("Spawning benchmark entities...");

    // Spawn 500 creatures in a grid pattern
    let grid_size = 23; // ~529 creatures
    let spacing = 40.0;
    let offset = -(grid_size as f32 * spacing) / 2.0;

    let mut creature_count = 0;
    for x in 0..grid_size {
        for y in 0..grid_size {
            if creature_count >= 500 {
                break;
            }

            let position = Vec2::new(offset + x as f32 * spacing, offset + y as f32 * spacing);

            // Mix of creature types
            let creature_type = match creature_count % 3 {
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
                    hunger: rand::random::<f32>() * 0.5,
                    thirst: rand::random::<f32>() * 0.5,
                    energy: 1.0 - rand::random::<f32>() * 0.3,
                    social: rand::random::<f32>() * 0.5,
                },
                state: CreatureState::Idle,
                age: Age(0.0),
                size: Size(0.8 + rand::random::<f32>() * 0.4),
                max_speed: MaxSpeed(40.0 + rand::random::<f32>() * 20.0),
                decision_timer: DecisionTimer::default(),
                current_target: CurrentTarget::None,
            });

            creature_count += 1;
        }
    }

    // Spawn resources in a scattered pattern
    let resource_count = 150;
    for i in 0..resource_count {
        let angle = (i as f32 / resource_count as f32) * std::f32::consts::TAU;
        let radius = 100.0 + (i as f32 % 10.0) * 50.0;
        let position = Vec2::new(angle.cos() * radius, angle.sin() * radius);

        let resource_type = if i % 2 == 0 { ResourceType::Food } else { ResourceType::Water };

        commands.spawn(ResourceBundle {
            resource: ResourceMarker,
            position: Position(position),
            resource_type: ResourceTypeComponent(resource_type),
            amount: ResourceAmount::new(100.0),
        });
    }

    benchmark.creature_count = creature_count;
    benchmark.resource_count = resource_count as u32;
    benchmark.start_time = Some(Instant::now());
    benchmark.min_fps = f32::MAX;
    benchmark.max_fps = 0.0;

    info!(
        "Spawned {} creatures and {} resources",
        creature_count, resource_count
    );
}

fn monitor_performance(
    diagnostics: Res<DiagnosticsStore>,
    mut benchmark: ResMut<BenchmarkData>,
    time: Res<Time>,
) {
    if benchmark.start_time.is_none() {
        return;
    }

    benchmark.frame_count += 1;
    benchmark.total_frame_time += time.delta();

    if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(value) = fps.smoothed() {
            benchmark.min_fps = benchmark.min_fps.min(value as f32);
            benchmark.max_fps = benchmark.max_fps.max(value as f32);

            // Log performance every 60 frames
            if benchmark.frame_count % 60 == 0 {
                let avg_fps =
                    benchmark.frame_count as f32 / benchmark.total_frame_time.as_secs_f32();
                info!(
                    "Frame {}: Current FPS: {:.1}, Avg FPS: {:.1}, Min: {:.1}, Max: {:.1}",
                    benchmark.frame_count, value, avg_fps, benchmark.min_fps, benchmark.max_fps
                );
            }
        }
    }
}

fn check_benchmark_completion(
    benchmark: Res<BenchmarkData>,
    mut app_exit: EventWriter<bevy::app::AppExit>,
    creatures: Query<&CreatureState, With<Creature>>,
) {
    if let Some(start_time) = benchmark.start_time {
        let elapsed = start_time.elapsed();

        // Run for 60 seconds
        if elapsed > Duration::from_secs(60) {
            let avg_fps = benchmark.frame_count as f32 / benchmark.total_frame_time.as_secs_f32();

            // Count active creatures
            let mut state_counts = std::collections::HashMap::new();
            for state in creatures.iter() {
                *state_counts.entry(format!("{:?}", state)).or_insert(0) += 1;
            }

            println!("\n=== BENCHMARK COMPLETE ===");
            println!("Duration: {:.1}s", elapsed.as_secs_f32());
            println!("Total frames: {}", benchmark.frame_count);
            println!("Average FPS: {:.1}", avg_fps);
            println!("Min FPS: {:.1}", benchmark.min_fps);
            println!("Max FPS: {:.1}", benchmark.max_fps);
            println!("Creatures: {}", benchmark.creature_count);
            println!("Resources: {}", benchmark.resource_count);
            println!("\nCreature states:");
            for (state, count) in state_counts {
                println!("  {}: {}", state, count);
            }
            println!("\n=== Performance Target: 60 FPS ===");
            println!(
                "Result: {}",
                if avg_fps >= 60.0 { "PASS ✓" } else { "FAIL ✗" }
            );

            app_exit.send(bevy::app::AppExit);
        }
    }
}
