# Cartoon Graphics Technical Specification

This document provides detailed technical specifications for implementing the cartoon isometric graphics system, addressing all gaps identified in the design documents.

## Table of Contents
1. [Sprite Specifications](#sprite-specifications)
2. [Color Palettes](#color-palettes)
3. [Memory and Performance Budgets](#memory-and-performance-budgets)
4. [Coordinate Systems](#coordinate-systems)
5. [Shader Requirements](#shader-requirements)
6. [Animation System Details](#animation-system-details)
7. [Asset Management](#asset-management)
8. [Plugin Dependencies](#plugin-dependencies)
9. [Save/Load Integration](#saveload-integration)
10. [Accessibility Features](#accessibility-features)
11. [Mod Support](#mod-support)
12. [Audio Integration](#audio-integration)

## Sprite Specifications

### Tile Dimensions
```
Base tile size: 64x32 pixels (2:1 isometric ratio)
Render scale options: 0.5x, 1x, 2x (for different screen resolutions)
Tile anchor point: Bottom center (32, 32)
```

### Creature Sprites
```
Frame dimensions: 48x48 pixels
Sprite sheet layout: 8x8 grid (64 frames total)
Anchor point: Bottom center (24, 48)
Shadow sprite: 32x16 pixels (separate layer)

Animation frame counts:
- Idle: 4 frames
- Walk: 8 frames  
- Run: 6 frames
- Eat: 6 frames
- Sleep: 4 frames
- Talk: 8 frames
- Attack: 6 frames
- Death: 8 frames
- Special/Emote: 4 frames each (happy, sad, angry, curious)
```

### Resource Sprites
```
Small resources (berries, seeds): 16x16 pixels
Medium resources (plants, rocks): 32x32 pixels
Large resources (trees, boulders): 64x64 pixels
Water tiles: 64x32 pixels (animated, 8 frames)
```

### UI Elements
```
Speech bubble: 9-slice, min 64x32, max 256x128
Emotion icons: 24x24 pixels
Health/need bars: 32x6 pixels
Selection highlight: 72x36 pixels (animated, 4 frames)
```

## Color Palettes

### Base Palette (Shared)
```rust
const PALETTE_BASE: &[(u8, u8, u8)] = &[
    (255, 255, 255), // White
    (0, 0, 0),       // Black
    (64, 64, 64),    // Dark Gray
    (128, 128, 128), // Gray
    (192, 192, 192), // Light Gray
];
```

### Biome Palettes
```rust
// Forest Biome
const PALETTE_FOREST: &[(u8, u8, u8)] = &[
    (34, 139, 34),   // Forest Green
    (46, 125, 50),   // Dark Green
    (139, 195, 74),  // Light Green
    (121, 85, 72),   // Tree Bark Brown
    (161, 136, 127), // Light Wood
    (255, 235, 59),  // Sunlight Yellow
    (156, 204, 101), // Leaf Green
    (104, 159, 56),  // Moss Green
];

// Desert Biome
const PALETTE_DESERT: &[(u8, u8, u8)] = &[
    (237, 201, 175), // Sand Light
    (205, 170, 125), // Sand Medium
    (173, 140, 95),  // Sand Dark
    (46, 125, 50),   // Oasis Green
    (30, 136, 229),  // Oasis Blue
    (255, 193, 7),   // Sun Yellow
    (139, 195, 74),  // Cactus Green
    (121, 85, 72),   // Rock Brown
];

// Tundra Biome
const PALETTE_TUNDRA: &[(u8, u8, u8)] = &[
    (240, 248, 255), // Snow White
    (176, 224, 230), // Ice Blue
    (119, 136, 153), // Stone Gray
    (70, 130, 180),  // Steel Blue
    (255, 250, 250), // Pure Snow
    (192, 192, 192), // Ice Gray
    (47, 79, 79),    // Dark Slate
    (112, 128, 144), // Slate Gray
];
```

### Creature Color Variations
```rust
// Base creature colors (tinted based on genetics)
const CREATURE_TINTS: &[(u8, u8, u8)] = &[
    (255, 255, 255), // No tint (albino)
    (255, 228, 196), // Peach
    (210, 180, 140), // Tan
    (139, 90, 43),   // Brown
    (105, 105, 105), // Gray
    (255, 218, 185), // Light Orange
    (188, 143, 143), // Rosy Brown
    (160, 82, 45),   // Sienna
];
```

## Memory and Performance Budgets

### Texture Atlas Limits
```rust
const MAX_ATLAS_SIZE: u32 = 4096; // Maximum texture atlas dimension
const MAX_ATLASES: usize = 8;     // Maximum number of texture atlases

// Per-atlas budgets
const CREATURE_ATLAS_SIZE: u32 = 2048;
const TERRAIN_ATLAS_SIZE: u32 = 2048;
const UI_ATLAS_SIZE: u32 = 1024;
const PARTICLE_ATLAS_SIZE: u32 = 512;
```

### Particle Limits
```rust
const MAX_PARTICLES_PER_EMITTER: u32 = 100;
const MAX_ACTIVE_EMITTERS: u32 = 50;
const MAX_WEATHER_PARTICLES: u32 = 1000;

// Per-effect limits
const EMOTION_PARTICLES: u32 = 20;
const ACTION_PARTICLES: u32 = 50;
const DEATH_PARTICLES: u32 = 100;
```

### Performance Thresholds
```rust
// When to trigger quality degradation
const FPS_THRESHOLD_HIGH: f32 = 55.0;    // Target maintenance
const FPS_THRESHOLD_MEDIUM: f32 = 45.0;  // Start reducing quality
const FPS_THRESHOLD_LOW: f32 = 30.0;     // Aggressive reduction

// Quality levels
#[derive(Debug, Clone, Copy)]
pub enum QualityLevel {
    Ultra,   // All effects, max particles, full animations
    High,    // Most effects, 80% particles, full animations
    Medium,  // Core effects, 50% particles, reduced animations
    Low,     // Minimal effects, 20% particles, key animations only
    Minimum, // No particles, critical animations only
}
```

## Coordinate Systems

### Screen to Isometric World
```rust
/// Convert screen coordinates to isometric world position
pub fn screen_to_world(screen_x: f32, screen_y: f32, camera: &Camera) -> Vec2 {
    let screen_pos = Vec2::new(screen_x, screen_y);
    let world_pos = camera.screen_to_world_2d(screen_pos);
    
    // Isometric transformation
    let iso_x = (world_pos.x / TILE_WIDTH + world_pos.y / TILE_HEIGHT) / 2.0;
    let iso_y = (world_pos.y / TILE_HEIGHT - world_pos.x / TILE_WIDTH) / 2.0;
    
    Vec2::new(iso_x, iso_y)
}

/// Convert isometric world position to screen coordinates
pub fn world_to_screen(world_x: f32, world_y: f32) -> Vec2 {
    let screen_x = (world_x - world_y) * TILE_WIDTH / 2.0;
    let screen_y = (world_x + world_y) * TILE_HEIGHT / 2.0;
    
    Vec2::new(screen_x, screen_y)
}

const TILE_WIDTH: f32 = 64.0;
const TILE_HEIGHT: f32 = 32.0;
```

### Depth Sorting
```rust
/// Calculate render depth for isometric sorting
pub fn calculate_depth(world_pos: Vec2, z_offset: f32) -> f32 {
    // Base depth from position
    let base_depth = world_pos.x + world_pos.y;
    
    // Apply z-offset for layering (tiles < objects < creatures < UI)
    base_depth + z_offset
}

// Z-offset constants
const Z_OFFSET_TILE: f32 = 0.0;
const Z_OFFSET_RESOURCE: f32 = 100.0;
const Z_OFFSET_CREATURE: f32 = 200.0;
const Z_OFFSET_PARTICLE: f32 = 300.0;
const Z_OFFSET_UI: f32 = 400.0;
```

## Shader Requirements

### Water Shader
```wgsl
// Water animation shader for tile-based water
@fragment
fn water_fragment(
    @location(0) uv: vec2<f32>,
    @builtin(position) position: vec4<f32>,
) -> @location(0) vec4<f32> {
    let time = globals.time;
    
    // Wave distortion
    let wave1 = sin(position.x * 0.1 + time * 2.0) * 0.05;
    let wave2 = cos(position.y * 0.1 + time * 1.5) * 0.05;
    
    // Sample texture with distortion
    let distorted_uv = uv + vec2<f32>(wave1, wave2);
    var color = textureSample(texture, sampler, distorted_uv);
    
    // Add shimmer
    let shimmer = sin(position.x * 0.5 + position.y * 0.5 + time * 3.0) * 0.1 + 0.9;
    color.rgb *= shimmer;
    
    return color;
}
```

### Day/Night Cycle Shader
```wgsl
// Global lighting shader for day/night cycle
@fragment
fn daynight_fragment(
    @location(0) uv: vec2<f32>,
    @location(1) world_position: vec2<f32>,
) -> @location(0) vec4<f32> {
    var color = textureSample(texture, sampler, uv);
    
    // Time of day (0.0 = midnight, 0.5 = noon, 1.0 = midnight)
    let time_of_day = fract(globals.time / DAY_DURATION);
    
    // Calculate sun position and intensity
    let sun_angle = time_of_day * 2.0 * PI;
    let sun_intensity = max(0.0, sin(sun_angle));
    
    // Dawn/dusk colors
    let dawn_color = vec3<f32>(1.0, 0.7, 0.5);
    let day_color = vec3<f32>(1.0, 1.0, 0.9);
    let dusk_color = vec3<f32>(0.8, 0.5, 0.7);
    let night_color = vec3<f32>(0.3, 0.3, 0.5);
    
    // Blend colors based on time
    var light_color: vec3<f32>;
    if (time_of_day < 0.25) {
        // Night to dawn
        light_color = mix(night_color, dawn_color, time_of_day * 4.0);
    } else if (time_of_day < 0.5) {
        // Dawn to day
        light_color = mix(dawn_color, day_color, (time_of_day - 0.25) * 4.0);
    } else if (time_of_day < 0.75) {
        // Day to dusk
        light_color = mix(day_color, dusk_color, (time_of_day - 0.5) * 4.0);
    } else {
        // Dusk to night
        light_color = mix(dusk_color, night_color, (time_of_day - 0.75) * 4.0);
    }
    
    // Apply lighting
    color.rgb *= light_color * (0.3 + sun_intensity * 0.7);
    
    return color;
}
```

### Outline Shader (Selection/Hover)
```wgsl
// Outline shader for selected entities
@fragment
fn outline_fragment(
    @location(0) uv: vec2<f32>,
) -> @location(0) vec4<f32> {
    let color = textureSample(texture, sampler, uv);
    
    // Check alpha threshold for outline
    if (color.a < 0.1) {
        // Sample neighboring pixels
        let offset = 1.0 / 256.0; // Assuming 256x256 texture
        let left = textureSample(texture, sampler, uv + vec2<f32>(-offset, 0.0)).a;
        let right = textureSample(texture, sampler, uv + vec2<f32>(offset, 0.0)).a;
        let top = textureSample(texture, sampler, uv + vec2<f32>(0.0, -offset)).a;
        let bottom = textureSample(texture, sampler, uv + vec2<f32>(0.0, offset)).a;
        
        // If any neighbor has alpha, draw outline
        if (left > 0.1 || right > 0.1 || top > 0.1 || bottom > 0.1) {
            return vec4<f32>(1.0, 1.0, 0.0, 1.0); // Yellow outline
        }
    }
    
    return color;
}
```

## Animation System Details

### Animation State Machine
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnimationState {
    Idle,
    Walking,
    Running,
    Eating,
    Sleeping,
    Talking,
    Attacking,
    Dying,
    Dead,
    Emoting(EmotionType),
}

#[derive(Debug, Clone, Copy)]
pub struct AnimationTransition {
    pub from: AnimationState,
    pub to: AnimationState,
    pub duration: f32,
    pub blend_curve: BlendCurve,
}

#[derive(Debug, Clone, Copy)]
pub enum BlendCurve {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

impl AnimationTransition {
    pub fn blend(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self.blend_curve {
            BlendCurve::Linear => t,
            BlendCurve::EaseIn => t * t,
            BlendCurve::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
            BlendCurve::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - 2.0 * (1.0 - t) * (1.0 - t)
                }
            }
        }
    }
}

// Transition definitions
const ANIMATION_TRANSITIONS: &[(AnimationState, AnimationState, f32, BlendCurve)] = &[
    (AnimationState::Idle, AnimationState::Walking, 0.2, BlendCurve::EaseIn),
    (AnimationState::Walking, AnimationState::Running, 0.15, BlendCurve::Linear),
    (AnimationState::Running, AnimationState::Walking, 0.15, BlendCurve::Linear),
    (AnimationState::Walking, AnimationState::Idle, 0.3, BlendCurve::EaseOut),
    // ... more transitions
];
```

### Frame Rate Adaptation
```rust
pub struct AnimationFrameRate {
    pub base_fps: f32,
    pub lod_multipliers: [f32; 4], // LOD levels 0-3
}

const ANIMATION_FRAMERATES: &[(AnimationState, AnimationFrameRate)] = &[
    (AnimationState::Idle, AnimationFrameRate {
        base_fps: 4.0,
        lod_multipliers: [1.0, 0.5, 0.25, 0.0], // Stop at LOD 3
    }),
    (AnimationState::Walking, AnimationFrameRate {
        base_fps: 8.0,
        lod_multipliers: [1.0, 0.75, 0.5, 0.25],
    }),
    (AnimationState::Running, AnimationFrameRate {
        base_fps: 12.0,
        lod_multipliers: [1.0, 0.75, 0.5, 0.5], // Maintain visibility
    }),
    // ... more states
];
```

## Asset Management

### Asset Loading Pipeline
```rust
pub struct AssetLoadingConfig {
    pub max_concurrent_loads: usize,
    pub retry_attempts: u32,
    pub retry_delay_ms: u64,
    pub fallback_strategy: FallbackStrategy,
}

#[derive(Debug, Clone, Copy)]
pub enum FallbackStrategy {
    UseDefault,      // Load default placeholder asset
    UseLastValid,    // Keep using previous asset
    HideEntity,      // Don't render if asset missing
    ShowError,       // Display error sprite
}

impl Default for AssetLoadingConfig {
    fn default() -> Self {
        Self {
            max_concurrent_loads: 8,
            retry_attempts: 3,
            retry_delay_ms: 100,
            fallback_strategy: FallbackStrategy::UseDefault,
        }
    }
}
```

### Fallback Assets
```rust
// Define paths to fallback assets
const FALLBACK_CREATURE_SPRITE: &str = "sprites/fallback/creature_default.png";
const FALLBACK_TILE_SPRITE: &str = "sprites/fallback/tile_default.png";
const FALLBACK_RESOURCE_SPRITE: &str = "sprites/fallback/resource_default.png";
const ERROR_SPRITE: &str = "sprites/fallback/error.png";

// Fallback colors for when textures can't load
const FALLBACK_CREATURE_COLOR: Color = Color::rgb(0.5, 0.5, 0.5);
const FALLBACK_TILE_COLOR: Color = Color::rgb(0.3, 0.3, 0.3);
const ERROR_COLOR: Color = Color::rgb(1.0, 0.0, 1.0); // Magenta
```

### Asset Preloading
```rust
pub struct PreloadManifest {
    pub essential: Vec<String>,    // Must load before game starts
    pub high_priority: Vec<String>, // Load immediately after essential
    pub normal: Vec<String>,       // Load during gameplay
    pub low_priority: Vec<String>, // Load when idle
}

// Example manifest
const PRELOAD_MANIFEST: PreloadManifest = PreloadManifest {
    essential: vec![
        "sprites/ui/loading.png",
        "sprites/creatures/basic_creature.png",
        "sprites/terrain/grass_tile.png",
    ],
    high_priority: vec![
        "sprites/creatures/creature_variations.png",
        "sprites/resources/common_resources.png",
    ],
    normal: vec![
        "sprites/particles/emotions.png",
        "sprites/weather/rain.png",
    ],
    low_priority: vec![
        "sprites/decorations/rare_decorations.png",
    ],
};
```

## Plugin Dependencies

### Required Bevy Plugins
```toml
[dependencies]
bevy = { version = "0.14", features = ["dynamic_linking"] }
bevy_ecs_tilemap = "0.14"  # For isometric tile rendering
bevy_hanabi = "0.12"       # For particle effects
bevy_easings = "0.14"      # For animation curves
bevy_asset_loader = "0.20" # For organized asset loading
bevy_kira_audio = "0.19"   # For audio integration
```

### Plugin Configuration
```rust
// Tilemap plugin configuration
pub struct TilemapConfig {
    pub chunk_size: UVec2,
    pub tile_size: Vec2,
    pub texture_atlas: Handle<Image>,
    pub render_method: RenderMethod,
}

impl Default for TilemapConfig {
    fn default() -> Self {
        Self {
            chunk_size: UVec2::new(32, 32),
            tile_size: Vec2::new(64.0, 32.0),
            texture_atlas: Handle::default(),
            render_method: RenderMethod::Batched,
        }
    }
}

// Particle system configuration
pub struct ParticleConfig {
    pub max_particles: u32,
    pub emission_rate: f32,
    pub lifetime: f32,
    pub initial_velocity: Vec3,
    pub acceleration: Vec3,
    pub size_curve: Curve<f32>,
    pub color_curve: Curve<Color>,
}
```

## Save/Load Integration

### Visual State Serialization
```rust
#[derive(Serialize, Deserialize)]
pub struct CreatureVisualState {
    pub animation_state: AnimationState,
    pub current_frame: usize,
    pub frame_timer: f32,
    pub expression: Option<EmotionType>,
    pub particle_emitters: Vec<ParticleEmitterState>,
    pub speech_bubble: Option<SpeechBubbleState>,
    pub color_tint: [f32; 4],
    pub facing_direction: f32,
}

#[derive(Serialize, Deserialize)]
pub struct ParticleEmitterState {
    pub effect_type: String,
    pub position_offset: [f32; 3],
    pub time_alive: f32,
    pub active: bool,
}

#[derive(Serialize, Deserialize)]
pub struct SpeechBubbleState {
    pub text: String,
    pub time_remaining: f32,
    pub emotion_icon: Option<String>,
}
```

### Camera State Persistence
```rust
#[derive(Serialize, Deserialize)]
pub struct CameraState {
    pub position: [f32; 3],
    pub zoom_level: f32,
    pub rotation: f32,
    pub follow_entity: Option<u32>, // Entity ID if following
}
```

## Accessibility Features

### Visual Accessibility
```rust
#[derive(Debug, Clone, Copy)]
pub enum ColorBlindMode {
    None,
    Protanopia,   // Red-blind
    Deuteranopia, // Green-blind
    Tritanopia,   // Blue-blind
    Monochrome,   // Full colorblind
}

pub struct AccessibilityConfig {
    pub colorblind_mode: ColorBlindMode,
    pub high_contrast: bool,
    pub reduce_motion: bool,
    pub particle_density: f32, // 0.0 to 1.0
    pub flash_reduction: bool,
    pub larger_ui: bool,
    pub outline_thickness: f32,
}

impl Default for AccessibilityConfig {
    fn default() -> Self {
        Self {
            colorblind_mode: ColorBlindMode::None,
            high_contrast: false,
            reduce_motion: false,
            particle_density: 1.0,
            flash_reduction: false,
            larger_ui: false,
            outline_thickness: 1.0,
        }
    }
}

// Color transformation matrices for colorblind modes
const PROTANOPIA_MATRIX: [[f32; 3]; 3] = [
    [0.567, 0.433, 0.0],
    [0.558, 0.442, 0.0],
    [0.0, 0.242, 0.758],
];

const DEUTERANOPIA_MATRIX: [[f32; 3]; 3] = [
    [0.625, 0.375, 0.0],
    [0.7, 0.3, 0.0],
    [0.0, 0.3, 0.7],
];

const TRITANOPIA_MATRIX: [[f32; 3]; 3] = [
    [0.95, 0.05, 0.0],
    [0.0, 0.433, 0.567],
    [0.0, 0.475, 0.525],
];
```

### Motion Sensitivity
```rust
pub struct MotionSensitivityConfig {
    pub disable_screen_shake: bool,
    pub disable_parallax: bool,
    pub reduce_animation_speed: f32, // Multiplier, 1.0 = normal
    pub disable_weather_effects: bool,
    pub disable_particle_effects: bool,
    pub smooth_camera_follow: bool,
    pub camera_smoothing_factor: f32,
}
```

## Mod Support

### Custom Sprite Loading
```rust
pub struct ModAssetManifest {
    pub mod_id: String,
    pub version: String,
    pub sprites: HashMap<String, SpriteDefinition>,
    pub animations: HashMap<String, AnimationDefinition>,
    pub particles: HashMap<String, ParticleDefinition>,
}

#[derive(Serialize, Deserialize)]
pub struct SpriteDefinition {
    pub path: String,
    pub sprite_type: SpriteType,
    pub dimensions: [u32; 2],
    pub anchor: [f32; 2],
    pub frames: Option<u32>,
    pub replace_original: bool,
}

#[derive(Serialize, Deserialize)]
pub enum SpriteType {
    Creature,
    Resource,
    Tile,
    UI,
    Particle,
}

// Mod loading system
pub struct ModLoader {
    pub mod_directory: PathBuf,
    pub loaded_mods: HashMap<String, ModAssetManifest>,
    pub sprite_overrides: HashMap<String, Handle<Image>>,
}

impl ModLoader {
    pub fn load_mod(&mut self, mod_path: &Path) -> Result<(), ModLoadError> {
        // Load manifest.json
        let manifest_path = mod_path.join("manifest.json");
        let manifest: ModAssetManifest = serde_json::from_reader(
            File::open(manifest_path)?
        )?;
        
        // Validate sprite dimensions and formats
        for (name, sprite_def) in &manifest.sprites {
            self.validate_sprite(&sprite_def)?;
            
            if sprite_def.replace_original {
                // Override original sprite
                let handle = self.load_sprite(&mod_path.join(&sprite_def.path))?;
                self.sprite_overrides.insert(name.clone(), handle);
            }
        }
        
        self.loaded_mods.insert(manifest.mod_id.clone(), manifest);
        Ok(())
    }
}
```

### Mod Asset Validation
```rust
pub struct ModAssetValidator {
    pub max_texture_size: u32,
    pub allowed_formats: Vec<String>,
    pub max_animation_frames: u32,
    pub max_file_size_mb: u32,
}

impl Default for ModAssetValidator {
    fn default() -> Self {
        Self {
            max_texture_size: 2048,
            allowed_formats: vec!["png".to_string(), "jpg".to_string()],
            max_animation_frames: 64,
            max_file_size_mb: 10,
        }
    }
}
```

## Audio Integration

### Sound Effect Triggers
```rust
#[derive(Debug, Clone)]
pub struct AnimationSoundTrigger {
    pub animation: AnimationState,
    pub frame: usize,
    pub sound_id: String,
    pub volume: f32,
    pub pitch_variation: f32,
}

// Define sound triggers for animations
const ANIMATION_SOUNDS: &[AnimationSoundTrigger] = &[
    AnimationSoundTrigger {
        animation: AnimationState::Walking,
        frame: 2,
        sound_id: "footstep_1".to_string(),
        volume: 0.5,
        pitch_variation: 0.1,
    },
    AnimationSoundTrigger {
        animation: AnimationState::Walking,
        frame: 6,
        sound_id: "footstep_2".to_string(),
        volume: 0.5,
        pitch_variation: 0.1,
    },
    AnimationSoundTrigger {
        animation: AnimationState::Eating,
        frame: 3,
        sound_id: "chomp".to_string(),
        volume: 0.7,
        pitch_variation: 0.2,
    },
    // ... more sound triggers
];
```

### Creature Vocalizations
```rust
pub struct VocalizationSystem {
    pub emotion_sounds: HashMap<EmotionType, Vec<String>>,
    pub action_sounds: HashMap<CreatureAction, Vec<String>>,
    pub ambient_sounds: HashMap<CreatureType, Vec<String>>,
}

// Example vocalization mapping
fn setup_vocalizations() -> VocalizationSystem {
    let mut system = VocalizationSystem::default();
    
    // Happy sounds
    system.emotion_sounds.insert(EmotionType::Happy, vec![
        "chirp_happy_1".to_string(),
        "chirp_happy_2".to_string(),
        "purr_content".to_string(),
    ]);
    
    // Action sounds
    system.action_sounds.insert(CreatureAction::Greeting, vec![
        "hello_chirp".to_string(),
        "friendly_trill".to_string(),
    ]);
    
    system
}
```

### Environmental Audio
```rust
pub struct BiomeAmbience {
    pub background_loops: Vec<String>,
    pub random_sounds: Vec<(String, f32)>, // (sound_id, probability)
    pub weather_sounds: HashMap<WeatherType, String>,
    pub time_based_sounds: HashMap<TimeOfDay, Vec<String>>,
}

// Example biome audio
const FOREST_AMBIENCE: BiomeAmbience = BiomeAmbience {
    background_loops: vec!["forest_birds", "wind_trees"],
    random_sounds: vec![
        ("bird_call_1", 0.05),
        ("branch_crack", 0.02),
        ("leaves_rustle", 0.1),
    ],
    weather_sounds: {
        let mut map = HashMap::new();
        map.insert(WeatherType::Rain, "rain_on_leaves");
        map.insert(WeatherType::Storm, "thunder_distant");
        map
    },
    time_based_sounds: {
        let mut map = HashMap::new();
        map.insert(TimeOfDay::Dawn, vec!["morning_birds"]);
        map.insert(TimeOfDay::Night, vec!["crickets", "owl_hoot"]);
        map
    },
};
```

## Implementation Priority

1. **Critical Path** (Week 1-2):
   - Sprite specifications and asset loading
   - Basic isometric rendering
   - Coordinate system implementation
   - Core animation system

2. **Essential Features** (Week 3-4):
   - Color palettes and biome rendering
   - Particle system integration
   - UI elements and speech bubbles
   - Save/load visual states

3. **Polish and Optimization** (Week 5-6):
   - Shader implementation
   - Performance optimization
   - Accessibility features
   - Audio integration

4. **Extended Features** (Post-launch):
   - Full mod support
   - Advanced shaders
   - Network synchronization
   - Additional accessibility options

## Testing Requirements

### Visual Testing
```rust
#[cfg(test)]
mod visual_tests {
    use super::*;
    
    #[test]
    fn test_coordinate_conversion() {
        let screen = Vec2::new(100.0, 50.0);
        let world = screen_to_world(screen.x, screen.y, &mock_camera());
        let back = world_to_screen(world.x, world.y);
        
        assert!((screen - back).length() < 0.01);
    }
    
    #[test]
    fn test_depth_sorting() {
        let creature_depth = calculate_depth(Vec2::new(10.0, 10.0), Z_OFFSET_CREATURE);
        let tile_depth = calculate_depth(Vec2::new(10.0, 10.0), Z_OFFSET_TILE);
        
        assert!(creature_depth > tile_depth);
    }
    
    #[test]
    fn test_color_transform() {
        let color = Color::rgb(1.0, 0.0, 0.0); // Red
        let transformed = apply_colorblind_mode(color, ColorBlindMode::Protanopia);
        
        // Red should appear different for protanopia
        assert!(transformed.r() < 0.9);
    }
}
```

### Performance Benchmarks
```rust
#[bench]
fn bench_sprite_batching(b: &mut Bencher) {
    let mut app = App::new();
    // Setup sprite batching system
    
    b.iter(|| {
        // Render 1000 sprites
        for _ in 0..1000 {
            render_sprite(&mut app);
        }
    });
}
```

## Conclusion

This technical specification provides the detailed implementation requirements for the cartoon isometric graphics system. It addresses all gaps identified in the original design documents and provides concrete values, formulas, and code examples for implementation.

Key deliverables:
- Exact sprite dimensions and layouts
- Complete color palette definitions
- Memory and performance budgets
- Coordinate transformation formulas
- Shader implementations
- Animation blending system
- Asset loading and fallback strategies
- Plugin dependencies and configuration
- Accessibility and mod support systems
- Audio integration specifications

With these specifications, the implementation can proceed without ambiguity, ensuring consistent results across the development team.