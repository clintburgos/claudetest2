# Sprite Assets

## Current Status
This directory contains placeholder sprite files for the cartoon isometric rendering system.

## Required Assets for Phase 1 (According to CARTOON_ISOMETRIC_IMPLEMENTATION_PLAN.md)

### Critical Path Assets (Must Have):

#### Creature Sprites (48x48 base size)
- `creature.png` - Placeholder creature sprite (currently using colored squares)
- Required animations per creature type:
  - `idle_*.png` (8 frames) - Standing animation cycle
  - `walk_*.png` (8 frames) - Walking animation cycle
  - `eat_*.png` (6 frames) - Eating animation
  - `sleep_*.png` (4 frames) - Sleeping animation
  - `talk_*.png` (6 frames) - Conversation animation

#### Expression Overlays (16x16)
- `expressions/happy.png` - Happy face overlay
- `expressions/sad.png` - Sad face overlay
- `expressions/angry.png` - Angry face overlay
- `expressions/scared.png` - Scared face overlay
- `expressions/curious.png` - Curious face overlay

#### Terrain Tiles (64x32 isometric)
- `terrain/grass/tile_*.png` - Grass tile variations
- `terrain/desert/tile_*.png` - Desert tile variations
- `terrain/forest/tile_*.png` - Forest tile variations
- `terrain/tundra/tile_*.png` - Tundra tile variations
- `terrain/water/tile_*.png` - Water tile variations

#### Resource Sprites (32x32)
- `food.png` - Generic food resource (currently using colored square)
- `water.png` - Water resource sprite (currently using colored square)
- Biome-specific resources:
  - `resources/berries_*.png` (3 growth stages) - Forest biome
  - `resources/cacti_water.png` - Desert biome
  - `resources/nuts_*.png` (3 growth stages) - Forest biome
  - `resources/ice_fish.png` - Tundra biome
  - `resources/seeds.png` - Grassland biome

#### Particle Effects (8x8 to 16x16)
- `effects/particles/heart.png` - Love/bonding particle
- `effects/particles/zzz.png` - Sleep particle
- `effects/particles/sparkle.png` - Generic sparkle effect
- `effects/particles/sweat.png` - Stress/heat particle
- `effects/particles/question.png` - Confusion particle
- `effects/particles/exclamation.png` - Alert particle

#### UI Elements
- `ui/speech_bubble.png` - Speech bubble background (9-slice)
- `ui/health_bar.png` - Health bar sprite
- `ui/need_icons/*.png` - Icons for hunger, thirst, social needs

## Asset Specifications

### Art Style Requirements:
- 2px black outlines for all sprites
- Soft, rounded shapes for creatures
- Large expressive eyes (40% of head size)
- Bright, saturated colors
- Consistent perspective (isometric 2:1 ratio, 30° angle)

### Technical Requirements:
- PNG format with transparency
- Power-of-2 dimensions where possible
- Sprite sheets should be organized in grids
- Maximum texture atlas size: 4096x4096
- Consistent anchor points for animations

## Current Implementation
The system is currently using colored rectangles as placeholders:
- Green squares for herbivores
- Red squares for carnivores
- Blue squares for water
- Brown squares for food
- Various colored squares for terrain

These placeholders are rendered through the standard Bevy sprite system while the cartoon rendering infrastructure is being developed.

## Migration Plan
Once proper sprite assets are created:
1. Replace placeholder colors with actual sprite loading
2. Enable sprite atlas generation
3. Activate animation systems
4. Enable particle effects
5. Switch from colored squares to sprite-based rendering