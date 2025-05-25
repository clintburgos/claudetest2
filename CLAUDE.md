# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Status

This is a creature simulation project in active development. Phase 1 implementation is underway.

**Current State:**
- ✅ Core simulation systems implemented (weeks 1-6)
- ✅ Bevy graphical application structure ready
- ✅ egui UI integration complete
- 🚧 Working on Phase 1 weeks 7-8 (presentation layer)
- 📍 See `/docs/PHASE_1_IMPLEMENTATION_GUIDE.md` for current phase details

## Documentation Navigation

**New engineers should start with these navigation tools:**
- `/docs/START_HERE.md` ⭐ - Role-based guide to find relevant docs
- `/docs/KEYWORD_INDEX.md` 🔍 - Search documentation by topic/keyword
- `/docs/ALL_DOCUMENTATION.md` 📚 - Complete list of all 100+ docs
- `/docs/DEVELOPER_CHEAT_SHEET.md` 📝 - Quick commands and references

**Documentation structure:**
- `/docs/INDEX.md` - Complete documentation index and navigation
- `/docs/guides/` - Development and implementation guides
  - `DEVELOPMENT_GUIDE.md` - Code style, best practices, workflow
  - `TECHNICAL_GUIDE.md` - Rust architecture and implementation
  - `TESTING_GUIDE.md` - Testing strategy and TDD approach
- `/docs/systems/` - System-specific documentation
  - Core systems (Creature, Decision, Conversation, World)
  - Biological systems (Genetics, Reproduction, Disease)
  - Social systems (Social, Territory, Tool Use)
  - Technical systems (UI, Resource, Audio, Combat)
- `/docs/reference/` - Technical reference and architecture
  - `PERFORMANCE.md` - Comprehensive performance optimization
  - `DESIGN_DECISIONS.md` - Key architectural choices
  - Additional technical references
- `/docs/diagrams/` - Visual mockups and architecture diagrams

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
- **Performance**: Aggressive optimization for 5000+ creatures at 60+ FPS
  - Spatial indexing for O(log n) queries
  - LOD system for animations and AI
  - Parallel processing with Rayon
  - Cache-friendly component design
- **Modular Architecture**: Separated simulation, world, rendering, and UI systems

See `/docs/INDEX.md` for complete documentation navigation.