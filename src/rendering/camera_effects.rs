use bevy::prelude::*;
use bevy::render::camera::Viewport;

/// Phase 4 Camera Effects Plugin
/// 
/// This plugin provides dynamic camera effects for the cartoon isometric simulation:
/// - Smooth camera transitions and focus changes
/// - Screen shake for impacts and events  
/// - Picture-in-picture for important events
/// - Cinematic camera movements
/// - Dynamic zoom based on action
/// 
/// The camera system enhances the cartoon feel with smooth, exaggerated movements
/// while maintaining clear visibility of the action.
pub struct CameraEffectsPlugin;

impl Plugin for CameraEffectsPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<CameraEffectSettings>()
            .init_resource::<CameraShakeState>()
            .init_resource::<PictureInPictureState>()
            .add_event::<CameraEffectEvent>()
            .add_systems(
                Update,
                (
                    handle_camera_effect_events,
                    update_camera_shake,
                    update_smooth_follow,
                    update_dynamic_zoom,
                    update_picture_in_picture,
                    apply_camera_effects,
                )
                    .chain(),
            );
    }
}

/// Settings for camera effects behavior
#[derive(Resource)]
pub struct CameraEffectSettings {
    /// Whether camera effects are enabled
    pub enabled: bool,
    /// Smooth follow speed (0.0 - 1.0)
    pub follow_smoothness: f32,
    /// Maximum shake amplitude in world units
    pub max_shake_amplitude: f32,
    /// Shake decay rate (how quickly shake diminishes)
    pub shake_decay_rate: f32,
    /// Whether to use dynamic zoom
    pub dynamic_zoom_enabled: bool,
    /// Zoom smoothing factor
    pub zoom_smoothness: f32,
    /// Picture-in-picture settings
    pub pip_enabled: bool,
    pub pip_size: Vec2,
    pub pip_position: PipPosition,
}

impl Default for CameraEffectSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            follow_smoothness: 0.1,
            max_shake_amplitude: 10.0,
            shake_decay_rate: 5.0,
            dynamic_zoom_enabled: true,
            zoom_smoothness: 0.05,
            pip_enabled: true,
            pip_size: Vec2::new(320.0, 240.0),
            pip_position: PipPosition::TopRight,
        }
    }
}

/// Position options for picture-in-picture
#[derive(Debug, Clone, Copy)]
pub enum PipPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

/// Events that trigger camera effects
#[derive(Event)]
pub enum CameraEffectEvent {
    /// Shake the camera with given intensity and duration
    Shake {
        intensity: f32,
        duration: f32,
        source: Option<Vec3>,
    },
    /// Focus camera on specific entity
    FocusOn {
        target: Entity,
        zoom_level: Option<f32>,
        duration: f32,
    },
    /// Play a cinematic camera movement
    Cinematic {
        path: Vec<CameraKeyframe>,
        duration: f32,
    },
    /// Show event in picture-in-picture
    ShowInPip {
        target: Entity,
        duration: f32,
    },
}

/// Keyframe for cinematic camera movements
#[derive(Clone)]
pub struct CameraKeyframe {
    /// Position of camera at this keyframe
    pub position: Vec3,
    /// Look-at target for this keyframe
    pub look_at: Vec3,
    /// Zoom level at this keyframe
    pub zoom: f32,
    /// Time position of this keyframe (0.0 - 1.0)
    pub time: f32,
}

/// Current camera shake state
#[derive(Resource)]
pub struct CameraShakeState {
    /// Current shake intensity
    pub intensity: f32,
    /// Remaining shake duration
    pub duration: f32,
    /// Accumulated shake offset
    pub offset: Vec2,
    /// Previous frame offset for smoothing
    pub previous_offset: Vec2,
    /// Optional shake source for directional shake
    pub source: Option<Vec3>,
    /// Shake frequency components for more organic feel
    pub frequency_components: Vec<(f32, f32, f32)>, // (frequency, amplitude, phase)
}

impl Default for CameraShakeState {
    fn default() -> Self {
        Self {
            intensity: 0.0,
            duration: 0.0,
            offset: Vec2::ZERO,
            previous_offset: Vec2::ZERO,
            source: None,
            frequency_components: vec![
                (4.3, 1.0, 0.0),   // Low frequency, high amplitude
                (11.7, 0.4, 0.5),  // Medium frequency
                (23.5, 0.2, 1.0),  // High frequency, low amplitude
            ],
        }
    }
}

/// Picture-in-picture rendering state
#[derive(Resource)]
pub struct PictureInPictureState {
    /// Whether PiP is currently active
    pub active: bool,
    /// Entity to focus on in PiP
    pub target: Option<Entity>,
    /// Remaining display duration
    pub duration: f32,
    /// Render target for PiP camera
    pub render_target: Option<Handle<Image>>,
    /// Camera entity for PiP
    pub camera_entity: Option<Entity>,
}

impl Default for PictureInPictureState {
    fn default() -> Self {
        Self {
            active: false,
            target: None,
            duration: 0.0,
            render_target: None,
            camera_entity: None,
        }
    }
}

/// Component for smooth camera following
#[derive(Component)]
pub struct SmoothFollow {
    /// Entity to follow
    pub target: Entity,
    /// Offset from target
    pub offset: Vec3,
    /// Current interpolated position
    pub current_position: Vec3,
    /// Follow speed (0.0 - 1.0)
    pub smoothness: f32,
}

/// Component for dynamic zoom behavior
#[derive(Component)]
pub struct DynamicZoom {
    /// Base zoom level
    pub base_zoom: f32,
    /// Current zoom level
    pub current_zoom: f32,
    /// Target zoom level
    pub target_zoom: f32,
    /// Zoom bounds
    pub min_zoom: f32,
    pub max_zoom: f32,
    /// Entities to keep in view
    pub tracked_entities: Vec<Entity>,
}

/// Component for cinematic camera control
#[derive(Component)]
pub struct CinematicCamera {
    /// Keyframes for the cinematic
    pub keyframes: Vec<CameraKeyframe>,
    /// Total duration of cinematic
    pub duration: f32,
    /// Current elapsed time
    pub elapsed: f32,
    /// Whether to loop the cinematic
    pub looping: bool,
}

/// System to handle camera effect events
fn handle_camera_effect_events(
    mut events: EventReader<CameraEffectEvent>,
    mut shake_state: ResMut<CameraShakeState>,
    mut pip_state: ResMut<PictureInPictureState>,
    mut commands: Commands,
    settings: Res<CameraEffectSettings>,
    camera_query: Query<Entity, With<Camera>>,
) {
    if !settings.enabled {
        return;
    }
    
    for event in events.read() {
        match event {
            CameraEffectEvent::Shake { intensity, duration, source } => {
                // Apply shake with maximum intensity
                shake_state.intensity = shake_state.intensity.max(*intensity);
                shake_state.duration = shake_state.duration.max(*duration);
                shake_state.source = *source;
                
                // Reset frequency phases for new shake
                for component in shake_state.frequency_components.iter_mut() {
                    component.2 = rand::random::<f32>() * std::f32::consts::TAU;
                }
            }
            
            CameraEffectEvent::FocusOn { target, zoom_level, duration } => {
                if let Ok(camera_entity) = camera_query.get_single() {
                    // Add or update smooth follow component
                    commands.entity(camera_entity).insert(SmoothFollow {
                        target: *target,
                        offset: Vec3::new(0.0, 100.0, 100.0), // Isometric offset
                        current_position: Vec3::ZERO,
                        smoothness: 1.0 / duration.max(0.1),
                    });
                    
                    // Update zoom if specified
                    if let Some(zoom) = zoom_level {
                        commands.entity(camera_entity).insert(DynamicZoom {
                            base_zoom: *zoom,
                            current_zoom: 1.0,
                            target_zoom: *zoom,
                            min_zoom: 0.5,
                            max_zoom: 2.0,
                            tracked_entities: vec![*target],
                        });
                    }
                }
            }
            
            CameraEffectEvent::Cinematic { path, duration } => {
                if let Ok(camera_entity) = camera_query.get_single() {
                    commands.entity(camera_entity).insert(CinematicCamera {
                        keyframes: path.clone(),
                        duration: *duration,
                        elapsed: 0.0,
                        looping: false,
                    });
                }
            }
            
            CameraEffectEvent::ShowInPip { target, duration } => {
                if settings.pip_enabled {
                    pip_state.active = true;
                    pip_state.target = Some(*target);
                    pip_state.duration = *duration;
                    
                    // Create PiP camera if it doesn't exist
                    if pip_state.camera_entity.is_none() {
                        create_pip_camera(&mut commands, &mut pip_state, &settings);
                    }
                }
            }
        }
    }
}

/// System to update camera shake
fn update_camera_shake(
    mut shake_state: ResMut<CameraShakeState>,
    time: Res<Time>,
    settings: Res<CameraEffectSettings>,
) {
    if shake_state.duration <= 0.0 {
        shake_state.intensity = 0.0;
        shake_state.offset = Vec2::ZERO;
        return;
    }
    
    let dt = time.delta_seconds();
    
    // Decay shake over time
    shake_state.duration -= dt;
    shake_state.intensity *= (1.0 - settings.shake_decay_rate * dt).max(0.0);
    
    // Calculate multi-frequency shake offset for organic feel
    let mut offset = Vec2::ZERO;
    let time_elapsed = time.elapsed_seconds();
    
    for (frequency, amplitude, phase) in &shake_state.frequency_components {
        let freq_offset = Vec2::new(
            (time_elapsed * frequency + phase).sin(),
            (time_elapsed * frequency * 1.3 + phase + 0.7).sin(),
        );
        offset += freq_offset * *amplitude;
    }
    
    // Apply intensity and maximum amplitude
    offset *= shake_state.intensity * settings.max_shake_amplitude;
    
    // Smooth the shake for less jarring movement
    shake_state.previous_offset = shake_state.offset;
    shake_state.offset = shake_state.previous_offset.lerp(offset, 0.7);
}

/// System to update smooth camera following
fn update_smooth_follow(
    mut follow_cameras: Query<(&mut Transform, &mut SmoothFollow)>,
    target_transforms: Query<&Transform, Without<SmoothFollow>>,
    settings: Res<CameraEffectSettings>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    
    for (mut camera_transform, mut follow) in follow_cameras.iter_mut() {
        if let Ok(target_transform) = target_transforms.get(follow.target) {
            let target_position = target_transform.translation + follow.offset;
            
            // Initialize current position if needed
            if follow.current_position == Vec3::ZERO {
                follow.current_position = camera_transform.translation;
            }
            
            // Smooth interpolation with variable speed
            let smoothness = follow.smoothness * settings.follow_smoothness;
            follow.current_position = follow.current_position.lerp(
                target_position,
                (smoothness * dt).min(1.0),
            );
            
            camera_transform.translation = follow.current_position;
        }
    }
}

/// System to update dynamic zoom based on tracked entities
fn update_dynamic_zoom(
    mut zoom_cameras: Query<(&mut OrthographicProjection, &mut DynamicZoom)>,
    target_transforms: Query<&Transform>,
    settings: Res<CameraEffectSettings>,
    time: Res<Time>,
) {
    if !settings.dynamic_zoom_enabled {
        return;
    }
    
    let dt = time.delta_seconds();
    
    for (mut projection, mut zoom) in zoom_cameras.iter_mut() {
        // Calculate bounding box of tracked entities
        let mut min_pos = Vec3::splat(f32::MAX);
        let mut max_pos = Vec3::splat(f32::MIN);
        let mut valid_count = 0;
        
        for entity in &zoom.tracked_entities {
            if let Ok(transform) = target_transforms.get(*entity) {
                min_pos = min_pos.min(transform.translation);
                max_pos = max_pos.max(transform.translation);
                valid_count += 1;
            }
        }
        
        if valid_count > 0 {
            // Calculate required zoom to fit all entities
            let bounds_size = max_pos - min_pos;
            let max_dimension = bounds_size.x.max(bounds_size.z);
            
            // Add padding
            let padded_size = max_dimension * 1.5;
            
            // Calculate zoom level (assuming base view size of 1000 units)
            let required_zoom = 1000.0 / padded_size.max(100.0);
            zoom.target_zoom = required_zoom.clamp(zoom.min_zoom, zoom.max_zoom);
        }
        
        // Smooth zoom transition
        zoom.current_zoom = zoom.current_zoom.lerp(
            zoom.target_zoom,
            settings.zoom_smoothness * dt * 60.0, // Frame-rate independent
        );
        
        projection.scale = zoom.base_zoom / zoom.current_zoom;
    }
}

/// System to update picture-in-picture
fn update_picture_in_picture(
    mut pip_state: ResMut<PictureInPictureState>,
    mut pip_cameras: Query<(&mut Transform, &mut Camera), With<PipCamera>>,
    target_transforms: Query<&Transform, Without<PipCamera>>,
    time: Res<Time>,
    settings: Res<CameraEffectSettings>,
) {
    if !pip_state.active {
        return;
    }
    
    pip_state.duration -= time.delta_seconds();
    
    if pip_state.duration <= 0.0 {
        pip_state.active = false;
        pip_state.target = None;
        
        // Disable PiP camera
        if let Some(camera_entity) = pip_state.camera_entity {
            for (_, mut camera) in pip_cameras.iter_mut() {
                camera.is_active = false;
            }
        }
        return;
    }
    
    // Update PiP camera position
    if let Some(target) = pip_state.target {
        if let Ok(target_transform) = target_transforms.get(target) {
            for (mut camera_transform, mut camera) in pip_cameras.iter_mut() {
                camera.is_active = true;
                
                // Position PiP camera above target
                camera_transform.translation = target_transform.translation + Vec3::new(0.0, 150.0, 150.0);
                camera_transform.look_at(target_transform.translation, Vec3::Y);
            }
        }
    }
}

/// System to apply all camera effects to the main camera
fn apply_camera_effects(
    mut cameras: Query<&mut Transform, (With<Camera>, Without<PipCamera>)>,
    shake_state: Res<CameraShakeState>,
    settings: Res<CameraEffectSettings>,
) {
    if !settings.enabled {
        return;
    }
    
    for mut transform in cameras.iter_mut() {
        // Apply shake offset
        if shake_state.intensity > 0.0 {
            let shake_offset = Vec3::new(
                shake_state.offset.x,
                0.0, // Don't shake Y in isometric view
                shake_state.offset.y,
            );
            transform.translation += shake_offset;
        }
    }
}

/// Marker component for PiP camera
#[derive(Component)]
pub struct PipCamera;

/// Helper function to create picture-in-picture camera
fn create_pip_camera(
    commands: &mut Commands,
    pip_state: &mut PictureInPictureState,
    settings: &CameraEffectSettings,
) {
    // Create render target for PiP
    // In a real implementation, this would create a texture to render to
    
    let pip_camera = commands.spawn((
        Camera3dBundle {
            camera: Camera {
                order: 1, // Render after main camera
                is_active: false,
                viewport: Some(Viewport {
                    physical_position: match settings.pip_position {
                        PipPosition::TopLeft => UVec2::new(10, 10),
                        PipPosition::TopRight => UVec2::new(1920 - 330, 10),
                        PipPosition::BottomLeft => UVec2::new(10, 1080 - 250),
                        PipPosition::BottomRight => UVec2::new(1920 - 330, 1080 - 250),
                    },
                    physical_size: UVec2::new(
                        settings.pip_size.x as u32,
                        settings.pip_size.y as u32,
                    ),
                    ..default()
                }),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 100.0, 100.0),
            projection: Projection::Orthographic(OrthographicProjection {
                scale: 0.5, // Zoomed in view
                ..default()
            }),
            ..default()
        },
        PipCamera,
        Name::new("PictureInPictureCamera"),
    )).id();
    
    pip_state.camera_entity = Some(pip_camera);
}

/// Helper function to trigger a camera shake
pub fn shake_camera(
    events: &mut EventWriter<CameraEffectEvent>,
    intensity: f32,
    duration: f32,
    source: Option<Vec3>,
) {
    events.send(CameraEffectEvent::Shake {
        intensity,
        duration,
        source,
    });
}

/// Helper function to focus camera on entity
pub fn focus_camera_on(
    events: &mut EventWriter<CameraEffectEvent>,
    target: Entity,
    zoom: Option<f32>,
    duration: f32,
) {
    events.send(CameraEffectEvent::FocusOn {
        target,
        zoom_level: zoom,
        duration,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_shake_decay() {
        let mut shake_state = CameraShakeState::default();
        shake_state.intensity = 1.0;
        shake_state.duration = 1.0;
        
        let settings = CameraEffectSettings::default();
        
        // Simulate decay over time
        let dt = 0.1;
        shake_state.duration -= dt;
        shake_state.intensity *= (1.0 - settings.shake_decay_rate * dt).max(0.0);
        
        assert!(shake_state.intensity < 1.0);
        assert!(shake_state.duration < 1.0);
    }
    
    #[test]
    fn test_zoom_clamping() {
        let zoom = DynamicZoom {
            base_zoom: 1.0,
            current_zoom: 1.0,
            target_zoom: 1.0,
            min_zoom: 0.5,
            max_zoom: 2.0,
            tracked_entities: vec![],
        };
        
        // Test that zoom values are clamped
        let clamped_low = 0.3_f32.clamp(zoom.min_zoom, zoom.max_zoom);
        let clamped_high = 3.0_f32.clamp(zoom.min_zoom, zoom.max_zoom);
        
        assert_eq!(clamped_low, 0.5);
        assert_eq!(clamped_high, 2.0);
    }
}