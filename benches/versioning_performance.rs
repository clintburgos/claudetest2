//! Benchmarks for entity versioning system performance
//!
//! Measures the overhead of version tracking and validation from Phase 1.1.

use creature_simulation::core::{EntityVersions, Version, VersionedEntity};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_entity_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("entity_allocation");

    group.bench_function("versioned_allocation", |b| {
        let versions = EntityVersions::new();
        b.iter(|| {
            let entity = versions.allocate();
            black_box(entity);
        });
    });

    group.bench_function("versioned_with_recycling", |b| {
        let versions = EntityVersions::new();

        // Pre-allocate and deallocate some entities for recycling
        let mut entities = Vec::new();
        for _ in 0..100 {
            entities.push(versions.allocate());
        }
        for e in entities {
            versions.deallocate(e);
        }

        b.iter(|| {
            let entity = versions.allocate();
            black_box(entity);
        });
    });

    group.finish();
}

fn bench_version_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("version_validation");

    let versions = EntityVersions::new();
    let valid_entities: Vec<_> = (0..1000).map(|_| versions.allocate()).collect();

    // Create some invalid entities
    let invalid_entities: Vec<_> = valid_entities
        .iter()
        .map(|e| VersionedEntity::new(e.id, e.generation + 1))
        .collect();

    group.bench_function("valid_check", |b| {
        let mut i = 0;
        b.iter(|| {
            let entity = &valid_entities[i % valid_entities.len()];
            let is_valid = versions.is_valid(*entity);
            black_box(is_valid);
            i += 1;
        });
    });

    group.bench_function("invalid_check", |b| {
        let mut i = 0;
        b.iter(|| {
            let entity = &invalid_entities[i % invalid_entities.len()];
            let is_valid = versions.is_valid(*entity);
            black_box(is_valid);
            i += 1;
        });
    });

    group.finish();
}

fn bench_version_component(c: &mut Criterion) {
    let mut group = c.benchmark_group("version_component");

    group.bench_function("direct_access", |b| {
        let component = Version::new(100.0f32);
        b.iter(|| {
            let value = component.data;
            black_box(value);
        });
    });

    group.bench_function("deref_access", |b| {
        let component = Version::new(100.0f32);
        b.iter(|| {
            let value = *component;
            black_box(value);
        });
    });

    group.bench_function("mutation_with_increment", |b| {
        let mut component = Version::new(100.0f32);
        b.iter(|| {
            *component += 1.0;
            black_box(component.generation);
        });
    });

    group.finish();
}

fn bench_bulk_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("bulk_operations");

    for size in [100, 1000, 5000].iter() {
        group.bench_function(format!("allocate_{}", size), |b| {
            b.iter(|| {
                let versions = EntityVersions::new();
                let mut entities = Vec::with_capacity(*size);
                for _ in 0..*size {
                    entities.push(versions.allocate());
                }
                black_box(entities);
            });
        });

        group.bench_function(format!("validate_{}", size), |b| {
            let versions = EntityVersions::new();
            let entities: Vec<_> = (0..*size).map(|_| versions.allocate()).collect();

            b.iter(|| {
                let mut valid_count = 0;
                for entity in &entities {
                    if versions.is_valid(*entity) {
                        valid_count += 1;
                    }
                }
                black_box(valid_count);
            });
        });
    }

    group.finish();
}

fn bench_concurrent_access(c: &mut Criterion) {
    use std::sync::Arc;
    use std::thread;

    let mut group = c.benchmark_group("concurrent_access");

    group.bench_function("concurrent_allocation", |b| {
        b.iter(|| {
            let versions = Arc::new(EntityVersions::new());
            let mut handles = vec![];

            for _ in 0..4 {
                let versions_clone = Arc::clone(&versions);
                let handle = thread::spawn(move || {
                    let mut entities = Vec::with_capacity(250);
                    for _ in 0..250 {
                        entities.push(versions_clone.allocate());
                    }
                    entities
                });
                handles.push(handle);
            }

            let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
            black_box(results);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_entity_allocation,
    bench_version_validation,
    bench_version_component,
    bench_bulk_operations,
    bench_concurrent_access
);
criterion_main!(benches);
