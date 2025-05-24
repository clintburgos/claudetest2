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
  - `isometric-creatures-mockup.svg` - Visual mockup of UI concepts
  - `isometric-world-mockup.svg` - Visual mockup of world biomes
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

See `/docs/design/` for complete design specifications.