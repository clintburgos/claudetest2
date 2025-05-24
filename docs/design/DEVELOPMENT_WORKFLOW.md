# Development Workflow Guide

## Overview
This guide outlines the development workflow, tools, and processes for contributing to the creature simulation project.

## Development Environment Setup

### Required Tools
```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable
rustup component add rustfmt clippy

# Development tools
cargo install cargo-watch
cargo install cargo-edit
cargo install bacon  # Better cargo workflow

# Performance tools
cargo install flamegraph
cargo install cargo-criterion

# Optional: mold linker for faster builds
# Ubuntu/Debian:
sudo apt install mold
# macOS:
brew install mold
```

### IDE Setup

#### VS Code
```json
// .vscode/settings.json
{
    "rust-analyzer.cargo.features": ["dev"],
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.imports.granularity.group": "crate",
    "rust-analyzer.imports.group.enable": true,
    "editor.formatOnSave": true,
    "[rust]": {
        "editor.defaultFormatter": "rust-lang.rust-analyzer"
    }
}
```

#### Recommended Extensions
- rust-analyzer
- crates
- Even Better TOML
- Error Lens
- GitLens

### Fast Compilation Setup

```toml
# .cargo/config.toml
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]

[target.x86_64-apple-darwin]
rustflags = ["-C", "link-arg=-fuse-ld=/opt/homebrew/bin/mold"]

[build]
# Use all CPU cores
jobs = 16

# Incremental compilation
incremental = true
```

```toml
# Cargo.toml - Development profile
[profile.dev]
opt-level = 1  # Some optimization for playable performance

[profile.dev.package."*"]
opt-level = 3  # Optimize dependencies

# Fast runtime, slow compilation
[profile.release-debug]
inherits = "release"
debug = true
```

## Development Workflow

### Feature Development

1. **Create Feature Branch**
```bash
git checkout -b feat/creature-emotions
```

2. **Continuous Development**
```bash
# Watch for changes and run checks
bacon clippy

# In another terminal, run the game
cargo watch -x "run --features dev"

# Run tests on change
cargo watch -x test
```

3. **Test Your Changes**
```bash
# Run all tests
cargo test

# Run specific test
cargo test creature::tests::test_emotions

# Run with output
cargo test -- --nocapture
```

4. **Performance Check**
```bash
# Run benchmarks
cargo criterion

# Profile with flamegraph
cargo flamegraph --dev
```

## Testing Strategy

### Test Organization
```
tests/
├── unit/           # Unit tests for individual components
├── integration/    # System integration tests
├── performance/    # Performance benchmarks
└── simulation/     # Full simulation tests
```

### Writing Tests

```rust
// Unit test example
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    
    #[test]
    fn test_creature_movement() {
        let mut world = World::new();
        
        let creature = world.spawn((
            Position(Vec2::ZERO),
            Velocity(Vec2::new(1.0, 0.0)),
        )).id();
        
        world.run_system_once(movement_system);
        
        let pos = world.get::<Position>(creature).unwrap();
        assert_eq!(pos.0.x, 1.0);
    }
}

// Integration test example
#[test]
fn test_creature_lifecycle() {
    let mut app = test_app();
    
    // Spawn creature
    let creature = spawn_test_creature(&mut app.world);
    
    // Run for 1000 frames
    for _ in 0..1000 {
        app.update();
    }
    
    // Verify creature still alive
    assert!(app.world.get::<Health>(creature).is_some());
}
```

## Debugging

### Debug Visualization

```rust
// Add debug systems during development
#[cfg(feature = "dev")]
app.add_systems(Update, (
    debug_draw_creature_paths,
    debug_show_spatial_grid,
    debug_print_performance,
));

// Debug components
#[cfg(feature = "dev")]
#[derive(Component)]
struct DebugPath {
    points: Vec<Vec2>,
    color: Color,
}
```

### Logging

```rust
// Use tracing for structured logging
use tracing::{debug, info, warn, error};

fn process_creature(creature: &Creature) {
    debug!(creature_id = ?creature.id, "Processing creature");
    
    if creature.health < 10.0 {
        warn!(
            creature_id = ?creature.id,
            health = creature.health,
            "Creature health critical"
        );
    }
}

// Set log level via environment
// RUST_LOG=creature_sim=debug cargo run
```

## Performance Profiling

### Using Tracy

```rust
// Add tracy client
[dependencies]
tracing-tracy = { version = "0.10", features = ["enable"] }

// Profile specific functions
#[tracing::instrument]
fn expensive_operation() {
    // Will appear in Tracy
}

// Manual profiling zones
fn update_creatures() {
    let _span = tracing::span!(tracing::Level::INFO, "update_creatures");
    // Code here
}
```

### Benchmarking

```rust
// benches/creature_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_pathfinding(c: &mut Criterion) {
    let mut group = c.benchmark_group("pathfinding");
    
    group.bench_function("find_path_small", |b| {
        let start = Vec2::ZERO;
        let end = Vec2::new(10.0, 10.0);
        b.iter(|| find_path(black_box(start), black_box(end)));
    });
    
    group.finish();
}

criterion_group!(benches, bench_pathfinding);
criterion_main!(benches);
```

## Code Quality

### Pre-commit Checks

```bash
#!/bin/bash
# .git/hooks/pre-commit

# Format code
cargo fmt --all -- --check

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
cargo test --all

# Check for TODO comments
if grep -r "TODO\|FIXME\|XXX" src/; then
    echo "Found TODO comments. Please address or create issues."
    exit 1
fi
```

### CI Pipeline

```yaml
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v2
    - uses: actions-rs/toolchain@v1
    - run: cargo fmt -- --check
    - run: cargo clippy -- -D warnings
    - run: cargo test --all
    - run: cargo bench --no-run
```

## Release Process

1. **Version Bump**
```bash
cargo set-version 0.2.0
```

2. **Update Changelog**
```markdown
# Changelog

## [0.2.0] - 2024-XX-XX
### Added
- Creature emotions system
- Weather effects

### Fixed
- Pathfinding in corners
```

3. **Tag Release**
```bash
git tag -a v0.2.0 -m "Release version 0.2.0"
git push origin v0.2.0
```

## Tips and Tricks

### Fast Iteration
```bash
# Skip optimization for faster builds
cargo run --profile dev-fast

# Run with specific creature count
CREATURE_COUNT=5000 cargo run --release

# Enable debug UI
cargo run --features dev-ui
```

### Common Issues

**Slow Compilation**
- Use `mold` linker
- Enable shared generics
- Minimize dependencies

**Performance Drops**
- Check system order
- Profile with Tracy
- Verify LOD is working

**Bevy Pitfalls**
- Remember `.chain()` for ordered systems
- Use `Commands` for spawning
- Check for system conflicts

---
*Last Updated: 2024-12-XX*