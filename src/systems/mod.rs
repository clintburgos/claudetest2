//! ECS Systems Module
//!
//! This module contains the Bevy ECS systems that drive the simulation forward.
//! Each system is responsible for updating specific aspects of the simulation
//! during each frame.
//!
//! # Module Structure
//!
//! - **`decision`** - AI decision-making system for creature behavior
//! - **`movement`** - Physics and pathfinding for creature movement
//! - **`resource_spawner`** - Manages resource spawning and distribution
//! - **`simulation`** - Main simulation orchestration and lifecycle
//!
//! # System Architecture
//!
//! Systems follow Bevy's ECS pattern:
//! - Query components for entities to process
//! - Apply transformations based on game rules
//! - Emit events for cross-system communication
//! - Maintain performance through batching and parallelization
//!
//! # Execution Order
//!
//! Systems are carefully ordered to ensure consistency:
//! 1. Input and time updates
//! 2. Decision making and AI
//! 3. Movement and physics
//! 4. Resource consumption and spawning
//! 5. Health and lifecycle updates
//! 6. Rendering and UI
//!
//! # Performance Considerations
//!
//! - Systems use parallel iteration where possible
//! - Spatial queries are optimized through indexing
//! - LOD systems reduce computation for distant entities
//! - Change detection minimizes unnecessary updates

pub mod decision;
pub mod movement;
pub mod resource_spawner;
pub mod simulation;

pub use decision::{
    Decision, DecisionContext, DecoupledDecisionPlugin, DecoupledDecisionSystem,
};
pub use movement::MovementSystem;
pub use resource_spawner::ResourceSpawner;
pub use simulation::Simulation;
