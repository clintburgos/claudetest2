# Memory & Knowledge Persistence Design

## Overview

The memory and knowledge system governs how creatures remember experiences, learn from interactions, and pass knowledge to future generations. This system creates emergent cultural behaviors and enables long-term learning across populations.

## Memory Architecture

### Memory Types
```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
enum Memory {
    Experience(ExperienceMemory),
    Social(SocialMemory),
    Location(LocationMemory),
    Learned(LearnedMemory),
    Inherited(InheritedMemory),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ExperienceMemory {
    memory_id: MemoryId,
    timestamp: SimTime,
    location: Vec2,
    emotion: Emotion,
    importance: f32, // 0.0-1.0
    details: ExperienceType,
    decay_rate: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
enum ExperienceType {
    FoundResource { resource_type: ResourceType, quality: f32 },
    Danger { threat_type: ThreatType, severity: f32 },
    Success { action: Action, outcome: f32 },
    Failure { action: Action, consequence: f32 },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct SocialMemory {
    memory_id: MemoryId,
    other_creature: CreatureId,
    interaction_type: InteractionType,
    timestamp: SimTime,
    trust_change: f32,
    conversation_topics: Vec<ConceptId>,
    emotional_tone: f32, // -1.0 to 1.0
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct LocationMemory {
    memory_id: MemoryId,
    location: Vec2,
    location_type: LocationType,
    last_visited: SimTime,
    visit_count: u32,
    associations: Vec<MemoryId>, // What happened here
    quality: f32, // How good/bad this place is
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct LearnedMemory {
    memory_id: MemoryId,
    concept: ConceptId,
    understanding: f32, // 0.0-1.0
    learned_from: LearningSource,
    applications: Vec<ApplicationRecord>,
    confidence: f32,
}
```

### Memory Storage Structure
```rust
struct MemorySystem {
    // Fast access for recent memories
    short_term: CircularBuffer<Memory>,
    short_term_capacity: usize, // 50
    
    // Long-term storage with importance-based retention
    long_term: BTreeMap<MemoryImportance, Memory>,
    long_term_capacity: usize, // 200
    
    // Specialized memory indices for fast queries
    spatial_memories: SpatialIndex<MemoryId>,
    social_memories: HashMap<CreatureId, Vec<MemoryId>>,
    concept_memories: HashMap<ConceptId, Vec<MemoryId>>,
    
    // Memory consolidation queue
    consolidation_queue: VecDeque<Memory>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct MemoryImportance {
    importance: OrderedFloat<f32>,
    timestamp: SimTime,
    memory_id: MemoryId,
}
```

## Memory Lifecycle

### Memory Formation
```rust
impl MemorySystem {
    fn form_memory(&mut self, experience: Experience) -> MemoryId {
        let importance = self.calculate_importance(&experience);
        
        let memory = match experience {
            Experience::ResourceFound { location, resource } => {
                Memory::Experience(ExperienceMemory {
                    memory_id: MemoryId::new(),
                    timestamp: self.current_time(),
                    location,
                    emotion: Emotion::Satisfaction,
                    importance,
                    details: ExperienceType::FoundResource {
                        resource_type: resource.resource_type,
                        quality: resource.quality,
                    },
                    decay_rate: self.calculate_decay_rate(importance),
                })
            },
            // ... other experience types
        };
        
        self.add_to_short_term(memory)
    }
    
    fn calculate_importance(&self, experience: &Experience) -> f32 {
        let base_importance = match experience {
            Experience::NearDeath => 1.0,
            Experience::FirstMeeting => 0.8,
            Experience::ResourceFound { resource } => {
                0.3 + (resource.quality * 0.4)
            },
            Experience::Conversation { emotional_impact } => {
                0.4 + (emotional_impact.abs() * 0.5)
            },
            _ => 0.5,
        };
        
        // Modify by current needs
        let need_modifier = self.calculate_need_relevance(experience);
        
        (base_importance * need_modifier).clamp(0.0, 1.0)
    }
}
```

### Memory Consolidation
```rust
impl MemorySystem {
    fn consolidate_memories(&mut self) {
        // Process short-term to long-term
        while self.short_term.len() > self.short_term_capacity {
            if let Some(memory) = self.short_term.pop_front() {
                self.consolidation_queue.push_back(memory);
            }
        }
        
        // Consolidate queued memories
        while let Some(memory) = self.consolidation_queue.pop_front() {
            let importance = self.get_importance(&memory);
            
            // Check if it should go to long-term
            if importance > self.get_minimum_importance() {
                self.add_to_long_term(memory);
            } else if self.should_compress(&memory) {
                self.compress_similar_memories(memory);
            }
        }
        
        // Maintain capacity limits
        self.prune_least_important();
    }
    
    fn compress_similar_memories(&mut self, new_memory: Memory) {
        // Find similar memories
        let similar = self.find_similar_memories(&new_memory);
        
        if similar.len() >= 3 {
            // Create compressed memory
            let compressed = self.create_pattern_memory(new_memory, similar);
            self.add_to_long_term(compressed);
            
            // Remove individual memories
            for memory_id in similar {
                self.remove_memory(memory_id);
            }
        }
    }
}
```

### Memory Decay
```rust
fn update_memory_decay(&mut self, delta_time: f32) {
    // Short-term memories decay faster
    for memory in &mut self.short_term {
        match memory {
            Memory::Experience(exp) => {
                exp.importance *= (1.0 - exp.decay_rate * delta_time);
            },
            // ... other memory types
        }
    }
    
    // Long-term memories decay slowly
    let decay_threshold = 0.1;
    let to_remove: Vec<_> = self.long_term
        .iter()
        .filter(|(imp, _)| imp.importance.0 < decay_threshold)
        .map(|(imp, _)| imp.clone())
        .collect();
        
    for importance in to_remove {
        self.long_term.remove(&importance);
    }
}

fn calculate_decay_rate(importance: f32, memory_type: &Memory) -> f32 {
    let base_decay = match memory_type {
        Memory::Experience(_) => 0.01,
        Memory::Social(_) => 0.005, // Social memories last longer
        Memory::Location(_) => 0.002, // Spatial memory very persistent
        Memory::Learned(_) => 0.001, // Knowledge decays slowly
        Memory::Inherited(_) => 0.0005, // Cultural memory very persistent
    };
    
    // Important memories decay slower
    base_decay * (1.0 - importance * 0.8)
}
```

## Knowledge Representation

### Concept System
```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Concept {
    concept_id: ConceptId,
    name: String,
    category: ConceptCategory,
    attributes: HashMap<String, f32>,
    relationships: Vec<ConceptRelation>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
enum ConceptCategory {
    Resource,
    Location,
    Creature,
    Action,
    Danger,
    Social,
    Abstract,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ConceptRelation {
    other_concept: ConceptId,
    relation_type: RelationType,
    strength: f32,
}

enum RelationType {
    IsA,      // Inheritance
    PartOf,   // Composition
    Causes,   // Causation
    Prevents, // Inhibition
    Near,     // Spatial
    Before,   // Temporal
}
```

### Knowledge Graph
```rust
struct KnowledgeGraph {
    concepts: HashMap<ConceptId, Concept>,
    edges: HashMap<ConceptId, Vec<Edge>>,
    
    // Fast lookup indices
    category_index: HashMap<ConceptCategory, HashSet<ConceptId>>,
    name_index: HashMap<String, ConceptId>,
}

struct Edge {
    from: ConceptId,
    to: ConceptId,
    relation: RelationType,
    weight: f32,
    learned_from: LearningSource,
}

impl KnowledgeGraph {
    fn add_learned_connection(
        &mut self,
        from: ConceptId,
        to: ConceptId,
        relation: RelationType,
        confidence: f32,
        source: LearningSource,
    ) {
        let edge = Edge {
            from,
            to,
            relation,
            weight: confidence,
            learned_from: source,
        };
        
        self.edges.entry(from)
            .or_insert_with(Vec::new)
            .push(edge);
            
        // Propagate learning through graph
        self.propagate_knowledge(from, confidence * 0.5);
    }
    
    fn query_knowledge(&self, concept: ConceptId) -> KnowledgeContext {
        let mut context = KnowledgeContext::new();
        
        // Direct connections
        if let Some(edges) = self.edges.get(&concept) {
            for edge in edges {
                context.add_relation(edge);
            }
        }
        
        // Indirect connections (2 hops)
        for (_, edges) in &self.edges {
            for edge in edges {
                if edge.to == concept {
                    context.add_incoming(edge);
                }
            }
        }
        
        context
    }
}
```

## Memory Retrieval

### Associative Retrieval
```rust
impl MemorySystem {
    fn retrieve_by_association(&self, trigger: MemoryTrigger) -> Vec<Memory> {
        match trigger {
            MemoryTrigger::Location(pos) => {
                self.spatial_memories
                    .query_radius(pos, 50.0)
                    .map(|id| self.get_memory(id))
                    .collect()
            },
            MemoryTrigger::Creature(id) => {
                self.social_memories
                    .get(&id)
                    .map(|ids| ids.iter()
                        .filter_map(|id| self.get_memory(*id))
                        .collect())
                    .unwrap_or_default()
            },
            MemoryTrigger::Concept(concept) => {
                self.retrieve_conceptual_memories(concept)
            },
            MemoryTrigger::Emotion(emotion) => {
                self.retrieve_emotional_memories(emotion)
            },
        }
    }
    
    fn retrieve_conceptual_memories(&self, concept: ConceptId) -> Vec<Memory> {
        let mut memories = Vec::new();
        
        // Direct concept memories
        if let Some(ids) = self.concept_memories.get(&concept) {
            memories.extend(ids.iter().filter_map(|id| self.get_memory(*id)));
        }
        
        // Related concepts through knowledge graph
        let related = self.knowledge.get_related_concepts(concept, 2);
        for (related_concept, strength) in related {
            if let Some(ids) = self.concept_memories.get(&related_concept) {
                let related_memories: Vec<_> = ids.iter()
                    .filter_map(|id| self.get_memory(*id))
                    .collect();
                    
                // Weight by relationship strength
                for mut memory in related_memories {
                    memory.adjust_relevance(strength);
                    memories.push(memory);
                }
            }
        }
        
        // Sort by relevance
        memories.sort_by(|a, b| b.relevance().partial_cmp(&a.relevance()).unwrap());
        memories.truncate(10); // Limit retrieval
        
        memories
    }
}
```

## Knowledge Inheritance

### Genetic Knowledge
```rust
#[derive(Clone, Serialize, Deserialize)]
struct GeneticKnowledge {
    // Instinctual knowledge encoded in genes
    innate_concepts: Vec<InnateConceptId>,
    fear_responses: HashMap<ConceptId, f32>,
    preference_biases: HashMap<ConceptId, f32>,
}

impl GeneticKnowledge {
    fn express(&self, development_stage: f32) -> ActiveKnowledge {
        ActiveKnowledge {
            concepts: self.innate_concepts
                .iter()
                .map(|id| (*id, development_stage))
                .collect(),
            fears: self.fear_responses
                .iter()
                .map(|(k, v)| (*k, v * development_stage))
                .collect(),
            preferences: self.preference_biases
                .iter()
                .map(|(k, v)| (*k, v * development_stage))
                .collect(),
        }
    }
}
```

### Cultural Transmission
```rust
struct CulturalKnowledge {
    shared_concepts: HashMap<ConceptId, CulturalConcept>,
    transmission_rules: Vec<TransmissionRule>,
    mutation_rate: f32,
}

#[derive(Clone)]
struct CulturalConcept {
    base_concept: ConceptId,
    variations: Vec<ConceptVariation>,
    prevalence: f32, // How common in population
    fidelity: f32,   // How accurately transmitted
}

struct TransmissionRule {
    trigger: TransmissionTrigger,
    concepts: Vec<ConceptId>,
    required_relationship: f32, // Minimum trust/familiarity
    success_rate: f32,
}

enum TransmissionTrigger {
    Conversation,
    Observation,
    Teaching,
    GroupActivity,
}

impl CulturalKnowledge {
    fn transmit(
        &self,
        from: &Creature,
        to: &mut Creature,
        trigger: TransmissionTrigger,
    ) -> Vec<ConceptId> {
        let relationship = from.get_relationship(to.id);
        let mut transmitted = Vec::new();
        
        for rule in &self.transmission_rules {
            if rule.trigger == trigger && relationship >= rule.required_relationship {
                for concept_id in &rule.concepts {
                    if rand::random::<f32>() < rule.success_rate {
                        let concept = self.mutate_concept(concept_id);
                        to.learn_concept(concept);
                        transmitted.push(*concept_id);
                    }
                }
            }
        }
        
        transmitted
    }
    
    fn mutate_concept(&self, concept_id: &ConceptId) -> Concept {
        if let Some(cultural) = self.shared_concepts.get(concept_id) {
            if rand::random::<f32>() < self.mutation_rate {
                // Create variation
                let variation = self.create_variation(&cultural.base_concept);
                variation
            } else {
                // Faithful transmission
                self.get_concept(cultural.base_concept).clone()
            }
        } else {
            self.get_concept(*concept_id).clone()
        }
    }
}
```

## Memory Queries

### Contextual Memory Search
```rust
struct MemoryQuery {
    context: QueryContext,
    filters: Vec<MemoryFilter>,
    sort_by: SortCriteria,
    limit: usize,
}

enum QueryContext {
    CurrentSituation {
        location: Vec2,
        needs: NeedState,
        nearby_creatures: Vec<CreatureId>,
    },
    ProblemSolving {
        goal: Goal,
        constraints: Vec<Constraint>,
    },
    Social {
        other_creature: CreatureId,
        relationship: f32,
        topic: Option<ConceptId>,
    },
}

enum MemoryFilter {
    TimeRange(SimTime, SimTime),
    LocationRadius(Vec2, f32),
    EmotionType(Emotion),
    ConceptRelated(ConceptId),
    ImportanceThreshold(f32),
}

impl MemorySystem {
    fn query(&self, query: MemoryQuery) -> Vec<Memory> {
        let mut results = self.get_relevant_memories(&query.context);
        
        // Apply filters
        for filter in &query.filters {
            results = self.apply_filter(results, filter);
        }
        
        // Sort results
        self.sort_memories(&mut results, &query.sort_by);
        
        // Limit results
        results.truncate(query.limit);
        
        results
    }
}
```

## Persistence Strategy

### Serialization Format
```rust
#[derive(Serialize, Deserialize)]
struct SerializedMemory {
    version: u16,
    creature_id: CreatureId,
    
    // Compressed memories
    short_term: Vec<CompressedMemory>,
    long_term: Vec<CompressedMemory>,
    
    // Knowledge graph as adjacency list
    concepts: Vec<(ConceptId, String)>,
    edges: Vec<(ConceptId, ConceptId, RelationType, f32)>,
    
    // Statistics for reconstruction
    total_memories_formed: u64,
    memory_compression_ratio: f32,
}

#[derive(Serialize, Deserialize)]
struct CompressedMemory {
    memory_type: u8, // Enum discriminant
    data: Vec<u8>,  // Bincode serialized
    importance: f32,
    timestamp: u32, // Relative to world time
}
```

### Memory Reconstruction
```rust
impl MemorySystem {
    fn save_compressed(&self) -> SerializedMemory {
        SerializedMemory {
            version: MEMORY_VERSION,
            creature_id: self.creature_id,
            short_term: self.compress_memories(&self.short_term),
            long_term: self.compress_memories(&self.long_term.values()),
            concepts: self.knowledge.export_concepts(),
            edges: self.knowledge.export_edges(),
            total_memories_formed: self.statistics.total_formed,
            memory_compression_ratio: self.calculate_compression_ratio(),
        }
    }
    
    fn load_compressed(data: SerializedMemory) -> Result<Self> {
        let mut system = MemorySystem::new(data.creature_id);
        
        // Reconstruct memories
        for compressed in data.short_term {
            system.short_term.push(decompress_memory(compressed)?);
        }
        
        for compressed in data.long_term {
            let memory = decompress_memory(compressed)?;
            system.add_to_long_term(memory);
        }
        
        // Reconstruct knowledge graph
        system.knowledge.import_concepts(data.concepts);
        system.knowledge.import_edges(data.edges);
        
        // Rebuild indices
        system.rebuild_indices();
        
        Ok(system)
    }
}
```