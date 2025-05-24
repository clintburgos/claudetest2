# Save/Load System Architecture

## Overview

The save/load system provides persistent storage for simulation state, supporting quick saves, autosaves, and version migration. The system uses a hybrid approach combining memory-mapped files for performance with compressed archives for long-term storage.

## Save File Structure

### File Format

Save files use a custom binary format with the following structure:

```
CRTR [4 bytes] - Magic number
Version [4 bytes] - Save format version
Header [variable] - Metadata and indices
Data Chunks [variable] - Compressed data sections
Checksum [32 bytes] - SHA-256 of all data
```

### Save File Layout

```
creature_sim_save_v1.crs
├── header.bin (uncompressed)
│   ├── Version info
│   ├── Creation timestamp
│   ├── Simulation time
│   ├── World seed
│   ├── Creature count
│   └── Chunk directory
├── world_data.lz4
│   ├── Terrain
│   ├── Resources
│   ├── Weather state
│   └── Spatial indices
├── creatures_data.lz4
│   ├── Entity IDs
│   ├── Components (columnar)
│   └── Relationships
├── systems_data.lz4
│   ├── Group memberships
│   ├── Territory claims
│   ├── Knowledge bases
│   └── Event queues
└── metadata.json (human-readable)
```

## Serialization Strategy

### Component Serialization

Components are serialized using a columnar format for better compression:

```rust
// Instead of Array of Structs (AoS)
struct CreatureSave {
    id: EntityId,
    position: Vec3,
    health: f32,
    // ... more fields
}
creatures: Vec<CreatureSave>

// Use Struct of Arrays (SoA)
struct CreaturesSave {
    ids: Vec<EntityId>,
    positions: Vec<Vec3>,
    healths: Vec<f32>,
    // ... more arrays
}
```

### Entity Reference Resolution

Entity references are converted to stable IDs during save:

```rust
pub struct EntityRef {
    generation: u32,
    index: u32,
}

pub struct SaveEntityMap {
    entity_to_save_id: HashMap<Entity, u64>,
    save_id_to_entity: HashMap<u64, Entity>,
}
```

### Memory Mapping Strategy

For large worlds, use memory-mapped files during save generation:

```rust
pub struct SaveWriter {
    mmap: MmapMut,
    offset: usize,
    chunk_index: Vec<ChunkDescriptor>,
}

pub struct ChunkDescriptor {
    chunk_type: ChunkType,
    offset: u64,
    compressed_size: u32,
    uncompressed_size: u32,
}
```

## Version Migration

### Version Compatibility

```rust
pub enum VersionCompatibility {
    FullyCompatible,      // Can load directly
    ForwardCompatible,    // Newer save, can load with defaults
    BackwardCompatible,   // Older save, needs migration
    Incompatible,         // Cannot load
}

pub trait SaveMigration {
    fn from_version(&self) -> u32;
    fn to_version(&self) -> u32;
    fn migrate(&self, data: &mut SaveData) -> Result<(), MigrationError>;
}
```

### Migration Pipeline

```rust
pub struct MigrationPipeline {
    migrations: Vec<Box<dyn SaveMigration>>,
}

impl MigrationPipeline {
    pub fn migrate(&self, save: &mut SaveData, target_version: u32) -> Result<()> {
        let mut current_version = save.version;
        
        while current_version < target_version {
            let migration = self.find_migration(current_version)?;
            migration.migrate(save)?;
            current_version = migration.to_version();
        }
        
        Ok(())
    }
}
```

## Save/Load Operations

### Save Process

1. **Pause Simulation** - Ensure consistent state
2. **Prepare Save Data**
   - Generate entity ID mappings
   - Snapshot component data
   - Capture system states
3. **Serialize Data**
   - Convert to columnar format
   - Resolve entity references
   - Compress chunks with LZ4
4. **Write to Disk**
   - Create temporary file
   - Write header and chunks
   - Calculate checksum
   - Atomic rename to final location
5. **Resume Simulation**

### Load Process

1. **Validate Save File**
   - Check magic number
   - Verify checksum
   - Check version compatibility
2. **Load Header**
   - Read metadata
   - Build chunk index
3. **Clear Current State**
   - Stop simulation
   - Clear all entities
   - Reset systems
4. **Deserialize Data**
   - Decompress chunks
   - Create entities
   - Restore components
   - Rebuild relationships
5. **Initialize Systems**
   - Rebuild spatial indices
   - Restore system states
   - Validate integrity
6. **Start Simulation**

### Autosave System

```rust
pub struct AutosaveConfig {
    enabled: bool,
    interval: Duration,
    keep_count: usize,
    compress: bool,
}

pub struct AutosaveManager {
    config: AutosaveConfig,
    last_save: Instant,
    save_queue: VecDeque<PathBuf>,
}
```

Autosave triggers:
- Time-based (configurable interval, default 5 minutes)
- Event-based (major creature death, world events)
- Before risky operations (time skip, mass events)

### Quick Save/Load

Quick saves use uncompressed format for speed:

```rust
pub struct QuickSave {
    // Only essential data for fast save/load
    creature_positions: Vec<Vec3>,
    creature_states: Vec<CreatureState>,
    resource_amounts: Vec<u32>,
    current_time: f64,
}
```

## Performance Considerations

### Compression Settings

| Data Type | Compression | Ratio | Speed |
|-----------|-------------|-------|-------|
| Positions | LZ4 HC | 3:1 | Fast |
| Genetics | ZSTD | 10:1 | Medium |
| Terrain | RLE + LZ4 | 20:1 | Fast |
| Strings | ZSTD Dict | 15:1 | Medium |

### Parallel Processing

```rust
pub async fn save_parallel(world: &World) -> Result<SaveData> {
    let (creatures, world_data, systems) = tokio::join!(
        save_creatures_async(world),
        save_world_async(world),
        save_systems_async(world)
    );
    
    Ok(SaveData {
        creatures: creatures?,
        world: world_data?,
        systems: systems?,
    })
}
```

### Incremental Saves

For very large simulations, support incremental saves:

```rust
pub struct IncrementalSave {
    base_save: PathBuf,
    deltas: Vec<SaveDelta>,
}

pub struct SaveDelta {
    timestamp: u64,
    changed_entities: HashSet<Entity>,
    changed_chunks: HashSet<ChunkId>,
}
```

## Save File Management

### Save Slots

```rust
pub struct SaveSlot {
    id: String,
    name: String,
    timestamp: DateTime<Utc>,
    play_time: Duration,
    creature_count: usize,
    world_seed: u64,
    screenshot: Option<PathBuf>,
}

pub struct SaveManager {
    save_directory: PathBuf,
    slots: Vec<SaveSlot>,
    max_slots: usize,
}
```

### Cloud Save Support

Future-proofing for cloud saves:

```rust
pub trait SaveStorage {
    fn list_saves(&self) -> Result<Vec<SaveInfo>>;
    fn load_save(&self, id: &str) -> Result<SaveData>;
    fn store_save(&self, id: &str, data: &SaveData) -> Result<()>;
    fn delete_save(&self, id: &str) -> Result<()>;
}

pub struct LocalStorage { /* ... */ }
pub struct CloudStorage { /* ... */ }
```

## Error Handling

### Save Corruption Recovery

```rust
pub struct SaveValidator {
    fn validate_save(&self, path: &Path) -> ValidationResult {
        // Check file integrity
        // Verify chunk checksums
        // Validate entity references
        // Check data constraints
    }
    
    fn attempt_recovery(&self, path: &Path) -> Result<SaveData> {
        // Try to recover partial data
        // Skip corrupted chunks
        // Rebuild missing indices
    }
}
```

### Rollback Support

Keep previous save for rollback:

```rust
pub fn save_with_backup(path: &Path, data: &SaveData) -> Result<()> {
    let backup = path.with_extension("bak");
    
    // Rename current save to backup
    if path.exists() {
        fs::rename(path, &backup)?;
    }
    
    // Try to save
    match write_save(path, data) {
        Ok(_) => {
            // Success, remove backup
            fs::remove_file(backup).ok();
            Ok(())
        }
        Err(e) => {
            // Failed, restore backup
            fs::rename(&backup, path)?;
            Err(e)
        }
    }
}
```

## Testing Strategy

### Save/Load Tests

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_save_load_roundtrip() {
        let world = create_test_world();
        let save_data = save_world(&world).unwrap();
        let loaded_world = load_world(&save_data).unwrap();
        assert_worlds_equal(&world, &loaded_world);
    }
    
    #[test]
    fn test_version_migration() {
        let old_save = load_test_save("v1_save.crs");
        let migrated = migrate_save(old_save, CURRENT_VERSION).unwrap();
        assert_eq!(migrated.version, CURRENT_VERSION);
    }
    
    #[test]
    fn test_corruption_recovery() {
        let corrupted = load_test_save("corrupted.crs");
        let recovered = recover_save(corrupted).unwrap();
        assert!(recovered.is_playable());
    }
}
```

## Implementation Priority

1. **Phase 1**: Basic save/load
   - Simple binary format
   - Full world serialization
   - No compression

2. **Phase 2**: Optimization
   - Compression support
   - Parallel save/load
   - Memory mapping

3. **Phase 3**: Advanced features
   - Version migration
   - Incremental saves
   - Cloud save prep