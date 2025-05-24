# Mod System API

## Overview

The mod system provides a safe, sandboxed environment for extending the simulation without compromising core functionality or performance. While not implemented in the initial release, the architecture is designed to support modding in the future.

## Design Principles

- **Safety First**: Mods cannot crash the simulation or corrupt saves
- **Performance Conscious**: Mods have resource limits and performance budgets
- **Compatibility**: Multiple mods can coexist without conflicts
- **Discoverability**: Easy to find, install, and manage mods
- **Documentation**: Clear API with examples and best practices

## Mod Architecture

### Mod Definition

```rust
pub struct ModDefinition {
    pub metadata: ModMetadata,
    pub dependencies: Vec<ModDependency>,
    pub capabilities: ModCapabilities,
    pub entry_points: ModEntryPoints,
}

pub struct ModMetadata {
    pub id: ModId,
    pub name: String,
    pub version: Version,
    pub author: String,
    pub description: String,
    pub license: License,
    pub tags: Vec<ModTag>,
}

pub struct ModDependency {
    pub mod_id: ModId,
    pub version_requirement: VersionReq,
    pub optional: bool,
}

pub struct ModCapabilities {
    pub max_memory: usize,
    pub max_cpu_time_per_frame: Duration,
    pub allowed_systems: HashSet<SystemType>,
    pub resource_access: ResourceAccess,
}

pub struct ModEntryPoints {
    pub on_load: Option<Symbol<fn(&mut ModContext)>>,
    pub on_enable: Option<Symbol<fn(&mut ModContext)>>,
    pub on_disable: Option<Symbol<fn(&mut ModContext)>>,
    pub on_unload: Option<Symbol<fn(&mut ModContext)>>,
    pub systems: Vec<ModSystem>,
}
```

### Mod Sandboxing

```rust
pub struct ModSandbox {
    pub memory_limit: usize,
    pub current_memory: AtomicUsize,
    pub cpu_budget: Duration,
    pub api_restrictions: ApiRestrictions,
    pub resource_quotas: ResourceQuotas,
}

pub struct ApiRestrictions {
    pub denied_functions: HashSet<String>,
    pub allowed_crates: HashSet<String>,
    pub max_entity_spawn_rate: f32,
    pub max_event_rate: f32,
}

pub struct ResourceQuotas {
    pub max_textures: u32,
    pub max_texture_memory: usize,
    pub max_sounds: u32,
    pub max_particles_per_frame: u32,
    pub max_ui_elements: u32,
}

impl ModSandbox {
    pub fn check_operation<T>(&self, op: ModOperation) -> Result<T, ModError> {
        // Check memory
        if self.current_memory.load(Ordering::Relaxed) > self.memory_limit {
            return Err(ModError::MemoryLimitExceeded);
        }
        
        // Check CPU budget
        if self.exceeded_cpu_budget() {
            return Err(ModError::CpuBudgetExceeded);
        }
        
        // Check specific operation
        match op {
            ModOperation::SpawnEntity => self.check_spawn_rate(),
            ModOperation::EmitEvent => self.check_event_rate(),
            ModOperation::AllocateTexture(size) => self.check_texture_quota(size),
            // ... other operations
        }
    }
}
```

## Mod API

### Core Systems Access

```rust
pub trait ModAPI {
    // Entity management
    fn spawn_entity(&mut self, bundle: ModEntityBundle) -> Result<Entity, ModError>;
    fn despawn_entity(&mut self, entity: Entity) -> Result<(), ModError>;
    fn get_component<T: Component>(&self, entity: Entity) -> Option<&T>;
    fn set_component<T: Component>(&mut self, entity: Entity, component: T) -> Result<(), ModError>;
    
    // Event system
    fn emit_event(&mut self, event: ModEvent) -> Result<(), ModError>;
    fn subscribe_event<E: Event>(&mut self, handler: ModEventHandler<E>) -> EventSubscription;
    
    // Resource access
    fn get_resource<R: Resource>(&self) -> Option<&R>;
    fn insert_resource<R: Resource>(&mut self, resource: R) -> Result<(), ModError>;
    
    // World queries
    fn query_entities(&self, filter: EntityFilter) -> Vec<Entity>;
    fn query_area(&self, area: AABB) -> Vec<Entity>;
}

pub struct ModContext {
    api: Box<dyn ModAPI>,
    mod_id: ModId,
    persistent_data: ModStorage,
    config: ModConfig,
}
```

### Component Extensions

```rust
// Mods can define new components
#[derive(ModComponent)]
pub struct CustomComponent {
    pub data: String,
    pub value: f32,
}

// Trait that mod components must implement
pub trait ModComponent: Send + Sync + 'static {
    fn type_id() -> ModComponentId;
    fn serialize(&self) -> Result<Vec<u8>, SerializeError>;
    fn deserialize(data: &[u8]) -> Result<Self, DeserializeError> where Self: Sized;
    fn schema() -> ComponentSchema;
}

// Component registry for mod components
pub struct ModComponentRegistry {
    components: HashMap<ModComponentId, ComponentMetadata>,
    validators: HashMap<ModComponentId, Box<dyn ComponentValidator>>,
}

impl ModComponentRegistry {
    pub fn register<C: ModComponent>(&mut self) -> Result<(), RegistrationError> {
        let id = C::type_id();
        let schema = C::schema();
        
        // Validate schema
        self.validate_schema(&schema)?;
        
        // Check for conflicts
        if self.components.contains_key(&id) {
            return Err(RegistrationError::DuplicateComponent(id));
        }
        
        self.components.insert(id, ComponentMetadata {
            schema,
            mod_id: current_mod_id(),
            size: std::mem::size_of::<C>(),
        });
        
        Ok(())
    }
}
```

### Behavior Modifications

```rust
pub struct BehaviorMod {
    pub target: BehaviorTarget,
    pub modification: BehaviorModification,
    pub priority: ModPriority,
    pub conditions: Vec<BehaviorCondition>,
}

pub enum BehaviorTarget {
    AllCreatures,
    CreatureType(CreatureType),
    CreaturesWithTrait(TraitType),
    SpecificCreature(Entity),
}

pub enum BehaviorModification {
    // Add new behaviors
    AddBehavior {
        behavior: ModBehavior,
        weight: f32,
    },
    
    // Modify existing behaviors
    ModifyWeight {
        behavior_type: BehaviorType,
        multiplier: f32,
    },
    
    // Override decisions
    OverrideDecision {
        condition: DecisionCondition,
        action: ModAction,
    },
    
    // Add new goals
    AddGoal {
        goal: ModGoal,
        priority: f32,
    },
}

pub trait ModBehavior: Send + Sync {
    fn evaluate(&self, creature: &Creature, world: &World) -> BehaviorScore;
    fn execute(&self, creature: Entity, world: &mut World) -> Result<(), BehaviorError>;
    fn can_interrupt(&self) -> bool;
}
```

### Visual Modifications

```rust
pub struct VisualMod {
    pub mod_type: VisualModType,
    pub assets: ModAssets,
    pub application_rules: Vec<ApplicationRule>,
}

pub enum VisualModType {
    // Texture replacements
    TextureOverride {
        target: TextureTarget,
        texture: Handle<Image>,
    },
    
    // New creature variants
    CreatureVariant {
        base_type: CreatureType,
        variant_id: VariantId,
        sprite_sheet: Handle<SpriteSheet>,
        animations: HashMap<AnimationId, AnimationData>,
    },
    
    // Particle effects
    ParticleEffect {
        trigger: ParticleTrigger,
        emitter: ParticleEmitterConfig,
    },
    
    // UI themes
    UITheme {
        theme_id: ThemeId,
        colors: ColorPalette,
        fonts: FontSet,
        layouts: LayoutOverrides,
    },
}

pub struct ModAssets {
    pub textures: HashMap<AssetId, TextureAsset>,
    pub sounds: HashMap<AssetId, SoundAsset>,
    pub data_files: HashMap<AssetId, DataAsset>,
}

impl ModAssets {
    pub fn validate(&self) -> Result<(), AssetError> {
        let total_size = self.calculate_total_size();
        
        if total_size > MAX_MOD_ASSET_SIZE {
            return Err(AssetError::TooLarge(total_size));
        }
        
        // Validate individual assets
        for (_, texture) in &self.textures {
            texture.validate()?;
        }
        
        Ok(())
    }
}
```

### Data Modifications

```rust
pub struct DataMod {
    pub data_type: DataModType,
    pub modifications: Vec<DataModification>,
    pub validation_rules: Vec<ValidationRule>,
}

pub enum DataModType {
    // Add new resources
    ResourceAddition {
        resource: ModResource,
        distribution: DistributionPattern,
        regeneration: RegenerationRule,
    },
    
    // Modify creature stats
    StatModification {
        stat_type: StatType,
        modifier: StatModifier,
        stacking: StackingRule,
    },
    
    // Add new traits
    TraitAddition {
        trait_def: ModTrait,
        inheritance: InheritanceRule,
        expression: ExpressionRule,
    },
    
    // Environmental changes
    EnvironmentModification {
        biome_changes: HashMap<BiomeType, BiomeModification>,
        weather_patterns: Vec<WeatherPattern>,
    },
}

pub struct ModResource {
    pub id: ResourceId,
    pub name: String,
    pub properties: ResourceProperties,
    pub interactions: Vec<ResourceInteraction>,
}

impl DataMod {
    pub fn apply(&self, world: &mut World) -> Result<(), DataModError> {
        // Validate all modifications first
        for modification in &self.modifications {
            self.validate_modification(modification, world)?;
        }
        
        // Apply modifications
        for modification in &self.modifications {
            modification.apply(world)?;
        }
        
        Ok(())
    }
}
```

## Mod Loading System

```rust
pub struct ModLoader {
    pub mod_directory: PathBuf,
    pub loaded_mods: HashMap<ModId, LoadedMod>,
    pub load_order: Vec<ModId>,
    pub compatibility_checker: CompatibilityChecker,
}

pub struct LoadedMod {
    pub definition: ModDefinition,
    pub sandbox: ModSandbox,
    pub instance: Box<dyn Mod>,
    pub state: ModState,
}

pub enum ModState {
    Loaded,
    Enabled,
    Disabled,
    Error(ModError),
}

impl ModLoader {
    pub fn load_mod(&mut self, path: &Path) -> Result<ModId, LoadError> {
        // Read mod manifest
        let manifest = self.read_manifest(path)?;
        
        // Check dependencies
        self.check_dependencies(&manifest)?;
        
        // Create sandbox
        let sandbox = ModSandbox::new(&manifest.capabilities);
        
        // Load mod code (future: use WASM)
        let module = self.load_module(path)?;
        
        // Create mod instance
        let instance = module.instantiate(sandbox)?;
        
        // Register mod
        let mod_id = manifest.metadata.id.clone();
        self.loaded_mods.insert(mod_id.clone(), LoadedMod {
            definition: manifest,
            sandbox,
            instance,
            state: ModState::Loaded,
        });
        
        Ok(mod_id)
    }
    
    pub fn enable_mod(&mut self, mod_id: &ModId) -> Result<(), ModError> {
        let mod_ref = self.loaded_mods.get_mut(mod_id)
            .ok_or(ModError::NotFound)?;
            
        // Check compatibility with enabled mods
        self.compatibility_checker.check(mod_id, &self.get_enabled_mods())?;
        
        // Enable mod
        mod_ref.instance.on_enable()?;
        mod_ref.state = ModState::Enabled;
        
        // Update load order
        self.update_load_order();
        
        Ok(())
    }
}
```

## Mod Communication

```rust
pub struct ModCommunication {
    pub channels: HashMap<ChannelId, ModChannel>,
    pub message_queue: VecDeque<ModMessage>,
}

pub struct ModChannel {
    pub id: ChannelId,
    pub subscribers: Vec<ModId>,
    pub message_type: MessageType,
    pub rate_limit: Option<RateLimit>,
}

pub struct ModMessage {
    pub sender: ModId,
    pub channel: ChannelId,
    pub content: MessageContent,
    pub timestamp: Instant,
}

pub enum MessageContent {
    Data(serde_json::Value),
    Event(ModEvent),
    Request { id: RequestId, data: serde_json::Value },
    Response { id: RequestId, result: Result<serde_json::Value, String> },
}

impl ModCommunication {
    pub fn send_message(&mut self, message: ModMessage) -> Result<(), CommError> {
        // Check rate limits
        if let Some(channel) = self.channels.get(&message.channel) {
            if let Some(limit) = &channel.rate_limit {
                limit.check(&message.sender)?;
            }
        }
        
        // Queue message
        self.message_queue.push_back(message);
        
        Ok(())
    }
    
    pub fn create_channel(&mut self, id: ChannelId, config: ChannelConfig) -> Result<(), CommError> {
        if self.channels.contains_key(&id) {
            return Err(CommError::ChannelExists);
        }
        
        self.channels.insert(id, ModChannel {
            id,
            subscribers: vec![],
            message_type: config.message_type,
            rate_limit: config.rate_limit,
        });
        
        Ok(())
    }
}
```

## Mod Configuration

```rust
pub struct ModConfig {
    pub user_settings: HashMap<String, ConfigValue>,
    pub schema: ConfigSchema,
    pub defaults: HashMap<String, ConfigValue>,
}

pub enum ConfigValue {
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Array(Vec<ConfigValue>),
    Object(HashMap<String, ConfigValue>),
}

pub struct ConfigSchema {
    pub fields: Vec<ConfigField>,
    pub validation_rules: Vec<ValidationRule>,
}

pub struct ConfigField {
    pub name: String,
    pub field_type: ConfigFieldType,
    pub description: String,
    pub default: Option<ConfigValue>,
    pub constraints: Vec<FieldConstraint>,
}

pub enum ConfigFieldType {
    Boolean,
    Integer { min: Option<i64>, max: Option<i64> },
    Float { min: Option<f64>, max: Option<f64> },
    String { max_length: Option<usize> },
    Enum { options: Vec<String> },
    Array { item_type: Box<ConfigFieldType> },
}
```

## Mod Distribution

```rust
pub struct ModPackage {
    pub manifest: ModManifest,
    pub code: ModCode,
    pub assets: ModAssets,
    pub documentation: Documentation,
    pub signature: Option<Signature>,
}

pub struct ModManifest {
    pub format_version: Version,
    pub mod_info: ModMetadata,
    pub requirements: Requirements,
    pub contents: Contents,
}

pub struct Requirements {
    pub game_version: VersionReq,
    pub dependencies: Vec<ModDependency>,
    pub platform: Option<Platform>,
}

pub struct ModRepository {
    pub url: Url,
    pub mods: Vec<ModListing>,
    pub categories: Vec<Category>,
    pub search_index: SearchIndex,
}

pub struct ModListing {
    pub metadata: ModMetadata,
    pub downloads: u64,
    pub rating: f32,
    pub screenshots: Vec<Url>,
    pub download_url: Url,
    pub file_size: u64,
}
```

## Example Mod

```rust
// Example: Seasonal Migration Mod
pub struct SeasonalMigrationMod;

impl Mod for SeasonalMigrationMod {
    fn metadata(&self) -> ModMetadata {
        ModMetadata {
            id: ModId::from("seasonal_migration"),
            name: "Seasonal Migration".into(),
            version: Version::new(1, 0, 0),
            author: "Example Author".into(),
            description: "Adds seasonal migration patterns".into(),
            license: License::MIT,
            tags: vec![ModTag::Behavior, ModTag::Seasons],
        }
    }
    
    fn on_enable(&mut self, ctx: &mut ModContext) -> Result<(), ModError> {
        // Register new behavior
        ctx.register_behavior(MigrationBehavior::new())?;
        
        // Subscribe to season change events
        ctx.subscribe_event::<SeasonChangeEvent>(|event| {
            self.on_season_change(event);
        })?;
        
        Ok(())
    }
}

struct MigrationBehavior {
    current_season: Season,
    migration_targets: HashMap<Season, BiomeType>,
}

impl ModBehavior for MigrationBehavior {
    fn evaluate(&self, creature: &Creature, world: &World) -> BehaviorScore {
        // Check if creature should migrate
        if self.should_migrate(creature, world) {
            BehaviorScore::new(0.8) // High priority
        } else {
            BehaviorScore::new(0.0)
        }
    }
    
    fn execute(&self, creature: Entity, world: &mut World) -> Result<(), BehaviorError> {
        // Set migration target
        let target_biome = self.migration_targets[&self.current_season];
        
        // Find nearest biome of target type
        let target = self.find_nearest_biome(creature, target_biome, world)?;
        
        // Set creature goal
        world.get_mut::<CreatureGoals>(creature)
            .map(|mut goals| {
                goals.set_migration_target(target);
            });
            
        Ok(())
    }
}
```

This mod system API provides a robust foundation for extending the simulation while maintaining safety, performance, and compatibility.