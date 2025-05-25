//! Versioned Entity System for tracking entity lifecycle and preventing stale references.
//!
//! This module provides an enhanced entity system with generation tracking to detect
//! when entity references become invalid. This prevents crashes from accessing despawned
//! entities and enables safe caching of entity references.

use bevy::ecs::query::{ReadOnlyWorldQuery, WorldQuery};
use bevy::prelude::*;
use std::sync::atomic::{AtomicU32, Ordering};

/// A versioned entity reference that includes both ID and generation.
///
/// The generation number increments each time an entity ID is recycled,
/// allowing detection of stale references to despawned entities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VersionedEntity {
    /// The entity ID
    pub id: u32,
    /// The generation number for this entity
    pub generation: u32,
}

impl VersionedEntity {
    pub fn new(id: u32, generation: u32) -> Self {
        Self { id, generation }
    }
}

/// Component wrapper that adds version tracking to any data type.
///
/// This allows systems to detect when components have been modified
/// and avoid unnecessary updates.
#[derive(Component, Debug, Clone)]
pub struct Version<T> {
    pub generation: u32,
    pub data: T,
}

impl<T> Version<T> {
    pub fn new(data: T) -> Self {
        Self {
            generation: 0,
            data,
        }
    }

    pub fn increment(&mut self) {
        self.generation = self.generation.wrapping_add(1);
    }

    pub fn with_generation(data: T, generation: u32) -> Self {
        Self { generation, data }
    }
}

impl<T> std::ops::Deref for Version<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> std::ops::DerefMut for Version<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.increment(); // Auto-increment on mutation
        &mut self.data
    }
}

/// Resource that tracks entity generations for version validation.
#[derive(Resource, Default)]
pub struct EntityVersions {
    /// Maps entity IDs to their current generation
    versions: dashmap::DashMap<u32, u32>,
    /// Next available entity ID
    next_id: AtomicU32,
    /// Stack of recycled entity IDs
    recycled: parking_lot::Mutex<Vec<u32>>,
}

impl EntityVersions {
    pub fn new() -> Self {
        Self::default()
    }

    /// Allocates a new versioned entity, recycling IDs when possible.
    pub fn allocate(&self) -> VersionedEntity {
        let (id, generation) = if let Some(recycled_id) = self.recycled.lock().pop() {
            // Reuse a recycled ID with the current generation (already incremented on deallocation)
            let generation = self.versions.get(&recycled_id).map(|g| *g).unwrap_or(1);
            (recycled_id, generation)
        } else {
            // Allocate a new ID
            let id = self.next_id.fetch_add(1, Ordering::Relaxed);
            (id, 0)
        };

        self.versions.insert(id, generation);
        VersionedEntity { id, generation }
    }

    /// Deallocates an entity, making its ID available for recycling.
    pub fn deallocate(&self, entity: VersionedEntity) -> bool {
        if self.is_valid(entity) {
            // Increment generation to invalidate all existing references
            self.versions.insert(entity.id, entity.generation + 1);
            self.recycled.lock().push(entity.id);
            true
        } else {
            false
        }
    }

    /// Checks if a versioned entity reference is still valid.
    pub fn is_valid(&self, entity: VersionedEntity) -> bool {
        self.versions
            .get(&entity.id)
            .map(|gen| *gen == entity.generation)
            .unwrap_or(false)
    }

    /// Gets the current generation for an entity ID.
    pub fn get_generation(&self, id: u32) -> Option<u32> {
        self.versions.get(&id).map(|g| *g)
    }
}

// Simplified version - the full VersionedQuery would require more complex type handling
// For now, users should validate entities manually using EntityVersions

/// Trait for converting between Bevy entities and versioned entities.
pub trait EntityVersioning {
    fn to_versioned(&self, versions: &EntityVersions) -> Option<VersionedEntity>;
    fn from_versioned(entity: VersionedEntity) -> Entity;
}

impl EntityVersioning for Entity {
    fn to_versioned(&self, versions: &EntityVersions) -> Option<VersionedEntity> {
        let id = self.index();
        versions.get_generation(id).map(|generation| VersionedEntity { id, generation })
    }

    fn from_versioned(entity: VersionedEntity) -> Entity {
        Entity::from_raw(entity.id)
    }
}

/// Plugin that adds entity versioning support to Bevy.
pub struct EntityVersioningPlugin;

impl Plugin for EntityVersioningPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EntityVersions>().add_systems(First, update_entity_versions);
    }
}

/// System that syncs Bevy entity spawns/despawns with our versioning system.
fn update_entity_versions(
    mut commands: Commands,
    versions: Res<EntityVersions>,
    mut removed: RemovedComponents<Version<()>>,
) {
    // Handle entity despawns
    for entity in removed.read() {
        if let Some(versioned) = entity.to_versioned(&versions) {
            versions.deallocate(versioned);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_allocation() {
        let versions = EntityVersions::new();

        let e1 = versions.allocate();
        let e2 = versions.allocate();

        assert_ne!(e1.id, e2.id);
        assert_eq!(e1.generation, 0);
        assert_eq!(e2.generation, 0);
    }

    #[test]
    fn test_entity_recycling() {
        let versions = EntityVersions::new();

        let e1 = versions.allocate();
        let id1 = e1.id;

        assert!(versions.deallocate(e1));

        let e2 = versions.allocate();
        assert_eq!(e2.id, id1); // Same ID
        assert_eq!(e2.generation, 1); // Incremented generation
    }

    #[test]
    fn test_version_validation() {
        let versions = EntityVersions::new();

        let e1 = versions.allocate();
        assert!(versions.is_valid(e1));

        // Create a stale reference
        let stale = VersionedEntity::new(e1.id, e1.generation + 1);
        assert!(!versions.is_valid(stale));
    }

    #[test]
    fn test_version_component() {
        let mut pos = Version::new((0.0, 0.0));
        assert_eq!(pos.generation, 0);

        // Mutation auto-increments version
        pos.data.0 = 5.0;
        assert_eq!(pos.generation, 0); // Direct access doesn't increment

        // DerefMut increments
        pos.0 = 10.0;
        assert_eq!(pos.generation, 1);
    }

    #[test]
    fn test_get_generation() {
        let versions = EntityVersions::new();

        let entity = versions.allocate();
        assert_eq!(versions.get_generation(entity.id), Some(0));

        // Non-existent entity
        assert_eq!(versions.get_generation(999), None);

        // After deallocation, generation increments
        versions.deallocate(entity);
        assert_eq!(versions.get_generation(entity.id), Some(1));
    }

    #[test]
    fn test_multiple_recycled_ids() {
        let versions = EntityVersions::new();

        // Allocate and deallocate multiple entities
        let e1 = versions.allocate();
        let e2 = versions.allocate();
        let e3 = versions.allocate();

        versions.deallocate(e2);
        versions.deallocate(e1);
        versions.deallocate(e3);

        // Should reuse in LIFO order (3, 1, 2)
        let reused1 = versions.allocate();
        assert_eq!(reused1.id, e3.id);
        assert_eq!(reused1.generation, 1);

        let reused2 = versions.allocate();
        assert_eq!(reused2.id, e1.id);
        assert_eq!(reused2.generation, 1);

        let reused3 = versions.allocate();
        assert_eq!(reused3.id, e2.id);
        assert_eq!(reused3.generation, 1);
    }

    #[test]
    fn test_entity_versioning_trait() {
        // The trait adds methods to Entity
        let entity = bevy::ecs::entity::Entity::from_raw(42);
        let versions = EntityVersions::new();

        // Test to_versioned method
        let versioned = entity.to_versioned(&versions);
        assert_eq!(versioned, None); // Entity not in version system yet

        // Allocate an entity with specific ID
        let allocated = versions.allocate();
        // Can't directly test with_generation as it's not implemented
    }

    #[test]
    fn test_concurrent_allocation() {
        use std::sync::Arc;
        use std::thread;

        let versions = Arc::new(EntityVersions::new());
        let mut handles = vec![];

        // Spawn multiple threads allocating entities
        for _ in 0..10 {
            let versions_clone = Arc::clone(&versions);
            let handle = thread::spawn(move || {
                let mut entities = vec![];
                for _ in 0..100 {
                    entities.push(versions_clone.allocate());
                }
                entities
            });
            handles.push(handle);
        }

        // Collect all allocated entities
        let mut all_entities = vec![];
        for handle in handles {
            all_entities.extend(handle.join().unwrap());
        }

        // Check that all IDs are unique
        let mut ids: Vec<_> = all_entities.iter().map(|e| e.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 1000); // Should have 1000 unique IDs
    }

    #[test]
    fn test_deallocate_invalid() {
        let versions = EntityVersions::new();

        // Can't deallocate non-existent entity
        let invalid = VersionedEntity::new(999, 0);
        assert!(!versions.deallocate(invalid));

        // Can't deallocate with wrong generation
        let entity = versions.allocate();
        let wrong_gen = VersionedEntity::new(entity.id, entity.generation + 1);
        assert!(!versions.deallocate(wrong_gen));
    }
}
