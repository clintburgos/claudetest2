use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use creature_simulation::{Vec2, core::*, simulation::*, systems::*};
use rand::Rng;

fn spawn_creatures(sim: &mut Simulation, count: usize) {
    let mut rng = rand::thread_rng();
    
    for _ in 0..count {
        let entity = sim.world.entities.create();
        let position = Vec2::new(
            rng.gen_range(0.0..1000.0),
            rng.gen_range(0.0..1000.0)
        );
        let mut creature = Creature::new(entity, position);
        
        // Randomize needs
        creature.needs.hunger = rng.gen_range(0.0..1.0);
        creature.needs.thirst = rng.gen_range(0.0..1.0);
        creature.needs.energy = rng.gen_range(0.3..1.0);
        
        sim.world.creatures.insert(entity, creature);
        sim.world.spatial_grid.insert(entity, position);
    }
}

fn spawn_resources(sim: &mut Simulation, count: usize) {
    let mut rng = rand::thread_rng();
    
    for _ in 0..count {
        let entity = sim.world.entities.create();
        let position = Vec2::new(
            rng.gen_range(0.0..1000.0),
            rng.gen_range(0.0..1000.0)
        );
        let resource_type = if rng.gen_bool(0.5) {
            ResourceType::Food
        } else {
            ResourceType::Water
        };
        
        let resource = Resource::new(entity, position, resource_type);
        sim.world.resources.insert(entity, resource);
        sim.world.spatial_grid.insert(entity, position);
    }
}

fn benchmark_simulation_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("simulation_update");
    
    for creature_count in [100, 250, 500].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(creature_count),
            creature_count,
            |b, &count| {
                let mut sim = Simulation::with_bounds(1000.0, 1000.0);
                spawn_creatures(&mut sim, count);
                spawn_resources(&mut sim, count / 5); // 20% resources
                
                b.iter(|| {
                    sim.update(black_box(1.0 / 60.0));
                });
            }
        );
    }
    
    group.finish();
}

fn benchmark_spatial_queries(c: &mut Criterion) {
    let mut sim = Simulation::with_bounds(1000.0, 1000.0);
    spawn_creatures(&mut sim, 500);
    
    let mut rng = rand::thread_rng();
    
    c.bench_function("spatial_query_500_creatures", |b| {
        b.iter(|| {
            let pos = Vec2::new(
                rng.gen_range(0.0..1000.0),
                rng.gen_range(0.0..1000.0)
            );
            let results = sim.world.spatial_grid.query_radius(black_box(pos), black_box(50.0));
            black_box(results);
        });
    });
}

fn benchmark_decision_making(c: &mut Criterion) {
    let mut sim = Simulation::with_bounds(1000.0, 1000.0);
    spawn_creatures(&mut sim, 500);
    spawn_resources(&mut sim, 100);
    
    let mut decision_system = DecisionSystem::new();
    
    c.bench_function("decision_system_500_creatures", |b| {
        b.iter(|| {
            decision_system.update(&mut sim.world);
        });
    });
}

fn benchmark_movement(c: &mut Criterion) {
    let mut sim = Simulation::with_bounds(1000.0, 1000.0);
    spawn_creatures(&mut sim, 500);
    
    // Set all creatures to moving state
    for (_, creature) in &mut sim.world.creatures {
        creature.start_moving(Vec2::new(500.0, 500.0));
    }
    
    let mut movement_system = MovementSystem::new();
    
    c.bench_function("movement_system_500_creatures", |b| {
        b.iter(|| {
            movement_system.update(&mut sim.world, black_box(1.0 / 60.0));
        });
    });
}

fn benchmark_memory_usage(c: &mut Criterion) {
    c.bench_function("memory_per_creature", |b| {
        b.iter(|| {
            let entity = Entity::new(1);
            let creature = Creature::new(entity, Vec2::new(0.0, 0.0));
            black_box(std::mem::size_of_val(&creature));
        });
    });
    
    c.bench_function("memory_world_500_creatures", |b| {
        b.iter(|| {
            let mut sim = Simulation::with_bounds(1000.0, 1000.0);
            spawn_creatures(&mut sim, 500);
            spawn_resources(&mut sim, 100);
            black_box(std::mem::size_of_val(&sim.world));
        });
    });
}

criterion_group!(
    benches, 
    benchmark_simulation_update,
    benchmark_spatial_queries,
    benchmark_decision_making,
    benchmark_movement,
    benchmark_memory_usage
);
criterion_main!(benches);