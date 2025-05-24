pub mod creature;
pub mod resource;
pub mod needs;
pub mod health;

pub use creature::{Creature, CreatureState};
pub use resource::{Resource, ResourceType};
pub use needs::Needs;
pub use health::Health;