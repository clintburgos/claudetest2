//! Simulation control for pause, step, and speed adjustment

use bevy::prelude::*;

/// Simulation control state
#[derive(Resource, Debug)]
pub struct SimulationControl {
    pub paused: bool,
    pub step_mode: bool,
    pub step_requested: bool,
    pub speed_multiplier: f32,
    pub frame_count: u64,
    pub simulation_time: f32,
}

impl Default for SimulationControl {
    fn default() -> Self {
        Self {
            paused: false,
            step_mode: false,
            step_requested: false,
            speed_multiplier: 1.0,
            frame_count: 0,
            simulation_time: 0.0,
        }
    }
}

impl SimulationControl {
    /// Toggle pause state
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
        if self.paused {
            info!("Simulation paused");
        } else {
            info!("Simulation resumed");
            self.step_mode = false;
        }
    }

    /// Enter step mode (pauses and allows frame-by-frame advancement)
    pub fn enter_step_mode(&mut self) {
        self.paused = true;
        self.step_mode = true;
        info!("Entered step mode");
    }

    /// Request a single step (only works in step mode)
    pub fn request_step(&mut self) {
        if self.step_mode {
            self.step_requested = true;
        }
    }

    /// Check if simulation should run this frame
    pub fn should_update(&mut self) -> bool {
        if !self.paused {
            return true;
        }

        if self.step_mode && self.step_requested {
            self.step_requested = false;
            return true;
        }

        false
    }

    /// Set simulation speed multiplier
    pub fn set_speed(&mut self, multiplier: f32) {
        self.speed_multiplier = multiplier.clamp(0.0, 10.0);
        info!("Simulation speed set to {}x", self.speed_multiplier);
    }

    /// Get effective delta time (considering pause and speed)
    pub fn get_delta_time(&self, base_delta: f32) -> f32 {
        if self.paused && !self.step_requested {
            0.0
        } else {
            base_delta * self.speed_multiplier
        }
    }

    /// Update frame counter and simulation time
    pub fn update(&mut self, delta: f32) {
        if self.should_update() {
            self.frame_count += 1;
            self.simulation_time += self.get_delta_time(delta);
        }
    }
}

/// Plugin for simulation control
pub struct SimulationControlPlugin;

impl Plugin for SimulationControlPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SimulationControl>()
            .add_systems(PreUpdate, update_simulation_control)
            .add_systems(Update, handle_simulation_input);
    }
}

/// Update simulation control state
fn update_simulation_control(mut control: ResMut<SimulationControl>, time: Res<Time>) {
    control.update(time.delta_seconds());
}

/// Handle keyboard input for simulation control
fn handle_simulation_input(
    keyboard: Option<Res<ButtonInput<KeyCode>>>,
    mut control: ResMut<SimulationControl>,
) {
    // Skip if no keyboard input available (e.g., in tests)
    let Some(keyboard) = keyboard else { return };
    // Space: Toggle pause
    if keyboard.just_pressed(KeyCode::Space) {
        control.toggle_pause();
    }

    // P: Enter step mode
    if keyboard.just_pressed(KeyCode::KeyP) {
        control.enter_step_mode();
    }

    // Period (.): Step one frame (in step mode)
    if keyboard.just_pressed(KeyCode::Period) {
        control.request_step();
    }

    // Number keys: Set speed
    if keyboard.just_pressed(KeyCode::Digit1) {
        control.set_speed(0.1);
    } else if keyboard.just_pressed(KeyCode::Digit2) {
        control.set_speed(0.25);
    } else if keyboard.just_pressed(KeyCode::Digit3) {
        control.set_speed(0.5);
    } else if keyboard.just_pressed(KeyCode::Digit4) {
        control.set_speed(1.0);
    } else if keyboard.just_pressed(KeyCode::Digit5) {
        control.set_speed(2.0);
    } else if keyboard.just_pressed(KeyCode::Digit6) {
        control.set_speed(5.0);
    } else if keyboard.just_pressed(KeyCode::Digit7) {
        control.set_speed(10.0);
    }

    // Plus/Minus: Adjust speed
    if keyboard.just_pressed(KeyCode::Equal) || keyboard.just_pressed(KeyCode::NumpadAdd) {
        let new_speed = (control.speed_multiplier * 1.5).min(10.0);
        control.set_speed(new_speed);
    }
    if keyboard.just_pressed(KeyCode::Minus) || keyboard.just_pressed(KeyCode::NumpadSubtract) {
        let new_speed = (control.speed_multiplier / 1.5).max(0.1);
        control.set_speed(new_speed);
    }
}

/// System run condition for simulation systems
pub fn simulation_should_run(control: Res<SimulationControl>) -> bool {
    !control.paused || (control.step_mode && control.step_requested)
}

/// Helper to get scaled time for systems
pub fn get_scaled_time(control: &SimulationControl, delta: f32) -> f32 {
    control.get_delta_time(delta)
}

// Note: Systems should use .run_if(simulation_should_run) directly
// due to Bevy's type system constraints

#[cfg(test)]
mod tests;
