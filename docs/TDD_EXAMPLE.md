# TDD Example: First Feature Implementation

This guide walks through implementing your first feature using Test-Driven Development.

## Example: Basic ECS Entity Creation

### Step 1: Write the Test First

Create `src/core/ecs.rs` and start with tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_create_entity() {
        let mut world = World::new();
        
        let entity = world.create_entity();
        
        assert!(world.is_alive(entity));
    }

    #[test]
    fn entities_have_unique_ids() {
        let mut world = World::new();
        
        let entity1 = world.create_entity();
        let entity2 = world.create_entity();
        
        assert_ne!(entity1, entity2);
    }

    #[test]
    fn can_destroy_entity() {
        let mut world = World::new();
        let entity = world.create_entity();
        
        world.destroy_entity(entity);
        
        assert!(!world.is_alive(entity));
    }
}
```

### Step 2: Run Tests (RED Phase)

```bash
cargo test
```

Tests will fail because `World` doesn't exist yet. Good! This is the RED phase.

### Step 3: Implement Minimum Code (GREEN Phase)

Now implement just enough to make tests pass:

```rust
pub type Entity = u32;

pub struct World {
    next_entity_id: Entity,
    alive_entities: std::collections::HashSet<Entity>,
}

impl World {
    pub fn new() -> Self {
        Self {
            next_entity_id: 0,
            alive_entities: std::collections::HashSet::new(),
        }
    }

    pub fn create_entity(&mut self) -> Entity {
        let entity = self.next_entity_id;
        self.next_entity_id += 1;
        self.alive_entities.insert(entity);
        entity
    }

    pub fn is_alive(&self, entity: Entity) -> bool {
        self.alive_entities.contains(&entity)
    }

    pub fn destroy_entity(&mut self, entity: Entity) {
        self.alive_entities.remove(&entity);
    }
}
```

### Step 4: Run Tests Again (GREEN Phase)

```bash
cargo test
```

All tests should pass! ✅

### Step 5: Refactor

Now improve the code while keeping tests green:

```rust
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity(u32);

pub struct World {
    next_entity_id: u32,
    alive_entities: HashSet<Entity>,
}

impl World {
    pub fn new() -> Self {
        Self {
            next_entity_id: 0,
            alive_entities: HashSet::new(),
        }
    }

    pub fn create_entity(&mut self) -> Entity {
        let entity = Entity(self.next_entity_id);
        self.next_entity_id = self.next_entity_id
            .checked_add(1)
            .expect("Entity ID overflow");
        self.alive_entities.insert(entity);
        entity
    }

    pub fn is_alive(&self, entity: Entity) -> bool {
        self.alive_entities.contains(&entity)
    }

    pub fn destroy_entity(&mut self, entity: Entity) -> bool {
        self.alive_entities.remove(&entity)
    }
}
```

### Step 6: Add Edge Case Tests

```rust
#[test]
fn destroying_nonexistent_entity_returns_false() {
    let mut world = World::new();
    let fake_entity = Entity(999);
    
    assert!(!world.destroy_entity(fake_entity));
}

#[test]
#[should_panic(expected = "Entity ID overflow")]
fn panics_on_entity_id_overflow() {
    let mut world = World::new();
    world.next_entity_id = u32::MAX;
    
    world.create_entity(); // Should panic
}
```

## Next: Components

Now that entities work, write tests for components:

```rust
#[test]
fn can_add_component_to_entity() {
    let mut world = World::new();
    let entity = world.create_entity();
    
    world.add_component(entity, Position { x: 5.0, y: 10.0 });
    
    assert!(world.has_component::<Position>(entity));
}

#[test]
fn can_get_component_from_entity() {
    let mut world = World::new();
    let entity = world.create_entity();
    let pos = Position { x: 5.0, y: 10.0 };
    
    world.add_component(entity, pos);
    
    let retrieved = world.get_component::<Position>(entity);
    assert_eq!(retrieved, Some(&pos));
}
```

This drives the component storage implementation!

## TDD Rhythm

1. **Think**: What behavior do I want?
2. **Test**: Write a test that describes it
3. **Red**: Run test, see it fail
4. **Green**: Write minimal code to pass
5. **Refactor**: Improve code quality
6. **Repeat**: Next behavior

## Benefits You'll Notice

- ✅ Confidence that code works
- ✅ Better API design (tests force good interfaces)
- ✅ Easier to refactor later
- ✅ Documentation through tests
- ✅ Catch bugs before they happen

---
*Last Updated: 2024-01-XX*
