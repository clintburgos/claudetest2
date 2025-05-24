use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity(u32);

impl Entity {
    pub fn new(id: u32) -> Self {
        Entity(id)
    }
    
    pub fn id(&self) -> u32 {
        self.0
    }
}

pub struct EntityManager {
    next_id: u32,
    active_entities: HashSet<Entity>,
    recycled_ids: Vec<u32>,
}

impl EntityManager {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            active_entities: HashSet::new(),
            recycled_ids: Vec::new(),
        }
    }
    
    pub fn create(&mut self) -> Entity {
        let id = self.recycled_ids.pop().unwrap_or_else(|| {
            let id = self.next_id;
            self.next_id += 1;
            id
        });
        
        let entity = Entity(id);
        self.active_entities.insert(entity);
        entity
    }
    
    pub fn destroy(&mut self, entity: Entity) -> bool {
        if self.active_entities.remove(&entity) {
            self.recycled_ids.push(entity.0);
            true
        } else {
            false
        }
    }
    
    pub fn is_alive(&self, entity: Entity) -> bool {
        self.active_entities.contains(&entity)
    }
    
    pub fn active_count(&self) -> usize {
        self.active_entities.len()
    }
    
    pub fn iter(&self) -> impl Iterator<Item = &Entity> {
        self.active_entities.iter()
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
}