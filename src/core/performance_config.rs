//! Performance configuration for Phase 1 targets

use bevy::prelude::*;

/// Performance configuration for the simulation
#[derive(Resource, Debug, Clone)]
pub struct PerformanceConfig {
    /// Target FPS (Phase 1: 60 FPS)
    pub target_fps: f32,
    /// Maximum number of active creatures (Phase 1: 500)
    pub max_creatures: usize,
    /// Update frequency divisors for different systems
    pub update_frequencies: UpdateFrequencies,
    /// LOD (Level of Detail) settings
    pub lod_settings: LodSettings,
    /// Spatial grid optimization settings
    pub spatial_settings: SpatialSettings,
}

#[derive(Debug, Clone)]
pub struct UpdateFrequencies {
    /// How often to update creature movement (1 = every frame, 2 = every other frame, etc.)
    pub movement_divisor: u32,
    /// How often to update creature decisions
    pub decision_divisor: u32,
    /// How often to update needs
    pub needs_divisor: u32,
    /// How often to update rendering
    pub render_divisor: u32,
}

#[derive(Debug, Clone)]
pub struct LodSettings {
    /// Distance beyond which creatures are updated less frequently
    pub lod_distance: f32,
    /// Update divisor for distant creatures
    pub distant_update_divisor: u32,
    /// Don't render creatures beyond this distance
    pub cull_distance: f32,
}

#[derive(Debug, Clone)]
pub struct SpatialSettings {
    /// Cell size for spatial grid (smaller = more precise but more memory)
    pub cell_size: f32,
    /// Maximum query radius
    pub max_query_radius: f32,
    /// Pre-allocate capacity for queries
    pub query_capacity: usize,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            target_fps: 60.0,
            max_creatures: 500,
            update_frequencies: UpdateFrequencies {
                movement_divisor: 1,
                decision_divisor: 2, // Decisions every 2 frames
                needs_divisor: 3,    // Needs every 3 frames
                render_divisor: 1,
            },
            lod_settings: LodSettings {
                lod_distance: 200.0,
                distant_update_divisor: 4,
                cull_distance: 500.0,
            },
            spatial_settings: SpatialSettings {
                cell_size: 50.0,
                max_query_radius: 100.0,
                query_capacity: 20,
            },
        }
    }
}

/// Plugin to manage performance configuration
pub struct PerformanceConfigPlugin;

impl Plugin for PerformanceConfigPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PerformanceConfig>()
            .init_resource::<FrameCounter>()
            .add_systems(Update, update_frame_counter);
    }
}

/// Tracks frame count for update frequency management
#[derive(Resource, Default)]
pub struct FrameCounter {
    pub frame: u32,
}

fn update_frame_counter(mut counter: ResMut<FrameCounter>) {
    counter.frame = counter.frame.wrapping_add(1);
}

/// Helper to check if a system should run this frame
pub fn should_run_system(frame_counter: &FrameCounter, divisor: u32) -> bool {
    divisor == 1 || frame_counter.frame % divisor == 0
}