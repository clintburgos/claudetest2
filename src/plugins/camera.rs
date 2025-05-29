use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use crate::rendering::isometric::screen_to_world;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraState>()
            .init_resource::<MiniMapConfig>()
            .add_systems(Startup, (setup_camera, setup_minimap).chain())
            .add_systems(
                Update,
                (
                    camera_movement,
                    camera_edge_panning,
                    camera_zoom,
                    smooth_camera_zoom,
                    camera_mouse_pan,
                    camera_click_to_focus,
                    camera_follow,
                    handle_camera_input,
                    update_camera_bounds,
                    update_minimap,
                    toggle_minimap,
                )
                    .chain(),
            );
    }
}

#[derive(Resource)]
pub struct CameraState {
    pub follow_entity: Option<Entity>,
    pub is_dragging: bool,
    pub zoom: f32,
    pub edge_pan_active: bool,
    pub edge_pan_speed: f32,
    pub click_to_focus_enabled: bool,
    pub smooth_zoom_target: f32,
    pub smooth_zoom_speed: f32,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            follow_entity: None,
            is_dragging: false,
            zoom: 1.0,
            edge_pan_active: true,
            edge_pan_speed: 300.0,
            click_to_focus_enabled: true,
            smooth_zoom_target: 1.0,
            smooth_zoom_speed: 5.0,
        }
    }
}

/// Resource for mini-map configuration
#[derive(Resource)]
pub struct MiniMapConfig {
    pub enabled: bool,
    pub size: Vec2,
    pub zoom_level: f32,
    pub entity_scale: f32,
}

impl Default for MiniMapConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            size: Vec2::new(200.0, 200.0),
            zoom_level: 0.1, // Show 10x more area than main view
            entity_scale: 2.0, // Make entities bigger on minimap
        }
    }
}


#[derive(Component)]
pub struct MainCamera {
    pub move_speed: f32,
    pub zoom_speed: f32,
    pub min_zoom: f32,
    pub max_zoom: f32,
    pub follow_smoothness: f32,
    pub edge_pan_margin: f32,
    pub edge_pan_acceleration: f32,
    pub click_focus_speed: f32,
}

impl Default for MainCamera {
    fn default() -> Self {
        Self {
            move_speed: 500.0,
            zoom_speed: 0.1,
            min_zoom: 0.25,  // Enhanced zoom range for Phase 2
            max_zoom: 4.0,
            follow_smoothness: 5.0,
            edge_pan_margin: 50.0,  // Pixels from screen edge to trigger pan
            edge_pan_acceleration: 2.0,  // Speed multiplier at screen edge
            click_focus_speed: 8.0,  // Smoothness for click-to-focus
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
    mut camera_state: ResMut<CameraState>,
    mut camera_query: Query<(&mut OrthographicProjection, &MainCamera), With<Camera>>,
) {
    let Ok((_projection, camera)) = camera_query.get_single_mut() else {
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
        // Update smooth zoom target instead of directly changing scale
        camera_state.smooth_zoom_target = (camera_state.smooth_zoom_target + zoom_delta)
            .clamp(camera.min_zoom, camera.max_zoom);
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
    
    // Toggle edge panning with P
    if keyboard.just_pressed(KeyCode::KeyP) {
        camera_state.edge_pan_active = !camera_state.edge_pan_active;
        info!("Edge panning: {}", if camera_state.edge_pan_active { "enabled" } else { "disabled" });
    }
}

/// System for edge panning - moves camera when cursor is near screen edges
/// 
/// # Edge Pan Mechanics
/// 
/// When the cursor is within `edge_pan_margin` pixels of the screen edge:
/// - Camera starts panning in that direction
/// - Speed increases closer to edge (acceleration factor)
/// - Multiple edges can be active (diagonal panning)
/// 
/// # Performance
/// 
/// Only active when edge_pan_active is true and no entity is being followed
fn camera_edge_panning(
    time: Res<Time>,
    windows: Query<&Window>,
    camera_state: Res<CameraState>,
    mut camera_query: Query<(&mut Transform, &MainCamera), With<Camera>>,
) {
    // Skip if edge panning disabled or following entity
    if !camera_state.edge_pan_active || camera_state.follow_entity.is_some() {
        return;
    }
    
    let Ok(window) = windows.get_single() else { return };
    let Ok((mut transform, camera)) = camera_query.get_single_mut() else { return };
    
    // Get cursor position
    let Some(cursor_pos) = window.cursor_position() else { return };
    
    let mut pan_direction = Vec2::ZERO;
    let margin = camera.edge_pan_margin;
    
    // Calculate pan direction based on cursor proximity to edges
    // Left edge
    if cursor_pos.x < margin {
        let strength = 1.0 - (cursor_pos.x / margin);
        pan_direction.x -= strength;
    }
    // Right edge
    if cursor_pos.x > window.width() - margin {
        let strength = (cursor_pos.x - (window.width() - margin)) / margin;
        pan_direction.x += strength;
    }
    // Top edge (remember Y is inverted in screen space)
    if cursor_pos.y < margin {
        let strength = 1.0 - (cursor_pos.y / margin);
        pan_direction.y += strength;
    }
    // Bottom edge
    if cursor_pos.y > window.height() - margin {
        let strength = (cursor_pos.y - (window.height() - margin)) / margin;
        pan_direction.y -= strength;
    }
    
    // Apply panning with acceleration
    if pan_direction.length_squared() > 0.0 {
        pan_direction = pan_direction.normalize();
        let speed = camera_state.edge_pan_speed * camera.edge_pan_acceleration;
        transform.translation.x += pan_direction.x * speed * time.delta_seconds();
        transform.translation.y += pan_direction.y * speed * time.delta_seconds();
    }
}

/// Smooth zoom interpolation for better feel
/// 
/// Instead of instant zoom changes, smoothly interpolates to target zoom level
fn smooth_camera_zoom(
    time: Res<Time>,
    mut camera_state: ResMut<CameraState>,
    mut camera_query: Query<&mut OrthographicProjection, With<MainCamera>>,
) {
    let Ok(mut projection) = camera_query.get_single_mut() else { return };
    
    // Smoothly interpolate zoom
    let current = projection.scale;
    let target = camera_state.smooth_zoom_target;
    
    if (current - target).abs() > 0.001 {
        let new_zoom = current + (target - current) * camera_state.smooth_zoom_speed * time.delta_seconds();
        projection.scale = new_zoom;
        camera_state.zoom = new_zoom;
    }
}

/// Click-to-focus camera movement
/// 
/// Left-click on the world to smoothly move camera to that position
fn camera_click_to_focus(
    mouse_button: Option<Res<ButtonInput<MouseButton>>>,
    windows: Query<&Window>,
    camera_state: Res<CameraState>,
    mut camera_query: Query<(&mut Transform, &OrthographicProjection, &MainCamera), With<Camera>>,
) {
    // Skip if not enabled
    if !camera_state.click_to_focus_enabled {
        return;
    }
    
    let Some(mouse_button) = mouse_button else { return };
    let Ok(window) = windows.get_single() else { return };
    let Ok((mut transform, projection, camera)) = camera_query.get_single_mut() else { return };
    
    // Left click to focus
    if mouse_button.just_pressed(MouseButton::Left) && !camera_state.is_dragging {
        if let Some(cursor_pos) = window.cursor_position() {
            // Convert screen position to world position
            let screen_pos = Vec2::new(
                cursor_pos.x - window.width() / 2.0,
                window.height() / 2.0 - cursor_pos.y,
            );
            
            let camera_offset = Vec2::new(transform.translation.x, transform.translation.y);
            let world_pos = screen_to_world(screen_pos, camera_offset, projection.scale);
            
            // Set target position (smooth movement handled by camera follow-like logic)
            // For now, just move directly - could enhance with target position in CameraState
            let target = Vec3::new(world_pos.x, world_pos.z, transform.translation.z);
            transform.translation = transform.translation.lerp(target, camera.click_focus_speed * 0.1);
        }
    }
}

/// Update camera bounds for culling optimization
/// 
/// Calculates visible world bounds and stores them for other systems to use
fn update_camera_bounds(
    camera_query: Query<(&Transform, &OrthographicProjection), With<MainCamera>>,
    windows: Query<&Window>,
    mut commands: Commands,
) {
    let Ok((transform, projection)) = camera_query.get_single() else { return };
    let Ok(window) = windows.get_single() else { return };
    
    let viewport_size = Vec2::new(window.width(), window.height());
    let camera_pos = transform.translation;
    
    // Calculate and store visible bounds
    let (min_bounds, max_bounds) = crate::rendering::isometric::camera::calculate_visible_bounds(
        camera_pos,
        viewport_size,
        projection.scale,
    );
    
    // Store bounds as a resource for other systems
    commands.insert_resource(CameraVisibleBounds { min_bounds, max_bounds });
}

/// Resource storing current camera visible bounds
#[derive(Resource, Default)]
pub struct CameraVisibleBounds {
    pub min_bounds: Vec3,
    pub max_bounds: Vec3,
}

/// Setup mini-map camera and UI elements
fn setup_minimap(
    mut commands: Commands,
) {
    // Mini-map background quad
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgba(0.1, 0.1, 0.1, 0.6),
                custom_size: Some(Vec2::new(200.0, 200.0)),
                ..default()
            },
            transform: Transform::from_xyz(-700.0, 400.0, 900.0), // Top-left corner, high Z
            ..default()
        },
        Name::new("MiniMap Background"),
    ));
    
    // Mini-map border
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgba(0.2, 0.2, 0.2, 0.8),
                custom_size: Some(Vec2::new(204.0, 204.0)),
                ..default()
            },
            transform: Transform::from_xyz(-700.0, 400.0, 899.0), // Behind background
            ..default()
        },
        Name::new("MiniMap Border"),
    ));
}

/// Update mini-map display
fn update_minimap(
    config: Res<MiniMapConfig>,
    camera_query: Query<(&Transform, &OrthographicProjection), With<MainCamera>>,
    creature_query: Query<&crate::components::Position, With<crate::components::Creature>>,
    resource_query: Query<&crate::components::Position, With<crate::components::ResourceMarker>>,
    mut gizmos: Gizmos,
) {
    if !config.enabled {
        return;
    }
    
    let Ok((camera_transform, projection)) = camera_query.get_single() else { return };
    
    // Calculate mini-map viewport in screen space
    let minimap_center = Vec2::new(-700.0, 400.0); // Match background position
    let minimap_size = config.size;
    let world_to_minimap_scale = config.zoom_level / projection.scale;
    
    // Draw camera viewport indicator
    let viewport_size = Vec2::new(100.0, 100.0) / config.zoom_level;
    let camera_pos_on_minimap = Vec2::new(
        minimap_center.x + camera_transform.translation.x * world_to_minimap_scale,
        minimap_center.y + camera_transform.translation.y * world_to_minimap_scale,
    );
    
    // Draw viewport rectangle
    gizmos.rect_2d(
        camera_pos_on_minimap,
        0.0,
        viewport_size,
        Color::rgba(1.0, 1.0, 1.0, 0.3),
    );
    
    // Draw creatures on mini-map
    for pos in creature_query.iter() {
        let minimap_pos = Vec2::new(
            minimap_center.x + pos.0.x * world_to_minimap_scale,
            minimap_center.y + pos.0.y * world_to_minimap_scale,
        );
        
        // Only draw if within minimap bounds
        if minimap_pos.x.abs() < minimap_size.x / 2.0 && minimap_pos.y.abs() < minimap_size.y / 2.0 {
            gizmos.circle_2d(
                minimap_pos,
                config.entity_scale,
                Color::rgba(0.2, 0.8, 0.2, 0.8),
            );
        }
    }
    
    // Draw resources on mini-map
    for pos in resource_query.iter() {
        let minimap_pos = Vec2::new(
            minimap_center.x + pos.0.x * world_to_minimap_scale,
            minimap_center.y + pos.0.y * world_to_minimap_scale,
        );
        
        // Only draw if within minimap bounds
        if minimap_pos.x.abs() < minimap_size.x / 2.0 && minimap_pos.y.abs() < minimap_size.y / 2.0 {
            gizmos.circle_2d(
                minimap_pos,
                config.entity_scale * 0.7,
                Color::rgba(0.8, 0.6, 0.2, 0.8),
            );
        }
    }
}

/// Toggle mini-map visibility with M key
fn toggle_minimap(
    keyboard: Option<Res<ButtonInput<KeyCode>>>,
    mut config: ResMut<MiniMapConfig>,
) {
    let Some(keyboard) = keyboard else { return };
    
    if keyboard.just_pressed(KeyCode::KeyM) {
        config.enabled = !config.enabled;
        info!("Mini-map {}", if config.enabled { "enabled" } else { "disabled" });
    }
}
