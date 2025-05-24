# Social & Conversation System

## Overview
The social system enables creatures to communicate, form relationships, and influence each other's behavior through meaningful conversations that impact survival and create emergent social dynamics.

**For comprehensive conversation system details, see:**
- [CONVERSATION_SYSTEM.md](./CONVERSATION_SYSTEM.md) - Complete architecture and design
- [CONVERSATION_IMPLEMENTATION.md](./CONVERSATION_IMPLEMENTATION.md) - Implementation guide
- [CONVERSATION_QUICK_REF.md](./CONVERSATION_QUICK_REF.md) - Quick reference

## High-Level Design

### Communication Model
Creatures exchange information through topic-based conversations rather than raw text:
- **Survival topics**: Food, water, danger, shelter locations
- **Social topics**: Relationships, group formation, conflict resolution
- **Knowledge sharing**: Environmental info, skills, gossip
- **Emotional expression**: Joy, grief, complaints, encouragement

### Key Features
- **Turn-based dialogue** with greeting, exchange, and closing phases
- **Information degradation** as knowledge spreads
- **Relationship evolution** through interactions
- **Decision influence** from trusted sources
- **Cultural knowledge** that spreads through groups
- **Gossip networks** that modify information

### Performance Optimization
- LOD system scales detail with distance
- Conversation pooling for efficiency
- Statistical outcomes for distant creatures
- Max 50 simultaneous conversations

## Core Systems

### Relationship Types

### Primary Relationships
1. **Parent-Child**: Strong influence, protection
2. **Siblings**: Moderate influence, cooperation
3. **Mates**: Strong influence, coordination
4. **Friends**: Positive influence, assistance
5. **Rivals**: Negative influence, competition
6. **Strangers**: Neutral, cautious

### Group Dynamics
- Groups form around high-sociability individuals
- Shared knowledge within groups
- Group identity affects decisions
- Cultural traits emerge from group behavior

## Conversation Flow

### Initiation
1. Proximity check (within communication range)
2. Attention check (both available)
3. Relationship check (willingness)
4. Topic selection (based on needs/knowledge)

### Exchange Process
1. Sender formulates message
2. Apply honesty modifier
3. Receiver interprets (with noise)
4. Receiver updates internal state
5. Possible response generated

### Memory System
- Recent conversations stored
- Important facts retained longer
- Social reputation tracked
- Knowledge can spread through network

## Emergent Behaviors

### Expected Patterns
- Information cascades
- Group decision making
- Cultural knowledge pools
- Reputation systems
- Social hierarchies
- Deception strategies

### Visualization
- Speech bubbles with concept icons
- Relationship network overlay
- Influence flow visualization
- Group boundary display
- Trust level indicators

---
*Last Updated: 2024-01-XX*
