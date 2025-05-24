# Conversation Outcome Mechanics Design

## Overview

The conversation outcome mechanics system determines how conversations between creatures affect their knowledge, relationships, behaviors, and the spread of information throughout the population. This creates emergent cultural behaviors and information networks.

## Trust Calculation

### Trust Model
```rust
struct TrustSystem {
    base_trust_factors: TrustFactors,
    trust_history: HashMap<(CreatureId, CreatureId), TrustHistory>,
    trust_decay_rate: f32,
}

struct TrustFactors {
    // Initial trust modifiers
    species_similarity: f32,     // 0.0-1.0
    personality_compatibility: f32, // -1.0-1.0
    appearance_factor: f32,      // 0.5-1.5
    reputation_weight: f32,      // 0.0-2.0
    
    // Interaction modifiers
    successful_cooperation: f32, // +0.1 per success
    failed_cooperation: f32,     // -0.2 per failure
    information_accuracy: f32,   // -0.5 to +0.5
    reciprocity_bonus: f32,     // +0.1 for balanced exchanges
}

struct TrustHistory {
    current_trust: f32,
    peak_trust: f32,
    interactions: VecDeque<TrustInteraction>,
    last_interaction: SimTime,
    
    // Tracking
    promises_made: u32,
    promises_kept: u32,
    information_shared: u32,
    information_accuracy_rate: f32,
}

impl TrustSystem {
    fn calculate_trust(
        &self,
        creature_a: &Creature,
        creature_b: &Creature,
    ) -> f32 {
        let history_key = (creature_a.id, creature_b.id);
        
        if let Some(history) = self.trust_history.get(&history_key) {
            // Existing relationship
            let time_since_interaction = current_time() - history.last_interaction;
            let decay = time_since_interaction.as_hours() as f32 * self.trust_decay_rate;
            
            (history.current_trust - decay).max(0.0)
        } else {
            // New relationship - calculate initial trust
            self.calculate_initial_trust(creature_a, creature_b)
        }
    }
    
    fn calculate_initial_trust(
        &self,
        creature_a: &Creature,
        creature_b: &Creature,
    ) -> f32 {
        let factors = &self.base_trust_factors;
        
        // Species similarity
        let species_trust = if creature_a.species == creature_b.species {
            factors.species_similarity
        } else {
            factors.species_similarity * 0.5
        };
        
        // Personality compatibility
        let personality_diff = (creature_a.personality - creature_b.personality).magnitude();
        let personality_trust = factors.personality_compatibility * 
            (1.0 - personality_diff);
        
        // Reputation from others
        let reputation = self.get_reputation_score(creature_b.id);
        let reputation_trust = reputation * factors.reputation_weight;
        
        // Appearance (size difference, health)
        let size_ratio = creature_a.size / creature_b.size;
        let appearance_trust = if size_ratio > 2.0 || size_ratio < 0.5 {
            factors.appearance_factor * 0.7 // Intimidation
        } else {
            factors.appearance_factor
        };
        
        let base_trust = 0.3; // Neutral starting point
        (base_trust + species_trust + personality_trust + 
         reputation_trust + appearance_trust).clamp(0.0, 1.0)
    }
    
    fn update_trust(
        &mut self,
        creature_a_id: CreatureId,
        creature_b_id: CreatureId,
        interaction: TrustInteraction,
    ) {
        let history = self.trust_history
            .entry((creature_a_id, creature_b_id))
            .or_insert_with(|| TrustHistory::new());
        
        // Apply trust change
        match interaction.outcome {
            InteractionOutcome::Success => {
                history.current_trust += self.base_trust_factors.successful_cooperation;
            },
            InteractionOutcome::Failure => {
                history.current_trust += self.base_trust_factors.failed_cooperation;
            },
            InteractionOutcome::Deception => {
                history.current_trust -= 0.3; // Major trust breach
            },
            InteractionOutcome::SharedAccurateInfo => {
                history.current_trust += self.base_trust_factors.information_accuracy;
                history.information_accuracy_rate = 
                    (history.information_accuracy_rate * history.information_shared as f32 + 1.0) /
                    (history.information_shared + 1) as f32;
            },
            InteractionOutcome::SharedInaccurateInfo => {
                history.current_trust -= self.base_trust_factors.information_accuracy.abs();
                history.information_accuracy_rate = 
                    (history.information_accuracy_rate * history.information_shared as f32) /
                    (history.information_shared + 1) as f32;
            },
        }
        
        history.current_trust = history.current_trust.clamp(0.0, 1.0);
        history.peak_trust = history.peak_trust.max(history.current_trust);
        history.last_interaction = current_time();
        history.interactions.push_back(interaction);
        
        // Maintain history size
        if history.interactions.len() > MAX_INTERACTION_HISTORY {
            history.interactions.pop_front();
        }
    }
}
```

## Information Accuracy

### Information Degradation
```rust
struct InformationAccuracy {
    initial_accuracy: f32,
    degradation_model: DegradationModel,
    
    // Factors affecting accuracy
    transmission_noise: f32,      // 0.01-0.1 per transmission
    time_decay: f32,             // 0.001 per hour
    cognitive_distortion: f32,    // Based on creature intelligence
    emotional_bias: f32,         // Based on emotional state
}

#[derive(Clone)]
struct Information {
    info_id: InformationId,
    content: InformationContent,
    accuracy: f32,
    source_chain: Vec<CreatureId>,
    mutations: Vec<Mutation>,
    timestamp: SimTime,
}

#[derive(Clone)]
enum InformationContent {
    ResourceLocation { 
        location: Vec2, 
        resource_type: ResourceType,
        amount_estimate: f32,
    },
    DangerWarning {
        threat_type: ThreatType,
        location: Vec2,
        severity: f32,
    },
    SocialGossip {
        subject: CreatureId,
        reputation_change: f32,
        event_type: GossipType,
    },
    CulturalKnowledge {
        concept: ConceptId,
        technique: TechniqueDescription,
        effectiveness: f32,
    },
}

impl InformationAccuracy {
    fn degrade_information(
        &self,
        info: &mut Information,
        transmitter: &Creature,
        receiver: &Creature,
    ) {
        // Base transmission noise
        let noise = rand_normal(0.0, self.transmission_noise);
        info.accuracy *= (1.0 - noise.abs());
        
        // Time decay
        let age = (current_time() - info.timestamp).as_hours() as f32;
        info.accuracy *= (1.0 - self.time_decay * age);
        
        // Cognitive distortion based on intelligence
        let avg_intelligence = (transmitter.intelligence + receiver.intelligence) / 2.0;
        let cognitive_factor = 0.5 + avg_intelligence * 0.5; // 0.5-1.0
        info.accuracy *= cognitive_factor;
        
        // Emotional bias
        let emotional_state = transmitter.get_emotional_state();
        if emotional_state.arousal > 0.8 {
            info.accuracy *= (1.0 - self.emotional_bias);
            
            // May add emotional coloring to information
            if rand::random::<f32>() < 0.3 {
                info.mutations.push(Mutation::EmotionalExaggeration {
                    emotion: emotional_state.dominant_emotion,
                    magnitude: emotional_state.arousal,
                });
            }
        }
        
        // Add transmitter to chain
        info.source_chain.push(transmitter.id);
        
        // Apply content-specific degradation
        match &mut info.content {
            InformationContent::ResourceLocation { location, amount_estimate, .. } => {
                // Location becomes less precise
                let drift = Vec2::random_in_circle(10.0 * (1.0 - info.accuracy));
                *location += drift;
                
                // Amount estimate becomes less accurate
                *amount_estimate *= rand_range(
                    0.5 + info.accuracy * 0.5,
                    1.5 - info.accuracy * 0.5
                );
            },
            InformationContent::SocialGossip { reputation_change, .. } => {
                // Gossip tends to become more extreme
                if reputation_change.abs() > 0.1 {
                    *reputation_change *= 1.0 + (1.0 - info.accuracy) * 0.5;
                }
            },
            _ => {},
        }
        
        info.accuracy = info.accuracy.clamp(0.0, 1.0);
    }
}
```

## Influence Spread

### Social Influence Network
```rust
struct InfluenceNetwork {
    influence_map: HashMap<CreatureId, InfluenceNode>,
    propagation_rules: PropagationRules,
    cultural_clusters: Vec<CulturalCluster>,
}

struct InfluenceNode {
    creature_id: CreatureId,
    influence_score: f32,
    
    // Connections
    followers: HashSet<CreatureId>,
    following: HashSet<CreatureId>,
    influence_strength: HashMap<CreatureId, f32>,
    
    // Influence domains
    expertise_areas: HashMap<ConceptCategory, f32>,
    credibility_scores: HashMap<InformationType, f32>,
}

struct PropagationRules {
    min_trust_for_influence: f32,
    influence_decay_rate: f32,
    max_influence_hops: u32,
    
    // Modifiers
    group_amplification: f32,    // Influence spreads faster in groups
    expertise_multiplier: f32,   // Experts have more influence
    novelty_bonus: f32,         // New information spreads faster
}

impl InfluenceNetwork {
    fn propagate_influence(
        &mut self,
        originator: CreatureId,
        information: Information,
        initial_recipients: Vec<CreatureId>,
    ) -> PropagationResult {
        let mut influenced = HashSet::new();
        let mut propagation_queue = VecDeque::new();
        
        // Initialize with direct recipients
        for recipient in initial_recipients {
            propagation_queue.push_back(PropagationStep {
                from: originator,
                to: recipient,
                information: information.clone(),
                hop_count: 0,
            });
        }
        
        while let Some(step) = propagation_queue.pop_front() {
            if step.hop_count >= self.propagation_rules.max_influence_hops {
                continue;
            }
            
            // Check if influence succeeds
            if self.attempt_influence(&step) {
                influenced.insert(step.to);
                
                // Get node's followers for further propagation
                if let Some(node) = self.influence_map.get(&step.to) {
                    for &follower in &node.followers {
                        if !influenced.contains(&follower) {
                            let mut next_info = step.information.clone();
                            
                            // Degrade information
                            self.degrade_for_propagation(&mut next_info, step.hop_count);
                            
                            propagation_queue.push_back(PropagationStep {
                                from: step.to,
                                to: follower,
                                information: next_info,
                                hop_count: step.hop_count + 1,
                            });
                        }
                    }
                }
            }
        }
        
        PropagationResult {
            total_influenced: influenced.len(),
            reach_by_hop: self.calculate_reach_by_hop(&influenced),
            final_accuracy: self.calculate_average_accuracy(&influenced),
        }
    }
    
    fn attempt_influence(&self, step: &PropagationStep) -> bool {
        let from_node = &self.influence_map[&step.from];
        let to_node = &self.influence_map[&step.to];
        
        // Base influence probability
        let influence_strength = from_node.influence_strength
            .get(&step.to)
            .unwrap_or(&0.0);
            
        if influence_strength < self.propagation_rules.min_trust_for_influence {
            return false;
        }
        
        // Expertise modifier
        let expertise = match &step.information.content {
            InformationContent::ResourceLocation { .. } => {
                from_node.expertise_areas.get(&ConceptCategory::Resources)
            },
            InformationContent::DangerWarning { .. } => {
                from_node.expertise_areas.get(&ConceptCategory::Dangers)
            },
            _ => None,
        }.unwrap_or(&0.5);
        
        let expertise_factor = 0.5 + expertise * self.propagation_rules.expertise_multiplier;
        
        // Novelty factor
        let is_novel = !to_node.has_similar_information(&step.information);
        let novelty_factor = if is_novel { 
            1.0 + self.propagation_rules.novelty_bonus 
        } else { 
            1.0 
        };
        
        // Calculate final probability
        let probability = influence_strength * expertise_factor * novelty_factor * 
            step.information.accuracy;
            
        rand::random::<f32>() < probability
    }
}
```

## Gossip Mechanics

### Gossip System
```rust
struct GossipSystem {
    gossip_pool: HashMap<CreatureId, Vec<Gossip>>,
    reputation_effects: HashMap<CreatureId, ReputationState>,
    gossip_rules: GossipRules,
}

#[derive(Clone)]
struct Gossip {
    subject: CreatureId,
    gossip_type: GossipType,
    origin_time: SimTime,
    spread_count: u32,
    veracity: f32, // How true it is
    juiciness: f32, // How interesting it is
    modifications: Vec<GossipModification>,
}

#[derive(Clone)]
enum GossipType {
    Heroic { deed: String },
    Scandalous { misdeed: String },
    Romantic { partner: CreatureId },
    Failure { context: String },
    Success { achievement: String },
    Secret { revelation: String },
}

struct GossipRules {
    spread_probability_base: f32,
    juiciness_multiplier: f32,
    veracity_decay: f32,
    max_modifications: u32,
    reputation_impact_curve: Curve,
}

impl GossipSystem {
    fn generate_gossip(
        &mut self,
        observer: &Creature,
        event: &ObservedEvent,
    ) -> Option<Gossip> {
        let gossip_type = match event {
            ObservedEvent::CreatureSuccess { creature_id, action } => {
                if action.is_noteworthy() {
                    Some(GossipType::Success { 
                        achievement: action.description() 
                    })
                } else {
                    None
                }
            },
            ObservedEvent::CreatureFailure { creature_id, action, consequence } => {
                if consequence.is_embarrassing() {
                    Some(GossipType::Failure { 
                        context: action.description() 
                    })
                } else {
                    None
                }
            },
            ObservedEvent::UnusualBehavior { creature_id, behavior } => {
                Some(GossipType::Scandalous { 
                    misdeed: behavior.description() 
                })
            },
            _ => None,
        }?;
        
        let juiciness = self.calculate_juiciness(&gossip_type, observer);
        
        Some(Gossip {
            subject: event.get_subject(),
            gossip_type,
            origin_time: current_time(),
            spread_count: 0,
            veracity: 0.8 + rand::random::<f32>() * 0.2, // 80-100% accurate initially
            juiciness,
            modifications: Vec::new(),
        })
    }
    
    fn spread_gossip(
        &mut self,
        gossip: &Gossip,
        from: &Creature,
        to: &Creature,
    ) -> Option<Gossip> {
        // Check if creature is interested
        let interest_level = self.calculate_interest(gossip, to);
        let spread_chance = self.gossip_rules.spread_probability_base * 
            interest_level * gossip.juiciness;
            
        if rand::random::<f32>() > spread_chance {
            return None;
        }
        
        // Modify gossip during transmission
        let mut modified = gossip.clone();
        modified.spread_count += 1;
        
        // Decay veracity
        modified.veracity *= (1.0 - self.gossip_rules.veracity_decay);
        
        // Potentially modify content
        if rand::random::<f32>() < 0.3 && 
           modified.modifications.len() < self.gossip_rules.max_modifications as usize {
            let modification = self.generate_modification(&modified.gossip_type, from, to);
            modified.modifications.push(modification);
            
            // Modifications make gossip juicier but less accurate
            modified.juiciness *= 1.1;
            modified.veracity *= 0.8;
        }
        
        // Update reputation
        self.apply_reputation_effect(&modified);
        
        Some(modified)
    }
    
    fn apply_reputation_effect(&mut self, gossip: &Gossip) {
        let reputation = self.reputation_effects
            .entry(gossip.subject)
            .or_insert_with(ReputationState::new);
            
        let impact = match &gossip.gossip_type {
            GossipType::Heroic { .. } => gossip.veracity * 0.5,
            GossipType::Scandalous { .. } => -gossip.veracity * 0.7,
            GossipType::Success { .. } => gossip.veracity * 0.3,
            GossipType::Failure { .. } => -gossip.veracity * 0.4,
            _ => 0.0,
        };
        
        // Apply with diminishing returns for repeated gossip
        let familiarity_factor = 1.0 / (1.0 + gossip.spread_count as f32 * 0.1);
        reputation.apply_change(impact * familiarity_factor);
    }
}
```

## Conversation Outcomes

### Outcome Determination
```rust
struct ConversationOutcome {
    outcome_type: OutcomeType,
    participants: Vec<CreatureId>,
    
    // Effects
    relationship_changes: HashMap<(CreatureId, CreatureId), f32>,
    knowledge_transfers: Vec<KnowledgeTransfer>,
    behavior_influences: Vec<BehaviorInfluence>,
    emotional_changes: HashMap<CreatureId, EmotionalChange>,
    
    // Metrics
    success_level: f32,
    information_quality: f32,
}

enum OutcomeType {
    MutualUnderstanding,
    Misunderstanding,
    Agreement,
    Disagreement,
    Teaching,
    Deception,
    Bonding,
    Conflict,
}

impl ConversationOutcomeCalculator {
    fn calculate_outcome(
        &self,
        conversation: &Conversation,
        participants: &[Creature],
    ) -> ConversationOutcome {
        let mut outcome = ConversationOutcome::new();
        
        // Analyze conversation flow
        let topic_alignment = self.analyze_topic_alignment(&conversation.topics);
        let emotional_sync = self.analyze_emotional_synchrony(participants);
        let trust_levels = self.get_mutual_trust_levels(participants);
        
        // Determine primary outcome type
        outcome.outcome_type = match (topic_alignment, emotional_sync, trust_levels) {
            (a, e, t) if a > 0.8 && e > 0.7 && t > 0.6 => OutcomeType::MutualUnderstanding,
            (a, e, t) if a < 0.3 || e < 0.3 => OutcomeType::Misunderstanding,
            (a, _, t) if a > 0.7 && t > 0.5 => OutcomeType::Agreement,
            (a, _, t) if a < 0.4 && t < 0.4 => OutcomeType::Disagreement,
            _ => OutcomeType::MutualUnderstanding,
        };
        
        // Calculate relationship changes
        for i in 0..participants.len() {
            for j in i+1..participants.len() {
                let change = self.calculate_relationship_change(
                    &participants[i],
                    &participants[j],
                    &outcome.outcome_type,
                    emotional_sync,
                );
                outcome.relationship_changes.insert(
                    (participants[i].id, participants[j].id),
                    change
                );
            }
        }
        
        // Process knowledge transfers
        outcome.knowledge_transfers = self.process_knowledge_transfers(
            conversation,
            participants,
            trust_levels,
        );
        
        // Calculate behavior influences
        outcome.behavior_influences = self.calculate_behavior_influences(
            conversation,
            participants,
            &outcome.knowledge_transfers,
        );
        
        // Emotional contagion
        outcome.emotional_changes = self.calculate_emotional_contagion(
            participants,
            emotional_sync,
        );
        
        outcome
    }
    
    fn process_knowledge_transfers(
        &self,
        conversation: &Conversation,
        participants: &[Creature],
        trust_levels: f32,
    ) -> Vec<KnowledgeTransfer> {
        let mut transfers = Vec::new();
        
        for exchange in &conversation.exchanges {
            if let ConversationContent::Information(info) = &exchange.content {
                // Determine if transfer succeeds
                let transfer_chance = trust_levels * info.clarity * 
                    participants[exchange.recipient].learning_ability;
                    
                if rand::random::<f32>() < transfer_chance {
                    transfers.push(KnowledgeTransfer {
                        from: exchange.speaker,
                        to: exchange.recipient,
                        knowledge: info.knowledge.clone(),
                        accuracy: info.accuracy * trust_levels,
                        integration_time: self.estimate_integration_time(
                            &info.knowledge,
                            &participants[exchange.recipient]
                        ),
                    });
                }
            }
        }
        
        transfers
    }
}
```

## Cultural Impact

### Meme Propagation
```rust
struct CulturalMeme {
    meme_id: MemeId,
    concept: ConceptId,
    expression: MemeExpression,
    
    // Propagation properties
    virality: f32,        // How easily it spreads
    stickiness: f32,      // How well it's remembered
    mutability: f32,      // How much it changes
    utility: f32,         // Practical value
    
    // Evolution tracking
    variant_tree: MemeVariantTree,
    adoption_count: u32,
    average_lifetime: Duration,
}

struct MemeExpression {
    behaviors: Vec<BehaviorPattern>,
    phrases: Vec<String>,
    techniques: Vec<Technique>,
    beliefs: Vec<Belief>,
}

impl CulturalMemeSystem {
    fn evaluate_meme_fitness(
        &self,
        meme: &CulturalMeme,
        population: &Population,
    ) -> f32 {
        let adoption_rate = meme.adoption_count as f32 / population.size() as f32;
        let retention_rate = self.calculate_retention_rate(meme);
        let mutation_success = self.evaluate_successful_variants(meme);
        
        // Fitness combines spread, retention, and adaptation
        adoption_rate * 0.4 + retention_rate * 0.4 + mutation_success * 0.2
    }
    
    fn mutate_meme(
        &self,
        original: &CulturalMeme,
        mutator: &Creature,
    ) -> CulturalMeme {
        let mut variant = original.clone();
        variant.meme_id = MemeId::new();
        
        // Mutation influenced by creature's creativity and understanding
        let mutation_strength = mutator.creativity * original.mutability;
        
        if rand::random::<f32>() < mutation_strength {
            // Modify behaviors
            for behavior in &mut variant.expression.behaviors {
                behavior.apply_variation(mutator.personality);
            }
            
            // Modify techniques based on experience
            for technique in &mut variant.expression.techniques {
                if let Some(improvement) = mutator.improve_technique(technique) {
                    *technique = improvement;
                    variant.utility *= 1.1; // Improvements increase utility
                }
            }
        }
        
        // Track lineage
        variant.variant_tree.add_branch(original.meme_id, variant.meme_id);
        
        variant
    }
}
```