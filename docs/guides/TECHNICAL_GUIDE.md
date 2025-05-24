# Technical Guide

## Table of Contents
1. [Architecture Overview](#architecture-overview)
2. [Getting Started](#getting-started)
3. [Implementation Details](#implementation-details)

---

## Architecture Overview

### Technology Stack

- **Game Engine**: [Bevy](https://bevyengine.org/) - Data-driven ECS game engine
- **Language**: Rust - Performance, safety, and concurrency
- **Rendering**: wgpu - Cross-platform graphics
- **UI Framework**: egui - Immediate mode GUI
- **Serialization**: serde - Save/load system
- **Parallelization**: rayon - Data parallelism

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        Game Client                          │
├─────────────────────────────────────────────────────────────┤
│                    Bevy ECS Framework                       │
├─────────────┬───────────┬──────────┬───────────┬──────────┤
│  Simulation │ Rendering │    UI    │   Audio   │  Input   │
│   Systems   │  Systems  │ Systems  │  Systems  │ Systems  │
├─────────────┴───────────┴──────────┴───────────┴──────────┤
│                    Core Components                          │
│  Creature • World • Resources • Social • Genetics • Time   │
├─────────────────────────────────────────────────────────────┤
│                   Platform Layer                            │
│         Windows • macOS • Linux • Web (future)             │
└─────────────────────────────────────────────────────────────┘
```

### Core Systems Organization

```rust
// Main game plugin structure
pub struct CreatureSimulationPlugin;

impl Plugin for CreatureSimulationPlugin {
    fn build(&self, app: &mut App) {
        app
            // Core plugins
            .add_plugin(WorldPlugin)
            .add_plugin(CreaturePlugin)
            .add_plugin(SimulationPlugin)
            .add_plugin(RenderingPlugin)
            .add_plugin(UIPlugin)
            
            // Resources
            .init_resource::<SimulationSettings>()
            .init_resource::<TimeControl>()
            
            // Events
            .add_event::<CreatureSpawnEvent>()
            .add_event::<CreatureDieEvent>()
            
            // Systems - ordered by stage
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(FixedTimestep::step(SIMULATION_TIMESTEP))
                    .with_system(update_creature_needs)
                    .with_system(process_decisions.after(update_creature_needs))
                    .with_system(execute_actions.after(process_decisions))
            );
    }
}
```

### Entity Component System (ECS)

The game uses Bevy's ECS architecture for maximum performance and flexibility:

```rust
// Entities are just IDs
type Entity = u32;

// Components are simple data structs
#[derive(Component)]
struct Position(Vec3);

#[derive(Component)]
struct Velocity(Vec3);

#[derive(Component)]
struct Health(f32);

// Systems operate on components
fn movement_system(
    time: Res<Time>,
    mut query: Query<(&mut Position, &Velocity)>,
) {
    for (mut pos, vel) in query.iter_mut() {
        pos.0 += vel.0 * time.delta_seconds();
    }
}

// Bundles group common components
#[derive(Bundle)]
struct CreatureBundle {
    position: Position,
    velocity: Velocity,
    health: Health,
    creature: Creature,
    #[bundle]
    sprite: SpriteBundle,
}
```

---

## Getting Started

### Prerequisites

- **Rust**: 1.70+ (install via [rustup](https://rustup.rs/))
- **Git**: For version control
- **C++ Compiler**: For linking (MSVC on Windows, gcc/clang on Linux/macOS)

### Development Setup

#### 1. Clone and Build

```bash
# Clone the repository
git clone https://github.com/yourusername/creature-simulation.git
cd creature-simulation

# Build in debug mode (faster compilation)
cargo build

# Build in release mode (optimized)
cargo build --release

# Run the game
cargo run --release
```

#### 2. IDE Setup

**VS Code** (Recommended)
```bash
# Install extensions
code --install-extension rust-lang.rust-analyzer
code --install-extension vadimcn.vscode-lldb
code --install-extension serayuzgur.crates
```

**.vscode/settings.json**:
```json
{
    "rust-analyzer.cargo.features": ["debug-ui"],
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.inlayHints.enable": true,
    "editor.formatOnSave": true
}
```

**IntelliJ IDEA / CLion**
- Install the Rust plugin
- Open project root as Cargo project

#### 3. Development Tools

```bash
# Install helpful development tools
cargo install cargo-watch    # Auto-rebuild on changes
cargo install cargo-expand   # Macro expansion
cargo install cargo-tree     # Dependency tree
cargo install cargo-bloat    # Binary size analysis

# Optional performance tools
cargo install flamegraph     # CPU profiling
cargo install cargo-criterion # Benchmarking
```

### Project Structure

```
creature-simulation/
├── Cargo.toml              # Project manifest
├── src/
│   ├── main.rs            # Entry point
│   ├── lib.rs             # Library root
│   ├── creature/          # Creature systems
│   │   ├── mod.rs
│   │   ├── components.rs  # Creature components
│   │   ├── systems.rs     # Creature systems
│   │   └── ai/           # AI subsystems
│   ├── world/            # World generation
│   ├── rendering/        # Rendering systems
│   ├── ui/              # User interface
│   └── utils/           # Shared utilities
├── assets/              # Game assets
│   ├── textures/
│   ├── sounds/
│   └── data/
├── tests/              # Integration tests
└── benches/           # Performance benchmarks
```

### Basic Bevy Concepts

#### Resources
Global data accessible to all systems:
```rust
#[derive(Resource)]
struct GameSettings {
    difficulty: Difficulty,
    creature_limit: usize,
}

// Access in systems
fn adjust_difficulty(
    mut settings: ResMut<GameSettings>,
    population: Res<PopulationStats>,
) {
    if population.total > settings.creature_limit {
        settings.difficulty = Difficulty::Hard;
    }
}
```

#### Events
Communication between systems:
```rust
#[derive(Event)]
struct CreatureSpawnEvent {
    position: Vec3,
    species: Species,
}

// Send events
fn spawn_creature(
    mut events: EventWriter<CreatureSpawnEvent>,
) {
    events.send(CreatureSpawnEvent {
        position: Vec3::new(100.0, 0.0, 100.0),
        species: Species::Herbivore,
    });
}

// Receive events
fn handle_spawns(
    mut events: EventReader<CreatureSpawnEvent>,
    mut commands: Commands,
) {
    for event in events.iter() {
        commands.spawn(CreatureBundle::new(event.position, event.species));
    }
}
```

#### Queries
Efficient component access:
```rust
// Simple query
fn heal_creatures(
    mut creatures: Query<&mut Health, With<Creature>>,
) {
    for mut health in creatures.iter_mut() {
        health.0 = (health.0 + 1.0).min(100.0);
    }
}

// Complex query with filters
fn update_hungry_creatures(
    mut creatures: Query<
        (&mut Behavior, &Position, &Hunger),
        (With<Creature>, Without<Sleeping>)
    >,
    food_sources: Query<&Position, With<Food>>,
) {
    for (mut behavior, pos, hunger) in creatures.iter_mut() {
        if hunger.0 > 80.0 {
            // Find nearest food
            if let Some(food_pos) = find_nearest(pos, &food_sources) {
                *behavior = Behavior::MoveTo(food_pos);
            }
        }
    }
}
```

---

## Implementation Details

### Component Design

Components should be small and focused:

```rust
// Good: Small, single-purpose components
#[derive(Component, Default)]
struct Position(Vec3);

#[derive(Component, Default)]
struct Velocity(Vec3);

#[derive(Component)]
struct Age(f32);

// Bad: Large monolithic component
#[derive(Component)]
struct CreatureData {
    position: Vec3,
    velocity: Vec3,
    health: f32,
    hunger: f32,
    age: f32,
    // ... many more fields
}

// Use marker components for tags
#[derive(Component)]
struct Carnivore;

#[derive(Component)]
struct Herbivore;

#[derive(Component)]
struct Sleeping;
```

### System Design

Systems should be focused and composable:

```rust
// Good: Focused systems that do one thing
fn apply_hunger(
    time: Res<Time>,
    mut creatures: Query<&mut Hunger>,
) {
    for mut hunger in creatures.iter_mut() {
        hunger.0 += HUNGER_RATE * time.delta_seconds();
    }
}

fn hunger_damage(
    mut creatures: Query<(&Hunger, &mut Health)>,
) {
    for (hunger, mut health) in creatures.iter_mut() {
        if hunger.0 > 90.0 {
            health.0 -= STARVATION_DAMAGE;
        }
    }
}

// System ordering
app.add_system(apply_hunger.before(hunger_damage));
```

### Performance Patterns

#### Spatial Indexing
```rust
#[derive(Resource)]
struct SpatialIndex {
    grid: HashMap<IVec3, Vec<Entity>>,
    cell_size: f32,
}

impl SpatialIndex {
    pub fn insert(&mut self, creature_id: CreatureId, position: Vec3) {
        let cell = self.world_to_cell(position);
        self.grid.entry(cell).or_default().push(creature_id);
    }
    
    pub fn query_radius(&self, center: Vec3, radius: f32) -> Vec<Entity> {
        let min = self.world_to_cell(center - Vec3::splat(radius));
        let max = self.world_to_cell(center + Vec3::splat(radius));
        
        let mut results = Vec::new();
        for x in min.x..=max.x {
            for y in min.y..=max.y {
                for z in min.z..=max.z {
                    if let Some(entities) = self.grid.get(&IVec3::new(x, y, z)) {
                        results.extend(entities);
                    }
                }
            }
        }
        results
    }
}
```

#### Change Detection
```rust
// Only process entities that changed
fn update_spatial_index(
    mut index: ResMut<SpatialIndex>,
    moved: Query<(Entity, &Position), Changed<Position>>,
) {
    for (entity, position) in moved.iter() {
        index.update(entity, position.0);
    }
}
```

#### Parallel Systems
```rust
// Bevy automatically runs compatible systems in parallel
// These can run simultaneously:
app.add_system(update_creature_hunger);
app.add_system(update_creature_thirst);
app.add_system(update_creature_energy);

// Use batch processing for heavy computation
fn process_ai_decisions(
    mut creatures: Query<(&mut Decision, &Brain)>,
    pool: Res<ComputeThreadPool>,
) {
    creatures.par_iter_mut().for_each_mut(|(mut decision, brain)| {
        *decision = brain.compute_decision();
    });
}
```

### Asset Management

```rust
// Asset loading
#[derive(Resource)]
struct GameAssets {
    creature_textures: HashMap<Species, Handle<Image>>,
    terrain_textures: HashMap<TerrainType, Handle<Image>>,
}

fn load_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let assets = GameAssets {
        creature_textures: HashMap::from([
            (Species::Herbivore, asset_server.load("textures/herbivore.png")),
            (Species::Carnivore, asset_server.load("textures/carnivore.png")),
        ]),
        terrain_textures: HashMap::from([
            (TerrainType::Grass, asset_server.load("textures/grass.png")),
            (TerrainType::Sand, asset_server.load("textures/sand.png")),
        ]),
    };
    
    commands.insert_resource(assets);
}
```

### Save System

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct SaveGame {
    version: u32,
    world_seed: u64,
    creatures: Vec<CreatureSaveData>,
    time: f64,
}

#[derive(Serialize, Deserialize)]
struct CreatureSaveData {
    position: [f32; 3],
    species: Species,
    age: f32,
    health: f32,
    genetics: GeneticData,
}

fn save_game(
    creatures: Query<(&Position, &Species, &Age, &Health, &Genetics), With<Creature>>,
    world: Res<WorldData>,
    time: Res<SimulationTime>,
) -> Result<(), SaveError> {
    let save = SaveGame {
        version: SAVE_VERSION,
        world_seed: world.seed,
        creatures: creatures.iter()
            .map(|(pos, species, age, health, genetics)| CreatureSaveData {
                position: pos.0.to_array(),
                species: *species,
                age: age.0,
                health: health.0,
                genetics: genetics.clone(),
            })
            .collect(),
        time: time.0,
    };
    
    let encoded = bincode::serialize(&save)?;
    std::fs::write("savegame.dat", encoded)?;
    Ok(())
}
```

### Debugging Tools

```rust
// Debug plugin for development
pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(WorldInspectorPlugin::new())
           .add_plugin(FrameTimeDiagnosticsPlugin)
           .add_plugin(LogDiagnosticsPlugin::default())
           .add_system(debug_creature_info);
    }
}

fn debug_creature_info(
    creatures: Query<(Entity, &Position, &Health, &Behavior)>,
    input: Res<Input<KeyCode>>,
) {
    if input.just_pressed(KeyCode::F3) {
        for (entity, pos, health, behavior) in creatures.iter() {
            info!("Creature {:?}: pos={:?}, health={}, behavior={:?}", 
                entity, pos.0, health.0, behavior);
        }
    }
}
```

### Platform Considerations

```rust
// Platform-specific code
#[cfg(target_os = "windows")]
fn platform_init() {
    // Windows-specific initialization
}

#[cfg(target_os = "macos")]
fn platform_init() {
    // macOS-specific initialization
}

#[cfg(target_arch = "wasm32")]
fn platform_init() {
    // Web-specific initialization
    console_error_panic_hook::set_once();
}

// Feature flags
#[cfg(feature = "debug-ui")]
app.add_plugin(DebugUIPlugin);

#[cfg(feature = "tracy")]
app.add_plugin(TracyPlugin);
```

### Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;
    
    #[test]
    fn test_creature_spawning() {
        let mut app = App::new();
        app.add_plugin(MinimalPlugins)
           .add_system(spawn_creature_system);
        
        app.world.spawn(CreatureBundle::default());
        app.update();
        
        let count = app.world.query::<&Creature>().iter(&app.world).count();
        assert_eq!(count, 1);
    }
}
```