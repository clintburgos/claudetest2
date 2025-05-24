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

---
*Last Updated: 2024-01-XX*
