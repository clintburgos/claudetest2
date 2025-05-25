//! Benchmarks for decision system performance
//!
//! Measures the performance improvements from decoupling the decision system
//! in Phase 1.2, comparing pure function performance with caching.

use bevy::math::Vec2;
use creature_simulation::core::VersionedEntity;
use creature_simulation::simulation::{CreatureState, ResourceType};
use creature_simulation::systems::decision_v2::{
    decision_functions, CreatureInfo, DecisionCache, DecisionContext, NeedState, ResourceInfo,
    ThreatInfo,
};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

fn create_simple_context() -> DecisionContext {
    DecisionContext {
        entity: VersionedEntity::new(1, 0),
        position: Vec2::new(100.0, 100.0),
        velocity: Vec2::ZERO,
        state: CreatureState::Idle,
        needs: NeedState {
            hunger: 0.5,
            thirst: 0.5,
            energy: 0.5,
            social: 0.5,
        },
        health: 1.0,
        energy: 0.5,
        nearby_resources: vec![],
        nearby_creatures: vec![],
        nearby_threats: vec![],
        time_since_last_decision: 1.0,
    }
}

fn create_complex_context() -> DecisionContext {
    DecisionContext {
        entity: VersionedEntity::new(1, 0),
        position: Vec2::new(100.0, 100.0),
        velocity: Vec2::new(1.0, 0.0),
        state: CreatureState::Moving {
            target: Vec2::new(150.0, 100.0),
        },
        needs: NeedState {
            hunger: 0.8,
            thirst: 0.6,
            energy: 0.3,
            social: 0.7,
        },
        health: 0.8,
        energy: 0.3,
        nearby_resources: vec![
            ResourceInfo {
                entity: VersionedEntity::new(2, 0),
                position: Vec2::new(110.0, 100.0),
                resource_type: ResourceType::Food,
                amount: 50.0,
                distance: 10.0,
            },
            ResourceInfo {
                entity: VersionedEntity::new(3, 0),
                position: Vec2::new(120.0, 110.0),
                resource_type: ResourceType::Water,
                amount: 100.0,
                distance: 22.4,
            },
            ResourceInfo {
                entity: VersionedEntity::new(4, 0),
                position: Vec2::new(90.0, 90.0),
                resource_type: ResourceType::Food,
                amount: 30.0,
                distance: 14.1,
            },
        ],
        nearby_creatures: vec![
            CreatureInfo {
                entity: VersionedEntity::new(5, 0),
                position: Vec2::new(105.0, 105.0),
                relationship: creature_simulation::systems::decision_v2::Relationship::Friendly,
                distance: 7.1,
            },
            CreatureInfo {
                entity: VersionedEntity::new(6, 0),
                position: Vec2::new(150.0, 100.0),
                relationship: creature_simulation::systems::decision_v2::Relationship::Neutral,
                distance: 50.0,
            },
        ],
        nearby_threats: vec![ThreatInfo {
            position: Vec2::new(200.0, 100.0),
            threat_level: 0.3,
            distance: 100.0,
        }],
        time_since_last_decision: 0.5,
    }
}

fn bench_decision_making(c: &mut Criterion) {
    let mut group = c.benchmark_group("decision_making");

    let simple_context = create_simple_context();
    let complex_context = create_complex_context();

    group.bench_function("simple_context", |b| {
        b.iter(|| {
            let decision = decision_functions::make_decision(black_box(&simple_context));
            black_box(decision);
        });
    });

    group.bench_function("complex_context", |b| {
        b.iter(|| {
            let decision = decision_functions::make_decision(black_box(&complex_context));
            black_box(decision);
        });
    });

    group.finish();
}

fn bench_decision_caching(c: &mut Criterion) {
    let mut group = c.benchmark_group("decision_caching");

    let cache = DecisionCache::new(1000);
    let contexts: Vec<_> = (0..100)
        .map(|i| {
            let mut ctx = create_simple_context();
            ctx.entity = VersionedEntity::new(i, 0);
            ctx.position = Vec2::new(i as f32 * 10.0, i as f32 * 10.0);
            ctx.needs.hunger = (i as f32 * 0.01) % 1.0;
            ctx
        })
        .collect();

    // Benchmark without cache
    group.bench_function("no_cache", |b| {
        b.iter(|| {
            for context in &contexts {
                let decision = decision_functions::make_decision(black_box(context));
                black_box(decision);
            }
        });
    });

    // Benchmark with cache (first pass - all misses)
    group.bench_function("cache_cold", |b| {
        b.iter(|| {
            let cache = DecisionCache::new(1000);
            for context in &contexts {
                if let Some(cached) = cache.get(context, 0.0) {
                    black_box(cached);
                } else {
                    let decision = decision_functions::make_decision(black_box(context));
                    cache.insert(context, decision.clone(), 0.0);
                    black_box(decision);
                }
            }
        });
    });

    // Benchmark with warm cache
    group.bench_function("cache_warm", |b| {
        // Pre-populate cache
        for context in &contexts {
            let decision = decision_functions::make_decision(context);
            cache.insert(context, decision, 0.0);
        }

        b.iter(|| {
            for context in &contexts {
                if let Some(cached) = cache.get(context, 0.0) {
                    black_box(cached);
                } else {
                    let decision = decision_functions::make_decision(black_box(context));
                    black_box(decision);
                }
            }
        });
    });

    group.finish();
}

fn bench_context_variations(c: &mut Criterion) {
    let mut group = c.benchmark_group("context_variations");

    // Benchmark different context complexities
    for num_resources in [0, 5, 10, 20].iter() {
        for num_creatures in [0, 5, 10].iter() {
            let mut context = create_simple_context();

            // Add resources
            context.nearby_resources = (0..*num_resources)
                .map(|i| ResourceInfo {
                    entity: VersionedEntity::new(i as u32 + 10, 0),
                    position: Vec2::new(100.0 + i as f32 * 10.0, 100.0),
                    resource_type: if i % 2 == 0 {
                        ResourceType::Food
                    } else {
                        ResourceType::Water
                    },
                    amount: 50.0,
                    distance: i as f32 * 10.0,
                })
                .collect();

            // Add creatures
            context.nearby_creatures = (0..*num_creatures)
                .map(|i| CreatureInfo {
                    entity: VersionedEntity::new(i as u32 + 100, 0),
                    position: Vec2::new(100.0 + i as f32 * 15.0, 100.0),
                    relationship: creature_simulation::systems::decision_v2::Relationship::Neutral,
                    distance: i as f32 * 15.0,
                })
                .collect();

            let id = format!("r{}_c{}", num_resources, num_creatures);
            group.bench_with_input(
                BenchmarkId::new("decision_complexity", &id),
                &context,
                |b, ctx| {
                    b.iter(|| {
                        let decision = decision_functions::make_decision(black_box(ctx));
                        black_box(decision);
                    });
                },
            );
        }
    }

    group.finish();
}

fn bench_threat_response(c: &mut Criterion) {
    let mut group = c.benchmark_group("threat_response");

    let mut safe_context = create_simple_context();
    safe_context.needs.hunger = 0.9; // Very hungry
    safe_context.nearby_resources.push(ResourceInfo {
        entity: VersionedEntity::new(2, 0),
        position: Vec2::new(110.0, 100.0),
        resource_type: ResourceType::Food,
        amount: 100.0,
        distance: 10.0,
    });

    let mut threat_context = safe_context.clone();
    threat_context.nearby_threats.push(ThreatInfo {
        position: Vec2::new(105.0, 100.0),
        threat_level: 0.9,
        distance: 5.0,
    });

    group.bench_function("no_threat", |b| {
        b.iter(|| {
            let decision = decision_functions::make_decision(black_box(&safe_context));
            black_box(decision);
        });
    });

    group.bench_function("with_threat", |b| {
        b.iter(|| {
            let decision = decision_functions::make_decision(black_box(&threat_context));
            black_box(decision);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_decision_making,
    bench_decision_caching,
    bench_context_variations,
    bench_threat_response
);
criterion_main!(benches);
