use std::collections::HashSet;

/// Represents a unique entity identifier in the simulation
/// 
/// Entities are lightweight IDs that can be associated with different
/// components (Creature, Resource, etc). IDs are recycled when entities
/// are destroyed to prevent unbounded growth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity(u32);

impl Entity {
    /// Creates a new entity with the given ID
    /// 
    /// # Arguments
    /// * `id` - The unique identifier for this entity
    /// 
    /// # Example
    /// ```
    /// let entity = Entity::new(42);
    /// assert_eq!(entity.id(), 42);
    /// ```
    pub fn new(id: u32) -> Self {
        Entity(id)
    }
    
    /// Returns the underlying ID value
    pub fn id(&self) -> u32 {
        self.0
    }
}

/// Manages entity lifecycle and ID allocation
/// 
/// The EntityManager handles creating and destroying entities while
/// efficiently recycling IDs to prevent unbounded growth. It maintains
/// a set of active entities and a pool of recycled IDs.
/// 
/// # Example
/// ```
/// let mut manager = EntityManager::new();
/// let entity = manager.create();
/// assert!(manager.is_alive(entity));
/// manager.destroy(entity);
/// assert!(!manager.is_alive(entity));
/// ```
pub struct EntityManager {
    next_id: u32,
    active_entities: HashSet<Entity>,
    recycled_ids: Vec<u32>,
}

impl EntityManager {
    /// Creates a new entity manager with pre-allocated capacity
    /// 
    /// Pre-allocates space for 1000 entities and 100 recycled IDs
    /// to reduce allocations during runtime.
    pub fn new() -> Self {
        Self {
            next_id: 0,
            active_entities: HashSet::with_capacity(1000),
            recycled_ids: Vec::with_capacity(100),
        }
    }
    
    /// Creates a new entity, recycling IDs when possible
    /// 
    /// # Returns
    /// A new unique Entity
    /// 
    /// # Panics
    /// Panics in debug mode if approaching u32::MAX entities
    pub fn create(&mut self) -> Entity {
        debug_assert!(
            self.next_id < u32::MAX - 10000,
            "Entity ID approaching overflow. Consider recycling more aggressively."
        );
        
        let id = self.recycled_ids.pop().unwrap_or_else(|| {
            let id = self.next_id;
            self.next_id += 1;
            id
        });
        
        let entity = Entity(id);
        self.active_entities.insert(entity);
        entity
    }
    
    /// Destroys an entity and recycles its ID
    /// 
    /// # Arguments
    /// * `entity` - The entity to destroy
    /// 
    /// # Returns
    /// `true` if the entity was active and successfully destroyed,
    /// `false` if the entity was not found
    pub fn destroy(&mut self, entity: Entity) -> bool {
        if self.active_entities.remove(&entity) {
            self.recycled_ids.push(entity.0);
            true
        } else {
            false
        }
    }
    
    /// Checks if an entity is currently active
    /// 
    /// # Arguments
    /// * `entity` - The entity to check
    /// 
    /// # Returns
    /// `true` if the entity is active, `false` otherwise
    pub fn is_alive(&self, entity: Entity) -> bool {
        self.active_entities.contains(&entity)
    }
    
    /// Returns the number of active entities
    pub fn active_count(&self) -> usize {
        self.active_entities.len()
    }
    
    /// Returns an iterator over all active entities
    pub fn iter(&self) -> impl Iterator<Item = &Entity> {
        self.active_entities.iter()
    }
    
    /// Returns the number of recycled IDs available
    pub fn recycled_count(&self) -> usize {
        self.recycled_ids.len()
    }
}

impl Default for EntityManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entity_creation() {
        let mut manager = EntityManager::new();
        let e1 = manager.create();
        let e2 = manager.create();
        
        assert_ne!(e1, e2);
        assert_eq!(manager.active_count(), 2);
        assert!(manager.is_alive(e1));
        assert!(manager.is_alive(e2));
    }
    
    #[test]
    fn entity_destruction() {
        let mut manager = EntityManager::new();
        let entity = manager.create();
        
        assert!(manager.destroy(entity));
        assert!(!manager.is_alive(entity));
        assert_eq!(manager.active_count(), 0);
        assert!(!manager.destroy(entity)); // Can't destroy twice
    }
    
    #[test]
    fn entity_id_recycling() {
        let mut manager = EntityManager::new();
        let e1 = manager.create();
        let id1 = e1.id();
        
        manager.destroy(e1);
        
        let e2 = manager.create();
        assert_eq!(e2.id(), id1); // ID should be recycled
    }
    
    #[test]
    fn entity_id_method() {
        let entity = Entity::new(42);
        assert_eq!(entity.id(), 42);
    }
    
    #[test]
    fn entity_manager_iter() {
        let mut manager = EntityManager::new();
        let e1 = manager.create();
        let e2 = manager.create();
        let e3 = manager.create();
        
        let entities: Vec<&Entity> = manager.iter().collect();
        assert_eq!(entities.len(), 3);
        assert!(entities.contains(&&e1));
        assert!(entities.contains(&&e2));
        assert!(entities.contains(&&e3));
    }
    
    #[test]
    fn entity_manager_default() {
        let manager = EntityManager::default();
        assert_eq!(manager.active_count(), 0);
    }
}