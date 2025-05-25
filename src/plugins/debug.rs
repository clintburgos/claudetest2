//! Debug plugin for development and testing

use crate::components::*;
use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};

/// Debug plugin with diagnostics and visualization
pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin)
            .init_resource::<DebugSettings>()
            .add_systems(
                Update,
                (
                    debug_creature_count,
                    debug_resource_count,
                    debug_overlay_system,
                    toggle_debug_visualization,
                ),
            );
    }
}

#[derive(Resource, Default)]
pub struct DebugSettings {
    pub show_fps: bool,
    pub show_entity_ids: bool,
    pub show_creature_states: bool,
    pub show_spatial_grid: bool,
}

fn debug_creature_count(
    query: Query<Entity, With<Creature>>,
    time: Res<Time>,
    mut timer: Local<Timer>,
) {
    if timer.duration() == std::time::Duration::ZERO {
        *timer = Timer::from_seconds(1.0, TimerMode::Repeating);
    }

    timer.tick(time.delta());
    if timer.just_finished() {
        let count = query.iter().count();
        info!("Active creatures: {}", count);
    }
}

fn debug_resource_count(
    query: Query<Entity, With<ResourceMarker>>,
    time: Res<Time>,
    mut timer: Local<Timer>,
) {
    if timer.duration() == std::time::Duration::ZERO {
        *timer = Timer::from_seconds(5.0, TimerMode::Repeating);
    }

    timer.tick(time.delta());
    if timer.just_finished() {
        let count = query.iter().count();
        info!("Active resources: {}", count);
    }
}

fn debug_overlay_system(
    mut commands: Commands,
    debug_settings: Res<DebugSettings>,
    creature_query: Query<(Entity, &Position, Option<&DebugVisualization>), With<Creature>>,
    mut gizmos: Gizmos,
) {
    if !debug_settings.show_entity_ids && !debug_settings.show_creature_states {
        return;
    }

    // Draw debug info for creatures
    for (entity, position, debug_viz) in creature_query.iter() {
        if debug_settings.show_entity_ids {
            // Draw entity ID above creature
            gizmos.line_2d(position.0, position.0 + Vec2::Y * 30.0, Color::WHITE);
        }

        // Add or update debug visualization component based on settings
        if debug_viz.is_none()
            && (debug_settings.show_entity_ids || debug_settings.show_creature_states)
        {
            commands.entity(entity).insert(DebugVisualization::default());
        }
    }

    // Draw spatial grid if enabled
    if debug_settings.show_spatial_grid {
        let grid_size = 50.0; // Match spatial grid cell size
        let grid_range = 10; // Number of cells to draw in each direction

        for x in -grid_range..=grid_range {
            for y in -grid_range..=grid_range {
                let pos = Vec2::new(x as f32 * grid_size, y as f32 * grid_size);
                gizmos.rect_2d(
                    pos,
                    0.0,
                    Vec2::splat(grid_size),
                    Color::rgba(0.5, 0.5, 0.5, 0.2),
                );
            }
        }
    }
}

fn toggle_debug_visualization(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut debug_settings: ResMut<DebugSettings>,
) {
    // F1: Toggle FPS display
    if keyboard.just_pressed(KeyCode::F1) {
        debug_settings.show_fps = !debug_settings.show_fps;
        info!(
            "FPS display: {}",
            if debug_settings.show_fps { "ON" } else { "OFF" }
        );
    }

    // F2: Toggle entity IDs
    if keyboard.just_pressed(KeyCode::F2) {
        debug_settings.show_entity_ids = !debug_settings.show_entity_ids;
        info!(
            "Entity IDs: {}",
            if debug_settings.show_entity_ids { "ON" } else { "OFF" }
        );
    }

    // F3: Toggle creature states
    if keyboard.just_pressed(KeyCode::F3) {
        debug_settings.show_creature_states = !debug_settings.show_creature_states;
        info!(
            "Creature states: {}",
            if debug_settings.show_creature_states { "ON" } else { "OFF" }
        );
    }

    // F4: Toggle spatial grid
    if keyboard.just_pressed(KeyCode::F4) {
        debug_settings.show_spatial_grid = !debug_settings.show_spatial_grid;
        info!(
            "Spatial grid: {}",
            if debug_settings.show_spatial_grid { "ON" } else { "OFF" }
        );
    }
}
