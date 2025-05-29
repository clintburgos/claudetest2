use bevy::prelude::*;
use creature_simulation::rendering::camera_effects::*;

/// Test camera shake state and decay
#[test]
fn test_camera_shake_state() {
    let mut shake = CameraShakeState::default();
    
    // Initial state should be zero
    assert_eq!(shake.intensity, 0.0);
    assert_eq!(shake.duration, 0.0);
    assert_eq!(shake.offset, Vec2::ZERO);
    
    // Apply shake
    shake.intensity = 1.0;
    shake.duration = 1.0;
    
    // Test decay
    let settings = CameraEffectSettings::default();
    let dt = 0.1;
    
    shake.duration -= dt;
    shake.intensity *= (1.0 - settings.shake_decay_rate * dt).max(0.0);
    
    assert!(shake.duration < 1.0);
    assert!(shake.intensity < 1.0);
    assert!(shake.intensity > 0.0); // Should still have some shake
}

/// Test camera effect settings
#[test]
fn test_camera_effect_settings() {
    let settings = CameraEffectSettings::default();
    
    assert!(settings.enabled);
    assert_eq!(settings.follow_smoothness, 0.1);
    assert_eq!(settings.max_shake_amplitude, 10.0);
    assert_eq!(settings.shake_decay_rate, 5.0);
    assert!(settings.dynamic_zoom_enabled);
    assert_eq!(settings.zoom_smoothness, 0.05);
    assert!(settings.pip_enabled);
    assert_eq!(settings.pip_size, Vec2::new(320.0, 240.0));
}

/// Test picture-in-picture positions
#[test]
fn test_pip_positions() {
    let positions = vec![
        PipPosition::TopLeft,
        PipPosition::TopRight,
        PipPosition::BottomLeft,
        PipPosition::BottomRight,
    ];
    
    for pos in positions {
        let viewport_pos = match pos {
            PipPosition::TopLeft => UVec2::new(10, 10),
            PipPosition::TopRight => UVec2::new(1920 - 330, 10),
            PipPosition::BottomLeft => UVec2::new(10, 1080 - 250),
            PipPosition::BottomRight => UVec2::new(1920 - 330, 1080 - 250),
        };
        
        // Verify positions are within screen bounds
        assert!(viewport_pos.x < 1920);
        assert!(viewport_pos.y < 1080);
    }
}

/// Test smooth follow component
#[test]
fn test_smooth_follow() {
    let follow = SmoothFollow {
        target: Entity::from_raw(1),
        offset: Vec3::new(0.0, 100.0, 100.0),
        current_position: Vec3::ZERO,
        smoothness: 0.1,
    };
    
    assert_eq!(follow.offset, Vec3::new(0.0, 100.0, 100.0));
    assert_eq!(follow.current_position, Vec3::ZERO);
    assert_eq!(follow.smoothness, 0.1);
}

/// Test dynamic zoom calculations
#[test]
fn test_dynamic_zoom() {
    let mut zoom = DynamicZoom {
        base_zoom: 1.0,
        current_zoom: 1.0,
        target_zoom: 2.0,
        min_zoom: 0.5,
        max_zoom: 3.0,
        tracked_entities: vec![],
    };
    
    // Test zoom clamping
    zoom.target_zoom = 5.0;
    let clamped = zoom.target_zoom.clamp(zoom.min_zoom, zoom.max_zoom);
    assert_eq!(clamped, 3.0);
    
    zoom.target_zoom = 0.1;
    let clamped = zoom.target_zoom.clamp(zoom.min_zoom, zoom.max_zoom);
    assert_eq!(clamped, 0.5);
}

/// Test camera keyframe for cinematics
#[test]
fn test_camera_keyframe() {
    let keyframe = CameraKeyframe {
        position: Vec3::new(100.0, 50.0, 100.0),
        look_at: Vec3::ZERO,
        zoom: 1.5,
        time: 0.5,
    };
    
    assert_eq!(keyframe.position, Vec3::new(100.0, 50.0, 100.0));
    assert_eq!(keyframe.look_at, Vec3::ZERO);
    assert_eq!(keyframe.zoom, 1.5);
    assert_eq!(keyframe.time, 0.5);
}

/// Test shake frequency components
#[test]
fn test_shake_frequencies() {
    let shake = CameraShakeState::default();
    
    // Should have multiple frequency components for organic shake
    assert_eq!(shake.frequency_components.len(), 3);
    
    // Verify frequency ordering (low to high)
    assert!(shake.frequency_components[0].0 < shake.frequency_components[1].0);
    assert!(shake.frequency_components[1].0 < shake.frequency_components[2].0);
    
    // Verify amplitude ordering (high to low)
    assert!(shake.frequency_components[0].1 > shake.frequency_components[2].1);
}

/// Test cinematic camera component
#[test]
fn test_cinematic_camera() {
    let keyframes = vec![
        CameraKeyframe {
            position: Vec3::ZERO,
            look_at: Vec3::Z * 10.0,
            zoom: 1.0,
            time: 0.0,
        },
        CameraKeyframe {
            position: Vec3::X * 100.0,
            look_at: Vec3::Z * 10.0,
            zoom: 2.0,
            time: 1.0,
        },
    ];
    
    let cinematic = CinematicCamera {
        keyframes: keyframes.clone(),
        duration: 5.0,
        elapsed: 0.0,
        looping: false,
    };
    
    assert_eq!(cinematic.keyframes.len(), 2);
    assert_eq!(cinematic.duration, 5.0);
    assert_eq!(cinematic.elapsed, 0.0);
    assert!(!cinematic.looping);
}

/// Test camera effect events
#[test]
fn test_camera_events() {
    // Test shake event
    let shake_event = CameraEffectEvent::Shake {
        intensity: 0.8,
        duration: 0.5,
        source: Some(Vec3::new(100.0, 0.0, 100.0)),
    };
    
    match shake_event {
        CameraEffectEvent::Shake { intensity, duration, source } => {
            assert_eq!(intensity, 0.8);
            assert_eq!(duration, 0.5);
            assert!(source.is_some());
        }
        _ => panic!("Wrong event type"),
    }
    
    // Test focus event
    let focus_event = CameraEffectEvent::FocusOn {
        target: Entity::from_raw(42),
        zoom_level: Some(1.5),
        duration: 2.0,
    };
    
    match focus_event {
        CameraEffectEvent::FocusOn { target, zoom_level, duration } => {
            assert_eq!(target, Entity::from_raw(42));
            assert_eq!(zoom_level, Some(1.5));
            assert_eq!(duration, 2.0);
        }
        _ => panic!("Wrong event type"),
    }
}

/// Integration test for camera effects system
#[test] 
#[ignore = "Requires full Bevy app - run with cargo test -- --ignored"]
fn test_camera_effects_integration() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(bevy::render::RenderPlugin::default())
        .add_plugins(CameraEffectsPlugin);
    
    // Verify resources are initialized
    assert!(app.world.contains_resource::<CameraEffectSettings>());
    assert!(app.world.contains_resource::<CameraShakeState>());
    assert!(app.world.contains_resource::<PictureInPictureState>());
    
    // Spawn a camera
    let camera_entity = app.world.spawn(Camera3dBundle::default()).id();
    
    // Send a shake event
    app.world.send_event(CameraEffectEvent::Shake {
        intensity: 1.0,
        duration: 1.0,
        source: None,
    });
    
    // Update the app
    app.update();
    
    // Check shake state was updated
    let shake_state = app.world.resource::<CameraShakeState>();
    assert_eq!(shake_state.intensity, 1.0);
    assert_eq!(shake_state.duration, 1.0);
}