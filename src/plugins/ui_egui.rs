use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

pub struct UiEguiPlugin;

impl Plugin for UiEguiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiState>()
            .add_systems(Update, ui_system);
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
    creatures: Query<&crate::components::Creature>,
    resources: Query<&crate::components::ResourceMarker>,
    time: Res<Time>,
    diagnostics: Res<bevy::diagnostic::DiagnosticsStore>,
) {
    // Top panel with basic stats
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
                ui.separator();
                
                ui.heading("Resources");
                ui.label(format!("Total Resources: {}", resources.iter().count()));
                
                if let Some(entity) = ui_state.selected_creature {
                    ui.separator();
                    ui.heading("Selected Creature");
                    ui.label(format!("Entity: {:?}", entity));
                    // Add more creature info here when available
                }
            });
    }

    // Debug window
    if ui_state.show_debug {
        egui::Window::new("Debug Info")
            .default_pos([10.0, 200.0])
            .show(contexts.ctx_mut(), |ui| {
                ui.heading("Performance");
                ui.label(format!("Frame Time: {:.2}ms", time.delta_seconds() * 1000.0));
                
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
                egui::Grid::new("controls_grid")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .show(ui, |ui| {
                        ui.label("Move:");
                        ui.label("WASD or Arrow Keys");
                        ui.end_row();
                        
                        ui.label("Zoom:");
                        ui.label("Q/E");
                        ui.end_row();
                    });
                
                ui.separator();
                ui.heading("Interaction");
                ui.label("Click: Select creature");
                ui.label("ESC: Deselect");
            });
    }
}