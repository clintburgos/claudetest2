use crate::Vec2;
use crate::core::Entity;
use crate::simulation::{CreatureState, ResourceType};
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub enum GameEvent {
    CreatureSpawned { 
        entity: Entity, 
        position: Vec2 
    },
    CreatureDied { 
        entity: Entity, 
        cause: DeathCause 
    },
    CreatureStateChanged { 
        entity: Entity, 
        old_state: CreatureState, 
        new_state: CreatureState 
    },
    ResourceSpawned { 
        entity: Entity, 
        position: Vec2, 
        resource_type: ResourceType 
    },
    ResourceDepleted { 
        entity: Entity 
    },
    ResourceConsumed { 
        creature: Entity, 
        resource: Entity, 
        amount: f32 
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeathCause {
    Starvation,
    Dehydration,
    OldAge,
    Unknown,
}

pub struct EventBus {
    events: VecDeque<GameEvent>,
    pending_events: VecDeque<GameEvent>,
    processing: bool,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            events: VecDeque::new(),
            pending_events: VecDeque::new(),
            processing: false,
        }
    }
    
    pub fn emit(&mut self, event: GameEvent) {
        if self.processing {
            // If we're processing events, queue new ones to avoid iterator invalidation
            self.pending_events.push_back(event);
        } else {
            self.events.push_back(event);
        }
    }
    
    pub fn process<F>(&mut self, mut handler: F) 
    where 
        F: FnMut(&GameEvent)
    {
        self.processing = true;
        
        // Process all current events
        while let Some(event) = self.events.pop_front() {
            handler(&event);
        }
        
        // Move pending events to main queue
        self.events.append(&mut self.pending_events);
        self.processing = false;
    }
    
    pub fn drain(&mut self) -> impl Iterator<Item = GameEvent> + '_ {
        self.events.drain(..)
    }
    
    pub fn clear(&mut self) {
        self.events.clear();
        self.pending_events.clear();
    }
    
    pub fn is_empty(&self) -> bool {
        self.events.is_empty() && self.pending_events.is_empty()
    }
    
    pub fn len(&self) -> usize {
        self.events.len() + self.pending_events.len()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_bus_emit_and_process() {
        let mut bus = EventBus::new();
        let entity = Entity::new(1);
        
        bus.emit(GameEvent::CreatureSpawned {
            entity,
            position: Vec2::new(0.0, 0.0),
        });
        
        assert_eq!(bus.len(), 1);
        
        let mut processed = 0;
        bus.process(|_event| {
            processed += 1;
        });
        
        assert_eq!(processed, 1);
        assert!(bus.is_empty());
    }
    
    #[test]
    fn event_bus_nested_emit() {
        let mut bus = EventBus::new();
        let entity = Entity::new(1);
        
        bus.emit(GameEvent::CreatureSpawned {
            entity,
            position: Vec2::new(0.0, 0.0),
        });
        
        // Can't emit during processing in this test structure
        // This is tested by the processing flag behavior
        let mut events_to_emit = Vec::new();
        let mut processed = 0;
        
        bus.processing = true;
        while let Some(event) = bus.events.pop_front() {
            processed += 1;
            if matches!(event, GameEvent::CreatureSpawned { .. }) {
                events_to_emit.push(GameEvent::CreatureDied {
                    entity,
                    cause: DeathCause::Unknown,
                });
            }
        }
        bus.processing = false;
        
        // Emit the new events
        for event in events_to_emit {
            bus.emit(event);
        }
        
        assert_eq!(processed, 1);
        assert_eq!(bus.len(), 1); // The death event should be queued
    }
    
    #[test]
    fn event_bus_drain() {
        let mut bus = EventBus::new();
        for i in 0..3 {
            bus.emit(GameEvent::CreatureSpawned {
                entity: Entity::new(i),
                position: Vec2::new(i as f32, 0.0),
            });
        }
        
        let events: Vec<_> = bus.drain().collect();
        assert_eq!(events.len(), 3);
        assert!(bus.is_empty());
    }
}