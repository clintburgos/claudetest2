# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Status

This is a creature simulation project with active design documentation. Key design decisions have been made for the UI, creature systems, and world generation.

## Architecture

Project structure:
- `/docs/design/` - Design documentation and specifications
  - `UI_DESIGN.md` - Comprehensive UI/visual design specification
  - `CREATURE_DESIGN.md` - Creature mechanics and systems
  - `WORLD_DESIGN.md` - Procedural world generation and biomes
  - `CONTROLS_INTERFACE.md` - Control schemes and interface layout
  - `TECHNICAL_ARCHITECTURE.md` - Rust implementation architecture
  - `RUST_GETTING_STARTED.md` - Quick start guide for development
  - `isometric-creatures-mockup.svg` - Visual mockup of UI concepts
  - `isometric-world-mockup.svg` - Visual mockup of world biomes
  - `interface-controls-mockup.svg` - Interface layout and controls
  - Additional system design docs (genetics, social, etc.)

## Key Design Decisions

### Visual Design
- **View**: Isometric perspective (30°/45° angle)
- **Art Style**: Cartoonish, over-exaggerated expressions
- **Creature Visibility**: Multiple systems ensure creatures remain visible when behind objects
- **Expression System**: Rich emotional displays with particles, colors, and animations

### World Design
- **Procedural Generation**: Seed-based world generation using Perlin noise
- **Biomes**: 8 distinct biomes with realistic transitions
- **Animated Environment**: Swaying grass, trees, water ripples, weather effects
- **Day/Night Cycle**: Dynamic lighting and weather systems

### Controls & Interface
- **Multi-scale Navigation**: Seamless zoom from world overview to individual creatures
- **Time Control**: 6 speed settings from pause to 1000x generational time
- **Smart Selection**: Click to select, follow mode, multi-select capabilities
- **Data Visualization**: 4 main views (Overview, Population, Genetics, Trends)
- **Adaptive UI**: Interface elements scale with zoom level

### Technical Implementation
- **Game Engine**: Bevy (Rust) with ECS architecture
- **UI Framework**: egui for immediate mode GUI
- **Rendering**: bevy_ecs_tilemap for isometric tiles
- **Parallelization**: Automatic system parallelization for performance
- **Modular Architecture**: Separated simulation, world, rendering, and UI systems

See `/docs/design/` for complete design specifications.