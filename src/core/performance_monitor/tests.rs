//! Tests for the performance monitoring system

#[cfg(test)]
mod tests {
    use super::super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_frame_metrics_tracking() {
        let mut monitor = PerformanceMonitor::default();

        // Begin and end a frame
        monitor.begin_frame();
        thread::sleep(Duration::from_millis(10));
        monitor.end_frame(100);

        // Check frame was recorded
        assert_eq!(monitor.current_frame, 1);
        assert!(!monitor.frame_history.is_empty());
    }

    #[test]
    fn test_system_timing() {
        let mut monitor = PerformanceMonitor::default();

        monitor.begin_frame();

        // Time a system
        monitor.begin_system("test_system");
        thread::sleep(Duration::from_millis(5));
        monitor.end_system("test_system");

        monitor.end_frame(100);

        // Check system time was recorded
        let frame = monitor.frame_history.back().unwrap();
        assert!(frame.system_times.contains_key("test_system"));
        let system_time = frame.system_times.get("test_system").unwrap();
        assert!(system_time.as_millis() >= 5);
    }

    #[test]
    fn test_system_over_budget_warning() {
        let mut monitor = PerformanceMonitor::default();
        monitor.thresholds.system_budget_ms.insert("slow_system", 1.0);

        monitor.begin_frame();
        monitor.begin_system("slow_system");
        thread::sleep(Duration::from_millis(5));
        monitor.end_system("slow_system");
        monitor.end_frame(100);

        // Should have a warning
        assert!(!monitor.warnings.is_empty());
        assert!(matches!(
            monitor.warnings[0],
            PerformanceWarning::SystemOverBudget { .. }
        ));
    }

    #[test]
    fn test_fps_calculation() {
        let mut monitor = PerformanceMonitor::default();

        // Simulate 10 frames
        for _ in 0..10 {
            monitor.begin_frame();
            thread::sleep(Duration::from_millis(16)); // ~60 FPS
            monitor.end_frame(100);
        }

        let stats = monitor.get_stats();
        assert!(stats.avg_fps > 50.0 && stats.avg_fps < 70.0);
        assert!(stats.entity_count == 100);
    }

    #[test]
    fn test_low_fps_warning() {
        let mut monitor = PerformanceMonitor::default();
        monitor.thresholds.critical_fps = 30.0;

        monitor.begin_frame();
        thread::sleep(Duration::from_millis(50)); // ~20 FPS
        monitor.end_frame(100);

        // Should have low FPS warning
        assert!(!monitor.warnings.is_empty());
        assert!(matches!(
            monitor.warnings[0],
            PerformanceWarning::LowFPS { .. }
        ));
    }

    #[test]
    fn test_frame_spike_detection() {
        let mut monitor = PerformanceMonitor::default();

        // Establish baseline with fast frames
        for _ in 0..35 {
            monitor.begin_frame();
            thread::sleep(Duration::from_millis(10));
            monitor.end_frame(100);
        }

        // Trigger update_baseline
        monitor.update_baseline();
        assert!(monitor.baseline_frame_time.is_some());

        // Create a spike
        monitor.begin_frame();
        thread::sleep(Duration::from_millis(50)); // Spike
        monitor.end_frame(100);

        // Should detect spike
        let has_spike = monitor
            .warnings
            .iter()
            .any(|w| matches!(w, PerformanceWarning::FrameSpike { .. }));
        assert!(has_spike);
    }

    #[test]
    fn test_quality_degradation() {
        let mut monitor = PerformanceMonitor::default();
        monitor.thresholds.warning_fps = 45.0;
        monitor.degradation_level = QualityLevel::High;

        // Simulate low FPS
        for _ in 0..65 {
            monitor.begin_frame();
            thread::sleep(Duration::from_millis(30)); // ~33 FPS
            monitor.end_frame(100);
        }

        monitor.adjust_quality();

        // Should degrade quality
        assert_eq!(monitor.degradation_level, QualityLevel::Medium);
    }

    #[test]
    fn test_quality_improvement() {
        let mut monitor = PerformanceMonitor::default();
        monitor.thresholds.target_fps = 60.0;
        monitor.degradation_level = QualityLevel::Low;

        // Simulate high FPS
        for _ in 0..65 {
            monitor.begin_frame();
            thread::sleep(Duration::from_millis(10)); // ~100 FPS
            monitor.end_frame(100);
        }

        monitor.adjust_quality();

        // Should improve quality
        assert_eq!(monitor.degradation_level, QualityLevel::Medium);
    }

    #[test]
    fn test_quality_settings() {
        let monitor = PerformanceMonitor::default();

        // Test each quality level
        let ultra_settings = monitor.get_quality_settings();
        assert!(ultra_settings.max_creatures.is_some());
        assert_eq!(ultra_settings.update_frequency, 1);
        assert!(ultra_settings.enable_shadows);

        let mut monitor = PerformanceMonitor::default();
        monitor.degradation_level = QualityLevel::Minimal;
        let minimal_settings = monitor.get_quality_settings();
        assert_eq!(minimal_settings.max_creatures, Some(150));
        assert_eq!(minimal_settings.update_frequency, 3);
        assert!(!minimal_settings.enable_shadows);
        assert!(!minimal_settings.enable_particles);
    }

    #[test]
    fn test_frame_history_limit() {
        let mut monitor = PerformanceMonitor::default();

        // Add more frames than history size
        for i in 0..150 {
            monitor.begin_frame();
            monitor.end_frame(i);
        }

        // History should be limited
        assert_eq!(monitor.frame_history.len(), monitor.history_size);
    }

    #[test]
    fn test_clear_warnings() {
        let mut monitor = PerformanceMonitor::default();

        // Add some warnings
        monitor.warnings.push(PerformanceWarning::LowFPS {
            current: 30.0,
            target: 60.0,
        });

        assert!(!monitor.warnings.is_empty());

        monitor.clear_warnings();
        assert!(monitor.warnings.is_empty());
    }

    #[test]
    fn test_performance_stats_percentiles() {
        let mut monitor = PerformanceMonitor::default();

        // Add varied frame times
        for i in 0..100 {
            monitor.begin_frame();
            let sleep_time = if i % 10 == 0 { 30 } else { 16 };
            thread::sleep(Duration::from_millis(sleep_time));
            monitor.end_frame(100);
        }

        let stats = monitor.get_stats();
        assert!(stats.min_fps < stats.avg_fps);
        assert!(stats.max_frame_ms > stats.avg_frame_ms);
    }

    #[test]
    fn test_empty_monitor_stats() {
        let monitor = PerformanceMonitor::default();
        let stats = monitor.get_stats();

        assert_eq!(stats.avg_fps, 0.0);
        assert_eq!(stats.entity_count, 0);
        assert_eq!(stats.quality_level, QualityLevel::High);
    }

    #[test]
    fn test_quality_bounds() {
        let mut monitor = PerformanceMonitor::default();

        // Test can't go above Ultra
        monitor.degradation_level = QualityLevel::Ultra;
        monitor.thresholds.target_fps = 60.0;

        // Simulate very high FPS
        for _ in 0..65 {
            monitor.begin_frame();
            thread::sleep(Duration::from_millis(5));
            monitor.end_frame(100);
        }

        monitor.adjust_quality();
        assert_eq!(monitor.degradation_level, QualityLevel::Ultra);

        // Test can't go below Minimal
        monitor.degradation_level = QualityLevel::Minimal;
        monitor.thresholds.warning_fps = 45.0;

        // Simulate very low FPS
        for _ in 0..65 {
            monitor.begin_frame();
            thread::sleep(Duration::from_millis(100));
            monitor.end_frame(100);
        }

        monitor.adjust_quality();
        assert_eq!(monitor.degradation_level, QualityLevel::Minimal);
    }

    #[test]
    fn test_profile_system_macro() {
        let mut monitor = PerformanceMonitor::default();

        monitor.begin_frame();

        let result = profile_system!(monitor, "test_macro", {
            thread::sleep(Duration::from_millis(5));
            42
        });

        assert_eq!(result, 42);

        monitor.end_frame(100);

        let frame = monitor.frame_history.back().unwrap();
        assert!(frame.system_times.contains_key("test_macro"));
    }
}
