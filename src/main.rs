use bevy::log::info;
use creature_simulation::simulation::{Creature, Resource, ResourceType};
use creature_simulation::systems::Simulation;
use creature_simulation::{Result, Vec2};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    info!("Starting creature simulation...");

    // Set up graceful shutdown
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        info!("Received interrupt signal, shutting down gracefully...");
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    // Create simulation with 500x500 world
    let mut sim = Simulation::with_bounds(500.0, 500.0);

    // Spawn some creatures
    info!("Spawning creatures...");
    for i in 0..10 {
        let entity = sim.world.entities.create();
        let x = 100.0 + (i as f32 % 5.0) * 50.0;
        let y = 100.0 + (i as f32 / 5.0).floor() * 50.0;
        let creature = Creature::new(entity, Vec2::new(x, y));
        sim.world.creatures.insert(entity, creature);
        sim.world.spatial_grid.insert(entity, Vec2::new(x, y));
    }

    // Spawn some resources
    info!("Spawning resources...");
    for i in 0..5 {
        // Food - place near creatures
        let entity = sim.world.entities.create();
        let x = 120.0 + (i as f32) * 40.0;
        let y = 120.0 + (i as f32 % 2.0) * 40.0;
        let food = Resource::new(entity, Vec2::new(x, y), ResourceType::Food);
        sim.world.resources.insert(entity, food);
        sim.world.spatial_grid.insert(entity, Vec2::new(x, y));

        // Water - place near creatures
        let entity = sim.world.entities.create();
        let x = 100.0 + (i as f32) * 40.0;
        let y = 160.0 + (i as f32 % 2.0) * 40.0;
        let water = Resource::new(entity, Vec2::new(x, y), ResourceType::Water);
        sim.world.resources.insert(entity, water);
        sim.world.spatial_grid.insert(entity, Vec2::new(x, y));
    }

    info!(
        "Starting simulation with {} creatures and {} resources",
        sim.world.creature_count(),
        sim.world.resource_count()
    );

    // Run simulation for a few seconds
    let mut last_frame = Instant::now();
    let mut total_frames = 0;
    let start_time = Instant::now();

    // Run for 5 seconds or until interrupted
    while running.load(Ordering::SeqCst) && start_time.elapsed() < Duration::from_secs(5) {
        let now = Instant::now();
        let dt = now.duration_since(last_frame).as_secs_f32();
        last_frame = now;

        // Update simulation
        let steps = sim.update(dt);
        total_frames += steps;

        // Log stats every second
        if total_frames % 60 == 0 {
            info!(
                "Frame {}: {} creatures alive, {} FPS",
                total_frames,
                sim.world.creatures.values().filter(|c| c.is_alive()).count(),
                1.0 / dt
            );
        }

        // Sleep to maintain roughly 60 FPS
        thread::sleep(Duration::from_millis(16));
    }

    info!("Simulation complete!");
    info!("Total frames: {}", total_frames);
    info!("Average FPS: {:.1}", total_frames as f32 / 5.0);
    info!(
        "Final stats: {} creatures alive, {} resources",
        sim.world.creatures.values().filter(|c| c.is_alive()).count(),
        sim.world.resource_count()
    );

    Ok(())
}
