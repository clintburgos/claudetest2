# Design Decisions Log

This document records all significant design decisions made during the project development.

## Decision Template

### DD-[NUMBER]: [DECISION TITLE]
- **Date**: [Date]
- **Status**: [Proposed/Accepted/Rejected/Superseded]
- **Deciders**: [Who made this decision]

#### Context
[What is the background and context for this decision?]

#### Decision
[What is the decision?]

#### Consequences
[What are the positive and negative consequences of this decision?]

#### Alternatives Considered
[What other options were considered and why were they rejected?]

### DD-002: Entity-Component-System (ECS) Architecture
- **Date**: 2024-01-XX
- **Status**: Proposed
- **Deciders**: Project team

#### Context
Need a flexible architecture that can handle hundreds of creatures with various traits and behaviors while maintaining performance.

#### Decision
Implement an Entity-Component-System architecture where:
- Entities: Creatures, resources, world objects
- Components: Needs, DNA, Position, Health, Social Memory, etc.
- Systems: Movement, Reproduction, Social, Metabolism, etc.

#### Consequences
Positive:
- Highly modular and extensible
- Excellent performance for large numbers of entities
- Easy to add new behaviors and traits
- Natural parallelization opportunities

Negative:
- More complex initial setup
- Less intuitive for simple behaviors
- Requires careful system ordering

#### Alternatives Considered
- Traditional OOP with Creature classes: Too rigid for varied behaviors
- Actor model: Overhead too high for 1000+ creatures

---

### DD-003: Genetic Representation
- **Date**: 2024-01-XX
- **Status**: Proposed
- **Deciders**: Project team

#### Context
Need to represent creature traits in a way that supports inheritance, mutation, and affects behavior.

#### Decision
Use a gene-based system where:
- Each gene is a float value between 0-1
- Genes map to specific traits (speed, size, sociability, etc.)
- Genes can have dominance/recessiveness
- Chromosomes group related genes
- Support for both discrete and continuous traits

#### Consequences
Positive:
- Realistic genetic model
- Easy to implement mutations
- Smooth trait variations
- Natural sexual reproduction implementation

Negative:
- More complex than simple attribute inheritance
- Requires balancing gene effects

#### Alternatives Considered
- Simple attribute averaging: Too simplistic, no emergent complexity
- Binary DNA strings: Harder to map to meaningful behaviors

---

### DD-004: Needs-Based Decision System
- **Date**: 2024-01-XX
- **Status**: Proposed
- **Deciders**: Project team

#### Context
Creatures need to make decisions based on multiple competing needs and environmental factors.

#### Decision
Implement a utility-based AI system where:
- Each need has a current value and urgency curve
- Actions have expected utility for each need
- Creatures calculate weighted utility scores
- Personality genes affect weight calculations
- Social interactions can modify utility calculations

#### Consequences
Positive:
- Emergent, believable behaviors
- Easy to tune and debug
- Naturally handles competing needs
- Personality variety from genes

Negative:
- Computationally intensive for many creatures
- Requires careful utility function design

#### Alternatives Considered
- Finite State Machines: Too rigid for complex behaviors
- Neural Networks: Too opaque, hard to debug
- Behavior Trees: Less emergent behavior

---
### DD-005: Conversation System Architecture
- **Date**: 2024-01-XX
- **Status**: Proposed
- **Deciders**: Project team

#### Context
Creatures need to communicate and influence each other's decisions through conversations, which is a unique feature of this simulation.

#### Decision
Implement a symbolic communication system where:
- Creatures exchange "concepts" rather than natural language
- Concepts include: needs states, warnings, social bonds, knowledge
- Each creature has communication traits (honesty, persuasiveness, gullibility)
- Conversations affect internal weights and memory
- Simple grammar: [Subject] [Verb] [Object] [Modifier]

#### Consequences
Positive:
- Computationally efficient vs. natural language
- Clear influence mechanics
- Allows for "language evolution"
- Can visualize communication patterns

Negative:
- Less intuitive for users than natural language
- Requires UI to "translate" conversations

#### Alternatives Considered
- Natural language processing: Too computationally expensive
- Simple state broadcasting: Not rich enough for meaningful influence
- Pre-scripted dialogues: No emergent communication

---

### DD-006: Multi-Scale Time System
- **Date**: 2024-01-XX
- **Status**: Proposed
- **Deciders**: Project team

#### Context
Users need to observe both immediate creature actions and long-term evolutionary patterns, requiring multiple time scales.

#### Decision
Implement a hierarchical time system with these scales:
1. **Action Time** (1x): Individual creature decisions
2. **Day/Night Cycle** (60x): Resource regeneration, sleep
3. **Life Cycle** (1000x): Birth to death progression
4. **Generational** (10000x): Evolution visualization
5. **Epoch** (100000x): Major environmental changes

With smooth interpolation between scales and LOD (Level of Detail) adjustments.

#### Consequences
Positive:
- Unique feature for observing different phenomena
- Efficient computation at high speeds via LOD
- Engaging for different user interests

Negative:
- Complex to implement smooth transitions
- UI challenge to show relevant info at each scale
- Potential for temporal inconsistencies

#### Alternatives Considered
- Fixed time speeds: Less flexible for users
- Discrete jumps only: Jarring user experience
- Single time scale: Misses key simulation aspects

---

### DD-007: Technology Stack
- **Date**: 2024-01-XX
- **Status**: Proposed
- **Deciders**: Project team

#### Context
Need to choose implementation technologies that support performance requirements and rich visualization.

#### Decision
Primary stack:
- **Language**: Rust (for core simulation)
- **Graphics**: wgpu (cross-platform, modern graphics)
- **UI Framework**: egui (immediate mode, good for data vis)
- **Scripting**: Rhai (for moddable behaviors)
- **Data**: bincode for save files, ron for configs

#### Consequences
Positive:
- Excellent performance for simulation
- Memory safety for complex systems
- Modern graphics capabilities
- Cross-platform support

Negative:
- Steeper learning curve
- Longer initial development
- Smaller ecosystem than Unity/Godot

#### Alternatives Considered
- Unity/C#: Overhead too high for massive simulations
- JavaScript/WebGL: Performance limitations
- C++: Memory safety concerns for complex system
- Godot: Less suitable for data-heavy simulations

---### DD-008: Test-Driven Development Approach
- **Date**: 2024-01-XX
- **Status**: Accepted
- **Deciders**: Project team

#### Context
Building a complex simulation with emergent behaviors requires confidence that individual systems work correctly and continue working as we add features. Need a systematic approach to ensure quality and catch regressions early.

#### Decision
Adopt Test-Driven Development (TDD) methodology:
1. Write tests BEFORE implementation
2. Follow Red-Green-Refactor cycle
3. Maintain >80% code coverage
4. Use property-based testing for genetic algorithms
5. Create integration tests for system interactions
6. Benchmark performance-critical paths

#### Consequences
Positive:
- Confidence in correctness
- Better API design (test-first forces good interfaces)
- Living documentation through tests
- Easier refactoring
- Catch emergent behavior issues early
- Performance regressions detected

Negative:
- Slower initial development
- Need to learn property testing
- Test maintenance overhead
- May need to refactor tests as design evolves

#### Alternatives Considered
- Test-after approach: Miss design benefits of TDD
- Manual testing only: Not scalable for complex systems
- Minimal testing: Too risky for simulation correctness

---