use bevy::prelude::*;
// use bevy_egui::{egui, EguiContexts};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiState>()
            .add_systems(Startup, setup_ui)
            .add_systems(Update, (update_fps_text, update_stats_text));
    }
}

#[derive(Resource, Default)]
pub struct UiState {
    pub show_debug: bool,
    pub show_stats: bool,
    pub selected_creature: Option<Entity>,
}

#[derive(Component)]
struct FpsText;

#[derive(Component)]
struct StatsText;

fn setup_ui(mut commands: Commands) {
    // Create UI camera
    commands.spawn(Camera2dBundle {
        camera: Camera {
            order: 1, // Render on top of main camera
            clear_color: ClearColorConfig::None,
            ..default()
        },
        ..default()
    });

    // FPS counter
    commands.spawn((
        TextBundle::from_section(
            "FPS: 0.0",
            TextStyle {
                font_size: 20.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        }),
        FpsText,
    ));

    // Stats text
    commands.spawn((
        TextBundle::from_section(
            "Creatures: 0 | Resources: 0",
            TextStyle {
                font_size: 20.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            ..default()
        }),
        StatsText,
    ));

    // Controls help text
    commands.spawn(
        TextBundle::from_section(
            "Controls: WASD/Arrows - Move | Q/E - Zoom | F1-F4 - Debug toggles",
            TextStyle {
                font_size: 16.0,
                color: Color::rgba(1.0, 1.0, 1.0, 0.7),
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        }),
    );
}

fn update_fps_text(
    time: Res<Time>,
    diagnostics: Res<bevy::diagnostic::DiagnosticsStore>,
    mut query: Query<&mut Text, With<FpsText>>,
) {
    for mut text in query.iter_mut() {
        if let Some(fps) = diagnostics.get(&bevy::diagnostic::FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                text.sections[0].value = format!("FPS: {:.1}", value);
            }
        }
    }
}

fn update_stats_text(
    creatures: Query<&crate::components::Creature>,
    resources: Query<&crate::components::ResourceMarker>,
    mut query: Query<&mut Text, With<StatsText>>,
) {
    for mut text in query.iter_mut() {
        let creature_count = creatures.iter().count();
        let resource_count = resources.iter().count();
        text.sections[0].value = format!(
            "Creatures: {} | Resources: {}",
            creature_count, resource_count
        );
    }
}
