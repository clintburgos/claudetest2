# Conversation System Quick Reference

## Core Concepts
```
Proximity → Topic Selection → Information Exchange → Outcomes
```

## Topic Categories

### Survival (High Priority)
- **FoodLocation**: Share food sources
- **WaterLocation**: Share water sources
- **DangerWarning**: Alert about threats
- **ShelterLocation**: Safe places

### Social (Medium Priority)
- **Introduction**: First meetings
- **RelationshipBuilding**: Deepen bonds
- **GroupFormation**: Create alliances
- **ConflictResolution**: Solve disputes

### Knowledge (Low Priority)
- **EnvironmentObservation**: Share discoveries
- **CreatureGossip**: Social information
- **TeachingSkill**: Transfer techniques
- **PhilosophicalMusing**: Abstract ideas

### Emotional (Variable Priority)
- **Celebration**: Share joy
- **Mourning**: Grieve together
- **Complaint**: Express frustration
- **Encouragement**: Boost morale

## Key Components

### ConversationCapability
```rust
eloquence: f32,      // Message clarity
empathy: f32,        // Emotional understanding
knowledge_retention: f32,  // Memory quality
style: CommunicationStyle
```

### Information Types
- **ResourceLocation**: Where to find things
- **AreaSafety**: Threat levels
- **CreatureInformation**: Social knowledge
- **Technique**: Learned behaviors
- **PhilosophicalIdea**: Abstract concepts

## Relationship Effects

### Positive Outcomes
- Friendship: +0.1 to +0.3
- Trust: +0.1 (if info accurate)
- Respect: +0.1 (from teaching)
- Romance: +0.2 (successful courtship)

### Negative Outcomes
- Trust: -0.2 (false information)
- Friendship: -0.1 (failed interactions)
- All metrics: -0.5 (major conflicts)

## Performance Guidelines

### Update Frequencies
- **LOD 0**: 10 Hz (full dialogue)
- **LOD 1**: 2 Hz (simplified)
- **LOD 2**: 1 Hz (info only)
- **LOD 3+**: 0.5 Hz (statistical)

### Optimization Rules
- Max 50 simultaneous conversations
- Pool all message generation
- Cache relationships for 1s
- Batch process by LOD level

## Implementation Checklist

### Setup
- [ ] Add ConversationCapability
- [ ] Initialize ConversationMemory
- [ ] Create SocialContext
- [ ] Register conversation events

### Processing
- [ ] Proximity detection
- [ ] Topic selection logic
- [ ] Turn-based exchange
- [ ] Outcome application

### Integration
- [ ] Link to decision system
- [ ] Update relationships
- [ ] Propagate knowledge
- [ ] Trigger animations

## Common Patterns

### Emergency Broadcasting
```rust
if danger_detected {
    broadcast_to_radius(
        DangerWarning,
        SHOUT_RADIUS,
        urgency: 1.0
    );
}
```

### Teaching Chains
```rust
if has_technique && nearby.lacks_technique {
    initiate_conversation(
        TeachingSkill(technique),
        engagement_required: 0.8
    );
}
```

### Gossip Networks
```rust
if has_social_info && friend_nearby {
    share_gossip(
        reliability * friendship_level
    );
}
```

## Debug Commands
```rust
// Visualize conversations
draw_conversation_bubbles()

// Show relationship networks
render_social_graph()

// Track information spread
trace_knowledge_propagation()
```

---
*Full docs: [CONVERSATION_SYSTEM.md](./CONVERSATION_SYSTEM.md)*
*Implementation: [CONVERSATION_IMPLEMENTATION.md](./CONVERSATION_IMPLEMENTATION.md)*