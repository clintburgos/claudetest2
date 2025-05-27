# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Status

This is a creature simulation project built with Rust and Bevy. The project features autonomous creatures with emergent behaviors, social dynamics, and real-time visualization.

**Current State - Phase 1 Complete (100%):**
- ✅ Core simulation systems implemented (entity, time, spatial, events)
- ✅ Bevy ECS architecture with plugins for rendering, UI, camera, selection
- ✅ egui integration for immediate mode UI
- ✅ Basic creature behaviors and resource systems
- ✅ Save/Load system (F5 to save, F8 to load)
- ✅ Resource regeneration system
- ✅ Performance optimization for 500 creatures
- ✅ Graceful performance degradation
- ✅ Determinism verification system
- ✅ Observation goals system
- ✅ Error recovery and boundary systems
- ✅ All Phase 1 requirements complete and production ready

**Recent Updates:**
- Implemented all missing Phase 1 systems from docs/design/CRITICAL_SYSTEMS.md
- Added performance optimizations (LOD, update frequencies, quality degradation)
- Implemented save/load functionality with auto-save
- Added resource regeneration to maintain ecosystem balance
- Fixed all warnings and clippy issues
- Optimized for 500 creatures at 60 FPS (release mode)
- All tests passing, ready for production deployment

**Upcoming - Cartoon Isometric UI (Phase 2):**
- 🎨 Cartoon-style isometric graphics with expressive creatures
- 🌍 Biome-specific resources (berries in forests, cacti water in deserts)
- ✨ Particle effects for emotions and actions
- 💬 Speech bubbles for creature conversations
- 🧬 Visual genetic variations (size, color, patterns)
- 📐 Seamless isometric tile rendering
- See `/docs/implementation/CARTOON_ISOMETRIC_IMPLEMENTATION_PLAN.md` for details

## Quick Commands

```bash
# Run the application
cargo run --release

# Run tests
cargo test

# Check code style
cargo fmt --all -- --check
cargo clippy --all-targets

# Format code
cargo fmt --all

# Run with debug features
cargo run
```

## Architecture Overview

```
src/
├── components/     # ECS components (Creature, Position, Selected, etc.)
├── systems/        # Game logic (movement, decisions, conversations)
├── plugins/        # Bevy plugins 
│   ├── camera.rs       # Camera controls and zoom
│   ├── rendering.rs    # Sprite rendering and animations
│   ├── selection.rs    # Mouse selection and highlighting
│   ├── simulation.rs   # Core simulation orchestration
│   ├── spatial.rs      # Spatial indexing updates
│   ├── ui_egui.rs      # egui-based UI panels
│   └── debug.rs        # Debug visualizations (F1-F4)
├── simulation/     # Core simulation logic
├── core/           # Foundation systems
└── utils/          # Utilities and helpers
```

## Key Files to Know

- `src/main.rs` - Application entry point, plugin registration
- `src/plugins/ui_egui.rs` - Main UI implementation with egui
- `src/systems/decision.rs` - Creature AI and decision making
- `src/components/mod.rs` - All ECS component definitions
- `tests/integration_tests.rs` - Comprehensive integration tests

## Code Style Guidelines

1. **No Comments**: Don't add comments unless explicitly requested
2. **Prefer Editing**: Always prefer editing existing files over creating new ones
3. **Use Existing Patterns**: Follow the established patterns in the codebase
4. **Test Everything**: Write tests for new functionality
5. **Performance First**: This is a real-time simulation, optimize aggressively

## CRITICAL: Verification Requirements

**NEVER declare any task complete without verifying:**
1. `cargo test` - ALL tests must pass with no failures
2. `cargo run` - The application must compile and run without errors
3. `cargo clippy` - Should run without errors (warnings are acceptable)

If any of these commands fail, you MUST fix the issues before considering the task done. This includes:
- Compilation errors
- Test failures
- Runtime panics
- Missing imports or dependencies

Always run these verification steps after making changes and before reporting completion.

## Common Tasks

### Adding a New Component
1. Define in `src/components/mod.rs`
2. Add to relevant bundles if needed
3. Update systems that should process it

### Adding a New System
1. Create in `src/systems/` or add to existing file
2. Register in appropriate plugin in `src/plugins/`
3. Consider system ordering and dependencies

### Adding UI Elements
1. Edit `src/plugins/ui_egui.rs`
2. Add to appropriate panel (ControlPanel, StatsPanel, etc.)
3. Wire up to simulation state/resources

### Performance Optimization
1. Use spatial grid for proximity queries
2. Minimize component lookups
3. Batch similar operations
4. Profile with `cargo flamegraph`

## Testing

- Unit tests: In module files with `#[cfg(test)]`
- Integration tests: In `tests/` directory
- Run specific test: `cargo test test_name`
- Run with output: `cargo test -- --nocapture`

## Debugging Tips

- F1: Toggle FPS display
- F2: Toggle entity IDs
- F3: Toggle creature states
- F4: Toggle spatial grid
- F5: Save game (quicksave)
- F8: Load game (quicksave)
- F9: Toggle performance profiler overlay
- Use `info!()` and `RUST_LOG=info cargo run`

## Documentation

Extensive documentation in `/docs/`:
- `/docs/START_HERE.md` - Entry point for new developers
- `/docs/guides/` - Development and implementation guides
- `/docs/systems/` - System-specific documentation
- `/docs/reference/` - Technical references
- `/docs/implementation/CARTOON_ISOMETRIC_IMPLEMENTATION_PLAN.md` - Cartoon UI roadmap
- `/docs/implementation/CARTOON_GRAPHICS_IMPLEMENTATION_DESIGN.md` - Technical design
- `/docs/implementation/cartoon-isometric-mockup.svg` - Visual mockup

## Current Focus Areas

1. **Cartoon Graphics Implementation**: Transforming to isometric view with expressive cartoon-style creatures
2. **Biome-Specific Resources**: Implementing unique resources per biome (berries, cacti water, etc.)
3. **Visual Feedback Systems**: Adding particle effects, speech bubbles, and emotional displays
4. **Animation System**: Implementing sprite-based animations with genetic variations
5. **Performance**: Maintaining 500+ creatures at 60 FPS with new visual features

## Common Issues & Solutions

**Issue**: "Plugin already added" error
**Solution**: Check for duplicate plugin registrations in main.rs and plugin files

**Issue**: Asset loading errors
**Solution**: Ensure assets exist in `/assets/sprites/` directory

**Issue**: Performance drops with many creatures
**Solution**: Check spatial grid usage, use release builds, profile hotspots

## Contact & Support

For questions or issues, create a GitHub issue with:
- Clear description of the problem
- Steps to reproduce
- Expected vs actual behavior
- Relevant code snippets