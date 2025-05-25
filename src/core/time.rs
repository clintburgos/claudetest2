//! Time management system for consistent simulation updates.
//! 
//! This module provides both variable and fixed timestep support,
//! time scaling for fast-forward/slow-motion, and pause functionality.
//! The fixed timestep ensures deterministic simulation behavior
//! regardless of frame rate variations.

use crate::config::time::*;

/// Represents the current game time state
/// 
/// Tracks total elapsed time, per-frame delta, and frame count.
/// Uses f64 for total time to avoid precision loss over long simulations.
#[derive(Debug, Clone)]
pub struct GameTime {
    /// Total elapsed game time in seconds (high precision)
    pub total_seconds: f64,
    /// Delta time for current frame in seconds
    pub delta_seconds: f32,
    /// Total number of frames processed
    pub frame_count: u64,
}

impl GameTime {
    /// Creates a new GameTime instance at zero
    pub fn new() -> Self {
        Self {
            total_seconds: 0.0,
            delta_seconds: 0.0,
            frame_count: 0,
        }
    }
    
    /// Updates game time with the given delta
    /// 
    /// # Arguments
    /// * `delta` - Time elapsed since last update in seconds
    pub fn advance(&mut self, delta: f32) {
        self.delta_seconds = delta;
        self.total_seconds += delta as f64;
        self.frame_count += 1;
    }
}

impl Default for GameTime {
    fn default() -> Self {
        Self::new()
    }
}

/// Manages game time, scaling, and fixed timestep updates.
/// 
/// The TimeSystem is responsible for converting real-world time into
/// consistent simulation updates. It implements the "fix your timestep"
/// pattern to ensure deterministic behavior.
/// 
/// # Features
/// - **Fixed timestep**: Simulation runs at consistent intervals
/// - **Accumulator pattern**: Handles varying frame rates gracefully
/// - **Time scaling**: 0.0 to 10.0x speed adjustment
/// - **Pause support**: Complete simulation halt
/// - **Spike protection**: Clamps large deltas to prevent spiral of death
/// - **Interpolation**: Smooth visual updates between fixed steps
/// 
/// # Architecture
/// The system accumulates real time until enough has passed for a fixed
/// timestep, then returns that timestep for simulation update. Any
/// remainder is kept for the next frame, and interpolation factor
/// indicates progress toward the next fixed update.
pub struct TimeSystem {
    accumulated_time: f64,
    game_time: f64,
    time_scale: f32,
    paused: bool,
    fixed_timestep: Option<f32>,
    max_delta: f32,
    /// Interpolation factor for smooth rendering between fixed updates
    interpolation: f32,
}

impl TimeSystem {
    /// Creates a new time system with configurable fixed timestep
    pub fn new() -> Self {
        Self {
            accumulated_time: 0.0,
            game_time: 0.0,
            time_scale: 1.0,
            paused: false,
            fixed_timestep: Some(FIXED_TIMESTEP),
            max_delta: MAX_DELTA,
            interpolation: 0.0,
        }
    }
    
    /// Updates time system with variable timestep
    /// 
    /// # Arguments
    /// * `real_dt` - Real time elapsed since last update
    /// 
    /// # Returns
    /// Scaled game time to use for this frame
    pub fn update(&mut self, real_dt: f32) -> f32 {
        if self.paused {
            return 0.0;
        }
        
        // Clamp delta time to prevent spiral of death
        let clamped_dt = real_dt.min(self.max_delta);
        
        let scaled_dt = clamped_dt * self.time_scale;
        self.accumulated_time += scaled_dt as f64;
        self.game_time += scaled_dt as f64;
        
        scaled_dt
    }
    
    /// Updates time system with fixed timestep
    /// 
    /// Accumulates time and returns a fixed timestep when enough
    /// time has accumulated. Also updates interpolation factor.
    /// 
    /// # Arguments
    /// * `real_dt` - Real time elapsed since last update
    /// 
    /// # Returns
    /// Some(timestep) when a fixed update should occur, None otherwise
    pub fn fixed_update(&mut self, real_dt: f32) -> Option<f32> {
        if self.paused {
            return None;
        }
        
        if let Some(timestep) = self.fixed_timestep {
            let clamped_dt = real_dt.min(self.max_delta);
            self.accumulated_time += clamped_dt as f64 * self.time_scale as f64;
            
            if self.accumulated_time >= timestep as f64 {
                self.accumulated_time -= timestep as f64;
                self.game_time += timestep as f64 * self.time_scale as f64;
                // Update interpolation after consuming timestep
                self.interpolation = (self.accumulated_time / timestep as f64) as f32;
                Some(timestep * self.time_scale)
            } else {
                // Update interpolation when not consuming timestep
                self.interpolation = (self.accumulated_time / timestep as f64) as f32;
                None
            }
        } else {
            Some(self.update(real_dt))
        }
    }
    
    /// Sets the time scale
    /// 
    /// # Arguments
    /// * `scale` - Time scale multiplier (clamped to configured maximum)
    pub fn set_time_scale(&mut self, scale: f32) {
        self.time_scale = scale.clamp(0.0, MAX_TIME_SCALE);
    }
    
    /// Pauses the simulation
    pub fn pause(&mut self) {
        self.paused = true;
    }
    
    /// Resumes the simulation
    pub fn resume(&mut self) {
        self.paused = false;
    }
    
    /// Toggles pause state
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }
    
    /// Returns true if simulation is paused
    pub fn is_paused(&self) -> bool {
        self.paused
    }
    
    /// Returns current time scale
    pub fn time_scale(&self) -> f32 {
        self.time_scale
    }
    
    /// Returns total game time elapsed
    pub fn game_time(&self) -> f64 {
        self.game_time
    }
    
    /// Returns interpolation factor for smooth rendering
    /// 
    /// Value between 0.0 and 1.0 representing progress
    /// towards next fixed timestep
    pub fn interpolation(&self) -> f32 {
        self.interpolation
    }
    
    /// Sets fixed timestep (None for variable timestep)
    pub fn set_fixed_timestep(&mut self, timestep: Option<f32>) {
        self.fixed_timestep = timestep;
        if timestep.is_none() {
            self.interpolation = 0.0;
        }
    }
    
    /// Returns the current fixed timestep if set
    pub fn fixed_timestep(&self) -> Option<f32> {
        self.fixed_timestep
    }
}

impl Default for TimeSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_system_basic() {
        let mut time = TimeSystem::new();
        
        let dt = time.update(0.016); // ~60 FPS
        assert!((dt - 0.016).abs() < 0.001);
        assert!((time.game_time() - 0.016).abs() < 0.001);
    }
    
    #[test]
    fn time_system_pause() {
        let mut time = TimeSystem::new();
        time.pause();
        
        let dt = time.update(0.016);
        assert_eq!(dt, 0.0);
        assert_eq!(time.game_time(), 0.0);
    }
    
    #[test]
    fn time_system_scale() {
        let mut time = TimeSystem::new();
        time.set_time_scale(2.0);
        
        let dt = time.update(0.016);
        assert!((dt - 0.032).abs() < 0.001); // 2x speed
    }
    
    #[test]
    fn time_system_fixed_timestep() {
        let mut time = TimeSystem::new();
        time.set_fixed_timestep(Some(0.02)); // 50 FPS fixed
        
        // Not enough accumulated time
        assert_eq!(time.fixed_update(0.01), None);
        
        // Enough accumulated time
        assert_eq!(time.fixed_update(0.015), Some(0.02));
    }
    
    #[test]
    fn time_system_max_delta() {
        let mut time = TimeSystem::new();
        
        // Large frame spike should be clamped
        let dt = time.update(1.0); // 1 second spike!
        assert_eq!(dt, 0.1); // Clamped to max_delta
    }
    
    #[test]
    fn time_system_resume() {
        let mut time = TimeSystem::new();
        time.pause();
        assert!(time.is_paused());
        
        time.resume();
        assert!(!time.is_paused());
        
        let dt = time.update(0.016);
        assert!((dt - 0.016).abs() < 0.001);
    }
    
    #[test]
    fn time_system_toggle_pause() {
        let mut time = TimeSystem::new();
        assert!(!time.is_paused());
        
        time.toggle_pause();
        assert!(time.is_paused());
        
        time.toggle_pause();
        assert!(!time.is_paused());
    }
    
    #[test]
    fn time_system_getters() {
        let mut time = TimeSystem::new();
        time.set_time_scale(3.0);
        
        assert_eq!(time.time_scale(), 3.0);
        assert!(!time.is_paused());
        
        time.update(0.1);
        assert!((time.game_time() - 0.3).abs() < 0.001); // 0.1 * 3.0
    }
    
    #[test]
    fn time_system_default() {
        let time = TimeSystem::default();
        assert_eq!(time.time_scale(), 1.0);
        assert!(!time.is_paused());
        assert_eq!(time.game_time(), 0.0);
        assert_eq!(time.interpolation(), 0.0);
    }
    
    #[test]
    fn time_system_interpolation() {
        let mut time = TimeSystem::new();
        time.set_fixed_timestep(Some(0.02)); // 50 FPS
        
        // Partial accumulation should update interpolation
        time.fixed_update(0.01);
        assert!((time.interpolation() - 0.5).abs() < 0.01);
        
        // Full timestep consumes one timestep and leaves remainder
        // Previous accumulated: 0.01, new accumulation: 0.015, total: 0.025
        // After consuming 0.02: remainder 0.005, interpolation = 0.005/0.02 = 0.25
        let result = time.fixed_update(0.015);
        assert!(result.is_some()); // Should return timestep
        assert!((time.interpolation() - 0.25).abs() < 0.01);
    }
    
    #[test]
    fn game_time_default() {
        let time = GameTime::default();
        assert_eq!(time.total_seconds, 0.0);
        assert_eq!(time.delta_seconds, 0.0);
        assert_eq!(time.frame_count, 0);
    }
}