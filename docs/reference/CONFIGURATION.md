# Configuration Constants

Central reference for all configuration values and constants used throughout the system.

## Performance Targets

```rust
pub const TARGET_FPS: u32 = 60;
pub const MAX_CREATURES: usize = 5000;
pub const MAX_ACTIVE_CREATURES: usize = 1000;  // Full detail
pub const MEMORY_BUDGET_MB: usize = 2048;     // 2GB total
```

## Time Constants

```rust
pub const SECONDS_PER_DAY: f32 = 120.0;       // 2 minutes real-time
pub const DAYS_PER_SEASON: u32 = 30;
pub const SEASONS_PER_YEAR: u32 = 4;
pub const TICKS_PER_SECOND: u32 = 60;

// Time scale presets
pub const TIME_SCALES: [f32; 6] = [
    0.0,    // Paused
    1.0,    // Normal (1x)
    5.0,    // Fast (5x)
    25.0,   // Very Fast (25x)
    100.0,  // Ultra Fast (100x)
    1000.0, // Max Speed (1000x)
];
```

## System Update Frequencies

```rust
pub const UPDATE_RATES: SystemUpdateRates = SystemUpdateRates {
    creature_ai: 10.0,          // 10 Hz
    physics: 60.0,              // 60 Hz
    rendering: 60.0,            // 60 Hz (vsync)
    ui: 30.0,                   // 30 Hz
    spatial_index: 10.0,        // 10 Hz
    statistics: 1.0,            // 1 Hz
    save_system: 0.1,           // Every 10 seconds
};
```

## LOD Distance Thresholds

```rust
pub const LOD_DISTANCES: [f32; 5] = [
    50.0,   // LOD 0: Full detail
    100.0,  // LOD 1: Reduced animations
    200.0,  // LOD 2: Simplified AI
    500.0,  // LOD 3: Basic updates only
    1000.0, // LOD 4: Statistical only
];
```

## Memory Pool Sizes

```rust
pub const POOL_SIZES: PoolConfiguration = PoolConfiguration {
    creatures: 5000,
    particles: 10000,
    ui_elements: 1000,
    sound_instances: 128,
    pathfinding_nodes: 50000,
};
```

## World Generation

```rust
pub const WORLD_SIZE: (u32, u32) = (512, 512);
pub const CHUNK_SIZE: u32 = 32;
pub const BIOME_TRANSITION_WIDTH: f32 = 16.0;
pub const MIN_BIOME_SIZE: f32 = 64.0;
```

## UI Configuration

```rust
pub const UI_CONFIG: UiConfiguration = UiConfiguration {
    panel_alpha: 0.95,
    animation_duration: 0.2,
    tooltip_delay: 0.5,
    double_click_time: 0.3,
    drag_threshold: 5.0,       // pixels
    zoom_speed: 0.1,
    pan_speed: 500.0,          // pixels/second
};
```

## Creature Defaults

```rust
pub const CREATURE_DEFAULTS: CreatureDefaults = CreatureDefaults {
    max_health: 100.0,
    max_energy: 100.0,
    max_hunger: 100.0,
    max_thirst: 100.0,
    base_speed: 50.0,          // units/second
    vision_range: 100.0,       // units
    interaction_range: 10.0,   // units
};
```

## Save System

```rust
pub const SAVE_CONFIG: SaveConfiguration = SaveConfiguration {
    autosave_interval: 300.0,  // 5 minutes
    max_autosaves: 3,
    compression_level: 6,      // zstd level
    chunk_size: 1048576,       // 1MB chunks
};
```

## Debug Settings

```rust
pub const DEBUG_DEFAULTS: DebugSettings = DebugSettings {
    show_fps: true,
    show_creature_info: false,
    show_collision_boxes: false,
    show_pathfinding: false,
    log_level: LogLevel::Info,
};
```

## Usage

All systems should reference these constants rather than defining their own:

```rust
use crate::config::{TARGET_FPS, MAX_CREATURES, TIME_SCALES};

impl TimeController {
    fn new() -> Self {
        Self {
            target_fps: TARGET_FPS,
            time_scales: TIME_SCALES.to_vec(),
            // ...
        }
    }
}
```

This ensures consistency across the codebase and makes it easy to tune performance by adjusting values in one place.