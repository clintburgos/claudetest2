use bevy::prelude::*;
use bevy::text::{Text2dBounds, Text2dBundle};
use crate::components::ConversationState;

/// Enhanced speech bubble plugin for Phase 4
/// 
/// Features:
/// - Dynamic sizing based on content
/// - Smooth animations and transitions
/// - Multiple bubble styles (speech, thought, shout)
/// - Emoji and icon support
/// - Queue system for multiple messages
pub struct EnhancedSpeechBubblePlugin;

impl Plugin for EnhancedSpeechBubblePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SpeechBubbleAssets::default())
            .add_systems(Startup, load_speech_bubble_assets)
            .add_systems(Update, (
                create_speech_bubbles,
                update_speech_bubble_content,
                animate_speech_bubbles,
                update_speech_bubble_positions,
                process_speech_queue,
                cleanup_finished_bubbles,
            ).chain());
    }
}

/// Speech bubble assets
#[derive(Resource, Default)]
pub struct SpeechBubbleAssets {
    pub bubble_texture: Handle<Image>,
    pub thought_texture: Handle<Image>,
    pub shout_texture: Handle<Image>,
    pub font: Handle<Font>,
    pub emoji_atlas: Handle<TextureAtlasLayout>,
}

/// Enhanced speech bubble component
#[derive(Component)]
pub struct SpeechBubble {
    pub owner: Entity,
    pub bubble_type: BubbleType,
    pub offset: Vec3,
    pub target_offset: Vec3,
    pub lifetime: Timer,
    pub animation_state: BubbleAnimationState,
    pub size: Vec2,
    pub target_size: Vec2,
    pub priority: u8,
}

/// Speech bubble types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BubbleType {
    Speech,   // Normal talking
    Thought,  // Thinking (cloud bubble)
    Shout,    // Loud/important (spiky bubble)
    Whisper,  // Quiet (small, faded)
    System,   // Game messages (different style)
}

/// Animation states for smooth transitions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BubbleAnimationState {
    Appearing,
    Idle,
    Disappearing,
    Bouncing,
}

/// Content that can be displayed in speech bubbles
#[derive(Component, Clone)]
pub enum BubbleContent {
    Text(String),
    Emoji(EmojiType),
    Mixed(Vec<ContentPart>),
}

#[derive(Clone)]
pub enum ContentPart {
    Text(String),
    Emoji(EmojiType),
    Icon(String),
}

/// Emoji types for visual communication
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EmojiType {
    Happy,
    Sad,
    Angry,
    Love,
    Question,
    Exclamation,
    Food,
    Sleep,
    Music,
    Star,
    Warning,
    Check,
}

/// Speech queue for managing multiple messages
#[derive(Component)]
pub struct SpeechQueue {
    pub messages: Vec<QueuedMessage>,
    pub current_index: usize,
    pub message_duration: f32,
}

#[derive(Clone)]
pub struct QueuedMessage {
    pub content: BubbleContent,
    pub bubble_type: BubbleType,
    pub duration: f32,
}

/// System to load speech bubble assets
fn load_speech_bubble_assets(
    mut assets: ResMut<SpeechBubbleAssets>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    assets.bubble_texture = asset_server.load("sprites/ui/speech_bubble.png");
    assets.thought_texture = asset_server.load("sprites/ui/thought_bubble.png");
    assets.shout_texture = asset_server.load("sprites/ui/shout_bubble.png");
    assets.font = asset_server.load("fonts/FiraMono-Medium.ttf");
    
    // Create emoji atlas
    let _texture_handle: Handle<Image> = asset_server.load("sprites/ui/emoji_atlas.png");
    let texture_atlas = TextureAtlasLayout::from_grid(
        Vec2::new(32.0, 32.0),
        4,
        3,
        None,
        None,
    );
    assets.emoji_atlas = texture_atlases.add(texture_atlas);
}

/// System to create speech bubbles with dynamic sizing
fn create_speech_bubbles(
    mut commands: Commands,
    assets: Res<SpeechBubbleAssets>,
    creatures_in_conversation: Query<
        (Entity, &Transform, &ConversationState),
        (Changed<ConversationState>, Without<SpeechBubble>)
    >,
    existing_bubbles: Query<&SpeechBubble>,
) {
    for (entity, transform, conversation_state) in creatures_in_conversation.iter() {
        // Skip if already has a bubble
        if existing_bubbles.iter().any(|b| b.owner == entity) {
            continue;
        }
        
        // Determine bubble content and type
        let (content, bubble_type) = match conversation_state {
            ConversationState::Greeting => (
                BubbleContent::Mixed(vec![
                    ContentPart::Emoji(EmojiType::Happy),
                    ContentPart::Text("Hello!".to_string()),
                ]),
                BubbleType::Speech,
            ),
            ConversationState::ShareInfo(_) => (
                BubbleContent::Mixed(vec![
                    ContentPart::Emoji(EmojiType::Question),
                    ContentPart::Text("Did you know?".to_string()),
                ]),
                BubbleType::Speech,
            ),
            ConversationState::RequestHelp => (
                BubbleContent::Emoji(EmojiType::Warning),
                BubbleType::Shout,
            ),
            ConversationState::OfferHelp => (
                BubbleContent::Mixed(vec![
                    ContentPart::Text("I can help!".to_string()),
                    ContentPart::Emoji(EmojiType::Love),
                ]),
                BubbleType::Speech,
            ),
            _ => (
                BubbleContent::Emoji(EmojiType::Question),
                BubbleType::Thought,
            ),
        };
        
        // Calculate bubble size based on content
        let bubble_size = calculate_bubble_size(&content);
        let bubble_offset = Vec3::new(0.0, 50.0, 10.0);
        
        // Create bubble entity
        let bubble_entity = commands.spawn((
            SpriteBundle {
                texture: match bubble_type {
                    BubbleType::Speech => assets.bubble_texture.clone(),
                    BubbleType::Thought => assets.thought_texture.clone(),
                    BubbleType::Shout => assets.shout_texture.clone(),
                    _ => assets.bubble_texture.clone(),
                },
                transform: Transform::from_translation(transform.translation + bubble_offset)
                    .with_scale(Vec3::ZERO), // Start at zero for animation
                sprite: Sprite {
                    custom_size: Some(bubble_size),
                    ..default()
                },
                ..default()
            },
            SpeechBubble {
                owner: entity,
                bubble_type,
                offset: bubble_offset,
                target_offset: bubble_offset,
                lifetime: Timer::from_seconds(4.0, TimerMode::Once),
                animation_state: BubbleAnimationState::Appearing,
                size: Vec2::ZERO,
                target_size: bubble_size,
                priority: 1,
            },
            content,
            Name::new("SpeechBubble"),
        )).id();
        
        // Add tail sprite as child
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::WHITE,
                    custom_size: Some(Vec2::new(15.0, 15.0)),
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(0.0, -bubble_size.y / 2.0 - 5.0, -0.1))
                    .with_rotation(Quat::from_rotation_z(std::f32::consts::PI / 4.0)),
                ..default()
            },
            Name::new("BubbleTail"),
        )).set_parent(bubble_entity);
    }
}

/// System to update speech bubble content rendering
fn update_speech_bubble_content(
    mut commands: Commands,
    assets: Res<SpeechBubbleAssets>,
    asset_server: Res<AssetServer>,
    bubbles: Query<(Entity, &BubbleContent, &SpeechBubble), Added<BubbleContent>>,
) {
    for (bubble_entity, content, bubble) in bubbles.iter() {
        match content {
            BubbleContent::Text(text) => {
                commands.spawn((
                    Text2dBundle {
                        text: Text::from_section(
                            text,
                            TextStyle {
                                font: assets.font.clone(),
                                font_size: 16.0,
                                color: Color::BLACK,
                            },
                        ),
                        text_2d_bounds: Text2dBounds {
                            size: bubble.target_size * 0.8, // Leave padding
                        },
                        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
                        ..default()
                    },
                )).set_parent(bubble_entity);
            }
            BubbleContent::Emoji(emoji) => {
                let index = emoji_to_atlas_index(*emoji);
                commands.spawn((
                    SpriteBundle {
                        texture: asset_server.load("sprites/ui/emoji_atlas.png"),
                        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0))
                            .with_scale(Vec3::splat(1.5)),
                        ..default()
                    },
                    TextureAtlas {
                        layout: assets.emoji_atlas.clone(),
                        index,
                    },
                )).set_parent(bubble_entity);
            }
            BubbleContent::Mixed(parts) => {
                let mut x_offset = -bubble.target_size.x / 2.0 + 20.0;
                
                for part in parts {
                    match part {
                        ContentPart::Text(text) => {
                            let text_width = text.len() as f32 * 8.0; // Approximate
                            commands.spawn((
                                Text2dBundle {
                                    text: Text::from_section(
                                        text,
                                        TextStyle {
                                            font: assets.font.clone(),
                                            font_size: 16.0,
                                            color: Color::BLACK,
                                        },
                                    ),
                                    transform: Transform::from_translation(Vec3::new(x_offset + text_width / 2.0, 0.0, 1.0)),
                                    ..default()
                                },
                            )).set_parent(bubble_entity);
                            x_offset += text_width + 5.0;
                        }
                        ContentPart::Emoji(emoji) => {
                            let index = emoji_to_atlas_index(*emoji);
                            commands.spawn((
                                SpriteBundle {
                                    texture: asset_server.load("sprites/ui/emoji_atlas.png"),
                                    transform: Transform::from_translation(Vec3::new(x_offset + 16.0, 0.0, 1.0)),
                                    ..default()
                                },
                                TextureAtlas {
                                    layout: assets.emoji_atlas.clone(),
                                    index,
                                },
                            )).set_parent(bubble_entity);
                            x_offset += 36.0;
                        }
                        ContentPart::Icon(_) => {
                            // TODO: Implement icon rendering
                            x_offset += 32.0;
                        }
                    }
                }
            }
        }
    }
}

/// System to animate speech bubbles
fn animate_speech_bubbles(
    time: Res<Time>,
    mut bubbles: Query<(&mut Transform, &mut SpeechBubble, &mut Sprite)>,
) {
    let dt = time.delta_seconds();
    
    for (mut transform, mut bubble, mut sprite) in bubbles.iter_mut() {
        match bubble.animation_state {
            BubbleAnimationState::Appearing => {
                // Scale up animation
                let target_scale = Vec3::ONE;
                transform.scale = transform.scale.lerp(target_scale, dt * 8.0);
                
                if transform.scale.distance(target_scale) < 0.01 {
                    bubble.animation_state = BubbleAnimationState::Idle;
                    transform.scale = target_scale;
                }
                
                // Elastic overshoot
                if transform.scale.x > 0.9 {
                    let overshoot = ((time.elapsed_seconds() * 10.0).sin() * 0.05 + 1.0).min(1.1);
                    transform.scale = Vec3::splat(overshoot);
                }
            }
            BubbleAnimationState::Idle => {
                // Gentle floating animation
                let float_offset = (time.elapsed_seconds() * 2.0).sin() * 2.0;
                bubble.offset.y = bubble.target_offset.y + float_offset;
                
                // Check if should start disappearing
                if bubble.lifetime.fraction_remaining() < 0.2 {
                    bubble.animation_state = BubbleAnimationState::Disappearing;
                }
            }
            BubbleAnimationState::Disappearing => {
                // Scale down and fade
                transform.scale = transform.scale.lerp(Vec3::ZERO, dt * 5.0);
                let alpha = bubble.lifetime.fraction_remaining() * 5.0;
                sprite.color.set_a(alpha);
            }
            BubbleAnimationState::Bouncing => {
                // Bounce animation for emphasis
                let bounce = ((time.elapsed_seconds() * 15.0).sin() * 0.1 + 1.0).abs();
                transform.scale = Vec3::splat(bounce);
            }
        }
        
        // Update size
        bubble.size = bubble.size.lerp(bubble.target_size, dt * 5.0);
        sprite.custom_size = Some(bubble.size);
    }
}

/// System to update positions
fn update_speech_bubble_positions(
    time: Res<Time>,
    mut bubbles: Query<(&mut Transform, &mut SpeechBubble)>,
    owners: Query<&Transform, Without<SpeechBubble>>,
) {
    for (mut bubble_transform, mut bubble) in bubbles.iter_mut() {
        bubble.lifetime.tick(time.delta());
        
        if let Ok(owner_transform) = owners.get(bubble.owner) {
            let target_pos = owner_transform.translation + bubble.offset;
            bubble_transform.translation = bubble_transform.translation.lerp(target_pos, 0.2);
        }
    }
}

/// System to process speech queues
fn process_speech_queue(
    time: Res<Time>,
    mut commands: Commands,
    mut queues: Query<(Entity, &mut SpeechQueue)>,
) {
    for (entity, mut queue) in queues.iter_mut() {
        if queue.current_index >= queue.messages.len() {
            continue;
        }
        
        // Update message timer
        queue.message_duration -= time.delta_seconds();
        
        if queue.message_duration <= 0.0 {
            queue.current_index += 1;
            
            if queue.current_index < queue.messages.len() {
                let message = &queue.messages[queue.current_index];
                queue.message_duration = message.duration;
                
                // Spawn new bubble for next message
                // TODO: Update existing bubble instead of creating new one
            } else {
                // Queue finished
                commands.entity(entity).remove::<SpeechQueue>();
            }
        }
    }
}

/// System to cleanup finished bubbles
fn cleanup_finished_bubbles(
    mut commands: Commands,
    bubbles: Query<(Entity, &SpeechBubble)>,
) {
    for (entity, bubble) in bubbles.iter() {
        if bubble.lifetime.finished() && bubble.animation_state == BubbleAnimationState::Disappearing {
            commands.entity(entity).despawn_recursive();
        }
    }
}

// Helper functions

fn calculate_bubble_size(content: &BubbleContent) -> Vec2 {
    match content {
        BubbleContent::Text(text) => {
            let char_count = text.len() as f32;
            let width = (char_count * 8.0 + 40.0).min(200.0);
            let lines = (char_count * 8.0 / 160.0).ceil();
            let height = lines * 20.0 + 30.0;
            Vec2::new(width, height)
        }
        BubbleContent::Emoji(_) => Vec2::new(60.0, 60.0),
        BubbleContent::Mixed(parts) => {
            let mut width = 0.0;
            for part in parts {
                match part {
                    ContentPart::Text(text) => width += text.len() as f32 * 8.0 + 5.0,
                    ContentPart::Emoji(_) | ContentPart::Icon(_) => width += 36.0,
                }
            }
            Vec2::new((width + 40.0).min(250.0), 60.0)
        }
    }
}

fn emoji_to_atlas_index(emoji: EmojiType) -> usize {
    match emoji {
        EmojiType::Happy => 0,
        EmojiType::Sad => 1,
        EmojiType::Angry => 2,
        EmojiType::Love => 3,
        EmojiType::Question => 4,
        EmojiType::Exclamation => 5,
        EmojiType::Food => 6,
        EmojiType::Sleep => 7,
        EmojiType::Music => 8,
        EmojiType::Star => 9,
        EmojiType::Warning => 10,
        EmojiType::Check => 11,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bubble_size_calculation() {
        let text_content = BubbleContent::Text("Hello!".to_string());
        let size = calculate_bubble_size(&text_content);
        assert!(size.x > 0.0 && size.y > 0.0);
        
        let emoji_content = BubbleContent::Emoji(EmojiType::Happy);
        let emoji_size = calculate_bubble_size(&emoji_content);
        assert_eq!(emoji_size, Vec2::new(60.0, 60.0));
    }
    
    #[test]
    fn test_emoji_atlas_index() {
        assert_eq!(emoji_to_atlas_index(EmojiType::Happy), 0);
        assert_eq!(emoji_to_atlas_index(EmojiType::Check), 11);
    }
}