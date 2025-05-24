# Decision Making Quick Reference

## Core Architecture
```
Inputs → Utility Scoring → Goal Selection → Action Planning → Execution
```

## Goals (by priority)
1. **FleeFromDanger** (100) - Immediate survival
2. **SatisfyThirst** (90) - Critical need
3. **SatisfyHunger** (80) - Important need
4. **SeekShelter** (70) - Safety
5. **Rest** (60) - Recovery
6. **Reproduce** (50) - Species survival
7. **Socialize** (40) - Social needs
8. **Explore** (30) - Curiosity

## Key Components

### Decision State
```rust
DecisionState {
    current_goal: Goal,
    current_action: Action,
    decision_cooldown: f32,
}
```

### Utility Calculation
- Base utility from need level
- Environmental modifiers
- Genetic trait modifiers
- Social influences
- Hysteresis factor: 1.2x

### Action Types
- **Movement**: MoveTo, Wander, FleeFrom
- **Resources**: Eat, Drink, Gather
- **Social**: Approach, Converse, Share
- **Biological**: Rest, Reproduce

## Performance Rules

### Update Frequencies by LOD
- **LOD 0**: 10 Hz (nearby)
- **LOD 1**: 2 Hz (medium)
- **LOD 2**: 1 Hz (far)
- **LOD 3+**: 0.5 Hz (very far)

### Optimization Checklist
- [ ] Cache utilities for 0.5-1s
- [ ] Reuse spatial queries
- [ ] Skip distant creatures
- [ ] Batch similar decisions
- [ ] Use events for side effects

## Common Patterns

### Adding a Goal
1. Define goal enum variant
2. Create UtilityEvaluator
3. Implement evaluate() method
4. Register with GoalRegistry

### Complex Actions
```rust
// Multi-step plan
steps.push_back(ActionStep {
    action: Action::MoveTo { target, speed },
    completion: ReachedPosition { tolerance: 1.0 },
    timeout: 10.0,
});
```

### Interrupt Handling
- Compare goal priorities
- Save current plan
- Execute interrupt
- Resume saved plan

## Debug Commands
```rust
// Visualize decisions
app.add_systems(Update, debug_draw_decisions);

// Track history
app.add_plugins(DecisionHistoryPlugin);

// Log changes
.add_systems(Update, log_decision_changes);
```

---
*Full docs: [DECISION_MAKING_SYSTEM.md](./DECISION_MAKING_SYSTEM.md)*
*Implementation: [DECISION_IMPLEMENTATION_GUIDE.md](./DECISION_IMPLEMENTATION_GUIDE.md)*