# Design Decisions

This document captures key design decisions made during the project and the rationale behind them.

## Core Philosophy

### Emergent Complexity from Simple Rules
**Decision**: Focus on simple, interacting systems rather than scripted behaviors.

**Rationale**: 
- More believable creature behaviors
- Infinite variety of scenarios
- Player discovery and surprise
- Easier to balance and maintain

**Trade-offs**:
- Harder to predict outcomes
- May produce unexpected edge cases
- Requires careful system tuning

### Performance First Architecture
**Decision**: Design every system with performance as a primary constraint.

**Rationale**:
- Enables 1000+ creature simulations
- Maintains 60+ FPS for smooth gameplay
- Allows for time acceleration features
- Provides headroom for future features

**Trade-offs**:
- More complex implementation
- Occasional approximations needed
- Higher initial development time

### Observable Cause and Effect
**Decision**: Every creature action should have clear, visible causes.

**Rationale**:
- Players can understand creature behavior
- Builds empathy with creatures
- Educational value
- Debugging is easier

**Trade-offs**:
- Limits some optimizations
- Requires extensive visualization systems
- May expose "gamey" mechanics

## Technical Decisions

### Entity Component System (Bevy)
**Decision**: Use ECS architecture with Bevy game engine.

**Rationale**:
- Excellent performance characteristics
- Natural parallelization
- Flexible composition
- Active community

**Trade-offs**:
- Steeper learning curve
- Less mature than traditional engines
- Rust learning requirement

### Isometric View
**Decision**: Fixed isometric camera instead of full 3D.

**Rationale**:
- Classic strategy game feel
- Easier to read creature actions
- Better performance (no occlusion)
- Simpler art requirements

**Trade-offs**:
- No camera rotation
- Limited perspective options
- Some depth ambiguity

### Utility-based AI
**Decision**: Creatures use utility functions for decision making.

**Rationale**:
- Nuanced, contextual decisions
- Easy to tune and balance
- Personality integration
- Explainable choices

**Trade-offs**:
- More complex than FSMs
- Requires careful utility design
- Can be computationally expensive

## Gameplay Decisions

### No Direct Control
**Decision**: Players observe but don't directly control creatures.

**Rationale**:
- Maintains simulation purity
- Creatures feel autonomous
- Focus on ecosystem management
- Unique gameplay niche

**Trade-offs**:
- Less immediate player agency
- May frustrate some players
- Harder to create directed experiences

### Real-time with Pause
**Decision**: Continuous simulation with pause/speed controls.

**Rationale**:
- Natural creature behaviors
- Flexible observation options
- Accommodates different playstyles
- Good for streaming/content creation

**Trade-offs**:
- Can't have turn-based mechanics
- Requires robust time scaling
- Performance scaling challenges

### Procedural World Generation
**Decision**: Worlds are procedurally generated from seeds.

**Rationale**:
- Infinite unique worlds
- Shareable via seeds
- Reduced asset requirements
- Exploration value

**Trade-offs**:
- No hand-crafted scenarios
- Potential for unbalanced worlds
- Generation time cost

## Simulation Decisions

### Realistic Genetics
**Decision**: Model actual genetic inheritance patterns.

**Rationale**:
- Educational value
- Believable evolution
- Emergent creature varieties
- Scientific accuracy

**Trade-offs**:
- Complex implementation
- May not be "fun" enough
- Requires careful visualization

### Need-based Behaviors
**Decision**: Creature actions driven by physiological/social needs.

**Rationale**:
- Predictable yet varied behavior
- Natural priority system
- Relatable to players
- Realistic motivation

**Trade-offs**:
- May seem repetitive
- Limits "interesting" behaviors
- Requires many systems

### Emotional States
**Decision**: Creatures have emotions that affect decisions.

**Rationale**:
- Increases empathy
- Adds behavioral variety
- Enables social bonding
- More engaging to watch

**Trade-offs**:
- Anthropomorphization risk
- Complex to balance
- Performance overhead

## Art Direction Decisions

### Cartoonish Creatures
**Decision**: Stylized rather than realistic creature designs.

**Rationale**:
- Broader appeal
- Clearer emotional expression
- Easier animation
- Avoids uncanny valley

**Trade-offs**:
- Less "serious" appearance
- May limit audience
- Harder to show subtle details

### Vibrant Colors
**Decision**: Bright, saturated color palette.

**Rationale**:
- Visually appealing
- Clear creature identification
- Happy atmosphere
- Good for streaming

**Trade-offs**:
- Less realistic
- May clash with serious themes
- Harder to show mood lighting

## Future Considerations

### Moddability
**Status**: Designed for but not implemented

**Reasoning**: Architecture supports mods but no tools yet

### Multiplayer
**Status**: Not planned for initial release

**Reasoning**: Adds significant complexity, focus on single-player first

### Mobile Version
**Status**: Possible future port

**Reasoning**: UI would need major redesign, performance concerns

### VR Support
**Status**: Not planned

**Reasoning**: Doesn't fit observation-based gameplay

## Design Principles

1. **Simplicity in rules, complexity in outcomes**
2. **Every action has visible consequences**
3. **Performance enables features**
4. **Creatures should feel alive**
5. **Player discovery over tutorials**
6. **Scientific accuracy where possible**
7. **Accessibility without compromising depth**