# Tool Use & Crafting System Design

## Overview

A sophisticated tool use system that models creature intelligence through object manipulation, problem-solving, and cultural transmission of tool-making knowledge. The system emphasizes emergent tool discovery and realistic learning curves.

## Core Tool System

```rust
pub struct ToolSystem {
    pub tool_registry: HashMap<ToolType, ToolDefinition>,
    pub crafting_recipes: HashMap<CraftedToolType, Recipe>,
    pub tool_knowledge: HashMap<EntityId, ToolKnowledge>,
    pub active_tool_use: HashMap<EntityId, ActiveToolUse>,
}

pub struct Tool {
    pub tool_type: ToolType,
    pub material: MaterialType,
    pub durability: f32,
    pub effectiveness: f32,
    pub size: ToolSize,
    pub modifications: Vec<ToolModification>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToolType {
    // Natural tools (found objects)
    Stone(StoneToolType),
    Stick(StickToolType),
    Leaf(LeafToolType),
    Shell(ShellToolType),
    
    // Modified tools
    SharpStone,
    PointedStick,
    LeafCup,
    
    // Crafted tools
    Hammer,
    Probe,
    Container,
    Scraper,
}

pub struct ToolDefinition {
    pub base_type: ToolType,
    pub valid_uses: Vec<ToolUse>,
    pub required_intelligence: f32,
    pub learning_difficulty: f32,
    pub grip_requirements: GripType,
}

pub enum ToolUse {
    // Food acquisition
    CrackNut { force_required: f32 },
    ExtractInsects { reach: f32 },
    DigRoots { depth: f32 },
    FishingProbe,
    
    // Defense/Offense
    Weapon { damage_bonus: f32 },
    ThrownProjectile { range: f32 },
    
    // Construction
    DiggingTool,
    NestBuilding,
    
    // Social
    Gift,
    Display,
    
    // Problem solving
    ReachExtension { added_reach: f32 },
    LeverageMultiplier { force_multiplier: f32 },
}
```

### Tool Discovery & Learning

```rust
pub struct ToolLearningSystem {
    pub discovery_engine: DiscoveryEngine,
    pub skill_progression: HashMap<EntityId, ToolSkills>,
    pub observation_learning: ObservationLearning,
    pub innovation_tracker: InnovationTracker,
}

pub struct ToolKnowledge {
    pub known_tools: HashMap<ToolType, ToolProficiency>,
    pub discovered_uses: HashMap<(ToolType, ToolUse), f32>, // Confidence
    pub crafting_knowledge: Vec<RecipeKnowledge>,
    pub teaching_ability: f32,
}

pub struct ToolProficiency {
    pub skill_level: f32,        // 0.0 = novice, 1.0 = master
    pub usage_count: u32,
    pub success_rate: f32,
    pub learned_from: LearningSource,
}

pub enum LearningSource {
    SelfDiscovery,
    Observation { teacher: EntityId },
    DirectTeaching { teacher: EntityId },
    TrialAndError,
    Accident,
}

impl ToolLearningSystem {
    pub fn attempt_tool_discovery(
        &mut self,
        creature: &mut Creature,
        available_objects: &[WorldObject],
        problem: &Problem,
    ) -> Option<ToolDiscovery> {
        // Check if creature has minimum intelligence
        if creature.cognitive_stats.intelligence < problem.min_intelligence {
            return None;
        }
        
        // Evaluate objects for tool potential
        let mut candidates = Vec::new();
        for object in available_objects {
            let tool_potential = self.evaluate_tool_potential(
                object,
                problem,
                &creature.physical_capabilities
            );
            
            if tool_potential > 0.0 {
                candidates.push((object, tool_potential));
            }
        }
        
        // Select best candidate with exploration factor
        if let Some((object, _)) = self.select_tool_candidate(candidates, creature) {
            // Attempt to use object as tool
            let success = self.try_tool_use(creature, object, problem);
            
            if success {
                // Record discovery
                let discovery = ToolDiscovery {
                    creature_id: creature.id,
                    tool_type: object.to_tool_type(),
                    use_case: problem.to_tool_use(),
                    timestamp: current_time(),
                    innovation_score: self.calculate_innovation_score(object, problem),
                };
                
                // Update creature's knowledge
                self.update_tool_knowledge(creature, &discovery);
                
                return Some(discovery);
            }
        }
        
        None
    }
    
    pub fn observe_tool_use(
        &mut self,
        observer: &mut Creature,
        demonstrator: &Creature,
        tool_use: &ActiveToolUse,
    ) -> LearningOutcome {
        // Check observation conditions
        let observation_quality = self.calculate_observation_quality(
            observer,
            demonstrator,
            tool_use
        );
        
        if observation_quality < 0.3 {
            return LearningOutcome::TooDistant;
        }
        
        // Learning chance based on intelligence and attention
        let learning_chance = observer.cognitive_stats.intelligence * 
                            observer.cognitive_stats.attention * 
                            observation_quality;
        
        if rand::random::<f32>() < learning_chance {
            // Add observed knowledge
            let proficiency = ToolProficiency {
                skill_level: 0.1, // Start low for observed learning
                usage_count: 0,
                success_rate: 0.0,
                learned_from: LearningSource::Observation { 
                    teacher: demonstrator.id 
                },
            };
            
            observer.tool_knowledge.known_tools.insert(
                tool_use.tool.tool_type,
                proficiency
            );
            
            LearningOutcome::LearnedByObservation
        } else {
            LearningOutcome::ObservedButNotLearned
        }
    }
}
```

### Active Tool Use

```rust
pub struct ActiveToolUse {
    pub tool: Tool,
    pub purpose: ToolUse,
    pub target: ToolTarget,
    pub start_time: f32,
    pub progress: f32,
    pub technique: UseTechnique,
}

pub enum ToolTarget {
    Object(EntityId),
    Location(Vec3),
    Creature(EntityId),
    Self_,
}

pub struct UseTechnique {
    pub grip: GripStyle,
    pub motion: MotionPattern,
    pub force: f32,
    pub precision: f32,
}

pub enum MotionPattern {
    Strike { angle: f32, velocity: f32 },
    Probe { depth: f32, angle: f32 },
    Sweep { arc: f32 },
    Twist { torque: f32 },
    Throw { trajectory: Vec3 },
}

impl ToolUseExecutor {
    pub fn execute_tool_use(
        &mut self,
        creature: &mut Creature,
        tool_use: &mut ActiveToolUse,
        delta_time: f32,
    ) -> ToolUseResult {
        // Check if creature maintains grip
        if !self.check_grip_stability(creature, &tool_use.tool, &tool_use.technique.grip) {
            return ToolUseResult::DroppedTool;
        }
        
        // Progress based on technique and skill
        let skill = creature.tool_knowledge
            .known_tools
            .get(&tool_use.tool.tool_type)
            .map(|p| p.skill_level)
            .unwrap_or(0.0);
        
        let progress_rate = self.calculate_progress_rate(
            &tool_use.purpose,
            &tool_use.technique,
            skill,
            &creature.physical_stats
        );
        
        tool_use.progress += progress_rate * delta_time;
        
        // Check completion
        if tool_use.progress >= 1.0 {
            self.complete_tool_use(creature, tool_use)
        } else {
            // Apply wear to tool
            tool_use.tool.durability -= 0.01 * delta_time;
            
            if tool_use.tool.durability <= 0.0 {
                ToolUseResult::ToolBroken
            } else {
                ToolUseResult::InProgress(tool_use.progress)
            }
        }
    }
    
    fn complete_tool_use(
        &mut self,
        creature: &mut Creature,
        tool_use: &ActiveToolUse,
    ) -> ToolUseResult {
        match &tool_use.purpose {
            ToolUse::CrackNut { .. } => {
                // Give food reward
                creature.inventory.add_food(FoodType::NutMeat, 5);
                
                // Improve skill
                if let Some(proficiency) = creature.tool_knowledge
                    .known_tools
                    .get_mut(&tool_use.tool.tool_type) {
                    proficiency.usage_count += 1;
                    proficiency.skill_level = (proficiency.skill_level + 0.05).min(1.0);
                    proficiency.success_rate = self.update_success_rate(proficiency);
                }
                
                ToolUseResult::Success(ToolUseOutcome::FoodObtained(5))
            }
            
            ToolUse::ExtractInsects { .. } => {
                let insects_found = self.probe_for_insects(&tool_use.target);
                creature.inventory.add_food(FoodType::Insect, insects_found);
                
                ToolUseResult::Success(ToolUseOutcome::FoodObtained(insects_found))
            }
            
            _ => ToolUseResult::Success(ToolUseOutcome::TaskCompleted)
        }
    }
}
```

### Tool Crafting

```rust
pub struct CraftingSystem {
    pub recipes: HashMap<CraftedToolType, Recipe>,
    pub material_properties: HashMap<MaterialType, MaterialProperties>,
    pub crafting_stations: HashMap<Vec3, CraftingStation>,
}

pub struct Recipe {
    pub result: CraftedToolType,
    pub ingredients: Vec<Ingredient>,
    pub technique: CraftingTechnique,
    pub difficulty: f32,
    pub time_required: f32,
    pub discovered_by: Option<EntityId>,
}

pub struct Ingredient {
    pub material: MaterialType,
    pub quantity: u32,
    pub quality_requirement: Option<f32>,
    pub preparation: Option<Preparation>,
}

pub enum CraftingTechnique {
    Knapping {
        strikes_required: u32,
        precision_needed: f32,
    },
    Binding {
        material: BindingMaterial,
        pattern: BindingPattern,
    },
    Shaping {
        method: ShapingMethod,
        iterations: u32,
    },
    Assembly {
        components: Vec<ComponentSlot>,
        order_matters: bool,
    },
}

pub struct CraftingAttempt {
    pub recipe: Recipe,
    pub crafter: EntityId,
    pub materials: Vec<(MaterialType, WorldObject)>,
    pub progress: f32,
    pub quality: f32,
    pub mistakes: u32,
}

impl CraftingSystem {
    pub fn attempt_crafting(
        &mut self,
        creature: &mut Creature,
        recipe: &Recipe,
        materials: Vec<WorldObject>,
    ) -> Result<CraftingAttempt, CraftingError> {
        // Validate materials
        if !self.validate_materials(&recipe.ingredients, &materials) {
            return Err(CraftingError::IncorrectMaterials);
        }
        
        // Check skill requirements
        let crafting_skill = creature.tool_knowledge
            .crafting_knowledge
            .iter()
            .find(|k| k.recipe_type == recipe.result)
            .map(|k| k.proficiency)
            .unwrap_or(0.0);
        
        if crafting_skill < recipe.difficulty * 0.5 {
            return Err(CraftingError::InsufficientSkill);
        }
        
        // Initialize crafting attempt
        Ok(CraftingAttempt {
            recipe: recipe.clone(),
            crafter: creature.id,
            materials: materials.into_iter()
                .map(|obj| (obj.material_type(), obj))
                .collect(),
            progress: 0.0,
            quality: crafting_skill,
            mistakes: 0,
        })
    }
    
    pub fn progress_crafting(
        &mut self,
        attempt: &mut CraftingAttempt,
        creature: &Creature,
        action: CraftingAction,
        delta_time: f32,
    ) -> CraftingProgress {
        match &attempt.recipe.technique {
            CraftingTechnique::Knapping { strikes_required, precision_needed } => {
                if let CraftingAction::Strike { force, angle } = action {
                    let precision = self.calculate_strike_precision(
                        creature,
                        force,
                        angle,
                        *precision_needed
                    );
                    
                    if precision > 0.7 {
                        attempt.progress += 1.0 / *strikes_required as f32;
                        attempt.quality *= 1.0 + precision * 0.1;
                    } else if precision < 0.3 {
                        attempt.mistakes += 1;
                        attempt.quality *= 0.9;
                    }
                }
            }
            
            CraftingTechnique::Binding { pattern, .. } => {
                if let CraftingAction::Wrap { tension, overlap } = action {
                    let correctness = self.evaluate_binding(
                        pattern,
                        tension,
                        overlap
                    );
                    
                    attempt.progress += correctness * 0.1 * delta_time;
                    attempt.quality = attempt.quality * 0.9 + correctness * 0.1;
                }
            }
            
            _ => {}
        }
        
        // Check completion
        if attempt.progress >= 1.0 {
            self.complete_crafting(attempt)
        } else if attempt.mistakes > 5 || attempt.quality < 0.2 {
            CraftingProgress::Failed(CraftingFailure::TooManyMistakes)
        } else {
            CraftingProgress::InProgress {
                percent: attempt.progress,
                quality: attempt.quality,
            }
        }
    }
    
    fn complete_crafting(&self, attempt: &CraftingAttempt) -> CraftingProgress {
        let tool = Tool {
            tool_type: ToolType::Crafted(attempt.recipe.result),
            material: attempt.materials[0].0, // Primary material
            durability: 100.0 * attempt.quality,
            effectiveness: attempt.quality,
            size: self.determine_tool_size(&attempt.materials),
            modifications: Vec::new(),
        };
        
        CraftingProgress::Completed(tool)
    }
}
```

### Tool Modification & Improvement

```rust
pub struct ToolModificationSystem {
    pub modification_types: HashMap<ModificationType, ModificationRequirements>,
    pub combination_effects: HashMap<(ModificationType, ModificationType), SynergyEffect>,
}

pub struct ToolModification {
    pub mod_type: ModificationType,
    pub quality: f32,
    pub applied_by: EntityId,
    pub timestamp: f32,
}

pub enum ModificationType {
    Sharpening { edge_angle: f32 },
    Hafting { handle_material: MaterialType },
    Reinforcement { binding_type: BindingType },
    WeightBalance { counterweight: f32 },
    Decoration { aesthetic_value: f32 },
}

impl ToolModificationSystem {
    pub fn apply_modification(
        &mut self,
        tool: &mut Tool,
        modification: ModificationType,
        creature: &Creature,
    ) -> ModificationResult {
        // Check if modification is valid for tool
        if !self.can_modify(tool, &modification) {
            return ModificationResult::Incompatible;
        }
        
        // Calculate modification quality based on skill
        let skill = creature.tool_knowledge
            .modification_skill
            .unwrap_or(0.0);
        
        let quality = self.calculate_modification_quality(
            skill,
            &modification,
            tool
        );
        
        // Apply effects
        match modification {
            ModificationType::Sharpening { edge_angle } => {
                tool.effectiveness *= 1.0 + quality * 0.5;
                tool.durability *= 1.0 - quality * 0.1; // Sharper = more fragile
            }
            
            ModificationType::Hafting { .. } => {
                tool.effectiveness *= 1.0 + quality * 0.3;
                tool.durability *= 1.0 + quality * 0.2;
            }
            
            _ => {}
        }
        
        // Record modification
        tool.modifications.push(ToolModification {
            mod_type: modification,
            quality,
            applied_by: creature.id,
            timestamp: current_time(),
        });
        
        ModificationResult::Success { quality }
    }
}
```

### Cultural Tool Transmission

```rust
pub struct CulturalTransmission {
    pub teaching_sessions: HashMap<(EntityId, EntityId), TeachingSession>,
    pub tool_traditions: HashMap<GroupId, ToolTradition>,
    pub innovation_spread: InnovationSpreadModel,
}

pub struct ToolTradition {
    pub group_id: GroupId,
    pub preferred_tools: Vec<ToolType>,
    pub unique_techniques: Vec<UseTechnique>,
    pub taboo_materials: Vec<MaterialType>,
    pub master_crafters: Vec<EntityId>,
}

pub struct TeachingSession {
    pub teacher: EntityId,
    pub student: EntityId,
    pub skill_taught: ToolSkill,
    pub progress: f32,
    pub teaching_quality: f32,
}

impl CulturalTransmission {
    pub fn initiate_teaching(
        &mut self,
        teacher: &Creature,
        student: &mut Creature,
        skill: ToolSkill,
    ) -> Result<TeachingSession, TeachingError> {
        // Validate teacher has skill
        let teacher_skill = teacher.get_tool_skill(&skill)
            .ok_or(TeachingError::TeacherLacksSkill)?;
        
        if teacher_skill < 0.5 {
            return Err(TeachingError::InsufficientMastery);
        }
        
        // Check student readiness
        if student.cognitive_stats.attention < 0.3 {
            return Err(TeachingError::StudentDistracted);
        }
        
        let session = TeachingSession {
            teacher: teacher.id,
            student: student.id,
            skill_taught: skill,
            progress: 0.0,
            teaching_quality: teacher.tool_knowledge.teaching_ability,
        };
        
        Ok(session)
    }
    
    pub fn demonstrate_technique(
        &mut self,
        session: &mut TeachingSession,
        demonstration: &ToolDemonstration,
    ) -> TeachingOutcome {
        // Quality of demonstration
        let demo_quality = self.evaluate_demonstration(demonstration);
        
        // Student learning based on attention and intelligence
        let learning_rate = session.teaching_quality * 
                          demo_quality * 
                          demonstration.student_attention * 
                          demonstration.student_intelligence;
        
        session.progress += learning_rate * 0.1;
        
        if session.progress >= 1.0 {
            TeachingOutcome::SkillLearned
        } else {
            TeachingOutcome::ProgressMade(session.progress)
        }
    }
    
    pub fn spread_innovation(
        &mut self,
        innovation: &ToolInnovation,
        population: &[Creature],
    ) -> InnovationSpread {
        let mut spread = InnovationSpread::new(innovation.clone());
        
        // Find early adopters
        for creature in population {
            let adoption_chance = self.calculate_adoption_probability(
                creature,
                innovation,
                &spread.current_adopters
            );
            
            if rand::random::<f32>() < adoption_chance {
                spread.adopt(creature.id, current_time());
            }
        }
        
        spread
    }
}
```

### Problem Solving with Tools

```rust
pub struct ToolProblemSolver {
    pub problem_types: HashMap<ProblemType, Vec<ToolSolution>>,
    pub creative_solutions: HashMap<EntityId, Vec<CreativeSolution>>,
}

pub struct Problem {
    pub problem_type: ProblemType,
    pub constraints: Vec<Constraint>,
    pub goal_state: GoalState,
    pub min_intelligence: f32,
}

pub enum ProblemType {
    OutOfReachFood { height: f32 },
    ProtectedResource { protection: ProtectionType },
    CrossingObstacle { obstacle: ObstacleType },
    BuildingShelter { requirements: ShelterRequirements },
}

pub struct ToolSolution {
    pub required_tools: Vec<ToolType>,
    pub technique: SolutionTechnique,
    pub success_rate: f32,
    pub innovation_level: f32,
}

impl ToolProblemSolver {
    pub fn solve_problem(
        &mut self,
        creature: &Creature,
        problem: &Problem,
        available_tools: &[Tool],
    ) -> Option<PlannedSolution> {
        // Check known solutions
        if let Some(solutions) = self.problem_types.get(&problem.problem_type) {
            for solution in solutions {
                if self.has_required_tools(&solution.required_tools, available_tools) {
                    if creature.cognitive_stats.intelligence >= solution.innovation_level {
                        return Some(self.plan_solution(creature, solution, available_tools));
                    }
                }
            }
        }
        
        // Try creative problem solving
        if creature.cognitive_stats.creativity > 0.7 {
            self.attempt_creative_solution(creature, problem, available_tools)
        } else {
            None
        }
    }
    
    fn attempt_creative_solution(
        &mut self,
        creature: &Creature,
        problem: &Problem,
        available_tools: &[Tool],
    ) -> Option<PlannedSolution> {
        // Analyze problem requirements
        let requirements = self.analyze_problem_requirements(problem);
        
        // Find tools with matching capabilities
        let mut tool_combinations = Vec::new();
        for tool in available_tools {
            let capabilities = self.analyze_tool_capabilities(tool);
            let match_score = self.match_capabilities_to_requirements(
                &capabilities,
                &requirements
            );
            
            if match_score > 0.5 {
                tool_combinations.push((tool, match_score));
            }
        }
        
        // Try combining tools
        if tool_combinations.len() >= 2 {
            let creative_solution = CreativeSolution {
                problem: problem.clone(),
                tools_used: tool_combinations.iter().map(|(t, _)| t.tool_type).collect(),
                technique: self.generate_novel_technique(&tool_combinations),
                discovered_by: creature.id,
                timestamp: current_time(),
            };
            
            self.creative_solutions
                .entry(creature.id)
                .or_insert_with(Vec::new)
                .push(creative_solution.clone());
            
            Some(self.plan_creative_solution(&creative_solution, available_tools))
        } else {
            None
        }
    }
}
```

### Tool Use Animation & Feedback

```rust
pub struct ToolUseVisualization {
    pub animation_sets: HashMap<(ToolType, ToolUse), AnimationSet>,
    pub particle_effects: HashMap<ToolInteraction, ParticleEffect>,
    pub sound_effects: HashMap<ToolAction, SoundEffect>,
}

pub struct ToolUseAnimation {
    pub creature_animation: CreatureAnimation,
    pub tool_motion: ToolMotion,
    pub interaction_point: Vec3,
    pub timing: AnimationTiming,
}

impl ToolUseVisualization {
    pub fn animate_tool_use(
        &self,
        creature: &Creature,
        tool_use: &ActiveToolUse,
    ) -> AnimationSequence {
        let base_animation = self.animation_sets
            .get(&(tool_use.tool.tool_type, tool_use.purpose))
            .cloned()
            .unwrap_or_default();
        
        // Adjust for creature physiology
        let adjusted = self.adjust_for_creature(base_animation, creature);
        
        // Add procedural elements
        let procedural = match &tool_use.technique.motion {
            MotionPattern::Strike { angle, velocity } => {
                self.generate_strike_animation(*angle, *velocity)
            }
            MotionPattern::Probe { depth, angle } => {
                self.generate_probe_animation(*depth, *angle)
            }
            _ => AnimationLayer::default(),
        };
        
        AnimationSequence {
            layers: vec![adjusted, procedural],
            duration: tool_use.estimated_duration(),
            blend_mode: BlendMode::Additive,
        }
    }
    
    pub fn create_impact_effects(
        &self,
        tool: &Tool,
        target: &ToolTarget,
        impact_force: f32,
    ) -> Vec<Effect> {
        let mut effects = Vec::new();
        
        // Particle effects
        let particle_type = match (tool.material, target) {
            (MaterialType::Stone, ToolTarget::Object(_)) => ParticleType::StoneDust,
            (MaterialType::Wood, _) => ParticleType::WoodChips,
            _ => ParticleType::GenericDebris,
        };
        
        effects.push(Effect::Particle(ParticleEffect {
            particle_type,
            count: (impact_force * 10.0) as u32,
            velocity: impact_force * 2.0,
            spread: 45.0,
            lifetime: 1.0,
        }));
        
        // Sound effects
        effects.push(Effect::Sound(self.get_impact_sound(tool, target, impact_force)));
        
        effects
    }
}
```

## Integration Points

### With Cognitive System
- Intelligence requirements for tool use
- Learning and memory of techniques
- Problem-solving capabilities

### With Social System
- Teaching and learning from others
- Cultural tool traditions
- Status from tool mastery

### With Physics System
- Tool weight and balance
- Impact forces and momentum
- Material properties

### With Animation System
- Complex tool manipulation animations
- Procedural motion generation
- Hand/grip positioning

## Performance Considerations

- Tool discovery checks only run when creatures encounter problems
- Crafting progress is updated at 10Hz instead of every frame
- Tool traditions are cached per group and updated weekly
- Animation blending uses pre-computed poses
- Physics calculations use simplified models for distant tools

## Balance Configuration

```rust
pub struct ToolUseBalance {
    // Learning rates
    pub base_learning_speed: f32,           // 0.1
    pub observation_bonus: f32,             // 1.5
    pub teaching_effectiveness: f32,        // 2.0
    
    // Intelligence thresholds
    pub min_tool_use_intelligence: f32,     // 0.3
    pub min_crafting_intelligence: f32,     // 0.5
    pub min_innovation_intelligence: f32,   // 0.7
    
    // Success rates
    pub novice_success_rate: f32,          // 0.3
    pub skilled_success_rate: f32,         // 0.7
    pub master_success_rate: f32,          // 0.95
    
    // Tool durability
    pub base_tool_durability: f32,          // 100.0
    pub durability_loss_rate: f32,         // 0.1
    pub crafted_tool_bonus: f32,           // 1.5
    
    // Innovation spread
    pub innovation_spread_rate: f32,        // 0.05
    pub cultural_resistance: f32,           // 0.3
    pub tradition_persistence: f32,         // 0.8
}