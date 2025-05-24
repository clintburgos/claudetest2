# UI System

## Table of Contents
1. [Visual Design](#visual-design)
2. [Controls & Interface](#controls--interface)
3. [State Management](#state-management)
4. [Quick Reference](#quick-reference)

---

## Visual Design

### Art Direction

The simulation uses a charming, approachable visual style that makes complex behaviors easy to read and emotionally engaging.

#### Isometric View
- **Camera Angle**: 30° elevation, 45° rotation
- **Benefits**: 
  - Natural depth perception
  - Clear creature visibility
  - Efficient tile rendering
  - Familiar to strategy game players

#### Creature Design
- **Style**: Cartoonish with exaggerated features
- **Size**: 1x1 to 2x2 tiles based on species
- **Key Features**:
  - Large, expressive eyes
  - Clear emotional indicators
  - Distinct silhouettes per species
  - Readable at all zoom levels

### Color Palette

```rust
pub struct ColorPalette {
    // Biome colors
    pub grassland: Color::rgb(0.4, 0.7, 0.3),
    pub forest: Color::rgb(0.2, 0.5, 0.2),
    pub desert: Color::rgb(0.9, 0.8, 0.5),
    pub tundra: Color::rgb(0.8, 0.9, 0.95),
    pub water: Color::rgb(0.3, 0.5, 0.8),
    
    // Creature states
    pub healthy: Color::rgb(0.3, 0.8, 0.3),
    pub hungry: Color::rgb(0.9, 0.6, 0.2),
    pub sick: Color::rgb(0.6, 0.6, 0.3),
    pub mating: Color::rgb(0.9, 0.3, 0.6),
    
    // UI elements
    pub ui_primary: Color::rgb(0.2, 0.3, 0.4),
    pub ui_secondary: Color::rgb(0.3, 0.4, 0.5),
    pub ui_accent: Color::rgb(0.4, 0.7, 0.9),
}
```

### Visual Feedback Systems

#### Emotional Expressions
```rust
pub enum EmotionalExpression {
    Happy { 
        eye_shape: EyeShape::Curved,
        mouth: MouthShape::Smile,
        bounce_amplitude: f32,
    },
    Sad {
        eye_shape: EyeShape::Droopy,
        mouth: MouthShape::Frown,
        tear_particles: bool,
    },
    Angry {
        eye_shape: EyeShape::Narrow,
        mouth: MouthShape::Scowl,
        steam_particles: bool,
    },
    Scared {
        eye_shape: EyeShape::Wide,
        body_shake: f32,
        sweat_particles: bool,
    },
    Curious {
        eye_shape: EyeShape::Wide,
        head_tilt: f32,
        question_mark: bool,
    },
}
```

#### Particle Effects
```rust
pub struct ParticleSystem {
    pub emotion_particles: HashMap<Emotion, ParticleEmitter>,
    pub action_particles: HashMap<ActionType, ParticleEmitter>,
    pub environment_particles: HashMap<Weather, ParticleEmitter>,
}

// Examples
impl ParticleEffects {
    pub fn emit_hearts(position: Vec3) {
        // Pink hearts for love/mating
    }
    
    pub fn emit_zzz(position: Vec3) {
        // Z's for sleeping
    }
    
    pub fn emit_sparkles(position: Vec3) {
        // Sparkles for successful actions
    }
}
```

### Creature Visibility System

Ensuring creatures remain visible is crucial for player engagement:

```rust
pub struct VisibilitySystem {
    outline_renderer: OutlineRenderer,
    transparency_handler: TransparencyHandler,
    indicator_system: IndicatorSystem,
}

impl VisibilitySystem {
    pub fn ensure_creature_visible(&mut self, creature: &Creature, camera: &Camera) {
        if self.is_behind_object(creature, camera) {
            // Apply transparency to occluding objects
            self.transparency_handler.make_transparent(
                self.get_occluding_objects(creature, camera),
                0.3 // 30% opacity
            );
            
            // Add outline to creature
            self.outline_renderer.add_outline(
                creature.id,
                OutlineStyle {
                    color: Color::WHITE,
                    thickness: 2.0,
                    pulse: true,
                }
            );
        }
        
        // Add floating indicator if very occluded
        if self.occlusion_percentage(creature, camera) > 0.8 {
            self.indicator_system.add_indicator(
                creature.id,
                IndicatorType::Arrow,
                creature.position + Vec3::Y * 5.0
            );
        }
    }
}
```

---

## Controls & Interface

### Input Schemes

#### Mouse Controls
| Action | Input | Description |
|--------|-------|-------------|
| Select | Left Click | Select creature/object |
| Multi-select | Shift + Drag | Box select multiple |
| Context Menu | Right Click | Open action menu |
| Pan Camera | Middle Drag | Move camera |
| Zoom | Scroll Wheel | Zoom in/out |
| Rotate Camera | Q/E | Rotate view |

#### Keyboard Shortcuts
| Key | Action |
|-----|--------|
| Space | Pause/Resume |
| 1-6 | Time speed presets |
| F | Follow selected |
| H | Return home |
| Tab | Cycle selection |
| Esc | Deselect all |

#### Gamepad Support
```rust
pub struct GamepadMapping {
    // Movement
    pub camera_pan: GamepadAxis::LeftStick,
    pub camera_rotate: GamepadAxis::RightStickX,
    pub camera_zoom: GamepadAxis::RightStickY,
    
    // Selection
    pub select: GamepadButton::A,
    pub context_menu: GamepadButton::X,
    pub cycle_selection: GamepadButton::RightBumper,
    
    // Time control
    pub pause: GamepadButton::Start,
    pub speed_up: GamepadButton::RightTrigger,
    pub slow_down: GamepadButton::LeftTrigger,
}
```

### Camera System

```rust
pub struct CameraController {
    pub mode: CameraMode,
    pub position: Vec3,
    pub zoom: f32,
    pub rotation: f32,
    pub bounds: CameraBounds,
    pub smoothing: f32,
}

pub enum CameraMode {
    Free,
    Follow { target: EntityId, offset: Vec3 },
    Overview,
    Cinematic { path: CameraPath },
}

impl CameraController {
    pub fn update(&mut self, input: &Input, delta: f32) {
        match self.mode {
            CameraMode::Free => {
                // Pan with mouse or keyboard
                if input.mouse_button_pressed(MouseButton::Middle) {
                    let delta = input.mouse_delta();
                    self.position.x -= delta.x * self.zoom;
                    self.position.z -= delta.y * self.zoom;
                }
                
                // Zoom with scroll wheel
                self.zoom *= 1.0 - input.scroll_delta() * 0.1;
                self.zoom = self.zoom.clamp(MIN_ZOOM, MAX_ZOOM);
                
                // Rotate with Q/E
                if input.key_pressed(KeyCode::Q) {
                    self.rotation -= ROTATION_SPEED * delta;
                }
                if input.key_pressed(KeyCode::E) {
                    self.rotation += ROTATION_SPEED * delta;
                }
            }
            
            CameraMode::Follow { target, offset } => {
                // Smoothly follow target
                let target_pos = get_entity_position(target);
                let desired_pos = target_pos + offset;
                self.position = self.position.lerp(desired_pos, self.smoothing * delta);
            }
            
            _ => {}
        }
        
        // Apply bounds
        self.position = self.bounds.clamp(self.position);
    }
}
```

### HUD Layout

```rust
pub struct HUDLayout {
    // Top bar
    pub time_controls: Rect,
    pub resource_bar: Rect,
    pub notification_area: Rect,
    
    // Bottom panel
    pub selection_panel: Rect,
    pub action_buttons: Rect,
    pub minimap: Rect,
    
    // Side panels
    pub data_views: Rect,
    pub quick_stats: Rect,
}

impl HUDLayout {
    pub fn adaptive_layout(&mut self, screen_size: Vec2) {
        // Scale UI elements based on screen size
        let ui_scale = (screen_size.x / 1920.0).min(1.0).max(0.7);
        
        // Reposition elements for different aspect ratios
        if screen_size.x / screen_size.y < 16.0 / 9.0 {
            // Narrower screen - stack vertically
            self.compact_layout();
        } else {
            // Wide screen - spread horizontally
            self.wide_layout();
        }
    }
}
```

### Selection System

```rust
pub struct SelectionSystem {
    pub selected: HashSet<EntityId>,
    pub selection_groups: [SelectionGroup; 10],
    pub last_selected: Option<EntityId>,
}

impl SelectionSystem {
    pub fn handle_click(&mut self, world_pos: Vec3, shift_held: bool) {
        let clicked_entity = self.get_entity_at(world_pos);
        
        match (clicked_entity, shift_held) {
            (Some(entity), true) => {
                // Add to selection
                self.selected.insert(entity);
            }
            (Some(entity), false) => {
                // Replace selection
                self.selected.clear();
                self.selected.insert(entity);
            }
            (None, false) => {
                // Clear selection
                self.selected.clear();
            }
            _ => {}
        }
        
        self.last_selected = clicked_entity;
    }
    
    pub fn box_select(&mut self, start: Vec2, end: Vec2) {
        let bounds = Rect::from_corners(start, end);
        
        for entity in get_visible_entities() {
            let screen_pos = world_to_screen(entity.position);
            if bounds.contains(screen_pos) {
                self.selected.insert(entity.id);
            }
        }
    }
}
```

---

## State Management

### UI State Architecture

```rust
pub struct UIState {
    // View state
    pub active_view: ViewType,
    pub camera_state: CameraState,
    pub time_control_state: TimeControlState,
    
    // Selection state
    pub selection: SelectionState,
    pub hover_info: Option<HoverInfo>,
    
    // Panel states
    pub panels: HashMap<PanelType, PanelState>,
    pub notifications: NotificationQueue,
    
    // Data visualization
    pub active_overlays: HashSet<OverlayType>,
    pub graph_settings: GraphSettings,
}

pub enum ViewType {
    Overview,
    Population,
    Genetics,
    Trends,
}

pub struct PanelState {
    pub visible: bool,
    pub pinned: bool,
    pub position: Vec2,
    pub size: Vec2,
    pub minimized: bool,
}
```

### Reactive Updates

```rust
pub struct UIUpdateSystem {
    update_queue: VecDeque<UIUpdate>,
    batch_timer: Timer,
    dirty_flags: DirtyFlags,
}

pub enum UIUpdate {
    CreatureSelected(EntityId),
    StatChanged { creature: EntityId, stat: StatType, value: f32 },
    NotificationAdded(Notification),
    ViewChanged(ViewType),
}

impl UIUpdateSystem {
    pub fn queue_update(&mut self, update: UIUpdate) {
        self.update_queue.push_back(update);
        
        // Mark relevant UI sections as dirty
        match &update {
            UIUpdate::CreatureSelected(_) => {
                self.dirty_flags.selection_panel = true;
                self.dirty_flags.info_panel = true;
            }
            UIUpdate::StatChanged { .. } => {
                self.dirty_flags.stats_display = true;
            }
            _ => {}
        }
    }
    
    pub fn process_updates(&mut self, ui: &mut UI) {
        // Batch updates for efficiency
        if self.batch_timer.tick(delta).just_finished() {
            while let Some(update) = self.update_queue.pop_front() {
                self.apply_update(ui, update);
            }
            
            // Only redraw dirty sections
            self.redraw_dirty_sections(ui);
            self.dirty_flags.clear();
        }
    }
}
```

### Data Binding

```rust
pub struct DataBinding<T> {
    source: Box<dyn Fn() -> T>,
    target: WidgetId,
    formatter: Box<dyn Fn(&T) -> String>,
    update_rate: f32,
    last_value: Option<T>,
}

impl<T: PartialEq + Clone> DataBinding<T> {
    pub fn update(&mut self, ui: &mut UI) {
        let current_value = (self.source)();
        
        // Only update if changed
        if self.last_value.as_ref() != Some(&current_value) {
            let formatted = (self.formatter)(&current_value);
            ui.set_text(self.target, &formatted);
            self.last_value = Some(current_value);
        }
    }
}

// Usage example
let health_binding = DataBinding {
    source: Box::new(|| selected_creature.health),
    target: health_label,
    formatter: Box::new(|h| format!("Health: {:.0}%", h)),
    update_rate: 0.1,
    last_value: None,
};
```

### Notification System

```rust
pub struct NotificationSystem {
    active_notifications: VecDeque<Notification>,
    notification_pool: Vec<NotificationWidget>,
    animation_controller: AnimationController,
}

pub struct Notification {
    id: NotificationId,
    notification_type: NotificationType,
    message: String,
    icon: Option<IconType>,
    duration: f32,
    priority: Priority,
    action: Option<NotificationAction>,
}

impl NotificationSystem {
    pub fn show_notification(&mut self, notification: Notification) {
        // Find or create widget
        let widget = self.get_or_create_widget();
        
        // Configure widget
        widget.set_message(&notification.message);
        if let Some(icon) = notification.icon {
            widget.set_icon(icon);
        }
        
        // Animate in
        self.animation_controller.animate(
            widget.id,
            Animation::SlideIn { 
                from: Vec2::new(SCREEN_WIDTH, widget.position.y),
                duration: 0.3,
                easing: Easing::EaseOutBack,
            }
        );
        
        // Queue removal
        self.queue_removal(notification.id, notification.duration);
    }
}
```

### Time Control UI

```rust
pub struct TimeControlUI {
    speed_selector: SpeedSelector,
    play_pause_button: Button,
    generation_display: Label,
    event_timeline: Timeline,
}

pub struct SpeedSelector {
    speeds: [TimeSpeed; 7],
    current_index: usize,
    smooth_transition: bool,
}

impl SpeedSelector {
    pub fn render(&self, ui: &mut UIContext) {
        ui.horizontal(|ui| {
            // Play/Pause toggle
            if ui.button(if self.is_paused() { "▶" } else { "⏸" }).clicked() {
                self.toggle_pause();
            }
            
            // Speed buttons
            for (i, speed) in self.speeds.iter().enumerate() {
                let selected = i == self.current_index;
                
                if ui.selectable_label(selected, speed.label).clicked() {
                    self.set_speed(i);
                }
            }
            
            // Generation counter
            ui.separator();
            ui.label(format!("Generation: {}", self.current_generation()));
        });
    }
}

const TIME_SPEEDS: [TimeSpeed; 7] = [
    TimeSpeed { multiplier: 0.0, label: "⏸" },
    TimeSpeed { multiplier: 1.0, label: "1x" },
    TimeSpeed { multiplier: 5.0, label: "5x" },
    TimeSpeed { multiplier: 20.0, label: "20x" },
    TimeSpeed { multiplier: 100.0, label: "100x" },
    TimeSpeed { multiplier: 500.0, label: "500x" },
    TimeSpeed { multiplier: 1000.0, label: "1000x" },
];
```

### Data Visualization Panels

```rust
pub struct DataVisualization {
    population_graph: PopulationGraph,
    genetics_view: GeneticsTreeView,
    social_network: SocialNetworkView,
    resource_heatmap: ResourceHeatmap,
}

pub struct PopulationGraph {
    time_series: TimeSeries<PopulationData>,
    species_breakdown: PieChart,
    birth_death_rates: LineGraph,
}

impl PopulationGraph {
    pub fn update(&mut self, population_data: &PopulationData) {
        // Add new data point
        self.time_series.add_point(current_time(), population_data.clone());
        
        // Update species breakdown
        self.species_breakdown.update_data(
            population_data.species_counts
                .iter()
                .map(|(species, count)| {
                    (species.name(), *count as f32)
                })
                .collect()
        );
        
        // Update birth/death rates
        self.birth_death_rates.add_point("Births", population_data.births_per_minute);
        self.birth_death_rates.add_point("Deaths", population_data.deaths_per_minute);
    }
}
```

---

## Quick Reference

### Control Schemes

For complete control mappings including keyboard, mouse, and gamepad support, see [Input System Implementation](../reference/INPUT_SYSTEM_IMPLEMENTATION.md#control-mappings).

### UI Element Sizes

```rust
const UI_SCALE_FACTORS: UiScale = UiScale {
    button_height: 32.0,
    icon_size: 24.0,
    panel_padding: 8.0,
    text_size_small: 12.0,
    text_size_normal: 14.0,
    text_size_large: 18.0,
    minimum_touch_target: 44.0, // Accessibility
};
```

### Color Coding Standards

| Element | Color | Meaning |
|---------|-------|---------|
| Green | `#4CAF50` | Healthy, positive |
| Yellow | `#FFC107` | Warning, needs attention |
| Red | `#F44336` | Danger, critical |
| Blue | `#2196F3` | Information, neutral |
| Purple | `#9C27B0` | Special, rare |

### Performance Guidelines

```rust
// Update rates for UI elements
const UI_UPDATE_RATES: UpdateRates = UpdateRates {
    critical_stats: 60.0,    // Every frame
    general_stats: 10.0,     // 10 Hz
    graphs: 1.0,             // 1 Hz
    minimap: 2.0,            // 2 Hz
    notifications: 60.0,     // Every frame (for animations)
};

// LOD for UI elements
pub fn should_update_ui_element(element: &UIElement, camera: &Camera) -> bool {
    match element.importance {
        Importance::Critical => true,
        Importance::High => camera.zoom < 100.0,
        Importance::Medium => camera.zoom < 50.0,
        Importance::Low => camera.zoom < 20.0,
    }
}
```

### Accessibility Features

```rust
pub struct AccessibilityOptions {
    // Visual
    pub ui_scale: f32,              // 0.8 - 1.5
    pub contrast_mode: ContrastMode,
    pub colorblind_mode: ColorblindMode,
    pub reduce_motion: bool,
    
    // Input
    pub sticky_keys: bool,
    pub key_repeat_delay: f32,
    pub mouse_sensitivity: f32,
    
    // Feedback
    pub screen_reader_hints: bool,
    pub audio_cues: bool,
    pub haptic_feedback: bool,
}
```

### Common UI Patterns

```rust
// Tooltip on hover
if ui.is_hovered() {
    show_tooltip(ui, |ui| {
        ui.label("Creature Status");
        ui.separator();
        ui.label(format!("Health: {:.0}%", creature.health));
        ui.label(format!("Hunger: {:.0}%", creature.hunger));
    });
}

// Contextual menu
if ui.response.secondary_clicked() {
    ui.menu("context_menu", |ui| {
        if ui.button("Follow").clicked() {
            camera.follow(creature.id);
        }
        if ui.button("View Details").clicked() {
            open_creature_panel(creature.id);
        }
    });
}

// Adaptive layout
let available_width = ui.available_width();
if available_width > 800.0 {
    ui.horizontal(|ui| { /* Wide layout */ });
} else {
    ui.vertical(|ui| { /* Narrow layout */ });
}
```