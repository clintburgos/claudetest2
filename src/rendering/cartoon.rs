use bevy::prelude::*;
use crate::components::{
    CartoonSprite, AnimationState, EmotionType, ExpressionOverlay
};
use std::collections::HashMap;

/// Main plugin for cartoon-style isometric rendering
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
                    update_expression_overlays,
                    render_biome_tiles,
                )
                    .chain(),
            );
    }
}

/// Resource containing loaded cartoon assets
#[derive(Resource, Default)]
pub struct CartoonAssets {
    pub creature_atlas: Handle<Image>,
    pub terrain_atlas: Handle<Image>,
    pub particle_textures: HashMap<String, Handle<Image>>,
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

fn setup_cartoon_rendering(
    mut _commands: Commands,
    asset_server: Res<AssetServer>,
    mut cartoon_assets: ResMut<CartoonAssets>,
    mut expression_system: ResMut<ExpressionSystem>,
) {
    // Load texture atlases
    cartoon_assets.creature_atlas = asset_server.load("sprites/creatures/creature_atlas.png");
    cartoon_assets.terrain_atlas = asset_server.load("sprites/terrain/terrain_atlas.png");
    
    // Load particle textures
    let particle_names = ["heart", "zzz", "sparkle", "sweat", "exclamation", "question"];
    for name in particle_names {
        cartoon_assets.particle_textures.insert(
            name.to_string(),
            asset_server.load(format!("sprites/particles/{}.png", name)),
        );
    }
    
    // Set up expression priorities
    expression_system.emotion_priorities.insert(EmotionType::Angry, 0.9);
    expression_system.emotion_priorities.insert(EmotionType::Frightened, 0.85);
    expression_system.emotion_priorities.insert(EmotionType::Sad, 0.7);
    expression_system.emotion_priorities.insert(EmotionType::Hungry, 0.6);
    expression_system.emotion_priorities.insert(EmotionType::Tired, 0.5);
    expression_system.emotion_priorities.insert(EmotionType::Happy, 0.4);
    expression_system.emotion_priorities.insert(EmotionType::Curious, 0.3);
    expression_system.emotion_priorities.insert(EmotionType::Content, 0.2);
    expression_system.emotion_priorities.insert(EmotionType::Neutral, 0.1);
    
    // Set up blend durations for smooth transitions
    expression_system.blend_durations.insert((EmotionType::Neutral, EmotionType::Happy), 0.3);
    expression_system.blend_durations.insert((EmotionType::Happy, EmotionType::Sad), 0.5);
    expression_system.blend_durations.insert((EmotionType::Neutral, EmotionType::Angry), 0.2);
}

fn update_cartoon_sprites(
    mut commands: Commands,
    _cartoon_assets: Res<CartoonAssets>,
    creatures_without_sprites: Query<
        (Entity, &crate::components::CreatureType),
        (
            With<crate::components::Creature>,
            Without<CartoonSprite>,
            Without<Sprite>,
        ),
    >,
) {
    for (entity, creature_type) in creatures_without_sprites.iter() {
        // Create cartoon sprite component
        let mut cartoon_sprite = CartoonSprite::default();
        
        // Set color tint based on creature type
        cartoon_sprite.body_modifiers.color_tint = match creature_type {
            crate::components::CreatureType::Herbivore => Color::rgb(0.7, 1.0, 0.7),
            crate::components::CreatureType::Carnivore => Color::rgb(1.0, 0.7, 0.7),
            crate::components::CreatureType::Omnivore => Color::rgb(0.9, 0.8, 0.7),
        };
        
        // Add sprite bundle (for now, use regular sprite)
        // TODO: Switch to texture atlas when sprites are ready
        commands.entity(entity).insert((
            SpriteBundle {
                sprite: Sprite {
                    color: cartoon_sprite.body_modifiers.color_tint,
                    custom_size: Some(Vec2::new(48.0, 48.0)),
                    ..default()
                },
                transform: Transform::from_scale(Vec3::splat(1.0)),
                ..default()
            },
            cartoon_sprite,
        ));
    }
}

fn update_creature_animations(
    _time: Res<Time>,
    mut query: Query<(
        &mut CartoonSprite,
        &mut Sprite,
        &crate::components::CreatureState,
        &crate::components::Velocity,
    )>,
) {
    for (mut cartoon_sprite, mut sprite, state, velocity) in query.iter_mut() {
        // Determine animation state based on creature state
        let new_animation = match state {
            crate::components::CreatureState::Idle => AnimationState::Idle,
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
        }
        
        // Placeholder: vary sprite color based on animation
        // This will be replaced with actual sprite atlas frames
        sprite.color = match cartoon_sprite.base_animation {
            AnimationState::Idle => cartoon_sprite.body_modifiers.color_tint,
            AnimationState::Walk => cartoon_sprite.body_modifiers.color_tint * 0.9,
            AnimationState::Run => cartoon_sprite.body_modifiers.color_tint * 0.8,
            AnimationState::Eat => cartoon_sprite.body_modifiers.color_tint * 1.1,
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

// Helper functions

#[allow(dead_code)]
fn get_animation_start_frame(animation: AnimationState) -> usize {
    match animation {
        AnimationState::Idle => 0,
        AnimationState::Walk => 4,
        AnimationState::Run => 12,
        AnimationState::Eat => 18,
        AnimationState::Sleep => 24,
        AnimationState::Talk => 28,
        AnimationState::Attack => 36,
        AnimationState::Death => 42,
        AnimationState::Special(special) => match special {
            crate::components::SpecialAnimation::Happy => 50,
            crate::components::SpecialAnimation::Sad => 54,
            crate::components::SpecialAnimation::Angry => 58,
            crate::components::SpecialAnimation::Curious => 62,
        },
    }
}

#[allow(dead_code)]
fn get_animation_frame_count(animation: AnimationState) -> usize {
    match animation {
        AnimationState::Idle => 4,
        AnimationState::Walk => 8,
        AnimationState::Run => 6,
        AnimationState::Eat => 6,
        AnimationState::Sleep => 4,
        AnimationState::Talk => 8,
        AnimationState::Attack => 6,
        AnimationState::Death => 8,
        AnimationState::Special(_) => 4,
    }
}

fn determine_emotion_from_state(
    needs: &crate::components::Needs,
    state: &crate::components::CreatureState,
) -> EmotionType {
    // Determine emotion based on needs and state
    if matches!(state, crate::components::CreatureState::Dead) {
        return EmotionType::Neutral;
    }
    
    // Check critical needs first
    if needs.get_lowest().1 < 0.2 {
        if needs.hunger > 0.8 {
            return EmotionType::Hungry;
        } else if needs.energy < 0.2 {
            return EmotionType::Tired;
        }
    }
    
    // Check if content
    if needs.get_lowest().1 > 0.7 {
        return EmotionType::Content;
    }
    
    // Default to neutral
    EmotionType::Neutral
}