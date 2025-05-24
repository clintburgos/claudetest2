# Creature Conversation System

## Overview
The conversation system enables meaningful social interactions between creatures, allowing them to share knowledge, influence each other's decisions, and form lasting relationships. Conversations are not just flavor text but have real gameplay impact on survival, behavior, and social dynamics.

## Architecture

### Core Concepts

```
Conversation Flow:
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│  Initiation  │────▶│   Exchange   │────▶│   Outcome    │
│  (Who/Why)   │     │ (What/How)   │     │  (Effects)   │
└──────────────┘     └──────────────┘     └──────────────┘
        │                    │                    │
        ▼                    ▼                    ▼
   Topic Selection    Information Flow    Relationship Change
   Proximity Check    Turn Taking         Knowledge Update
   Social Rules       Emotion Display     Decision Influence
```

### Component Structure

```rust
// Conversation components
#[derive(Component)]
pub struct ConversationCapability {
    // Language/communication traits
    eloquence: f32,          // 0.0-1.0, affects influence
    empathy: f32,            // 0.0-1.0, affects relationship building
    knowledge_retention: f32, // 0.0-1.0, affects memory
    
    // Current conversation state
    current_conversation: Option<ConversationId>,
    conversation_cooldown: f32,
    
    // Communication style (from genetics)
    style: CommunicationStyle,
}

#[derive(Component)]
pub struct ConversationMemory {
    // What this creature knows
    knowledge_base: KnowledgeBase,
    
    // Recent conversations
    conversation_history: VecDeque<ConversationRecord>,
    
    // Who told them what
    information_sources: HashMap<Information, (Entity, f64)>,
}

#[derive(Component)]
pub struct SocialContext {
    // Relationship tracking
    relationships: HashMap<Entity, Relationship>,
    
    // Group dynamics
    reputation: f32,
    influence_level: f32,
    
    // Social preferences
    preferred_conversation_size: usize,
    introversion_level: f32,
}
```

## Conversation Topics

### Topic Categories

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConversationTopic {
    // Survival Information
    FoodLocation(ResourceId),
    WaterLocation(ResourceId),
    DangerWarning(ThreatType),
    ShelterLocation(LocationId),
    
    // Social Topics  
    Introduction,
    RelationshipBuilding,
    GroupFormation,
    ConflictResolution,
    
    // Knowledge Sharing
    EnvironmentObservation,
    CreatureGossip(Entity),
    TeachingSkill(SkillType),
    PhilosophicalMusing,
    
    // Emotional Expression
    Celebration(EventType),
    Mourning(Entity),
    Complaint(ComplaintType),
    Encouragement,
    
    // Reproductive
    Courtship,
    PairBonding,
    NestPlanning,
}

impl ConversationTopic {
    pub fn urgency(&self) -> f32 {
        match self {
            Self::DangerWarning(_) => 1.0,
            Self::FoodLocation(_) => 0.8,
            Self::WaterLocation(_) => 0.8,
            Self::Mourning(_) => 0.7,
            Self::Courtship => 0.6,
            Self::RelationshipBuilding => 0.4,
            Self::PhilosophicalMusing => 0.1,
            _ => 0.5,
        }
    }
    
    pub fn social_distance_requirement(&self) -> f32 {
        match self {
            Self::Introduction => 5.0,
            Self::Courtship => 2.0,
            Self::ConflictResolution => 3.0,
            Self::DangerWarning(_) => 10.0, // Can shout warnings
            _ => 3.0,
        }
    }
}
```

### Topic Selection

```rust
pub fn select_conversation_topic(
    initiator: &CreatureContext,
    target: &CreatureContext,
    relationship: &Relationship,
) -> ConversationTopic {
    let mut topic_weights = HashMap::new();
    
    // Weight topics based on context
    
    // Urgent needs
    if initiator.needs.hunger < 30.0 {
        topic_weights.insert(
            ConversationTopic::FoodLocation(ResourceId::Any),
            80.0,
        );
    }
    
    // Relationship status
    if relationship.friendship < 0.2 {
        topic_weights.insert(ConversationTopic::Introduction, 60.0);
    } else if relationship.friendship > 0.8 {
        topic_weights.insert(ConversationTopic::PhilosophicalMusing, 40.0);
    }
    
    // Recent events
    if let Some(recent_death) = initiator.recent_deaths.last() {
        topic_weights.insert(
            ConversationTopic::Mourning(recent_death.entity),
            70.0,
        );
    }
    
    // Knowledge gaps
    if initiator.knowledge_base.is_missing_critical_info() {
        topic_weights.insert(ConversationTopic::EnvironmentObservation, 50.0);
    }
    
    // Romantic interest
    if initiator.is_ready_to_mate() && target.is_compatible_mate() {
        topic_weights.insert(ConversationTopic::Courtship, 65.0);
    }
    
    // Select weighted random topic
    weighted_random_selection(topic_weights)
}
```
## Conversation Exchange Mechanics

### Turn-Taking System

```rust
#[derive(Debug, Clone)]
pub struct ConversationState {
    id: ConversationId,
    participants: Vec<Entity>,
    current_speaker: usize,
    topic: ConversationTopic,
    turns_taken: u32,
    max_turns: u32,
    
    // Conversation quality metrics
    engagement_level: f32,
    information_exchanged: Vec<Information>,
    emotional_tone: EmotionalTone,
}

#[derive(Debug, Clone)]
pub enum ConversationPhase {
    Greeting,
    TopicIntroduction,
    InformationExchange,
    EmotionalResponse,
    Closing,
}

pub fn process_conversation_turn(
    conversation: &mut ConversationState,
    speaker: &CreatureContext,
    listener: &CreatureContext,
) -> ConversationTurnResult {
    let mut result = ConversationTurnResult::default();
    
    match conversation.get_current_phase() {
        ConversationPhase::Greeting => {
            // Initial pleasantries based on relationship
            result.message = generate_greeting(speaker, listener);
            result.relationship_change = 0.1;
        }
        
        ConversationPhase::TopicIntroduction => {
            // Speaker introduces the topic
            result.message = introduce_topic(speaker, conversation.topic);
            result.sets_tone = true;
        }
        
        ConversationPhase::InformationExchange => {
            // Core information transfer
            let info = extract_relevant_information(speaker, conversation.topic);
            result.information_shared = info;
            result.message = format_information(info, speaker.eloquence);
        }
        
        ConversationPhase::EmotionalResponse => {
            // Listener responds emotionally
            result.emotional_impact = calculate_emotional_impact(
                &conversation.topic,
                listener.empathy,
            );
            result.message = generate_emotional_response(listener, result.emotional_impact);
        }
        
        ConversationPhase::Closing => {
            // Wrap up conversation
            result.message = generate_farewell(speaker, conversation.engagement_level);
            result.ends_conversation = true;
        }
    }
    
    result
}
```

### Information Types and Transfer

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Information {
    // Location knowledge
    ResourceLocation {
        resource_type: ResourceType,
        position: Vec2,
        last_seen: f64,
        reliability: f32,
    },
    
    // Environmental knowledge
    AreaSafety {
        area: AreaId,
        threat_level: f32,
        threat_type: Option<ThreatType>,
    },
    
    // Social knowledge
    CreatureInformation {
        subject: Entity,
        info_type: CreatureInfoType,
        timestamp: f64,
    },
    
    // Learned behaviors
    Technique {
        skill: SkillType,
        efficiency_modifier: f32,
    },
    
    // Abstract concepts
    PhilosophicalIdea {
        concept: ConceptType,
        understanding_level: f32,
    },
}

#[derive(Debug, Clone)]
pub enum CreatureInfoType {
    Location(Vec2),
    HealthStatus(f32),
    EmotionalState(EmotionalState),
    RelationshipStatus(Entity, RelationshipType),
    RecentActivity(Activity),
}

pub fn transfer_information(
    speaker: &mut ConversationMemory,
    listener: &mut ConversationMemory,
    info: Information,
    transfer_quality: f32,
) {
    // Check if speaker actually has this information
    if !speaker.knowledge_base.contains(&info) {
        return; // Can't share what you don't know
    }
    
    // Degrade information based on transfer quality
    let degraded_info = match info {
        Information::ResourceLocation { mut reliability, .. } => {
            reliability *= transfer_quality;
            info.with_reliability(reliability)
        }
        Information::AreaSafety { mut threat_level, .. } => {
            // Threat info might be exaggerated or downplayed
            threat_level *= transfer_quality + (1.0 - transfer_quality) * rand::random::<f32>();
            info.with_threat_level(threat_level)
        }
        _ => info,
    };
    
    // Add to listener's knowledge with source tracking
    listener.knowledge_base.add(degraded_info.clone());
    listener.information_sources.insert(
        degraded_info,
        (speaker.entity, current_time()),
    );
}
```

### Conversation Content Generation

```rust
pub struct ConversationGenerator {
    templates: HashMap<(ConversationTopic, CommunicationStyle), Vec<MessageTemplate>>,
    personality_modifiers: HashMap<PersonalityTrait, MessageModifier>,
}

#[derive(Clone)]
pub struct MessageTemplate {
    base_text: &'static str,
    placeholders: Vec<Placeholder>,
    emotional_tone: EmotionalTone,
}

#[derive(Clone)]
pub enum Placeholder {
    CreatureName,
    ResourceType,
    Location,
    Emotion,
    TimeAgo,
}

impl ConversationGenerator {
    pub fn generate_message(
        &self,
        topic: ConversationTopic,
        speaker: &CreatureContext,
        context: &ConversationContext,
    ) -> Message {
        // Select template based on topic and style
        let templates = self.templates.get(&(topic, speaker.style)).unwrap();
        let template = weighted_random_select(templates, |t| {
            t.emotional_tone.compatibility(speaker.current_emotion)
        });
        
        // Fill in placeholders
        let mut text = template.base_text.to_string();
        for placeholder in &template.placeholders {
            let replacement = match placeholder {
                Placeholder::CreatureName => speaker.name.clone(),
                Placeholder::ResourceType => context.relevant_resource.to_string(),
                Placeholder::Location => format_location(context.location),
                Placeholder::Emotion => speaker.current_emotion.to_string(),
                Placeholder::TimeAgo => format_time_ago(context.timestamp),
            };
            text = text.replace(&placeholder.token(), &replacement);
        }
        
        // Apply personality modifiers
        for (trait, modifier) in &self.personality_modifiers {
            if speaker.has_trait(trait) {
                text = modifier.apply(text);
            }
        }
        
        Message {
            text,
            tone: template.emotional_tone,
            gestures: generate_gestures(speaker.current_emotion),
        }
    }
}
```
## Relationship and Influence Mechanics

### Relationship Evolution

```rust
#[derive(Debug, Clone)]
pub struct Relationship {
    // Core metrics
    friendship: f32,        // -1.0 to 1.0
    trust: f32,            // 0.0 to 1.0  
    romantic_interest: f32, // 0.0 to 1.0
    respect: f32,          // 0.0 to 1.0
    
    // History
    first_meeting: f64,
    last_interaction: f64,
    interaction_count: u32,
    
    // Shared experiences
    shared_meals: u32,
    dangers_faced_together: u32,
    offspring_together: Vec<Entity>,
    
    // Communication history
    topics_discussed: HashSet<ConversationTopic>,
    information_shared: Vec<Information>,
    conflicts: Vec<ConflictRecord>,
}

pub fn update_relationship_from_conversation(
    relationship: &mut Relationship,
    conversation: &ConversationRecord,
    speaker_traits: &PersonalityTraits,
    listener_traits: &PersonalityTraits,
) {
    // Base relationship change from conversation quality
    let base_change = conversation.engagement_level * 0.1;
    
    // Modify based on topic
    let topic_modifier = match conversation.topic {
        ConversationTopic::RelationshipBuilding => 2.0,
        ConversationTopic::ConflictResolution => {
            if conversation.successful {
                3.0 // Big boost for resolving conflicts
            } else {
                -1.0 // Penalty for failed resolution
            }
        }
        ConversationTopic::Courtship => {
            if listener_traits.is_receptive() {
                1.5
            } else {
                -0.5 // Unwanted advances harm relationship
            }
        }
        ConversationTopic::DangerWarning(_) => 1.5, // Grateful for warning
        _ => 1.0,
    };
    
    // Personality compatibility
    let compatibility = calculate_personality_compatibility(speaker_traits, listener_traits);
    
    // Update relationship metrics
    relationship.friendship += base_change * topic_modifier * compatibility;
    relationship.friendship = relationship.friendship.clamp(-1.0, 1.0);
    
    // Trust changes based on information accuracy
    if let Some(verified_info) = conversation.information_verified {
        if verified_info {
            relationship.trust += 0.1;
        } else {
            relationship.trust -= 0.2; // Lies hurt more than truth helps
        }
    }
    
    // Update other metrics based on specific topics
    match conversation.topic {
        ConversationTopic::Courtship if conversation.successful => {
            relationship.romantic_interest += 0.2;
        }
        ConversationTopic::TeachingSkill(_) => {
            relationship.respect += 0.1;
        }
        _ => {}
    }
    
    // Record interaction
    relationship.last_interaction = current_time();
    relationship.interaction_count += 1;
    relationship.topics_discussed.insert(conversation.topic);
}
```

### Influence and Decision Impact

```rust
#[derive(Component)]
pub struct SocialInfluence {
    // How much this creature influences others
    influence_power: f32,
    charisma: f32,
    reputation: f32,
    
    // Recent influence events
    influenced_decisions: VecDeque<InfluenceEvent>,
}

#[derive(Debug, Clone)]
pub struct InfluenceEvent {
    target: Entity,
    decision_changed: Goal,
    influence_strength: f32,
    timestamp: f64,
}

pub fn apply_conversation_influence(
    speaker_influence: &SocialInfluence,
    listener_decisions: &mut DecisionState,
    conversation: &ConversationRecord,
    relationship: &Relationship,
) -> Option<InfluenceEvent> {
    // Calculate influence strength
    let influence_strength = calculate_influence_strength(
        speaker_influence,
        relationship,
        conversation.topic,
    );
    
    // Determine which goal to influence based on topic
    let influenced_goal = match conversation.topic {
        ConversationTopic::FoodLocation(_) => Some(Goal::SatisfyHunger),
        ConversationTopic::DangerWarning(_) => Some(Goal::FleeFromDanger),
        ConversationTopic::GroupFormation => Some(Goal::Socialize),
        ConversationTopic::ShelterLocation(_) => Some(Goal::SeekShelter),
        _ => None,
    };
    
    if let Some(goal) = influenced_goal {
        if influence_strength > INFLUENCE_THRESHOLD {
            // Modify utility scores for influenced goal
            listener_decisions.utility_modifiers.insert(
                goal,
                UtilityModifier {
                    multiplier: 1.0 + influence_strength,
                    duration: 300.0, // 5 minutes
                    source: InfluenceSource::Conversation(speaker_influence.entity),
                },
            );
            
            return Some(InfluenceEvent {
                target: listener_decisions.entity,
                decision_changed: goal,
                influence_strength,
                timestamp: current_time(),
            });
        }
    }
    
    None
}

fn calculate_influence_strength(
    speaker: &SocialInfluence,
    relationship: &Relationship,
    topic: ConversationTopic,
) -> f32 {
    // Base influence from speaker's power
    let base = speaker.influence_power;
    
    // Multiply by relationship factors
    let relationship_multiplier = 
        relationship.trust * 0.5 + 
        relationship.respect * 0.3 + 
        relationship.friendship.max(0.0) * 0.2;
    
    // Topic urgency affects influence
    let urgency_multiplier = topic.urgency();
    
    // Charisma bonus
    let charisma_bonus = speaker.charisma * 0.2;
    
    (base + charisma_bonus) * relationship_multiplier * urgency_multiplier
}
```
## Performance Optimization

### Conversation Pooling

```rust
#[derive(Resource)]
pub struct ConversationPool {
    active_conversations: HashMap<ConversationId, ConversationState>,
    pending_conversations: VecDeque<ConversationRequest>,
    conversation_budget: usize, // Max simultaneous conversations
    
    // Reusable allocations
    message_buffer: String,
    information_buffer: Vec<Information>,
}

impl ConversationPool {
    pub fn start_conversation(
        &mut self,
        initiator: Entity,
        target: Entity,
        topic: ConversationTopic,
    ) -> Option<ConversationId> {
        // Check if under budget
        if self.active_conversations.len() >= self.conversation_budget {
            // Queue for later
            self.pending_conversations.push_back(ConversationRequest {
                initiator,
                target,
                topic,
                priority: topic.urgency(),
            });
            return None;
        }
        
        // Reuse completed conversation slots
        let id = ConversationId::new();
        let state = ConversationState {
            id,
            participants: vec![initiator, target],
            current_speaker: 0,
            topic,
            turns_taken: 0,
            max_turns: calculate_max_turns(topic),
            engagement_level: 1.0,
            information_exchanged: Vec::new(),
            emotional_tone: EmotionalTone::Neutral,
        };
        
        self.active_conversations.insert(id, state);
        Some(id)
    }
}
```

### LOD-Based Conversation Detail

```rust
pub fn process_conversation_with_lod(
    conversation: &mut ConversationState,
    participants: &[(Entity, LODLevel)],
    pool: &mut ConversationPool,
) -> ConversationUpdate {
    let min_lod = participants.iter().map(|(_, lod)| lod.0).min().unwrap_or(3);
    
    match min_lod {
        0 => {
            // Full conversation simulation
            full_conversation_simulation(conversation, participants, pool)
        }
        1 => {
            // Simplified dialogue, full information transfer
            simplified_conversation(conversation, participants)
        }
        2 => {
            // Just information transfer, no dialogue
            information_only_transfer(conversation, participants)
        }
        _ => {
            // Statistical outcome only
            statistical_conversation_result(conversation)
        }
    }
}

fn full_conversation_simulation(
    conversation: &mut ConversationState,
    participants: &[(Entity, LODLevel)],
    pool: &mut ConversationPool,
) -> ConversationUpdate {
    // Generate actual dialogue
    pool.message_buffer.clear();
    let message = generate_full_message(
        conversation,
        &participants[conversation.current_speaker],
        &mut pool.message_buffer,
    );
    
    // Process emotional responses
    let emotional_impact = calculate_full_emotional_impact(
        &message,
        &participants,
    );
    
    // Transfer information with degradation
    let information = extract_and_degrade_information(
        conversation.topic,
        participants[conversation.current_speaker].0,
    );
    
    ConversationUpdate {
        message: Some(message),
        emotional_changes: emotional_impact,
        information_transferred: information,
        animation_triggers: generate_conversation_animations(&conversation),
    }
}
```

### Batch Processing

```rust
pub fn batch_process_conversations(
    mut conversations: Query<&mut ConversationState>,
    participants: Query<(&Position, &LODLevel, &ConversationCapability)>,
    mut pool: ResMut<ConversationPool>,
    time: Res<Time>,
) {
    // Group by LOD level for cache efficiency
    let mut lod_groups: [Vec<Entity>; 4] = Default::default();
    
    for (entity, conv_state) in &conversations {
        if let Ok((_, lod, _)) = participants.get(entity) {
            lod_groups[lod.0.min(3) as usize].push(entity);
        }
    }
    
    // Process each LOD group with appropriate detail
    for (lod_level, entities) in lod_groups.iter().enumerate() {
        let update_frequency = match lod_level {
            0 => 0.1,  // 10 Hz for nearby
            1 => 0.5,  // 2 Hz for medium
            2 => 1.0,  // 1 Hz for far
            _ => 2.0,  // 0.5 Hz for very far
        };
        
        for &entity in entities {
            if let Ok(mut conv_state) = conversations.get_mut(entity) {
                if time.elapsed_seconds() % update_frequency < time.delta_seconds() {
                    process_conversation_turn(&mut conv_state, lod_level);
                }
            }
        }
    }
}
```

## Integration with Other Systems

### Decision System Integration

```rust
impl Plugin for ConversationPlugin {
    fn build(&self, app: &mut App) {
        app
            // Events that trigger conversations
            .add_event::<ConversationRequestEvent>()
            .add_event::<ConversationCompleteEvent>()
            .add_event::<InformationSharedEvent>()
            
            // Systems
            .add_systems(
                Update,
                (
                    // Conversation initiation from decisions
                    initiate_conversations_from_decisions
                        .after(creature_decision_system),
                    
                    // Main conversation processing
                    process_active_conversations,
                    
                    // Apply conversation outcomes
                    apply_conversation_influences
                        .before(creature_decision_system),
                    
                    update_relationships_from_conversations,
                    
                    // Cleanup
                    cleanup_completed_conversations,
                )
                .chain()
                .in_set(GameSet::Social),
            );
    }
}
```

### Knowledge Propagation

```rust
pub fn propagate_critical_information(
    creatures: Query<(&Position, &ConversationMemory)>,
    spatial_index: Res<SpatialIndex>,
    mut events: EventWriter<ConversationRequestEvent>,
) {
    // Find creatures with critical information
    for (position, memory) in &creatures {
        if let Some(critical_info) = memory.get_most_critical_information() {
            // Find nearby creatures who don't have this info
            let nearby = spatial_index.query_radius(position.0, SHOUT_RADIUS);
            
            for other_entity in nearby {
                if let Ok((_, other_memory)) = creatures.get(other_entity) {
                    if !other_memory.knows(&critical_info) {
                        // Trigger urgent conversation
                        events.send(ConversationRequestEvent {
                            initiator: memory.entity,
                            target: other_entity,
                            topic: critical_info.to_topic(),
                            urgency: critical_info.urgency(),
                        });
                    }
                }
            }
        }
    }
}
```
## Emergent Behaviors

### Gossip Networks

```rust
pub fn simulate_gossip_spread(
    conversations: Query<&ConversationState>,
    memories: Query<&mut ConversationMemory>,
) {
    // Track how information spreads through social networks
    for conversation in &conversations {
        if matches!(conversation.topic, ConversationTopic::CreatureGossip(_)) {
            // Information gets modified as it spreads
            let gossip_degradation = 0.9;
            let rumor_exaggeration = 1.1;
            
            // Some creatures are more reliable gossips than others
            let speaker_reliability = get_gossip_reliability(conversation.current_speaker);
            
            // Gossip can become distorted
            if random::<f32>() > speaker_reliability {
                // Information becomes rumor
                modify_information_to_rumor(&mut conversation.information_exchanged);
            }
        }
    }
}
```

### Cultural Knowledge

```rust
#[derive(Component)]
pub struct CulturalKnowledge {
    // Shared group knowledge
    group_techniques: HashMap<SkillType, Technique>,
    group_beliefs: HashSet<PhilosophicalIdea>,
    group_traditions: Vec<Tradition>,
    
    // How well integrated into group culture
    cultural_integration: f32,
}

pub fn spread_cultural_knowledge(
    groups: Query<&GroupMembership>,
    conversations: Query<(&ConversationState, &CulturalKnowledge)>,
    mut knowledge: Query<&mut CulturalKnowledge>,
) {
    for (conversation, speaker_culture) in &conversations {
        if conversation.engagement_level > 0.7 {
            // High-quality conversations can spread culture
            for &participant in &conversation.participants {
                if let Ok(mut participant_culture) = knowledge.get_mut(participant) {
                    // Transfer techniques with teaching
                    if matches!(conversation.topic, ConversationTopic::TeachingSkill(_)) {
                        transfer_technique(
                            &speaker_culture.group_techniques,
                            &mut participant_culture.group_techniques,
                            conversation.engagement_level,
                        );
                    }
                    
                    // Philosophical ideas spread through discussion
                    if matches!(conversation.topic, ConversationTopic::PhilosophicalMusing) {
                        spread_philosophy(
                            &speaker_culture.group_beliefs,
                            &mut participant_culture.group_beliefs,
                        );
                    }
                }
            }
        }
    }
}
```

### Relationship Networks

```rust
pub fn analyze_social_networks(
    relationships: Query<&Relationship>,
    creatures: Query<&CreatureId>,
) -> SocialNetworkAnalysis {
    // Build adjacency graph
    let mut network = Graph::new();
    
    for (entity, relationships) in &relationships {
        for (other, relationship) in relationships {
            if relationship.friendship > 0.5 {
                network.add_edge(entity, other, relationship.friendship);
            }
        }
    }
    
    // Find communities
    let communities = detect_communities(&network);
    
    // Identify key individuals
    let influencers = find_high_centrality_nodes(&network);
    let bridges = find_bridge_nodes(&network);
    
    SocialNetworkAnalysis {
        communities,
        influencers,
        bridges,
        average_connections: calculate_average_degree(&network),
        clustering_coefficient: calculate_clustering(&network),
    }
}
```

## Example Conversations

### Example 1: Food Sharing
```
[Context: Hungry creature meets well-fed friend]

Blinky: *stomach growls* "Haven't eaten in so long..."
Spark: "Oh! I found berries near the big rock earlier!"
Blinky: *perks up* "Really? Where exactly?"
Spark: "Follow the stream north, then look for the mossy stones."
Blinky: *grateful* "Thank you, friend!"

[Outcome: 
- Blinky gains ResourceLocation information
- Friendship +0.2
- Blinky's next goal becomes SatisfyHunger with known target]
```

### Example 2: Danger Warning
```
[Context: Creature spots predator, warns others]

Alert: *agitated* "DANGER! Big threat near the watering hole!"
Calm: *startled* "What kind of threat?"
Alert: "Something huge with sharp teeth! Saw it catch Tiny!"
Calm: *fearful* "We should warn the others!"

[Outcome:
- AreaSafety information spreads
- Both creatures' goals change to FleeFromDanger
- Information propagates to nearby creatures]
```

### Example 3: Courtship
```
[Context: Mutual attraction during mating season]

Shimmer: *shy approach* "Beautiful day, isn't it?"
Glow: *interested* "More beautiful with you here."
Shimmer: *hopeful* "Perhaps we could explore together?"
Glow: *accepting* "I'd like that very much."

[Outcome:
- Romantic interest +0.3
- Pair bond initiated
- Both goals change to spend time together]
```

## Configuration and Tuning

```rust
#[derive(Resource)]
pub struct ConversationSettings {
    // Performance
    pub max_simultaneous_conversations: usize,
    pub conversation_update_rate: f32,
    pub message_generation_budget_ms: f32,
    
    // Gameplay
    pub information_degradation_rate: f32,
    pub gossip_reliability_base: f32,
    pub influence_threshold: f32,
    pub relationship_change_rate: f32,
    
    // Content
    pub enable_philosophical_musings: bool,
    pub enable_gossip: bool,
    pub enable_teaching: bool,
    pub max_conversation_turns: u32,
}

impl Default for ConversationSettings {
    fn default() -> Self {
        Self {
            max_simultaneous_conversations: 50,
            conversation_update_rate: 0.2, // 5 Hz
            message_generation_budget_ms: 0.1,
            
            information_degradation_rate: 0.9,
            gossip_reliability_base: 0.7,
            influence_threshold: 0.3,
            relationship_change_rate: 0.1,
            
            enable_philosophical_musings: true,
            enable_gossip: true,
            enable_teaching: true,
            max_conversation_turns: 10,
        }
    }
}
```

## Best Practices

### Content Guidelines
1. **Keep messages short** - 1-2 sentences max for performance
2. **Use templates** - Pre-generate variations for common topics
3. **Emotion > Words** - Visual cues matter more than text
4. **Meaningful outcomes** - Every conversation should matter

### Performance Guidelines  
1. **Pool everything** - Reuse conversation objects and buffers
2. **LOD aggressively** - Full conversations only for nearby creatures
3. **Batch by type** - Process similar conversations together
4. **Cache relationships** - Don't recalculate every frame

### Design Guidelines
1. **Information has value** - Make knowledge worth sharing
2. **Relationships matter** - Strong bonds affect survival
3. **Culture emerges** - Let groups develop unique characteristics
4. **Stories unfold** - Design for memorable moments

## Debugging and Visualization

```rust
#[cfg(feature = "debug")]
pub fn debug_draw_conversations(
    conversations: Query<(&ConversationState, &Position)>,
    participants: Query<&Position>,
    mut gizmos: Gizmos,
) {
    for (conversation, pos) in &conversations {
        // Draw speech bubbles
        gizmos.circle_2d(pos.0, 10.0, Color::WHITE);
        
        // Draw connection lines between participants
        for &participant in &conversation.participants {
            if let Ok(other_pos) = participants.get(participant) {
                gizmos.line_2d(pos.0, other_pos.0, Color::rgba(1.0, 1.0, 1.0, 0.3));
            }
        }
        
        // Show topic icon
        draw_topic_icon(&mut gizmos, pos.0 + Vec2::new(0.0, 15.0), conversation.topic);
    }
}
```

---
*Last Updated: 2024-12-XX*