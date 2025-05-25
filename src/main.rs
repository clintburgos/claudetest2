use bevy::log::info;
use creature_simulation::config::demo::*;
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
    .expect("Failed to set Ctrl-C handler: Another handler may already be registered or system permissions may be insufficient");

    // Create simulation with default world size
    let mut sim = Simulation::with_bounds(DEFAULT_WORLD_WIDTH, DEFAULT_WORLD_HEIGHT);

    // Spawn some creatures
    info!("Spawning creatures...");
    for i in 0..DEMO_CREATURE_COUNT {
        let entity = sim.world.entities.create();
        let x = CREATURE_GRID_START_X + (i as f32 % CREATURES_PER_ROW) * CREATURE_GRID_SIZE;
        let y = CREATURE_GRID_START_Y + (i as f32 / CREATURES_PER_ROW).floor() * CREATURE_GRID_SIZE;
        let creature = Creature::new(entity, Vec2::new(x, y));
        sim.world.creatures.insert(entity, creature);
        sim.world.spatial_grid.insert(entity, Vec2::new(x, y));
    }

    // Spawn some resources
    info!("Spawning resources...");
    for i in 0..DEMO_RESOURCE_COUNT {
        // Food - place near creatures
        let entity = sim.world.entities.create();
        let x = FOOD_X_OFFSET + (i as f32) * RESOURCE_SPACING;
        let y = FOOD_Y_OFFSET + (i as f32 % RESOURCE_Y_MODULO) * RESOURCE_SPACING;
        let food = Resource::new(entity, Vec2::new(x, y), ResourceType::Food);
        sim.world.resources.insert(entity, food);
        sim.world.spatial_grid.insert(entity, Vec2::new(x, y));

        // Water - place near creatures
        let entity = sim.world.entities.create();
        let x = WATER_X_OFFSET + (i as f32) * RESOURCE_SPACING;
        let y = WATER_Y_OFFSET + (i as f32 % RESOURCE_Y_MODULO) * RESOURCE_SPACING;
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

    // Run for demo duration or until interrupted
    while running.load(Ordering::SeqCst) && start_time.elapsed() < Duration::from_secs(DEMO_DURATION_SECONDS) {
        let now = Instant::now();
        let dt = now.duration_since(last_frame).as_secs_f32();
        last_frame = now;

        // Update simulation
        let steps = sim.update(dt);
        total_frames += steps;

        // Log stats every second
        if total_frames % TARGET_FPS as u32 == 0 {
            info!(
                "Frame {}: {} creatures alive, {} FPS",
                total_frames,
                sim.world.creatures.values().filter(|c| c.is_alive()).count(),
                1.0 / dt
            );
        }

        // Sleep to maintain target FPS
        thread::sleep(Duration::from_millis(FRAME_SLEEP_MS));
    }

    info!("Simulation complete!");
    info!("Total frames: {}", total_frames);
    info!("Average FPS: {:.1}", total_frames as f32 / DEMO_DURATION_SECONDS as f32);
    info!(
        "Final stats: {} creatures alive, {} resources",
        sim.world.creatures.values().filter(|c| c.is_alive()).count(),
        sim.world.resource_count()
    );

    Ok(())
}
