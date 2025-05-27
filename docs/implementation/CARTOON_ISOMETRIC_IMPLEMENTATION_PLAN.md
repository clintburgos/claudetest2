# Cartoon Isometric UI Implementation Plan

## Overview
This document outlines the transformation of the creature simulation from its current functional rendering to a vibrant cartoon-style isometric world with expressive creatures and biome-based environments.

## Phase 1: Asset Creation & Pipeline Setup (Week 1-2)

### 1.1 Sprite Asset Requirements
```
assets/sprites/
├── creatures/
│   ├── herbivore/
│   │   ├── idle_*.png (8 frames)
│   │   ├── walk_*.png (8 frames)
│   │   ├── eat_*.png (6 frames)
│   │   ├── sleep_*.png (4 frames)
│   │   ├── talk_*.png (6 frames)
│   │   └── expressions/
│   │       ├── happy.png
│   │       ├── sad.png
│   │       ├── angry.png
│   │       ├── scared.png
│   │       └── curious.png
│   ├── carnivore/
│   └── omnivore/
├── terrain/
│   ├── grass/
│   │   ├── tile_*.png (variations)
│   │   └── transitions/
│   ├── desert/
│   ├── forest/
│   ├── tundra/
│   └── water/
├── resources/
│   ├── berries_*.png (growth stages)
│   ├── nuts_*.png
│   ├── water_puddle.png
│   └── shelter_*.png
├── effects/
│   ├── particles/
│   │   ├── heart.png
│   │   ├── zzz.png
│   │   ├── sparkle.png
│   │   └── sweat.png
│   └── weather/
│       ├── rain.png
│       ├── snow.png
│       └── fog.png
└── ui/
    ├── speech_bubble.png
    ├── health_bar.png
    └── need_icons/
```

### 1.2 Art Style Guidelines
- **Creatures**: Round, soft shapes with large expressive eyes (40% of head)
- **Color Palette**: Bright, saturated colors with subtle gradients
- **Outlines**: 2px black outlines for all sprites
- **Size**: 64x64 base for creatures, 32x32 for tiles
- **Perspective**: Isometric 2:1 ratio (30° angle)

### 1.3 Asset Loading System Enhancement
```rust
// src/plugins/asset_manager.rs
pub struct AssetManager {
    creature_animations: HashMap<(Species, CreatureState), Handle<AnimationClip>>,
    terrain_tiles: HashMap<BiomeType, Vec<Handle<Image>>>,
    particle_textures: HashMap<ParticleType, Handle<Image>>,
    expression_overlays: HashMap<EmotionType, Handle<Image>>,
}
```

## Phase 2: Isometric World Rendering (Week 2-3)

### 2.1 Enhanced Isometric Camera System
```rust
// src/plugins/camera.rs enhancements
- Smooth zoom with constraints (0.5x - 3.0x)
- Edge panning with acceleration
- Click-to-focus on creatures
- Mini-map integration
```

### 2.2 Terrain Rendering System
```rust
// src/systems/terrain_renderer.rs
- Chunk-based tile rendering
- Biome transition blending
- Height variation with elevation tiles
- Decorative element placement (rocks, plants)
- Water animation system
```

### 2.3 Biome Generation Enhancement
```rust
// src/systems/world_generation.rs
- Multi-layer noise for realistic biomes
- Temperature/moisture based biome selection
- Resource spawn tables per biome
- Landmark generation (caves, oases)
```

### 2.4 Depth Sorting Improvements
- Multi-layer sorting (ground, shadows, creatures, effects, UI)
- Transparency for occluded creatures
- Shadow rendering for depth perception

## Phase 3: Creature Visual Systems (Week 3-4)

### 3.1 Animated Sprite Component Enhancement
```rust
#[derive(Component)]
pub struct CartoonCreature {
    base_animation: AnimationState,
    expression_overlay: Option<Expression>,
    body_modifiers: BodyModifiers, // size, color tint based on genetics
    accessory_slots: Vec<Accessory>, // for tools, decorations
}

#[derive(Component)]
pub struct Expression {
    emotion: EmotionType,
    intensity: f32,
    eye_direction: Vec2,
    mouth_state: MouthState,
}
```

### 3.2 Genetic Trait Visualization
- **Size**: 0.7x - 1.3x scale based on size genes
- **Color**: Hue shifts based on genetic markers
- **Features**: Ear size, tail length, body proportions
- **Patterns**: Spots, stripes based on genetics

### 3.3 Emotion & State Visualization
```rust
// Visual feedback for states
- Hunger: Stomach growl animation, droopy posture
- Thirst: Panting animation, dry mouth visual
- Tired: Slow blink, yawning, droopy ears
- Social: Heart particles when bonding
- Mating: Blush effect, heart eyes
- Sick: Green tint, dizzy stars
- Fighting: Angry eyebrows, smoke from ears
```

### 3.4 Action Animations
- **Eating**: Chomping with food particles
- **Drinking**: Lapping with splash effects
- **Sleeping**: Z particles, peaceful expression
- **Walking**: Bouncy movement with dust clouds
- **Running**: Speed lines, urgent expression
- **Talking**: Speech bubble with emotion icons
- **Tool Use**: Holding and using animations

## Phase 4: Effects & Polish (Week 4-5)

### 4.1 Particle System Implementation
```rust
// src/systems/particle_system.rs
pub enum ParticleEffect {
    Love { emitter: Entity, target: Option<Entity> },
    Sleep { position: Vec3, duration: f32 },
    Success { position: Vec3, burst_count: u32 },
    Weather { type: WeatherType, density: f32 },
    Action { type: ActionType, position: Vec3 },
}
```

### 4.2 Weather & Environmental Effects
- Rain with puddle formation
- Snow accumulation on terrain
- Wind affecting particle directions
- Fog reducing visibility
- Day/night lighting changes

### 4.3 UI Enhancement
```rust
// Cartoon-style UI elements
- Speech bubbles with tail pointing to speaker
- Floating health/need bars above creatures
- Comic-style action indicators ("!", "?", "...")
- Smooth camera transitions to events
- Picture-in-picture for important events
```

### 4.4 Audio Integration
- Creature vocalizations matching emotions
- Ambient biome sounds
- Weather sound effects
- Action feedback sounds
- Musical stings for events

## Phase 5: Biome-Specific Features (Week 5-6)

### 5.1 Biome Characteristics
```rust
pub struct BiomeConfig {
    // Visual
    base_tiles: Vec<TileType>,
    decoration_density: f32,
    color_palette: ColorPalette,
    ambient_particles: Vec<ParticleType>,
    
    // Resources
    exclusive_resources: Vec<ResourceType>,
    resource_abundance: HashMap<ResourceType, f32>,
    resource_quality_modifier: f32,
    
    // Environmental
    temperature_range: (f32, f32),
    weather_probabilities: HashMap<WeatherType, f32>,
    movement_speed_modifier: f32,
}
```

### 5.2 Biome-Specific Resources
- **Forest**: Berries, Nuts, Mushrooms, Wood
- **Desert**: Cacti Water, Desert Fruit, Stones
- **Tundra**: Ice Fish, Snow Berries, Fur
- **Grassland**: Seeds, Grass, Flowers
- **Ocean/Coast**: Shellfish, Seaweed, Salt

### 5.3 Biome Transitions
- Gradual tile blending at borders
- Mixed resource spawning in transition zones
- Environmental effect blending

## Phase 6: Performance Optimization (Week 6)

### 6.1 Sprite Batching
- Texture atlas for all creature sprites
- Instanced rendering for terrain tiles
- Particle pooling system

### 6.2 LOD System for Visuals
```rust
pub enum VisualLOD {
    Full,     // All animations, particles, details
    Reduced,  // Basic animations, some particles
    Minimal,  // Static sprites, no particles
}
```

### 6.3 Culling & Optimization
- Frustum culling for off-screen elements
- Animation update throttling by distance
- Particle count limits
- Terrain chunk loading/unloading

## Implementation Priority Order

1. **Critical Path** (Must Have):
   - Basic isometric terrain rendering
   - Creature sprite loading and display
   - Expression system for emotions
   - Biome generation with visual distinction
   - Resource sprites per biome

2. **High Priority** (Should Have):
   - Smooth animations for all actions
   - Particle effects for states
   - Weather system
   - Genetic trait visualization
   - Speech bubbles for conversations

3. **Nice to Have** (Could Have):
   - Advanced water effects
   - Seasonal changes
   - Tool/accessory system
   - Complex particle interactions
   - Picture-in-picture events

## Technical Considerations

### Bevy Integration
- Use bevy_ecs_tilemap for terrain
- bevy_particle_system for effects
- Custom animation system for sprites
- Enhanced sprite sorting for isometric

### Asset Pipeline
- Automated sprite sheet generation
- Hot reloading for development
- Compression for production builds

### Memory Management
- Texture atlas limits (4096x4096)
- Sprite pooling for particles
- Dynamic LOD switching

## Success Metrics
- 60 FPS with 500 creatures on screen
- < 100ms scene transition time
- < 500MB memory usage
- Smooth animations without stuttering
- Clear visual communication of all states

## Risks & Mitigation
- **Asset Creation Time**: Use placeholder art initially
- **Performance Impact**: Implement LOD early
- **Complexity**: Incremental implementation
- **Art Consistency**: Create style guide first