# Quick Start Guide

## Setting Up the Development Environment

### Prerequisites
- Rust 1.75 or later
- Git
- A GPU that supports WebGPU (most modern GPUs)

### Initial Setup

1. **Initialize the Rust project**:
```bash
cd /Users/clintonburgos/Documents/Code/claudetest2
cargo init --name creature-sim
```

2. **Add core dependencies to Cargo.toml**:
```toml
[package]
name = "creature-sim"
version = "0.1.0"
edition = "2021"

[dependencies]
# Graphics & UI
wgpu = "0.18"
winit = "0.29"
egui = "0.25"
egui-wgpu = "0.25"
egui-winit = "0.25"

# Core systems
rand = "0.8"
rand_chacha = "0.3"  # For reproducible randomness
serde = { version = "1.0", features = ["derive"] }
bincode = "1.3"
ron = "0.8"

# Utilities
anyhow = "1.0"
log = "0.4"
env_logger = "0.11"

# Math
glam = "0.25"  # For vectors and matrices

# Time
instant = "0.1"  # Cross-platform time

[dev-dependencies]
criterion = "0.5"
proptest = "1.4"
```

3. **Create the basic project structure**:
```bash
mkdir -p src/{core,creatures,social,rendering,ui}
mkdir -p assets/shaders
mkdir -p config
```

## First Implementation Steps

### 1. Core ECS Module (src/core/ecs.rs)
Start with a simple Entity-Component-System:

```rust
// Basic entity is just an ID
pub type Entity = u32;

// Component storage trait
pub trait Component: 'static + Send + Sync {}

// System trait
pub trait System {
    fn update(&mut self, world: &mut World, dt: f32);
}

// World holds all entities and components
pub struct World {
    // Implementation details...
}
```

### 2. Time Controller (src/core/time.rs)
Implement the time scaling system:

```rust
pub struct TimeController {
    scale: f32,
    accumulated_time: f32,
    current_tick: u64,
}
```

### 3. Basic Rendering (src/main.rs)
Set up the window and basic rendering:

```rust
use winit::{event_loop::EventLoop, window::WindowBuilder};

fn main() {
    env_logger::init();
    
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Creature Simulation")
        .build(&event_loop)
        .unwrap();
    
    // Set up wgpu...
    // Run simulation loop...
}
```

## Next Steps

After basic setup:
1. Implement world grid system
2. Add creature entities with position
3. Create basic needs system
4. Add simple movement
5. Implement basic UI with egui

See [Implementation Plan](./implementation/IMPLEMENTATION_PLAN.md) for detailed phases.

---
*Last Updated: 2024-01-XX*
