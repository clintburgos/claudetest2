# Decision Making Implementation Guide

## Quick Start

### 1. Basic Setup

```rust
// In your main.rs or plugin setup
app.add_plugins(DecisionMakingPlugin);

// Spawn a creature with decision components
commands.spawn(CreatureBundle {
    creature: Creature::new("Blinky"),
    position: Position(Vec2::new(100.0, 100.0)),
    needs: Needs::default(),
    
    // Decision components
    decision_state: DecisionState::default(),
    decision_context: DecisionContext::default(),
    utility_scores: UtilityScores::default(),
    action_plan: ActionPlan::default(),
    
    // Optional components
    social_influence: SocialInfluence::default(),
    decision_cache: DecisionCache::default(),
    
    ..default()
});
```

### 2. Plugin Structure

```rust
pub struct DecisionMakingPlugin;

impl Plugin for DecisionMakingPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<DecisionSettings>()
            .init_resource::<GoalRegistry>()
            
            // Events
            .add_event::<GoalChangedEvent>()
            .add_event::<ActionCompleteEvent>()
            .add_event::<PlanFailedEvent>()
            
            // Systems - ordered by execution
            .add_systems(
                Update,
                (
                    // Perception phase
                    update_decision_contexts,
                    
                    // Decision phase
                    (
                        calculate_utilities,
                        apply_social_influences,
                        select_goals,
                        generate_action_plans,
                    ).chain()
                    .run_if(should_update_decisions),
                    
                    // Execution phase
                    (
                        validate_current_actions,
                        execute_actions,
                        check_action_completion,
                    ).chain(),
                    
                    // Cleanup
                    cleanup_completed_plans,
                )
                .in_set(GameSet::AI),
            )
            
            // Debug systems
            .add_systems(
                Update,
                (
                    debug_decision_visualization,
                    log_decision_changes,
                )
                .run_if(resource_exists::<DebugSettings>())
                .in_set(GameSet::Debug),
            );
    }
}
```

## Common Patterns

### Adding New Goals

```rust
// 1. Define the goal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CustomGoal {
    BuildNest,
    DefendTerritory,
}

// 2. Create utility evaluator
pub struct NestBuildingUtility;

impl UtilityEvaluator for NestBuildingUtility {
    fn evaluate(&self, context: &DecisionContext, needs: &Needs) -> f32 {
        // Calculate utility based on:
        // - Safety need
        // - Available materials
        // - Current weather
        let safety_factor = (100.0 - needs.safety) / 100.0;
        let material_factor = context.nearby_resources
            .iter()
            .filter(|(t, _, _)| *t == ResourceType::BuildingMaterial)
            .count() as f32 / 5.0; // Normalize to 0-1
        
        safety_factor * material_factor * 100.0
    }
    
    fn requirements_met(&self, context: &DecisionContext) -> bool {
        // Need building materials nearby
        context.nearby_resources
            .iter()
            .any(|(t, _, _)| *t == ResourceType::BuildingMaterial)
    }
}

// 3. Register with the system
fn register_custom_goals(mut registry: ResMut<GoalRegistry>) {
    registry.register(
        Goal::Custom(CustomGoal::BuildNest),
        Box::new(NestBuildingUtility),
    );
}
```

### Complex Action Sequences

```rust
fn plan_nest_building(
    context: &DecisionContext,
    position: &Position,
) -> VecDeque<ActionStep> {
    let mut steps = VecDeque::new();
    
    // Step 1: Find suitable location
    steps.push_back(ActionStep {
        action: Action::SearchFor {
            target: SearchTarget::SafeLocation,
            radius: 30.0,
        },
        completion_condition: CompletionCondition::Custom(Box::new(|world, entity| {
            // Check if we found a safe spot
            world.get::<FoundLocation>(entity).is_some()
        })),
        timeout: 60.0,
    });
    
    // Step 2: Gather materials (repeat 3 times)
    for _ in 0..3 {
        steps.push_back(ActionStep {
            action: Action::GatherResource {
                resource_type: ResourceType::BuildingMaterial,
                amount: 1,
            },
            completion_condition: CompletionCondition::InventoryContains {
                item: ResourceType::BuildingMaterial,
                amount: 1,
            },
            timeout: 30.0,
        });
        
        steps.push_back(ActionStep {
            action: Action::ReturnToLocation {
                location_marker: LocationMarker::NestSite,
            },
            completion_condition: CompletionCondition::ReachedMarker {
                marker: LocationMarker::NestSite,
                tolerance: 2.0,
            },
            timeout: 20.0,
        });
    }
    
    // Step 3: Build nest
    steps.push_back(ActionStep {
        action: Action::BuildStructure {
            structure_type: StructureType::Nest,
            build_time: 5.0,
        },
        completion_condition: CompletionCondition::TimeElapsed { seconds: 5.0 },
        timeout: 10.0,
    });
    
    steps
}
```

### Interrupt Handling

```rust
pub fn handle_interrupts(
    mut creatures: Query<(
        &mut DecisionState,
        &mut ActionPlan,
        &Needs,
        &DecisionContext,
    )>,
    danger_events: EventReader<DangerEvent>,
) {
    for event in danger_events.read() {
        if let Ok((mut state, mut plan, needs, context)) = creatures.get_mut(event.creature) {
            // Check if current goal should be interrupted
            let current_priority = state.current_goal.priority();
            let danger_priority = Goal::FleeFromDanger.priority();
            
            if danger_priority > current_priority {
                // Save current plan for later
                let saved_plan = std::mem::take(&mut plan.steps);
                
                // Create escape plan
                plan.steps = plan_escape(&event.danger_source, &context.position);
                
                // Store saved plan to resume later
                plan.interrupted_plan = Some(Box::new(saved_plan));
                
                state.current_goal = Goal::FleeFromDanger;
                if let Some(first_step) = plan.steps.front() {
                    state.current_action = first_step.action.clone();
                }
            }
        }
    }
}
```

## Performance Tips

### 1. Batch Similar Decisions

```rust
// Process creatures in batches by current goal
pub fn batched_decision_system(
    mut creatures: Query<(&mut DecisionState, &Goal)>,
) {
    // Group by goal
    let mut goal_groups: HashMap<Goal, Vec<Entity>> = HashMap::new();
    
    for (entity, _, goal) in &creatures {
        goal_groups.entry(*goal).or_default().push(entity);
    }
    
    // Process each group with optimized logic
    for (goal, entities) in goal_groups {
        match goal {
            Goal::SatisfyHunger => process_hungry_creatures_batch(entities),
            Goal::Socialize => process_social_creatures_batch(entities),
            // ...
        }
    }
}
```

### 2. Spatial Partitioning for Decisions

```rust
// Only update decisions for creatures in active chunks
pub fn spatial_decision_updates(
    spatial_index: Res<SpatialIndex>,
    camera: Query<&Transform, With<Camera>>,
    mut creatures: Query<(&Position, &mut DecisionState)>,
) {
    let camera_pos = camera.single().translation.truncate();
    let active_chunks = spatial_index.get_active_chunks(camera_pos, ACTIVE_RADIUS);
    
    for chunk in active_chunks {
        for entity in chunk.entities {
            if let Ok((pos, mut decision)) = creatures.get_mut(entity) {
                // Update only creatures in active chunks
                decision.needs_update = true;
            }
        }
    }
}
```

## Debugging

### Visual Debug Mode

```rust
// Add debug visualization
app.add_systems(Update, debug_draw_decisions.run_if(debug_mode));

fn debug_draw_decisions(
    creatures: Query<(&Position, &DecisionState, &ActionPlan)>,
    mut gizmos: Gizmos,
) {
    for (pos, decision, plan) in &creatures {
        // Draw current goal as colored circle
        let color = goal_to_color(decision.current_goal);
        gizmos.circle_2d(pos.0, 5.0, color);
        
        // Draw planned path
        if let Some(path) = plan.get_planned_path() {
            for window in path.windows(2) {
                gizmos.line_2d(window[0], window[1], color);
            }
        }
        
        // Draw action target
        if let Some(target) = decision.current_action.target_position() {
            gizmos.line_2d(pos.0, target, color.with_a(0.5));
            gizmos.circle_2d(target, 3.0, color);
        }
    }
}
```

### Decision History Tracking

```rust
#[derive(Component)]
pub struct DecisionHistory {
    entries: VecDeque<DecisionEntry>,
    max_entries: usize,
}

#[derive(Debug)]
struct DecisionEntry {
    timestamp: f64,
    goal: Goal,
    utilities: HashMap<Goal, f32>,
    action: Action,
    context_summary: String,
}

pub fn track_decisions(
    mut creatures: Query<(
        &DecisionState,
        &UtilityScores,
        &mut DecisionHistory,
        &DecisionContext,
    ), Changed<DecisionState>>,
    time: Res<Time>,
) {
    for (state, scores, mut history, context) in &mut creatures {
        history.entries.push_front(DecisionEntry {
            timestamp: time.elapsed_seconds_f64(),
            goal: state.current_goal,
            utilities: scores.scores.clone(),
            action: state.current_action.clone(),
            context_summary: format!(
                "Nearby: {} creatures, {} resources",
                context.nearby_creatures.len(),
                context.nearby_resources.len()
            ),
        });
        
        history.entries.truncate(history.max_entries);
    }
}
```

---
*For architecture details, see [DECISION_MAKING_SYSTEM.md](./DECISION_MAKING_SYSTEM.md)*