# Developer Cheat Sheet

Quick commands and references for common tasks.

## 🚀 Quick Commands

### Build & Run
```bash
# Build
cargo build

# Run (debug)
cargo run

# Run (release/optimized)
cargo run --release

# Check compilation
cargo check

# Run tests
cargo test

# Run benchmarks
cargo bench
```

### Git Workflow
```bash
# Create feature branch
git checkout -b feature/your-feature

# Stage changes
git add -A

# Commit with message
git commit -m "feat: add new behavior system"

# Push to remote
git push origin feature/your-feature
```

## 🎮 Application Controls

### Camera
- **Move**: WASD or Arrow keys
- **Zoom**: Q (out) / E (in)

### Debug Toggles
- **F1**: Toggle FPS display
- **F2**: Toggle entity IDs
- **F3**: Toggle creature states
- **F4**: Toggle spatial grid

### UI Windows (egui)
- Click buttons in top panel to toggle windows
- Stats, Debug, Controls windows available

## 📁 Key File Locations

### Configuration
- `src/config.rs` - All game constants and settings
- `Cargo.toml` - Dependencies and build settings

### Core Systems
- `src/plugins/` - Bevy plugins (camera, rendering, UI, etc.)
- `src/components/` - ECS components
- `src/systems/` - Game systems (simulation, movement, etc.)
- `src/rendering/` - Rendering pipeline

### Entry Points
- `src/main.rs` - Application entry
- `src/lib.rs` - Library root

## 🔧 Common Tasks

### Add a New Component
```rust
// In src/components/your_component.rs
#[derive(Component)]
pub struct YourComponent {
    pub field: f32,
}

// Don't forget to export in src/components/mod.rs
pub use your_component::YourComponent;
```

### Add a New System
```rust
// In src/systems/your_system.rs
pub fn your_system(
    query: Query<&YourComponent>,
) {
    for component in query.iter() {
        // System logic
    }
}

// Register in plugin
app.add_systems(Update, your_system);
```

### Add a Debug Visualization
```rust
// In debug overlay system
if debug_settings.show_your_feature {
    gizmos.circle_2d(position, radius, Color::GREEN);
}
```

## 🐛 Debugging Tips

### Performance Issues
1. Check with `cargo run --release`
2. Use `RUST_LOG=info` for timing logs
3. Profile with `cargo flamegraph`

### Entity Not Appearing
1. Check if sprite component added
2. Verify position is in camera view
3. Check z-order/layer

### System Not Running
1. Verify system added to plugin
2. Check system set ordering
3. Ensure run conditions met

## 📊 Performance Guidelines

### Target Metrics
- **FPS**: 60+ with 500 creatures
- **Frame time**: <16ms
- **Memory**: <100MB for Phase 1

### Quick Optimizations
- Use spatial grid for queries
- Batch similar operations
- Avoid per-frame allocations
- Use `changed()` queries

## 🔍 Finding Things

### Can't find a system/component?
```bash
# Search for struct/function definitions
rg "struct YourThing"
rg "fn your_function"

# Find usages
rg "YourThing::"
```

### Need documentation?
1. Check `docs/KEYWORD_INDEX.md`
2. Search in `docs/ALL_DOCUMENTATION.md`
3. Look at `docs/START_HERE.md`

## 📝 Code Style Quick Reference

### Naming
- Systems: `verb_noun_system` (e.g., `update_creature_sprites`)
- Components: `PascalCase` (e.g., `CreatureSprite`)
- Resources: `PascalCase` (e.g., `SpatialGrid`)

### Comments
- Avoid unless necessary
- Document "why", not "what"
- Use `///` for public API docs

### Error Handling
- Use `Result<T>` for fallible operations
- Log errors with appropriate level
- Provide context in error messages

## 🚨 Important Constants

From `src/config.rs`:
- Spatial grid cell size: `50.0`
- Fixed timestep: `1/60 sec`
- Max time scale: `10.0x`
- Creature search radius: `50.0`
- Interaction distance: `2.0`

## 💡 Pro Tips

1. **Run `cargo check` frequently** - Faster than full build
2. **Use `cargo watch`** - Auto-rebuild on changes
3. **Enable fast compile** - Already set in `.cargo/config.toml`
4. **Use `#[derive(Debug)]`** - Makes debugging easier
5. **Test in release mode** - Performance can differ significantly

---

Need more detail? Check the full documentation in `/docs/`!