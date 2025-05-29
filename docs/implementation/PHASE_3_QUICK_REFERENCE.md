# Phase 3 Quick Reference Guide

## Overview
Phase 3 implements the Creature Visual Systems, transforming basic sprites into expressive, animated characters with genetic variations and dynamic attachments.

## Key Components

### 1. Sprite Atlas Structure
- **Size**: 2048x1024 per species
- **Layout**: 8 rows (variations + expressions + effects)
- **Frame Size**: 48x48 for animations, 96x96 for expressions
- **UV Mapping**: Use `AtlasUVMapping` helper class

### 2. Animation System
```rust
// Core animation types
AnimationType::{Idle, Walk, Run, Eat, Sleep, Talk, Attack, Death}

// Transition priorities (higher interrupts lower)
Death: 11, Attack: 10, Eat: 5, Walk: 2, Idle: 1

// Blend duration defaults
Fast transitions: 0.1-0.2s (combat)
Normal transitions: 0.2-0.3s (movement)
Slow transitions: 0.5-1.0s (sleep)
```

### 3. Expression System
```rust
// Emotion priorities (higher overrides lower)
Angry: 0.9, Frightened: 0.85, Sad: 0.7, Happy: 0.4, Neutral: 0.1

// Expression components
- Eye state (openness, pupil position, shape)
- Mouth shape (bezier curves)
- Eyebrow state (height, angle, curve)
- Effects (hearts, anger veins, tears)
```

### 4. Genetic Variations
```rust
// Visual traits from genetics
Size: 0.7x to 1.3x scale
Colors: 8 base tints + patterns
Patterns: None, Spots, Stripes, Patches, Gradient
Pattern intensity: 0.0 to 1.0
```

### 5. Attachment Points
```rust
// Standard attachment locations
head: (0, 20) - hats, accessories
left_hand: (-12, 5) - tools, items
right_hand: (12, 5) - weapons, tools
back: (0, 10) - backpacks, wings
waist: (0, 0) - belts, pouches
tail_tip: (-15, -5) - decorations
```

### 6. Animation Layers
```rust
// Layer system
Base: Full body locomotion (walk/run)
Overlay: Upper body actions (eat/talk)
Additive: Procedural effects (shiver/bounce)

// Masking options
Full body, Upper body only, Individual limbs
```

## Implementation Checklist

### Component Setup
- [ ] Add `CartoonCreature` component with genetic modifiers
- [ ] Add `ExpressionController` for emotion management  
- [ ] Add `GeneticPattern` for visual variations
- [ ] Add `CreatureAttachmentPoints` for items
- [ ] Add `AnimationLayers` for blending

### Systems Required
- [ ] `update_animation_state` - State machine transitions
- [ ] `sync_expression_to_emotion` - AI to visual mapping
- [ ] `render_genetic_patterns` - Pattern shader application
- [ ] `update_attachment_transforms` - Item positioning
- [ ] `blend_animation_layers` - Multi-layer blending

### Assets Needed
- [ ] Creature atlas textures (2048x1024)
- [ ] Expression overlays (96x96 per emotion)
- [ ] Pattern masks for genetic variations
- [ ] Tool/accessory sprites
- [ ] Particle textures for effects

## Performance Considerations

### LOD Levels
```rust
Full (0-50 units): All features enabled
High (50-100): No expression blending
Medium (100-200): Reduced animations, basic emotions
Low (200-400): Key frames only, no particles
Minimal (400+): Static sprites
```

### Optimization Tips
1. Use sprite batching for same-species creatures
2. Cache expression overlays
3. Limit particle emitters per creature
4. Reduce animation update frequency at distance
5. Disable pattern shaders in Low quality

## Common Integration Points

### With AI System
```rust
// Map AI states to animations
match creature_state {
    Hungry => request_expression(EmotionType::Hungry),
    Attacking => set_animation(AnimationType::Attack),
    Fleeing => set_animation(AnimationType::Run),
}
```

### With Physics
```rust
// Velocity-based animation selection
let speed = velocity.length();
let animation = match speed {
    0.0..=0.1 => AnimationType::Idle,
    0.1..=5.0 => AnimationType::Walk,
    _ => AnimationType::Run,
};
```

### With Inventory
```rust
// Attach held items
commands.entity(creature).with_children(|parent| {
    parent.spawn((
        AttachedItem {
            attachment_point: "right_hand".to_string(),
            item_type: ItemType::Tool(tool),
            custom_offset: Vec2::ZERO,
            custom_rotation: 0.0,
            inherit_animation: true,
        },
        // ... sprite components
    ));
});
```

## Debugging

### Visual Debug Options
- F10: Show attachment points
- F11: Display emotion states
- F12: Highlight animation layers
- Shift+F10: Show UV boundaries
- Shift+F11: Display blend weights

### Common Issues
1. **Wrong animation playing**: Check state machine priorities
2. **Expressions not showing**: Verify emotion controller updates
3. **Patterns not rendering**: Check shader compilation
4. **Attachments misaligned**: Verify frame offsets
5. **Performance drops**: Reduce LOD thresholds

## Key Files
- Animation state machine: `src/systems/animation.rs`
- Expression controller: `src/systems/expression.rs`
- Pattern rendering: `src/rendering/patterns.rs`
- Attachment system: `src/systems/attachments.rs`
- Atlas loading: `src/plugins/sprite_loading.rs`

## Testing
```rust
// Spawn test creature with all features
commands.run_system(spawn_test_creature_with_visuals);

// Test animation transitions
#[test]
fn test_walk_to_run_transition() {
    // Verify smooth blending
}

// Test expression priorities
#[test]
fn test_emotion_override_priority() {
    // Verify angry overrides happy
}
```

## Next Steps
After Phase 3 implementation:
1. Phase 4: Particle effects and weather
2. Phase 5: Biome-specific features
3. Phase 6: Performance optimization

Remember: Focus on getting basic animations working first, then add expressions, then patterns, and finally attachments.