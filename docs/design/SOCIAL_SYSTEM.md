# Social & Conversation System

## Overview
The social system enables creatures to communicate, form relationships, and influence each other's behavior through symbolic conversations.

## Conversation Model

### Concept-Based Communication
Instead of natural language, creatures exchange symbolic concepts:

```rust
enum ConceptType {
    Need(NeedType, Intensity),      // "I am hungry"
    Location(ResourceType, Direction), // "Food is north"
    Warning(ThreatType, Position),    // "Danger there"
    Social(RelationType, CreatureId), // "Be my friend"
    Knowledge(Fact, Certainty),       // "This is safe"
    Emotion(EmotionType, Intensity),  // "I am happy"
}

struct Message {
    sender: CreatureId,
    receiver: CreatureId,
    concept: ConceptType,
    honesty: f32,  // 0.0 = lie, 1.0 = truth
    urgency: f32,  // 0.0 = casual, 1.0 = urgent
}
```

### Communication Traits
From genetics:
- **Eloquence**: Message clarity/effectiveness
- **Persuasiveness**: Influence strength
- **Honesty**: Tendency to tell truth
- **Gullibility**: Susceptibility to influence
- **Empathy**: Understanding others' needs

## Influence Mechanics

### Decision Weight Modification
When receiving a message, creature adjusts decision weights:

```
influence = message.urgency * 
           sender.persuasiveness * 
           receiver.gullibility * 
           relationship_modifier

new_weight = old_weight + (influence * concept_weight_delta)
```

### Trust System
- Trust built through successful interactions
- Trust damaged by detected lies
- Trust affects influence acceptance
- Trust inherited partially from parents

## Relationship Types

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
