# Conversation System

## Table of Contents
1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Implementation Guide](#implementation-guide)
4. [Quick Reference](#quick-reference)

---

## Overview

### What Makes Conversations Meaningful

Conversations aren't just flavor text - they directly affect creature survival and behavior:

1. **Knowledge Transfer**
   - Creatures share resource locations
   - Danger warnings save lives
   - Skills propagate through teaching

2. **Decision Influence**  
   - Trusted friends change goal priorities
   - Social pressure affects choices
   - Group consensus emerges

3. **Relationship Building**
   - Strong bonds improve cooperation
   - Trust affects information reliability
   - Romance leads to offspring

4. **Cultural Evolution**
   - Groups develop unique knowledge
   - Techniques spread through populations
   - Philosophical ideas create identity

### Why It Matters

Conversations create the "soul" of the simulation:
- Creatures feel alive through communication
- Players see relationships form and break
- Knowledge flows create dynamic ecosystems
- Stories emerge from simple interactions

The conversation system transforms creatures from isolated entities into a living, breathing society where information, relationships, and culture evolve naturally through meaningful interactions.

---

## Architecture

### Core Components

```rust
pub struct ConversationSystem {
    active_conversations: HashMap<ConversationId, Conversation>,
    conversation_manager: ConversationManager,
    dialogue_generator: DialogueGenerator,
    outcome_processor: OutcomeProcessor,
}

pub struct Conversation {
    id: ConversationId,
    participants: Vec<EntityId>,
    topic: ConversationTopic,
    state: ConversationState,
    context: ConversationContext,
    history: Vec<DialogueExchange>,
}

pub enum ConversationTopic {
    // Survival topics
    FoodLocation { resource: ResourceType, location: Vec3 },
    WaterSource { quality: f32, location: Vec3 },
    DangerWarning { threat_type: ThreatType, location: Vec3, severity: f32 },
    
    // Social topics
    Gossip { subject: EntityId, information: GossipType },
    RelationshipBuilding { activity: BondingActivity },
    ConflictResolution { issue: ConflictType },
    
    // Knowledge topics
    TeachingSkill { skill: SkillType, technique: Technique },
    PhilosophicalIdea { concept: PhilosophicalConcept },
    CulturalTradition { tradition: TraditionType },
}

pub struct ConversationContext {
    location: Vec3,
    time_of_day: f32,
    weather: Weather,
    nearby_creatures: Vec<EntityId>,
    environmental_factors: Vec<EnvironmentalFactor>,
    urgency_level: f32,
}

pub enum ConversationState {
    Initiating,
    Greeting,
    TopicIntroduction,
    InformationExchange,
    EmotionalResponse,
    Concluding,
    Ended,
}
```

### Information Flow

```rust
pub struct InformationPacket {
    content: Information,
    accuracy: f32,          // 1.0 = perfect, 0.0 = completely wrong
    source_trust: f32,      // How much the receiver trusts the source
    urgency: f32,          // How important this info is
    decay_rate: f32,       // How fast the info becomes outdated
}

pub enum Information {
    Location { target: LocationTarget, position: Vec3, confidence: f32 },
    Warning { danger: DangerType, severity: f32, direction: Vec3 },
    Skill { skill_type: SkillType, technique: Vec<ActionStep> },
    Social { gossip: GossipContent, subject: EntityId },
}

impl InformationPacket {
    pub fn degrade_accuracy(&mut self, steps: u32) {
        // Information degrades as it passes between creatures
        self.accuracy *= 0.9_f32.powi(steps as i32);
        
        // Add noise based on degradation
        if self.accuracy < 0.8 {
            match &mut self.content {
                Information::Location { position, confidence, .. } => {
                    let noise = Vec3::random() * (1.0 - self.accuracy) * 10.0;
                    *position += noise;
                    *confidence *= self.accuracy;
                }
                _ => {}
            }
        }
    }
}
```

### Dialogue Generation

```rust
pub struct DialogueGenerator {
    personality_templates: HashMap<PersonalityType, DialogueStyle>,
    relationship_modifiers: HashMap<RelationshipType, ToneModifier>,
    urgency_overrides: Vec<UrgencyRule>,
}

impl DialogueGenerator {
    pub fn generate_dialogue(
        &self,
        speaker: &Creature,
        topic: &ConversationTopic,
        context: &ConversationContext,
    ) -> DialogueLine {
        let base_style = self.personality_templates[&speaker.personality];
        let relationship_tone = self.get_relationship_tone(speaker, context);
        let urgency_factor = self.calculate_urgency(topic, speaker);
        
        // Generate appropriate dialogue
        match (topic, urgency_factor) {
            (ConversationTopic::DangerWarning { .. }, u) if u > 0.8 => {
                self.generate_urgent_warning(speaker, topic)
            }
            (ConversationTopic::FoodLocation { .. }, _) if speaker.hunger < 20.0 => {
                self.generate_desperate_request(speaker, topic)
            }
            _ => self.generate_normal_dialogue(speaker, topic, base_style, relationship_tone)
        }
    }
    
    fn generate_urgent_warning(&self, speaker: &Creature, topic: &ConversationTopic) -> DialogueLine {
        DialogueLine {
            text: self.create_warning_text(topic),
            emotion: Emotion::Fear,
            volume: 1.0, // Shouting
            gestures: vec![Gesture::Pointing, Gesture::Retreat],
        }
    }
}
```

### Conversation Outcomes

```rust
pub struct ConversationOutcome {
    knowledge_transfers: Vec<KnowledgeTransfer>,
    relationship_changes: Vec<RelationshipChange>,
    decision_influences: Vec<DecisionInfluence>,
    emotional_impacts: Vec<EmotionalImpact>,
}

pub struct KnowledgeTransfer {
    from: EntityId,
    to: EntityId,
    knowledge: Information,
    retention_probability: f32,
}

pub struct DecisionInfluence {
    creature: EntityId,
    goal_changes: Vec<(GoalType, f32)>, // Goal and priority delta
    immediate_action: Option<Action>,
}

impl OutcomeProcessor {
    pub fn process_conversation(
        &mut self,
        conversation: &Conversation,
        participants: &mut [&mut Creature],
    ) -> ConversationOutcome {
        let mut outcome = ConversationOutcome::default();
        
        // Process based on topic and exchanges
        match &conversation.topic {
            ConversationTopic::FoodLocation { resource, location } => {
                // Transfer location knowledge
                for (i, exchange) in conversation.history.iter().enumerate() {
                    if exchange.contains_information() {
                        outcome.knowledge_transfers.push(KnowledgeTransfer {
                            from: exchange.speaker,
                            to: exchange.listener,
                            knowledge: Information::Location {
                                target: LocationTarget::Resource(*resource),
                                position: *location,
                                confidence: 0.8,
                            },
                            retention_probability: participants[1].memory_capacity(),
                        });
                    }
                }
                
                // Influence decisions
                if participants[1].hunger < 50.0 {
                    outcome.decision_influences.push(DecisionInfluence {
                        creature: participants[1].id,
                        goal_changes: vec![(GoalType::SatisfyHunger, 0.3)],
                        immediate_action: Some(Action::MoveTo(*location)),
                    });
                }
            }
            _ => {}
        }
        
        outcome
    }
}
```

### Performance Optimization

The conversation system uses the LOD (Level of Detail) system for performance optimization. See [Performance Guide](../reference/PERFORMANCE.md#lod-system) for details on:
- Distance-based detail levels
- Simplified vs full conversation processing
- Statistical outcome application for distant creatures
- Integration with the global LOD manager

---

## Implementation Guide

### Starting a Conversation

```rust
impl CreatureConversationBehavior {
    pub fn try_start_conversation(
        &mut self,
        creature: &Creature,
        nearby_creatures: &[EntityId],
        world: &World,
    ) -> Option<ConversationRequest> {
        // Check if creature wants to converse
        if !self.should_converse(creature) {
            return None;
        }
        
        // Find suitable conversation partner
        let partner = self.find_best_partner(creature, nearby_creatures, world)?;
        
        // Determine topic based on needs and context
        let topic = self.select_topic(creature, partner, world);
        
        // Create conversation request
        Some(ConversationRequest {
            initiator: creature.id,
            target: partner.id,
            topic,
            urgency: self.calculate_urgency(creature, &topic),
        })
    }
    
    fn should_converse(&self, creature: &Creature) -> bool {
        // Don't converse if in danger
        if creature.threat_level > 0.7 {
            return false;
        }
        
        // Check social needs
        if creature.social_need > 0.6 {
            return true;
        }
        
        // Check for urgent information to share
        if creature.has_urgent_info() {
            return true;
        }
        
        // Random chance based on personality
        rand::random::<f32>() < creature.sociability * 0.1
    }
    
    fn select_topic(&self, creature: &Creature, partner: &Creature, world: &World) -> ConversationTopic {
        // Priority system for topic selection
        if creature.has_danger_info() && !partner.knows_danger() {
            return ConversationTopic::DangerWarning {
                threat_type: creature.get_known_threat(),
                location: creature.get_threat_location(),
                severity: creature.get_threat_severity(),
            };
        }
        
        if creature.hunger > 70.0 && partner.hunger < 50.0 {
            return ConversationTopic::FoodLocation {
                resource: ResourceType::Food,
                location: Vec3::ZERO, // Will be filled by partner
            };
        }
        
        // Default to social bonding
        ConversationTopic::RelationshipBuilding {
            activity: BondingActivity::SmallTalk,
        }
    }
}
```

### Processing Dialogue Exchanges

```rust
impl DialogueProcessor {
    pub fn process_exchange(
        &mut self,
        speaker: &mut Creature,
        listener: &mut Creature,
        dialogue: &DialogueLine,
        topic: &ConversationTopic,
    ) -> ExchangeResult {
        // Update emotional states
        listener.emotional_state.react_to_dialogue(dialogue);
        
        // Process information transfer
        if let Some(info) = self.extract_information(dialogue, topic) {
            self.transfer_information(speaker, listener, info);
        }
        
        // Update relationship
        let relationship_delta = self.calculate_relationship_impact(
            dialogue,
            speaker.personality,
            listener.personality,
        );
        
        listener.update_relationship(speaker.id, relationship_delta);
        
        // Determine response
        let response_type = self.determine_response(listener, dialogue, topic);
        
        ExchangeResult {
            emotional_impact: listener.emotional_state.get_change(),
            knowledge_gained: info.is_some(),
            relationship_change: relationship_delta,
            response_type,
        }
    }
    
    fn transfer_information(
        &self,
        speaker: &Creature,
        listener: &mut Creature,
        info: Information,
    ) {
        // Calculate retention based on:
        // - Listener's intelligence
        // - Information complexity
        // - Current cognitive load
        // - Trust in speaker
        
        let trust = listener.get_trust(speaker.id);
        let retention_chance = listener.intelligence * trust * 0.8;
        
        if rand::random::<f32>() < retention_chance {
            listener.memory.add_information(info, speaker.id);
        }
    }
}
```

### Memory Integration

```rust
pub struct ConversationMemory {
    recent_conversations: RingBuffer<ConversationSummary>,
    learned_information: HashMap<InfoType, Vec<LearnedInfo>>,
    conversation_partners: HashMap<EntityId, PartnerHistory>,
}

pub struct ConversationSummary {
    partner: EntityId,
    topic: ConversationTopic,
    outcome: ConversationOutcome,
    timestamp: f64,
    location: Vec3,
}

pub struct LearnedInfo {
    content: Information,
    source: EntityId,
    confidence: f32,
    timestamp: f64,
    verified: bool,
}

impl ConversationMemory {
    pub fn remember_conversation(&mut self, summary: ConversationSummary) {
        // Add to recent conversations
        self.recent_conversations.push(summary.clone());
        
        // Update partner history
        self.conversation_partners
            .entry(summary.partner)
            .or_insert_with(PartnerHistory::new)
            .add_conversation(&summary);
        
        // Extract and store learned information
        for knowledge in summary.outcome.knowledge_transfers {
            self.learned_information
                .entry(knowledge.info_type())
                .or_insert_with(Vec::new)
                .push(LearnedInfo {
                    content: knowledge.knowledge,
                    source: summary.partner,
                    confidence: knowledge.initial_confidence(),
                    timestamp: summary.timestamp,
                    verified: false,
                });
        }
    }
    
    pub fn verify_information(&mut self, info_type: InfoType, actual: &Information) -> bool {
        if let Some(learned_infos) = self.learned_information.get_mut(&info_type) {
            for info in learned_infos {
                if info.content.matches(actual) {
                    info.verified = true;
                    info.confidence = 1.0;
                    
                    // Increase trust in source
                    return true;
                } else if info.content.contradicts(actual) {
                    info.confidence *= 0.5;
                    
                    // Decrease trust in source
                    return false;
                }
            }
        }
        false
    }
}
```

### Cultural Knowledge System

```rust
pub struct CulturalKnowledge {
    group_knowledge: HashMap<GroupId, GroupKnowledge>,
    knowledge_spread: KnowledgeSpreadModel,
}

pub struct GroupKnowledge {
    shared_information: Vec<Information>,
    unique_techniques: Vec<Technique>,
    philosophical_ideas: Vec<PhilosophicalConcept>,
    traditions: Vec<Tradition>,
    knowledge_holders: HashMap<Information, Vec<EntityId>>,
}

impl CulturalKnowledge {
    pub fn spread_knowledge(
        &mut self,
        conversation: &Conversation,
        outcome: &ConversationOutcome,
    ) {
        for transfer in &outcome.knowledge_transfers {
            // Check if this creates new group knowledge
            let from_group = self.get_creature_group(transfer.from);
            let to_group = self.get_creature_group(transfer.to);
            
            if from_group != to_group {
                // Knowledge spreading between groups
                self.knowledge_spread.record_transfer(
                    from_group,
                    to_group,
                    transfer.knowledge.clone(),
                );
                
                // Add to receiving group's knowledge
                self.group_knowledge
                    .entry(to_group)
                    .or_insert_with(GroupKnowledge::new)
                    .add_knowledge(transfer.knowledge.clone(), transfer.to);
            }
        }
    }
    
    pub fn get_group_exclusive_knowledge(&self, group: GroupId) -> Vec<Information> {
        let group_knowledge = &self.group_knowledge[&group];
        
        group_knowledge.shared_information
            .iter()
            .filter(|info| {
                // Check if other groups have this knowledge
                !self.group_knowledge
                    .iter()
                    .any(|(g, gk)| g != &group && gk.has_knowledge(info))
            })
            .cloned()
            .collect()
    }
}
```

---

## Quick Reference

### Conversation Topics Priority

1. **Danger Warning** (Urgency > 0.8)
2. **Critical Needs** (Hunger/Thirst < 20)
3. **Resource Sharing** (Group benefit)
4. **Skill Teaching** (Knowledge spread)
5. **Social Bonding** (Relationship building)
6. **Gossip** (Low priority)

### Key Parameters

```rust
// Conversation initiation
const MIN_CONVERSATION_DISTANCE: f32 = 3.0;
const MAX_CONVERSATION_DISTANCE: f32 = 5.0;
const CONVERSATION_COOLDOWN: f32 = 30.0; // seconds

// Information transfer
const BASE_RETENTION_RATE: f32 = 0.7;
const TRUST_MULTIPLIER: f32 = 1.5;
const DEGRADATION_PER_STEP: f32 = 0.9;

// Performance
const MAX_SIMULTANEOUS_CONVERSATIONS: usize = 50;
const CONVERSATION_UPDATE_RATE: f32 = 2.0; // Hz
const MAX_EXCHANGES_PER_CONVERSATION: usize = 10;
```

### Common Patterns

```rust
// Check if creatures can converse
fn can_converse(c1: &Creature, c2: &Creature) -> bool {
    let distance = c1.position.distance(c2.position);
    distance <= MAX_CONVERSATION_DISTANCE
        && c1.can_see(c2)
        && c2.can_see(c1)
        && !c1.is_in_conversation()
        && !c2.is_in_conversation()
        && c1.last_conversation_with(c2.id) + CONVERSATION_COOLDOWN < current_time()
}

// Quick outcome generation for distant conversations
fn generate_statistical_outcome(topic: &ConversationTopic) -> ConversationOutcome {
    match topic {
        ConversationTopic::FoodLocation { .. } => {
            ConversationOutcome {
                knowledge_transfers: vec![/* statistical transfer */],
                relationship_changes: vec![RelationshipChange::small_positive()],
                ..Default::default()
            }
        }
        _ => ConversationOutcome::default()
    }
}
```

### Performance Considerations

- Keep active conversations < 50
- Use LOD system aggressively
- Cache conversation partners
- Batch outcome processing
- Statistical outcomes for distant creatures

### Example Conversation Flow

**Scenario**: Hungry creature meets well-fed friend

1. **Proximity Detection**: Within 3 units
2. **Need Assessment**: Hunger < 20 (critical)
3. **Topic Selection**: FoodLocation (urgent)
4. **Dialogue Exchange**:
   - "Haven't eaten in so long..."
   - "I found berries near the big rock!"
   - "Where exactly?"
   - "Follow stream north, look for moss"
5. **Outcomes**:
   - Knowledge: Berry location gained
   - Relationship: Friendship +0.2
   - Decision: Goal → SatisfyHunger
   - Trust: Increases if info accurate

### Emergent Behaviors

Examples of emergent narratives:
- A creature lies about food location, destroying trust
- Warning about danger spreads rapidly, saving colony
- Teaching chains create skilled subpopulations
- Gossip networks form social hierarchies

### Performance Metrics

- **Target**: 50 simultaneous conversations
- **Update Rate**: 0.1-2Hz based on LOD
- **Memory Budget**: < 1KB per creature
- **Frame Impact**: < 2ms total