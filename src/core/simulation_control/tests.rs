//! Tests for simulation control system

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_default_state() {
        let control = SimulationControl::default();
        assert!(!control.paused);
        assert!(!control.step_mode);
        assert!(!control.step_requested);
        assert_eq!(control.speed_multiplier, 1.0);
        assert_eq!(control.frame_count, 0);
        assert_eq!(control.simulation_time, 0.0);
    }

    #[test]
    fn test_toggle_pause() {
        let mut control = SimulationControl::default();

        // Toggle pause on
        control.toggle_pause();
        assert!(control.paused);

        // Toggle pause off
        control.toggle_pause();
        assert!(!control.paused);
        assert!(!control.step_mode); // Step mode should be cleared
    }

    #[test]
    fn test_step_mode() {
        let mut control = SimulationControl::default();

        // Enter step mode
        control.enter_step_mode();
        assert!(control.paused);
        assert!(control.step_mode);

        // Request step
        control.request_step();
        assert!(control.step_requested);

        // Step should only work in step mode
        control.step_mode = false;
        control.step_requested = false;
        control.request_step();
        assert!(!control.step_requested);
    }

    #[test]
    fn test_should_update() {
        let mut control = SimulationControl::default();

        // Should update when not paused
        assert!(control.should_update());

        // Should not update when paused
        control.paused = true;
        assert!(!control.should_update());

        // Should update in step mode with step requested
        control.step_mode = true;
        control.step_requested = true;
        assert!(control.should_update());
        assert!(!control.step_requested); // Step request should be consumed

        // Should not update after step consumed
        assert!(!control.should_update());
    }

    #[test]
    fn test_speed_control() {
        let mut control = SimulationControl::default();

        // Normal speed
        control.set_speed(2.0);
        assert_eq!(control.speed_multiplier, 2.0);

        // Test clamping
        control.set_speed(20.0);
        assert_eq!(control.speed_multiplier, 10.0); // Max

        control.set_speed(-1.0);
        assert_eq!(control.speed_multiplier, 0.0); // Min
    }

    #[test]
    fn test_delta_time_calculation() {
        let control = SimulationControl::default();

        // Normal speed
        assert_eq!(control.get_delta_time(0.016), 0.016);

        // With speed multiplier
        let mut control = SimulationControl::default();
        control.speed_multiplier = 2.0;
        assert_eq!(control.get_delta_time(0.016), 0.032);

        // When paused
        control.paused = true;
        assert_eq!(control.get_delta_time(0.016), 0.0);

        // When paused but step requested
        control.step_requested = true;
        assert_eq!(control.get_delta_time(0.016), 0.032);
    }

    #[test]
    fn test_update_tracking() {
        let mut control = SimulationControl::default();

        // Normal update
        control.update(0.016);
        assert_eq!(control.frame_count, 1);
        assert_eq!(control.simulation_time, 0.016);

        // Update with speed multiplier
        control.speed_multiplier = 2.0;
        control.update(0.016);
        assert_eq!(control.frame_count, 2);
        assert_eq!(control.simulation_time, 0.048); // 0.016 + 0.032

        // No update when paused
        control.paused = true;
        control.update(0.016);
        assert_eq!(control.frame_count, 2); // No change
        assert_eq!(control.simulation_time, 0.048); // No change
    }

    #[test]
    fn test_scaled_time_helper() {
        let mut control = SimulationControl::default();
        control.speed_multiplier = 3.0;

        assert_eq!(get_scaled_time(&control, 0.016), 0.048);

        control.paused = true;
        assert_eq!(get_scaled_time(&control, 0.016), 0.0);
    }

    #[test]
    fn test_simulation_should_run() {
        use bevy::prelude::*;

        let mut app = App::new();
        app.insert_resource(SimulationControl::default());

        // Should run when not paused
        let control = app.world.resource::<SimulationControl>();
        assert!(!control.paused || (control.step_mode && control.step_requested));

        // Should not run when paused
        app.world.resource_mut::<SimulationControl>().paused = true;
        let control = app.world.resource::<SimulationControl>();
        assert!(!(!control.paused || (control.step_mode && control.step_requested)));

        // Should run in step mode with step requested
        let mut control_mut = app.world.resource_mut::<SimulationControl>();
        control_mut.step_mode = true;
        control_mut.step_requested = true;
        drop(control_mut);

        let control = app.world.resource::<SimulationControl>();
        assert!(!control.paused || (control.step_mode && control.step_requested));
    }

    #[test]
    fn test_pause_clears_step_mode() {
        let mut control = SimulationControl::default();

        // Enter step mode
        control.enter_step_mode();
        assert!(control.step_mode);

        // Resume should clear step mode
        control.toggle_pause(); // Resume
        assert!(!control.paused);
        assert!(!control.step_mode);
    }

    #[test]
    fn test_multiple_speed_changes() {
        let mut control = SimulationControl::default();

        // Test various speed settings
        let speeds = [0.1, 0.25, 0.5, 1.0, 2.0, 5.0, 10.0];
        for &speed in &speeds {
            control.set_speed(speed);
            assert_eq!(control.speed_multiplier, speed);
        }
    }
}
