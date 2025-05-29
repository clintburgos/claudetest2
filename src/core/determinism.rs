//! Deterministic random number generation for reproducible simulations

use bevy::prelude::*;
use rand::RngCore;
use rand::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;
use std::collections::HashMap;

/// System identifier for RNG isolation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SystemId {
    Movement,
    Decision,
    ResourceSpawn,
    CreatureSpawn,
    Genetics,
    Environment,
}

/// Deterministic RNG manager resource
#[derive(Resource)]
pub struct DeterministicRng {
    master_seed: u64,
    system_rngs: HashMap<SystemId, Xoshiro256PlusPlus>,
    frame_count: u64,
}

impl DeterministicRng {
    /// Create a new deterministic RNG with a master seed
    pub fn new(master_seed: u64) -> Self {
        Self {
            master_seed,
            system_rngs: HashMap::new(),
            frame_count: 0,
        }
    }

    /// Get the master seed
    pub fn seed(&self) -> u64 {
        self.master_seed
    }

    /// Get the current frame count
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Set the frame count (for loading saves)
    pub fn set_frame_count(&mut self, frame: u64) {
        self.frame_count = frame;
    }

    /// Get or create an RNG for a specific system
    pub fn get_rng(&mut self, system: SystemId) -> &mut Xoshiro256PlusPlus {
        let master_seed = self.master_seed;
        let system_seed = master_seed ^ (system as u64);

        self.system_rngs
            .entry(system)
            .or_insert_with(|| Xoshiro256PlusPlus::seed_from_u64(system_seed))
    }

    /// Generate a random f32 in range [0, 1)
    pub fn gen_range_f32(&mut self, system: SystemId) -> f32 {
        let rng = self.get_rng(system);
        // Generate uniform float in [0, 1)
        let value = rng.next_u32();
        (value as f32) / (u32::MAX as f32)
    }

    /// Generate a random f32 in specified range
    pub fn gen_range(&mut self, system: SystemId, min: f32, max: f32) -> f32 {
        let t = self.gen_range_f32(system);
        min + t * (max - min)
    }

    /// Generate a random integer in range [min, max)
    pub fn gen_range_i32(&mut self, system: SystemId, min: i32, max: i32) -> i32 {
        let range = (max - min) as u32;
        let rng = self.get_rng(system);
        let value = rng.next_u32() % range;
        min + value as i32
    }

    /// Generate a random boolean with given probability
    pub fn gen_bool(&mut self, system: SystemId, probability: f32) -> bool {
        self.gen_range_f32(system) < probability
    }

    /// Generate a random unit vector
    pub fn gen_direction(&mut self, system: SystemId) -> Vec2 {
        let angle = self.gen_range(system, 0.0, std::f32::consts::TAU);
        Vec2::new(angle.cos(), angle.sin())
    }

    /// Advance frame counter (for frame-based determinism)
    pub fn advance_frame(&mut self) {
        self.frame_count += 1;
    }

    /// Reset all RNGs to initial state
    pub fn reset(&mut self) {
        self.system_rngs.clear();
        self.frame_count = 0;
    }

    /// Set a new master seed and reset
    pub fn set_seed(&mut self, seed: u64) {
        self.master_seed = seed;
        self.reset();
    }
}

impl Default for DeterministicRng {
    fn default() -> Self {
        // Use current time as default seed for variety
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(42);

        Self::new(seed)
    }
}

/// Configuration for deterministic simulation
#[derive(Resource, Debug, Clone)]
pub struct DeterminismConfig {
    pub enabled: bool,
    pub seed: Option<u64>,
    pub checksum_frequency: u32,
    pub log_checksums: bool,
}

impl Default for DeterminismConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            seed: None,             // Will use current time if None
            checksum_frequency: 60, // Every second at 60 FPS
            log_checksums: false,
        }
    }
}

/// Frame checksum for determinism verification
#[derive(Debug, Clone, PartialEq)]
pub struct FrameChecksum {
    pub frame: u64,
    pub creature_checksum: u64,
    pub resource_checksum: u64,
    pub position_checksum: u64,
}

impl FrameChecksum {
    /// Calculate checksum for current frame
    pub fn calculate(
        frame: u64,
        creature_query: &Query<
            (&crate::components::Position, &crate::components::Health),
            With<crate::components::Creature>,
        >,
        resource_query: &Query<
            &crate::components::Position,
            With<crate::components::ResourceMarker>,
        >,
    ) -> Self {
        let mut creature_checksum = 0u64;
        let mut position_checksum = 0u64;

        // Checksum creatures (sorted by position for determinism)
        let mut creatures: Vec<_> = creature_query.iter().collect();
        creatures.sort_by(|a, b| {
            a.0 .0
                .x
                .partial_cmp(&b.0 .0.x)
                .unwrap()
                .then(a.0 .0.y.partial_cmp(&b.0 .0.y).unwrap())
        });

        for (pos, health) in creatures {
            // Simple checksum combining position and health
            let pos_bits = ((pos.0.x as i32) as u64) << 32 | ((pos.0.y as i32) as u64);
            let health_bits = (health.current * 1000.0) as u64;
            creature_checksum = creature_checksum.wrapping_add(pos_bits ^ health_bits);
            position_checksum = position_checksum.wrapping_add(pos_bits);
        }

        // Checksum resources
        let mut resource_checksum = 0u64;
        let mut resources: Vec<_> = resource_query.iter().collect();
        resources.sort_by(|a, b| {
            a.0.x.partial_cmp(&b.0.x).unwrap().then(a.0.y.partial_cmp(&b.0.y).unwrap())
        });

        for pos in resources {
            let pos_bits = ((pos.0.x as i32) as u64) << 32 | ((pos.0.y as i32) as u64);
            resource_checksum = resource_checksum.wrapping_add(pos_bits);
        }

        FrameChecksum {
            frame,
            creature_checksum,
            resource_checksum,
            position_checksum,
        }
    }
}

/// History of frame checksums for verification
#[derive(Resource, Default)]
pub struct ChecksumHistory {
    checksums: Vec<FrameChecksum>,
    max_history: usize,
}

/// Verifier for determinism in replays and debugging
#[derive(Resource, Default)]
pub struct DeterminismVerifier {
    pub replay_checksums: Option<Vec<FrameChecksum>>,
    pub desync_detected: bool,
    pub desync_frame: Option<u64>,
}

impl ChecksumHistory {
    pub fn new(max_history: usize) -> Self {
        Self {
            checksums: Vec::with_capacity(max_history),
            max_history,
        }
    }

    pub fn add(&mut self, checksum: FrameChecksum) {
        if self.checksums.len() >= self.max_history {
            self.checksums.remove(0);
        }
        self.checksums.push(checksum);
    }

    pub fn get_recent(&self, count: usize) -> &[FrameChecksum] {
        let start = self.checksums.len().saturating_sub(count);
        &self.checksums[start..]
    }
}

/// Plugin for deterministic simulation
pub struct DeterminismPlugin;

impl Plugin for DeterminismPlugin {
    fn build(&self, app: &mut App) {
        // Initialize config if not already present
        if !app.world.contains_resource::<DeterminismConfig>() {
            app.insert_resource(DeterminismConfig::default());
        }

        // Get config
        let config = app.world.get_resource::<DeterminismConfig>().cloned().unwrap_or_default();

        let seed = config.seed.unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(42)
        });

        info!("Initializing deterministic RNG with seed: {}", seed);

        app.insert_resource(DeterministicRng::new(seed))
            .insert_resource(ChecksumHistory::new(300)) // 5 seconds at 60 FPS
            .insert_resource(DeterminismVerifier::default())
            .add_systems(Last, (calculate_frame_checksum, verify_determinism).chain());
    }
}

/// System to calculate frame checksums
fn calculate_frame_checksum(
    config: Res<DeterminismConfig>,
    mut rng: ResMut<DeterministicRng>,
    mut history: ResMut<ChecksumHistory>,
    creature_query: Query<
        (&crate::components::Position, &crate::components::Health),
        With<crate::components::Creature>,
    >,
    resource_query: Query<&crate::components::Position, With<crate::components::ResourceMarker>>,
) {
    if !config.enabled {
        return;
    }

    rng.advance_frame();

    // Calculate checksum at configured frequency
    if rng.frame_count % config.checksum_frequency as u64 == 0 {
        let checksum = FrameChecksum::calculate(rng.frame_count, &creature_query, &resource_query);

        if config.log_checksums {
            debug!(
                "Frame {} checksum: creatures={:x}, resources={:x}, positions={:x}",
                checksum.frame,
                checksum.creature_checksum,
                checksum.resource_checksum,
                checksum.position_checksum
            );
        }

        history.add(checksum);
    }
}

/// System to verify determinism by comparing checksums
fn verify_determinism(
    mut verifier: ResMut<DeterminismVerifier>,
    history: Res<ChecksumHistory>,
    config: Res<DeterminismConfig>,
) {
    if !config.enabled {
        return;
    }

    // Only verify if we have replay checksums to compare against
    if verifier.replay_checksums.is_none() {
        return;
    }

    // Check latest checksum against replay
    if let Some(latest) = history.checksums.last() {
        // Clone to avoid borrow checker issues
        let latest_frame = latest.frame;
        let latest_creature_checksum = latest.creature_checksum;
        let latest_resource_checksum = latest.resource_checksum;
        let latest_position_checksum = latest.position_checksum;
        
        // Find corresponding frame in replay
        if let Some(ref replay_checksums) = verifier.replay_checksums {
            if let Some(replay_checksum) = replay_checksums.iter().find(|c| c.frame == latest_frame) {
                if (latest_creature_checksum != replay_checksum.creature_checksum ||
                   latest_resource_checksum != replay_checksum.resource_checksum ||
                   latest_position_checksum != replay_checksum.position_checksum) && !verifier.desync_detected {
                    error!(
                        "DESYNC DETECTED at frame {}! Current: c={:x} r={:x} p={:x}, Expected: c={:x} r={:x} p={:x}",
                        latest_frame,
                        latest_creature_checksum,
                        latest_resource_checksum,
                        latest_position_checksum,
                        replay_checksum.creature_checksum,
                        replay_checksum.resource_checksum,
                        replay_checksum.position_checksum,
                    );
                    verifier.desync_detected = true;
                    verifier.desync_frame = Some(latest_frame);
                }
            }
        }
    }
}

/// Extension trait for seeded random generation
pub trait SeededRandom {
    fn random_f32(&mut self, system: SystemId) -> f32;
    fn random_range(&mut self, system: SystemId, min: f32, max: f32) -> f32;
    fn random_bool(&mut self, system: SystemId, probability: f32) -> bool;
    fn random_direction(&mut self, system: SystemId) -> Vec2;
}

impl SeededRandom for ResMut<'_, DeterministicRng> {
    fn random_f32(&mut self, system: SystemId) -> f32 {
        self.gen_range_f32(system)
    }

    fn random_range(&mut self, system: SystemId, min: f32, max: f32) -> f32 {
        self.gen_range(system, min, max)
    }

    fn random_bool(&mut self, system: SystemId, probability: f32) -> bool {
        self.gen_bool(system, probability)
    }

    fn random_direction(&mut self, system: SystemId) -> Vec2 {
        self.gen_direction(system)
    }
}

#[cfg(test)]
mod tests;
