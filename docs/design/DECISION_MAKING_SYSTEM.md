# Creature Decision Making System

## Overview
The decision-making system is the brain of each creature, transforming needs, environment perception, and social influences into concrete actions. It uses a hierarchical, utility-based AI architecture optimized for performance at scale.

## Architecture

### Core Concepts

```
Input Layer:
┌─────────────┐ ┌─────────────┐ ┌─────────────┐
│    Needs    │ │ Environment │ │   Social    │
│  (Internal) │ │ (Perception)│ │ (Relations) │
└──────┬──────┘ └──────┬──────┘ └──────┬──────┘
       │               │               │
       └───────────────┴───────────────┘
                       │
                  ┌────▼────┐
                  │ Utility  │
                  │  Scorer  │
                  └────┬────┘
                       │
              ┌────────▼────────┐
              │ Decision Tree   │
              │ (Hierarchical)  │
              └────────┬────────┘
                       │
                 ┌─────▼─────┐
                 │  Actions   │
                 │ (Concrete) │
                 └───────────┘
```

### Component Structure

```rust
// Core decision-making components
#[derive(Component, Debug, Clone)]
pub struct DecisionState {
    current_goal: Goal,
    current_action: Action,
    decision_cooldown: f32,
    last_decision_time: f64,
}

#[derive(Component, Debug)]
pub struct UtilityScores {
    scores: HashMap<Goal, f32>,
    dirty: bool,
}

#[derive(Component, Debug)]
pub struct ActionPlan {
    steps: VecDeque<ActionStep>,
    current_step: usize,
    plan_validity: f32,
}

#[derive(Component)]
pub struct DecisionContext {
    // Cached perception data
    nearby_creatures: Vec<(Entity, f32)>, // (entity, distance)
    nearby_resources: Vec<(ResourceType, Vec2, f32)>, // (type, position, amount)
    environmental_factors: EnvironmentData,
    
    // Cached for performance
    last_update: f64,
    update_frequency: f32,
}
```

## Utility-Based Decision Making

### Utility Calculation

```rust
pub trait UtilityEvaluator: Send + Sync {
    fn evaluate(&self, context: &DecisionContext, needs: &Needs) -> f32;
    fn requirements_met(&self, context: &DecisionContext) -> bool;
}

// Example evaluators
pub struct HungerUtility;

impl UtilityEvaluator for HungerUtility {
    fn evaluate(&self, context: &DecisionContext, needs: &Needs) -> f32 {
        // Base utility from need
        let base = (100.0 - needs.hunger) / 100.0;
        
        // Modify based on food availability
        let food_modifier = if context.nearby_resources.iter()
            .any(|(t, _, _)| *t == ResourceType::Food) {
            1.2 // Boost if food is nearby
        } else {
            0.8 // Reduce if no food visible
        };
        
        // Urgency curve - exponential as hunger increases
        let urgency = (base * 2.0).exp() - 1.0;
        
        urgency * food_modifier
    }
    
    fn requirements_met(&self, context: &DecisionContext) -> bool {
        // Can always evaluate hunger
        true
    }
}

pub struct SocialUtility;

impl UtilityEvaluator for SocialUtility {
    fn evaluate(&self, context: &DecisionContext, needs: &Needs) -> f32 {
        let base = (100.0 - needs.social) / 100.0;
        
        // Boost if friendly creatures nearby
        let social_modifier = context.nearby_creatures.len() as f32 * 0.1;
        
        base * (1.0 + social_modifier).min(2.0)
    }
    
    fn requirements_met(&self, context: &DecisionContext) -> bool {
        // Only if other creatures are nearby
        !context.nearby_creatures.is_empty()
    }
}
```

### Goal System

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Goal {
    SatisfyHunger,
    SatisfyThirst,
    Socialize,
    Rest,
    Explore,
    Reproduce,
    FleeFromDanger,
    SeekShelter,
}

impl Goal {
    pub fn get_evaluator(&self) -> Box<dyn UtilityEvaluator> {
        match self {
            Goal::SatisfyHunger => Box::new(HungerUtility),
            Goal::SatisfyThirst => Box::new(ThirstUtility),
            Goal::Socialize => Box::new(SocialUtility),
            Goal::Rest => Box::new(RestUtility),
            Goal::Explore => Box::new(ExploreUtility),
            Goal::Reproduce => Box::new(ReproductionUtility),
            Goal::FleeFromDanger => Box::new(DangerUtility),
            Goal::SeekShelter => Box::new(ShelterUtility),
        }
    }
    
    pub fn priority(&self) -> u8 {
        match self {
            Goal::FleeFromDanger => 100,  // Highest priority
            Goal::SatisfyThirst => 90,
            Goal::SatisfyHunger => 80,
            Goal::SeekShelter => 70,
            Goal::Rest => 60,
            Goal::Reproduce => 50,
            Goal::Socialize => 40,
            Goal::Explore => 30,          // Lowest priority
        }
    }
}
```
## Action System

### Action Hierarchy

```rust
#[derive(Debug, Clone)]
pub enum Action {
    // Movement actions
    MoveTo { target: Vec2, speed: MoveSpeed },
    Wander { direction: Vec2, duration: f32 },
    FleeFrom { threat: Vec2, urgency: f32 },
    
    // Resource actions
    Eat { food_entity: Entity },
    Drink { water_position: Vec2 },
    GatherResource { resource: Entity, resource_type: ResourceType },
    
    // Social actions
    ApproachCreature { target: Entity },
    InitiateConversation { target: Entity, topic: ConversationTopic },
    ShareInformation { target: Entity, info: Information },
    
    // Biological actions
    Rest { duration: f32 },
    Reproduce { partner: Entity },
    
    // Complex actions (decomposed into steps)
    SearchFor { resource_type: ResourceType, search_radius: f32 },
    BuildShelter { location: Vec2 },
}

#[derive(Debug, Clone, Copy)]
pub enum MoveSpeed {
    Slow = 1,
    Normal = 2,
    Fast = 3,
    Sprint = 4,
}

// Action steps for complex behaviors
#[derive(Debug, Clone)]
pub struct ActionStep {
    pub action: Action,
    pub completion_condition: CompletionCondition,
    pub timeout: f32,
}

#[derive(Debug, Clone)]
pub enum CompletionCondition {
    ReachedPosition { tolerance: f32 },
    TimeElapsed { seconds: f32 },
    NeedSatisfied { need_type: NeedType, threshold: f32 },
    CustomCondition(fn(&World, Entity) -> bool),
}
```

### Action Execution

```rust
pub fn execute_action_system(
    mut commands: Commands,
    time: Res<Time>,
    mut creatures: Query<(
        Entity,
        &mut DecisionState,
        &mut ActionPlan,
        &mut Velocity,
        &Position,
        &mut Needs,
    )>,
    resources: Query<&Resource>,
    spatial_index: Res<SpatialIndex>,
) {
    for (entity, mut decision, mut plan, mut velocity, position, mut needs) in &mut creatures {
        // Execute current action
        match &decision.current_action {
            Action::MoveTo { target, speed } => {
                execute_move_to(
                    &mut velocity,
                    position,
                    *target,
                    *speed,
                    &time,
                );
            }
            
            Action::Eat { food_entity } => {
                if let Ok(resource) = resources.get(*food_entity) {
                    needs.hunger = (needs.hunger + resource.nutrition).min(100.0);
                    commands.entity(*food_entity).despawn();
                    decision.current_action = Action::Idle;
                }
            }
            
            Action::InitiateConversation { target, topic } => {
                commands.spawn(ConversationEvent {
                    initiator: entity,
                    target: *target,
                    topic: *topic,
                });
                decision.current_action = Action::Idle;
            }
            
            // ... handle other actions
        }
        
        // Check if current step is complete
        if let Some(step) = plan.steps.get(plan.current_step) {
            if is_action_complete(&step.completion_condition, entity, &time) {
                plan.current_step += 1;
                
                // Move to next step or complete plan
                if plan.current_step >= plan.steps.len() {
                    decision.current_action = Action::Idle;
                    plan.steps.clear();
                } else if let Some(next_step) = plan.steps.get(plan.current_step) {
                    decision.current_action = next_step.action.clone();
                }
            }
        }
    }
}

fn execute_move_to(
    velocity: &mut Velocity,
    position: &Position,
    target: Vec2,
    speed: MoveSpeed,
    time: &Time,
) {
    let direction = (target - position.0).normalize_or_zero();
    let speed_multiplier = match speed {
        MoveSpeed::Slow => 0.5,
        MoveSpeed::Normal => 1.0,
        MoveSpeed::Fast => 1.5,
        MoveSpeed::Sprint => 2.0,
    };
    
    velocity.0 = direction * BASE_CREATURE_SPEED * speed_multiplier;
}
```
## Decision Pipeline

### Main Decision System

```rust
pub fn creature_decision_system(
    time: Res<Time>,
    mut creatures: Query<(
        Entity,
        &Position,
        &Needs,
        &Genetics,
        &mut DecisionState,
        &mut DecisionContext,
        &mut UtilityScores,
        &mut ActionPlan,
        &LODLevel,
    )>,
    spatial_index: Res<SpatialIndex>,
    world_info: Res<WorldInfo>,
) {
    for (entity, pos, needs, genetics, mut decision, mut context, mut scores, mut plan, lod) in &mut creatures {
        // Skip if on cooldown or too distant
        if time.elapsed_seconds_f64() < decision.last_decision_time + decision.decision_cooldown {
            continue;
        }
        
        // LOD-based decision frequency
        let decision_frequency = match lod.0 {
            0 => 0.1,  // 10 Hz for nearby creatures
            1 => 0.5,  // 2 Hz for medium distance
            2 => 1.0,  // 1 Hz for far
            _ => 2.0,  // 0.5 Hz for very far
        };
        
        if time.elapsed_seconds_f64() < context.last_update + decision_frequency {
            continue;
        }
        
        // Update context (perception)
        update_decision_context(&mut context, entity, pos, &spatial_index, &world_info);
        
        // Calculate utilities for all goals
        update_utility_scores(&mut scores, &context, needs, genetics);
        
        // Select best goal
        if let Some(new_goal) = select_best_goal(&scores, &decision.current_goal) {
            if new_goal != decision.current_goal {
                // Goal changed, create new action plan
                let new_plan = create_action_plan(new_goal, &context, pos, needs);
                
                plan.steps = new_plan.steps;
                plan.current_step = 0;
                plan.plan_validity = 1.0;
                
                if let Some(first_step) = plan.steps.front() {
                    decision.current_action = first_step.action.clone();
                }
                
                decision.current_goal = new_goal;
            }
        }
        
        // Update cooldown
        decision.last_decision_time = time.elapsed_seconds_f64();
        decision.decision_cooldown = decision_frequency;
    }
}

fn update_decision_context(
    context: &mut DecisionContext,
    entity: Entity,
    position: &Position,
    spatial_index: &SpatialIndex,
    world_info: &WorldInfo,
) {
    // Find nearby entities
    let nearby = spatial_index.query_radius(position.0, PERCEPTION_RADIUS);
    
    context.nearby_creatures.clear();
    context.nearby_resources.clear();
    
    for (other_entity, other_pos, entity_type) in nearby {
        if other_entity == entity { continue; }
        
        let distance = position.0.distance(other_pos);
        
        match entity_type {
            EntityType::Creature => {
                context.nearby_creatures.push((other_entity, distance));
            }
            EntityType::Resource(resource_type) => {
                context.nearby_resources.push((resource_type, other_pos, 1.0));
            }
        }
    }
    
    // Sort by distance
    context.nearby_creatures.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    context.nearby_resources.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());
    
    // Update environmental factors
    context.environmental_factors = world_info.get_environment_at(position.0);
    context.last_update = time.elapsed_seconds_f64();
}
```

### Goal Selection

```rust
fn update_utility_scores(
    scores: &mut UtilityScores,
    context: &DecisionContext,
    needs: &Needs,
    genetics: &Genetics,
) {
    scores.scores.clear();
    
    for goal in Goal::iter() {
        let evaluator = goal.get_evaluator();
        
        // Skip if requirements not met
        if !evaluator.requirements_met(context) {
            continue;
        }
        
        // Base utility
        let mut utility = evaluator.evaluate(context, needs);
        
        // Apply genetic modifiers
        utility *= genetics.get_trait_modifier(goal.associated_trait());
        
        // Apply priority boost
        utility *= 1.0 + (goal.priority() as f32 / 100.0);
        
        scores.scores.insert(goal, utility);
    }
    
    scores.dirty = false;
}

fn select_best_goal(
    scores: &UtilityScores,
    current_goal: &Goal,
) -> Option<Goal> {
    // Hysteresis to prevent goal thrashing
    const HYSTERESIS_FACTOR: f32 = 1.2;
    
    let current_score = scores.scores.get(current_goal).copied().unwrap_or(0.0);
    
    scores.scores
        .iter()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .and_then(|(goal, score)| {
            if goal == current_goal || *score > current_score * HYSTERESIS_FACTOR {
                Some(*goal)
            } else {
                None
            }
        })
}
```
## Action Planning

### Plan Generation

```rust
fn create_action_plan(
    goal: Goal,
    context: &DecisionContext,
    position: &Position,
    needs: &Needs,
) -> ActionPlan {
    let steps = match goal {
        Goal::SatisfyHunger => plan_food_acquisition(context, position),
        Goal::SatisfyThirst => plan_water_acquisition(context, position),
        Goal::Socialize => plan_social_interaction(context, position),
        Goal::Explore => plan_exploration(position),
        Goal::Rest => plan_rest(needs),
        Goal::Reproduce => plan_reproduction(context),
        Goal::FleeFromDanger => plan_escape(context, position),
        Goal::SeekShelter => plan_shelter_search(context, position),
    };
    
    ActionPlan {
        steps,
        current_step: 0,
        plan_validity: 1.0,
    }
}

fn plan_food_acquisition(
    context: &DecisionContext,
    position: &Position,
) -> VecDeque<ActionStep> {
    let mut steps = VecDeque::new();
    
    // Check if food is already visible
    if let Some((_, food_pos, _)) = context.nearby_resources
        .iter()
        .find(|(t, _, _)| *t == ResourceType::Food)
    {
        // Direct path to food
        steps.push_back(ActionStep {
            action: Action::MoveTo {
                target: *food_pos,
                speed: MoveSpeed::Normal,
            },
            completion_condition: CompletionCondition::ReachedPosition { tolerance: 1.0 },
            timeout: 10.0,
        });
        
        steps.push_back(ActionStep {
            action: Action::Eat {
                food_entity: Entity::PLACEHOLDER, // Will be resolved when reached
            },
            completion_condition: CompletionCondition::NeedSatisfied {
                need_type: NeedType::Hunger,
                threshold: 80.0,
            },
            timeout: 2.0,
        });
    } else {
        // Search for food
        steps.push_back(ActionStep {
            action: Action::SearchFor {
                resource_type: ResourceType::Food,
                search_radius: 50.0,
            },
            completion_condition: CompletionCondition::TimeElapsed { seconds: 30.0 },
            timeout: 30.0,
        });
    }
    
    steps
}

fn plan_social_interaction(
    context: &DecisionContext,
    position: &Position,
) -> VecDeque<ActionStep> {
    let mut steps = VecDeque::new();
    
    if let Some((target, _)) = context.nearby_creatures.first() {
        // Approach the nearest creature
        steps.push_back(ActionStep {
            action: Action::ApproachCreature { target: *target },
            completion_condition: CompletionCondition::ReachedPosition { tolerance: 5.0 },
            timeout: 15.0,
        });
        
        // Initiate conversation
        steps.push_back(ActionStep {
            action: Action::InitiateConversation {
                target: *target,
                topic: choose_conversation_topic(context),
            },
            completion_condition: CompletionCondition::TimeElapsed { seconds: 0.1 },
            timeout: 1.0,
        });
    }
    
    steps
}
```

## Performance Optimizations

### Caching Strategy

```rust
// Cache frequently accessed data
#[derive(Component)]
pub struct DecisionCache {
    // Cached goal utilities
    goal_utilities: HashMap<Goal, (f32, f64)>, // (utility, timestamp)
    cache_duration: f32,
    
    // Cached paths
    cached_paths: HashMap<Vec2, (Vec<Vec2>, f64)>, // (path, timestamp)
    
    // Cached resource locations
    known_resources: HashMap<ResourceType, Vec<(Vec2, f64)>>, // (position, last_seen)
}

impl DecisionCache {
    pub fn get_cached_utility(&self, goal: Goal, current_time: f64) -> Option<f32> {
        self.goal_utilities
            .get(&goal)
            .filter(|(_, timestamp)| current_time - timestamp < self.cache_duration as f64)
            .map(|(utility, _)| *utility)
    }
    
    pub fn cache_utility(&mut self, goal: Goal, utility: f32, current_time: f64) {
        self.goal_utilities.insert(goal, (utility, current_time));
    }
}
```
### LOD-Based Decision Making

```rust
pub fn apply_decision_lod(
    lod: &LODLevel,
    scores: &mut UtilityScores,
    context: &mut DecisionContext,
) {
    match lod.0 {
        0 => {
            // Full fidelity - all goals evaluated
            // No modifications needed
        }
        1 => {
            // Reduced - skip exploration and complex social
            scores.scores.remove(&Goal::Explore);
            context.nearby_creatures.truncate(5); // Only consider 5 nearest
        }
        2 => {
            // Minimal - only basic needs
            scores.scores.retain(|goal, _| {
                matches!(goal, Goal::SatisfyHunger | Goal::SatisfyThirst | Goal::Rest)
            });
            context.nearby_creatures.clear();
        }
        _ => {
            // Statistical only - use predetermined behavior patterns
            scores.scores.clear();
            scores.scores.insert(Goal::Wander, 1.0);
        }
    }
}
```

## Social Influences

### Incorporating Social Factors

```rust
#[derive(Component)]
pub struct SocialInfluence {
    // Recent conversations affect decisions
    conversation_history: VecDeque<ConversationMemory>,
    
    // Group dynamics
    group_membership: Option<GroupId>,
    group_role: GroupRole,
    
    // Relationships affect utility
    relationship_modifiers: HashMap<Entity, f32>,
}

#[derive(Debug, Clone)]
pub struct ConversationMemory {
    partner: Entity,
    topic: ConversationTopic,
    outcome: ConversationOutcome,
    timestamp: f64,
    influence_strength: f32,
}

impl SocialInfluence {
    pub fn apply_social_modifiers(
        &self,
        goal: Goal,
        base_utility: f32,
        context: &DecisionContext,
    ) -> f32 {
        let mut modified = base_utility;
        
        // Recent conversations can influence decisions
        for memory in &self.conversation_history {
            if memory.topic.relates_to_goal(goal) {
                modified *= 1.0 + memory.influence_strength;
            }
        }
        
        // Group dynamics
        if let Some(group_id) = self.group_membership {
            modified *= self.group_role.decision_modifier(goal);
        }
        
        // Nearby friends/rivals affect decisions
        for (creature, distance) in &context.nearby_creatures {
            if let Some(relationship_modifier) = self.relationship_modifiers.get(creature) {
                // Friends make social goals more attractive
                if matches!(goal, Goal::Socialize | Goal::Reproduce) {
                    modified *= 1.0 + (relationship_modifier / distance);
                }
            }
        }
        
        modified
    }
}
```

## Debugging and Visualization

### Debug Components

```rust
#[cfg(feature = "debug")]
#[derive(Component)]
pub struct DecisionDebugInfo {
    pub utility_history: VecDeque<(f64, HashMap<Goal, f32>)>,
    pub goal_changes: VecDeque<(f64, Goal, Goal)>, // (time, from, to)
    pub plan_failures: VecDeque<(f64, String)>,
    pub decision_frequency: f32,
}

#[cfg(feature = "debug")]
pub fn debug_decision_system(
    creatures: Query<(
        Entity,
        &Name,
        &DecisionState,
        &UtilityScores,
        &DecisionDebugInfo,
    )>,
    mut egui_context: ResMut<EguiContext>,
) {
    egui::Window::new("Decision Debug")
        .show(egui_context.ctx_mut(), |ui| {
            for (entity, name, decision, scores, debug) in &creatures {
                ui.collapsing(format!("{} ({:?})", name.0, entity), |ui| {
                    ui.label(format!("Current Goal: {:?}", decision.current_goal));
                    ui.label(format!("Current Action: {:?}", decision.current_action));
                    
                    ui.separator();
                    ui.label("Utility Scores:");
                    
                    for (goal, score) in &scores.scores {
                        ui.horizontal(|ui| {
                            ui.label(format!("{:?}:", goal));
                            ui.add(egui::ProgressBar::new(*score / 100.0));
                            ui.label(format!("{:.2}", score));
                        });
                    }
                    
                    // Plot utility history
                    if !debug.utility_history.is_empty() {
                        ui.separator();
                        ui.label("Utility History:");
                        // ... plot code
                    }
                });
            }
        });
}
```
## Integration with Other Systems

### Genetics Integration

```rust
pub trait GeneticTrait {
    fn modify_utility(&self, goal: Goal, base_utility: f32) -> f32;
    fn modify_action_speed(&self, action: &Action, base_speed: f32) -> f32;
}

impl Genetics {
    pub fn get_decision_modifiers(&self) -> DecisionModifiers {
        DecisionModifiers {
            aggression: self.get_trait_value(TraitType::Aggression),
            sociability: self.get_trait_value(TraitType::Sociability),
            curiosity: self.get_trait_value(TraitType::Curiosity),
            caution: self.get_trait_value(TraitType::Caution),
            metabolism: self.get_trait_value(TraitType::Metabolism),
        }
    }
}

pub struct DecisionModifiers {
    pub aggression: f32,   // Affects flee/fight decisions
    pub sociability: f32,  // Affects social goal utility
    pub curiosity: f32,    // Affects exploration utility
    pub caution: f32,      // Affects danger perception
    pub metabolism: f32,   // Affects hunger/thirst urgency
}
```

### Conversation System Integration

```rust
#[derive(Event)]
pub struct ConversationCompleteEvent {
    pub participants: [Entity; 2],
    pub topic: ConversationTopic,
    pub outcome: ConversationOutcome,
    pub shared_information: Vec<Information>,
}

pub fn process_conversation_outcomes(
    mut events: EventReader<ConversationCompleteEvent>,
    mut creatures: Query<(&mut SocialInfluence, &mut DecisionCache)>,
) {
    for event in events.read() {
        for &participant in &event.participants {
            if let Ok((mut social, mut cache)) = creatures.get_mut(participant) {
                // Record conversation memory
                social.conversation_history.push_front(ConversationMemory {
                    partner: event.participants[1 - 0], // Other participant
                    topic: event.topic,
                    outcome: event.outcome,
                    timestamp: time.elapsed_seconds_f64(),
                    influence_strength: event.outcome.influence_value(),
                });
                
                // Share resource knowledge
                for info in &event.shared_information {
                    if let Information::ResourceLocation { resource_type, position } = info {
                        cache.known_resources
                            .entry(*resource_type)
                            .or_default()
                            .push((*position, time.elapsed_seconds_f64()));
                    }
                }
                
                // Limit history size
                social.conversation_history.truncate(20);
            }
        }
    }
}
```

## Testing Strategies

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_utility_calculation() {
        let mut needs = Needs {
            hunger: 20.0, // Very hungry
            thirst: 50.0,
            social: 80.0,
            ..default()
        };
        
        let context = DecisionContext {
            nearby_resources: vec![(ResourceType::Food, Vec2::new(10.0, 10.0), 1.0)],
            ..default()
        };
        
        let hunger_utility = HungerUtility.evaluate(&context, &needs);
        assert!(hunger_utility > 50.0); // Should be high priority
        
        // Test with no food nearby
        let empty_context = DecisionContext::default();
        let hunger_utility_no_food = HungerUtility.evaluate(&empty_context, &needs);
        assert!(hunger_utility_no_food < hunger_utility); // Should be lower without food
    }
    
    #[test]
    fn test_goal_selection_hysteresis() {
        let mut scores = UtilityScores::default();
        scores.scores.insert(Goal::SatisfyHunger, 80.0);
        scores.scores.insert(Goal::Socialize, 90.0);
        
        let current = Goal::SatisfyHunger;
        
        // Should not switch due to hysteresis
        let selected = select_best_goal(&scores, &current);
        assert_eq!(selected, Some(Goal::SatisfyHunger));
        
        // Should switch if difference is large enough
        scores.scores.insert(Goal::Socialize, 100.0);
        let selected = select_best_goal(&scores, &current);
        assert_eq!(selected, Some(Goal::Socialize));
    }
}
```

### Integration Tests

```rust
#[test]
fn test_full_decision_pipeline() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        DecisionPlugin,
    ));
    
    // Spawn test creature
    let creature = app.world.spawn((
        CreatureBundle::default(),
        Needs { hunger: 10.0, ..default() },
        DecisionState::default(),
        DecisionContext::default(),
        UtilityScores::default(),
    )).id();
    
    // Place food nearby
    let food = app.world.spawn((
        Position(Vec2::new(5.0, 5.0)),
        Resource { 
            resource_type: ResourceType::Food,
            nutrition: 50.0,
        },
    )).id();
    
    // Run decision system
    app.update();
    
    // Check that creature decided to seek food
    let decision = app.world.get::<DecisionState>(creature).unwrap();
    assert_eq!(decision.current_goal, Goal::SatisfyHunger);
}
```

## Configuration

### Decision System Settings

```rust
#[derive(Resource)]
pub struct DecisionSettings {
    // Utility calculation
    pub utility_update_frequency: f32,
    pub hysteresis_factor: f32,
    
    // Performance
    pub max_creatures_per_frame: usize,
    pub enable_caching: bool,
    pub cache_duration: f32,
    
    // LOD settings
    pub lod_distances: [f32; 4],
    pub lod_decision_rates: [f32; 4],
    
    // Debug
    pub debug_visualization: bool,
    pub track_decision_history: bool,
}

impl Default for DecisionSettings {
    fn default() -> Self {
        Self {
            utility_update_frequency: 0.5,
            hysteresis_factor: 1.2,
            max_creatures_per_frame: 100,
            enable_caching: true,
            cache_duration: 1.0,
            lod_distances: [50.0, 200.0, 500.0, 1000.0],
            lod_decision_rates: [0.1, 0.5, 1.0, 2.0],
            debug_visualization: cfg!(debug_assertions),
            track_decision_history: cfg!(debug_assertions),
        }
    }
}
```

## Best Practices Summary

1. **Keep utility functions pure** - No side effects, only calculations
2. **Use caching aggressively** - Decisions don't need to be frame-perfect
3. **Apply LOD early** - Distant creatures get simplified decision making
4. **Test goal transitions** - Ensure hysteresis prevents thrashing
5. **Profile decision frequency** - Most creatures don't need 60Hz decisions
6. **Make actions atomic** - Each action should be completable independently
7. **Use events for side effects** - Keep systems decoupled

---
*Last Updated: 2024-12-XX*