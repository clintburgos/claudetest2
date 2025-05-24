# Technical Architecture & Rust Implementation

## Overview
This document outlines the recommended Rust frameworks, architectural patterns, and best practices for implementing the creature simulation. For performance optimization strategies, see [PERFORMANCE_OPTIMIZATION.md](./PERFORMANCE_OPTIMIZATION.md).

## Recommended Rust Frameworks

### Core Game Engine: Bevy
**Why Bevy:**
- **ECS Architecture**: Perfect for managing thousands of creatures efficiently
- **Built-in Systems**: Transform, rendering, input handling
- **Parallelization**: Automatic system parallelization for performance
- **Hot Reloading**: Fast iteration during development
- **Active Community**: Extensive plugins and examples

```toml
[dependencies]
bevy = "0.13"
bevy_egui = "0.25"  # For UI
bevy_ecs_tilemap = "0.13"  # For isometric tile rendering
```

### Alternative: Macroquad
**Consider if:**
- You want simpler, immediate-mode rendering
- Less complex but more direct control
- Easier learning curve

### UI Framework: egui (via bevy_egui)
**Why egui:**
- Immediate mode GUI perfect for data-heavy displays
- Excellent for dynamic panels and data visualization
- Easy integration with Bevy
- Good performance with lots of updating data

### Additional Key Dependencies
```toml
# Procedural Generation
noise = "0.8"  # Perlin noise for world generation
rand = "0.8"
rand_chacha = "0.3"  # Deterministic RNG for seeds

# Serialization
serde = { version = "1.0", features = ["derive"] }
ron = "0.8"  # Rust Object Notation for save files

# Data Structures
petgraph = "0.6"  # For family trees and social networks
nalgebra = "0.32"  # For genetic algorithms

# Performance
rayon = "1.7"  # Parallel iterators
parking_lot = "0.12"  # Better mutex performance

# Development
tracing = "0.1"  # Structured logging
tracing-subscriber = "0.3"
criterion = "0.5"  # Benchmarking
```

## Architecture Design

### High-Level Architecture
```
┌─────────────────────────────────────────────────────────────┐
│                        Main Game Loop                        │
├─────────────────────────────────────────────────────────────┤
│                          Bevy App                            │
├─────────────┬─────────────┬─────────────┬─────────────────┤
│   Simulation│     World    │   Rendering │      UI         │
│    Systems  │   Systems    │   Systems   │   Systems       │
├─────────────┼─────────────┼─────────────┼─────────────────┤
│  ECS World  │ Chunk Manager│  Isometric  │     egui        │
│  Creatures  │    Biomes    │   Renderer  │   Panels        │
│  Genetics   │  Resources   │ Animations  │   Graphs        │
│   Social    │   Weather    │   Camera    │  Controls       │
└─────────────┴─────────────┴─────────────┴─────────────────┘
```

### ECS Component Design

```rust
// Component Size Guidelines (for cache efficiency)
// Hot components: < 16 bytes (accessed every frame)
// Warm components: < 64 bytes (accessed frequently)
// Cold components: No strict limit (accessed rarely)

// Core creature components
#[derive(Component)]
struct Creature {
    id: CreatureId,
    name: String,
    age: f32,
    generation: u32,
}

#[derive(Component)]
struct Position {  // 8 bytes - HOT component ✓
    x: f32,
    y: f32,
}

#[derive(Component)]
struct Velocity(Vec2);  // 8 bytes - HOT component ✓

#[derive(Component)]
struct Needs {  // 24 bytes - WARM component ✓
    hunger: f32,
    thirst: f32,
    social: f32,
    energy: f32,
    safety: f32,
    rest: f32,
}

#[derive(Component)]
struct Genetics {
    dna: DNA,
    expressed_traits: Vec<Trait>,
}

#[derive(Component)]
struct Social {
    relationships: HashMap<CreatureId, Relationship>,
    current_conversation: Option<Conversation>,
}

#[derive(Component)]
struct CreatureSprite {
    base_sprite: Handle<Image>,
    expression: Expression,
    animation_state: AnimationState,
}
// World components
#[derive(Component)]
struct Tile {
    biome: BiomeType,
    elevation: f32,
    temperature: f32,
    moisture: f32,
}

#[derive(Component)]
struct TileAnimation {
    animation_type: TileAnimationType,
    phase: f32,
    speed: f32,
}
```

### System Organization

```rust
// Plugin structure
pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .insert_resource(TimeController::default())
            .insert_resource(WorldSettings::default())
            
            // Events
            .add_event::<CreatureBornEvent>()
            .add_event::<CreatureDeathEvent>()
            .add_event::<ConversationEvent>()
            
            // Systems - ordered by stage
            .add_systems(Update, (
                // Input stage
                handle_camera_input,
                handle_time_controls,
                handle_selection,
                
                // Simulation stage (scaled by time)
                update_needs.run_if(should_update_simulation),
                process_decisions.run_if(should_update_simulation),
                handle_movement.run_if(should_update_simulation),
                handle_conversations.run_if(should_update_simulation),
                handle_reproduction.run_if(should_update_simulation),
                
                // World stage
                update_weather,
                grow_resources,
                animate_tiles,
                
                // Rendering stage
                update_creature_sprites,
                update_visibility_solutions,
                cull_distant_animations,
            ).chain());
    }
}
```

### Module Structure
```
src/
├── main.rs              # Bevy app setup
├── lib.rs               # Public API
│
├── simulation/          # Core simulation logic
│   ├── mod.rs
│   ├── creature.rs      # Creature components and systems
│   ├── genetics.rs      # DNA and trait systems
│   ├── needs.rs         # Need calculation and priorities
│   ├── decisions.rs     # AI decision making
│   ├── social.rs        # Conversations and relationships
│   └── lifecycle.rs     # Birth, aging, death
│
├── world/              # World management
│   ├── mod.rs
│   ├── generation.rs   # Procedural generation
│   ├── chunk.rs        # Chunk loading/unloading
│   ├── biome.rs        # Biome definitions
│   ├── weather.rs      # Weather systems
│   └── resources.rs    # Food/water spawning
│
├── rendering/          # Visual systems
│   ├── mod.rs
│   ├── isometric.rs    # Isometric projection
│   ├── creature_renderer.rs
│   ├── tile_renderer.rs
│   ├── animations.rs   # Sprite animations
│   ├── particles.rs    # Emotion particles
│   └── visibility.rs   # Occlusion handling
│
├── ui/                 # User interface
│   ├── mod.rs
│   ├── panels.rs       # egui panels
│   ├── graphs.rs       # Data visualization
│   ├── controls.rs     # Input handling
│   └── camera.rs       # Camera controller
│
├── data/              # Data structures
│   ├── mod.rs
│   ├── constants.rs   # Game constants
│   ├── types.rs       # Common types
│   └── save.rs        # Save/load system
│
└── utils/             # Utilities
    ├── mod.rs
    ├── math.rs        # Helper functions
    ├── random.rs      # RNG utilities
    └── performance.rs # Profiling helpers
```
## Best Practices

### 1. ECS Design Principles
- **Composition over Inheritance**: Use small, focused components
- **Data-Oriented Design**: Keep components small and cache-friendly
- **System Parallelization**: Ensure systems can run in parallel when possible

```rust
// Good: Small, focused components
#[derive(Component)]
struct Velocity { x: f32, y: f32 }

// Bad: Monolithic component
#[derive(Component)]
struct CreatureData {
    position: Vec2,
    velocity: Vec2,
    health: f32,
    // ... 50 more fields
}
```

### 2. Performance Optimization
- **Use Query Filters**: Only process entities that need updates
- **Batch Operations**: Group similar operations together
- **Spatial Indexing**: Use quadtrees for creature queries

```rust
// Efficient query with filters
fn update_hungry_creatures(
    mut query: Query<(&mut Needs, &Position), With<Hungry>>,
    food_map: Res<FoodMap>,
) {
    // Only processes creatures marked as Hungry
}
```

### 3. Time Scaling Architecture
```rust
#[derive(Resource)]
struct TimeController {
    speed: TimeSpeed,
    accumulated_time: f32,
    update_threshold: f32,
}

fn should_update_simulation(
    time: Res<Time>,
    mut controller: ResMut<TimeController>,
) -> bool {
    controller.accumulated_time += time.delta_seconds() * controller.speed.multiplier();
    if controller.accumulated_time >= controller.update_threshold {
        controller.accumulated_time -= controller.update_threshold;
        true
    } else {
        false
    }
}
```

### 4. Chunk-Based World Management
```rust
#[derive(Resource)]
struct ChunkManager {
    loaded_chunks: HashMap<ChunkCoord, Chunk>,
    active_radius: u32,
}

impl ChunkManager {
    fn update_loaded_chunks(&mut self, camera_pos: Vec2) {
        let center_chunk = world_to_chunk(camera_pos);
        
        // Unload distant chunks
        self.loaded_chunks.retain(|coord, _| {
            coord.distance_to(center_chunk) <= self.active_radius
        });
        
        // Load new chunks
        for coord in chunks_in_radius(center_chunk, self.active_radius) {
            self.loaded_chunks.entry(coord).or_insert_with(|| {
                generate_chunk(coord)
            });
        }
    }
}
```
### 5. Genetics Implementation
```rust
use nalgebra::DVector;

#[derive(Clone, Debug)]
struct DNA {
    genes: DVector<f32>,  // Continuous values 0.0-1.0
}

impl DNA {
    fn crossover(&self, other: &DNA, rng: &mut impl Rng) -> DNA {
        let mut child_genes = self.genes.clone();
        
        // Uniform crossover
        for i in 0..child_genes.len() {
            if rng.gen_bool(0.5) {
                child_genes[i] = other.genes[i];
            }
        }
        
        // Mutation
        if rng.gen_bool(MUTATION_RATE) {
            let gene_idx = rng.gen_range(0..child_genes.len());
            child_genes[gene_idx] += rng.gen_range(-0.1..0.1);
            child_genes[gene_idx] = child_genes[gene_idx].clamp(0.0, 1.0);
        }
        
        DNA { genes: child_genes }
    }
}
```

### 6. Save System
```rust
#[derive(Serialize, Deserialize)]
struct SaveGame {
    version: u32,
    seed: u64,
    time: GameTime,
    creatures: Vec<CreatureSaveData>,
    world_chunks: HashMap<ChunkCoord, ChunkData>,
}

fn save_game(world: &World) -> Result<(), SaveError> {
    let save_data = build_save_data(world);
    let encoded = ron::to_string(&save_data)?;
    std::fs::write("save.ron", encoded)?;
    Ok(())
}
```
### 7. Testing Strategy
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_creature_needs_decay() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, update_needs);
        
        // Spawn test creature
        let creature = app.world.spawn((
            Creature::default(),
            Needs { hunger: 100.0, thirst: 100.0, ..default() },
        )).id();
        
        // Run simulation
        app.update();
        
        // Check needs decreased
        let needs = app.world.get::<Needs>(creature).unwrap();
        assert!(needs.hunger < 100.0);
    }
}
```

## Development Workflow

### 1. Incremental Development
1. Start with basic ECS and creature movement
2. Add one system at a time with tests
3. Implement UI panels incrementally
4. Add optimizations only after profiling

### 2. Debugging Tools
- Use `bevy-inspector-egui` for runtime component inspection
- Add debug visualization systems (relationship lines, need bars)
- Implement time control early for easier testing

### 3. Performance Monitoring
```rust
fn performance_diagnostics(diagnostics: Res<Diagnostics>) {
    if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(average) = fps.average() {
            if average < 30.0 {
                warn!("Low FPS: {:.2}", average);
            }
        }
    }
}
```

---
*Last Updated: 2024-12-XX*