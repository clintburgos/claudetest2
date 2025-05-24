# Creature Design Specification

## Overview
This document details the design and implementation of creatures in the simulation.

## Creature Components

### Core Components
1. **Identity**
   - Unique ID
   - Name (generated)
   - Age
   - Generation number
   - Parent IDs

2. **Physical**
   - Position (x, y)
   - Size (affects food needs, speed)
   - Health (0-100)
   - Energy (0-100)

3. **Genetics**
   - DNA structure (see Genetics System)
   - Expressed traits
   - Mutation history

4. **Needs**
   - Hunger (0-100)
   - Thirst (0-100)
   - Social (0-100)
   - Reproduction (0-100)
   - Safety (0-100)
   - Rest (0-100)

5. **Mental**
   - Current goal
   - Decision weights  
   - Mood state
   - Stress level
   - **See [DECISION_MAKING_SYSTEM.md](./DECISION_MAKING_SYSTEM.md) for detailed AI architecture**

6. **Social**
   - Relationships map
   - Communication history
   - Group membership
   - Reputation scores

## Creature Lifecycle

### Birth
1. Inherit DNA from parents
2. Apply mutations
3. Express genes → traits
4. Initialize needs at safe levels
5. Place near parents

### Life Stages
- **Infant** (0-10% lifespan): Dependent, learning
- **Youth** (10-25%): Growing, exploring
- **Adult** (25-80%): Full capabilities
- **Elder** (80-100%): Declining abilities, wisdom

### Death Conditions
- Starvation (hunger = 0)
- Dehydration (thirst = 0)  
- Health depletion
- Old age
- Predation (future feature)

## Trait System

### Physical Traits (from genes)
- **Size**: 0.5-2.0x base
- **Speed**: 0.5-2.0x base
- **Metabolism**: Energy consumption rate
- **Lifespan**: 50-150% base
- **Fertility**: Reproduction chance
- **Strength**: Resource gathering efficiency

### Mental Traits (from genes)
- **Intelligence**: Decision quality
- **Memory**: Social history capacity
- **Sociability**: Group formation tendency
- **Aggression**: Competition behavior
- **Curiosity**: Exploration tendency
- **Caution**: Risk avoidance

### Derived Attributes
- Max health = base * size * 0.8
- Max energy = base * (2 - metabolism)
- Movement cost = base * size * metabolism
- Communication range = base * sociability

## Visual Representation

### UI Design
For detailed information about creature visual design, expression systems, and UI implementation, see [UI_DESIGN.md](./UI_DESIGN.md).

Key visual features include:
- **Isometric View**: 30°/45° perspective with proper depth sorting
- **Expressive Animations**: Cartoonish, over-exaggerated emotions
- **Visibility Solutions**: Multiple systems ensure creatures remain visible when occluded
- **Particle Effects**: Emotion indicators like hearts, steam, thought bubbles
- **Dynamic Colors**: Mood-based color shifts and glowing effects

### Visual States
Creatures display different visual states based on their internal state:
- **Emotions**: Happy, sad, angry, confused, tired, excited
- **Needs**: Visual indicators for hunger, thirst, social needs
- **Health**: Color saturation and posture changes
- **Age**: Size and movement speed variations

---
*Last Updated: 2024-01-XX*
