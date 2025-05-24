# Phase 1 Implementation Guide

Welcome! This guide will help you navigate the creature simulation codebase and find all relevant documentation for implementing Phase 1.

## Quick Start

**Goal**: Build a working creature simulation with 500 creatures at 60 FPS in 12 weeks.

**Key Documents**:
1. Start here: [`/docs/design/PHASE_1_ARCHITECTURE.md`](./design/PHASE_1_ARCHITECTURE.md) - Complete Phase 1 technical design
2. Class structure: [`/docs/design/PHASE_1_CLASS_DIAGRAM.md`](./design/PHASE_1_CLASS_DIAGRAM.md) - UML diagrams and relationships
3. Missing systems: [`/docs/design/CRITICAL_SYSTEMS.md`](./design/CRITICAL_SYSTEMS.md) - Error handling, profiling, debugging

## Project Overview

This is a creature simulation where players observe (not control) an ecosystem of creatures with emergent behaviors. Think of it as a digital ant farm or aquarium.

**Phase 1 Scope**:
- 500 creatures with basic needs (hunger, thirst, energy)
- Simple 2D world with resources (food, water)
- Basic movement and decision making
- Camera controls and minimal UI
- Error recovery and performance monitoring

**NOT in Phase 1**:
- Groups, social behaviors, or conversations
- Complex animations or particle effects
- Cultural evolution or advanced AI
- Save/load functionality
- Modding support

## Development Workflow

### Code Style and Standards

**Primary References**:
- [`/docs/design/CODE_STYLE_GUIDE.md`](./design/CODE_STYLE_GUIDE.md) - Rust conventions and patterns
- [`/docs/design/BEST_PRACTICES_SUMMARY.md`](./design/BEST_PRACTICES_SUMMARY.md) - General development practices
- [`/docs/design/DEVELOPMENT_WORKFLOW.md`](./design/DEVELOPMENT_WORKFLOW.md) - Git workflow and CI/CD

**Key Principles**:
```rust
// ✅ Good: Clear, simple, testable
pub struct Creature {
    pub position: Vec2,
    pub health: Health,
    pub needs: Needs,
}

// ❌ Bad: Over-engineered for Phase 1
pub struct Creature<T: Component, S: System> {
    components: HashMap<TypeId, Box<dyn Any>>,
    // ... complex generic system
}
```

### Testing Strategy

**Primary Reference**: [`/docs/TESTING_STRATEGY.md`](./TESTING_STRATEGY.md)

**Phase 1 Testing Priorities**:
1. Unit tests for core data structures
2. Integration tests for system interactions
3. Performance benchmarks for spatial queries
4. Simple simulation tests (100 creatures for 1 minute)

```rust
#[test]
fn creature_needs_update() {
    let mut creature = Creature::default();
    creature.needs.update(1.0); // 1 second
    assert!(creature.needs.hunger > 0.0);
}
```

## Architecture Overview

### System Layers

```
┌─────────────────────────────────────┐
│         Presentation Layer          │
│   Camera • Renderer • Basic UI      │
├─────────────────────────────────────┤
│         Simulation Layer            │
│  Creatures • Movement • Needs •     │
│  Resources • Decisions              │
├─────────────────────────────────────┤
│           Core Layer                │
│  Time • Events • Spatial Grid •     │
│  Error Boundary • Profiler          │
└─────────────────────────────────────┘
```

### Key Systems for Phase 1

1. **Entity System** (Week 1)
   - Simple entity IDs, not full ECS
   - [`/docs/design/PHASE_1_ARCHITECTURE.md#entity-model`](./design/PHASE_1_ARCHITECTURE.md#entity-model)

2. **Spatial Grid** (Week 1-2)
   - 50-unit cells for ~10 creatures per cell
   - [`/docs/design/PHASE_1_ARCHITECTURE.md#spatial-grid-system`](./design/PHASE_1_ARCHITECTURE.md#spatial-grid-system)

3. **Time System** (Week 2)
   - Fixed timestep, max 10x speed
   - [`/docs/design/PHASE_1_ARCHITECTURE.md#time-system`](./design/PHASE_1_ARCHITECTURE.md#time-system)

4. **Creature System** (Week 3-4)
   - Basic needs and health
   - [`/docs/design/PHASE_1_ARCHITECTURE.md#creature-model`](./design/PHASE_1_ARCHITECTURE.md#creature-model)

5. **Movement System** (Week 3-4)
   - Simple point-to-point movement
   - [`/docs/design/PHASE_1_ARCHITECTURE.md#movement-system`](./design/PHASE_1_ARCHITECTURE.md#movement-system)

6. **Decision System** (Week 5-6)
   - Priority-based needs satisfaction
   - [`/docs/design/PHASE_1_ARCHITECTURE.md#decision-system`](./design/PHASE_1_ARCHITECTURE.md#decision-system)

## Performance Guidelines

**Primary Reference**: [`/docs/reference/PERFORMANCE_TARGETS.md`](./reference/PERFORMANCE_TARGETS.md)

**Phase 1 Targets**:
- 500 creatures at 60 FPS on GTX 1060
- 16ms frame budget (6ms update, 10ms render)
- Memory usage: ~100MB total

**Performance Checklist**:
- [ ] Use spatial grid for all proximity queries
- [ ] Update creatures in batches
- [ ] Profile before optimizing
- [ ] Test on min-spec hardware regularly

## Common Pitfalls to Avoid

### 1. Over-Engineering
```rust
// ❌ Don't do this in Phase 1
impl CreatureSystem {
    pub fn update(&mut self, world: &mut World) {
        self.parallel_update_with_cache_invalidation_and_lod_culling(world);
    }
}

// ✅ Do this instead
impl CreatureSystem {
    pub fn update(&mut self, world: &mut World, dt: f32) {
        for creature in &mut world.creatures {
            creature.update_needs(dt);
        }
    }
}
```

### 2. Premature Optimization
- Don't implement caching until profiler shows it's needed
- Don't use parallel processing for 500 creatures
- Don't implement LOD in Phase 1

### 3. Feature Creep
- No groups or social systems
- No save/load
- No complex animations
- Keep it simple!

## Debug Tools

**Primary Reference**: [`/docs/design/CRITICAL_SYSTEMS.md#debug-tooling`](./design/CRITICAL_SYSTEMS.md#debug-tooling)

**Essential Debug Features**:
1. Creature inspector (click to see stats)
2. Pause and step simulation
3. Visual overlays (needs, health, IDs)
4. Performance overlay (FPS, frame time)

## Where to Find Everything

### Core Design Documents
- **Phase 1 Architecture**: [`/docs/design/PHASE_1_ARCHITECTURE.md`](./design/PHASE_1_ARCHITECTURE.md)
- **Class Diagrams**: [`/docs/design/PHASE_1_CLASS_DIAGRAM.md`](./design/PHASE_1_CLASS_DIAGRAM.md)
- **Critical Systems**: [`/docs/design/CRITICAL_SYSTEMS.md`](./design/CRITICAL_SYSTEMS.md)

### Best Practices
- **Code Style**: [`/docs/design/CODE_STYLE_GUIDE.md`](./design/CODE_STYLE_GUIDE.md)
- **Performance**: [`/docs/design/PERFORMANCE_FIRST.md`](./design/PERFORMANCE_FIRST.md)
- **Testing**: [`/docs/TESTING_STRATEGY.md`](./TESTING_STRATEGY.md)

### System References (for future phases)
- **Full System List**: [`/docs/INDEX.md`](./INDEX.md)
- **System Interactions**: [`/docs/reference/SYSTEM_INTERACTION_MATRIX.md`](./reference/SYSTEM_INTERACTION_MATRIX.md)

### Performance and Optimization
- **Performance Targets**: [`/docs/reference/PERFORMANCE_TARGETS.md`](./reference/PERFORMANCE_TARGETS.md)
- **Optimization Guide**: [`/docs/design/PERFORMANCE_OPTIMIZATION.md`](./design/PERFORMANCE_OPTIMIZATION.md)

## Getting Started Checklist

1. [ ] Read Phase 1 Architecture document
2. [ ] Review class diagrams
3. [ ] Set up Rust development environment
4. [ ] Clone repo and run `cargo test`
5. [ ] Implement Entity and EntityManager
6. [ ] Add basic tests
7. [ ] Continue with Week 1 goals

## Implementation Schedule

### Week 1-2: Foundation
- Entity system
- Time system  
- Spatial grid
- Basic event bus

### Week 3-4: Creatures
- Creature struct and basic needs
- Movement system
- Health system

### Week 5-6: Behavior
- Resource spawning
- Decision making
- Need satisfaction

### Week 7-8: Presentation  
- Camera controls
- Basic rendering
- Minimal UI

### Week 9-10: Polish
- Error recovery
- Performance profiling
- Debug tools

### Week 11-12: Testing
- Integration tests
- Performance optimization
- Bug fixes

## Questions?

If something isn't clear:
1. Check the primary references listed above
2. Look for examples in test files
3. Keep it simple - if it seems complex, it's probably not for Phase 1

Remember: **Make it work, make it right, then make it fast.**

Good luck! 🚀