# Documentation Index

## Quick Start
- [README](README.md) - Project introduction
- [Quick Start Guide](QUICK_START.md) - Get up and running quickly
- [Project Overview](PROJECT_OVERVIEW.md) - High-level project vision and goals

## Development Guides
- [Development Guide](guides/DEVELOPMENT_GUIDE.md) - Code style, best practices, and workflow
- [Technical Guide](guides/TECHNICAL_GUIDE.md) - Architecture, Rust setup, and implementation
- [Testing Guide](guides/TESTING_GUIDE.md) - Testing strategy and TDD approach
- [Debugging Guide](DEBUGGING_GUIDE.md) - Troubleshooting common issues

## System Documentation

### Core Systems
- [Creature System](systems/CREATURE_SYSTEM.md) - Core creature design and lifecycle
- [Decision System](systems/DECISION_SYSTEM.md) - AI decision-making architecture
- [Conversation System](systems/CONVERSATION_SYSTEM.md) - Communication and dialogue
- [World System](systems/WORLD_SYSTEM.md) - Procedural generation and biomes

### Biological Systems
- [Genetics System](systems/GENETICS_SYSTEM.md) - DNA, inheritance, and evolution
- [Reproduction System](systems/REPRODUCTION_SYSTEM.md) - Mating and offspring
- [Disease & Health System](systems/DISEASE_HEALTH_SYSTEM.md) - Health mechanics

### Social Systems
- [Social System](systems/SOCIAL_SYSTEM.md) - Relationships and group dynamics
- [Territory System](systems/TERRITORY_SYSTEM.md) - Territory claiming and defense
- [Tool Use System](systems/TOOL_USE_SYSTEM.md) - Tool creation and usage

### Technical Systems
- [UI System](systems/UI_SYSTEM.md) - User interface and controls
- [Resource System](systems/RESOURCE_SYSTEM.md) - Resource management
- [Audio System](systems/AUDIO_SYSTEM.md) - Sound effects and music
- [Combat System](systems/COMBAT_SYSTEM.md) - Fighting mechanics

## Reference Documentation
- [Configuration](reference/CONFIGURATION.md) - All system constants and settings
- [Performance Guide](reference/PERFORMANCE.md) - Optimization strategies and targets
- [Memory Architecture](reference/MEMORY_ARCHITECTURE.md) - Memory management and persistence
- [Design Decisions](reference/DESIGN_DECISIONS.md) - Key architectural choices
- [Time Scaling](reference/TIME_SCALING.md) - Time control implementation
- [System Integration](reference/SYSTEM_INTEGRATION_ARCHITECTURE.md) - How systems interact
- [Error Handling](reference/ERROR_HANDLING_SYSTEM.md) - Error management patterns
- [Input System](reference/INPUT_SYSTEM_IMPLEMENTATION.md) - Input handling
- [Asset Pipeline](reference/ASSET_PIPELINE.md) - Asset loading and management

## Visual Design
- [Diagrams](diagrams/) - System architecture and UI mockups
  - [Conversation System Diagram](diagrams/conversation-system-diagram.svg)
  - [Decision Making Architecture](diagrams/decision-making-architecture.svg)
  - [Interface Controls Mockup](diagrams/interface-controls-mockup.svg)
  - [Isometric Creatures Mockup](diagrams/isometric-creatures-mockup.svg)
  - [Isometric World Mockup](diagrams/isometric-world-mockup.svg)

## Implementation
- [Implementation Plan](IMPLEMENTATION_PLAN.md) - Development roadmap
- [Summary & Next Steps](SUMMARY_AND_NEXT_STEPS.md) - Current status

## Architecture Overview

```
docs/
├── Quick Start & Overview
│   ├── README.md
│   ├── QUICK_START.md
│   └── PROJECT_OVERVIEW.md
│
├── guides/              # How-to guides
│   ├── DEVELOPMENT_GUIDE.md
│   ├── TECHNICAL_GUIDE.md
│   └── TESTING_GUIDE.md
│
├── systems/             # System documentation
│   ├── Core Systems
│   ├── Biological Systems
│   ├── Social Systems
│   └── Technical Systems
│
├── reference/           # Technical reference
│   ├── Performance & Optimization
│   ├── Architecture Decisions
│   └── Implementation Details
│
└── diagrams/           # Visual documentation
```

## Key Design Principles

1. **Performance First** - 60+ FPS with 5000+ creatures
2. **Emergent Behavior** - Complex behaviors from simple rules
3. **Meaningful Communication** - Conversations affect survival
4. **Time Scale Flexibility** - Real-time to generational viewing
5. **Modular Architecture** - Clean system separation with ECS

## Technology Stack

- **Game Engine**: Bevy (Rust)
- **UI Framework**: egui
- **Rendering**: bevy_ecs_tilemap (isometric)
- **Performance**: Rayon for parallelization
- **Build**: Cargo with optimized release builds