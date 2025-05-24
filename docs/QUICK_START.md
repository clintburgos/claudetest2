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
# Testing
criterion = { version = "0.5", features = ["html_reports"] }
proptest = "1.4"
proptest-derive = "0.4"
rstest = "0.18"
approx = "0.5"
```

3. **Create the basic project structure**:
```bash
mkdir -p src/{core,creatures,social,rendering,ui}
mkdir -p assets/shaders
mkdir -p config
mkdir -p tests/{unit,integration,property,benchmarks}
```

4. **Set up test infrastructure**:
```bash
# Create test directories
mkdir -p tests/unit/{creatures,genetics,world,social}
mkdir -p tests/integration
mkdir -p tests/property
mkdir -p tests/benchmarks

# Create benches directory for Criterion
mkdir benches
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


## Starting with Test-Driven Development

### 1. Your First Test

Create `src/core/mod.rs`:
```rust
pub mod ecs;
```

Create `src/core/ecs.rs` and start with a test:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_starts_empty() {
        let world = World::new();
        assert_eq!(world.entity_count(), 0);
    }
}
```

### 2. Run the Test (RED)
```bash
cargo test
```

The test will fail - perfect! This is TDD working correctly.

### 3. Make it Pass (GREEN)
Add minimal implementation:
```rust
pub struct World {
    entity_count: usize,
}

impl World {
    pub fn new() -> Self {
        Self { entity_count: 0 }
    }
    
    pub fn entity_count(&self) -> usize {
        self.entity_count
    }
}
```

### 4. Verify Success
```bash
cargo test
```

Test passes! Continue this cycle for each feature.

## TDD Workflow Commands

```bash
# Run all tests
cargo test

# Run specific test module
cargo test core::ecs

# Run tests and show output
cargo test -- --nocapture

# Run tests in watch mode (install cargo-watch first)
cargo watch -x test

# Run only unit tests
cargo test --lib

# Run integration tests
cargo test --test '*'

# Generate test coverage report (install cargo-tarpaulin first)
cargo tarpaulin --out Html
```

## Next Steps

1. **Follow TDD for Phase 1**:
   - Read the [TDD Example](./TDD_EXAMPLE.md) for detailed walkthrough
   - Check [Testing Strategy](./TESTING_STRATEGY.md) for patterns
   - Write tests before implementing each component

2. **Implement Core ECS**:
   - Start with entity creation tests
   - Add component storage tests
   - Build system execution tests

3. **Build Incrementally**:
   - Each feature starts with a failing test
   - Implement only enough to pass
   - Refactor while keeping tests green

See [Implementation Plan](./implementation/IMPLEMENTATION_PLAN.md) for detailed phases.

---
*Last Updated: 2024-01-XX*
