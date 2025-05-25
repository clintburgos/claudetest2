//! Core Systems Module
//!
//! This module contains the fundamental building blocks of the creature simulation.
//! It provides essential infrastructure that all other systems depend on.
//!
//! # Module Structure
//!
//! - **`entity`** - Entity management system for tracking all simulation objects
//! - **`error`** - Error types and error handling infrastructure
//! - **`events`** - Event bus for inter-system communication
//! - **`spatial`** - Spatial indexing for efficient position-based queries
//! - **`time`** - Game time management and time scaling
//! - **`world`** - World representation, bounds, and statistics
//!
//! # Design Philosophy
//!
//! The core module emphasizes:
//! - **Performance**: Data structures optimized for cache locality and parallel access
//! - **Modularity**: Clean interfaces between systems with minimal coupling
//! - **Scalability**: Designed to handle thousands of entities efficiently
//!
//! # Key Abstractions
//!
//! - `Entity` and `EntityManager` provide unique identification and lifecycle management
//! - `SpatialGrid` enables O(log n) neighbor queries for movement and interaction
//! - `EventBus` facilitates decoupled communication between systems
//! - `GameTime` handles variable time scaling from pause to 1000x speed

pub mod entity;
pub mod error;
pub mod events;
pub mod spatial;
pub mod time;
pub mod world;

pub use entity::{Entity, EntityManager};
pub use error::{ErrorBoundary, SimulationError};
pub use events::{DeathCause, EventBus, GameEvent};
pub use spatial::{GridCoord, SpatialGrid};
pub use time::{GameTime, TimeSystem};
pub use world::{World, WorldBounds, WorldStats};
