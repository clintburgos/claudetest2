# Cartoon Graphics Asset Details Specification

This document provides complete asset specifications to fill gaps in the main implementation documentation.

## Particle System Assets

### Particle Texture Specifications

#### Base Particle Sprites
- **Dimensions**: 32x32 pixels (power of 2 for GPU optimization)
- **Format**: PNG with alpha channel
- **Color Mode**: RGBA 8-bit per channel
- **Compression**: PNG-8 with alpha for smaller textures

#### Particle Types and Sizes

| Particle Type | Texture Size | Atlas Position | Animation Frames |
|--------------|--------------|----------------|------------------|
| Emotion Bubbles | 32x32 | Row 0 | 8 frames |
| Food Crumbs | 16x16 | Row 1 | 4 frames |
| Water Droplets | 16x16 | Row 2 | 6 frames |
| Dust Clouds | 32x32 | Row 3 | 4 frames |
| Hearts (Love) | 24x24 | Row 4 | 2 frames |
| Anger Symbols | 24x24 | Row 5 | 3 frames |
| Sleep Z's | 16x16 | Row 6 | 1 frame |
| Sparkles | 16x16 | Row 7 | 4 frames |

#### Particle Atlas Layout
```
assets/sprites/particles/particle_atlas.png (256x256)
├── Row 0: Emotion bubbles (8x 32x32)
├── Row 1-2: Small particles (16x 16x16)
├── Row 3: Dust effects (4x 32x32)
├── Row 4-5: Status symbols (8x 24x24)
├── Row 6-7: Tiny particles (16x 16x16)
```

### Weather Particle Specifications

| Weather Type | Texture Size | Density | File Path |
|-------------|--------------|---------|-----------|
| Rain | 4x16 | 200/screen | assets/sprites/weather/rain.png |
| Snow | 8x8 | 150/screen | assets/sprites/weather/snow.png |
| Leaves | 16x16 | 50/screen | assets/sprites/weather/leaves.png |
| Sand | 4x4 | 300/screen | assets/sprites/weather/sand.png |

## Font Specifications

### UI Fonts

#### Primary Font (Headers)
- **Family**: "Cartoon Sans" (custom bitmap font)
- **File**: `assets/fonts/cartoon_sans.ttf`
- **Sizes**: 16, 20, 24, 32 pixels
- **Weight**: Bold (700)
- **Fallback**: System sans-serif

#### Secondary Font (Body Text)
- **Family**: "Pixel Comic" (bitmap font)
- **File**: `assets/fonts/pixel_comic.ttf`
- **Sizes**: 12, 14, 16 pixels
- **Weight**: Regular (400)
- **Fallback**: System monospace

#### Icon Font
- **Family**: "Creature Icons"
- **File**: `assets/fonts/creature_icons.ttf`
- **Size**: 16, 24 pixels
- **Glyphs**: Custom creature/resource symbols

### Font Loading Configuration
```rust
// In assets/fonts/fonts.ron
(
    fonts: [
        (
            name: "cartoon_sans",
            path: "fonts/cartoon_sans.ttf",
            sizes: [16, 20, 24, 32],
        ),
        (
            name: "pixel_comic",
            path: "fonts/pixel_comic.ttf",
            sizes: [12, 14, 16],
        ),
        (
            name: "creature_icons",
            path: "fonts/creature_icons.ttf",
            sizes: [16, 24],
        ),
    ],
    default_font: "pixel_comic",
    default_size: 14,
)
```

## Shader File Paths

### Core Shaders

| Shader | File Path | Purpose |
|--------|-----------|---------|
| Isometric Transform | `assets/shaders/isometric.wgsl` | World to screen projection |
| Creature Rendering | `assets/shaders/creature.wgsl` | Sprite + outline + tint |
| Terrain Blending | `assets/shaders/terrain_blend.wgsl` | Tile edge softening |
| Water Animation | `assets/shaders/water.wgsl` | Animated water tiles |
| Particle System | `assets/shaders/particles.wgsl` | GPU particle rendering |
| UI Elements | `assets/shaders/ui_cartoon.wgsl` | Cartoon UI effects |
| Post-Processing | `assets/shaders/post_process.wgsl` | Screen effects |

### Shader Includes
```
assets/shaders/
├── includes/
│   ├── common.wgsl      // Shared functions
│   ├── noise.wgsl       // Noise functions
│   ├── color.wgsl       // Color utilities
│   └── animation.wgsl   // Animation helpers
├── isometric.wgsl
├── creature.wgsl
├── terrain_blend.wgsl
├── water.wgsl
├── particles.wgsl
├── ui_cartoon.wgsl
└── post_process.wgsl
```

## Audio File Specifications

### Sound Effects Format
- **Format**: OGG Vorbis (best compression)
- **Sample Rate**: 44.1 kHz
- **Bit Depth**: 16-bit
- **Channels**: Mono (for 3D positioning)
- **Compression**: Quality 6 (balanced)

### Music Format
- **Format**: OGG Vorbis
- **Sample Rate**: 44.1 kHz
- **Bit Depth**: 16-bit
- **Channels**: Stereo
- **Compression**: Quality 8 (higher quality)

### Audio File Organization
```
assets/audio/
├── sfx/
│   ├── creatures/
│   │   ├── chirp_happy.ogg
│   │   ├── chirp_sad.ogg
│   │   └── footsteps.ogg
│   ├── environment/
│   │   ├── water_splash.ogg
│   │   └── wind_ambient.ogg
│   └── ui/
│       ├── button_click.ogg
│       └── notification.ogg
└── music/
    ├── menu_theme.ogg
    └── gameplay_ambient.ogg
```

## Texture Atlas Specifications

### Creature Atlas Details
- **Max Size**: 2048x2048 pixels
- **Padding**: 2 pixels between sprites
- **Format**: PNG with premultiplied alpha
- **Mipmap Levels**: 4 (for LOD system)
- **Organization**: By creature type, then animation state

### Terrain Atlas Details
- **Max Size**: 2048x2048 pixels
- **Tile Padding**: 1 pixel (for filtering)
- **Format**: PNG without alpha (opaque tiles)
- **Mipmap Levels**: 3
- **Organization**: By biome, then tile variant

### UI Atlas Details
- **Max Size**: 1024x1024 pixels
- **9-Slice Borders**: 8 pixels
- **Format**: PNG with alpha
- **Organization**: By UI element type

## Asset Loading Priority

### Critical Assets (Load First)
1. Terrain base tiles
2. Creature idle animations
3. UI framework sprites
4. Essential particle textures

### Secondary Assets (Load On-Demand)
1. Creature action animations
2. Weather particles
3. Emotion bubbles
4. Sound effects

### Optional Assets (Background Loading)
1. Music tracks
2. Rare animation states
3. Seasonal decorations
4. Achievement icons

## Memory Budget Allocations

| Asset Type | Memory Budget | Notes |
|------------|---------------|-------|
| Creature Textures | 32 MB | All species and variations |
| Terrain Textures | 24 MB | All biomes |
| Particle Textures | 8 MB | All effects |
| UI Textures | 16 MB | All panels and widgets |
| Fonts | 4 MB | All font files |
| Shaders | 2 MB | Compiled shaders |
| **Total** | **86 MB** | Within 128 MB target |

## Asset Validation Requirements

### Texture Validation
- Power of 2 dimensions (except UI elements)
- Proper alpha channel handling
- No color profile embedding
- Maximum dimension: 4096 pixels

### Audio Validation
- Proper loop points for ambient sounds
- Normalized volume levels (-6 dB peak)
- No clipping or distortion
- Fade in/out for smooth transitions

### Font Validation
- Complete ASCII character set
- Kerning pairs defined
- Proper baseline alignment
- Hinting for small sizes