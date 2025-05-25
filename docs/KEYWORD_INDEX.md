# Documentation Keyword Index

This keyword-based index helps you quickly find relevant documentation by topic or search term.

## Architecture & Design

### Core Architecture
- **ECS (Entity Component System)**: [Technical Guide](guides/TECHNICAL_GUIDE.md), [System Integration](reference/SYSTEM_INTEGRATION_ARCHITECTURE.md)
- **Bevy Framework**: [Technical Guide](guides/TECHNICAL_GUIDE.md), [Bevy Migration Plan](/BEVY_MIGRATION_PLAN.md)
- **System Design**: [Design Decisions](reference/DESIGN_DECISIONS.md), [System Integration](reference/SYSTEM_INTEGRATION_ARCHITECTURE.md)
- **Architecture Patterns**: [Technical Guide](guides/TECHNICAL_GUIDE.md), [Phase 1 Architecture](design/PHASE_1_ARCHITECTURE.md)
- **Class Diagrams**: [Phase 1 Class Diagram](design/PHASE_1_CLASS_DIAGRAM.md)
- **Critical Systems**: [Critical Systems](design/CRITICAL_SYSTEMS.md)

### Visual Design
- **Isometric View**: [UI System](systems/UI_SYSTEM.md), [World Mockup](diagrams/isometric-world-mockup.svg)
- **Art Style**: [UI System](systems/UI_SYSTEM.md), [Creature Mockup](diagrams/isometric-creatures-mockup.svg)
- **UI/UX Design**: [UI System](systems/UI_SYSTEM.md), [Interface Controls](diagrams/interface-controls-mockup.svg)
- **egui Integration**: [EGUI Integration](EGUI_INTEGRATION.md), [UI System](systems/UI_SYSTEM.md)

## Implementation Guides

### Getting Started
- **Quick Start**: [Quick Start Guide](QUICK_START.md), [README](README.md)
- **Project Setup**: [Technical Guide](guides/TECHNICAL_GUIDE.md), [Final Setup Summary](FINAL_SETUP_SUMMARY.md)
- **Development Workflow**: [Development Guide](guides/DEVELOPMENT_GUIDE.md)
- **Rust Setup**: [Technical Guide](guides/TECHNICAL_GUIDE.md)

### Development Process
- **Code Style**: [Development Guide](guides/DEVELOPMENT_GUIDE.md)
- **Best Practices**: [Development Guide](guides/DEVELOPMENT_GUIDE.md)
- **Testing Strategy**: [Testing Guide](guides/TESTING_GUIDE.md), [Testing Strategy](TESTING_STRATEGY.md)
- **TDD (Test-Driven Development)**: [Testing Guide](guides/TESTING_GUIDE.md)
- **Debugging**: [Debugging Guide](DEBUGGING_GUIDE.md)
- **Error Handling**: [Error Handling System](reference/ERROR_HANDLING_SYSTEM.md)

### Implementation Planning
- **Roadmap**: [Implementation Plan](implementation/IMPLEMENTATION_PLAN.md)
- **Phase 1**: [Phase 1 Implementation Guide](PHASE_1_IMPLEMENTATION_GUIDE.md), [Phase 1 Summary](implementation/PHASE1_SUMMARY.md)
- **Improvement Plan**: [Improvement Plan](implementation/IMPROVEMENT_PLAN.md), [Quick Reference](implementation/IMPROVEMENT_QUICK_REFERENCE.md)
- **Current Status**: [Summary & Next Steps](SUMMARY_AND_NEXT_STEPS.md), [Implementation Ready Summary](IMPLEMENTATION_READY_SUMMARY.md)

## Systems & Components

### Core Systems
- **Creatures**: [Creature System](systems/CREATURE_SYSTEM.md)
  - Life cycle, attributes, behaviors
  - Expression system, emotional states
- **AI/Decision Making**: [Decision System](systems/DECISION_SYSTEM.md), [Decision Architecture](diagrams/decision-making-architecture.svg)
  - State machines, behavior trees
  - Goal evaluation, action selection
- **Communication**: [Conversation System](systems/CONVERSATION_SYSTEM.md), [Conversation Diagram](diagrams/conversation-system-diagram.svg)
  - Dialogue system, emotional communication
  - Knowledge transfer, social bonding
- **World Generation**: [World System](systems/WORLD_SYSTEM.md)
  - Procedural generation, biomes
  - Environmental effects, weather

### Biological Systems
- **Genetics/DNA**: [Genetics System](systems/GENETICS_SYSTEM.md)
  - Inheritance, mutations, evolution
  - Trait expression, genetic drift
- **Reproduction**: [Reproduction System](systems/REPRODUCTION_SYSTEM.md)
  - Mating behaviors, offspring
  - Sexual selection, parental care
- **Health/Disease**: [Disease & Health System](systems/DISEASE_HEALTH_SYSTEM.md)
  - Immune system, contagion
  - Aging, death mechanics

### Social Systems
- **Relationships**: [Social System](systems/SOCIAL_SYSTEM.md)
  - Friendship, rivalry, family bonds
  - Trust, reputation, influence
- **Groups/Tribes**: [Group Dynamics System](systems/GROUP_DYNAMICS_SYSTEM.md)
  - Leadership, cooperation
  - Cultural transmission
- **Territory**: [Territory System](systems/TERRITORY_SYSTEM.md)
  - Claiming, defending, resources
  - Migration patterns
- **Culture**: [Cultural Evolution System](systems/CULTURAL_EVOLUTION_SYSTEM.md)
  - Traditions, knowledge preservation
  - Tool use propagation

### Resource Management
- **Resources**: [Resource System](systems/RESOURCE_SYSTEM.md), [Resource Taxonomy](systems/RESOURCE_TAXONOMY.md)
  - Food, water, shelter
  - Gathering, storage, sharing
- **Tools**: [Tool Use System](systems/TOOL_USE_SYSTEM.md)
  - Creation, usage, teaching
  - Tool evolution, specialization

### Technical Systems
- **Time Control**: [Time Scaling](reference/TIME_SCALING.md), [Time Scaling System](systems/TIME_SCALING_SYSTEM.md)
  - Speed controls (pause to 1000x)
  - Generational viewing
- **Spatial Indexing**: [Spatial Indexing System](systems/SPATIAL_INDEXING_SYSTEM.md)
  - Efficient neighbor queries
  - Collision detection
- **LOD (Level of Detail)**: [LOD System](systems/LOD_SYSTEM.md)
  - Performance optimization
  - Distance-based quality
- **Animation**: [Animation System](systems/ANIMATION_SYSTEM.md), [Particle System](systems/PARTICLE_SYSTEM.md)
  - Creature animations, expressions
  - Environmental effects
- **Events**: [Event System](systems/EVENT_SYSTEM.md), [Notification System](systems/NOTIFICATION_SYSTEM.md)
  - System communication
  - User notifications

### UI & Controls
- **Interface**: [UI System](systems/UI_SYSTEM.md), [Interface Mockup](diagrams/interface-controls-mockup.svg)
- **Input Handling**: [Input System Implementation](reference/INPUT_SYSTEM_IMPLEMENTATION.md)
- **Camera Controls**: [UI System](systems/UI_SYSTEM.md)
- **Data Visualization**: [UI System](systems/UI_SYSTEM.md), [Observer Tools](systems/OBSERVER_TOOLS.md)
- **Tutorial**: [Tutorial & Onboarding System](systems/TUTORIAL_ONBOARDING_SYSTEM.md)

### Persistence & Data
- **Save/Load**: [Save Load System](systems/SAVE_LOAD_SYSTEM.md)
- **Memory Management**: [Memory Architecture](reference/MEMORY_ARCHITECTURE.md)
- **Cache**: [Cache Management System](systems/CACHE_MANAGEMENT_SYSTEM.md)
- **Configuration**: [Configuration](reference/CONFIGURATION.md)

### Audio & Effects
- **Sound**: [Audio System](systems/AUDIO_SYSTEM.md)
- **Music**: [Audio System](systems/AUDIO_SYSTEM.md)
- **Particles**: [Particle System](systems/PARTICLE_SYSTEM.md)

### Combat & Conflict
- **Fighting**: [Combat System](systems/COMBAT_SYSTEM.md)
- **Conflict Resolution**: [Combat System](systems/COMBAT_SYSTEM.md), [Territory System](systems/TERRITORY_SYSTEM.md)

## Performance & Optimization

### Performance Targets
- **FPS Goals**: [Performance](reference/PERFORMANCE.md), [Performance Targets](reference/PERFORMANCE_TARGETS.md)
- **Creature Count**: [Performance](reference/PERFORMANCE.md) (5000+ at 60+ FPS)
- **Memory Usage**: [Memory Architecture](reference/MEMORY_ARCHITECTURE.md)

### Optimization Techniques
- **Spatial Indexing**: [Spatial Indexing System](systems/SPATIAL_INDEXING_SYSTEM.md)
- **LOD System**: [LOD System](systems/LOD_SYSTEM.md)
- **Parallel Processing**: [Performance](reference/PERFORMANCE.md)
- **Cache Optimization**: [Cache Management System](systems/CACHE_MANAGEMENT_SYSTEM.md)
- **Component Design**: [Technical Guide](guides/TECHNICAL_GUIDE.md)

### Profiling & Monitoring
- **Performance Profiling**: [Performance](reference/PERFORMANCE.md)
- **Debug Tools**: [Debugging Guide](DEBUGGING_GUIDE.md), [Observer Tools](systems/OBSERVER_TOOLS.md)

## Development Workflow

### Version Control
- **Git Workflow**: [Development Guide](guides/DEVELOPMENT_GUIDE.md)
- **Code Review**: [Development Guide](guides/DEVELOPMENT_GUIDE.md)
- **Principal Engineer Review**: [Phase 1 Principal Engineer Review](implementation/PHASE1_PRINCIPAL_ENGINEER_REVIEW.md)

### Build & Deploy
- **Build Process**: [Technical Guide](guides/TECHNICAL_GUIDE.md)
- **Asset Pipeline**: [Asset Pipeline](reference/ASSET_PIPELINE.md)
- **Sprites**: [Sprites README](../assets/sprites/README.md)

### Testing & Quality
- **Unit Testing**: [Testing Guide](guides/TESTING_GUIDE.md)
- **Integration Testing**: [Testing Guide](guides/TESTING_GUIDE.md)
- **Performance Testing**: [Performance](reference/PERFORMANCE.md)
- **TDD Examples**: [Testing Guide](guides/TESTING_GUIDE.md)

### Documentation
- **Documentation Index**: [INDEX](INDEX.md)
- **Design Completion**: [Design Completion Summary](DESIGN_COMPLETION_SUMMARY.md)
- **Weekly Summaries**: [Week 5-6 Summary](/WEEK_5_6_IMPLEMENTATION_SUMMARY.md)

## Advanced Topics

### Modding & Extensions
- **Mod System**: [Mod System API](systems/MOD_SYSTEM_API.md)
- **Plugin Architecture**: [System Integration](reference/SYSTEM_INTEGRATION_ARCHITECTURE.md)

### System Interactions
- **System Matrix**: [System Interaction Matrix](reference/SYSTEM_INTERACTION_MATRIX.md)
- **Integration Patterns**: [System Integration Architecture](reference/SYSTEM_INTEGRATION_ARCHITECTURE.md)

### Presentation Layer
- **Setup Guide**: [Presentation Layer Setup](PRESENTATION_LAYER_SETUP.md)
- **Rendering Pipeline**: [Technical Guide](guides/TECHNICAL_GUIDE.md)

## Quick Reference by File Type

### Overviews & Summaries
- Project: [Project Overview](PROJECT_OVERVIEW.md), [README](README.md)
- Implementation: [Implementation Summary](/IMPLEMENTATION_SUMMARY.md), [Phase 1 Review](/PHASE_1_REVIEW.md)
- Design: [Design Completion Summary](DESIGN_COMPLETION_SUMMARY.md)

### Step-by-Step Guides
- Getting Started: [Quick Start](QUICK_START.md)
- Development: [Development Guide](guides/DEVELOPMENT_GUIDE.md)
- Testing: [Testing Guide](guides/TESTING_GUIDE.md)
- Implementation: [Phase 1 Implementation Guide](PHASE_1_IMPLEMENTATION_GUIDE.md)

### Technical References
- Architecture: [Technical Guide](guides/TECHNICAL_GUIDE.md), [System Integration](reference/SYSTEM_INTEGRATION_ARCHITECTURE.md)
- Performance: [Performance](reference/PERFORMANCE.md), [Performance Targets](reference/PERFORMANCE_TARGETS.md)
- Configuration: [Configuration](reference/CONFIGURATION.md)

### Visual Documentation
- System Diagrams: [diagrams/](diagrams/)
- UI Mockups: [Interface Controls](diagrams/interface-controls-mockup.svg), [Isometric World](diagrams/isometric-world-mockup.svg)

## Search Tips

1. **Use Ctrl+F** to search for specific keywords in this index
2. **Common abbreviations**: ECS, LOD, TDD, UI, UX, AI, FPS
3. **Check multiple related terms**: "creature" → "entity", "animal", "organism"
4. **Look in categories**: Systems are grouped by type (Core, Biological, Social, etc.)
5. **Cross-references**: Many topics appear in multiple documents