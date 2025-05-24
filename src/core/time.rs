#[derive(Debug, Clone)]
pub struct GameTime {
    pub total_seconds: f64,
    pub delta_seconds: f32,
    pub frame_count: u64,
}

impl GameTime {
    pub fn new() -> Self {
        Self {
            total_seconds: 0.0,
            delta_seconds: 0.0,
            frame_count: 0,
        }
    }
}

impl Default for GameTime {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TimeSystem {
    accumulated_time: f64,
    game_time: f64,
    time_scale: f32,
    paused: bool,
    fixed_timestep: Option<f32>,
    max_delta: f32,
}

impl TimeSystem {
    pub fn new() -> Self {
        Self {
            accumulated_time: 0.0,
            game_time: 0.0,
            time_scale: 1.0,
            paused: false,
            fixed_timestep: Some(1.0 / 60.0), // 60 FPS fixed timestep
            max_delta: 0.1, // Max 100ms per frame
        }
    }
    
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
                Some(timestep * self.time_scale)
            } else {
                None
            }
        } else {
            Some(self.update(real_dt))
        }
    }
    
    pub fn set_time_scale(&mut self, scale: f32) {
        self.time_scale = scale.clamp(0.0, 10.0); // Phase 1: Max 10x
    }
    
    pub fn pause(&mut self) {
        self.paused = true;
    }
    
    pub fn resume(&mut self) {
        self.paused = false;
    }
    
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }
    
    pub fn is_paused(&self) -> bool {
        self.paused
    }
    
    pub fn time_scale(&self) -> f32 {
        self.time_scale
    }
    
    pub fn game_time(&self) -> f64 {
        self.game_time
    }
    
    pub fn set_fixed_timestep(&mut self, timestep: Option<f32>) {
        self.fixed_timestep = timestep;
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
}