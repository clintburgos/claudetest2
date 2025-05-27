use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use crate::components::{
    CartoonSprite, AnimationState, EmotionType, ExpressionOverlay, AnimatedSprite
};
use std::collections::HashMap;

// Animation timing constants (in seconds)
/// Frame time for idle animation - slow, relaxed breathing
const IDLE_FRAME_TIME: f32 = 0.3;
/// Frame time for walking animation - normal pace
const WALK_FRAME_TIME: f32 = 0.15;
/// Frame time for running animation - fast movement
const RUN_FRAME_TIME: f32 = 0.1;
/// Frame time for eating animation - moderate chewing speed
const EAT_FRAME_TIME: f32 = 0.2;
/// Frame time for sleeping animation - very slow breathing
const SLEEP_FRAME_TIME: f32 = 0.5;
/// Frame time for talking animation - moderate speech animation
const TALK_FRAME_TIME: f32 = 0.2;
/// Frame time for attack animation - fast, aggressive movement
const ATTACK_FRAME_TIME: f32 = 0.1;
/// Frame time for death animation - slow fade/collapse
const DEATH_FRAME_TIME: f32 = 0.3;
/// Frame time for special animations - moderate emotion display
const SPECIAL_FRAME_TIME: f32 = 0.25;

// Sprite dimensions
/// Base creature sprite size in pixels
const CREATURE_SPRITE_SIZE: f32 = 48.0;
/// Isometric tile width in pixels (2:1 ratio)
const TILE_WIDTH: f32 = 64.0;
/// Isometric tile height in pixels
const TILE_HEIGHT: f32 = 32.0;
/// Texture atlas grid size (8x8 = 64 sprites)
const ATLAS_GRID_SIZE: usize = 8;

// Genetic variation constants
/// Minimum creature size multiplier from genetics
const MIN_SIZE_SCALE: f32 = 0.7;
/// Maximum creature size multiplier from genetics
const MAX_SIZE_SCALE: f32 = 1.3;
/// Maximum hue shift from genetic color variation
const MAX_HUE_SHIFT: f32 = 0.1;
/// Pattern threshold for stripe pattern
const STRIPE_PATTERN_THRESHOLD: f32 = 0.7;
/// Pattern threshold for spot pattern
const SPOT_PATTERN_THRESHOLD: f32 = 0.4;

// Emotion threshold constants
/// Hunger level that triggers hungry emotion
const HUNGER_EMOTION_THRESHOLD: f32 = 0.8;
/// Energy level that triggers tired emotion
const TIRED_EMOTION_THRESHOLD: f32 = 0.2;
/// Critical need level that triggers frightened emotion
const CRITICAL_NEED_THRESHOLD: f32 = 0.1;
/// Low need level that triggers angry emotion (with low social)
const ANGRY_NEED_THRESHOLD: f32 = 0.3;
/// Low need level that triggers sad emotion
const SAD_NEED_THRESHOLD: f32 = 0.4;
/// High need level that triggers content emotion
const CONTENT_NEED_THRESHOLD: f32 = 0.7;
/// Minimum need level for curious emotion while moving
const CURIOUS_NEED_THRESHOLD: f32 = 0.5;

// Visual state modifiers
/// Color multiplier for sleeping creatures
const SLEEP_COLOR_MULTIPLIER: f32 = 0.7;
/// Color multiplier for dead creatures
const DEATH_COLOR_MULTIPLIER: f32 = 0.3;

// Movement thresholds
/// Velocity threshold for running animation (units/second)
const RUN_VELOCITY_THRESHOLD: f32 = 2.0;

/// Main plugin for cartoon-style isometric rendering
/// 
/// This plugin handles:
/// - Loading and managing sprite atlases for creatures and terrain
/// - Updating creature animations based on their state
/// - Managing expression overlays for emotional states
/// - Rendering biome-specific terrain tiles
pub struct CartoonRenderingPlugin;

impl Plugin for CartoonRenderingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CartoonAssets::default())
            .insert_resource(BiomeRenderer::default())
            .insert_resource(ExpressionSystem::default())
            .insert_resource(AssetLoadingState::default())
            .insert_resource(CartoonVisualConfig::default())
            .add_systems(Startup, setup_cartoon_rendering)
            .add_systems(
                Update,
                (
                    check_asset_loading_state,
                    update_cartoon_sprites,
                    update_creature_animations,
                    animate_sprites,
                    update_expression_overlays,
                    render_biome_tiles,
                )
                    .chain(),
            )
            // Add particle effects and speech bubble plugins
            .add_plugins((
                crate::rendering::ParticleEffectsPlugin,
                crate::rendering::SpeechBubblePlugin,
            ));
    }
}

/// Resource containing loaded cartoon assets and texture atlases
/// 
/// Stores handles to all sprite sheets and their corresponding atlas layouts
/// for efficient sprite rendering
#[derive(Resource, Default)]
pub struct CartoonAssets {
    /// Handle to the creature sprite sheet image
    pub creature_atlas: Handle<Image>,
    /// Handle to the terrain sprite sheet image
    pub terrain_atlas: Handle<Image>,
    /// Texture atlas layout for creature sprites
    pub creature_atlas_layout: Handle<TextureAtlasLayout>,
    /// Texture atlas layout for terrain sprites
    pub terrain_atlas_layout: Handle<TextureAtlasLayout>,
    /// Individual particle textures mapped by name
    pub particle_textures: HashMap<String, Handle<Image>>,
    /// Mesh handles for different biome tile types
    pub tile_meshes: HashMap<BiomeType, Handle<Mesh>>,
}

/// Biome rendering system
#[derive(Resource)]
pub struct BiomeRenderer {
    pub chunk_size: u32,
    pub visible_chunks: Vec<ChunkCoord>,
    pub tile_entities: HashMap<ChunkCoord, Vec<Entity>>,
}

impl Default for BiomeRenderer {
    fn default() -> Self {
        Self {
            chunk_size: 16,
            visible_chunks: Vec::new(),
            tile_entities: HashMap::new(),
        }
    }
}

/// Expression management system
#[derive(Resource, Default)]
pub struct ExpressionSystem {
    pub emotion_priorities: HashMap<EmotionType, f32>,
    pub blend_durations: HashMap<(EmotionType, EmotionType), f32>,
}

/// Visual configuration parameters for the cartoon rendering system
/// Allows runtime tweaking of visual properties without recompiling
#[derive(Resource)]
pub struct CartoonVisualConfig {
    /// Animation speed multiplier (1.0 = normal speed)
    pub animation_speed_multiplier: f32,
    /// Global brightness adjustment for all sprites
    pub global_brightness: f32,
    /// Enable/disable particle effects
    pub particles_enabled: bool,
    /// Maximum particle count for performance
    pub max_particles: usize,
    /// Enable/disable expression overlays
    pub expressions_enabled: bool,
    /// Shadow opacity (0.0 = no shadows, 1.0 = full shadows)
    pub shadow_opacity: f32,
    /// Outline thickness for sprites
    pub outline_thickness: f32,
    /// Enable/disable genetic variations
    pub genetic_variations_enabled: bool,
    /// Quality preset
    pub quality_preset: QualityPreset,
}

impl Default for CartoonVisualConfig {
    fn default() -> Self {
        Self {
            animation_speed_multiplier: 1.0,
            global_brightness: 1.0,
            particles_enabled: true,
            max_particles: 1000,
            expressions_enabled: true,
            shadow_opacity: 0.3,
            outline_thickness: 2.0,
            genetic_variations_enabled: true,
            quality_preset: QualityPreset::High,
        }
    }
}

/// Quality presets for different performance levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QualityPreset {
    Ultra,   // All effects, maximum quality
    High,    // Most effects, good performance
    Medium,  // Balanced quality and performance
    Low,     // Reduced effects for better performance
    Minimum, // Minimal effects for low-end devices
}

/// Tracks the loading state of cartoon rendering assets
/// Used to handle loading failures and provide feedback
#[derive(Resource, Default)]
pub struct AssetLoadingState {
    pub creature_atlas_loaded: bool,
    pub terrain_atlas_loaded: bool,
    pub particle_assets_loaded: bool,
    pub loading_failed: bool,
    pub error_message: Option<String>,
}

/// Chunk coordinate for terrain rendering
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct ChunkCoord {
    pub x: i32,
    pub y: i32,
}

/// Biome types for terrain rendering
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum BiomeType {
    Forest,
    Desert,
    Grassland,
    Tundra,
    Ocean,
}

/// Initialize cartoon rendering resources and load sprite atlases
/// 
/// This system attempts to load sprite atlases and falls back to procedural
/// generation if assets are missing. This ensures the game remains playable
/// during development or if assets fail to load.
fn setup_cartoon_rendering(
    _commands: Commands,
    _asset_server: Res<AssetServer>,
    mut cartoon_assets: ResMut<CartoonAssets>,
    mut expression_system: ResMut<ExpressionSystem>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    mut particle_assets: ResMut<crate::rendering::particles::ParticleAssets>,
    mut images: ResMut<Assets<Image>>,
    mut loading_state: ResMut<AssetLoadingState>,
) {
    info!("Initializing cartoon rendering system with procedural fallbacks");
    // Load texture atlases with fallback to procedural generation
    // We'll use procedural generation as the default for now since assets don't exist yet
    // When real assets are added, they will automatically be loaded instead
    
    // Generate procedural creature atlas as fallback
    // This creates a simple colored atlas that demonstrates the animation system
    info!("Generating procedural creature atlas as placeholder");
    cartoon_assets.creature_atlas = images.add(generate_procedural_creature_atlas());
    
    // Generate procedural terrain atlas as fallback
    // This creates colored isometric tiles for each biome type
    info!("Generating procedural terrain atlas as placeholder");
    cartoon_assets.terrain_atlas = images.add(generate_procedural_terrain_atlas());
    
    // Note: When actual sprite files are added to assets/sprites/, replace the above with:
    // cartoon_assets.creature_atlas = asset_server.load("sprites/creatures/creature_atlas.png");
    // cartoon_assets.terrain_atlas = asset_server.load("sprites/terrain/terrain_atlas.png");
    
    // Mark procedural assets as loaded immediately
    loading_state.creature_atlas_loaded = true;
    loading_state.terrain_atlas_loaded = true;
    
    // Create texture atlas layouts
    // Creature atlas: 8x8 grid of 48x48 sprites
    // Layout supports 64 unique animation frames:
    // - Rows 0-1: Idle animations (8 frames)
    // - Rows 2-3: Movement animations (16 frames)
    // - Rows 4-5: Action animations (16 frames)
    // - Rows 6-7: Special/emotion animations (16 frames)
    let creature_layout = TextureAtlasLayout::from_grid(
        Vec2::new(CREATURE_SPRITE_SIZE, CREATURE_SPRITE_SIZE), // Individual sprite size in pixels
        ATLAS_GRID_SIZE, // Columns in atlas
        ATLAS_GRID_SIZE, // Rows in atlas
        None, // No padding between sprites
        None, // No offset from atlas origin
    );
    cartoon_assets.creature_atlas_layout = texture_atlases.add(creature_layout);
    
    // Terrain atlas: 8x8 grid of 64x32 sprites (isometric tiles)
    // Standard 2:1 isometric ratio for proper depth sorting
    // Atlas organization:
    // - Row 0: Grassland tiles (8 variants)
    // - Row 1: Forest tiles (8 variants)
    // - Row 2: Desert tiles (8 variants)
    // - Row 3: Tundra tiles (8 variants)
    // - Row 4: Ocean/water tiles (8 variants)
    // - Row 5-7: Transition tiles and decorations
    let terrain_layout = TextureAtlasLayout::from_grid(
        Vec2::new(TILE_WIDTH, TILE_HEIGHT), // Isometric tile dimensions (2:1 ratio)
        ATLAS_GRID_SIZE, // Columns in atlas
        ATLAS_GRID_SIZE, // Rows in atlas
        None, // No padding between tiles
        None, // No offset from atlas origin
    );
    cartoon_assets.terrain_atlas_layout = texture_atlases.add(terrain_layout);
    
    // Load particle textures with fallback generation
    let particle_names = ["heart", "zzz", "sparkle", "sweat", "exclamation", "question"];
    for name in particle_names {
        // Generate simple procedural particle as fallback
        // When real particle assets are added, they can be loaded with asset_server.load()
        info!("Generating procedural '{}' particle as placeholder", name);
        let handle = images.add(generate_procedural_particle(name));
        
        // Store in both cartoon assets and particle assets
        cartoon_assets.particle_textures.insert(
            name.to_string(),
            handle.clone(),
        );
        particle_assets.textures.insert(
            name.to_string(),
            handle,
        );
    }
    
    // Mark particle assets as loaded
    loading_state.particle_assets_loaded = true;
    
    // Set up expression priorities for emotion system
    // Higher priority emotions override lower priority ones
    // Priority scale: 0.0 (lowest) to 1.0 (highest)
    // Critical emotions (danger/survival) have highest priority
    expression_system.emotion_priorities.insert(EmotionType::Angry, 0.9);      // Immediate threat response
    expression_system.emotion_priorities.insert(EmotionType::Frightened, 0.85); // Fear/danger response
    expression_system.emotion_priorities.insert(EmotionType::Sad, 0.7);        // Negative emotional state
    expression_system.emotion_priorities.insert(EmotionType::Hungry, 0.6);     // Basic need (food)
    expression_system.emotion_priorities.insert(EmotionType::Tired, 0.5);      // Basic need (rest)
    expression_system.emotion_priorities.insert(EmotionType::Happy, 0.4);      // Positive emotional state
    expression_system.emotion_priorities.insert(EmotionType::Curious, 0.3);    // Exploration state
    expression_system.emotion_priorities.insert(EmotionType::Content, 0.2);    // Satisfied state
    expression_system.emotion_priorities.insert(EmotionType::Neutral, 0.1);    // Default/baseline state
    
    // Set up blend durations for smooth transitions between emotions
    expression_system.blend_durations.insert((EmotionType::Neutral, EmotionType::Happy), 0.3);
    expression_system.blend_durations.insert((EmotionType::Happy, EmotionType::Sad), 0.5);
    expression_system.blend_durations.insert((EmotionType::Neutral, EmotionType::Angry), 0.2);
}

/// System to check and update asset loading state
/// Monitors asset handles and updates loading status
fn check_asset_loading_state(
    mut loading_state: ResMut<AssetLoadingState>,
    cartoon_assets: Res<CartoonAssets>,
    _asset_server: Res<AssetServer>,
    images: Res<Assets<Image>>,
) {
    // Check if we're using procedural assets (always loaded)
    if images.contains(&cartoon_assets.creature_atlas) {
        loading_state.creature_atlas_loaded = true;
    }
    
    if images.contains(&cartoon_assets.terrain_atlas) {
        loading_state.terrain_atlas_loaded = true;
    }
    
    // Check particle assets
    let all_particles_loaded = cartoon_assets.particle_textures.values()
        .all(|handle| images.contains(handle));
    loading_state.particle_assets_loaded = all_particles_loaded;
    
    // Log loading progress periodically
    if !loading_state.creature_atlas_loaded || !loading_state.terrain_atlas_loaded {
        debug!(
            "Asset loading status - Creature: {}, Terrain: {}, Particles: {}",
            loading_state.creature_atlas_loaded,
            loading_state.terrain_atlas_loaded,
            loading_state.particle_assets_loaded
        );
    }
}

/// System to create sprite components for creatures that don't have them yet
/// Uses the creature atlas to render animated sprites with fallback support
fn update_cartoon_sprites(
    mut commands: Commands,
    cartoon_assets: Res<CartoonAssets>,
    loading_state: Res<AssetLoadingState>,
    config: Res<CartoonVisualConfig>,
    creatures_without_sprites: Query<
        (Entity, &crate::components::CreatureType, &crate::components::Genetics),
        (
            With<crate::components::Creature>,
            Without<CartoonSprite>,
            Without<TextureAtlas>,
        ),
    >,
) {
    // Skip if assets aren't loaded yet
    if cartoon_assets.creature_atlas_layout == Handle::default() {
        return;
    }
    
    // Skip if creature atlas hasn't loaded
    if !loading_state.creature_atlas_loaded {
        return;
    }
    
    for (entity, creature_type, genetics) in creatures_without_sprites.iter() {
        // Create cartoon sprite component with genetic variations
        let mut cartoon_sprite = CartoonSprite::default();
        
        // Apply genetic variations to body modifiers if enabled
        if config.genetic_variations_enabled {
            // Size gene ranges from 0.0 to 1.0, mapped to MIN_SIZE_SCALE-MAX_SIZE_SCALE
            // This provides 60% size variation while keeping creatures recognizable
            let size_range = MAX_SIZE_SCALE - MIN_SIZE_SCALE;
            cartoon_sprite.body_modifiers.size_scale = MIN_SIZE_SCALE + (genetics.size * size_range);
        } else {
            // Use default size without variation
            cartoon_sprite.body_modifiers.size_scale = 1.0;
        }
        
        // Color tint based on creature type with genetic variation
        let base_color = match creature_type {
            crate::components::CreatureType::Herbivore => Color::rgb(0.7, 1.0, 0.7),
            crate::components::CreatureType::Carnivore => Color::rgb(1.0, 0.7, 0.7),
            crate::components::CreatureType::Omnivore => Color::rgb(0.9, 0.8, 0.7),
        };
        
        // Apply genetic color variation if enabled
        if config.genetic_variations_enabled {
            // Color gene (0.0-1.0) centered at 0.5, creates -MAX_HUE_SHIFT to +MAX_HUE_SHIFT
            // Keeps creatures recognizable while adding visual variety
            let hue_shift = (genetics.color - 0.5) * (MAX_HUE_SHIFT * 2.0);
            cartoon_sprite.body_modifiers.color_tint = Color::rgb(
                (base_color.r() + hue_shift).clamp(0.0, 1.0),
                base_color.g(),
                (base_color.b() - hue_shift).clamp(0.0, 1.0),
            );
        } else {
            cartoon_sprite.body_modifiers.color_tint = base_color;
        }
        
        // Determine pattern type based on genetics
        // High pattern values create stripes, medium creates spots, low creates no pattern
        cartoon_sprite.body_modifiers.pattern_type = if genetics.pattern > STRIPE_PATTERN_THRESHOLD {
            crate::components::PatternType::Stripes
        } else if genetics.pattern > SPOT_PATTERN_THRESHOLD {
            crate::components::PatternType::Spots
        } else {
            crate::components::PatternType::None
        };
        
        // Create animated sprite component for idle animation
        let idle_frames = (0..4).collect(); // First 4 frames are idle animation
        let animated_sprite = AnimatedSprite::new(idle_frames, 0.2, true);
        
        // Add sprite bundle with texture atlas
        // Use try_insert to handle potential conflicts gracefully
        if let Some(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.insert((
            SpriteBundle {
                sprite: Sprite {
                    color: cartoon_sprite.body_modifiers.color_tint,
                    custom_size: Some(Vec2::new(
                        CREATURE_SPRITE_SIZE * cartoon_sprite.body_modifiers.size_scale,
                        CREATURE_SPRITE_SIZE * cartoon_sprite.body_modifiers.size_scale
                    )),
                    ..default()
                },
                texture: cartoon_assets.creature_atlas.clone(),
                transform: Transform::from_scale(Vec3::splat(1.0)),
                ..default()
            },
            TextureAtlas {
                layout: cartoon_assets.creature_atlas_layout.clone(),
                index: 0,
            },
            cartoon_sprite,
            animated_sprite,
            ));
        } else {
            warn!("Failed to add cartoon sprite to entity {:?}", entity);
        }
    }
}

/// System to update creature animations based on their current state
/// Changes the animation frames when creature behavior changes
fn update_creature_animations(
    _time: Res<Time>,
    config: Res<CartoonVisualConfig>,
    mut query: Query<(
        &mut CartoonSprite,
        &mut AnimatedSprite,
        &mut Sprite,
        &crate::components::CreatureState,
        &crate::components::Velocity,
        Option<&crate::components::ConversationState>,
    )>,
) {
    for (mut cartoon_sprite, mut animated_sprite, mut sprite, state, velocity, conversation) in query.iter_mut() {
        // Determine the appropriate animation based on current state
        let new_animation = determine_animation_state(state, velocity, conversation);
        
        // Update animation if it has changed
        if cartoon_sprite.base_animation != new_animation {
            apply_animation_change(
                &mut cartoon_sprite,
                &mut animated_sprite,
                new_animation,
                &config
            );
        }
        
        // Apply visual modifiers based on current animation
        apply_animation_color_modifiers(&mut sprite, &cartoon_sprite);
    }
}

/// Determine which animation state should be active based on creature state
/// 
/// Priority order:
/// 1. Death (overrides all)
/// 2. Conversation (when idle)
/// 3. Action states (eating, drinking, resting)
/// 4. Movement states (walk/run based on velocity)
/// 5. Default idle
fn determine_animation_state(
    state: &crate::components::CreatureState,
    velocity: &crate::components::Velocity,
    conversation: Option<&crate::components::ConversationState>,
) -> AnimationState {
    match state {
        crate::components::CreatureState::Dead => AnimationState::Death,
        crate::components::CreatureState::Idle => {
            // Talking takes priority when idle
            if conversation.is_some() {
                AnimationState::Talk
            } else {
                AnimationState::Idle
            }
        },
        crate::components::CreatureState::Moving { .. } => {
            // Use velocity magnitude to determine walk vs run animation
            if velocity.0.length() > RUN_VELOCITY_THRESHOLD {
                AnimationState::Run
            } else {
                AnimationState::Walk
            }
        },
        crate::components::CreatureState::Eating => AnimationState::Eat,
        crate::components::CreatureState::Drinking => AnimationState::Eat, // Reuse eat animation for drinking
        crate::components::CreatureState::Resting => AnimationState::Sleep,
    }
}

/// Apply animation changes to the sprite components
/// 
/// Updates:
/// - Animation frame range
/// - Frame timing
/// - Looping behavior
fn apply_animation_change(
    cartoon_sprite: &mut CartoonSprite,
    animated_sprite: &mut AnimatedSprite,
    new_animation: AnimationState,
    config: &CartoonVisualConfig,
) {
    // Update the animation state
    cartoon_sprite.base_animation = new_animation;
    
    // Get frame range for the new animation
    let (start_frame, frame_count) = get_animation_frames(new_animation);
    animated_sprite.frames = (start_frame..start_frame + frame_count).collect();
    animated_sprite.current_frame = 0;
    
    // Set animation speed based on type
    let frame_time = get_animation_frame_time(new_animation, config);
    animated_sprite.timer = Timer::from_seconds(frame_time, TimerMode::Repeating);
    
    // Death animations should not loop
    animated_sprite.looping = !matches!(new_animation, AnimationState::Death);
}

/// Get the frame timing for each animation type
/// Lower values result in faster animations
/// The timing is affected by the global animation speed multiplier
fn get_animation_frame_time(animation: AnimationState, config: &CartoonVisualConfig) -> f32 {
    let base_time = match animation {
        AnimationState::Idle => IDLE_FRAME_TIME,
        AnimationState::Walk => WALK_FRAME_TIME,
        AnimationState::Run => RUN_FRAME_TIME,
        AnimationState::Eat => EAT_FRAME_TIME,
        AnimationState::Sleep => SLEEP_FRAME_TIME,
        AnimationState::Talk => TALK_FRAME_TIME,
        AnimationState::Attack => ATTACK_FRAME_TIME,
        AnimationState::Death => DEATH_FRAME_TIME,
        AnimationState::Special(_) => SPECIAL_FRAME_TIME,
    };
    
    // Apply speed multiplier (higher multiplier = faster animation)
    base_time / config.animation_speed_multiplier.max(0.1)
}

/// Apply color modifiers to sprites based on their animation state
/// 
/// Visual feedback:
/// - Sleeping creatures appear darker (70% brightness)
/// - Dead creatures are heavily darkened (30% brightness)
/// - All other states use normal coloring
fn apply_animation_color_modifiers(
    sprite: &mut Sprite,
    cartoon_sprite: &CartoonSprite,
) {
    sprite.color = match cartoon_sprite.base_animation {
        AnimationState::Sleep => cartoon_sprite.body_modifiers.color_tint * SLEEP_COLOR_MULTIPLIER,
        AnimationState::Death => cartoon_sprite.body_modifiers.color_tint * DEATH_COLOR_MULTIPLIER,
        _ => cartoon_sprite.body_modifiers.color_tint,
    };
}

/// System to update facial expression overlays based on creature emotions
/// Modifies eye, mouth, and brow positions to convey emotional states
fn update_expression_overlays(
    config: Res<CartoonVisualConfig>,
    mut query: Query<(
        &mut CartoonSprite,
        &crate::components::Needs,
        &crate::components::CreatureState,
    )>,
) {
    // Skip if expressions are disabled
    if !config.expressions_enabled {
        return;
    }
    for (mut cartoon_sprite, needs, state) in query.iter_mut() {
        // Determine emotion based on needs and state
        let emotion = determine_emotion_from_state(needs, state);
        
        // Ensure expression overlay exists
        ensure_expression_overlay(&mut cartoon_sprite);
        
        // Apply emotion-specific facial features
        if let Some(ref mut overlay) = cartoon_sprite.expression_overlay {
            apply_emotion_to_expression(overlay, emotion);
        }
    }
}

/// Ensure the creature has an expression overlay component
/// Creates a default neutral expression if none exists
fn ensure_expression_overlay(cartoon_sprite: &mut CartoonSprite) {
    if cartoon_sprite.expression_overlay.is_none() {
        cartoon_sprite.expression_overlay = Some(ExpressionOverlay {
            eye_offset: Vec2::ZERO,
            eye_scale: 1.0,
            mouth_curve: 0.0,
            mouth_open: 0.0,
            brow_angle: 0.0,
        });
    }
}

/// Apply emotion-specific parameters to facial expression overlay
/// 
/// Expression parameters:
/// - mouth_curve: -1.0 (frown) to 1.0 (smile)
/// - eye_scale: 0.5 (squinted) to 1.5 (wide open)
/// - brow_angle: -30° (angry) to 30° (sad)
fn apply_emotion_to_expression(overlay: &mut ExpressionOverlay, emotion: EmotionType) {
    // Define expression parameters for each emotion
    let (mouth_curve, eye_scale, brow_angle) = match emotion {
        EmotionType::Happy => (
            0.5,   // Curved smile
            1.1,   // Slightly wider eyes
            -10.0  // Slightly raised brows
        ),
        EmotionType::Sad => (
            -0.5,  // Downturned mouth
            0.9,   // Slightly drooped eyes
            10.0   // Raised inner brows
        ),
        EmotionType::Angry => (
            -0.3,  // Slight frown
            0.8,   // Narrowed eyes
            -20.0  // Furrowed brows
        ),
        EmotionType::Frightened => (
            -0.2,  // Slightly open mouth
            1.3,   // Wide eyes
            15.0   // Raised brows
        ),
        EmotionType::Tired => (
            -0.1,  // Slight droop
            0.7,   // Half-closed eyes
            5.0    // Relaxed brows
        ),
        EmotionType::Hungry => (
            -0.2,  // Slight frown
            1.0,   // Normal eyes
            0.0    // Neutral brows
        ),
        EmotionType::Curious => (
            0.1,   // Slight smile
            1.2,   // Wide, alert eyes
            -5.0   // One raised brow (asymmetric in full impl)
        ),
        EmotionType::Content => (
            0.3,   // Gentle smile
            1.0,   // Relaxed eyes
            0.0    // Neutral brows
        ),
        EmotionType::Neutral => (
            0.0,   // Neutral mouth
            1.0,   // Normal eyes
            0.0    // Neutral brows
        ),
    };
    
    // Apply the expression parameters
    overlay.mouth_curve = mouth_curve;
    overlay.eye_scale = eye_scale;
    overlay.brow_angle = brow_angle;
}

fn render_biome_tiles(
    mut _commands: Commands,
    _biome_renderer: Res<BiomeRenderer>,
    camera_query: Query<&Transform, With<crate::plugins::MainCamera>>,
) {
    // This will be implemented when we have the world/biome system in place
    // For now, just a placeholder
    if let Ok(_camera_transform) = camera_query.get_single() {
        // Calculate visible chunks based on camera position
        // Render terrain tiles for visible chunks
    }
}

/// System to animate sprites by updating their texture atlas indices
fn animate_sprites(
    time: Res<Time>,
    mut query: Query<(&mut AnimatedSprite, &mut TextureAtlas)>,
    loading_state: Res<AssetLoadingState>,
) {
    // Skip animation if assets haven't loaded
    if !loading_state.creature_atlas_loaded {
        return;
    }
    
    for (mut animated_sprite, mut atlas) in query.iter_mut() {
        // Tick the animation timer
        animated_sprite.timer.tick(time.delta());
        
        // Update frame if timer finished
        if animated_sprite.timer.finished() {
            if animated_sprite.looping {
                // Loop back to start
                animated_sprite.current_frame = 
                    (animated_sprite.current_frame + 1) % animated_sprite.frames.len();
            } else {
                // Play once and stop at last frame
                animated_sprite.current_frame = 
                    (animated_sprite.current_frame + 1).min(animated_sprite.frames.len() - 1);
            }
            
            // Update the sprite's texture atlas index
            if let Some(&frame_index) = animated_sprite.frames.get(animated_sprite.current_frame) {
                atlas.index = frame_index;
            }
        }
    }
}

// Helper functions

/// Generate a procedural creature atlas as fallback when assets are missing
/// Creates a simple 8x8 grid of colored rectangles to represent different animation frames
fn generate_procedural_creature_atlas() -> Image {
    let atlas_size = (ATLAS_GRID_SIZE * CREATURE_SPRITE_SIZE as usize) as usize;
    let mut data = vec![0u8; atlas_size * atlas_size * 4]; // RGBA
    
    // Generate different colored rectangles for each animation type
    for row in 0..ATLAS_GRID_SIZE {
        for col in 0..ATLAS_GRID_SIZE {
            let x_start = (col * CREATURE_SPRITE_SIZE as usize) as usize;
            let y_start = (row * CREATURE_SPRITE_SIZE as usize) as usize;
            
            // Choose color based on animation type (row)
            let (r, g, b) = match row {
                0 => (100, 200, 100), // Idle - green
                1 => (100, 150, 200), // Walk - blue
                2 => (200, 150, 100), // Run - orange
                3 => (200, 200, 100), // Eat - yellow
                4 => (150, 150, 200), // Sleep - purple
                5 => (200, 100, 150), // Talk - pink
                6 => (200, 100, 100), // Attack - red
                _ => (150, 150, 150), // Other - gray
            };
            
            // Fill the sprite area with color
            for y in 0..CREATURE_SPRITE_SIZE as usize {
                for x in 0..CREATURE_SPRITE_SIZE as usize {
                    let idx = ((y_start + y) * atlas_size + (x_start + x)) * 4;
                    if idx + 3 < data.len() {
                        // Create a simple creature shape (circle in center)
                        let cx = CREATURE_SPRITE_SIZE / 2.0;
                        let cy = CREATURE_SPRITE_SIZE / 2.0;
                        let dx = x as f32 - cx;
                        let dy = y as f32 - cy;
                        let dist = (dx * dx + dy * dy).sqrt();
                        
                        if dist < cx * 0.8 {
                            data[idx] = r;
                            data[idx + 1] = g;
                            data[idx + 2] = b;
                            data[idx + 3] = 255;
                        }
                    }
                }
            }
        }
    }
    
    Image::new(
        Extent3d {
            width: atlas_size as u32,
            height: atlas_size as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
    )
}

/// Generate a procedural terrain atlas as fallback when assets are missing
/// Creates a simple 8x8 grid of isometric tiles in different colors for biomes
fn generate_procedural_terrain_atlas() -> Image {
    let atlas_width = (ATLAS_GRID_SIZE * TILE_WIDTH as usize) as usize;
    let atlas_height = (ATLAS_GRID_SIZE * TILE_HEIGHT as usize) as usize;
    let mut data = vec![0u8; atlas_width * atlas_height * 4]; // RGBA
    
    // Generate different colored tiles for each biome type
    for row in 0..ATLAS_GRID_SIZE {
        for col in 0..ATLAS_GRID_SIZE {
            let x_start = (col * TILE_WIDTH as usize) as usize;
            let y_start = (row * TILE_HEIGHT as usize) as usize;
            
            // Choose color based on biome type (row)
            let (r, g, b) = match row {
                0 => (100, 200, 100), // Grassland - green
                1 => (50, 150, 50),   // Forest - dark green
                2 => (220, 200, 150), // Desert - sandy
                3 => (200, 220, 240), // Tundra - icy blue
                4 => (50, 100, 200),  // Ocean - blue
                _ => (150, 150, 150), // Other - gray
            };
            
            // Fill the tile area with isometric diamond shape
            for y in 0..TILE_HEIGHT as usize {
                for x in 0..TILE_WIDTH as usize {
                    let idx = ((y_start + y) * atlas_width + (x_start + x)) * 4;
                    if idx + 3 < data.len() {
                        // Create isometric diamond shape
                        let cx = TILE_WIDTH / 2.0;
                        let cy = TILE_HEIGHT / 2.0;
                        let dx = (x as f32 - cx).abs();
                        let dy = (y as f32 - cy).abs();
                        
                        // Isometric diamond formula
                        if dx / (TILE_WIDTH / 2.0) + dy / (TILE_HEIGHT / 2.0) < 0.9 {
                            data[idx] = r;
                            data[idx + 1] = g;
                            data[idx + 2] = b;
                            data[idx + 3] = 255;
                        }
                    }
                }
            }
        }
    }
    
    Image::new(
        Extent3d {
            width: atlas_width as u32,
            height: atlas_height as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
    )
}

/// Generate a simple procedural particle texture as fallback
/// Creates basic shapes for different particle types
fn generate_procedural_particle(particle_type: &str) -> Image {
    let size = 16usize; // Standard particle size
    let mut data = vec![0u8; size * size * 4]; // RGBA
    
    // Choose color and shape based on particle type
    match particle_type {
        "heart" => {
            // Red heart shape
            for y in 0..size {
                for x in 0..size {
                    let idx = (y * size + x) * 4;
                    // Simple heart formula
                    let nx = (x as f32 / size as f32 - 0.5) * 2.0;
                    let ny = (y as f32 / size as f32 - 0.5) * 2.0;
                    if (nx * nx + (ny + 0.3) * (ny + 0.3) < 0.3) ||
                       ((nx - 0.25).abs() < 0.2 && ny > -0.3 && ny < 0.1) ||
                       ((nx + 0.25).abs() < 0.2 && ny > -0.3 && ny < 0.1) {
                        data[idx] = 255; // R
                        data[idx + 1] = 100; // G
                        data[idx + 2] = 150; // B
                        data[idx + 3] = 255; // A
                    }
                }
            }
        },
        "zzz" => {
            // Blue Z shape
            for y in 0..size {
                for x in 0..size {
                    let idx = (y * size + x) * 4;
                    // Simple Z pattern
                    if (y < 3 && x > 4 && x < 12) ||
                       (y > size - 3 && x > 4 && x < 12) ||
                       ((x + y) > 14 && (x + y) < 18) {
                        data[idx] = 100; // R
                        data[idx + 1] = 150; // G
                        data[idx + 2] = 255; // B
                        data[idx + 3] = 255; // A
                    }
                }
            }
        },
        "sparkle" => {
            // Yellow sparkle
            let center = size / 2;
            for y in 0..size {
                for x in 0..size {
                    let idx = (y * size + x) * 4;
                    let dx = (x as i32 - center as i32).abs();
                    let dy = (y as i32 - center as i32).abs();
                    if dx + dy < 4 || (dx < 2 && dy < 6) || (dx < 6 && dy < 2) {
                        data[idx] = 255; // R
                        data[idx + 1] = 255; // G
                        data[idx + 2] = 100; // B
                        data[idx + 3] = 255; // A
                    }
                }
            }
        },
        _ => {
            // Default circle for other particles
            let center = size / 2;
            for y in 0..size {
                for x in 0..size {
                    let idx = (y * size + x) * 4;
                    let dx = x as i32 - center as i32;
                    let dy = y as i32 - center as i32;
                    if dx * dx + dy * dy < (center as i32 * center as i32 * 3 / 4) {
                        data[idx] = 200; // R
                        data[idx + 1] = 200; // G
                        data[idx + 2] = 200; // B
                        data[idx + 3] = 255; // A
                    }
                }
            }
        }
    }
    
    Image::new(
        Extent3d {
            width: size as u32,
            height: size as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
    )
}

/// Get the starting frame and frame count for each animation type
/// Based on the sprite atlas layout (8x8 grid = 64 total frames)
/// 
/// Atlas organization:
/// - Row 0 (0-3): Idle animation - gentle breathing/swaying
/// - Row 0 (4-11): Walk cycle - 8 frames for smooth movement
/// - Row 1 (12-17): Run cycle - 6 frames, faster than walk
/// - Row 2 (18-23): Eating animation - chewing/swallowing
/// - Row 3 (24-27): Sleep animation - slow breathing, eyes closed
/// - Row 3-4 (28-35): Talk animation - mouth movements, gestures
/// - Row 4-5 (36-41): Attack animation - aggressive postures
/// - Row 5-6 (42-49): Death animation - collapse sequence
/// - Row 6-7 (50-65): Special animations - emotional expressions
/// 
/// Returns (start_frame, frame_count) tuple
fn get_animation_frames(animation: AnimationState) -> (usize, usize) {
    match animation {
        AnimationState::Idle => (0, 4),      // Frames 0-3: breathing cycle
        AnimationState::Walk => (4, 8),      // Frames 4-11: full walk cycle
        AnimationState::Run => (12, 6),      // Frames 12-17: running cycle
        AnimationState::Eat => (18, 6),      // Frames 18-23: eating sequence
        AnimationState::Sleep => (24, 4),    // Frames 24-27: sleep breathing
        AnimationState::Talk => (28, 8),     // Frames 28-35: talking gestures
        AnimationState::Attack => (36, 6),   // Frames 36-41: attack sequence
        AnimationState::Death => (42, 8),    // Frames 42-49: death animation
        AnimationState::Special(special) => match special {
            crate::components::SpecialAnimation::Happy => (50, 4),   // Frames 50-53: joy expression
            crate::components::SpecialAnimation::Sad => (54, 4),     // Frames 54-57: sadness
            crate::components::SpecialAnimation::Angry => (58, 4),   // Frames 58-61: anger
            crate::components::SpecialAnimation::Curious => (62, 4), // Frames 62-65: curiosity
        },
    }
}

/// Determine emotion type based on creature's needs and current state
/// Maps AI state to visual emotions following the priority system
fn determine_emotion_from_state(
    needs: &crate::components::Needs,
    state: &crate::components::CreatureState,
) -> EmotionType {
    // Dead creatures show no emotion
    if matches!(state, crate::components::CreatureState::Dead) {
        return EmotionType::Neutral;
    }
    
    // Check critical needs first (high priority emotions)
    let lowest_need = needs.get_lowest();
    
    // Extreme hunger triggers hunger emotion
    if needs.hunger > HUNGER_EMOTION_THRESHOLD {
        return EmotionType::Hungry;
    }
    
    // Extreme tiredness triggers tired emotion
    if needs.energy < TIRED_EMOTION_THRESHOLD {
        return EmotionType::Tired;
    }
    
    // Check for fear-inducing situations (critical needs)
    if lowest_need.1 < CRITICAL_NEED_THRESHOLD {
        return EmotionType::Frightened;
    }
    
    // Anger from unmet needs (low need + low social = frustration)
    if lowest_need.1 < ANGRY_NEED_THRESHOLD && needs.social < ANGRY_NEED_THRESHOLD {
        return EmotionType::Angry;
    }
    
    // Sadness from prolonged low needs
    if lowest_need.1 < SAD_NEED_THRESHOLD {
        return EmotionType::Sad;
    }
    
    // Happy when eating or drinking
    if matches!(state, crate::components::CreatureState::Eating | crate::components::CreatureState::Drinking) {
        return EmotionType::Happy;
    }
    
    // Content when all needs are satisfied
    if lowest_need.1 > CONTENT_NEED_THRESHOLD {
        return EmotionType::Content;
    }
    
    // Curious when exploring (moving with adequate needs)
    if matches!(state, crate::components::CreatureState::Moving { .. }) && lowest_need.1 > CURIOUS_NEED_THRESHOLD {
        return EmotionType::Curious;
    }
    
    // Default to neutral
    EmotionType::Neutral
}