# UI Design Specification

## Overview
This document details the visual design and user interface decisions for the creature simulation, focusing on an isometric view with expressive, cartoonish creatures. For control schemes and interface layout, see [CONTROLS_INTERFACE.md](./CONTROLS_INTERFACE.md).

## View System

### Isometric Perspective
- **Angle**: 30°/45° isometric projection
- **Grid**: Visible isometric grid for spatial reference
- **Camera**: Fixed angle with pan and zoom capabilities
- **Depth Sorting**: Proper layering based on y-position and z-height

### Zoom Levels
The camera supports 5 distinct zoom levels:
- **Macro** (1:100): World overview, population heatmaps
- **Regional** (1:50): Biome overview, migration patterns  
- **Local** (1:20): Multiple creatures, resource distribution
- **Close** (1:5): Individual creatures with names/stats
- **Intimate** (1:1): Single creature detail, full emotions

## Creature Visual Design

### Art Style
- **Cartoonishly Over-Exaggerated**: High expressiveness for emotional readability
- **Shape Language**: Round, soft shapes for friendly appearance
- **Size**: Creatures occupy roughly 60x80 pixel area in default zoom

### Expression System

#### Facial Features
- **Eyes**:
  - Disproportionately large (window to emotions)
  - Dynamic pupil movement and dilation
  - Shape morphing:
    - Happy: Curved crescents
    - Sad: Droopy with visible pupils looking down
    - Angry: Angled/triangular with furrowed brows
    - Surprised: Wide circles with tiny pupils
    - Dizzy: Spiral or star shapes
    - Love: Heart-shaped
  - Eyebrows that can float above head for surprise

- **Mouth**:
  - Highly flexible shape morphing
  - Happy: Wide upward curve
  - Sad: Small downward curve
  - Angry: Tight frown or showing teeth
  - Surprised: Perfect 'O' shape
  - Can stretch beyond body bounds for emphasis

#### Body Language
- **Squash and Stretch**: 
  - Compress on landing (to 80% height)
  - Elongate when jumping (to 120% height)
  - Stretch horizontally when moving fast
  
- **Idle Animations**:
  - Constant breathing (subtle scale pulse)
  - Blinking cycle
  - Slight bounce/sway
  - Tail or appendage movement

- **Emotion-Specific Animations**:
  - Happy: Bouncing in place, higher idle position
  - Sad: Slumped posture, slower breathing
  - Angry: Rapid vibration/shaking
  - Excited: Quick random movements
  - Tired: Occasional yawn, droopy posture

#### Emotional Indicators
- **Particle Effects**:
  - Hearts: Love, happiness, affection
  - Steam clouds: Anger, frustration
  - Sweat drops: Nervousness, exertion
  - Stars: Dizziness, confusion
  - Z's: Sleepiness
  - Sparkles: Excitement, discovery
  - Rain cloud: Deep sadness
  - Lightning bolt: Sudden realization
  - Musical notes: Contentment, singing

- **Thought Bubbles**:
  - Float above creature
  - Show icons for current needs/thoughts:
    - Food icons when hungry
    - Water drop when thirsty
    - Heart when seeking mate
    - House when seeking shelter
    - Question mark when confused
    - Lightbulb for ideas

- **Color Shifts**:
  - Base color can shift slightly with mood
  - Redder tint when angry
  - Bluer tint when sad
  - Brighter saturation when happy
  - Desaturated when sick/tired

## Visibility Solutions

### Occlusion Handling
When creatures go behind objects, multiple systems ensure visibility:

#### 1. X-Ray Outline System
- **Glowing Silhouette**: Colored outline visible through obstacles
- **Color Coding**: Outline color matches creature's team/mood
- **Intensity**: Pulsing glow to draw attention
- **Arrow Indicator**: Downward arrow above creature position

#### 2. Dynamic Transparency
- **Proximity-Based**: Objects become transparent when creatures are near
- **Fade Range**: 30-70% opacity based on distance
- **Smooth Transitions**: Animated opacity changes
- **Height-Selective**: Only affects objects at creature's height level

#### 3. Floor Indicators
- **Persistent Shadow**: Always visible on ground plane
- **Position Ring**: Dashed circle showing creature location
- **Trail Effect**: Brief movement trail on floor
- **Team Color**: Colored glow on ground beneath creature

#### 4. Dithering Pattern
- **Checkerboard Transparency**: For less important obstacles
- **Maintains Structure**: Shows object shape while revealing creatures
- **Performance Friendly**: No real transparency needed

### Combined Approach
The system uses intelligent combination:
- Primary: Glowing outline + floor shadow
- Secondary: Thought bubbles/emotion particles above everything
- Contextual: Dynamic transparency for nearby objects
- Fallback: Dithering for performance-constrained situations

## Isometric-Specific Adaptations

### Billboard Elements
Certain elements break isometric rules for clarity:
- Facial features can face camera
- Thought bubbles always upright
- UI elements (health bars, names) in screen space

### Depth Cues
- Stronger shadows for grounded feel
- Rim lighting to separate from background
- Size variation with distance (subtle)
- Fog/atmospheric perspective for large scenes

### Animation Considerations
- Vertical movement emphasized (jumps, bounces)
- Horizontal movement follows isometric angles
- Turning animations show multiple angles
- Special moves can temporarily break perspective

## Technical Implementation

### Sprite System
- **Base Sprites**: 8 directions (N, NE, E, SE, S, SW, W, NW)
- **Expression Overlays**: Separate facial feature sprites
- **Particle System**: For emotion effects
- **Shadow Sprites**: Separate for proper layering

### Performance Optimization
- **LOD System**: Reduce animation complexity at distance
- **Emotion Culling**: Only show particles when zoomed in
- **Batch Rendering**: Group similar creatures
- **Texture Atlasing**: All creature sprites in atlases

### Accessibility
- **High Contrast Mode**: Stronger outlines
- **Colorblind Modes**: Shape-based emotion indicators
- **Motion Reduction**: Option to reduce idle animations
- **Emotion Text**: Optional text labels for emotions

## Visual Hierarchy

### Priority Order (highest to lowest)
1. Player-controlled creatures
2. Creatures in critical states (dying, mating)
3. Active/moving creatures  
4. Idle creatures
5. Sleeping/resting creatures
6. Environmental objects
7. Terrain/background

### Attention Direction
- Brighter colors for important creatures
- Motion attracts attention
- Particle effects for critical events
- Audio cues match visual importance

## Future Considerations

### Scalability
- System displays 50-200 creatures on screen (from 1000+ total in simulation)
- Graceful degradation of effects
- Optional quality settings
- Mobile-friendly simplified mode

### Extensibility  
- Easy to add new emotions
- Modular expression system
- Customizable creature appearances
- Themed visual packs possible

---
*Last Updated: 2024-12-XX*
*Reference: See `/docs/design/isometric-creatures-mockup.svg` for visual examples*
