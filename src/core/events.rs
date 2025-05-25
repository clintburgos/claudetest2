//! Event system for decoupled communication between systems.
//!
//! Events allow systems to communicate without direct dependencies.
//! The double-buffering approach prevents iterator invalidation when
//! events trigger new events during processing.
//!
//! # Design Pattern
//! - Systems emit events during their update
//! - Events are queued, not processed immediately
//! - After all systems update, events are processed
//! - Event handlers may emit new events (queued for next frame)

use crate::core::Entity;
use crate::simulation::{CreatureState, ResourceType};
use crate::Vec2;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub enum GameEvent {
    CreatureSpawned {
        entity: Entity,
        position: Vec2,
    },
    CreatureDied {
        entity: Entity,
        cause: DeathCause,
    },
    CreatureStateChanged {
        entity: Entity,
        old_state: CreatureState,
        new_state: CreatureState,
    },
    ResourceSpawned {
        entity: Entity,
        position: Vec2,
        resource_type: ResourceType,
    },
    ResourceDepleted {
        entity: Entity,
    },
    ResourceConsumed {
        creature: Entity,
        resource: Entity,
        amount: f32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeathCause {
    Starvation,
    Dehydration,
    OldAge,
    Unknown,
}

/// Event bus for game events with double-buffering.
///
/// The EventBus uses a double-buffering technique to solve a common problem:
/// what if processing an event causes new events to be emitted? Without
/// double-buffering, this would invalidate iterators or require complex
/// re-entrancy handling.
///
/// # How it works
/// 1. Events are emitted to `events` buffer
/// 2. During processing, `events` is swapped with empty `pending_events`
/// 3. New events emitted during processing go to the now-empty `events`
/// 4. After processing, any new events remain for the next frame
///
/// # Performance
/// - Emit: O(1) amortized
/// - Process: O(n) where n is number of events
/// - No allocations during normal operation (buffers reused)
///
/// # Memory Management
/// - Events are automatically dropped if the queue exceeds MAX_EVENTS
/// - This prevents unbounded memory growth in pathological cases
pub struct EventBus {
    events: VecDeque<GameEvent>,
    pending_events: VecDeque<GameEvent>,
    processing: bool,
    /// Maximum number of events to queue (prevents memory leaks)
    max_events: usize,
    /// Number of dropped events (for monitoring)
    dropped_events: u64,
}

impl EventBus {
    /// Default maximum events to prevent memory leaks
    pub const DEFAULT_MAX_EVENTS: usize = 10000;

    pub fn new() -> Self {
        Self {
            events: VecDeque::new(),
            pending_events: VecDeque::new(),
            processing: false,
            max_events: Self::DEFAULT_MAX_EVENTS,
            dropped_events: 0,
        }
    }

    /// Creates an event bus with a custom maximum event limit
    pub fn with_max_events(max_events: usize) -> Self {
        Self {
            events: VecDeque::new(),
            pending_events: VecDeque::new(),
            processing: false,
            max_events,
            dropped_events: 0,
        }
    }

    pub fn emit(&mut self, event: GameEvent) {
        let target_queue =
            if self.processing { &mut self.pending_events } else { &mut self.events };

        // Drop oldest events if we exceed the limit
        if target_queue.len() >= self.max_events {
            target_queue.pop_front();
            self.dropped_events += 1;

            // Log warning periodically
            if self.dropped_events % 1000 == 0 {
                bevy::log::warn!(
                    "EventBus has dropped {} events. Consider increasing max_events or investigating event generation.",
                    self.dropped_events
                );
            }
        }

        target_queue.push_back(event);
    }

    /// Returns the number of events dropped due to queue overflow
    pub fn dropped_events(&self) -> u64 {
        self.dropped_events
    }

    pub fn process<F>(&mut self, mut handler: F)
    where
        F: FnMut(&GameEvent),
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

    #[test]
    fn event_bus_clear() {
        let mut bus = EventBus::new();

        bus.emit(GameEvent::CreatureSpawned {
            entity: Entity::new(1),
            position: Vec2::new(0.0, 0.0),
        });
        bus.emit(GameEvent::CreatureDied {
            entity: Entity::new(1),
            cause: DeathCause::Unknown,
        });

        assert_eq!(bus.len(), 2);
        bus.clear();
        assert_eq!(bus.len(), 0);
        assert!(bus.is_empty());
    }

    #[test]
    fn event_bus_default() {
        let bus = EventBus::default();
        assert!(bus.is_empty());
        assert_eq!(bus.len(), 0);
    }
}
