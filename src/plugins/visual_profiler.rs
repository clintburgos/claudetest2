//! Visual profiler overlay for real-time performance monitoring
//!
//! Displays comprehensive performance metrics including FPS, frame times, memory usage,
//! entity counts, system timing breakdowns, and spatial grid statistics.
//! Toggle with F9 key.

use bevy::prelude::*;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy_egui::{egui, EguiContexts};
use std::collections::VecDeque;

use crate::core::memory_profiler::MemoryProfiler;
use crate::core::performance_monitor::{PerformanceMonitor, PerformanceWarning};
use crate::plugins::spatial::SpatialGrid;

/// Plugin for visual profiler overlay
pub struct VisualProfilerPlugin;

impl Plugin for VisualProfilerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VisualProfilerState>()
            .init_resource::<PerformanceHistory>()
            .add_systems(Update, (
                toggle_profiler,
                update_performance_history,
                render_profiler_overlay.run_if(resource_equals(VisualProfilerState { visible: true })),
            ).chain());
    }
}

/// State for the visual profiler
#[derive(Resource, Default, PartialEq)]
struct VisualProfilerState {
    visible: bool,
}

/// Historical performance data for graphs
#[derive(Resource)]
struct PerformanceHistory {
    fps_history: VecDeque<f32>,
    frame_time_history: VecDeque<f32>,
    entity_count_history: VecDeque<f32>,
    memory_history: VecDeque<f32>,
    history_size: usize,
}

impl Default for PerformanceHistory {
    fn default() -> Self {
        let history_size = 120; // 2 seconds at 60 FPS
        Self {
            fps_history: VecDeque::with_capacity(history_size),
            frame_time_history: VecDeque::with_capacity(history_size),
            entity_count_history: VecDeque::with_capacity(history_size),
            memory_history: VecDeque::with_capacity(history_size),
            history_size,
        }
    }
}

/// System to toggle profiler visibility with F9
fn toggle_profiler(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<VisualProfilerState>,
) {
    if keyboard.just_pressed(KeyCode::F9) {
        state.visible = !state.visible;
        if state.visible {
            info!("Visual profiler enabled");
        } else {
            info!("Visual profiler disabled");
        }
    }
}

/// System to update performance history
fn update_performance_history(
    mut history: ResMut<PerformanceHistory>,
    perf_monitor: Option<Res<PerformanceMonitor>>,
    memory_profiler: Option<Res<MemoryProfiler>>,
    diagnostics: Res<DiagnosticsStore>,
) {
    let history_size = history.history_size;
    
    // Get FPS from diagnostics
    if let Some(fps) = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
    {
        add_to_history(&mut history.fps_history, fps as f32, history_size);
    }

    // Get frame time from diagnostics
    if let Some(frame_time) = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(|d| d.smoothed())
    {
        add_to_history(&mut history.frame_time_history, frame_time as f32 * 1000.0, history_size);
    }

    // Get entity count and other stats from performance monitor
    if let Some(monitor) = perf_monitor {
        let stats = monitor.get_stats();
        add_to_history(&mut history.entity_count_history, stats.entity_count as f32, history_size);
    }

    // Get memory stats
    if let Some(profiler) = memory_profiler {
        let stats = profiler.get_stats();
        let memory_mb = stats.estimated_memory as f32 / (1024.0 * 1024.0);
        add_to_history(&mut history.memory_history, memory_mb, history_size);
    }
}

/// Helper to add value to history deque
fn add_to_history(history: &mut VecDeque<f32>, value: f32, max_size: usize) {
    if history.len() >= max_size {
        history.pop_front();
    }
    history.push_back(value);
}

/// System to render the profiler overlay
fn render_profiler_overlay(
    mut contexts: EguiContexts,
    perf_monitor: Option<Res<PerformanceMonitor>>,
    memory_profiler: Option<Res<MemoryProfiler>>,
    spatial_grid: Option<Res<SpatialGrid>>,
    history: Res<PerformanceHistory>,
    diagnostics: Res<DiagnosticsStore>,
) {
    let ctx = contexts.ctx_mut();

    // Configure overlay style
    let window_margin = egui::Margin::symmetric(10.0, 10.0);
    let window_frame = egui::Frame::window(&ctx.style())
        .fill(egui::Color32::from_rgba_unmultiplied(20, 20, 20, 220))
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 60, 60)))
        .inner_margin(window_margin)
        .rounding(egui::Rounding::same(5.0));

    // Position in top-right corner
    egui::Window::new("Performance Profiler")
        .default_pos(egui::pos2(ctx.screen_rect().width() - 400.0, 10.0))
        .default_width(380.0)
        .collapsible(true)
        .resizable(true)
        .frame(window_frame)
        .show(ctx, |ui| {
            ui.style_mut().spacing.item_spacing = egui::vec2(8.0, 4.0);

            // FPS and Frame Time Section
            ui.colored_label(egui::Color32::from_rgb(100, 200, 100), "📊 Performance Metrics");
            ui.separator();

            if let Some(fps) = diagnostics
                .get(&FrameTimeDiagnosticsPlugin::FPS)
                .and_then(|d| d.smoothed())
            {
                let fps_color = get_fps_color(fps as f32);
                ui.horizontal(|ui| {
                    ui.label("FPS:");
                    ui.colored_label(fps_color, format!("{:.1}", fps));
                    
                    if let Some(frame_time) = diagnostics
                        .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
                        .and_then(|d| d.smoothed())
                    {
                        ui.label("Frame:");
                        ui.colored_label(fps_color, format!("{:.2}ms", frame_time * 1000.0));
                    }
                });

                // FPS Graph
                plot_graph(ui, "FPS History", &history.fps_history, 0.0, 120.0, |v| *v, fps_color);
            }

            // Performance Monitor Stats
            if let Some(monitor) = perf_monitor {
                let stats = monitor.get_stats();
                
                ui.spacing();
                ui.colored_label(egui::Color32::from_rgb(100, 150, 200), "⚡ System Performance");
                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Entities:");
                    ui.strong(stats.entity_count.to_string());
                    ui.label("Quality:");
                    ui.colored_label(
                        get_quality_color(&stats.quality_level),
                        format!("{:?}", stats.quality_level)
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("Avg Frame:");
                    ui.label(format!("{:.2}ms", stats.avg_frame_ms));
                    ui.label("Max Frame:");
                    let max_color = if stats.max_frame_ms > 16.67 {
                        egui::Color32::from_rgb(200, 100, 100)
                    } else {
                        egui::Color32::from_rgb(100, 200, 100)
                    };
                    ui.colored_label(max_color, format!("{:.2}ms", stats.max_frame_ms));
                });

                // Frame Time Graph
                plot_graph(ui, "Frame Time (ms)", &history.frame_time_history, 0.0, 33.0, |v| *v, egui::Color32::from_rgb(200, 150, 100));

                // Performance Warnings
                let warnings = monitor.get_warnings();
                if !warnings.is_empty() {
                    ui.spacing();
                    ui.colored_label(egui::Color32::from_rgb(200, 100, 100), "⚠️ Performance Warnings");
                    ui.separator();
                    
                    let max_warnings = 5;
                    for (i, warning) in warnings.iter().take(max_warnings).enumerate() {
                        render_warning(ui, warning);
                        if i < warnings.len().min(max_warnings) - 1 {
                            ui.separator();
                        }
                    }
                    
                    if warnings.len() > max_warnings {
                        ui.label(format!("... and {} more", warnings.len() - max_warnings));
                    }
                }
            }

            // Memory Stats
            if let Some(profiler) = memory_profiler {
                let stats = profiler.get_stats();
                
                ui.spacing();
                ui.colored_label(egui::Color32::from_rgb(150, 100, 200), "💾 Memory Usage");
                ui.separator();

                let memory_mb = stats.estimated_memory as f32 / (1024.0 * 1024.0);
                ui.horizontal(|ui| {
                    ui.label("Est. Memory:");
                    ui.strong(format!("{:.1} MB", memory_mb));
                    ui.label("Components:");
                    ui.label(stats.total_components.to_string());
                });

                // Memory Graph
                plot_graph(ui, "Memory (MB)", &history.memory_history, 0.0, 100.0, |v| *v, egui::Color32::from_rgb(150, 100, 200));

                // Component breakdown
                if !stats.component_stats.is_empty() {
                    ui.spacing();
                    ui.label("Component Breakdown:");
                    ui.indent("component_breakdown", |ui| {
                        for (name, comp_stats) in stats.component_stats.iter().take(5) {
                            ui.horizontal(|ui| {
                                ui.label(format!("{}:", name));
                                ui.label(format!("{} ({:.1} KB)", 
                                    comp_stats.count, 
                                    comp_stats.estimated_size as f32 / 1024.0
                                ));
                            });
                        }
                    });
                }

                // Memory leak warnings
                if !stats.potential_leaks.is_empty() {
                    ui.spacing();
                    ui.colored_label(egui::Color32::from_rgb(200, 50, 50), "🚨 Potential Memory Leaks");
                    for leak in &stats.potential_leaks {
                        ui.label(format!("• {}", leak));
                    }
                }
            }

            // Spatial Grid Stats
            if let Some(_grid) = spatial_grid {
                ui.spacing();
                ui.colored_label(egui::Color32::from_rgb(100, 200, 150), "🗺️ Spatial Grid");
                ui.separator();

                // Note: We'd need to expose grid stats from SpatialGrid
                ui.label("Grid statistics available when spatial grid exposes metrics");
            }

            // Controls hint
            ui.spacing();
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Press");
                ui.colored_label(egui::Color32::from_rgb(150, 150, 150), "F9");
                ui.label("to toggle profiler");
            });
        });
}

/// Render a performance warning
fn render_warning(ui: &mut egui::Ui, warning: &PerformanceWarning) {
    match warning {
        PerformanceWarning::LowFPS { current, target } => {
            ui.horizontal(|ui| {
                ui.colored_label(egui::Color32::from_rgb(200, 100, 100), "Low FPS:");
                ui.label(format!("{:.1} / {:.1}", current, target));
            });
        }
        PerformanceWarning::SystemOverBudget { system, time_ms, budget_ms } => {
            ui.horizontal(|ui| {
                ui.colored_label(egui::Color32::from_rgb(200, 150, 100), format!("{} over budget:", system));
                ui.label(format!("{:.1}ms / {:.1}ms", time_ms, budget_ms));
            });
        }
        PerformanceWarning::FrameSpike { frame, time_ms, baseline_ms } => {
            ui.horizontal(|ui| {
                ui.colored_label(egui::Color32::from_rgb(200, 100, 150), "Frame spike:");
                ui.label(format!("#{} {:.1}ms (baseline: {:.1}ms)", frame, time_ms, baseline_ms));
            });
        }
        PerformanceWarning::MemoryPressure { used_mb, available_mb } => {
            ui.horizontal(|ui| {
                ui.colored_label(egui::Color32::from_rgb(200, 50, 50), "Memory pressure:");
                ui.label(format!("{:.1}MB / {:.1}MB", used_mb, available_mb));
            });
        }
    }
}

/// Plot a simple line graph
fn plot_graph<F>(
    ui: &mut egui::Ui,
    label: &str,
    data: &VecDeque<f32>,
    min_value: f32,
    max_value: f32,
    value_fn: F,
    color: egui::Color32,
) where
    F: Fn(&f32) -> f32,
{
    if data.is_empty() {
        return;
    }

    ui.label(label);
    
    let height = 50.0;
    let (response, painter) = ui.allocate_painter(
        egui::Vec2::new(ui.available_width(), height),
        egui::Sense::hover(),
    );

    let rect = response.rect;
    painter.rect_filled(rect, 2.0, egui::Color32::from_rgba_unmultiplied(40, 40, 40, 200));

    // Draw grid lines
    let grid_color = egui::Color32::from_rgba_unmultiplied(60, 60, 60, 100);
    for i in 0..=4 {
        let y = rect.top() + (rect.height() * i as f32 / 4.0);
        painter.line_segment(
            [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
            egui::Stroke::new(0.5, grid_color),
        );
    }

    // Plot data
    if data.len() > 1 {
        let points: Vec<egui::Pos2> = data
            .iter()
            .enumerate()
            .map(|(i, value)| {
                let v = value_fn(value);
                let x = rect.left() + (rect.width() * i as f32 / (data.len() - 1) as f32);
                let normalized = ((v - min_value) / (max_value - min_value)).clamp(0.0, 1.0);
                let y = rect.bottom() - (rect.height() * normalized);
                egui::pos2(x, y)
            })
            .collect();

        for window in points.windows(2) {
            painter.line_segment(
                [window[0], window[1]],
                egui::Stroke::new(1.5, color),
            );
        }

        // Draw current value
        if let Some(last_value) = data.back() {
            let v = value_fn(last_value);
            ui.horizontal(|ui| {
                ui.label("Current:");
                ui.colored_label(color, format!("{:.1}", v));
            });
        }
    }
}

/// Get color based on FPS value
fn get_fps_color(fps: f32) -> egui::Color32 {
    if fps >= 60.0 {
        egui::Color32::from_rgb(100, 200, 100)
    } else if fps >= 45.0 {
        egui::Color32::from_rgb(200, 200, 100)
    } else if fps >= 30.0 {
        egui::Color32::from_rgb(200, 150, 100)
    } else {
        egui::Color32::from_rgb(200, 100, 100)
    }
}

/// Get color based on quality level
fn get_quality_color(quality: &crate::core::performance_monitor::QualityLevel) -> egui::Color32 {
    use crate::core::performance_monitor::QualityLevel;
    match quality {
        QualityLevel::Ultra => egui::Color32::from_rgb(100, 150, 250),
        QualityLevel::High => egui::Color32::from_rgb(100, 200, 100),
        QualityLevel::Medium => egui::Color32::from_rgb(200, 200, 100),
        QualityLevel::Low => egui::Color32::from_rgb(200, 150, 100),
        QualityLevel::Minimal => egui::Color32::from_rgb(200, 100, 100),
    }
}