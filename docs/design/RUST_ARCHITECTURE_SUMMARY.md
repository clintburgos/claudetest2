# Rust Architecture Summary

## Technology Stack

### Core Framework: Bevy
- **ECS Architecture** for managing hundreds of creatures efficiently
- **Built-in parallelization** for optimal performance
- **Plugin system** for modular code organization

### Key Libraries
- **bevy_egui**: Immediate mode UI for data-heavy displays
- **bevy_ecs_tilemap**: Efficient isometric tile rendering
- **noise**: Perlin noise for procedural world generation
- **petgraph**: Graph structures for family trees and social networks
- **nalgebra**: Mathematical operations for genetics

## Architecture Highlights

### 1. Entity Component System (ECS)
```rust
// Entities are just IDs
// Components are data
// Systems are behavior

// Small, focused components
Position, Velocity, Needs, Genetics, Social

// Systems process components in parallel
movement_system, needs_system, genetics_system
```

### 2. Modular Plugin Architecture
- **SimulationPlugin**: Creature behavior and lifecycle
- **WorldPlugin**: Procedural generation and resources
- **RenderingPlugin**: Isometric view and animations
- **UIPlugin**: egui panels and controls

### 3. Performance Strategies
- **Chunk-based world loading**
- **LOD system for animations**
- **Query filters to process only what's needed**
- **Time-scaled simulation updates**

### 4. Data Flow
```
Input → Time Controller → Filtered Systems → World State → Rendering
                ↑                                  ↓
                └──────── Feedback Loop ←──────────┘
```

## Best Practices for This Project

### DO:
- Use small, composable components
- Leverage Bevy's automatic parallelization
- Test systems in isolation
- Profile before optimizing
- Use query filters aggressively

### DON'T:
- Create monolithic components
- Update all creatures every frame
- Premature optimization
- Fight the ECS paradigm
- Ignore Bevy's built-in features

## Development Phases
1. **Foundation**: Basic ECS and movement
2. **World**: Procedural generation and chunks
3. **Behavior**: Needs, decisions, genetics
4. **Polish**: UI, animations, optimizations

## Key Design Decisions
- **Bevy over raw ECS**: Ecosystem and features
- **egui over custom UI**: Fast iteration on data-heavy UI
- **Tilemap over 3D**: Performance for many creatures
- **ECS over OOP**: Better performance and parallelization

---
*For detailed implementation: [TECHNICAL_ARCHITECTURE.md](./TECHNICAL_ARCHITECTURE.md)*
*To start coding: [RUST_GETTING_STARTED.md](./RUST_GETTING_STARTED.md)*