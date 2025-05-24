# Decision Making System

## Table of Contents
1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Implementation Guide](#implementation-guide)
4. [Quick Reference](#quick-reference)

---

## Overview

The decision-making system is the brain of each creature, enabling them to evaluate their environment, consider multiple goals, and choose appropriate actions. It creates believable, emergent behaviors through a utility-based AI approach combined with personality traits and learned experiences.

### Key Features

- **Multi-goal evaluation**: Creatures balance competing needs
- **Personality-driven**: Decisions reflect individual traits
- **Context-aware**: Environmental factors influence choices
- **Learning system**: Past experiences shape future decisions
- **Emergent behavior**: Complex behaviors arise from simple rules

---

## Architecture

### Core Components

```rust
pub struct DecisionSystem {
    goal_evaluator: GoalEvaluator,
    action_selector: ActionSelector,
    memory_integration: MemoryIntegration,
    personality_modifiers: PersonalityModifiers,
}

pub struct CreatureGoals {
    active_goals: Vec<Goal>,
    goal_history: RingBuffer<CompletedGoal>,
    interrupted_goals: Vec<InterruptedGoal>,
}

pub struct Goal {
    goal_type: GoalType,
    priority: f32,
    urgency: f32,
    utility: f32,
    target: Option<Target>,
    plan: Option<ActionPlan>,
    start_time: f64,
    timeout: Option<f64>,
}

#[derive(Clone, Debug)]
pub enum GoalType {
    // Survival goals
    SatisfyHunger,
    SatisfyThirst,
    FindShelter,
    EscapeDanger,
    Rest,
    
    // Social goals
    SeekCompanionship,
    FindMate,
    CareForOffspring,
    EstablishTerritory,
    
    // Exploration goals
    Explore,
    InvestigateNovelty,
    LearnSkill,
    
    // Group goals
    FollowLeader,
    ProtectGroup,
    ShareResource,
}

pub enum Target {
    Location(Vec3),
    Entity(EntityId),
    Resource(ResourceType, Vec3),
    Area(Bounds),
}
```

### Utility Calculation

```rust
pub struct UtilityCalculator {
    base_utilities: HashMap<GoalType, f32>,
    need_curves: HashMap<NeedType, UtilityCurve>,
    context_modifiers: Vec<ContextModifier>,
}

pub struct UtilityCurve {
    curve_type: CurveType,
    parameters: CurveParameters,
}

pub enum CurveType {
    Linear,
    Exponential,
    Sigmoid,
    Quadratic,
    Custom(Box<dyn Fn(f32) -> f32>),
}

impl UtilityCalculator {
    pub fn calculate_goal_utility(
        &self,
        goal_type: &GoalType,
        creature: &Creature,
        context: &DecisionContext,
    ) -> f32 {
        let base_utility = self.base_utilities[goal_type];
        
        // Apply need-based modifiers
        let need_modifier = match goal_type {
            GoalType::SatisfyHunger => {
                self.need_curves[&NeedType::Hunger].evaluate(creature.hunger)
            }
            GoalType::SatisfyThirst => {
                self.need_curves[&NeedType::Thirst].evaluate(creature.thirst)
            }
            GoalType::Rest => {
                self.need_curves[&NeedType::Energy].evaluate(100.0 - creature.energy)
            }
            GoalType::SeekCompanionship => {
                self.need_curves[&NeedType::Social].evaluate(creature.loneliness)
            }
            _ => 1.0,
        };
        
        // Apply personality modifiers
        let personality_modifier = self.get_personality_modifier(
            goal_type,
            &creature.personality
        );
        
        // Apply context modifiers
        let context_modifier = self.evaluate_context_modifiers(
            goal_type,
            context,
            creature
        );
        
        // Calculate final utility
        base_utility * need_modifier * personality_modifier * context_modifier
    }
    
    fn evaluate_context_modifiers(
        &self,
        goal_type: &GoalType,
        context: &DecisionContext,
        creature: &Creature,
    ) -> f32 {
        let mut modifier = 1.0;
        
        // Environmental factors
        match goal_type {
            GoalType::FindShelter => {
                if context.weather == Weather::Storm {
                    modifier *= 2.0;
                }
                if context.time_of_day.is_night() {
                    modifier *= 1.5;
                }
            }
            GoalType::Explore => {
                if context.danger_level > 0.5 {
                    modifier *= 0.3;
                }
                if creature.energy < 30.0 {
                    modifier *= 0.5;
                }
            }
            _ => {}
        }
        
        // Social factors
        if context.nearby_creatures.len() > 10 {
            match goal_type {
                GoalType::SeekCompanionship => modifier *= 0.5,
                GoalType::EscapeDanger => modifier *= 1.5,
                _ => {}
            }
        }
        
        modifier
    }
}
```

### Action Selection

```rust
pub struct ActionSelector {
    action_library: HashMap<GoalType, Vec<ActionTemplate>>,
    feasibility_checker: FeasibilityChecker,
    cost_estimator: CostEstimator,
}

pub struct ActionTemplate {
    action_type: ActionType,
    preconditions: Vec<Precondition>,
    effects: Vec<Effect>,
    base_cost: ActionCost,
}

pub struct ActionPlan {
    goal: GoalType,
    steps: Vec<PlannedAction>,
    total_cost: ActionCost,
    expected_utility: f32,
    alternatives: Vec<ActionPlan>,
}

pub struct PlannedAction {
    action: ActionType,
    target: Option<Target>,
    duration: f32,
    energy_cost: f32,
    success_probability: f32,
}

impl ActionSelector {
    pub fn select_action_for_goal(
        &self,
        goal: &Goal,
        creature: &Creature,
        world: &WorldState,
    ) -> Option<ActionPlan> {
        // Get possible actions for this goal
        let action_templates = &self.action_library[&goal.goal_type];
        
        // Filter by feasibility
        let feasible_actions: Vec<_> = action_templates.iter()
            .filter(|template| {
                self.feasibility_checker.is_feasible(
                    template,
                    creature,
                    world
                )
            })
            .collect();
        
        if feasible_actions.is_empty() {
            return None;
        }
        
        // Evaluate each action
        let mut action_plans = Vec::new();
        for template in feasible_actions {
            let plan = self.create_action_plan(
                template,
                goal,
                creature,
                world
            );
            
            if let Some(plan) = plan {
                action_plans.push(plan);
            }
        }
        
        // Select best plan based on utility/cost ratio
        action_plans.into_iter()
            .max_by_key(|plan| {
                ((plan.expected_utility / plan.total_cost.time) * 1000.0) as i32
            })
    }
    
    fn create_action_plan(
        &self,
        template: &ActionTemplate,
        goal: &Goal,
        creature: &Creature,
        world: &WorldState,
    ) -> Option<ActionPlan> {
        let mut steps = Vec::new();
        let mut current_state = creature.get_state();
        
        // Simple planning - just one step for now
        // Could be expanded to multi-step planning
        let action = PlannedAction {
            action: template.action_type.clone(),
            target: self.find_best_target(&template.action_type, creature, world),
            duration: self.estimate_duration(template, creature),
            energy_cost: template.base_cost.energy * creature.efficiency_modifier(),
            success_probability: self.estimate_success(template, creature, world),
        };
        
        steps.push(action);
        
        // Calculate total cost
        let total_cost = ActionCost {
            time: steps.iter().map(|s| s.duration).sum(),
            energy: steps.iter().map(|s| s.energy_cost).sum(),
            risk: 1.0 - steps.iter().map(|s| s.success_probability).product::<f32>(),
        };
        
        Some(ActionPlan {
            goal: goal.goal_type.clone(),
            steps,
            total_cost,
            expected_utility: goal.utility * self.estimate_success(template, creature, world),
            alternatives: Vec::new(),
        })
    }
}
```

### Memory Integration

```rust
pub struct MemoryIntegration {
    experience_database: HashMap<(GoalType, ActionType), ExperienceRecord>,
    recent_failures: RingBuffer<FailureRecord>,
    learned_associations: HashMap<ContextPattern, PreferredAction>,
}

pub struct ExperienceRecord {
    successes: u32,
    failures: u32,
    average_cost: ActionCost,
    average_reward: f32,
    context_correlations: HashMap<ContextFeature, f32>,
}

impl MemoryIntegration {
    pub fn adjust_action_utility(
        &self,
        action: &ActionType,
        goal: &GoalType,
        base_utility: f32,
        context: &DecisionContext,
    ) -> f32 {
        let key = (goal.clone(), action.clone());
        
        if let Some(experience) = self.experience_database.get(&key) {
            let success_rate = experience.successes as f32 / 
                              (experience.successes + experience.failures) as f32;
            
            // Adjust utility based on past performance
            let experience_modifier = 0.5 + success_rate * 0.5;
            
            // Check for learned context associations
            let context_bonus = self.evaluate_context_associations(
                experience,
                context
            );
            
            base_utility * experience_modifier * (1.0 + context_bonus)
        } else {
            // No experience - use exploration bonus
            base_utility * 1.1
        }
    }
    
    pub fn record_outcome(
        &mut self,
        goal: GoalType,
        action: ActionType,
        outcome: ActionOutcome,
        context: DecisionContext,
    ) {
        let key = (goal, action);
        let record = self.experience_database.entry(key).or_insert_with(|| {
            ExperienceRecord {
                successes: 0,
                failures: 0,
                average_cost: ActionCost::default(),
                average_reward: 0.0,
                context_correlations: HashMap::new(),
            }
        });
        
        // Update statistics
        match outcome.result {
            OutcomeResult::Success => {
                record.successes += 1;
                record.average_reward = 
                    (record.average_reward * (record.successes - 1) as f32 + outcome.reward) / 
                    record.successes as f32;
            }
            OutcomeResult::Failure => {
                record.failures += 1;
                
                // Record failure for analysis
                self.recent_failures.push(FailureRecord {
                    goal: key.0,
                    action: key.1,
                    context: context.clone(),
                    reason: outcome.failure_reason,
                    timestamp: current_time(),
                });
            }
        }
        
        // Update context correlations
        self.update_context_correlations(record, &context, &outcome);
    }
}
```

### Personality Influence

```rust
pub struct PersonalityModifiers {
    trait_influences: HashMap<(PersonalityTrait, GoalType), f32>,
}

pub struct Personality {
    courage: f32,       // 0.0 = cowardly, 1.0 = brave
    curiosity: f32,     // 0.0 = cautious, 1.0 = explorative
    aggression: f32,    // 0.0 = passive, 1.0 = aggressive
    sociability: f32,   // 0.0 = solitary, 1.0 = social
    patience: f32,      // 0.0 = impulsive, 1.0 = deliberate
}

impl PersonalityModifiers {
    pub fn apply_personality(
        &self,
        goal_utilities: &mut HashMap<GoalType, f32>,
        personality: &Personality,
    ) {
        for (goal_type, utility) in goal_utilities.iter_mut() {
            let modifier = match goal_type {
                GoalType::Explore => {
                    0.5 + personality.curiosity * 0.5 + personality.courage * 0.25
                }
                GoalType::EscapeDanger => {
                    1.5 - personality.courage * 0.5
                }
                GoalType::SeekCompanionship => {
                    0.5 + personality.sociability * 0.5
                }
                GoalType::EstablishTerritory => {
                    0.5 + personality.aggression * 0.3 + personality.patience * 0.2
                }
                _ => 1.0,
            };
            
            *utility *= modifier;
        }
    }
    
    pub fn influence_action_selection(
        &self,
        personality: &Personality,
        action_type: &ActionType,
    ) -> f32 {
        match action_type {
            ActionType::Fight => 0.5 + personality.aggression * 0.5,
            ActionType::Flee => 1.5 - personality.courage * 0.5,
            ActionType::Investigate => 0.5 + personality.curiosity * 0.5,
            ActionType::Wait => 0.5 + personality.patience * 0.5,
            _ => 1.0,
        }
    }
}
```

---

## Implementation Guide

### Decision Loop

```rust
impl DecisionMaker {
    pub fn make_decision(
        &mut self,
        creature: &mut Creature,
        world: &WorldState,
        delta_time: f32,
    ) -> Option<Action> {
        // Step 1: Update decision context
        let context = self.build_context(creature, world);
        
        // Step 2: Evaluate all possible goals
        let mut goal_utilities = HashMap::new();
        for goal_type in GoalType::all() {
            let utility = self.utility_calculator.calculate_goal_utility(
                &goal_type,
                creature,
                &context,
            );
            goal_utilities.insert(goal_type, utility);
        }
        
        // Step 3: Apply personality modifiers
        self.personality_modifiers.apply_personality(
            &mut goal_utilities,
            &creature.personality,
        );
        
        // Step 4: Select highest utility goal
        let selected_goal = goal_utilities.iter()
            .max_by_key(|(_, utility)| (*utility * 1000.0) as i32)
            .map(|(goal_type, utility)| Goal {
                goal_type: goal_type.clone(),
                priority: *utility,
                urgency: self.calculate_urgency(goal_type, creature),
                utility: *utility,
                target: None,
                plan: None,
                start_time: world.current_time,
                timeout: Some(world.current_time + 300.0), // 5 minute timeout
            })?;
        
        // Step 5: Generate action plan for selected goal
        let action_plan = self.action_selector.select_action_for_goal(
            &selected_goal,
            creature,
            world,
        )?;
        
        // Step 6: Execute first action in plan
        creature.current_goal = Some(selected_goal);
        creature.current_plan = Some(action_plan.clone());
        
        action_plan.steps.first().map(|step| {
            Action {
                action_type: step.action.clone(),
                target: step.target.clone(),
            }
        })
    }
    
    fn build_context(&self, creature: &Creature, world: &WorldState) -> DecisionContext {
        DecisionContext {
            time_of_day: world.time_of_day,
            weather: world.weather.clone(),
            season: world.season,
            danger_level: self.assess_danger(creature, world),
            nearby_creatures: world.get_creatures_near(creature.position, 50.0),
            nearby_resources: world.get_resources_near(creature.position, 100.0),
            recent_events: creature.memory.get_recent_events(60.0), // Last minute
        }
    }
    
    fn calculate_urgency(&self, goal_type: &GoalType, creature: &Creature) -> f32 {
        match goal_type {
            GoalType::SatisfyHunger => {
                // Exponential urgency as hunger approaches critical
                (creature.hunger / 100.0).powf(3.0)
            }
            GoalType::EscapeDanger => {
                // Always urgent
                1.0
            }
            GoalType::CareForOffspring => {
                // Urgent if offspring in danger
                creature.offspring_danger_level()
            }
            _ => 0.5, // Default moderate urgency
        }
    }
}
```

### Goal Interruption

```rust
impl GoalInterruption {
    pub fn should_interrupt_current_goal(
        &self,
        current_goal: &Goal,
        new_goal: &Goal,
        creature: &Creature,
    ) -> bool {
        // Always interrupt for critical survival needs
        if matches!(new_goal.goal_type, GoalType::EscapeDanger) {
            return true;
        }
        
        // Check urgency difference
        let urgency_difference = new_goal.urgency - current_goal.urgency;
        if urgency_difference > 0.5 {
            return true;
        }
        
        // Check utility difference
        let utility_ratio = new_goal.utility / current_goal.utility;
        if utility_ratio > 2.0 {
            return true;
        }
        
        // Consider progress on current goal
        if let Some(progress) = current_goal.calculate_progress() {
            // Less likely to interrupt if close to completion
            if progress > 0.8 {
                return false;
            }
        }
        
        // Personality-based interruption
        creature.personality.patience < 0.3 && utility_ratio > 1.2
    }
    
    pub fn handle_interruption(
        &mut self,
        creature: &mut Creature,
        current_goal: Goal,
        new_goal: Goal,
    ) {
        // Store interrupted goal if it's worth resuming
        if current_goal.utility > 0.3 && current_goal.calculate_progress().unwrap_or(0.0) > 0.2 {
            creature.goals.interrupted_goals.push(InterruptedGoal {
                goal: current_goal,
                interruption_time: current_time(),
                interruption_reason: InterruptionReason::HigherPriority(new_goal.goal_type.clone()),
            });
        }
        
        // Switch to new goal
        creature.current_goal = Some(new_goal);
        creature.current_plan = None; // Will be regenerated
    }
}
```

### Learning from Experience

```rust
impl LearningSystem {
    pub fn learn_from_outcome(
        &mut self,
        creature: &mut Creature,
        action: &Action,
        outcome: &ActionOutcome,
        context: &DecisionContext,
    ) {
        // Update experience records
        self.memory_integration.record_outcome(
            creature.current_goal.as_ref().unwrap().goal_type.clone(),
            action.action_type.clone(),
            outcome.clone(),
            context.clone(),
        );
        
        // Adjust future preferences based on outcome
        match outcome.result {
            OutcomeResult::Success => {
                // Reinforce successful action-context pairs
                self.reinforce_behavior(creature, action, context, outcome.reward);
                
                // Update creature's confidence
                creature.confidence = (creature.confidence + 0.1).min(1.0);
            }
            OutcomeResult::Failure => {
                // Learn to avoid this action in similar contexts
                self.discourage_behavior(creature, action, context);
                
                // Reduce confidence
                creature.confidence = (creature.confidence - 0.05).max(0.0);
                
                // Analyze failure pattern
                if self.detect_repeated_failure(creature, action) {
                    self.develop_alternative_strategy(creature, action.action_type.clone());
                }
            }
        }
    }
    
    fn detect_repeated_failure(&self, creature: &Creature, action: &Action) -> bool {
        let recent_failures = self.memory_integration.recent_failures
            .iter()
            .filter(|f| {
                f.action == action.action_type &&
                f.timestamp > current_time() - 300.0 // Last 5 minutes
            })
            .count();
        
        recent_failures >= 3
    }
    
    fn develop_alternative_strategy(
        &mut self,
        creature: &mut Creature,
        failed_action: ActionType,
    ) {
        // Find alternative actions for the same goal
        if let Some(goal) = &creature.current_goal {
            let alternatives = self.action_selector.action_library[&goal.goal_type]
                .iter()
                .filter(|template| template.action_type != failed_action)
                .collect::<Vec<_>>();
            
            if !alternatives.is_empty() {
                // Boost preference for alternatives
                for alt in alternatives {
                    creature.action_preferences
                        .entry(alt.action_type.clone())
                        .and_modify(|pref| *pref *= 1.5)
                        .or_insert(1.5);
                }
            }
        }
    }
}
```

---

## Quick Reference

### Goal Priority Formula

```
Final Priority = Base Utility × Need Modifier × Personality Modifier × Context Modifier
```

### Common Goal Utilities

| Goal | Base Utility | Primary Need | Urgency Curve |
|------|--------------|--------------|---------------|
| SatisfyHunger | 0.8 | Hunger | Exponential |
| SatisfyThirst | 0.9 | Thirst | Exponential |
| EscapeDanger | 1.0 | Safety | Binary |
| Rest | 0.6 | Energy | Linear |
| SeekCompanionship | 0.5 | Social | Sigmoid |
| Explore | 0.3 | Curiosity | Linear |

### Personality Trait Effects

| Trait | Increased Goals | Decreased Goals |
|-------|----------------|-----------------|
| High Courage | Explore, Investigate | EscapeDanger |
| High Curiosity | Explore, LearnSkill | Rest, Hide |
| High Aggression | Fight, EstablishTerritory | Flee, ShareResource |
| High Sociability | SeekCompanionship, ShareResource | Explore (alone) |
| High Patience | Wait, Plan | Rush, Panic |

### Decision Timing

Decision update frequencies are managed by the LOD system. See [Performance Guide](../reference/PERFORMANCE.md#lod-system) for update rates based on distance to camera.

### Common Decision Patterns

```rust
// Emergency override pattern
if creature.health < 20.0 || creature.immediate_danger() {
    return Action::Flee(nearest_safe_location);
}

// Need satisfaction pattern
let most_urgent_need = creature.get_most_urgent_need();
if most_urgent_need.urgency > 0.8 {
    return satisfy_need(most_urgent_need);
}

// Social coordination pattern
if let Some(group_goal) = creature.group.current_goal() {
    if should_follow_group(creature, group_goal) {
        return contribute_to_group_goal(group_goal);
    }
}
```

### Performance Tips

1. **Cache goal utilities** for 1-5 seconds
2. **Use spatial indices** for target finding
3. **Limit plan depth** to 3-5 actions
4. **Batch similar decisions** for groups
5. **Skip non-critical decisions** at high time scales

### Debugging Decisions

```rust
// Log decision process
log::debug!(
    "Creature {} selected goal {:?} with utility {}",
    creature.id,
    selected_goal.goal_type,
    selected_goal.utility
);

// Visualize decision factors
if debug_mode {
    draw_debug_info(DebugInfo {
        goal_utilities,
        personality_influence,
        context_factors,
        selected_action,
    });
}
```