# Phase 4: UI Enhancements Design

## Overview

This document provides the complete technical design for UI enhancements in Phase 4, including speech bubbles, floating UI elements, comic-style indicators, camera systems, and picture-in-picture functionality.

## Speech Bubble System

### Core Components

```rust
// Speech bubble component
#[derive(Component)]
pub struct SpeechBubble {
    pub text: String,
    pub style: BubbleStyle,
    pub duration: f32,
    pub priority: i32,
    pub target_entity: Entity,
    pub offset: Vec3,
    pub tail_type: TailType,
    pub animation_state: BubbleAnimationState,
}

#[derive(Clone, Copy)]
pub enum BubbleStyle {
    Normal,
    Thought,   // Cloud-like bubble
    Shout,     // Spiky edges
    Whisper,   // Dotted outline
    System,    // Different color/style for system messages
}

#[derive(Clone, Copy)]
pub enum TailType {
    Normal,         // Standard speech tail
    Thought,        // Bubble chain
    None,           // No tail
    MultiPoint(u8), // Multiple speakers
}

#[derive(Clone)]
pub struct BubbleAnimationState {
    pub scale: f32,
    pub alpha: f32,
    pub wobble: f32,
    pub phase: AnimationPhase,
}

// Speech bubble rendering data
#[derive(Component)]
pub struct SpeechBubbleRenderer {
    pub mesh: Handle<Mesh>,
    pub material: Handle<BubbleMaterial>,
    pub text_entity: Entity,
    pub tail_entity: Entity,
}
```

### Dynamic Bubble Sizing

```rust
// Calculate bubble size based on text
pub fn calculate_bubble_size(
    text: &str,
    font_size: f32,
    max_width: f32,
) -> (Vec2, Vec<String>) {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut current_width = 0.0;
    
    let char_width = font_size * 0.6; // Approximate character width
    
    for word in words {
        let word_width = word.len() as f32 * char_width;
        
        if current_width + word_width > max_width && !current_line.is_empty() {
            lines.push(current_line.trim().to_string());
            current_line = word.to_string();
            current_width = word_width;
        } else {
            if !current_line.is_empty() {
                current_line.push(' ');
                current_width += char_width;
            }
            current_line.push_str(word);
            current_width += word_width;
        }
    }
    
    if !current_line.is_empty() {
        lines.push(current_line);
    }
    
    let width = lines.iter()
        .map(|line| line.len() as f32 * char_width)
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(0.0);
    
    let height = lines.len() as f32 * font_size * 1.5;
    let padding = font_size;
    
    (Vec2::new(width + padding * 2.0, height + padding * 2.0), lines)
}

// Generate bubble mesh with tail
pub fn generate_bubble_mesh(
    size: Vec2,
    style: BubbleStyle,
    tail_type: TailType,
    speaker_offset: Vec3,
) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    
    match style {
        BubbleStyle::Normal => {
            // Rounded rectangle with tail
            let vertices = generate_rounded_rect_vertices(size, 8.0);
            let tail_vertices = generate_tail_vertices(tail_type, size, speaker_offset);
            
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, tail_vertices);
        }
        BubbleStyle::Thought => {
            // Cloud-like shape
            let vertices = generate_cloud_vertices(size, 5);
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        }
        BubbleStyle::Shout => {
            // Spiky/star shape
            let vertices = generate_spiky_vertices(size, 12);
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        }
        _ => {}
    }
    
    mesh
}
```

### Bubble Management System

```rust
// System to manage speech bubble lifecycle
pub fn manage_speech_bubbles(
    mut commands: Commands,
    mut bubbles: Query<(Entity, &mut SpeechBubble, &mut Transform)>,
    speakers: Query<&Transform, Without<SpeechBubble>>,
    time: Res<Time>,
    camera: Query<(&Camera, &Transform)>,
) {
    let (camera, camera_transform) = camera.single();
    let dt = time.delta_seconds();
    
    for (entity, mut bubble, mut bubble_transform) in bubbles.iter_mut() {
        // Update duration
        bubble.duration -= dt;
        
        if bubble.duration <= 0.0 {
            // Fade out animation
            bubble.animation_state.phase = AnimationPhase::FadeOut;
        }
        
        // Update position to follow speaker
        if let Ok(speaker_transform) = speakers.get(bubble.target_entity) {
            let world_pos = speaker_transform.translation + bubble.offset;
            
            // Convert to screen space
            let screen_pos = camera.world_to_viewport(camera_transform, world_pos);
            
            if let Some(screen_pos) = screen_pos {
                // Keep bubble on screen
                let clamped_pos = clamp_bubble_to_screen(screen_pos, bubble.size);
                bubble_transform.translation = camera.viewport_to_world(
                    camera_transform,
                    clamped_pos,
                    0.1, // UI layer depth
                ).unwrap_or(world_pos);
            }
        }
        
        // Update animation
        update_bubble_animation(&mut bubble.animation_state, dt);
        
        // Remove if animation complete
        if matches!(bubble.animation_state.phase, AnimationPhase::Complete) {
            commands.entity(entity).despawn_recursive();
        }
    }
}

// Prevent overlapping bubbles
pub fn arrange_speech_bubbles(
    mut bubbles: Query<(&SpeechBubble, &mut Transform)>,
) {
    let mut bubble_positions: Vec<(Entity, Rect)> = Vec::new();
    
    // First pass: collect all bubble positions
    for (bubble, transform) in bubbles.iter() {
        let rect = Rect::from_center_size(
            transform.translation.truncate(),
            bubble.size,
        );
        bubble_positions.push((bubble.target_entity, rect));
    }
    
    // Second pass: adjust overlapping bubbles
    for i in 0..bubble_positions.len() {
        for j in (i + 1)..bubble_positions.len() {
            let (rect_a, rect_b) = (&bubble_positions[i].1, &bubble_positions[j].1);
            
            if rect_a.overlaps(rect_b) {
                // Move bubbles apart
                let overlap = calculate_overlap(rect_a, rect_b);
                let adjustment = overlap * 0.5;
                
                // Update positions
                bubble_positions[i].1.min.y += adjustment.y;
                bubble_positions[j].1.min.y -= adjustment.y;
            }
        }
    }
    
    // Apply adjusted positions
    for ((entity, rect), (_, mut transform)) in bubble_positions.iter().zip(bubbles.iter_mut()) {
        transform.translation.x = rect.center().x;
        transform.translation.y = rect.center().y;
    }
}
```

## Floating UI Elements

### Health and Need Bars

```rust
// Floating health bar component
#[derive(Component)]
pub struct FloatingHealthBar {
    pub current: f32,
    pub max: f32,
    pub offset: Vec3,
    pub size: Vec2,
    pub colors: HealthBarColors,
    pub visibility_mode: VisibilityMode,
}

#[derive(Clone)]
pub struct HealthBarColors {
    pub healthy: Color,      // > 75%
    pub warning: Color,      // 25-75%
    pub critical: Color,     // < 25%
    pub background: Color,
    pub border: Color,
}

#[derive(Clone, Copy)]
pub enum VisibilityMode {
    Always,
    OnDamage { timeout: f32 },
    OnHover,
    OnSelection,
}

// Render floating UI elements
pub fn render_floating_ui(
    mut commands: Commands,
    creatures: Query<(Entity, &Health, &Transform, Option<&Selected>)>,
    mut health_bars: Query<(&mut FloatingHealthBar, &mut Transform), Without<Creature>>,
    camera: Query<(&Camera, &Transform)>,
    ui_settings: Res<UISettings>,
) {
    let (camera, camera_transform) = camera.single();
    
    for (entity, health, creature_transform, selected) in creatures.iter() {
        // Determine if health bar should be visible
        let should_show = match ui_settings.health_bar_mode {
            VisibilityMode::Always => true,
            VisibilityMode::OnSelection => selected.is_some(),
            VisibilityMode::OnDamage { .. } => health.recently_damaged(),
            _ => false,
        };
        
        if should_show {
            // Calculate screen position
            let world_pos = creature_transform.translation + Vec3::Y * 2.0;
            if let Some(screen_pos) = camera.world_to_viewport(camera_transform, world_pos) {
                // Spawn or update health bar
                spawn_or_update_health_bar(
                    &mut commands,
                    entity,
                    screen_pos,
                    health.current / health.max,
                    &ui_settings.health_bar_colors,
                );
            }
        }
    }
}

// Multi-bar system for needs
#[derive(Component)]
pub struct FloatingNeedBars {
    pub bars: Vec<NeedBar>,
    pub layout: BarLayout,
    pub offset: Vec3,
}

pub struct NeedBar {
    pub need_type: NeedType,
    pub value: f32,
    pub icon: Handle<Image>,
    pub color: Color,
}

pub enum BarLayout {
    Horizontal { spacing: f32 },
    Vertical { spacing: f32 },
    Circular { radius: f32 },
}
```

### Comic Style Indicators

```rust
// Comic effect indicators
#[derive(Component)]
pub struct ComicIndicator {
    pub indicator_type: IndicatorType,
    pub animation: IndicatorAnimation,
    pub duration: f32,
    pub offset: Vec3,
}

#[derive(Clone, Copy)]
pub enum IndicatorType {
    Exclamation,  // "!"
    Question,     // "?"
    Ellipsis,     // "..."
    Heart,        // "♥"
    Anger,        // "💢"
    Sweat,        // "💧"
    Sleep,        // "ZZZ"
    Music,        // "♪♫"
    Idea,         // "💡"
}

pub struct IndicatorAnimation {
    pub bounce_height: f32,
    pub bounce_speed: f32,
    pub rotation_speed: f32,
    pub scale_pulse: f32,
    pub fade_in_time: f32,
    pub fade_out_time: f32,
}

// Animate comic indicators
pub fn animate_comic_indicators(
    mut indicators: Query<(&mut Transform, &mut ComicIndicator, &Handle<ColorMaterial>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    time: Res<Time>,
) {
    for (mut transform, mut indicator, material_handle) in indicators.iter_mut() {
        let elapsed = time.elapsed_seconds();
        let anim = &indicator.animation;
        
        // Bounce animation
        let bounce = (elapsed * anim.bounce_speed).sin() * anim.bounce_height;
        transform.translation.y += bounce;
        
        // Rotation
        transform.rotation = Quat::from_rotation_z(elapsed * anim.rotation_speed);
        
        // Scale pulse
        let scale = 1.0 + (elapsed * 2.0).sin() * anim.scale_pulse;
        transform.scale = Vec3::splat(scale);
        
        // Fade based on duration
        indicator.duration -= time.delta_seconds();
        let alpha = if indicator.duration < anim.fade_out_time {
            indicator.duration / anim.fade_out_time
        } else if elapsed < anim.fade_in_time {
            elapsed / anim.fade_in_time
        } else {
            1.0
        };
        
        if let Some(material) = materials.get_mut(material_handle) {
            material.color.set_a(alpha);
        }
    }
}
```

## Camera System

### Smooth Camera Transitions

```rust
// Camera controller for smooth transitions
#[derive(Component)]
pub struct CameraController {
    pub target: CameraTarget,
    pub transition: Option<CameraTransition>,
    pub constraints: CameraConstraints,
}

#[derive(Clone)]
pub enum CameraTarget {
    Position(Vec3),
    Entity(Entity),
    Area { center: Vec3, radius: f32 },
    Multiple(Vec<Entity>), // Frame multiple entities
}

pub struct CameraTransition {
    pub from: Transform,
    pub to_target: CameraTarget,
    pub duration: f32,
    pub elapsed: f32,
    pub easing: EasingFunction,
    pub completion_callback: Option<Box<dyn Fn() + Send + Sync>>,
}

pub struct CameraConstraints {
    pub min_zoom: f32,
    pub max_zoom: f32,
    pub bounds: Option<Rect>,
    pub smoothing: f32,
}

// Update camera with smooth transitions
pub fn update_camera_controller(
    mut camera_query: Query<(&mut Transform, &mut CameraController)>,
    targets: Query<&Transform, Without<CameraController>>,
    time: Res<Time>,
) {
    for (mut camera_transform, mut controller) in camera_query.iter_mut() {
        // Handle ongoing transition
        if let Some(transition) = &mut controller.transition {
            transition.elapsed += time.delta_seconds();
            let t = (transition.elapsed / transition.duration).min(1.0);
            let eased_t = apply_easing(t, transition.easing);
            
            // Calculate target position
            let target_pos = match &transition.to_target {
                CameraTarget::Position(pos) => *pos,
                CameraTarget::Entity(entity) => {
                    targets.get(*entity).map(|t| t.translation).unwrap_or(camera_transform.translation)
                }
                CameraTarget::Area { center, radius } => {
                    // Position camera to show entire area
                    calculate_camera_position_for_area(*center, *radius)
                }
                CameraTarget::Multiple(entities) => {
                    // Calculate bounding box of all entities
                    calculate_camera_position_for_entities(entities, &targets)
                }
            };
            
            // Interpolate position
            camera_transform.translation = transition.from.translation.lerp(target_pos, eased_t);
            camera_transform.rotation = transition.from.rotation.slerp(
                Quat::look_at(target_pos - camera_transform.translation, Vec3::Y),
                eased_t
            );
            
            // Complete transition
            if t >= 1.0 {
                if let Some(callback) = transition.completion_callback.take() {
                    callback();
                }
                controller.transition = None;
            }
        } else {
            // Regular smooth follow
            match &controller.target {
                CameraTarget::Entity(entity) => {
                    if let Ok(target_transform) = targets.get(*entity) {
                        let target_pos = target_transform.translation;
                        camera_transform.translation = camera_transform.translation.lerp(
                            target_pos,
                            controller.constraints.smoothing * time.delta_seconds()
                        );
                    }
                }
                _ => {}
            }
        }
        
        // Apply constraints
        apply_camera_constraints(&mut camera_transform, &controller.constraints);
    }
}

// Camera shake for impacts
#[derive(Component)]
pub struct CameraShake {
    pub intensity: f32,
    pub duration: f32,
    pub frequency: f32,
    pub decay: f32,
    pub elapsed: f32,
}

pub fn apply_camera_shake(
    mut cameras: Query<(&mut Transform, &CameraShake)>,
    time: Res<Time>,
) {
    for (mut transform, shake) in cameras.iter_mut() {
        let elapsed = shake.elapsed;
        let remaining = (shake.duration - elapsed).max(0.0);
        let decay = (remaining / shake.duration).powf(shake.decay);
        
        let offset = Vec3::new(
            (elapsed * shake.frequency).sin() * shake.intensity * decay,
            (elapsed * shake.frequency * 1.3).sin() * shake.intensity * decay * 0.7,
            0.0,
        );
        
        transform.translation += offset;
    }
}
```

## Picture-in-Picture System

```rust
// Picture-in-picture window component
#[derive(Component)]
pub struct PictureInPicture {
    pub viewport: Rect,
    pub render_target: Handle<Image>,
    pub camera_entity: Entity,
    pub priority: i32,
    pub style: PiPStyle,
}

pub struct PiPStyle {
    pub border_width: f32,
    pub border_color: Color,
    pub corner_radius: f32,
    pub shadow: Option<Shadow>,
    pub title: Option<String>,
}

// PiP window manager
#[derive(Resource)]
pub struct PiPManager {
    pub windows: Vec<PiPWindow>,
    pub layout: PiPLayout,
    pub max_windows: usize,
}

pub struct PiPWindow {
    pub id: u32,
    pub entity: Entity,
    pub importance: f32,
    pub auto_close: Option<f32>,
}

pub enum PiPLayout {
    BottomRight { margin: f32, spacing: f32 },
    TopRight { margin: f32, spacing: f32 },
    Grid { columns: u32, margin: f32, spacing: f32 },
    Custom(Box<dyn Fn(usize, Vec2) -> Rect + Send + Sync>),
}

// Create PiP window for important events
pub fn create_pip_window(
    mut commands: Commands,
    mut pip_manager: ResMut<PiPManager>,
    events: EventReader<ImportantGameEvent>,
    mut images: ResMut<Assets<Image>>,
) {
    for event in events.iter() {
        if should_create_pip(&event) {
            // Create render target
            let size = Extent3d {
                width: 320,
                height: 240,
                depth_or_array_layers: 1,
            };
            
            let mut render_target = Image {
                texture_descriptor: TextureDescriptor {
                    label: Some("pip_render_target"),
                    size,
                    dimension: TextureDimension::D2,
                    format: TextureFormat::Bgra8UnormSrgb,
                    mip_level_count: 1,
                    sample_count: 1,
                    usage: TextureUsages::TEXTURE_BINDING
                        | TextureUsages::COPY_DST
                        | TextureUsages::RENDER_ATTACHMENT,
                    view_formats: &[],
                },
                ..default()
            };
            
            render_target.resize(size);
            let render_target_handle = images.add(render_target);
            
            // Create PiP camera
            let pip_camera = commands.spawn((
                Camera3dBundle {
                    camera: Camera {
                        target: RenderTarget::Image(render_target_handle.clone()),
                        viewport: Some(Viewport {
                            physical_position: UVec2::ZERO,
                            physical_size: UVec2::new(320, 240),
                            ..default()
                        }),
                        ..default()
                    },
                    transform: Transform::from_translation(event.position + Vec3::Y * 10.0)
                        .looking_at(event.position, Vec3::Y),
                    ..default()
                },
                PiPCamera {
                    tracking_entity: event.entity,
                    zoom_level: 2.0,
                },
            )).id();
            
            // Calculate viewport position
            let viewport = calculate_pip_viewport(&pip_manager.layout, pip_manager.windows.len());
            
            // Create PiP window UI
            let pip_entity = commands.spawn((
                PictureInPicture {
                    viewport,
                    render_target: render_target_handle,
                    camera_entity: pip_camera,
                    priority: event.importance as i32,
                    style: PiPStyle {
                        border_width: 2.0,
                        border_color: Color::WHITE,
                        corner_radius: 8.0,
                        shadow: Some(Shadow {
                            color: Color::rgba(0.0, 0.0, 0.0, 0.5),
                            offset: Vec2::new(2.0, -2.0),
                            blur_radius: 4.0,
                        }),
                        title: Some(event.description.clone()),
                    },
                },
                Name::new(format!("PiP_{}", event.description)),
            )).id();
            
            // Add to manager
            pip_manager.windows.push(PiPWindow {
                id: pip_entity.index(),
                entity: pip_entity,
                importance: event.importance,
                auto_close: Some(10.0), // Close after 10 seconds
            });
            
            // Limit number of windows
            if pip_manager.windows.len() > pip_manager.max_windows {
                // Remove least important
                pip_manager.windows.sort_by(|a, b| b.importance.partial_cmp(&a.importance).unwrap());
                if let Some(window) = pip_manager.windows.pop() {
                    commands.entity(window.entity).despawn_recursive();
                }
            }
        }
    }
}

// Render PiP windows
pub fn render_pip_windows(
    windows: Query<(&PictureInPicture, &Handle<Image>)>,
    mut egui_context: ResMut<EguiContext>,
) {
    for (pip, image_handle) in windows.iter() {
        let ctx = egui_context.ctx_mut();
        
        egui::Window::new(pip.style.title.as_deref().unwrap_or(""))
            .fixed_rect(egui::Rect::from_min_size(
                egui::pos2(pip.viewport.min.x, pip.viewport.min.y),
                egui::vec2(pip.viewport.width(), pip.viewport.height()),
            ))
            .frame(egui::Frame {
                inner_margin: egui::style::Margin::same(0.0),
                outer_margin: egui::style::Margin::same(0.0),
                rounding: egui::Rounding::same(pip.style.corner_radius),
                shadow: pip.style.shadow.map(|s| egui::epaint::Shadow {
                    extrusion: s.blur_radius,
                    color: egui::Color32::from_rgba_premultiplied(
                        (s.color.r() * 255.0) as u8,
                        (s.color.g() * 255.0) as u8,
                        (s.color.b() * 255.0) as u8,
                        (s.color.a() * 255.0) as u8,
                    ),
                }).unwrap_or_default(),
                fill: egui::Color32::BLACK,
                stroke: egui::Stroke::new(
                    pip.style.border_width,
                    egui::Color32::from_rgb(
                        (pip.style.border_color.r() * 255.0) as u8,
                        (pip.style.border_color.g() * 255.0) as u8,
                        (pip.style.border_color.b() * 255.0) as u8,
                    ),
                ),
            })
            .show(ctx, |ui| {
                // Render the image from the render target
                ui.image(egui::load::SizedTexture::new(
                    image_handle.id(),
                    egui::vec2(pip.viewport.width(), pip.viewport.height()),
                ));
            });
    }
}
```

## Animation System Integration

```rust
// UI animation curves
#[derive(Clone, Copy)]
pub enum UIAnimationCurve {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Bounce,
    Elastic,
    Back,
}

pub fn apply_ui_animation_curve(t: f32, curve: UIAnimationCurve) -> f32 {
    match curve {
        UIAnimationCurve::Linear => t,
        UIAnimationCurve::EaseIn => t * t,
        UIAnimationCurve::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
        UIAnimationCurve::EaseInOut => {
            if t < 0.5 {
                2.0 * t * t
            } else {
                1.0 - 2.0 * (1.0 - t) * (1.0 - t)
            }
        }
        UIAnimationCurve::Bounce => {
            if t < 0.5 {
                8.0 * t * t * t * t
            } else {
                1.0 - 8.0 * (1.0 - t).powi(4)
            }
        }
        UIAnimationCurve::Elastic => {
            (2.0 * std::f32::consts::PI * t).sin() * (1.0 - t).exp()
        }
        UIAnimationCurve::Back => {
            let c = 1.70158;
            t * t * ((c + 1.0) * t - c)
        }
    }
}

// UI element transitions
#[derive(Component)]
pub struct UITransition {
    pub property: UIProperty,
    pub from: f32,
    pub to: f32,
    pub duration: f32,
    pub elapsed: f32,
    pub curve: UIAnimationCurve,
    pub on_complete: Option<Box<dyn Fn() + Send + Sync>>,
}

#[derive(Clone, Copy)]
pub enum UIProperty {
    PositionX,
    PositionY,
    Scale,
    Alpha,
    Rotation,
}
```

## Performance Considerations

```rust
// UI batching system
pub struct UIBatchRenderer {
    pub speech_bubbles: Vec<BubbleBatch>,
    pub health_bars: Vec<BarBatch>,
    pub indicators: Vec<IndicatorBatch>,
}

pub struct BubbleBatch {
    pub texture: Handle<Image>,
    pub instances: Vec<BubbleInstance>,
}

// Batch UI rendering for performance
pub fn batch_ui_elements(
    bubbles: Query<(&SpeechBubble, &Transform)>,
    health_bars: Query<(&FloatingHealthBar, &Transform)>,
    indicators: Query<(&ComicIndicator, &Transform)>,
    mut batch_renderer: ResMut<UIBatchRenderer>,
) {
    // Clear previous batches
    batch_renderer.speech_bubbles.clear();
    batch_renderer.health_bars.clear();
    batch_renderer.indicators.clear();
    
    // Batch speech bubbles by style
    let mut bubble_batches: HashMap<BubbleStyle, Vec<BubbleInstance>> = HashMap::new();
    
    for (bubble, transform) in bubbles.iter() {
        bubble_batches.entry(bubble.style)
            .or_default()
            .push(BubbleInstance {
                transform: *transform,
                color: Color::WHITE,
                uv_offset: Vec2::ZERO,
            });
    }
    
    // Convert to batch format
    for (style, instances) in bubble_batches {
        batch_renderer.speech_bubbles.push(BubbleBatch {
            texture: get_bubble_texture(style),
            instances,
        });
    }
}

// UI culling
pub fn cull_ui_elements(
    camera: Query<&Camera>,
    mut ui_elements: Query<(&Transform, &mut Visibility), With<UIElement>>,
) {
    let camera = camera.single();
    
    for (transform, mut visibility) in ui_elements.iter_mut() {
        // Simple frustum check
        let in_view = camera.viewport.map_or(true, |viewport| {
            let pos = transform.translation.truncate();
            pos.x >= 0.0 && pos.x <= viewport.physical_size.x as f32 &&
            pos.y >= 0.0 && pos.y <= viewport.physical_size.y as f32
        });
        
        visibility.is_visible = in_view;
    }
}
```

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bubble_text_wrapping() {
        let text = "This is a long text that should wrap to multiple lines";
        let (size, lines) = calculate_bubble_size(text, 16.0, 200.0);
        
        assert!(lines.len() > 1);
        assert!(size.x <= 200.0 + 32.0); // Max width + padding
    }
    
    #[test]
    fn test_ui_animation_curves() {
        // Test that curves start at 0 and end at 1
        for curve in [
            UIAnimationCurve::Linear,
            UIAnimationCurve::EaseIn,
            UIAnimationCurve::EaseOut,
            UIAnimationCurve::EaseInOut,
        ] {
            assert!((apply_ui_animation_curve(0.0, curve) - 0.0).abs() < 0.001);
            assert!((apply_ui_animation_curve(1.0, curve) - 1.0).abs() < 0.001);
        }
    }
    
    #[test]
    fn test_pip_layout() {
        let layout = PiPLayout::Grid { columns: 2, margin: 10.0, spacing: 5.0 };
        
        for i in 0..4 {
            let rect = calculate_pip_viewport(&layout, i);
            assert!(rect.width() > 0.0);
            assert!(rect.height() > 0.0);
        }
    }
}
```