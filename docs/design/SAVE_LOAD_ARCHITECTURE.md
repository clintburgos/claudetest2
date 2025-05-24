# Save/Load System Architecture

## Overview

The save/load system handles world persistence, save file management, and version compatibility. It must efficiently serialize large worlds while maintaining backward compatibility and handling corruption gracefully.

## Architecture Design

### Save File Structure
```rust
#[derive(Serialize, Deserialize)]
struct SaveFile {
    header: SaveHeader,
    world_data: CompressedData<WorldState>,
    creature_data: CompressedData<Vec<CreatureState>>,
    resource_data: CompressedData<Vec<ResourceState>>,
    metadata: SaveMetadata,
}

#[derive(Serialize, Deserialize)]
struct SaveHeader {
    version: SaveVersion,
    magic_number: u32, // 0x53494D55 ("SIMU")
    timestamp: DateTime<Utc>,
    world_seed: u64,
    world_size: (u32, u32),
    creature_count: u32,
    play_time: Duration,
    checksum: u64,
}

#[derive(Serialize, Deserialize)]
struct SaveVersion {
    major: u16, // Breaking changes
    minor: u16, // New features
    patch: u16, // Bug fixes
}
```

### Compression Strategy
```rust
struct CompressedData<T> {
    compression_type: CompressionType,
    uncompressed_size: usize,
    compressed_data: Vec<u8>,
    _phantom: PhantomData<T>,
}

enum CompressionType {
    None,
    Zstd { level: i32 },      // Default, good balance
    Lz4 { acceleration: i32 }, // Fast, lower ratio
    Brotli { quality: u32 },   // Slow, best ratio
}

impl<T: Serialize + DeserializeOwned> CompressedData<T> {
    fn compress(data: &T, compression: CompressionType) -> Result<Self> {
        let serialized = bincode::serialize(data)?;
        let uncompressed_size = serialized.len();
        
        let compressed_data = match compression {
            CompressionType::None => serialized,
            CompressionType::Zstd { level } => {
                zstd::encode_all(&serialized[..], level)?
            },
            CompressionType::Lz4 { acceleration } => {
                lz4::compress(&serialized, acceleration)?
            },
            CompressionType::Brotli { quality } => {
                brotli::compress(&serialized, quality)?
            },
        };
        
        Ok(CompressedData {
            compression_type: compression,
            uncompressed_size,
            compressed_data,
            _phantom: PhantomData,
        })
    }
}
```

## Serialization Strategy

### World State Chunking
```rust
#[derive(Serialize, Deserialize)]
struct WorldState {
    chunks: HashMap<ChunkCoord, WorldChunk>,
    global_state: GlobalWorldState,
}

#[derive(Serialize, Deserialize)]
struct WorldChunk {
    terrain: TerrainData,
    structures: Vec<Structure>,
    last_modified: Instant,
    version: u32,
}

#[derive(Serialize, Deserialize)]
struct GlobalWorldState {
    time_of_day: f32,
    current_season: Season,
    weather: Weather,
    total_days_elapsed: u32,
}

// Only save modified chunks
fn save_world_delta(
    world: &World,
    last_save: &SaveState,
) -> HashMap<ChunkCoord, WorldChunk> {
    world.chunks
        .iter()
        .filter(|(coord, chunk)| {
            chunk.last_modified > last_save.timestamp
        })
        .map(|(coord, chunk)| (*coord, chunk.to_save_format()))
        .collect()
}
```

### Creature State Serialization
```rust
#[derive(Serialize, Deserialize)]
struct CreatureState {
    // Identity
    id: CreatureId,
    name: Option<String>,
    
    // Core attributes
    position: Vec2,
    rotation: f32,
    health: f32,
    energy: f32,
    age: f32,
    
    // Genetics (compressed)
    dna: CompressedDNA,
    
    // Needs
    hunger: f32,
    thirst: f32,
    social: f32,
    safety: f32,
    
    // Relationships (IDs only)
    relationships: Vec<(CreatureId, RelationshipType, f32)>,
    
    // Memory (limited)
    recent_memories: CircularBuffer<Memory>,
    important_memories: Vec<Memory>,
    
    // Stats
    lifetime_stats: CreatureStats,
}

// Compress DNA to save space
#[derive(Serialize, Deserialize)]
struct CompressedDNA {
    // Store as bit-packed array instead of full floats
    genes: BitVec,
    mutation_points: Vec<(u16, u8)>, // position, value
}
```

### Resource State
```rust
#[derive(Serialize, Deserialize)]
struct ResourceState {
    resource_type: ResourceType,
    position: Vec2,
    current_amount: f32,
    quality: f32,
    regenerating: bool,
    last_consumed: Option<Instant>,
}
```

## Version Compatibility

### Migration System
```rust
trait SaveMigration {
    fn from_version(&self) -> SaveVersion;
    fn to_version(&self) -> SaveVersion;
    fn migrate(&self, data: &mut SaveData) -> Result<()>;
}

struct MigrationChain {
    migrations: Vec<Box<dyn SaveMigration>>,
}

impl MigrationChain {
    fn migrate_save(
        &self,
        save: SaveFile,
        target_version: SaveVersion,
    ) -> Result<SaveFile> {
        let mut current_version = save.header.version;
        let mut data = save;
        
        while current_version < target_version {
            let migration = self.find_migration(current_version)?;
            migration.migrate(&mut data)?;
            current_version = migration.to_version();
        }
        
        Ok(data)
    }
}

// Example migration
struct MigrationV1ToV2;
impl SaveMigration for MigrationV1ToV2 {
    fn migrate(&self, data: &mut SaveData) -> Result<()> {
        // Add new fields with defaults
        for creature in &mut data.creatures {
            creature.energy = 100.0; // New field
        }
        Ok(())
    }
}
```

### Backward Compatibility Rules
1. Never remove fields, only deprecate
2. Always provide defaults for new fields
3. Use version-specific serialization
4. Keep migration code for 10 major versions

## Performance Optimizations

### Incremental Saving
```rust
struct IncrementalSaveSystem {
    save_queue: VecDeque<SaveTask>,
    background_thread: Option<JoinHandle<()>>,
    current_snapshot: Arc<RwLock<WorldSnapshot>>,
}

enum SaveTask {
    Creature(CreatureId),
    Chunk(ChunkCoord),
    Resources(Vec<ResourceId>),
    GlobalState,
}

impl IncrementalSaveSystem {
    fn queue_save(&mut self, task: SaveTask) {
        self.save_queue.push_back(task);
        self.wake_background_thread();
    }
    
    fn background_save_loop(&self) {
        while let Some(task) = self.save_queue.pop_front() {
            match task {
                SaveTask::Creature(id) => {
                    self.save_creature(id);
                },
                SaveTask::Chunk(coord) => {
                    self.save_chunk(coord);
                },
                // ...
            }
            
            // Yield to prevent blocking
            thread::sleep(Duration::from_millis(1));
        }
    }
}
```

### Memory-Mapped Files
```rust
use memmap2::{MmapMut, MmapOptions};

struct MemoryMappedSave {
    file: File,
    mmap: MmapMut,
    header_size: usize,
}

impl MemoryMappedSave {
    fn new(path: &Path, size: usize) -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;
            
        file.set_len(size as u64)?;
        
        let mmap = unsafe {
            MmapOptions::new()
                .map_mut(&file)?
        };
        
        Ok(MemoryMappedSave {
            file,
            mmap,
            header_size: 1024, // Reserve for header
        })
    }
    
    fn write_section(&mut self, offset: usize, data: &[u8]) -> Result<()> {
        let start = self.header_size + offset;
        let end = start + data.len();
        
        self.mmap[start..end].copy_from_slice(data);
        self.mmap.flush_range(start, data.len())?;
        
        Ok(())
    }
}
```

## Save Management

### Auto-Save System
```rust
struct AutoSaveConfig {
    interval: Duration,        // 5 minutes
    keep_count: usize,        // Keep last 3 auto-saves
    save_on_quit: bool,       // true
    save_before_risky: bool,  // Save before migrations
}

struct AutoSaveManager {
    config: AutoSaveConfig,
    last_save: Instant,
    save_slots: CircularBuffer<SaveSlot>,
}

impl AutoSaveManager {
    fn update(&mut self, world: &World, delta_time: Duration) {
        if self.last_save.elapsed() >= self.config.interval {
            self.perform_autosave(world);
        }
    }
    
    fn perform_autosave(&mut self, world: &World) {
        let slot = self.get_next_slot();
        
        // Save in background
        let world_snapshot = world.create_snapshot();
        thread::spawn(move || {
            save_to_slot(world_snapshot, slot);
        });
        
        self.last_save = Instant::now();
    }
}
```

### Save Slots
```rust
#[derive(Serialize, Deserialize)]
struct SaveSlot {
    slot_number: u8,
    save_name: String,
    save_type: SaveType,
    thumbnail: Option<Vec<u8>>, // Small world preview
    metadata: SaveMetadata,
}

enum SaveType {
    Manual,
    AutoSave,
    QuickSave,
    Checkpoint,
}

#[derive(Serialize, Deserialize)]
struct SaveMetadata {
    created_at: DateTime<Utc>,
    play_time: Duration,
    world_stats: WorldStats,
    achievements: Vec<Achievement>,
    user_notes: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct WorldStats {
    total_creatures: u32,
    living_creatures: u32,
    total_births: u64,
    total_deaths: u64,
    unique_species: u32,
    resources_consumed: u64,
    conversations_had: u64,
}
```

## Error Handling

### Corruption Detection
```rust
fn verify_save_integrity(save: &SaveFile) -> Result<()> {
    // Verify magic number
    if save.header.magic_number != SAVE_MAGIC_NUMBER {
        return Err(SaveError::InvalidFormat);
    }
    
    // Verify checksum
    let calculated = calculate_checksum(save);
    if calculated != save.header.checksum {
        return Err(SaveError::ChecksumMismatch);
    }
    
    // Verify data decompression
    save.world_data.verify_decompression()?;
    save.creature_data.verify_decompression()?;
    save.resource_data.verify_decompression()?;
    
    // Verify data bounds
    if save.header.creature_count != save.creature_data.len() {
        return Err(SaveError::DataMismatch);
    }
    
    Ok(())
}
```

### Recovery Strategies
```rust
enum RecoveryStrategy {
    LoadBackup,
    PartialLoad { skip_corrupted: bool },
    RegenerateFromSeed,
    StartNew,
}

fn attempt_recovery(
    save_path: &Path,
    strategy: RecoveryStrategy,
) -> Result<SaveFile> {
    match strategy {
        RecoveryStrategy::LoadBackup => {
            let backup = find_latest_backup(save_path)?;
            load_save_file(backup)
        },
        RecoveryStrategy::PartialLoad { skip_corrupted } => {
            let mut save = load_raw_save(save_path)?;
            repair_corrupted_sections(&mut save, skip_corrupted)?;
            Ok(save)
        },
        RecoveryStrategy::RegenerateFromSeed => {
            let header = load_header_only(save_path)?;
            regenerate_world(header.world_seed, header.world_size)
        },
        RecoveryStrategy::StartNew => {
            Err(SaveError::Unrecoverable)
        },
    }
}
```

## Large World Handling

### Streaming System
```rust
struct StreamingSaveSystem {
    active_chunks: HashMap<ChunkCoord, WorldChunk>,
    chunk_cache: LruCache<ChunkCoord, WorldChunk>,
    save_file: MemoryMappedSave,
    chunk_index: ChunkIndex,
}

#[derive(Serialize, Deserialize)]
struct ChunkIndex {
    entries: HashMap<ChunkCoord, ChunkEntry>,
}

#[derive(Serialize, Deserialize)]
struct ChunkEntry {
    file_offset: u64,
    compressed_size: u32,
    uncompressed_size: u32,
    last_modified: DateTime<Utc>,
}

impl StreamingSaveSystem {
    fn load_chunk(&mut self, coord: ChunkCoord) -> Result<&WorldChunk> {
        // Check active chunks
        if self.active_chunks.contains_key(&coord) {
            return Ok(&self.active_chunks[&coord]);
        }
        
        // Check cache
        if let Some(chunk) = self.chunk_cache.get(&coord) {
            self.active_chunks.insert(coord, chunk.clone());
            return Ok(&self.active_chunks[&coord]);
        }
        
        // Load from disk
        let entry = self.chunk_index.entries.get(&coord)
            .ok_or(SaveError::ChunkNotFound)?;
            
        let chunk = self.load_chunk_from_disk(entry)?;
        self.active_chunks.insert(coord, chunk);
        
        Ok(&self.active_chunks[&coord])
    }
}
```