# Asset Pipeline Design

## Overview

A comprehensive asset pipeline that handles loading, processing, caching, and hot-reloading of all game assets including textures, models, audio, animations, and data files. The system is designed for efficient streaming, minimal load times, and development iteration speed.

## Core Pipeline Architecture

```rust
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct AssetPipeline {
    pub loader: AssetLoader,
    pub processor: AssetProcessor,
    pub cache: AssetCache,
    pub registry: AssetRegistry,
    pub hot_reload: HotReloadSystem,
    pub streaming: StreamingSystem,
}

pub struct AssetRegistry {
    assets: HashMap<AssetId, AssetMetadata>,
    path_mapping: HashMap<PathBuf, AssetId>,
    type_indices: HashMap<AssetType, Vec<AssetId>>,
    dependencies: HashMap<AssetId, Vec<AssetId>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AssetId(pub u64);

pub struct AssetMetadata {
    pub id: AssetId,
    pub path: PathBuf,
    pub asset_type: AssetType,
    pub version: u32,
    pub size: usize,
    pub load_priority: LoadPriority,
    pub compression: CompressionType,
    pub dependencies: Vec<AssetId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetType {
    Texture,
    Model,
    Animation,
    Audio,
    Shader,
    Material,
    ParticleSystem,
    Font,
    UI,
    Data,
    Script,
}

pub enum LoadPriority {
    Critical,    // Required for game to start
    High,        // Needed soon
    Normal,      // Standard priority
    Low,         // Can be deferred
    OnDemand,    // Load only when needed
}
```

### Asset Loading System

```rust
pub struct AssetLoader {
    io_thread_pool: ThreadPool,
    decoders: HashMap<AssetType, Box<dyn AssetDecoder>>,
    load_queue: Arc<RwLock<LoadQueue>>,
    active_loads: Arc<RwLock<HashMap<AssetId, LoadHandle>>>,
}

pub struct LoadQueue {
    critical: VecDeque<LoadRequest>,
    high: VecDeque<LoadRequest>,
    normal: VecDeque<LoadRequest>,
    low: VecDeque<LoadRequest>,
}

pub struct LoadRequest {
    pub asset_id: AssetId,
    pub path: PathBuf,
    pub asset_type: AssetType,
    pub completion: oneshot::Sender<Result<RawAsset, LoadError>>,
}

pub trait AssetDecoder: Send + Sync {
    fn decode(&self, data: &[u8]) -> Result<RawAsset, DecodeError>;
    fn supported_extensions(&self) -> &[&str];
}

// Texture decoder example
pub struct TextureDecoder {
    supported_formats: Vec<TextureFormat>,
}

impl AssetDecoder for TextureDecoder {
    fn decode(&self, data: &[u8]) -> Result<RawAsset, DecodeError> {
        // Detect format
        let format = detect_image_format(data)?;
        
        let decoded = match format {
            ImageFormat::Png => {
                let decoder = png::Decoder::new(data);
                let mut reader = decoder.read_info()?;
                let mut buf = vec![0; reader.output_buffer_size()];
                reader.next_frame(&mut buf)?;
                
                RawTexture {
                    width: reader.info().width,
                    height: reader.info().height,
                    format: TextureFormat::from_png_color(reader.info().color_type),
                    data: buf,
                    mipmaps: None,
                }
            }
            ImageFormat::Jpg => decode_jpeg(data)?,
            ImageFormat::Dds => decode_dds(data)?, // Pre-compressed
            ImageFormat::Ktx2 => decode_ktx2(data)?, // GPU compressed
            _ => return Err(DecodeError::UnsupportedFormat),
        };
        
        Ok(RawAsset::Texture(decoded))
    }
    
    fn supported_extensions(&self) -> &[&str] {
        &["png", "jpg", "jpeg", "dds", "ktx2"]
    }
}

impl AssetLoader {
    pub async fn load_asset(&self, request: LoadRequest) -> Result<RawAsset, LoadError> {
        // Check if already loading
        if let Some(handle) = self.active_loads.read().await.get(&request.asset_id) {
            return handle.wait().await;
        }
        
        // Add to queue based on priority
        let priority = self.registry.get_metadata(&request.asset_id)?.load_priority;
        self.load_queue.write().await.push(priority, request);
        
        // Process queue
        self.process_load_queue().await
    }
    
    async fn process_load_queue(&self) -> Result<RawAsset, LoadError> {
        let mut queue = self.load_queue.write().await;
        
        // Get next request by priority
        let request = queue.critical.pop_front()
            .or_else(|| queue.high.pop_front())
            .or_else(|| queue.normal.pop_front())
            .or_else(|| queue.low.pop_front())
            .ok_or(LoadError::NoRequestsQueued)?;
        
        // Spawn IO task
        let decoder = self.decoders.get(&request.asset_type)
            .ok_or(LoadError::NoDecoderForType)?;
        
        let handle = self.io_thread_pool.spawn(async move {
            // Read file
            let data = tokio::fs::read(&request.path).await?;
            
            // Decode
            decoder.decode(&data)
        });
        
        // Store handle
        self.active_loads.write().await.insert(request.asset_id, handle);
        
        handle.await
    }
}
```

### Asset Processing Pipeline

```rust
pub struct AssetProcessor {
    processors: HashMap<AssetType, Box<dyn AssetProcessorTrait>>,
    optimization_settings: OptimizationSettings,
    platform_settings: PlatformSettings,
}

pub trait AssetProcessorTrait: Send + Sync {
    fn process(
        &self,
        raw: RawAsset,
        settings: &ProcessingSettings,
    ) -> Result<ProcessedAsset, ProcessingError>;
}

pub struct TextureProcessor {
    compressor: TextureCompressor,
    mipmap_generator: MipmapGenerator,
    format_converter: FormatConverter,
}

impl AssetProcessorTrait for TextureProcessor {
    fn process(
        &self,
        raw: RawAsset,
        settings: &ProcessingSettings,
    ) -> Result<ProcessedAsset, ProcessingError> {
        let RawAsset::Texture(raw_texture) = raw else {
            return Err(ProcessingError::WrongAssetType);
        };
        
        let mut texture = raw_texture;
        
        // Generate mipmaps if needed
        if settings.generate_mipmaps && texture.mipmaps.is_none() {
            texture.mipmaps = Some(self.mipmap_generator.generate(&texture)?);
        }
        
        // Convert format for platform
        if settings.target_format != texture.format {
            texture = self.format_converter.convert(texture, settings.target_format)?;
        }
        
        // Compress if beneficial
        if settings.compression_enabled {
            let compressed = self.compressor.compress(&texture, settings.compression_quality)?;
            
            // Only use compressed if smaller
            if compressed.size() < texture.size() {
                texture = compressed;
            }
        }
        
        // Apply optimizations
        if settings.optimize_for_gpu {
            texture = self.optimize_for_gpu_upload(texture)?;
        }
        
        Ok(ProcessedAsset::Texture(texture))
    }
}

// Model processor with LOD generation
pub struct ModelProcessor {
    lod_generator: LODGenerator,
    optimizer: MeshOptimizer,
    tangent_generator: TangentGenerator,
}

impl ModelProcessor {
    fn process_model(&self, raw_model: RawModel, settings: &ProcessingSettings) -> Result<ProcessedModel, ProcessingError> {
        let mut model = raw_model;
        
        // Generate LODs
        if settings.generate_lods {
            let lods = self.lod_generator.generate_lods(&model, &settings.lod_settings)?;
            model.lods = lods;
        }
        
        // Optimize meshes
        for mesh in &mut model.meshes {
            // Generate tangents if missing
            if mesh.tangents.is_none() && settings.generate_tangents {
                mesh.tangents = Some(self.tangent_generator.generate(mesh)?);
            }
            
            // Optimize vertex data
            *mesh = self.optimizer.optimize_mesh(mesh, &settings.optimization_settings)?;
        }
        
        // Pack into GPU-friendly format
        let packed = self.pack_for_gpu(&model)?;
        
        Ok(ProcessedModel {
            meshes: packed.meshes,
            materials: model.materials,
            skeleton: model.skeleton,
            bounding_box: calculate_bounds(&model),
        })
    }
}
```

### Asset Cache System

```rust
pub struct AssetCache {
    memory_cache: Arc<RwLock<MemoryCache>>,
    disk_cache: DiskCache,
    gpu_cache: GpuCache,
    cache_policy: CachePolicy,
}

pub struct MemoryCache {
    assets: HashMap<AssetId, CachedAsset>,
    lru: LruCache<AssetId>,
    total_size: usize,
    max_size: usize,
}

pub struct CachedAsset {
    pub data: Arc<dyn Any + Send + Sync>,
    pub size: usize,
    pub last_access: Instant,
    pub access_count: u32,
    pub pin_count: u32,
}

pub struct DiskCache {
    cache_dir: PathBuf,
    index: DiskCacheIndex,
    compression: CompressionType,
}

pub struct GpuCache {
    texture_cache: GpuTextureCache,
    buffer_cache: GpuBufferCache,
    allocated: usize,
    max_size: usize,
}

impl AssetCache {
    pub async fn get<T: Asset>(&self, id: AssetId) -> Option<Arc<T>> {
        // Check memory cache
        if let Some(cached) = self.memory_cache.read().await.get(&id) {
            if let Ok(asset) = cached.data.downcast::<T>() {
                cached.touch();
                return Some(asset);
            }
        }
        
        // Check disk cache
        if let Some(data) = self.disk_cache.load(id).await? {
            let asset = T::from_bytes(&data)?;
            self.memory_cache.write().await.insert(id, asset.clone());
            return Some(asset);
        }
        
        None
    }
    
    pub async fn cache<T: Asset>(&self, id: AssetId, asset: T) {
        let size = asset.size_bytes();
        
        // Evict if needed
        if self.memory_cache.read().await.would_exceed_limit(size) {
            self.evict_lru().await;
        }
        
        // Add to memory cache
        self.memory_cache.write().await.insert(id, CachedAsset {
            data: Arc::new(asset),
            size,
            last_access: Instant::now(),
            access_count: 0,
            pin_count: 0,
        });
        
        // Write to disk cache if appropriate
        if self.cache_policy.should_disk_cache(&asset) {
            self.disk_cache.store(id, &asset).await;
        }
    }
    
    async fn evict_lru(&self) {
        let mut cache = self.memory_cache.write().await;
        
        while cache.total_size > cache.max_size * 9 / 10 {
            if let Some(id) = cache.lru.pop() {
                if let Some(asset) = cache.assets.get(&id) {
                    if asset.pin_count == 0 {
                        cache.total_size -= asset.size;
                        cache.assets.remove(&id);
                    }
                }
            } else {
                break;
            }
        }
    }
}
```

### Hot Reload System

```rust
pub struct HotReloadSystem {
    watcher: FileWatcher,
    reload_queue: Arc<Mutex<VecDeque<ReloadRequest>>>,
    asset_mapping: Arc<RwLock<HashMap<PathBuf, AssetId>>>,
    reload_handlers: HashMap<AssetType, Box<dyn ReloadHandler>>,
}

pub trait ReloadHandler: Send + Sync {
    fn can_hot_reload(&self) -> bool;
    fn reload(&self, old: &ProcessedAsset, new: RawAsset) -> Result<ProcessedAsset, ReloadError>;
    fn update_references(&self, old_id: AssetId, new_asset: &ProcessedAsset);
}

impl HotReloadSystem {
    pub fn start(self: Arc<Self>) {
        // Start file watcher
        let watcher_handle = self.clone();
        std::thread::spawn(move || {
            watcher_handle.watch_loop();
        });
        
        // Start reload processor
        let processor_handle = self.clone();
        tokio::spawn(async move {
            processor_handle.process_reloads().await;
        });
    }
    
    fn watch_loop(&self) {
        let (tx, rx) = channel();
        let mut watcher = notify::watcher(tx, Duration::from_millis(100)).unwrap();
        
        // Watch asset directories
        watcher.watch("assets/", RecursiveMode::Recursive).unwrap();
        
        loop {
            match rx.recv() {
                Ok(DebouncedEvent::Write(path)) |
                Ok(DebouncedEvent::Create(path)) => {
                    if let Some(asset_id) = self.get_asset_id(&path) {
                        self.queue_reload(asset_id, path);
                    }
                }
                Ok(DebouncedEvent::Remove(path)) => {
                    self.handle_asset_removal(&path);
                }
                _ => {}
            }
        }
    }
    
    async fn process_reloads(&self) {
        loop {
            if let Some(request) = self.reload_queue.lock().await.pop_front() {
                match self.reload_asset(request).await {
                    Ok(()) => log::info!("Hot reloaded asset: {:?}", request.asset_id),
                    Err(e) => log::error!("Failed to hot reload: {:?}", e),
                }
            }
            
            tokio::time::sleep(Duration::from_millis(16)).await;
        }
    }
    
    async fn reload_asset(&self, request: ReloadRequest) -> Result<(), ReloadError> {
        // Load new version
        let raw_asset = self.loader.load_raw(&request.path).await?;
        
        // Get old asset
        let old_asset = self.cache.get(request.asset_id).await
            .ok_or(ReloadError::AssetNotCached)?;
        
        // Process with hot reload handler
        let handler = self.reload_handlers.get(&request.asset_type)
            .ok_or(ReloadError::NoHandlerForType)?;
        
        let new_asset = handler.reload(&old_asset, raw_asset)?;
        
        // Update cache
        self.cache.replace(request.asset_id, new_asset.clone()).await;
        
        // Update references
        handler.update_references(request.asset_id, &new_asset);
        
        Ok(())
    }
}
```

### Asset Streaming

```rust
pub struct StreamingSystem {
    streaming_thread: JoinHandle<()>,
    request_queue: Arc<Mutex<BinaryHeap<StreamRequest>>>,
    active_streams: Arc<RwLock<HashMap<AssetId, StreamHandle>>>,
    bandwidth_limiter: BandwidthLimiter,
}

pub struct StreamRequest {
    pub asset_id: AssetId,
    pub priority: f32,
    pub deadline: Option<Instant>,
    pub quality_levels: Vec<QualityLevel>,
    pub completion: oneshot::Sender<StreamHandle>,
}

pub struct StreamHandle {
    pub asset_id: AssetId,
    pub current_quality: QualityLevel,
    pub loaded_percentage: f32,
    pub data: Arc<RwLock<PartialAsset>>,
}

pub enum PartialAsset {
    Texture {
        base_level: Option<TextureData>,
        mip_levels: Vec<Option<TextureData>>,
        current_mip: u32,
    },
    Audio {
        header: AudioHeader,
        chunks: Vec<Option<AudioChunk>>,
        current_chunk: usize,
    },
    Model {
        base_lod: Option<ModelData>,
        detail_lods: Vec<Option<ModelData>>,
        current_lod: u32,
    },
}

impl StreamingSystem {
    pub async fn stream_asset(&self, request: StreamRequest) -> StreamHandle {
        // Add to priority queue
        self.request_queue.lock().await.push(request);
        
        // Get handle
        let handle = self.create_stream_handle(request.asset_id).await;
        
        // Start streaming
        self.process_stream(handle.clone()).await;
        
        handle
    }
    
    async fn process_stream(&self, handle: StreamHandle) {
        let metadata = self.registry.get_metadata(handle.asset_id).await?;
        
        match metadata.asset_type {
            AssetType::Texture => {
                self.stream_texture(handle, metadata).await?;
            }
            AssetType::Audio => {
                self.stream_audio(handle, metadata).await?;
            }
            AssetType::Model => {
                self.stream_model(handle, metadata).await?;
            }
            _ => {
                // Non-streamable asset, load fully
                self.load_full_asset(handle, metadata).await?;
            }
        }
    }
    
    async fn stream_texture(&self, handle: StreamHandle, metadata: AssetMetadata) {
        let mut partial = PartialAsset::Texture {
            base_level: None,
            mip_levels: vec![None; metadata.mip_count],
            current_mip: metadata.mip_count - 1, // Start with lowest quality
        };
        
        // Stream from lowest to highest quality mipmap
        for mip_level in (0..metadata.mip_count).rev() {
            let mip_data = self.load_texture_mip(&metadata.path, mip_level).await?;
            
            // Apply bandwidth limiting
            self.bandwidth_limiter.consume(mip_data.size()).await;
            
            // Update partial asset
            if let PartialAsset::Texture { ref mut mip_levels, ref mut current_mip, .. } = &mut partial {
                mip_levels[mip_level as usize] = Some(mip_data);
                *current_mip = mip_level;
            }
            
            // Update handle
            handle.data.write().await = partial.clone();
            handle.loaded_percentage = (metadata.mip_count - mip_level) as f32 / metadata.mip_count as f32;
            
            // Check if we should stop (e.g., asset no longer needed)
            if self.should_cancel_stream(&handle) {
                break;
            }
        }
    }
}
```

### Asset Bundling & Packing

```rust
pub struct AssetBundler {
    bundle_settings: BundleSettings,
    packer: AssetPacker,
    compressor: BundleCompressor,
}

pub struct Bundle {
    pub id: BundleId,
    pub assets: Vec<AssetId>,
    pub total_size: usize,
    pub compressed_size: usize,
    pub format: BundleFormat,
}

pub struct BundleSettings {
    pub max_bundle_size: usize,
    pub compression: CompressionSettings,
    pub alignment: usize,
    pub group_by: GroupingStrategy,
}

pub enum GroupingStrategy {
    Type,      // Group similar asset types
    Usage,     // Group by usage patterns
    Level,     // Group by game level
    Priority,  // Group by load priority
}

impl AssetBundler {
    pub fn create_bundles(&self, assets: Vec<AssetMetadata>) -> Vec<Bundle> {
        // Group assets
        let groups = match self.bundle_settings.group_by {
            GroupingStrategy::Type => self.group_by_type(assets),
            GroupingStrategy::Usage => self.group_by_usage(assets),
            GroupingStrategy::Level => self.group_by_level(assets),
            GroupingStrategy::Priority => self.group_by_priority(assets),
        };
        
        // Pack into bundles
        let mut bundles = Vec::new();
        for group in groups {
            let packed_bundles = self.pack_group(group);
            bundles.extend(packed_bundles);
        }
        
        bundles
    }
    
    fn pack_group(&self, assets: Vec<AssetMetadata>) -> Vec<Bundle> {
        let mut bundles = Vec::new();
        let mut current_bundle = Vec::new();
        let mut current_size = 0;
        
        for asset in assets {
            if current_size + asset.size > self.bundle_settings.max_bundle_size {
                // Finalize current bundle
                if !current_bundle.is_empty() {
                    bundles.push(self.finalize_bundle(current_bundle));
                }
                current_bundle = vec![asset];
                current_size = asset.size;
            } else {
                current_bundle.push(asset);
                current_size += asset.size;
            }
        }
        
        // Last bundle
        if !current_bundle.is_empty() {
            bundles.push(self.finalize_bundle(current_bundle));
        }
        
        bundles
    }
    
    fn finalize_bundle(&self, assets: Vec<AssetMetadata>) -> Bundle {
        let bundle_id = BundleId::new();
        
        // Pack assets
        let packed_data = self.packer.pack_assets(&assets);
        
        // Compress
        let compressed = self.compressor.compress(&packed_data, &self.bundle_settings.compression);
        
        Bundle {
            id: bundle_id,
            assets: assets.iter().map(|a| a.id).collect(),
            total_size: packed_data.len(),
            compressed_size: compressed.len(),
            format: BundleFormat::Custom,
        }
    }
}
```

### Platform-Specific Processing

```rust
pub struct PlatformAssetProcessor {
    target_platform: Platform,
    texture_formats: HashMap<Platform, TextureFormat>,
    audio_formats: HashMap<Platform, AudioFormat>,
    optimization_profiles: HashMap<Platform, OptimizationProfile>,
}

#[derive(Debug, Clone, Copy)]
pub enum Platform {
    Windows,
    Mac,
    Linux,
    WebGL,
    Mobile,
}

pub struct OptimizationProfile {
    pub max_texture_size: u32,
    pub texture_compression: TextureCompression,
    pub audio_quality: AudioQuality,
    pub model_complexity: ModelComplexity,
    pub shader_optimization: ShaderOptimization,
}

impl PlatformAssetProcessor {
    pub fn process_for_platform(&self, asset: RawAsset, platform: Platform) -> ProcessedAsset {
        let profile = &self.optimization_profiles[&platform];
        
        match asset {
            RawAsset::Texture(texture) => {
                self.process_texture_for_platform(texture, platform, profile)
            }
            RawAsset::Audio(audio) => {
                self.process_audio_for_platform(audio, platform, profile)
            }
            RawAsset::Model(model) => {
                self.process_model_for_platform(model, platform, profile)
            }
            _ => asset,
        }
    }
    
    fn process_texture_for_platform(
        &self,
        mut texture: RawTexture,
        platform: Platform,
        profile: &OptimizationProfile,
    ) -> ProcessedAsset {
        // Resize if too large
        if texture.width > profile.max_texture_size || texture.height > profile.max_texture_size {
            texture = resize_texture(texture, profile.max_texture_size);
        }
        
        // Convert to platform format
        let target_format = self.texture_formats[&platform];
        texture = convert_texture_format(texture, target_format);
        
        // Apply platform-specific compression
        match platform {
            Platform::Mobile => {
                texture = compress_texture_etc2(texture);
            }
            Platform::Windows | Platform::Linux => {
                texture = compress_texture_bc7(texture);
            }
            Platform::Mac => {
                texture = compress_texture_astc(texture);
            }
            Platform::WebGL => {
                texture = compress_texture_basis(texture);
            }
        }
        
        ProcessedAsset::Texture(texture)
    }
}
```

### Asset Validation & Testing

```rust
pub struct AssetValidator {
    validators: HashMap<AssetType, Box<dyn AssetValidatorTrait>>,
    validation_rules: ValidationRules,
}

pub trait AssetValidatorTrait: Send + Sync {
    fn validate(&self, asset: &ProcessedAsset) -> ValidationResult;
}

pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub stats: AssetStats,
}

pub struct TextureValidator {
    max_size: u32,
    required_formats: Vec<TextureFormat>,
    power_of_two: bool,
}

impl AssetValidatorTrait for TextureValidator {
    fn validate(&self, asset: &ProcessedAsset) -> ValidationResult {
        let ProcessedAsset::Texture(texture) = asset else {
            return ValidationResult::error("Not a texture");
        };
        
        let mut result = ValidationResult::default();
        
        // Check size limits
        if texture.width > self.max_size || texture.height > self.max_size {
            result.errors.push(ValidationError::TextureTooLarge {
                width: texture.width,
                height: texture.height,
                max: self.max_size,
            });
        }
        
        // Check power of two
        if self.power_of_two {
            if !is_power_of_two(texture.width) || !is_power_of_two(texture.height) {
                result.warnings.push(ValidationWarning::NonPowerOfTwo);
            }
        }
        
        // Check format
        if !self.required_formats.contains(&texture.format) {
            result.errors.push(ValidationError::UnsupportedFormat {
                format: texture.format,
                supported: self.required_formats.clone(),
            });
        }
        
        // Collect stats
        result.stats = AssetStats {
            uncompressed_size: texture.calculate_size(),
            compressed_size: texture.compressed_size(),
            load_time_estimate: estimate_load_time(texture),
        };
        
        result.valid = result.errors.is_empty();
        result
    }
}
```

## Asset Pipeline Configuration

```rust
pub struct AssetPipelineConfig {
    // Directories
    pub source_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub output_dir: PathBuf,
    
    // Processing settings
    pub texture_settings: TextureProcessingSettings,
    pub audio_settings: AudioProcessingSettings,
    pub model_settings: ModelProcessingSettings,
    
    // Cache settings
    pub memory_cache_size: usize,
    pub disk_cache_size: usize,
    pub gpu_cache_size: usize,
    
    // Streaming settings
    pub streaming_enabled: bool,
    pub max_concurrent_streams: usize,
    pub bandwidth_limit: usize,
    
    // Development settings
    pub hot_reload_enabled: bool,
    pub validation_enabled: bool,
    pub profiling_enabled: bool,
}

pub struct TextureProcessingSettings {
    pub default_format: TextureFormat,
    pub generate_mipmaps: bool,
    pub max_size: u32,
    pub compression_quality: f32,
    pub srgb_textures: Vec<String>, // Texture name patterns
}
```

## Integration Points

### With Rendering System
- GPU resource management
- Texture streaming
- Shader compilation
- Material updates

### With Audio System
- Audio streaming
- Format conversion
- Spatial audio data

### With Game Systems
- Asset references
- Hot reload notifications
- Memory pressure handling

## Performance Metrics

```rust
pub struct AssetPipelineMetrics {
    pub total_assets_loaded: u64,
    pub cache_hit_rate: f32,
    pub average_load_time: Duration,
    pub streaming_bandwidth: f32,
    pub memory_usage: AssetMemoryUsage,
    pub hot_reloads_per_minute: f32,
}

pub struct AssetMemoryUsage {
    pub textures: usize,
    pub models: usize,
    pub audio: usize,
    pub other: usize,
    pub total: usize,
}