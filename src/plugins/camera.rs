use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraState>()
            .add_systems(Startup, setup_camera)
            .add_systems(
                Update,
                (
                    camera_movement,
                    camera_zoom,
                    camera_mouse_pan,
                    camera_follow,
                    handle_camera_input,
                )
                    .chain(),
            );
    }
}

#[derive(Resource, Default)]
pub struct CameraState {
    pub follow_entity: Option<Entity>,
    pub is_dragging: bool,
}

#[derive(Component)]
pub struct MainCamera {
    pub move_speed: f32,
    pub zoom_speed: f32,
    pub min_zoom: f32,
    pub max_zoom: f32,
    pub follow_smoothness: f32,
}

impl Default for MainCamera {
    fn default() -> Self {
        Self {
            move_speed: 500.0,
            zoom_speed: 0.1,
            min_zoom: 0.5,
            max_zoom: 5.0,
            follow_smoothness: 5.0,
        }
    }
}

fn setup_camera(mut commands: Commands) {
    // Spawn 2D camera with isometric-friendly settings
    commands.spawn((
        Camera2dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 100.0),
            ..default()
        },
        MainCamera::default(),
        Name::new("Main Camera"),
    ));
}

fn camera_movement(
    time: Res<Time>,
    keyboard: Option<Res<ButtonInput<KeyCode>>>,
    mut camera_query: Query<(&mut Transform, &MainCamera), With<Camera>>,
) {
    // Skip if no keyboard input available (e.g., in tests)
    let Some(keyboard) = keyboard else { return };
    let Ok((mut transform, camera)) = camera_query.get_single_mut() else {
        return;
    };

    let mut direction = Vec3::ZERO;

    // WASD movement
    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }

    if direction.length() > 0.0 {
        direction = direction.normalize();
        transform.translation += direction * camera.move_speed * time.delta_seconds();
    }
}

fn camera_zoom(
    keyboard: Option<Res<ButtonInput<KeyCode>>>,
    mut mouse_wheel: EventReader<MouseWheel>,
    mut camera_query: Query<(&mut OrthographicProjection, &MainCamera), With<Camera>>,
) {
    let Ok((mut projection, camera)) = camera_query.get_single_mut() else {
        return;
    };

    let mut zoom_delta = 0.0;

    // Q/E for zoom
    if let Some(keyboard) = keyboard {
        if keyboard.pressed(KeyCode::KeyQ) {
            zoom_delta -= camera.zoom_speed;
        }
        if keyboard.pressed(KeyCode::KeyE) {
            zoom_delta += camera.zoom_speed;
        }
    }

    // Mouse wheel zoom
    for event in mouse_wheel.read() {
        zoom_delta -= event.y * camera.zoom_speed * 0.5;
    }

    if zoom_delta != 0.0 {
        projection.scale = (projection.scale + zoom_delta).clamp(camera.min_zoom, camera.max_zoom);
    }
}

fn camera_mouse_pan(
    mouse_button: Option<Res<ButtonInput<MouseButton>>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut camera_state: ResMut<CameraState>,
    mut camera_query: Query<(&mut Transform, &OrthographicProjection), With<MainCamera>>,
) {
    let Ok((mut transform, projection)) = camera_query.get_single_mut() else {
        return;
    };

    // Skip if no mouse input available (e.g., in tests)
    if let Some(mouse_button) = mouse_button {
        // Middle mouse button or right mouse button for panning
        if mouse_button.just_pressed(MouseButton::Middle)
            || mouse_button.just_pressed(MouseButton::Right)
        {
            camera_state.is_dragging = true;
            camera_state.follow_entity = None; // Stop following when manually panning
        }

        if mouse_button.just_released(MouseButton::Middle)
            || mouse_button.just_released(MouseButton::Right)
        {
            camera_state.is_dragging = false;
        }
    }

    if camera_state.is_dragging {
        for event in mouse_motion.read() {
            // Scale movement by zoom level
            let pan_speed = projection.scale;
            transform.translation.x -= event.delta.x * pan_speed;
            transform.translation.y += event.delta.y * pan_speed;
        }
    }
}

fn camera_follow(
    time: Res<Time>,
    camera_state: Res<CameraState>,
    mut camera_query: Query<(&mut Transform, &MainCamera), Without<crate::components::Creature>>,
    creature_query: Query<&crate::components::Position, With<crate::components::Creature>>,
) {
    let Some(follow_entity) = camera_state.follow_entity else {
        return;
    };

    let Ok((mut camera_transform, camera)) = camera_query.get_single_mut() else {
        return;
    };

    let Ok(target_pos) = creature_query.get(follow_entity) else {
        return;
    };

    // Smooth follow using lerp
    let target = Vec3::new(
        target_pos.0.x,
        target_pos.0.y,
        camera_transform.translation.z,
    );
    camera_transform.translation = camera_transform
        .translation
        .lerp(target, camera.follow_smoothness * time.delta_seconds());
}

fn handle_camera_input(
    keyboard: Option<Res<ButtonInput<KeyCode>>>,
    mut camera_state: ResMut<CameraState>,
) {
    // Skip if no keyboard input available (e.g., in tests)
    let Some(keyboard) = keyboard else { return };

    // ESC to stop following
    if keyboard.just_pressed(KeyCode::Escape) {
        camera_state.follow_entity = None;
    }
}
