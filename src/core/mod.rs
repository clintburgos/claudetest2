pub mod entity;
pub mod world;
pub mod time;
pub mod spatial;
pub mod events;
pub mod error;

pub use entity::{Entity, EntityManager};
pub use world::{World, WorldBounds, WorldStats};
pub use time::{GameTime, TimeSystem};
pub use spatial::{SpatialGrid, GridCoord};
pub use events::{EventBus, GameEvent, DeathCause};
pub use error::{ErrorBoundary, SimulationError};