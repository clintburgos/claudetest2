# Creature Simulation

A real-time creature simulation featuring emergent behaviors, social dynamics, and evolution. Watch as creatures navigate their world, form relationships, and adapt to survive.

## 🎮 Features

- **Living Creatures**: Autonomous entities with needs, emotions, and decision-making
- **Social Systems**: Creatures form relationships, have conversations, and influence each other
- **Dynamic World**: Procedurally generated biomes with resources and environmental challenges
- **Real-time Visualization**: Isometric view with smooth animations and expressive creatures
- **Time Control**: Pause, slow down, or speed up to 1000x for generational observations
- **Data Insights**: Population statistics, genetic trends, and behavioral analysis

## 🚀 Quick Start

```bash
# Clone the repository
git clone <repository-url>
cd creature-simulation

# Build and run
cargo run --release

# Run tests
cargo test

# Run with debug features
cargo run
```

### Controls

- **Mouse**: Click to select creatures, drag to pan camera
- **Scroll**: Zoom in/out
- **Space**: Pause/unpause simulation
- **Number Keys (1-6)**: Set simulation speed
- **Tab**: Toggle UI panels
- **F1-F4**: Debug visualizations

## 🏗️ Architecture

Built with Rust and Bevy game engine for maximum performance:

```
src/
├── components/     # ECS components (Creature, Position, etc.)
├── systems/        # Game logic (movement, decisions, etc.)
├── plugins/        # Bevy plugins (rendering, UI, etc.)
├── simulation/     # Core simulation logic
├── core/           # Foundation systems (entity, time, spatial)
└── utils/          # Utilities and helpers
```

### Key Technologies

- **Bevy**: Modern ECS game engine for Rust
- **egui**: Immediate mode GUI for controls and data
- **bevy_ecs_tilemap**: Efficient isometric tile rendering
- **ahash**: High-performance hashing for spatial indexing

## 📊 Performance

Optimized for large-scale simulations:

- 500+ creatures at 60 FPS (Phase 1)
- 5000+ creatures planned (Phase 2)
- Spatial indexing for O(log n) proximity queries
- Parallel processing with Rayon
- Cache-friendly component layout

## 🧬 Simulation Features

### Creature Behaviors
- **Basic Needs**: Hunger, thirst, energy management
- **Movement**: Pathfinding with obstacle avoidance
- **Social**: Conversations, relationships, group dynamics
- **Survival**: Resource gathering, threat avoidance

### World Systems
- **Biomes**: 8 distinct biomes with unique resources
- **Resources**: Food and water with regeneration
- **Day/Night**: Dynamic lighting and creature schedules
- **Weather**: Environmental effects on creature behavior

## 🛠️ Development

### Prerequisites

- Rust 1.70+ (latest stable)
- cargo installed

### Building from Source

```bash
# Debug build
cargo build

# Release build with optimizations
cargo build --release

# Run tests
cargo test

# Run clippy lints
cargo clippy --all-targets

# Format code
cargo fmt
```

### Project Structure

- `/src` - Source code
- `/assets` - Sprites and resources
- `/tests` - Integration tests
- `/docs` - Extensive documentation
- `/examples` - Example usage and demos

## 📚 Documentation

Comprehensive documentation available in `/docs`:

- [Start Here](/docs/START_HERE.md) - Role-based navigation
- [Development Guide](/docs/guides/DEVELOPMENT_GUIDE.md) - Code style and practices
- [Technical Guide](/docs/guides/TECHNICAL_GUIDE.md) - Architecture details
- [API Reference](/docs/reference/) - Component and system docs

## 🤝 Contributing

Contributions welcome! Please read our development guides:

1. Check existing issues or create a new one
2. Fork the repository
3. Create a feature branch
4. Follow code style guidelines
5. Add tests for new features
6. Submit a pull request

## 📝 License

[License information to be added]

## 🙏 Acknowledgments

Built with these excellent Rust crates:
- Bevy game engine
- egui for immediate mode UI
- glam for fast math
- And many more in Cargo.toml