pub mod entity;
pub mod world;
pub mod time;
pub mod spatial;
pub mod events;
pub mod error;

pub use entity::{Entity, EntityManager};
pub use world::World;
pub use time::{GameTime, TimeSystem};
pub use spatial::{SpatialGrid, GridCoord};
pub use events::{EventBus, GameEvent};
pub use error::{ErrorBoundary, SimulationError};