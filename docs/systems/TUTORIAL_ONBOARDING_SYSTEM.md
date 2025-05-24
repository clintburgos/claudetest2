# Tutorial and Onboarding System

## Overview

The tutorial system provides a gentle, discovery-based introduction to the simulation while respecting the design philosophy of "player discovery over tutorials". It uses contextual hints, interactive tooltips, and optional guided experiences.

## Design Philosophy

- **Non-intrusive**: Never block the simulation or force tutorials
- **Contextual**: Provide information when relevant
- **Progressive**: Reveal complexity gradually
- **Skippable**: All tutorials can be dismissed or disabled
- **Discovery-focused**: Encourage exploration over instruction

## Tutorial System Architecture

```rust
pub struct TutorialSystem {
    pub active_tutorials: Vec<ActiveTutorial>,
    pub hint_system: HintSystem,
    pub tooltip_manager: TooltipManager,
    pub progress_tracker: ProgressTracker,
    pub discovery_journal: DiscoveryJournal,
}

pub struct ActiveTutorial {
    pub id: TutorialId,
    pub tutorial_type: TutorialType,
    pub current_step: usize,
    pub completion_state: CompletionState,
    pub display_mode: DisplayMode,
}

pub enum TutorialType {
    FirstLaunch,
    CameraControls,
    TimeControls,
    CreatureSelection,
    DataVisualization,
    AdvancedFeatures,
}

pub enum DisplayMode {
    Subtle,      // Small hints at screen edge
    Prominent,   // Highlighted UI elements
    Interactive, // Requires user action
    Passive,     // Just information display
}
```

## Contextual Hint System

```rust
pub struct HintSystem {
    pub triggers: Vec<HintTrigger>,
    pub active_hints: PriorityQueue<Hint>,
    pub cooldowns: HashMap<HintId, Instant>,
    pub user_preferences: HintPreferences,
}

pub enum HintTrigger {
    FirstTimeAction {
        action: UserAction,
        hint_id: HintId,
    },
    
    InactivityPeriod {
        duration: Duration,
        context: HintContext,
    },
    
    EventOccurred {
        event_type: EventType,
        hint_id: HintId,
    },
    
    FeatureUnused {
        feature: FeatureType,
        usage_threshold: Duration,
    },
    
    PerformanceIssue {
        issue_type: PerformanceIssue,
        suggestion: HintId,
    },
}

pub struct Hint {
    pub id: HintId,
    pub content: HintContent,
    pub priority: HintPriority,
    pub display_duration: Duration,
    pub dismissible: bool,
}

pub enum HintContent {
    Text(String),
    
    Animated {
        text: String,
        animation: HintAnimation,
    },
    
    Interactive {
        text: String,
        target_element: UIElement,
        action_prompt: String,
    },
    
    Discovery {
        title: String,
        description: String,
        related_features: Vec<FeatureType>,
    },
}
```

## Progressive Disclosure

```rust
pub struct ProgressiveDisclosure {
    pub complexity_levels: Vec<ComplexityLevel>,
    pub current_level: usize,
    pub unlock_conditions: HashMap<ComplexityLevel, UnlockCondition>,
}

pub struct ComplexityLevel {
    pub name: String,
    pub features: HashSet<FeatureType>,
    pub ui_elements: HashSet<UIElement>,
    pub data_views: HashSet<DataView>,
}

pub enum UnlockCondition {
    TimeSpent(Duration),
    ActionsPerformed(Vec<UserAction>, u32),
    CreaturesObserved(u32),
    EventsWitnessed(Vec<EventType>),
    Manual, // User requests more features
}

impl ProgressiveDisclosure {
    pub fn update(&mut self, user_stats: &UserStats) {
        let next_level = self.current_level + 1;
        
        if next_level < self.complexity_levels.len() {
            if let Some(condition) = self.unlock_conditions.get(&self.complexity_levels[next_level]) {
                if condition.is_met(user_stats) {
                    self.unlock_next_level();
                }
            }
        }
    }
    
    fn unlock_next_level(&mut self) {
        self.current_level += 1;
        let level = &self.complexity_levels[self.current_level];
        
        // Notify about new features
        for feature in &level.features {
            self.notify_feature_unlocked(feature);
        }
    }
}
```

## Interactive Tooltips

```rust
pub struct TooltipManager {
    pub static_tooltips: HashMap<UIElement, Tooltip>,
    pub dynamic_tooltips: HashMap<Entity, EntityTooltip>,
    pub smart_positioning: SmartPositioning,
    pub detail_levels: TooltipDetailLevel,
}

pub struct Tooltip {
    pub basic_info: String,
    pub expanded_info: Option<ExpandedTooltip>,
    pub related_topics: Vec<TopicLink>,
    pub show_delay: Duration,
}

pub struct ExpandedTooltip {
    pub sections: Vec<TooltipSection>,
    pub examples: Vec<Example>,
    pub tips: Vec<String>,
}

pub struct EntityTooltip {
    pub generator: Box<dyn Fn(&Entity, &World) -> TooltipContent>,
    pub update_frequency: Duration,
    pub cache: Option<(Instant, TooltipContent)>,
}

pub enum TooltipDetailLevel {
    Minimal,    // Just names and basic stats
    Standard,   // Add descriptions
    Detailed,   // Include mechanics explanations
    Educational, // Full information with examples
}

impl TooltipManager {
    pub fn create_creature_tooltip(&self, creature: &Creature, detail: TooltipDetailLevel) -> TooltipContent {
        match detail {
            TooltipDetailLevel::Minimal => TooltipContent {
                title: format!("Creature #{}", creature.id),
                lines: vec![
                    format!("Age: {:.1} days", creature.age / 86400.0),
                    format!("Health: {:.0}%", creature.health.percentage() * 100.0),
                ],
                ..Default::default()
            },
            
            TooltipDetailLevel::Standard => TooltipContent {
                title: format!("Creature #{}", creature.id),
                lines: vec![
                    format!("Age: {:.1} days", creature.age / 86400.0),
                    format!("Health: {:.0}%", creature.health.percentage() * 100.0),
                    format!("Hunger: {:.0}%", creature.needs.hunger * 100.0),
                    format!("Group: {}", creature.group.map(|g| g.to_string()).unwrap_or("None".into())),
                ],
                expandable: true,
                ..Default::default()
            },
            
            TooltipDetailLevel::Detailed => {
                // Include genetics, relationships, current activity, etc.
                self.create_detailed_creature_tooltip(creature)
            }
            
            TooltipDetailLevel::Educational => {
                // Include explanations of mechanics
                self.create_educational_creature_tooltip(creature)
            }
        }
    }
}
```

## Discovery Journal

```rust
pub struct DiscoveryJournal {
    pub entries: Vec<JournalEntry>,
    pub categories: HashMap<DiscoveryCategory, Vec<JournalEntry>>,
    pub unlock_notifications: bool,
}

pub struct JournalEntry {
    pub id: EntryId,
    pub category: DiscoveryCategory,
    pub title: String,
    pub content: JournalContent,
    pub discovered_at: Instant,
    pub discovery_context: DiscoveryContext,
}

pub enum DiscoveryCategory {
    CreatureBehaviors,
    GroupDynamics,
    EnvironmentalPatterns,
    EvolutionaryChanges,
    EmergentPhenomena,
    GameMechanics,
}

pub enum JournalContent {
    Observation {
        description: String,
        evidence: Vec<Evidence>,
        related_entries: Vec<EntryId>,
    },
    
    Milestone {
        achievement: String,
        statistics: HashMap<String, String>,
        unlock_bonus: Option<UnlockBonus>,
    },
    
    Tutorial {
        learned_mechanic: String,
        practice_suggestions: Vec<String>,
        advanced_tips: Vec<String>,
    },
}

pub struct DiscoveryContext {
    pub trigger_event: Option<Event>,
    pub location: Option<Vec3>,
    pub involved_entities: Vec<Entity>,
    pub time_scale: f32,
}

impl DiscoveryJournal {
    pub fn check_for_discoveries(&mut self, world: &World, events: &[Event]) {
        // Check for behavior discoveries
        for event in events {
            if let Some(discovery) = self.analyze_event_for_discovery(event, world) {
                self.add_entry(discovery);
            }
        }
        
        // Check for milestone discoveries
        if let Some(milestone) = self.check_milestones(world) {
            self.add_entry(milestone);
        }
    }
    
    fn analyze_event_for_discovery(&self, event: &Event, world: &World) -> Option<JournalEntry> {
        match event {
            Event::GroupFormed { members, .. } if members.len() > 10 => {
                Some(self.create_discovery(
                    "Large Group Formation",
                    "Observed creatures forming an unusually large group",
                    DiscoveryCategory::GroupDynamics,
                ))
            }
            
            Event::CulturalInnovation { innovation, .. } => {
                Some(self.create_discovery(
                    "Cultural Innovation",
                    &format!("Witnessed the emergence of: {}", innovation.describe()),
                    DiscoveryCategory::EmergentPhenomena,
                ))
            }
            
            // ... other discoveries
            _ => None,
        }
    }
}
```

## Guided Experiences

```rust
pub struct GuidedExperience {
    pub experience_type: ExperienceType,
    pub scenario: Scenario,
    pub objectives: Vec<Objective>,
    pub hints_enabled: bool,
}

pub enum ExperienceType {
    // Optional structured experiences
    FirstCreature,      // Follow a single creature
    GroupObservation,   // Watch group dynamics
    SeasonalCycle,      // Observe seasonal changes
    EvolutionWitness,   // Track genetic changes
    EcosystemBalance,   // Understand resource cycles
}

pub struct Scenario {
    pub name: String,
    pub description: String,
    pub setup: ScenarioSetup,
    pub expected_duration: Duration,
}

pub struct Objective {
    pub description: String,
    pub completion_condition: CompletionCondition,
    pub optional: bool,
    pub hint: Option<String>,
}

pub enum CompletionCondition {
    ObserveEvent(EventType),
    TimeElapsed(Duration),
    CreatureAction(ActionType),
    DataThreshold { metric: MetricType, value: f32 },
    UserAction(UserAction),
}
```

## First Launch Experience

```rust
pub struct FirstLaunchExperience {
    pub stages: Vec<OnboardingStage>,
    pub current_stage: usize,
    pub can_skip: bool,
}

pub enum OnboardingStage {
    Welcome {
        message: String,
        duration: Duration,
    },
    
    CameraIntroduction {
        prompts: Vec<CameraPrompt>,
        practice_target: Option<Entity>,
    },
    
    TimeControlIntroduction {
        speed_demonstrations: Vec<(f32, String)>,
    },
    
    CreatureIntroduction {
        highlight_creature: Entity,
        basic_info: Vec<String>,
    },
    
    ObservationEncouragement {
        suggestions: Vec<String>,
        discovery_prompts: Vec<String>,
    },
}

impl FirstLaunchExperience {
    pub fn create_default() -> Self {
        Self {
            stages: vec![
                OnboardingStage::Welcome {
                    message: "Welcome to the Creature Simulation. You are about to observe a living ecosystem.".into(),
                    duration: Duration::from_secs(5),
                },
                
                OnboardingStage::CameraIntroduction {
                    prompts: vec![
                        CameraPrompt::Move("Use WASD to move the camera"),
                        CameraPrompt::Zoom("Scroll to zoom in and out"),
                        CameraPrompt::Rotate("Hold right-click to rotate"),
                    ],
                    practice_target: None,
                },
                
                OnboardingStage::ObservationEncouragement {
                    suggestions: vec![
                        "Click on any creature to learn more about it".into(),
                        "Watch how creatures interact with each other".into(),
                        "Notice how they search for food and water".into(),
                    ],
                    discovery_prompts: vec![
                        "What patterns do you notice?".into(),
                        "How do creatures behave differently?".into(),
                    ],
                },
            ],
            current_stage: 0,
            can_skip: true,
        }
    }
}
```

## Smart Help System

```rust
pub struct SmartHelpSystem {
    pub context_analyzer: ContextAnalyzer,
    pub help_database: HelpDatabase,
    pub suggestion_engine: SuggestionEngine,
}

pub struct ContextAnalyzer {
    pub recent_actions: RingBuffer<UserAction>,
    pub current_focus: Option<Entity>,
    pub active_tools: HashSet<ToolType>,
    pub confusion_indicators: Vec<ConfusionIndicator>,
}

pub enum ConfusionIndicator {
    RepeatedAction { action: UserAction, count: u32 },
    RapidToolSwitching { switches: u32, duration: Duration },
    CameraSpinning { rotations: f32, duration: Duration },
    UIElementHovering { element: UIElement, duration: Duration },
}

impl SmartHelpSystem {
    pub fn analyze_user_behavior(&mut self, action: UserAction) {
        self.context_analyzer.recent_actions.push(action);
        
        // Detect confusion patterns
        if let Some(indicator) = self.detect_confusion() {
            self.offer_contextual_help(indicator);
        }
        
        // Proactive suggestions
        if let Some(suggestion) = self.suggestion_engine.get_suggestion(&self.context_analyzer) {
            self.display_suggestion(suggestion);
        }
    }
}
```

## Integration

```rust
pub fn setup_tutorial_system(world: &mut World) {
    let mut tutorial_system = TutorialSystem::new();
    
    // Configure based on user preferences
    let preferences = load_user_preferences();
    tutorial_system.configure(preferences);
    
    // Setup first launch if needed
    if is_first_launch() {
        tutorial_system.queue_experience(FirstLaunchExperience::create_default());
    }
    
    // Register hint triggers
    tutorial_system.hint_system.register_triggers(vec![
        HintTrigger::FirstTimeAction {
            action: UserAction::SelectCreature,
            hint_id: HintId::CreatureSelection,
        },
        HintTrigger::EventOccurred {
            event_type: EventType::GroupFormed,
            hint_id: HintId::GroupDynamics,
        },
        // ... more triggers
    ]);
    
    world.insert_resource(tutorial_system);
}

pub fn update_tutorial_system(
    mut tutorial: ResMut<TutorialSystem>,
    user_input: Res<UserInput>,
    events: Res<Events>,
    world: Res<World>,
) {
    // Update active tutorials
    tutorial.update_active_tutorials(&user_input);
    
    // Check for discoveries
    tutorial.discovery_journal.check_for_discoveries(&world, &events.get_recent());
    
    // Update hints
    tutorial.hint_system.update(&user_input, &world);
    
    // Progressive disclosure
    if let Some(stats) = get_user_stats() {
        tutorial.progressive_disclosure.update(&stats);
    }
}
```

This tutorial system provides gentle guidance while maintaining the philosophy of discovery-based learning, ensuring new users can understand the simulation without being overwhelmed by forced tutorials.