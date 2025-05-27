use bevy::prelude::*;
use crate::components::{
    CartoonSprite, AnimationState, EmotionType, ExpressionOverlay, AnimatedSprite
};
use std::collections::HashMap;

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
            .add_systems(Startup, setup_cartoon_rendering)
            .add_systems(
                Update,
                (
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
fn setup_cartoon_rendering(
    mut _commands: Commands,
    asset_server: Res<AssetServer>,
    mut cartoon_assets: ResMut<CartoonAssets>,
    mut expression_system: ResMut<ExpressionSystem>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    mut particle_assets: ResMut<crate::rendering::particles::ParticleAssets>,
) {
    // Load texture atlases
    cartoon_assets.creature_atlas = asset_server.load("sprites/creatures/creature_atlas.png");
    cartoon_assets.terrain_atlas = asset_server.load("sprites/terrain/terrain_atlas.png");
    
    // Create texture atlas layouts
    // Creature atlas: 8x8 grid of 48x48 sprites
    // Layout supports 64 unique animation frames:
    // - Rows 0-1: Idle animations (8 frames)
    // - Rows 2-3: Movement animations (16 frames)
    // - Rows 4-5: Action animations (16 frames)
    // - Rows 6-7: Special/emotion animations (16 frames)
    let creature_layout = TextureAtlasLayout::from_grid(
        Vec2::new(48.0, 48.0), // Individual sprite size in pixels
        8, // Columns in atlas
        8, // Rows in atlas
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
        Vec2::new(64.0, 32.0), // Isometric tile dimensions (2:1 ratio)
        8, // Columns in atlas
        8, // Rows in atlas
        None, // No padding between tiles
        None, // No offset from atlas origin
    );
    cartoon_assets.terrain_atlas_layout = texture_atlases.add(terrain_layout);
    
    // Load particle textures for both systems
    let particle_names = ["heart", "zzz", "sparkle", "sweat", "exclamation", "question"];
    for name in particle_names {
        let handle = asset_server.load(format!("sprites/particles/{}.png", name));
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

/// System to create sprite components for creatures that don't have them yet
/// Uses the creature atlas to render animated sprites
fn update_cartoon_sprites(
    mut commands: Commands,
    cartoon_assets: Res<CartoonAssets>,
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
    
    for (entity, creature_type, genetics) in creatures_without_sprites.iter() {
        // Create cartoon sprite component with genetic variations
        let mut cartoon_sprite = CartoonSprite::default();
        
        // Apply genetic variations to body modifiers
        // Size gene ranges from 0.0 to 1.0, mapped to 0.7x-1.3x scale
        // This provides 60% size variation while keeping creatures recognizable
        cartoon_sprite.body_modifiers.size_scale = 0.7 + (genetics.size * 0.6);
        
        // Color tint based on creature type with genetic variation
        let base_color = match creature_type {
            crate::components::CreatureType::Herbivore => Color::rgb(0.7, 1.0, 0.7),
            crate::components::CreatureType::Carnivore => Color::rgb(1.0, 0.7, 0.7),
            crate::components::CreatureType::Omnivore => Color::rgb(0.9, 0.8, 0.7),
        };
        
        // Apply genetic color variation (slight hue shift)
        // Color gene (0.0-1.0) centered at 0.5, creates -0.1 to +0.1 hue shift
        // Keeps creatures recognizable while adding visual variety
        let hue_shift = (genetics.color - 0.5) * 0.2;
        cartoon_sprite.body_modifiers.color_tint = Color::rgb(
            (base_color.r() + hue_shift).clamp(0.0, 1.0),
            base_color.g(),
            (base_color.b() - hue_shift).clamp(0.0, 1.0),
        );
        
        // Determine pattern type based on genetics
        cartoon_sprite.body_modifiers.pattern_type = if genetics.pattern > 0.7 {
            crate::components::PatternType::Stripes
        } else if genetics.pattern > 0.4 {
            crate::components::PatternType::Spots
        } else {
            crate::components::PatternType::None
        };
        
        // Create animated sprite component for idle animation
        let idle_frames = (0..4).collect(); // First 4 frames are idle animation
        let animated_sprite = AnimatedSprite::new(idle_frames, 0.2, true);
        
        // Add sprite bundle with texture atlas
        commands.entity(entity).insert((
            SpriteBundle {
                sprite: Sprite {
                    color: cartoon_sprite.body_modifiers.color_tint,
                    custom_size: Some(Vec2::new(
                        48.0 * cartoon_sprite.body_modifiers.size_scale,
                        48.0 * cartoon_sprite.body_modifiers.size_scale
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
    }
}

/// System to update creature animations based on their current state
/// Changes the animation frames when creature behavior changes
fn update_creature_animations(
    _time: Res<Time>,
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
        // Determine animation state based on creature state
        let new_animation = match state {
            crate::components::CreatureState::Idle => {
                // Check if talking
                if conversation.is_some() {
                    AnimationState::Talk
                } else {
                    AnimationState::Idle
                }
            },
            crate::components::CreatureState::Moving { .. } => {
                if velocity.0.length() > 2.0 {
                    AnimationState::Run
                } else {
                    AnimationState::Walk
                }
            }
            crate::components::CreatureState::Eating => AnimationState::Eat,
            crate::components::CreatureState::Drinking => AnimationState::Eat, // Reuse eat animation
            crate::components::CreatureState::Resting => AnimationState::Sleep,
            crate::components::CreatureState::Dead => AnimationState::Death,
        };
        
        // Update animation state if changed
        if cartoon_sprite.base_animation != new_animation {
            cartoon_sprite.base_animation = new_animation;
            
            // Update animated sprite frames based on new animation
            let (start_frame, frame_count) = get_animation_frames(new_animation);
            animated_sprite.frames = (start_frame..start_frame + frame_count).collect();
            animated_sprite.current_frame = 0;
            
            // Adjust animation speed based on state
            // Frame times in seconds - lower values = faster animation
            let frame_time = match new_animation {
                AnimationState::Idle => 0.3,      // Slow, relaxed breathing
                AnimationState::Walk => 0.15,     // Normal walking pace
                AnimationState::Run => 0.1,       // Fast movement
                AnimationState::Eat => 0.2,       // Moderate chewing speed
                AnimationState::Sleep => 0.5,     // Very slow breathing
                AnimationState::Talk => 0.2,      // Moderate speech animation
                AnimationState::Attack => 0.1,    // Fast, aggressive movement
                AnimationState::Death => 0.3,     // Slow fade/collapse
                AnimationState::Special(_) => 0.25, // Moderate emotion display
            };
            animated_sprite.timer = Timer::from_seconds(frame_time, TimerMode::Repeating);
            
            // Update looping based on animation type
            animated_sprite.looping = !matches!(new_animation, AnimationState::Death);
        }
        
        // Apply any color modifiers based on state
        sprite.color = match cartoon_sprite.base_animation {
            AnimationState::Sleep => cartoon_sprite.body_modifiers.color_tint * 0.7,
            AnimationState::Death => cartoon_sprite.body_modifiers.color_tint * 0.3,
            _ => cartoon_sprite.body_modifiers.color_tint,
        };
    }
}

fn update_expression_overlays(
    mut query: Query<(
        &mut CartoonSprite,
        &crate::components::Needs,
        &crate::components::CreatureState,
    )>,
) {
    for (mut cartoon_sprite, needs, state) in query.iter_mut() {
        // Determine emotion based on needs and state
        let emotion = determine_emotion_from_state(needs, state);
        
        // Update expression overlay
        if let Some(ref mut overlay) = cartoon_sprite.expression_overlay {
            // Update expression parameters based on emotion
            match emotion {
                EmotionType::Happy => {
                    overlay.mouth_curve = 0.5;
                    overlay.eye_scale = 1.1;
                    overlay.brow_angle = -10.0;
                }
                EmotionType::Sad => {
                    overlay.mouth_curve = -0.5;
                    overlay.eye_scale = 0.9;
                    overlay.brow_angle = 10.0;
                }
                EmotionType::Angry => {
                    overlay.mouth_curve = -0.3;
                    overlay.eye_scale = 0.8;
                    overlay.brow_angle = -20.0;
                }
                _ => {
                    overlay.mouth_curve = 0.0;
                    overlay.eye_scale = 1.0;
                    overlay.brow_angle = 0.0;
                }
            }
        } else {
            // Create new expression overlay
            cartoon_sprite.expression_overlay = Some(ExpressionOverlay {
                eye_offset: Vec2::ZERO,
                eye_scale: 1.0,
                mouth_curve: 0.0,
                mouth_open: 0.0,
                brow_angle: 0.0,
            });
        }
    }
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
) {
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
    
    // Extreme hunger (>80% hungry triggers hunger emotion)
    if needs.hunger > 0.8 {
        return EmotionType::Hungry;
    }
    
    // Extreme tiredness (<20% energy triggers tired emotion)
    if needs.energy < 0.2 {
        return EmotionType::Tired;
    }
    
    // Check for fear-inducing situations (any need below 10%)
    if lowest_need.1 < 0.1 {
        return EmotionType::Frightened;
    }
    
    // Anger from unmet needs (low need + low social = frustration)
    if lowest_need.1 < 0.3 && needs.social < 0.3 {
        return EmotionType::Angry;
    }
    
    // Sadness from prolonged low needs (any need below 40%)
    if lowest_need.1 < 0.4 {
        return EmotionType::Sad;
    }
    
    // Happy when eating or drinking
    if matches!(state, crate::components::CreatureState::Eating | crate::components::CreatureState::Drinking) {
        return EmotionType::Happy;
    }
    
    // Content when all needs are satisfied (all needs above 70%)
    if lowest_need.1 > 0.7 {
        return EmotionType::Content;
    }
    
    // Curious when exploring (moving with needs above 50% = not urgent)
    if matches!(state, crate::components::CreatureState::Moving { .. }) && lowest_need.1 > 0.5 {
        return EmotionType::Curious;
    }
    
    // Default to neutral
    EmotionType::Neutral
}