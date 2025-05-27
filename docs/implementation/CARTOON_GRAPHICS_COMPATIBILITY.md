# Cartoon Graphics Compatibility Specification

This document details version compatibility, platform requirements, and network considerations for the cartoon graphics implementation.

## Bevy Version Compatibility

### Minimum Required Version
- **Bevy**: 0.14.0 or higher
- **Reason**: Requires improved asset system and WGSL shader support

### Tested Versions
| Bevy Version | Status | Notes |
|--------------|--------|-------|
| 0.14.0 | ✅ Supported | Minimum version |
| 0.14.1 | ✅ Supported | Recommended |
| 0.15.0 | ✅ Supported | Latest features |
| 0.13.x | ❌ Not Supported | Missing required shader features |

### Feature Requirements
```toml
[dependencies]
bevy = { version = "0.14", features = [
    "bevy_asset",          # Asset loading system
    "bevy_render",         # Rendering pipeline
    "bevy_sprite",         # 2D sprite rendering
    "bevy_text",           # Text rendering
    "bevy_ui",             # UI system
    "bevy_winit",          # Window management
    "png",                 # PNG loading
    "vorbis",              # OGG audio support
    "bevy_egui",           # egui integration
    "serialize",           # Save/load support
] }

# Graphics dependencies
wgpu = "0.19"              # WebGPU renderer
image = "0.24"             # Image processing
glyph_brush = "0.7"        # Font rendering

# Optional performance features
bevy_prototype_lyon = "0.11"  # Vector graphics (optional)
bevy_particle_systems = "0.11" # GPU particles (optional)
```

## Platform Compatibility

### Desktop Platforms

#### Windows
- **Minimum**: Windows 10 (64-bit)
- **Graphics API**: DirectX 12, Vulkan, or DirectX 11
- **GPU**: DirectX 11 compatible
- **Special Considerations**:
  - High DPI support enabled by default
  - Window transparency for UI effects

#### macOS
- **Minimum**: macOS 10.15 (Catalina)
- **Graphics API**: Metal
- **GPU**: Metal-compatible GPU
- **Special Considerations**:
  - Retina display support
  - Notarization required for distribution

#### Linux
- **Minimum**: Ubuntu 20.04 or equivalent
- **Graphics API**: Vulkan or OpenGL 4.5
- **GPU**: Vulkan 1.2 compatible
- **Special Considerations**:
  - Wayland and X11 support
  - Various distro testing required

### Web Platform (Future)

#### WebAssembly
- **Target**: wasm32-unknown-unknown
- **Graphics API**: WebGPU or WebGL2
- **Limitations**:
  - Reduced texture sizes (1024x1024 max)
  - Simplified shaders for WebGL2
  - No compute shaders in WebGL2

#### Browser Requirements
| Browser | Minimum Version | WebGPU | WebGL2 |
|---------|----------------|--------|--------|
| Chrome | 113+ | ✅ | ✅ |
| Firefox | 114+ | 🚧 | ✅ |
| Safari | 17+ | 🚧 | ✅ |
| Edge | 113+ | ✅ | ✅ |

### Mobile Platforms (Future)

#### iOS
- **Minimum**: iOS 14.0
- **Graphics API**: Metal
- **Limitations**:
  - Reduced particle count (500 max)
  - Simplified LOD system
  - Touch-optimized UI required

#### Android
- **Minimum**: Android 8.0 (API 26)
- **Graphics API**: Vulkan or OpenGL ES 3.2
- **Limitations**:
  - Variable performance profiles
  - Memory constraints on low-end devices

## Graphics API Requirements

### Minimum GPU Features
```rust
// Required features check
let required_features = Features {
    // Texture features
    TEXTURE_COMPRESSION_BC: true,      // Desktop compression
    TEXTURE_COMPRESSION_ETC2: false,   // Mobile (optional)
    
    // Rendering features
    TEXTURE_BINDING_ARRAY: true,       // For texture atlases
    SAMPLED_TEXTURE_ARRAY: true,       // For terrain layers
    MULTI_DRAW_INDIRECT: true,         // For instanced rendering
    
    // Shader features
    SHADER_FLOAT16: false,             // Optional optimization
    SHADER_INT16: false,               // Optional optimization
};
```

### Shader Model Requirements
- **Minimum**: Shader Model 5.0 (DirectX 11)
- **WGSL Features Used**:
  - Texture arrays
  - Instanced rendering
  - Basic compute (optional)
  - Standard derivatives

## Network Requirements (Future Multiplayer)

### Bandwidth Specifications
| Update Type | Frequency | Size | Bandwidth |
|------------|-----------|------|-----------|
| Creature Position | 10 Hz | 12 bytes | 120 bytes/s |
| Creature State | 2 Hz | 32 bytes | 64 bytes/s |
| Resource Changes | On change | 8 bytes | ~20 bytes/s |
| Chat Messages | On send | 128 bytes | Variable |
| **Per Creature** | - | - | **~204 bytes/s** |

### Network Optimization
- **500 Creatures**: ~100 KB/s upload, ~100 KB/s download
- **Delta Compression**: Reduce by 60-70%
- **Actual Bandwidth**: ~30-40 KB/s per client
- **Latency Tolerance**: 200ms for smooth play

### Serialization Format
```rust
// Network message format
#[derive(Serialize, Deserialize)]
struct CreatureUpdate {
    id: u32,                    // 4 bytes
    position: (f32, f32),       // 8 bytes
    animation_state: u8,        // 1 byte
    direction: u8,              // 1 byte
    flags: u16,                 // 2 bytes (various states)
}

// Compressed format: 16 bytes per creature update
```

## Performance Profiles

### Quality Settings by Platform

| Platform | Default Quality | Max Creatures | Particle Limit |
|----------|----------------|---------------|----------------|
| Desktop High-end | Ultra | 1000 | 2000 |
| Desktop Mid-range | High | 500 | 1000 |
| Desktop Low-end | Medium | 250 | 500 |
| Web (WebGPU) | Medium | 250 | 500 |
| Web (WebGL2) | Low | 150 | 250 |
| Mobile High-end | Medium | 200 | 400 |
| Mobile Low-end | Low | 100 | 200 |

### Adaptive Performance
```rust
// Platform detection and adaptation
fn detect_platform_capabilities() -> PerformanceProfile {
    match (
        cfg!(target_arch = "wasm32"),
        cfg!(target_os = "ios"),
        cfg!(target_os = "android"),
    ) {
        (true, _, _) => PerformanceProfile::Web,
        (_, true, _) => PerformanceProfile::MobileIOS,
        (_, _, true) => PerformanceProfile::MobileAndroid,
        _ => detect_desktop_capabilities(),
    }
}
```

## Build Configuration

### Platform-Specific Features
```toml
[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen = "0.2"
web-sys = "0.3"
console_error_panic_hook = "0.1"

[target.'cfg(target_os = "ios")'.dependencies]
bevy = { version = "0.14", features = ["ios"] }

[target.'cfg(target_os = "android")'.dependencies]
bevy = { version = "0.14", features = ["android"] }

[target.'cfg(windows)'.dependencies]
bevy = { version = "0.14", features = ["windows"] }
```

### Conditional Compilation
```rust
// Platform-specific code
#[cfg(target_arch = "wasm32")]
fn load_assets() -> AssetLoaderConfig {
    AssetLoaderConfig {
        max_texture_size: 1024,
        use_compressed_textures: false,
        particle_limit: 500,
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn load_assets() -> AssetLoaderConfig {
    AssetLoaderConfig {
        max_texture_size: 2048,
        use_compressed_textures: true,
        particle_limit: 1000,
    }
}
```

## Migration Compatibility

### From Current Rendering System
1. **Coordinate System**: Automatic conversion provided
2. **Asset Loading**: Backward compatible paths
3. **Save Files**: Automatic migration on load
4. **Configuration**: Legacy settings preserved

### Breaking Changes
- Minimum Bevy version increased to 0.14
- New shader pipeline (automatic fallback provided)
- UI system migration (compatibility layer included)

### Deprecation Schedule
| Feature | Deprecated | Removed | Migration Path |
|---------|------------|---------|----------------|
| Old sprite system | v2.0 | v3.0 | Use cartoon sprites |
| Direct coordinate access | v2.0 | v2.5 | Use isometric helpers |
| Legacy UI | v2.0 | v3.0 | Use egui panels |