pub mod entity;
pub mod error;
pub mod events;
pub mod spatial;
pub mod spatial_v2;
pub mod time;
pub mod versioned_entity;
pub mod world;

pub use entity::{Entity, EntityManager};
pub use error::{ErrorBoundary, SimulationError};
pub use events::{DeathCause, EventBus, GameEvent};
pub use spatial::{GridCoord, SpatialGrid};
pub use spatial_v2::{OptimizedSpatialPlugin, SpatialHashGrid};
pub use time::{GameTime, TimeSystem};
pub use versioned_entity::{EntityVersioningPlugin, EntityVersions, Version, VersionedEntity};
pub use world::{World, WorldBounds, WorldStats};
