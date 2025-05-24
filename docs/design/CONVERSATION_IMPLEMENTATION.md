# Conversation Implementation Guide

## Quick Start

### Basic Conversation Setup

```rust
// Add conversation capability to creatures
commands.spawn(CreatureBundle {
    // ... other components
    
    conversation: ConversationCapability {
        eloquence: genetics.get_trait(Trait::Eloquence),
        empathy: genetics.get_trait(Trait::Empathy),
        knowledge_retention: 0.8,
        current_conversation: None,
        conversation_cooldown: 0.0,
        style: CommunicationStyle::from_personality(&personality),
    },
    
    memory: ConversationMemory::default(),
    
    social_context: SocialContext {
        relationships: HashMap::new(),
        reputation: 0.5,
        influence_level: 0.5,
        preferred_conversation_size: 2,
        introversion_level: genetics.get_trait(Trait::Introversion),
    },
});
```

### Plugin Configuration

```rust
pub struct ConversationPlugin {
    settings: ConversationSettings,
}

impl Plugin for ConversationPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(self.settings.clone())
            .insert_resource(ConversationPool::new(self.settings.max_simultaneous_conversations))
            .insert_resource(ConversationTemplates::load())
            
            .add_event::<ConversationRequestEvent>()
            .add_event::<ConversationCompleteEvent>()
            .add_event::<InformationSharedEvent>()
            
            .add_systems(
                Update,
                (
                    // Initiation
                    detect_conversation_opportunities,
                    process_conversation_requests,
                    
                    // Processing
                    update_active_conversations,
                    
                    // Outcomes
                    apply_conversation_outcomes,
                    
                    // Visualization
                    render_conversation_bubbles,
                )
                .chain()
                .in_set(ConversationSet),
            );
    }
}
```

## Common Conversation Patterns

### Proximity-Based Conversations

```rust
fn detect_conversation_opportunities(
    creatures: Query<(Entity, &Position, &ConversationCapability, &Needs)>,
    spatial_index: Res<SpatialIndex>,
    mut events: EventWriter<ConversationRequestEvent>,
    time: Res<Time>,
) {
    for (entity, pos, conv_cap, needs) in &creatures {
        // Skip if on cooldown
        if conv_cap.conversation_cooldown > 0.0 {
            continue;
        }
        
        // Skip if already conversing
        if conv_cap.current_conversation.is_some() {
            continue;
        }
        
        // Find nearby creatures
        let nearby = spatial_index.query_radius(pos.0, CONVERSATION_RANGE);
        
        for other_entity in nearby {
            if other_entity == entity { continue; }
            
            if let Ok((_, _, other_conv, _)) = creatures.get(other_entity) {
                if other_conv.current_conversation.is_none() {
                    // Determine if conversation should start
                    let topic = select_appropriate_topic(entity, other_entity, needs);
                    
                    if should_initiate_conversation(needs, topic) {
                        events.send(ConversationRequestEvent {
                            initiator: entity,
                            target: other_entity,
                            topic,
                            priority: topic.urgency(),
                        });
                    }
                }
            }
        }
    }
}
```

### Information Verification

```rust
fn verify_shared_information(
    mut creatures: Query<&mut ConversationMemory>,
    world_state: Res<WorldState>,
    mut events: EventWriter<InformationVerifiedEvent>,
) {
    for mut memory in &mut creatures {
        let mut verified_info = Vec::new();
        
        for (info, (source, timestamp)) in &memory.information_sources {
            match info {
                Information::ResourceLocation { position, resource_type, .. } => {
                    // Check if resource still exists
                    let still_exists = world_state.check_resource_at(*position, *resource_type);
                    
                    if !still_exists {
                        verified_info.push((info.clone(), false));
                    }
                }
                Information::AreaSafety { area, threat_level, .. } => {
                    // Check current threat level
                    let actual_threat = world_state.get_threat_level(*area);
                    
                    if (actual_threat - threat_level).abs() > 0.3 {
                        verified_info.push((info.clone(), false));
                    }
                }
                _ => {}
            }
        }
        
        // Update trust in sources based on verification
        for (info, accurate) in verified_info {
            if let Some((source, _)) = memory.information_sources.get(&info) {
                events.send(InformationVerifiedEvent {
                    source: *source,
                    accurate,
                });
            }
        }
    }
}
```

### Group Conversations

```rust
fn enable_group_conversations(
    mut pool: ResMut<ConversationPool>,
    groups: Query<&GroupMembership>,
    positions: Query<&Position>,
) {
    // Find clusters of creatures
    let clusters = find_conversation_clusters(&positions, GROUP_CONVERSATION_RADIUS);
    
    for cluster in clusters {
        if cluster.len() > 2 && cluster.len() <= MAX_GROUP_SIZE {
            // Check if they're in the same group
            let same_group = cluster.iter()
                .filter_map(|e| groups.get(*e).ok())
                .map(|g| g.group_id)
                .collect::<HashSet<_>>()
                .len() == 1;
            
            if same_group {
                // Start group conversation
                let topic = ConversationTopic::GroupFormation;
                pool.start_group_conversation(cluster, topic);
            }
        }
    }
}
```

## Advanced Features

### Conversation Styles

```rust
#[derive(Debug, Clone, Copy)]
pub enum CommunicationStyle {
    Direct,      // Get to the point
    Flowery,     // Elaborate and poetic
    Cautious,    // Careful with information
    Enthusiastic,// Excited sharing
    Wise,        // Teaching-focused
    Playful,     // Humor and games
}

impl CommunicationStyle {
    pub fn modify_message(&self, base: String) -> String {
        match self {
            Self::Direct => base, // No modification
            Self::Flowery => add_flourishes(base),
            Self::Cautious => add_hedging(base),
            Self::Enthusiastic => add_excitement(base),
            Self::Wise => add_profundity(base),
            Self::Playful => add_humor(base),
        }
    }
    
    pub fn affects_relationship_building(&self) -> f32 {
        match self {
            Self::Playful => 1.2,
            Self::Enthusiastic => 1.1,
            Self::Wise => 1.0,
            Self::Flowery => 0.9,
            Self::Direct => 0.8,
            Self::Cautious => 0.7,
        }
    }
}
```

### Memory Decay

```rust
fn decay_conversation_memories(
    mut creatures: Query<&mut ConversationMemory>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_seconds_f64();
    
    for mut memory in &mut creatures {
        // Recent conversations fade from history
        memory.conversation_history.retain(|record| {
            current_time - record.timestamp < CONVERSATION_MEMORY_DURATION
        });
        
        // Information degrades over time
        for (info, (source, timestamp)) in &mut memory.information_sources {
            if let Information::ResourceLocation { ref mut reliability, .. } = info {
                let age = current_time - *timestamp;
                *reliability *= (1.0 - age / INFORMATION_DECAY_TIME).max(0.0);
            }
        }
        
        // Remove unreliable information
        memory.information_sources.retain(|info, _| {
            match info {
                Information::ResourceLocation { reliability, .. } => *reliability > 0.1,
                _ => true,
            }
        });
    }
}
```

### Conversation Animations

```rust
fn animate_conversations(
    conversations: Query<(&ConversationState, &Position)>,
    mut creatures: Query<(&mut AnimationState, &mut EmotionDisplay)>,
    time: Res<Time>,
) {
    for (conversation, conv_pos) in &conversations {
        let speaker_entity = conversation.participants[conversation.current_speaker];
        
        if let Ok((mut anim, mut emotion)) = creatures.get_mut(speaker_entity) {
            // Speaking animation
            anim.play(AnimationClip::Talking);
            
            // Gesture based on topic
            match conversation.topic {
                ConversationTopic::DangerWarning(_) => {
                    anim.add_gesture(Gesture::PointUrgently);
                    emotion.show(Emotion::Alarmed);
                }
                ConversationTopic::FoodLocation(_) => {
                    anim.add_gesture(Gesture::PointDirection);
                    emotion.show(Emotion::Helpful);
                }
                ConversationTopic::Courtship => {
                    anim.add_gesture(Gesture::ShyApproach);
                    emotion.show(Emotion::Romantic);
                }
                _ => {}
            }
        }
        
        // Listener reactions
        for (i, &participant) in conversation.participants.iter().enumerate() {
            if i != conversation.current_speaker {
                if let Ok((mut anim, mut emotion)) = creatures.get_mut(participant) {
                    anim.play(AnimationClip::Listening);
                    
                    // Nod occasionally
                    if (time.elapsed_seconds() * 2.0).sin() > 0.8 {
                        anim.add_gesture(Gesture::Nod);
                    }
                }
            }
        }
    }
}
```

## Performance Monitoring

```rust
fn monitor_conversation_performance(
    pool: Res<ConversationPool>,
    diagnostics: Res<Diagnostics>,
    mut perf_stats: ResMut<ConversationPerfStats>,
) {
    perf_stats.active_conversations = pool.active_conversations.len();
    perf_stats.queued_conversations = pool.pending_conversations.len();
    
    if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(fps_value) = fps.value() {
            if fps_value < 55.0 && pool.active_conversations.len() > 30 {
                warn!("Conversation system may be impacting performance");
            }
        }
    }
}
```

## Testing Conversations

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_information_transfer() {
        let mut speaker_memory = ConversationMemory::default();
        let mut listener_memory = ConversationMemory::default();
        
        let info = Information::ResourceLocation {
            resource_type: ResourceType::Food,
            position: Vec2::new(100.0, 100.0),
            last_seen: 0.0,
            reliability: 1.0,
        };
        
        speaker_memory.knowledge_base.add(info.clone());
        
        transfer_information(
            &mut speaker_memory,
            &mut listener_memory,
            info.clone(),
            0.8, // 80% transfer quality
        );
        
        assert!(listener_memory.knowledge_base.contains(&info));
        assert_eq!(listener_memory.get_reliability(&info), Some(0.8));
    }
    
    #[test]
    fn test_relationship_evolution() {
        let mut relationship = Relationship::default();
        let conversation = ConversationRecord {
            topic: ConversationTopic::RelationshipBuilding,
            engagement_level: 0.8,
            successful: true,
            ..default()
        };
        
        update_relationship_from_conversation(
            &mut relationship,
            &conversation,
            &PersonalityTraits::default(),
            &PersonalityTraits::default(),
        );
        
        assert!(relationship.friendship > 0.0);
    }
}
```

---
*For architecture details, see [CONVERSATION_SYSTEM.md](./CONVERSATION_SYSTEM.md)*