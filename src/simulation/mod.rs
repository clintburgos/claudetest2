//! Simulation Components Module
//!
//! This module defines the core components and behaviors of creatures and resources
//! within the simulation world. It models the biological and behavioral aspects of
//! digital life forms.
//!
//! # Module Structure
//!
//! - **`creature`** - Creature entities with state, traits, and lifecycle management
//! - **`health`** - Health system including damage, healing, and death mechanics
//! - **`needs`** - Biological needs like hunger, thirst, and energy
//! - **`resource`** - Environmental resources that creatures consume
//!
//! # Creature Model
//!
//! Creatures in the simulation are complex entities with:
//! - Physical traits (size, speed, strength)
//! - Biological needs that must be satisfied
//! - Health that degrades from damage or unmet needs
//! - State machines for behavior control
//!
//! # Resource System
//!
//! Resources represent consumables in the environment:
//! - Food sources provide nutrition
//! - Water sources satisfy thirst
//! - Resources respawn based on biome characteristics
//! - Scarcity drives creature competition and migration

pub mod creature;
pub mod health;
pub mod needs;
pub mod resource;

pub use creature::{Creature, CreatureState};
pub use health::Health;
pub use needs::Needs;
pub use resource::{Resource, ResourceType};
