//! Selection system for clicking on creatures

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::ecs::system::SystemParam;

pub struct SelectionPlugin;

// Bundle related queries together to reduce function parameters
#[derive(SystemParam)]
struct SelectionQueries<'w, 's> {
    windows: Query<'w, 's, &'static Window, With<PrimaryWindow>>,
    camera_query: Query<'w, 's, (&'static Camera, &'static GlobalTransform), With<crate::plugins::camera::MainCamera>>,
    creatures: Query<'w, 's, (Entity, &'static crate::components::Position), With<crate::components::Creature>>,
    selected_query: Query<'w, 's, Entity, With<Selected>>,
}

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (handle_mouse_clicks, update_selection_visuals));
    }
}

#[derive(Component)]
pub struct Selected;

fn handle_mouse_clicks(
    mouse_button: Option<Res<ButtonInput<MouseButton>>>,
    queries: SelectionQueries,
    mut ui_state: ResMut<crate::plugins::ui_egui::UiState>,
    mut camera_state: ResMut<crate::plugins::camera::CameraState>,
    mut commands: Commands,
) {
    // Skip if no mouse input available (e.g., in tests)
    let Some(mouse_button) = mouse_button else {
        return;
    };

    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = queries.windows.get_single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let Ok((camera, camera_transform)) = queries.camera_query.get_single() else {
        return;
    };

    // Convert screen coordinates to world coordinates
    let Some(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position)
    else {
        return;
    };

    // Clear previous selection
    for entity in queries.selected_query.iter() {
        commands.entity(entity).remove::<Selected>();
    }

    // Find closest creature within selection radius
    let selection_radius = 30.0;
    let mut closest_creature = None;
    let mut closest_distance = f32::MAX;

    for (entity, position) in queries.creatures.iter() {
        let distance = position.0.distance(world_position);
        if distance < selection_radius && distance < closest_distance {
            closest_distance = distance;
            closest_creature = Some(entity);
        }
    }

    if let Some(entity) = closest_creature {
        // Select the creature
        commands.entity(entity).insert(Selected);
        ui_state.selected_creature = Some(entity);

        // If shift is held, also follow the creature
        if queries.windows.single().cursor.grab_mode == bevy::window::CursorGrabMode::None {
            camera_state.follow_entity = Some(entity);
        }
    } else {
        // Clicked on empty space - deselect
        ui_state.selected_creature = None;
        camera_state.follow_entity = None;
    }
}

fn update_selection_visuals(
    selected_creatures: Query<Entity, With<Selected>>,
    mut gizmos: Gizmos,
    positions: Query<&crate::components::Position>,
) {
    for entity in selected_creatures.iter() {
        if let Ok(position) = positions.get(entity) {
            // Draw selection circle
            gizmos.circle_2d(position.0, 25.0, Color::YELLOW);
        }
    }
}
