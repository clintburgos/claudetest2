//! AI and decision-making components

use bevy::prelude::*;

/// Timer for making decisions
#[derive(Component, Debug)]
pub struct DecisionTimer {
    pub timer: Timer,
    pub last_decision_time: f32,
}

impl Default for DecisionTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.5, TimerMode::Repeating),
            last_decision_time: 0.0,
        }
    }
}

/// Current target or goal
#[derive(Component, Debug, Clone)]
pub enum CurrentTarget {
    None,
    Position(Vec2),
    Entity(Entity),
}

impl Default for CurrentTarget {
    fn default() -> Self {
        Self::None
    }
}

/// Cached decision for performance
#[derive(Component, Debug, Clone)]
pub struct CachedDecision {
    pub decision: Decision,
    pub hash: u64,
}

/// State of a creature's conversation
#[derive(Component, Debug, Clone, PartialEq)]
pub enum ConversationState {
    Greeting,
    ShareInfo(String),
    RequestHelp,
    OfferHelp,
    Farewell,
}

/// Possible decisions a creature can make
#[derive(Debug, Clone, PartialEq)]
pub enum Decision {
    Idle,
    Move {
        target: Vec2,
        urgency: f32,
    },
    Consume {
        resource: Entity,
        resource_type: crate::components::ResourceType,
    },
    Rest {
        duration: f32,
    },
    Flee {
        from: Vec2,
        urgency: f32,
    },
    Socialize {
        with: Entity,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_decision_timer_default() {
        let timer = DecisionTimer::default();
        assert_eq!(timer.timer.duration().as_secs_f32(), 0.5);
        assert_eq!(timer.timer.mode(), TimerMode::Repeating);
        assert_eq!(timer.last_decision_time, 0.0);
    }
    
    #[test]
    fn test_decision_timer_tick() {
        let mut timer = DecisionTimer::default();
        assert!(!timer.timer.finished());
        
        // Tick by 0.5 seconds
        timer.timer.tick(std::time::Duration::from_secs_f32(0.5));
        assert!(timer.timer.just_finished());
    }
    
    #[test]
    fn test_current_target_variants() {
        let none = CurrentTarget::None;
        assert!(matches!(none, CurrentTarget::None));
        
        let position = CurrentTarget::Position(Vec2::new(10.0, 20.0));
        match position {
            CurrentTarget::Position(pos) => {
                assert_eq!(pos.x, 10.0);
                assert_eq!(pos.y, 20.0);
            }
            _ => panic!("Expected Position variant"),
        }
        
        let entity = CurrentTarget::Entity(Entity::from_raw(42));
        match entity {
            CurrentTarget::Entity(e) => {
                assert_eq!(e.index(), 42);
            }
            _ => panic!("Expected Entity variant"),
        }
    }
    
    #[test]
    fn test_current_target_default() {
        let target = CurrentTarget::default();
        assert!(matches!(target, CurrentTarget::None));
    }
    
    #[test]
    fn test_cached_decision() {
        let decision = Decision::Idle;
        let cached = CachedDecision {
            decision: decision.clone(),
            hash: 12345,
        };
        
        assert_eq!(cached.hash, 12345);
        assert!(matches!(cached.decision, Decision::Idle));
    }
    
    #[test]
    fn test_decision_idle() {
        let decision = Decision::Idle;
        assert_eq!(decision, Decision::Idle);
    }
    
    #[test]
    fn test_decision_move() {
        let decision = Decision::Move {
            target: Vec2::new(100.0, 200.0),
            urgency: 0.8,
        };
        
        match decision {
            Decision::Move { target, urgency } => {
                assert_eq!(target.x, 100.0);
                assert_eq!(target.y, 200.0);
                assert_eq!(urgency, 0.8);
            }
            _ => panic!("Expected Move decision"),
        }
    }
    
    #[test]
    fn test_decision_consume() {
        let resource_entity = Entity::from_raw(123);
        let decision = Decision::Consume {
            resource: resource_entity,
            resource_type: crate::components::ResourceType::Food,
        };
        
        match decision {
            Decision::Consume { resource, resource_type } => {
                assert_eq!(resource.index(), 123);
                assert_eq!(resource_type, crate::components::ResourceType::Food);
            }
            _ => panic!("Expected Consume decision"),
        }
    }
    
    #[test]
    fn test_decision_rest() {
        let decision = Decision::Rest { duration: 5.5 };
        
        match decision {
            Decision::Rest { duration } => {
                assert_eq!(duration, 5.5);
            }
            _ => panic!("Expected Rest decision"),
        }
    }
    
    #[test]
    fn test_decision_flee() {
        let decision = Decision::Flee {
            from: Vec2::new(50.0, 60.0),
            urgency: 1.0,
        };
        
        match decision {
            Decision::Flee { from, urgency } => {
                assert_eq!(from.x, 50.0);
                assert_eq!(from.y, 60.0);
                assert_eq!(urgency, 1.0);
            }
            _ => panic!("Expected Flee decision"),
        }
    }
    
    #[test]
    fn test_decision_socialize() {
        let target_entity = Entity::from_raw(456);
        let decision = Decision::Socialize { with: target_entity };
        
        match decision {
            Decision::Socialize { with } => {
                assert_eq!(with.index(), 456);
            }
            _ => panic!("Expected Socialize decision"),
        }
    }
    
    #[test]
    fn test_decision_equality() {
        let decision1 = Decision::Idle;
        let decision2 = Decision::Idle;
        assert_eq!(decision1, decision2);
        
        let move1 = Decision::Move {
            target: Vec2::new(1.0, 2.0),
            urgency: 0.5,
        };
        let move2 = Decision::Move {
            target: Vec2::new(1.0, 2.0),
            urgency: 0.5,
        };
        assert_eq!(move1, move2);
        
        let move3 = Decision::Move {
            target: Vec2::new(3.0, 4.0),
            urgency: 0.5,
        };
        assert_ne!(move1, move3);
    }
    
    #[test]
    fn test_decision_clone() {
        let original = Decision::Move {
            target: Vec2::new(10.0, 20.0),
            urgency: 0.7,
        };
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }
}
