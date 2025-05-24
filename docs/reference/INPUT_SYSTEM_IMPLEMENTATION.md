# Input System Implementation Design

## Overview

A comprehensive input system that handles keyboard, mouse, gamepad, and touch inputs with support for remapping, accessibility, buffering, and complex input combinations. The system integrates with Bevy's input handling while providing advanced features for a smooth user experience.

## Core Architecture

```rust
use bevy::prelude::*;
use std::collections::{HashMap, VecDeque};
use std::time::Duration;

// Input contexts for different game states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputContext {
    MainMenu,
    Gameplay,
    CreatureSelection,
    CameraControl,
    TimeControl,
    DataVisualization,
    DialogBox,
    TextInput,
}

// Unified input action enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputAction {
    // Camera
    CameraMove(Direction2D),
    CameraRotate(RotationDirection),
    CameraZoom(ZoomDirection),
    CameraReset,
    
    // Selection
    Select,
    MultiSelect,
    SelectAll,
    DeselectAll,
    CycleSelection,
    
    // Time Control
    Pause,
    Play,
    SpeedUp,
    SlowDown,
    SetSpeed(u32),
    
    // UI Navigation
    UINavigate(Direction2D),
    UIConfirm,
    UICancel,
    UITab,
    UIEscape,
    
    // Creature Control
    FollowCreature,
    IssueCommand(CreatureCommand),
    OpenCreatureDetails,
    
    // System
    Save,
    Load,
    Screenshot,
    ToggleFullscreen,
    OpenSettings,
    
    // Debug
    ToggleDebugInfo,
    TogglePerformanceStats,
    ToggleColliders,
}

// Input source abstraction
#[derive(Debug, Clone)]
pub enum InputSource {
    Keyboard(KeyCode),
    MouseButton(MouseButton),
    MouseWheel,
    MouseMotion,
    GamepadButton(GamepadButton),
    GamepadAxis(GamepadAxis),
    GamepadStick(GamepadStick),
    Touch(TouchPhase),
    Gesture(GestureType),
}

// Input binding with modifiers
#[derive(Debug, Clone)]
pub struct InputBinding {
    pub source: InputSource,
    pub modifiers: InputModifiers,
    pub action: InputAction,
    pub context: InputContext,
    pub hold_time: Option<Duration>,
    pub repeat_rate: Option<Duration>,
}

#[derive(Debug, Clone, Default)]
pub struct InputModifiers {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub meta: bool,
}
```

### Input Manager

```rust
pub struct InputManager {
    bindings: HashMap<InputContext, Vec<InputBinding>>,
    active_contexts: Vec<InputContext>,
    input_buffer: InputBuffer,
    gesture_recognizer: GestureRecognizer,
    gamepad_manager: GamepadManager,
    accessibility: AccessibilitySettings,
    remapping_profile: RemappingProfile,
}

pub struct InputBuffer {
    buffer: VecDeque<BufferedInput>,
    max_age: Duration,
    max_size: usize,
}

pub struct BufferedInput {
    action: InputAction,
    timestamp: Instant,
    consumed: bool,
}

impl InputManager {
    pub fn update(&mut self, input: &Input<KeyCode>, time: &Time) {
        self.input_buffer.clean_old_inputs(time.elapsed());
        
        // Process raw inputs for active contexts
        for context in &self.active_contexts {
            if let Some(bindings) = self.bindings.get(context) {
                for binding in bindings {
                    if self.check_binding(binding, input, time) {
                        self.input_buffer.add(BufferedInput {
                            action: binding.action,
                            timestamp: time.startup() + time.elapsed(),
                            consumed: false,
                        });
                    }
                }
            }
        }
    }
    
    pub fn consume_action(&mut self, action: InputAction) -> bool {
        for input in &mut self.input_buffer.buffer {
            if input.action == action && !input.consumed {
                input.consumed = true;
                return true;
            }
        }
        false
    }
    
    pub fn is_action_pressed(&self, action: InputAction) -> bool {
        self.input_buffer.buffer.iter()
            .any(|input| input.action == action && !input.consumed)
    }
    
    fn check_binding(&self, binding: &InputBinding, input: &Input<KeyCode>, time: &Time) -> bool {
        // Check modifiers
        if !self.check_modifiers(&binding.modifiers, input) {
            return false;
        }
        
        // Check source
        match &binding.source {
            InputSource::Keyboard(key) => {
                if let Some(hold_time) = binding.hold_time {
                    input.pressed(*key) && 
                    self.get_key_hold_duration(*key, time) >= hold_time
                } else {
                    input.just_pressed(*key)
                }
            }
            // Handle other input sources...
            _ => false,
        }
    }
}
```

### Gesture Recognition

```rust
pub struct GestureRecognizer {
    active_touches: HashMap<u64, TouchInfo>,
    gesture_detectors: Vec<Box<dyn GestureDetector>>,
    recognized_gestures: VecDeque<RecognizedGesture>,
}

pub struct TouchInfo {
    id: u64,
    start_position: Vec2,
    current_position: Vec2,
    start_time: Instant,
    velocity: Vec2,
    path: Vec<Vec2>,
}

pub trait GestureDetector: Send + Sync {
    fn update(&mut self, touches: &HashMap<u64, TouchInfo>, delta_time: f32);
    fn check_gesture(&self) -> Option<GestureType>;
    fn reset(&mut self);
}

#[derive(Debug, Clone)]
pub enum GestureType {
    Tap { position: Vec2 },
    DoubleTap { position: Vec2 },
    LongPress { position: Vec2, duration: Duration },
    Swipe { direction: Vec2, velocity: f32 },
    Pinch { scale_delta: f32, center: Vec2 },
    Rotate { angle_delta: f32, center: Vec2 },
    Pan { delta: Vec2 },
}

// Pinch gesture detector
pub struct PinchDetector {
    initial_distance: Option<f32>,
    current_distance: f32,
    center: Vec2,
}

impl GestureDetector for PinchDetector {
    fn update(&mut self, touches: &HashMap<u64, TouchInfo>, _delta_time: f32) {
        if touches.len() == 2 {
            let positions: Vec<Vec2> = touches.values()
                .map(|t| t.current_position)
                .collect();
            
            let distance = (positions[0] - positions[1]).length();
            self.center = (positions[0] + positions[1]) / 2.0;
            
            if self.initial_distance.is_none() {
                self.initial_distance = Some(distance);
            }
            
            self.current_distance = distance;
        } else {
            self.reset();
        }
    }
    
    fn check_gesture(&self) -> Option<GestureType> {
        if let Some(initial) = self.initial_distance {
            let scale_delta = self.current_distance / initial;
            if (scale_delta - 1.0).abs() > 0.1 {
                return Some(GestureType::Pinch {
                    scale_delta,
                    center: self.center,
                });
            }
        }
        None
    }
    
    fn reset(&mut self) {
        self.initial_distance = None;
    }
}
```

### Gamepad Support

```rust
pub struct GamepadManager {
    connected_gamepads: HashMap<usize, GamepadInfo>,
    dead_zones: DeadZoneSettings,
    vibration_settings: VibrationSettings,
    button_mappings: HashMap<GamepadButton, InputAction>,
    axis_mappings: HashMap<GamepadAxis, AxisMapping>,
}

pub struct GamepadInfo {
    id: usize,
    name: String,
    layout: GamepadLayout,
    battery_level: Option<f32>,
    last_input: Instant,
}

pub enum GamepadLayout {
    Xbox,
    PlayStation,
    Switch,
    Generic,
}

pub struct DeadZoneSettings {
    stick_threshold: f32,
    trigger_threshold: f32,
    shape: DeadZoneShape,
}

pub enum DeadZoneShape {
    Circular,
    Square,
    Cross,
}

impl GamepadManager {
    pub fn process_gamepad_input(
        &mut self,
        gamepads: &Gamepads,
        axes: &Axis<GamepadAxis>,
        buttons: &Input<GamepadButton>,
    ) -> Vec<InputAction> {
        let mut actions = Vec::new();
        
        for gamepad in gamepads.iter() {
            // Process buttons
            for (button_type, action) in &self.button_mappings {
                let button = GamepadButton::new(gamepad, *button_type);
                if buttons.just_pressed(button) {
                    actions.push(*action);
                }
            }
            
            // Process axes with dead zone
            for (axis_type, mapping) in &self.axis_mappings {
                let axis = GamepadAxis::new(gamepad, *axis_type);
                if let Some(value) = axes.get(axis) {
                    let adjusted = self.apply_dead_zone(value, *axis_type);
                    if adjusted.abs() > 0.0 {
                        actions.push(mapping.to_action(adjusted));
                    }
                }
            }
        }
        
        actions
    }
    
    fn apply_dead_zone(&self, value: f32, axis: GamepadAxisType) -> f32 {
        let threshold = match axis {
            GamepadAxisType::LeftStickX | GamepadAxisType::LeftStickY |
            GamepadAxisType::RightStickX | GamepadAxisType::RightStickY => {
                self.dead_zones.stick_threshold
            }
            GamepadAxisType::LeftZ | GamepadAxisType::RightZ => {
                self.dead_zones.trigger_threshold
            }
            _ => 0.0,
        };
        
        if value.abs() < threshold {
            0.0
        } else {
            // Rescale to maintain full range after dead zone
            let sign = value.signum();
            let magnitude = (value.abs() - threshold) / (1.0 - threshold);
            sign * magnitude
        }
    }
    
    pub fn set_vibration(
        &mut self,
        gamepad: usize,
        intensity: VibrationIntensity,
        duration: Duration,
    ) {
        if let Some(info) = self.connected_gamepads.get(&gamepad) {
            // Platform-specific vibration code
            match info.layout {
                GamepadLayout::PlayStation => {
                    // DualShock/DualSense specific vibration
                }
                GamepadLayout::Xbox => {
                    // Xbox controller vibration
                }
                _ => {
                    // Generic rumble
                }
            }
        }
    }
}
```

### Input Remapping

```rust
pub struct RemappingProfile {
    name: String,
    custom_bindings: HashMap<(InputContext, InputAction), InputBinding>,
    disabled_actions: HashSet<InputAction>,
    sensitivity_settings: SensitivitySettings,
}

pub struct RemappingUI {
    active_profile: String,
    profiles: HashMap<String, RemappingProfile>,
    remapping_state: Option<RemappingState>,
}

pub struct RemappingState {
    action: InputAction,
    context: InputContext,
    waiting_for_input: bool,
    conflict_check: bool,
}

impl RemappingUI {
    pub fn start_remapping(&mut self, action: InputAction, context: InputContext) {
        self.remapping_state = Some(RemappingState {
            action,
            context,
            waiting_for_input: true,
            conflict_check: true,
        });
    }
    
    pub fn handle_remapping_input(&mut self, input: InputSource) -> RemappingResult {
        if let Some(state) = &self.remapping_state {
            // Check for conflicts
            if state.conflict_check {
                if let Some(conflict) = self.check_binding_conflict(&input, state.context) {
                    return RemappingResult::Conflict(conflict);
                }
            }
            
            // Create new binding
            let binding = InputBinding {
                source: input,
                modifiers: self.capture_current_modifiers(),
                action: state.action,
                context: state.context,
                hold_time: None,
                repeat_rate: None,
            };
            
            // Update profile
            if let Some(profile) = self.profiles.get_mut(&self.active_profile) {
                profile.custom_bindings.insert((state.context, state.action), binding);
            }
            
            self.remapping_state = None;
            RemappingResult::Success
        } else {
            RemappingResult::NotRemapping
        }
    }
}
```

### Accessibility Features

```rust
pub struct AccessibilitySettings {
    // Input assistance
    pub sticky_keys: bool,
    pub slow_keys: SlowKeySettings,
    pub repeat_keys: RepeatKeySettings,
    pub toggle_keys: ToggleKeySettings,
    
    // Mouse/pointer assistance
    pub mouse_keys: bool,
    pub pointer_precision: PointerPrecisionSettings,
    pub click_assist: ClickAssistSettings,
    
    // Gamepad assistance
    pub button_hold_assist: bool,
    pub adaptive_triggers: bool,
    pub reduced_motion: bool,
    
    // General
    pub one_handed_mode: Option<HandPreference>,
    pub simplified_inputs: bool,
}

pub struct SlowKeySettings {
    pub enabled: bool,
    pub delay: Duration,
    pub audio_feedback: bool,
}

pub struct ClickAssistSettings {
    pub enabled: bool,
    pub dwell_clicking: bool,
    pub dwell_time: Duration,
    pub click_lock: bool,
}

impl AccessibilityInputProcessor {
    pub fn process_input(
        &mut self,
        raw_input: RawInput,
        settings: &AccessibilitySettings,
    ) -> ProcessedInput {
        let mut processed = ProcessedInput::from(raw_input);
        
        if settings.sticky_keys {
            processed = self.apply_sticky_keys(processed);
        }
        
        if settings.slow_keys.enabled {
            processed = self.apply_slow_keys(processed, &settings.slow_keys);
        }
        
        if settings.one_handed_mode.is_some() {
            processed = self.apply_one_handed_remapping(processed, settings.one_handed_mode);
        }
        
        if settings.simplified_inputs {
            processed = self.simplify_inputs(processed);
        }
        
        processed
    }
    
    fn apply_sticky_keys(&mut self, input: ProcessedInput) -> ProcessedInput {
        // Make modifier keys "sticky" - press once to activate, again to deactivate
        if input.is_modifier() {
            self.toggle_sticky_modifier(input.key);
        }
        
        // Apply stuck modifiers to other keys
        let mut modified = input;
        for modifier in &self.stuck_modifiers {
            modified.add_modifier(*modifier);
        }
        
        modified
    }
}
```

### Input Buffering & Combos

```rust
pub struct ComboSystem {
    combo_definitions: HashMap<String, ComboDefinition>,
    active_sequences: Vec<ActiveCombo>,
    combo_window: Duration,
}

pub struct ComboDefinition {
    name: String,
    sequence: Vec<ComboInput>,
    max_delay: Duration,
    action: InputAction,
    context: InputContext,
}

pub struct ComboInput {
    input: InputSource,
    timing: ComboTiming,
}

pub enum ComboTiming {
    Simultaneous,
    Sequential { max_delay: Duration },
    Hold { duration: Duration },
}

impl ComboSystem {
    pub fn process_input(&mut self, input: &InputSource, timestamp: Instant) -> Option<InputAction> {
        // Update active combos
        self.active_sequences.retain_mut(|combo| {
            combo.update(input, timestamp)
        });
        
        // Check for combo completions
        for combo in &self.active_sequences {
            if combo.is_complete() {
                let definition = &self.combo_definitions[&combo.definition_name];
                return Some(definition.action);
            }
        }
        
        // Start new combo sequences
        for (name, definition) in &self.combo_definitions {
            if definition.matches_start(input) {
                self.active_sequences.push(ActiveCombo::new(name.clone(), timestamp));
            }
        }
        
        None
    }
}

// Advanced input buffer for fighting game style inputs
pub struct FightingGameBuffer {
    buffer: CircularBuffer<TimedInput>,
    buffer_size: usize,
    input_window: Duration,
}

impl FightingGameBuffer {
    pub fn check_motion(&self, motion: MotionInput) -> bool {
        match motion {
            MotionInput::QuarterCircleForward => {
                self.check_sequence(&[
                    DirectionInput::Down,
                    DirectionInput::DownForward,
                    DirectionInput::Forward,
                ])
            }
            MotionInput::DragonPunch => {
                self.check_sequence(&[
                    DirectionInput::Forward,
                    DirectionInput::Down,
                    DirectionInput::DownForward,
                ])
            }
            MotionInput::FullCircle => {
                self.check_sequence(&[
                    DirectionInput::Forward,
                    DirectionInput::DownForward,
                    DirectionInput::Down,
                    DirectionInput::DownBack,
                    DirectionInput::Back,
                    DirectionInput::UpBack,
                    DirectionInput::Up,
                    DirectionInput::UpForward,
                    DirectionInput::Forward,
                ])
            }
        }
    }
}
```

### Context Management

```rust
pub struct InputContextManager {
    context_stack: Vec<InputContext>,
    context_priorities: HashMap<InputContext, u32>,
    exclusive_contexts: HashSet<InputContext>,
}

impl InputContextManager {
    pub fn push_context(&mut self, context: InputContext) {
        if self.exclusive_contexts.contains(&context) {
            // Clear stack for exclusive contexts
            self.context_stack.clear();
        }
        self.context_stack.push(context);
        self.sort_by_priority();
    }
    
    pub fn pop_context(&mut self) -> Option<InputContext> {
        self.context_stack.pop()
    }
    
    pub fn is_context_active(&self, context: InputContext) -> bool {
        self.context_stack.contains(&context)
    }
    
    fn sort_by_priority(&mut self) {
        self.context_stack.sort_by_key(|ctx| {
            self.context_priorities.get(ctx).copied().unwrap_or(0)
        });
    }
}
```

### Platform-Specific Implementation

```rust
#[cfg(target_os = "windows")]
mod windows {
    use winapi::um::xinput::{XInputGetState, XINPUT_STATE};
    
    pub fn get_controller_state(index: u32) -> Option<ControllerState> {
        unsafe {
            let mut state = XINPUT_STATE::default();
            if XInputGetState(index, &mut state) == 0 {
                Some(ControllerState::from_xinput(state))
            } else {
                None
            }
        }
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use game_controller_sys::GCController;
    
    pub fn get_controller_state(index: u32) -> Option<ControllerState> {
        // macOS Game Controller framework
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use evdev::{Device, InputEventKind};
    
    pub fn get_controller_state(index: u32) -> Option<ControllerState> {
        // Linux evdev
    }
}
```

### Input Recording & Playback

```rust
pub struct InputRecorder {
    recording: Option<InputRecording>,
    playback: Option<InputPlayback>,
}

pub struct InputRecording {
    start_time: Instant,
    events: Vec<RecordedInput>,
    metadata: RecordingMetadata,
}

pub struct RecordedInput {
    timestamp: Duration,
    input: InputEvent,
    context: InputContext,
}

impl InputRecorder {
    pub fn start_recording(&mut self, metadata: RecordingMetadata) {
        self.recording = Some(InputRecording {
            start_time: Instant::now(),
            events: Vec::new(),
            metadata,
        });
    }
    
    pub fn record_input(&mut self, input: InputEvent, context: InputContext) {
        if let Some(recording) = &mut self.recording {
            let timestamp = recording.start_time.elapsed();
            recording.events.push(RecordedInput {
                timestamp,
                input,
                context,
            });
        }
    }
    
    pub fn start_playback(&mut self, recording: InputRecording) {
        self.playback = Some(InputPlayback {
            recording,
            current_index: 0,
            start_time: Instant::now(),
            speed_multiplier: 1.0,
        });
    }
}
```

## Integration with Game Systems

```rust
// Integration with camera system
impl CameraControlSystem {
    fn handle_input(&mut self, input_manager: &mut InputManager) {
        if input_manager.consume_action(InputAction::CameraMove(Direction2D::Up)) {
            self.camera.translate(Vec3::Y * self.move_speed);
        }
        
        if let Some(zoom) = input_manager.get_axis_value(InputAxis::CameraZoom) {
            self.camera.zoom(zoom * self.zoom_speed);
        }
    }
}

// Integration with UI system
impl UIInputHandler {
    fn handle_navigation(&mut self, input_manager: &InputManager) {
        if input_manager.is_action_pressed(InputAction::UINavigate(Direction2D::Down)) {
            self.focus_next_element();
        }
        
        if input_manager.is_action_pressed(InputAction::UIConfirm) {
            self.activate_focused_element();
        }
    }
}
```

## Performance Considerations

- Input processing runs at fixed 120Hz for consistency
- Gesture recognition uses spatial hashing for multi-touch
- Combo detection uses finite state machines for efficiency
- Input events are pooled to reduce allocations
- Platform-specific code is conditionally compiled

## Testing

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_combo_detection() {
        let mut combo_system = ComboSystem::new();
        combo_system.add_combo(ComboDefinition {
            name: "dash".to_string(),
            sequence: vec![
                ComboInput {
                    input: InputSource::Keyboard(KeyCode::Right),
                    timing: ComboTiming::Sequential { max_delay: Duration::from_millis(200) },
                },
                ComboInput {
                    input: InputSource::Keyboard(KeyCode::Right),
                    timing: ComboTiming::Sequential { max_delay: Duration::from_millis(200) },
                },
            ],
            action: InputAction::Dash,
            context: InputContext::Gameplay,
        });
        
        let now = Instant::now();
        combo_system.process_input(&InputSource::Keyboard(KeyCode::Right), now);
        assert!(combo_system.process_input(
            &InputSource::Keyboard(KeyCode::Right), 
            now + Duration::from_millis(100)
        ).is_some());
    }
}
```