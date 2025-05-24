# Getting Started with Rust Implementation

## Project Setup

### 1. Create New Project
```bash
cargo new creature-simulation --bin
cd creature-simulation
```

### 2. Initial Cargo.toml
```toml
[package]
name = "creature-simulation"
version = "0.1.0"
edition = "2021"

[dependencies]
# Core
bevy = { version = "0.13", features = ["dynamic_linking"] }
bevy_egui = "0.25"
bevy_ecs_tilemap = "0.13"

# Utils
rand = "0.8"
noise = "0.8"
serde = { version = "1.0", features = ["derive"] }
ron = "0.8"

# Development
tracing = "0.1"
tracing-subscriber = "0.3"

[profile.dev]
opt-level = 1  # Better performance in debug

[profile.dev.package."*"]
opt-level = 3  # Optimize dependencies
```

### 3. Basic main.rs Structure
```rust
use bevy::prelude::*;
use bevy_egui::EguiPlugin;

mod simulation;
mod world;
mod rendering;
mod ui;

fn main() {
    App::new()
        // Bevy plugins
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Creature Simulation".to_string(),
                resolution: (1600., 900.).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin)
        
        // Our plugins
        .add_plugins((
            simulation::SimulationPlugin,
            world::WorldPlugin,
            rendering::RenderingPlugin,
            ui::UIPlugin,
        ))
        
        .run();
}
```

## Implementation Order

### Phase 1: Foundation (Week 1-2)
1. **Basic ECS Structure**
   - Create core components (Position, Creature)
   - Simple movement system
   - Basic camera controls

2. **Isometric Rendering**
   - Set up bevy_ecs_tilemap
   - Render basic isometric grid
   - Creature sprites

### Phase 2: World (Week 3-4)
1. **Procedural Generation**
   - Implement noise-based terrain
   - Create biome system
   - Add chunk loading

2. **Resources**
   - Spawn food/water
   - Basic resource consumption

### Phase 3: Creature Behavior (Week 5-6)
1. **Need System**
   - Implement all needs
   - Decision making
   - Movement towards resources

2. **Genetics**
   - DNA representation
   - Trait expression
   - Reproduction system

### Phase 4: UI & Polish (Week 7-8)
1. **egui Integration**
   - Info panels
   - Time controls
   - Data visualization

2. **Visual Polish**
   - Animations
   - Particle effects
   - Visibility solutions

## Code Examples to Get Started

### Basic Creature Component
```rust
// In simulation/creature.rs
use bevy::prelude::*;

#[derive(Component, Default)]
pub struct Creature {
    pub name: String,
    pub age: f32,
    pub generation: u32,
}

#[derive(Component, Default)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Bundle, Default)]
pub struct CreatureBundle {
    pub creature: Creature,
    pub position: Position,
    pub velocity: Velocity,
    pub sprite: SpriteBundle,
}
```

### Simple Movement System
```rust
// In simulation/movement.rs
pub fn move_creatures(
    time: Res<Time>,
    mut query: Query<(&Velocity, &mut Transform), With<Creature>>,
) {
    for (velocity, mut transform) in &mut query {
        transform.translation.x += velocity.x * time.delta_seconds();
        transform.translation.y += velocity.y * time.delta_seconds();
    }
}
```

## Common Pitfalls to Avoid

1. **Don't Over-Engineer Early**
   - Start simple, add complexity gradually
   - Get basic systems working before optimization

2. **Bevy-Specific Issues**
   - Remember to add `.chain()` when system order matters
   - Use `Commands` for spawning/despawning
   - Be careful with mutable queries

3. **Performance Traps**
   - Don't update every creature every frame at high speeds
   - Use change detection where possible
   - Profile before optimizing

## Next Steps
1. Set up the project structure
2. Implement basic creature spawning and movement
3. Add isometric camera and rendering
4. Build from there!

---
*See [TECHNICAL_ARCHITECTURE.md](./TECHNICAL_ARCHITECTURE.md) for detailed architecture*