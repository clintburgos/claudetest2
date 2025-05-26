use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Resource, Default)]
pub struct ObservationGoals {
    pub goals: HashMap<String, Goal>,
    pub completed_goals: Vec<CompletedGoal>,
    pub active_goal: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Goal {
    pub id: String,
    pub name: String,
    pub description: String,
    pub goal_type: GoalType,
    pub progress: f32,
    pub target: f32,
    pub completed: bool,
    pub reward_description: String,
}

#[derive(Clone, Debug)]
pub enum GoalType {
    ObserveCreatureCount(u32),
    ObserveResourceConsumption(u32),
    ObserveCreatureDeath,
    ObserveCreatureBirth,
    ObserveTimeElapsed(f32),
    ObserveCreatureHunger,
    ObserveCreatureThirst,
    DiscoverBehaviorPattern,
}

#[derive(Clone, Debug)]
pub struct CompletedGoal {
    pub goal_id: String,
    pub completed_at: f32,
}

impl ObservationGoals {
    pub fn new() -> Self {
        let mut goals = HashMap::new();
        
        // Phase 1 starter goals
        goals.insert("first_observation".to_string(), Goal {
            id: "first_observation".to_string(),
            name: "First Observation".to_string(),
            description: "Observe the simulation for 30 seconds".to_string(),
            goal_type: GoalType::ObserveTimeElapsed(30.0),
            progress: 0.0,
            target: 30.0,
            completed: false,
            reward_description: "You've taken your first steps as an observer!".to_string(),
        });
        
        goals.insert("population_watcher".to_string(), Goal {
            id: "population_watcher".to_string(),
            name: "Population Watcher".to_string(),
            description: "Observe when the population reaches 100 creatures".to_string(),
            goal_type: GoalType::ObserveCreatureCount(100),
            progress: 0.0,
            target: 100.0,
            completed: false,
            reward_description: "You've witnessed a thriving ecosystem!".to_string(),
        });
        
        goals.insert("resource_observer".to_string(), Goal {
            id: "resource_observer".to_string(),
            name: "Resource Observer".to_string(),
            description: "Watch creatures consume 50 resources".to_string(),
            goal_type: GoalType::ObserveResourceConsumption(50),
            progress: 0.0,
            target: 50.0,
            completed: false,
            reward_description: "You understand the basics of creature survival!".to_string(),
        });
        
        goals.insert("life_cycle".to_string(), Goal {
            id: "life_cycle".to_string(),
            name: "Circle of Life".to_string(),
            description: "Witness a creature death".to_string(),
            goal_type: GoalType::ObserveCreatureDeath,
            progress: 0.0,
            target: 1.0,
            completed: false,
            reward_description: "You've observed the natural cycle of life.".to_string(),
        });
        
        goals.insert("hunger_observer".to_string(), Goal {
            id: "hunger_observer".to_string(),
            name: "Hunger Observer".to_string(),
            description: "Watch a creature become critically hungry".to_string(),
            goal_type: GoalType::ObserveCreatureHunger,
            progress: 0.0,
            target: 1.0,
            completed: false,
            reward_description: "You've learned about creature needs!".to_string(),
        });
        
        Self {
            goals,
            completed_goals: Vec::new(),
            active_goal: Some("first_observation".to_string()),
        }
    }
    
    pub fn complete_goal(&mut self, goal_id: &str, time: f32) {
        if let Some(goal) = self.goals.get_mut(goal_id) {
            if !goal.completed {
                goal.completed = true;
                goal.progress = goal.target;
                self.completed_goals.push(CompletedGoal {
                    goal_id: goal_id.to_string(),
                    completed_at: time,
                });
                
                // Auto-select next incomplete goal
                self.active_goal = self.goals.iter()
                    .find(|(_, g)| !g.completed)
                    .map(|(id, _)| id.clone());
            }
        }
    }
    
    pub fn update_progress(&mut self, goal_id: &str, progress: f32) {
        if let Some(goal) = self.goals.get_mut(goal_id) {
            goal.progress = progress.min(goal.target);
            if goal.progress >= goal.target {
                goal.completed = true;
            }
        }
    }
    
    pub fn get_completion_percentage(&self) -> f32 {
        let total = self.goals.len() as f32;
        let completed = self.goals.values().filter(|g| g.completed).count() as f32;
        (completed / total) * 100.0
    }
}

pub fn update_observation_goals(
    mut goals: ResMut<ObservationGoals>,
    time: Res<Time>,
    creature_count: Query<&crate::components::Creature>,
    mut creature_died_events: EventReader<crate::plugins::CreatureDiedEvent>,
    mut resource_consumed_events: EventReader<crate::plugins::ResourceConsumedEvent>,
    needs_query: Query<&crate::components::Needs>,
) {
    let current_time = time.elapsed_seconds();
    
    // Update time-based goals
    if let Some(goal) = goals.goals.get("first_observation") {
        if !goal.completed {
            goals.update_progress("first_observation", current_time);
            if current_time >= 30.0 {
                goals.complete_goal("first_observation", current_time);
                info!("Goal completed: First Observation!");
            }
        }
    }
    
    // Update creature count goals
    let creature_count = creature_count.iter().count() as f32;
    if let Some(goal) = goals.goals.get("population_watcher") {
        if !goal.completed {
            goals.update_progress("population_watcher", creature_count);
            if creature_count >= 100.0 {
                goals.complete_goal("population_watcher", current_time);
                info!("Goal completed: Population Watcher!");
            }
        }
    }
    
    // Update resource consumption goals
    for _event in resource_consumed_events.read() {
        if let Some(goal) = goals.goals.get_mut("resource_observer") {
            if !goal.completed {
                goal.progress += 1.0;
                if goal.progress >= goal.target {
                    goals.complete_goal("resource_observer", current_time);
                    info!("Goal completed: Resource Observer!");
                }
            }
        }
    }
    
    // Update death observation goals
    for _event in creature_died_events.read() {
        if let Some(goal) = goals.goals.get("life_cycle") {
            if !goal.completed {
                goals.complete_goal("life_cycle", current_time);
                info!("Goal completed: Circle of Life!");
            }
        }
    }
    
    // Check for critically hungry creatures
    for needs in needs_query.iter() {
        if needs.hunger > 80.0 {
            if let Some(goal) = goals.goals.get("hunger_observer") {
                if !goal.completed {
                    goals.complete_goal("hunger_observer", current_time);
                    info!("Goal completed: Hunger Observer!");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_goal_creation() {
        let goals = ObservationGoals::new();
        assert_eq!(goals.goals.len(), 5);
        assert!(goals.active_goal.is_some());
    }

    #[test]
    fn test_goal_completion() {
        let mut goals = ObservationGoals::new();
        goals.complete_goal("first_observation", 30.0);
        
        assert!(goals.goals.get("first_observation").unwrap().completed);
        assert_eq!(goals.completed_goals.len(), 1);
    }

    #[test]
    fn test_progress_update() {
        let mut goals = ObservationGoals::new();
        goals.update_progress("population_watcher", 50.0);
        
        let goal = goals.goals.get("population_watcher").unwrap();
        assert_eq!(goal.progress, 50.0);
        assert!(!goal.completed);
        
        goals.update_progress("population_watcher", 100.0);
        let goal = goals.goals.get("population_watcher").unwrap();
        assert!(goal.completed);
    }

    #[test]
    fn test_completion_percentage() {
        let mut goals = ObservationGoals::new();
        assert_eq!(goals.get_completion_percentage(), 0.0);
        
        goals.complete_goal("first_observation", 30.0);
        assert_eq!(goals.get_completion_percentage(), 20.0);
        
        goals.complete_goal("population_watcher", 60.0);
        assert_eq!(goals.get_completion_percentage(), 40.0);
    }
}