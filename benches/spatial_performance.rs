//! Benchmarks for spatial system performance
//!
//! Compares the original spatial grid with the optimized spatial hash grid
//! to measure performance improvements from Phase 1.3.

use bevy::prelude::{Entity as BevyEntity, Vec2};
use creature_simulation::core::{Entity as CoreEntity, SpatialGrid, SpatialHashGrid};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

fn create_core_entities(count: usize) -> Vec<(CoreEntity, Vec2)> {
    (0..count)
        .map(|i| {
            let entity = CoreEntity::new(i as u32);
            let position = Vec2::new((i as f32 * 13.0) % 1000.0, (i as f32 * 17.0) % 1000.0);
            (entity, position)
        })
        .collect()
}

fn create_bevy_entities(count: usize) -> Vec<(BevyEntity, Vec2)> {
    (0..count)
        .map(|i| {
            let entity = BevyEntity::from_raw(i as u32);
            let position = Vec2::new((i as f32 * 13.0) % 1000.0, (i as f32 * 17.0) % 1000.0);
            (entity, position)
        })
        .collect()
}

fn bench_spatial_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("spatial_insert");

    for size in [100, 1000, 5000, 10000].iter() {
        let core_entities = create_core_entities(*size);
        let bevy_entities = create_bevy_entities(*size);

        // Benchmark original spatial grid
        group.bench_with_input(BenchmarkId::new("original", size), size, |b, _| {
            b.iter(|| {
                let mut grid = SpatialGrid::new(10.0);
                for (entity, position) in &core_entities {
                    grid.insert(*entity, *position);
                }
            });
        });

        // Benchmark optimized spatial hash grid
        group.bench_with_input(BenchmarkId::new("optimized", size), size, |b, _| {
            b.iter(|| {
                let grid = SpatialHashGrid::new(10.0);
                for (entity, position) in &bevy_entities {
                    grid.update_entity(*entity, *position);
                }
            });
        });
    }

    group.finish();
}

fn bench_spatial_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("spatial_query");

    for size in [1000, 5000, 10000].iter() {
        let core_entities = create_core_entities(*size);
        let bevy_entities = create_bevy_entities(*size);

        // Setup original grid
        let mut original_grid = SpatialGrid::new(10.0);
        for (entity, position) in &core_entities {
            original_grid.insert(*entity, *position);
        }

        // Setup optimized grid
        let optimized_grid = SpatialHashGrid::new(10.0);
        for (entity, position) in &bevy_entities {
            optimized_grid.update_entity(*entity, *position);
        }

        // Benchmark queries
        group.bench_with_input(BenchmarkId::new("original", size), size, |b, _| {
            b.iter(|| {
                let results =
                    original_grid.query_radius(black_box(Vec2::new(500.0, 500.0)), black_box(50.0));
                black_box(results);
            });
        });

        group.bench_with_input(BenchmarkId::new("optimized", size), size, |b, _| {
            b.iter(|| {
                let results = optimized_grid
                    .query_radius(black_box(Vec2::new(500.0, 500.0)), black_box(50.0));
                black_box(results);
            });
        });
    }

    group.finish();
}

fn bench_spatial_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("spatial_update");

    let size = 5000;
    let core_entities = create_core_entities(size);
    let bevy_entities = create_bevy_entities(size);

    // Setup grids
    let mut original_grid = SpatialGrid::new(10.0);
    for (entity, position) in &core_entities {
        original_grid.insert(*entity, *position);
    }

    let optimized_grid = SpatialHashGrid::new(10.0);
    for (entity, position) in &bevy_entities {
        optimized_grid.update_entity(*entity, *position);
    }

    // Create movement updates
    let core_updates: Vec<_> = core_entities
        .iter()
        .take(100)
        .map(|(entity, pos)| (*entity, *pos + Vec2::new(5.0, 5.0)))
        .collect();

    let bevy_updates: Vec<_> = bevy_entities
        .iter()
        .take(100)
        .map(|(entity, pos)| (*entity, *pos + Vec2::new(5.0, 5.0)))
        .collect();

    group.bench_function("original", |b| {
        b.iter(|| {
            for (entity, new_pos) in &core_updates {
                original_grid.insert(*entity, *new_pos);
            }
        });
    });

    group.bench_function("optimized", |b| {
        b.iter(|| {
            for (entity, new_pos) in &bevy_updates {
                optimized_grid.update_entity(*entity, *new_pos);
            }
        });
    });

    group.bench_function("optimized_batch", |b| {
        b.iter(|| {
            optimized_grid.update_entities_batch(&bevy_updates);
        });
    });

    group.finish();
}

fn bench_cache_effectiveness(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_effectiveness");

    let bevy_entities = create_bevy_entities(10000);
    let optimized_grid = SpatialHashGrid::new(10.0);

    for (entity, position) in &bevy_entities {
        optimized_grid.update_entity(*entity, *position);
    }

    // Benchmark repeated queries to same location (cache hits)
    group.bench_function("repeated_queries", |b| {
        b.iter(|| {
            for _ in 0..10 {
                let results = optimized_grid
                    .query_radius(black_box(Vec2::new(500.0, 500.0)), black_box(50.0));
                black_box(results);
            }
        });
    });

    // Benchmark different queries (cache misses)
    group.bench_function("varied_queries", |b| {
        let mut i = 0.0;
        b.iter(|| {
            for _ in 0..10 {
                let results = optimized_grid
                    .query_radius(black_box(Vec2::new(500.0 + i, 500.0 + i)), black_box(50.0));
                black_box(results);
                i += 100.0;
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_spatial_insert,
    bench_spatial_query,
    bench_spatial_update,
    bench_cache_effectiveness
);
criterion_main!(benches);
