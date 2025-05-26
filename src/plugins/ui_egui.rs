use crate::core::performance_monitor::PerformanceMonitor;
use crate::core::simulation_control::SimulationControl;
use crate::systems::observation_goals::ObservationGoals;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

pub struct UiEguiPlugin;

impl Plugin for UiEguiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiState>().add_systems(Update, ui_system);
    }
}

#[derive(Resource, Default)]
pub struct UiState {
    pub show_debug: bool,
    pub show_stats: bool,
    pub show_controls: bool,
    pub selected_creature: Option<Entity>,
}

fn ui_system(
    mut contexts: EguiContexts,
    mut ui_state: ResMut<UiState>,
    mut settings: ResMut<crate::plugins::SimulationSettings>,
    simulation_control: Res<SimulationControl>,
    performance_monitor: Res<PerformanceMonitor>,
    observation_goals: Res<ObservationGoals>,
    creatures: Query<(
        Entity,
        &crate::components::Creature,
        &crate::components::Position,
        &crate::components::Health,
        &crate::components::Needs,
        &crate::components::CreatureState,
        &crate::components::CreatureType,
    )>,
    resources: Query<&crate::components::ResourceMarker>,
    _time: Res<Time>,
    diagnostics: Res<bevy::diagnostic::DiagnosticsStore>,
) {
    // Top panel with basic stats and time controls
    egui::TopBottomPanel::top("top_panel").show(contexts.ctx_mut(), |ui| {
        ui.horizontal(|ui| {
            // FPS display
            if let Some(fps) = diagnostics.get(&bevy::diagnostic::FrameTimeDiagnosticsPlugin::FPS) {
                if let Some(value) = fps.smoothed() {
                    ui.label(format!("FPS: {:.1}", value));
                    ui.separator();
                }
            }

            // Entity counts
            ui.label(format!("Creatures: {}", creatures.iter().count()));
            ui.separator();
            ui.label(format!("Resources: {}", resources.iter().count()));
            ui.separator();

            // Time controls
            ui.label("Speed:");
            if ui.button(if settings.paused { "▶" } else { "⏸" }).clicked() {
                settings.paused = !settings.paused;
            }

            let speed_buttons = [
                ("0.5x", 0.5),
                ("1x", 1.0),
                ("2x", 2.0),
                ("5x", 5.0),
                ("10x", 10.0),
            ];

            for (label, speed) in speed_buttons {
                if ui.selectable_label(settings.speed_multiplier == speed, label).clicked() {
                    settings.speed_multiplier = speed;
                }
            }

            ui.separator();

            // Toggle buttons
            if ui.button("📊 Stats").clicked() {
                ui_state.show_stats = !ui_state.show_stats;
            }
            if ui.button("🐛 Debug").clicked() {
                ui_state.show_debug = !ui_state.show_debug;
            }
            if ui.button("🎮 Controls").clicked() {
                ui_state.show_controls = !ui_state.show_controls;
            }
        });
    });

    // Stats window
    if ui_state.show_stats {
        egui::Window::new("Statistics")
            .default_pos([10.0, 50.0])
            .show(contexts.ctx_mut(), |ui| {
                let alive_count = creatures.iter().count();

                ui.heading("Population");
                ui.label(format!("Total Creatures: {}", alive_count));

                // Count by type
                let mut herbivores = 0;
                let mut carnivores = 0;
                let mut omnivores = 0;

                for (_, _, _, _, _, _, creature_type) in creatures.iter() {
                    match creature_type {
                        crate::components::CreatureType::Herbivore => herbivores += 1,
                        crate::components::CreatureType::Carnivore => carnivores += 1,
                        crate::components::CreatureType::Omnivore => omnivores += 1,
                    }
                }

                ui.label(format!("Herbivores: {}", herbivores));
                ui.label(format!("Carnivores: {}", carnivores));
                ui.label(format!("Omnivores: {}", omnivores));

                ui.separator();

                ui.heading("Resources");
                ui.label(format!("Total Resources: {}", resources.iter().count()));

                if let Some(selected_entity) = ui_state.selected_creature {
                    if let Ok((_entity, _, position, health, needs, state, creature_type)) =
                        creatures.get(selected_entity)
                    {
                        ui.separator();
                        ui.heading("Selected Creature");

                        egui::Grid::new("creature_details")
                            .num_columns(2)
                            .spacing([40.0, 4.0])
                            .show(ui, |ui| {
                                ui.label("Type:");
                                ui.label(format!("{:?}", creature_type));
                                ui.end_row();

                                ui.label("State:");
                                ui.label(format!("{:?}", state));
                                ui.end_row();

                                ui.label("Position:");
                                ui.label(format!("({:.1}, {:.1})", position.0.x, position.0.y));
                                ui.end_row();

                                ui.label("Health:");
                                ui.add(
                                    egui::ProgressBar::new(health.current / health.max)
                                        .text(format!("{:.0}/{:.0}", health.current, health.max)),
                                );
                                ui.end_row();

                                ui.label("Hunger:");
                                ui.add(
                                    egui::ProgressBar::new(needs.hunger)
                                        .text(format!("{:.0}%", needs.hunger * 100.0)),
                                );
                                ui.end_row();

                                ui.label("Thirst:");
                                ui.add(
                                    egui::ProgressBar::new(needs.thirst)
                                        .text(format!("{:.0}%", needs.thirst * 100.0)),
                                );
                                ui.end_row();

                                ui.label("Energy:");
                                ui.add(
                                    egui::ProgressBar::new(needs.energy)
                                        .text(format!("{:.0}%", needs.energy * 100.0)),
                                );
                                ui.end_row();
                            });
                    } else {
                        ui.label("Selected creature no longer exists");
                        ui_state.selected_creature = None;
                    }
                }
            });
    }

    // Observation Goals window
    egui::Window::new("Observation Goals")
        .default_pos([10.0, 400.0])
        .show(contexts.ctx_mut(), |ui| {
            ui.heading("Mission Objectives");
            
            // Show completion percentage
            let completion = observation_goals.get_completion_percentage();
            ui.add(egui::ProgressBar::new(completion / 100.0)
                .text(format!("{:.0}% Complete", completion)));
            
            ui.separator();
            
            // Show active goal
            if let Some(active_goal_id) = &observation_goals.active_goal {
                if let Some(goal) = observation_goals.goals.get(active_goal_id) {
                    ui.heading("Current Goal");
                    ui.label(&goal.name);
                    ui.label(&goal.description);
                    ui.add(egui::ProgressBar::new(goal.progress / goal.target)
                        .text(format!("{:.0}/{:.0}", goal.progress, goal.target)));
                }
            }
            
            ui.separator();
            
            // List all goals
            ui.heading("All Goals");
            egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                for goal in observation_goals.goals.values() {
                    ui.horizontal(|ui| {
                        if goal.completed {
                            ui.label("✓");
                        } else {
                            ui.label("○");
                        }
                        ui.label(&goal.name);
                    });
                }
            });
        });

    // Debug window
    if ui_state.show_debug {
        egui::Window::new("Debug Info")
            .default_pos([10.0, 200.0])
            .show(contexts.ctx_mut(), |ui| {
                ui.heading("Performance");

                // Get performance stats
                let perf_stats = performance_monitor.get_stats();

                ui.label(format!("Average FPS: {:.1}", perf_stats.avg_fps));
                ui.label(format!("Min FPS: {:.1}", perf_stats.min_fps));
                ui.label(format!(
                    "Frame Time: {:.2}ms (max: {:.2}ms)",
                    perf_stats.avg_frame_ms, perf_stats.max_frame_ms
                ));
                ui.label(format!("Quality Level: {:?}", perf_stats.quality_level));

                if perf_stats.warning_count > 0 {
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 150, 50),
                        format!("⚠️ {} performance warnings", perf_stats.warning_count),
                    );
                }

                ui.separator();
                ui.heading("Simulation");
                ui.label(format!("Frame: {}", simulation_control.frame_count));
                ui.label(format!(
                    "Sim Time: {:.1}s",
                    simulation_control.simulation_time
                ));
                ui.label(format!("Speed: {}x", simulation_control.speed_multiplier));
                if simulation_control.paused {
                    ui.colored_label(egui::Color32::from_rgb(255, 255, 0), "PAUSED");
                }
                if simulation_control.step_mode {
                    ui.colored_label(egui::Color32::from_rgb(0, 255, 255), "STEP MODE");
                }

                ui.separator();
                ui.heading("Debug Toggles");
                ui.label("Press F1-F4 for debug visualizations:");
                ui.label("F1: Toggle FPS overlay");
                ui.label("F2: Toggle entity IDs");
                ui.label("F3: Toggle creature states");
                ui.label("F4: Toggle spatial grid");
            });
    }

    // Controls window
    if ui_state.show_controls {
        egui::Window::new("Controls")
            .default_pos([10.0, 350.0])
            .show(contexts.ctx_mut(), |ui| {
                ui.heading("Camera Controls");
                egui::Grid::new("controls_grid").num_columns(2).spacing([40.0, 4.0]).show(
                    ui,
                    |ui| {
                        ui.label("Move:");
                        ui.label("WASD or Arrow Keys");
                        ui.end_row();

                        ui.label("Zoom:");
                        ui.label("Q/E");
                        ui.end_row();
                    },
                );

                ui.separator();
                ui.heading("Simulation");
                egui::Grid::new("sim_controls_grid").num_columns(2).spacing([40.0, 4.0]).show(
                    ui,
                    |ui| {
                        ui.label("Pause/Resume:");
                        ui.label("Space");
                        ui.end_row();

                        ui.label("Step Mode:");
                        ui.label("P");
                        ui.end_row();

                        ui.label("Step Frame:");
                        ui.label(". (period)");
                        ui.end_row();

                        ui.label("Speed:");
                        ui.label("1-7 or +/-");
                        ui.end_row();
                    },
                );

                ui.separator();
                ui.heading("Interaction");
                ui.label("Click: Select creature");
                ui.label("ESC: Deselect");
            });
    }
}
