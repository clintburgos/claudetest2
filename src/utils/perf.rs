//! Performance measurement and profiling utilities.

use crate::config::time::MS_TO_SECONDS;
use std::time::{Duration, Instant};

/// Simple performance timer for measuring code execution time.
pub struct PerfTimer {
    label: String,
    start: Instant,
    threshold_ms: Option<f32>,
}

impl PerfTimer {
    /// Creates a new performance timer that starts immediately.
    ///
    /// # Arguments
    /// * `label` - Label for this measurement
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            start: Instant::now(),
            threshold_ms: None,
        }
    }

    /// Creates a timer that only logs if duration exceeds threshold.
    ///
    /// # Arguments
    /// * `label` - Label for this measurement
    /// * `threshold_ms` - Only log if duration exceeds this many milliseconds
    pub fn new_with_threshold(label: impl Into<String>, threshold_ms: f32) -> Self {
        Self {
            label: label.into(),
            start: Instant::now(),
            threshold_ms: Some(threshold_ms),
        }
    }

    /// Returns elapsed time without stopping the timer.
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Returns elapsed time in milliseconds.
    pub fn elapsed_ms(&self) -> f32 {
        self.elapsed().as_secs_f32() * MS_TO_SECONDS
    }

    /// Stops the timer and logs the result.
    pub fn stop(self) {
        let elapsed_ms = self.elapsed_ms();

        match self.threshold_ms {
            Some(threshold) if elapsed_ms < threshold => {
                // Don't log if under threshold
            },
            _ => {
                bevy::log::debug!("{}: {:.2}ms", self.label, elapsed_ms);
            },
        }
    }
}

/// Measures and returns the execution time of a closure.
///
/// # Arguments
/// * `f` - Function to measure
///
/// # Returns
/// Tuple of (result, duration_ms)
pub fn measure<F, R>(f: F) -> (R, f32)
where
    F: FnOnce() -> R,
{
    let start = Instant::now();
    let result = f();
    let duration = start.elapsed().as_secs_f32() * MS_TO_SECONDS;
    (result, duration)
}

/// Measures execution time and logs if it exceeds a threshold.
///
/// # Arguments
/// * `label` - Label for the measurement
/// * `threshold_ms` - Log only if execution exceeds this duration
/// * `f` - Function to measure
///
/// # Returns
/// The result of the function
pub fn measure_with_threshold<F, R>(label: &str, threshold_ms: f32, f: F) -> R
where
    F: FnOnce() -> R,
{
    let timer = PerfTimer::new_with_threshold(label, threshold_ms);
    let result = f();
    timer.stop();
    result
}

/// Simple frame time tracker for monitoring performance.
#[derive(Debug)]
pub struct FrameTimer {
    last_frame: Instant,
    frame_times: Vec<f32>,
    max_samples: usize,
}

impl FrameTimer {
    /// Creates a new frame timer.
    ///
    /// # Arguments
    /// * `max_samples` - Maximum number of frame times to track
    pub fn new(max_samples: usize) -> Self {
        Self {
            last_frame: Instant::now(),
            frame_times: Vec::with_capacity(max_samples),
            max_samples,
        }
    }

    /// Updates the frame timer and returns delta time.
    pub fn update(&mut self) -> f32 {
        let now = Instant::now();
        let delta = now.duration_since(self.last_frame).as_secs_f32();
        self.last_frame = now;

        // Track frame time
        if self.frame_times.len() >= self.max_samples {
            self.frame_times.remove(0);
        }
        self.frame_times.push(delta);

        delta
    }

    /// Returns the average frame time over recent samples.
    pub fn average_frame_time(&self) -> f32 {
        if self.frame_times.is_empty() {
            return 0.0;
        }

        let sum: f32 = self.frame_times.iter().sum();
        sum / self.frame_times.len() as f32
    }

    /// Returns the average FPS over recent samples.
    pub fn average_fps(&self) -> f32 {
        let avg_frame_time = self.average_frame_time();
        if avg_frame_time > 0.0 {
            1.0 / avg_frame_time
        } else {
            0.0
        }
    }

    /// Returns the minimum and maximum frame times.
    pub fn min_max_frame_time(&self) -> (f32, f32) {
        if self.frame_times.is_empty() {
            return (0.0, 0.0);
        }

        let min = self.frame_times.iter().copied().fold(f32::INFINITY, f32::min);
        let max = self.frame_times.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        (min, max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::demo::FRAME_SLEEP_MS;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_perf_timer() {
        let timer = PerfTimer::new("test");
        thread::sleep(Duration::from_millis(10));
        let elapsed = timer.elapsed_ms();
        assert!(elapsed >= 10.0);
    }

    #[test]
    fn test_measure() {
        let (result, duration) = measure(|| {
            thread::sleep(Duration::from_millis(10));
            42
        });

        assert_eq!(result, 42);
        assert!(duration >= 10.0);
    }

    #[test]
    fn test_frame_timer() {
        let mut timer = FrameTimer::new(3);

        // Simulate some frames
        thread::sleep(Duration::from_millis(FRAME_SLEEP_MS));
        let dt1 = timer.update();
        assert!(dt1 > 0.0);

        thread::sleep(Duration::from_millis(FRAME_SLEEP_MS));
        let dt2 = timer.update();
        assert!(dt2 > 0.0);

        // Check averages
        let avg_fps = timer.average_fps();
        assert!(avg_fps > 0.0 && avg_fps < 100.0); // Reasonable FPS range

        let (min, max) = timer.min_max_frame_time();
        assert!(min > 0.0 && min <= max);
    }
}
