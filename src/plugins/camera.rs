use bevy::prelude::*;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera)
            .add_systems(Update, (camera_movement, camera_zoom).chain());
    }
}

#[derive(Component)]
pub struct MainCamera {
    pub move_speed: f32,
    pub zoom_speed: f32,
    pub min_zoom: f32,
    pub max_zoom: f32,
}

impl Default for MainCamera {
    fn default() -> Self {
        Self {
            move_speed: 500.0,
            zoom_speed: 0.1,
            min_zoom: 0.5,
            max_zoom: 5.0,
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
    keyboard: Res<ButtonInput<KeyCode>>,
    mut camera_query: Query<(&mut Transform, &MainCamera), With<Camera>>,
) {
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
    keyboard: Res<ButtonInput<KeyCode>>,
    mut camera_query: Query<(&mut OrthographicProjection, &MainCamera), With<Camera>>,
) {
    let Ok((mut projection, camera)) = camera_query.get_single_mut() else {
        return;
    };

    let mut zoom_delta = 0.0;

    // Q/E for zoom
    if keyboard.pressed(KeyCode::KeyQ) {
        zoom_delta -= camera.zoom_speed;
    }
    if keyboard.pressed(KeyCode::KeyE) {
        zoom_delta += camera.zoom_speed;
    }

    if zoom_delta != 0.0 {
        projection.scale = (projection.scale + zoom_delta).clamp(camera.min_zoom, camera.max_zoom);
    }
}