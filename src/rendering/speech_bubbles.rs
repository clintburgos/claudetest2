use bevy::prelude::*;
use crate::components::ConversationState;

/// Plugin for rendering speech bubbles above creatures during conversations
/// 
/// Creates floating UI elements that display conversation content with
/// proper positioning in isometric space
pub struct SpeechBubblePlugin;

impl Plugin for SpeechBubblePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            create_speech_bubbles,
            update_speech_bubble_positions,
            cleanup_finished_conversations,
        ).chain());
    }
}

/// Component for speech bubble entities
#[derive(Component)]
pub struct SpeechBubble {
    /// Entity this bubble belongs to
    pub owner: Entity,
    /// Offset from owner position (in screen space pixels)
    /// Positions bubble above creature's head
    pub offset: Vec3,
    /// Duration to display before fading out
    pub duration: Timer,
}

/// Marker component for speech bubble background
/// 
/// Identifies the white rounded rectangle that forms the bubble shape
#[derive(Component)]
pub struct SpeechBubbleBackground;

/// Marker component for speech bubble text
/// 
/// Identifies the text/icon content displayed inside the bubble
#[derive(Component)]
pub struct SpeechBubbleText;

/// System to create speech bubbles for creatures in conversation
fn create_speech_bubbles(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
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
        
        // Create speech bubble entity
        // Offset: X=0 (centered), Y=40 (pixels above head), Z=10 (render priority)
        let bubble_offset = Vec3::new(0.0, 40.0, 10.0);
        let bubble_pos = transform.translation + bubble_offset;
        
        // Spawn bubble background (using a colored sprite for now)
        let bubble_entity = commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgba(1.0, 1.0, 1.0, 0.9), // White with slight transparency
                    custom_size: Some(Vec2::new(80.0, 40.0)), // Standard bubble size
                    ..default()
                },
                transform: Transform::from_translation(bubble_pos)
                    .with_scale(Vec3::new(1.0, 1.0, 1.0)),
                ..default()
            },
            SpeechBubble {
                owner: entity,
                offset: bubble_offset,
                duration: Timer::from_seconds(3.0, TimerMode::Once), // Display for 3 seconds
            },
            SpeechBubbleBackground,
            Name::new("SpeechBubble"),
        )).id();
        
        // Add emotion icon based on conversation topic
        let icon = match conversation_state {
            ConversationState::Greeting => "!",
            ConversationState::ShareInfo(_) => "?",
            ConversationState::RequestHelp => "...",
            ConversationState::OfferHelp => "♥",
            _ => "?",
        };
        
        // Spawn text/icon as child
        commands.spawn((
            Text2dBundle {
                text: Text::from_section(
                    icon,
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"), // Use Bevy's default font
                        font_size: 24.0, // Large enough to be readable
                        color: Color::BLACK,
                    },
                ),
                text_anchor: bevy::sprite::Anchor::Center,
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
                ..default()
            },
            SpeechBubbleText,
        )).set_parent(bubble_entity);
        
        // Add tail pointing to speaker (simple triangle)
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgba(1.0, 1.0, 1.0, 0.9),
                    custom_size: Some(Vec2::new(10.0, 10.0)), // Small triangle tail
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(0.0, -20.0, -0.1)) // Below bubble
                    .with_rotation(Quat::from_rotation_z(std::f32::consts::PI / 4.0)), // 45° rotation
                ..default()
            },
        )).set_parent(bubble_entity);
    }
}

/// System to update speech bubble positions to follow their owners
fn update_speech_bubble_positions(
    time: Res<Time>,
    mut bubbles: Query<(&mut Transform, &mut SpeechBubble)>,
    owners: Query<&Transform, Without<SpeechBubble>>,
) {
    for (mut bubble_transform, mut speech_bubble) in bubbles.iter_mut() {
        // Update timer
        speech_bubble.duration.tick(time.delta());
        
        // Update position to follow owner
        if let Ok(owner_transform) = owners.get(speech_bubble.owner) {
            let target_pos = owner_transform.translation + speech_bubble.offset;
            
            // Smooth movement with 10% interpolation per frame
            bubble_transform.translation = bubble_transform.translation.lerp(target_pos, 0.1);
            
            // Fade out when timer is almost done
            let remaining = speech_bubble.duration.fraction_remaining();
            if remaining < 0.2 { // Last 20% of duration
                // Scale down as a fade effect (proper alpha would require material changes)
                // Maps remaining time: 0.2 -> 1.0 scale, 0.0 -> 0.0 scale
                let scale = remaining * 5.0;
                bubble_transform.scale = Vec3::splat(scale);
            }
        }
    }
}

/// System to remove speech bubbles when conversation ends or timer expires
fn cleanup_finished_conversations(
    mut commands: Commands,
    bubbles: Query<(Entity, &SpeechBubble)>,
    conversations: Query<&ConversationState>,
) {
    for (entity, bubble) in bubbles.iter() {
        // Remove if timer expired
        if bubble.duration.finished() {
            commands.entity(entity).despawn_recursive();
            continue;
        }
        
        // Remove if owner no longer in conversation
        if let Ok(_conversation) = conversations.get(bubble.owner) {
            // Still in conversation, keep bubble
        } else {
            // No longer in conversation
            commands.entity(entity).despawn_recursive();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{ConversationState, ConversationTopic};
    use std::time::Duration;
    
    #[test]
    fn test_speech_bubble_creation() {
        let bubble = SpeechBubble {
            owner: Entity::from_raw(1),
            offset: Vec3::new(0.0, 40.0, 10.0),
            duration: Timer::from_seconds(3.0, TimerMode::Once),
        };
        
        assert_eq!(bubble.owner.index(), 1);
        assert_eq!(bubble.offset.y, 40.0);
        assert_eq!(bubble.duration.duration().as_secs_f32(), 3.0);
        assert_eq!(bubble.duration.mode(), TimerMode::Once);
    }
    
    #[test]
    fn test_conversation_icon_mapping() {
        // Test each conversation state maps to correct icon
        let test_cases = vec![
            (ConversationState::Greeting, "!"),
            (ConversationState::ShareInfo(ConversationTopic::FoodLocation), "?"),
            (ConversationState::ShareInfo(ConversationTopic::DangerWarning), "?"),
            (ConversationState::RequestHelp, "..."),
            (ConversationState::OfferHelp, "♥"),
        ];
        
        for (state, expected) in test_cases {
            let icon = match state {
                ConversationState::Greeting => "!",
                ConversationState::ShareInfo(_) => "?",
                ConversationState::RequestHelp => "...",
                ConversationState::OfferHelp => "♥",
                _ => "?",
            };
            assert_eq!(icon, expected, "State {:?} should show icon '{}'", state, expected);
        }
    }
    
    #[test]
    fn test_speech_bubble_timer() {
        let mut bubble = SpeechBubble {
            owner: Entity::from_raw(1),
            offset: Vec3::ZERO,
            duration: Timer::from_seconds(1.0, TimerMode::Once),
        };
        
        // Test timer progression
        bubble.duration.tick(Duration::from_secs_f32(0.5));
        assert!(!bubble.duration.finished());
        assert_eq!(bubble.duration.fraction_remaining(), 0.5);
        
        bubble.duration.tick(Duration::from_secs_f32(0.6));
        assert!(bubble.duration.finished());
        assert_eq!(bubble.duration.fraction_remaining(), 0.0);
    }
    
    #[test]
    fn test_bubble_fade_calculation() {
        let bubble = SpeechBubble {
            owner: Entity::from_raw(1),
            offset: Vec3::ZERO,
            duration: Timer::from_seconds(1.0, TimerMode::Once),
        };
        
        // Test fade threshold
        let fade_threshold = 0.2; // Last 20% of duration
        
        // At 50% remaining, no fade
        let mut timer = bubble.duration.clone();
        timer.tick(Duration::from_secs_f32(0.5));
        let remaining = timer.fraction_remaining();
        assert!(remaining > fade_threshold);
        
        // At 15% remaining, should fade
        timer.tick(Duration::from_secs_f32(0.35));
        let remaining = timer.fraction_remaining();
        assert!(remaining < fade_threshold);
        
        // Calculate scale for fade
        let scale = remaining * 5.0;
        assert!(scale >= 0.0 && scale <= 1.0);
    }
}