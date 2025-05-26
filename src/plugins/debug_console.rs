//! In-game debug console for runtime tweaking

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use std::collections::VecDeque;

/// Maximum number of console history entries
const MAX_HISTORY: usize = 100;
const MAX_OUTPUT_LINES: usize = 50;

/// Console state resource
#[derive(Resource)]
pub struct DebugConsole {
    /// Is the console visible?
    pub visible: bool,
    /// Current input text
    pub input: String,
    /// Command history
    pub history: VecDeque<String>,
    /// Current history index (for up/down navigation)
    pub history_index: Option<usize>,
    /// Console output
    pub output: VecDeque<ConsoleMessage>,
    /// Available commands
    pub commands: std::collections::HashMap<String, Box<dyn ConsoleCommand>>,
}

#[derive(Clone)]
pub struct ConsoleMessage {
    pub text: String,
    pub level: MessageLevel,
    pub timestamp: f32,
}

#[derive(Clone, Copy, PartialEq)]
pub enum MessageLevel {
    Info,
    Success,
    Warning,
    Error,
}

impl MessageLevel {
    fn color(&self) -> egui::Color32 {
        match self {
            MessageLevel::Info => egui::Color32::GRAY,
            MessageLevel::Success => egui::Color32::from_rgb(100, 200, 100),
            MessageLevel::Warning => egui::Color32::from_rgb(255, 200, 100),
            MessageLevel::Error => egui::Color32::from_rgb(255, 100, 100),
        }
    }
}

/// Trait for console commands
pub trait ConsoleCommand: Send + Sync {
    /// Execute the command with given arguments
    fn execute(&self, args: &[&str], world: &mut World) -> Result<String, String>;
    
    /// Get help text for this command
    fn help(&self) -> &str;
    
    /// Get usage example
    fn usage(&self) -> &str;
    
    /// Clone the command
    fn clone_box(&self) -> Box<dyn ConsoleCommand>;
}

impl Clone for Box<dyn ConsoleCommand> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl Default for DebugConsole {
    fn default() -> Self {
        let mut console = Self {
            visible: false,
            input: String::new(),
            history: VecDeque::with_capacity(MAX_HISTORY),
            history_index: None,
            output: VecDeque::with_capacity(MAX_OUTPUT_LINES),
            commands: std::collections::HashMap::new(),
        };
        
        // Register built-in commands
        console.register_builtin_commands();
        console
    }
}

impl DebugConsole {
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
        if self.visible {
            self.input.clear();
            self.history_index = None;
        }
    }
    
    pub fn log(&mut self, text: impl Into<String>, level: MessageLevel, time: f32) {
        let message = ConsoleMessage {
            text: text.into(),
            level,
            timestamp: time,
        };
        
        self.output.push_back(message);
        while self.output.len() > MAX_OUTPUT_LINES {
            self.output.pop_front();
        }
    }
    
    
    fn register_builtin_commands(&mut self) {
        // Help command
        self.commands.insert("help".to_string(), Box::new(HelpCommand));
        
        // Clear command
        self.commands.insert("clear".to_string(), Box::new(ClearCommand));
        
        // Spawn command
        self.commands.insert("spawn".to_string(), Box::new(SpawnCommand));
        
        // Kill command
        self.commands.insert("kill".to_string(), Box::new(KillCommand));
        
        // Set command for tweaking values
        self.commands.insert("set".to_string(), Box::new(SetCommand));
        
        // Get command for reading values
        self.commands.insert("get".to_string(), Box::new(GetCommand));
        
        // Stats command
        self.commands.insert("stats".to_string(), Box::new(StatsCommand));
        
        // Time command
        self.commands.insert("time".to_string(), Box::new(TimeCommand));
    }
    
    pub fn register_command(&mut self, name: String, command: Box<dyn ConsoleCommand>) {
        self.commands.insert(name, command);
    }
}

// Built-in commands

#[derive(Clone)]
struct HelpCommand;
impl ConsoleCommand for HelpCommand {
    fn execute(&self, args: &[&str], world: &mut World) -> Result<String, String> {
        let console = world.resource::<DebugConsole>();
        
        if args.is_empty() {
            let mut commands: Vec<_> = console.commands.keys().collect();
            commands.sort();
            
            let mut output = String::from("Available commands:\n");
            for cmd in commands {
                output.push_str(&format!("  {}\n", cmd));
            }
            output.push_str("\nType 'help <command>' for more information.");
            Ok(output)
        } else {
            let cmd_name = args[0];
            if let Some(cmd) = console.commands.get(cmd_name) {
                Ok(format!("{}\nUsage: {}", cmd.help(), cmd.usage()))
            } else {
                Err(format!("Unknown command: '{}'", cmd_name))
            }
        }
    }
    
    fn help(&self) -> &str {
        "Display help information for commands"
    }
    
    fn usage(&self) -> &str {
        "help [command]"
    }
    
    fn clone_box(&self) -> Box<dyn ConsoleCommand> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
struct ClearCommand;
impl ConsoleCommand for ClearCommand {
    fn execute(&self, _args: &[&str], world: &mut World) -> Result<String, String> {
        world.resource_mut::<DebugConsole>().output.clear();
        Ok("Console cleared".to_string())
    }
    
    fn help(&self) -> &str {
        "Clear the console output"
    }
    
    fn usage(&self) -> &str {
        "clear"
    }
    
    fn clone_box(&self) -> Box<dyn ConsoleCommand> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
struct SpawnCommand;
impl ConsoleCommand for SpawnCommand {
    fn execute(&self, args: &[&str], world: &mut World) -> Result<String, String> {
        if args.len() < 2 {
            return Err("Usage: spawn <type> <count> [x] [y]".to_string());
        }
        
        let entity_type = args[0];
        let count: usize = args[1].parse()
            .map_err(|_| "Invalid count")?;
        
        let x = args.get(2).and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.0);
        let y = args.get(3).and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.0);
        
        match entity_type {
            "creature" => {
                for i in 0..count {
                    let offset = Vec2::new((i as f32) * 20.0, 0.0);
                    world.spawn(crate::components::CreatureBundle::new(Vec2::new(x, y) + offset, 1.0));
                }
                Ok(format!("Spawned {} creatures at ({}, {})", count, x, y))
            }
            "food" => {
                for i in 0..count {
                    let offset = Vec2::new((i as f32) * 20.0, 0.0);
                    world.spawn(crate::components::ResourceBundle::new(
                        Vec2::new(x, y) + offset,
                        crate::components::ResourceType::Food,
                        50.0
                    ));
                }
                Ok(format!("Spawned {} food at ({}, {})", count, x, y))
            }
            "water" => {
                for i in 0..count {
                    let offset = Vec2::new((i as f32) * 20.0, 0.0);
                    world.spawn(crate::components::ResourceBundle::new(
                        Vec2::new(x, y) + offset,
                        crate::components::ResourceType::Water,
                        50.0
                    ));
                }
                Ok(format!("Spawned {} water at ({}, {})", count, x, y))
            }
            _ => Err(format!("Unknown entity type: '{}'", entity_type))
        }
    }
    
    fn help(&self) -> &str {
        "Spawn entities in the world"
    }
    
    fn usage(&self) -> &str {
        "spawn <creature|food|water> <count> [x] [y]"
    }
    
    fn clone_box(&self) -> Box<dyn ConsoleCommand> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
struct KillCommand;
impl ConsoleCommand for KillCommand {
    fn execute(&self, args: &[&str], world: &mut World) -> Result<String, String> {
        if args.is_empty() {
            return Err("Usage: kill <all|creatures|resources|entity_id>".to_string());
        }
        
        match args[0] {
            "all" => {
                let mut count = 0;
                let entities: Vec<_> = world.iter_entities()
                    .map(|e| e.id())
                    .collect();
                
                for entity in entities {
                    if world.get::<crate::components::Creature>(entity).is_some()
                        || world.get::<crate::components::ResourceMarker>(entity).is_some() {
                        world.despawn(entity);
                        count += 1;
                    }
                }
                Ok(format!("Killed {} entities", count))
            }
            "creatures" => {
                let mut count = 0;
                let entities: Vec<_> = world.iter_entities()
                    .filter(|e| world.get::<crate::components::Creature>(e.id()).is_some())
                    .map(|e| e.id())
                    .collect();
                
                for entity in entities {
                    world.despawn(entity);
                    count += 1;
                }
                Ok(format!("Killed {} creatures", count))
            }
            "resources" => {
                let mut count = 0;
                let entities: Vec<_> = world.iter_entities()
                    .filter(|e| world.get::<crate::components::ResourceMarker>(e.id()).is_some())
                    .map(|e| e.id())
                    .collect();
                
                for entity in entities {
                    world.despawn(entity);
                    count += 1;
                }
                Ok(format!("Killed {} resources", count))
            }
            entity_str => {
                // Try to parse as entity ID
                if let Ok(index) = entity_str.parse::<u32>() {
                    let entity = Entity::from_raw(index);
                    if world.get_entity(entity).is_some() {
                        world.despawn(entity);
                        Ok(format!("Killed entity {:?}", entity))
                    } else {
                        Err(format!("Entity {:?} not found", entity))
                    }
                } else {
                    Err(format!("Invalid entity ID: '{}'", entity_str))
                }
            }
        }
    }
    
    fn help(&self) -> &str {
        "Kill entities in the world"
    }
    
    fn usage(&self) -> &str {
        "kill <all|creatures|resources|entity_id>"
    }
    
    fn clone_box(&self) -> Box<dyn ConsoleCommand> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
struct SetCommand;
impl ConsoleCommand for SetCommand {
    fn execute(&self, args: &[&str], world: &mut World) -> Result<String, String> {
        if args.len() < 2 {
            return Err("Usage: set <variable> <value>".to_string());
        }
        
        let variable = args[0];
        let value_str = args[1];
        
        match variable {
            "timescale" => {
                let value: f32 = value_str.parse()
                    .map_err(|_| "Invalid float value")?;
                
                if let Some(mut control) = world.get_resource_mut::<crate::core::simulation_control::SimulationControl>() {
                    control.speed_multiplier = value;
                    Ok(format!("Time scale set to {}", value))
                } else {
                    Err("SimulationControl resource not found".to_string())
                }
            }
            "creature_speed" => {
                let value: f32 = value_str.parse()
                    .map_err(|_| "Invalid float value")?;
                
                if let Some(mut config) = world.get_resource_mut::<crate::simulation::SimulationConfig>() {
                    config.creature_base_speed = value;
                    Ok(format!("Creature base speed set to {}", value))
                } else {
                    Err("SimulationConfig resource not found".to_string())
                }
            }
            "debug_mode" => {
                let value: bool = value_str.parse()
                    .map_err(|_| "Invalid boolean value (use true/false)")?;
                
                if let Some(mut config) = world.get_resource_mut::<crate::simulation::SimulationConfig>() {
                    config.debug_mode = value;
                    Ok(format!("Debug mode set to {}", value))
                } else {
                    Err("SimulationConfig resource not found".to_string())
                }
            }
            _ => Err(format!("Unknown variable: '{}'. Available: timescale, creature_speed, debug_mode", variable))
        }
    }
    
    fn help(&self) -> &str {
        "Set runtime configuration values"
    }
    
    fn usage(&self) -> &str {
        "set <variable> <value>"
    }
    
    fn clone_box(&self) -> Box<dyn ConsoleCommand> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
struct GetCommand;
impl ConsoleCommand for GetCommand {
    fn execute(&self, args: &[&str], world: &mut World) -> Result<String, String> {
        if args.is_empty() {
            return Err("Usage: get <variable>".to_string());
        }
        
        let variable = args[0];
        
        match variable {
            "timescale" => {
                if let Some(control) = world.get_resource::<crate::core::simulation_control::SimulationControl>() {
                    Ok(format!("Time scale: {}", control.speed_multiplier))
                } else {
                    Err("SimulationControl resource not found".to_string())
                }
            }
            "creature_speed" => {
                if let Some(config) = world.get_resource::<crate::simulation::SimulationConfig>() {
                    Ok(format!("Creature base speed: {}", config.creature_base_speed))
                } else {
                    Err("SimulationConfig resource not found".to_string())
                }
            }
            "debug_mode" => {
                if let Some(config) = world.get_resource::<crate::simulation::SimulationConfig>() {
                    Ok(format!("Debug mode: {}", config.debug_mode))
                } else {
                    Err("SimulationConfig resource not found".to_string())
                }
            }
            _ => Err(format!("Unknown variable: '{}'", variable))
        }
    }
    
    fn help(&self) -> &str {
        "Get runtime configuration values"
    }
    
    fn usage(&self) -> &str {
        "get <variable>"
    }
    
    fn clone_box(&self) -> Box<dyn ConsoleCommand> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
struct StatsCommand;
impl ConsoleCommand for StatsCommand {
    fn execute(&self, _args: &[&str], world: &mut World) -> Result<String, String> {
        let creature_count = world.iter_entities()
            .filter(|e| world.get::<crate::components::Creature>(e.id()).is_some())
            .count();
            
        let food_count = world.iter_entities()
            .filter(|e| {
                if let Some(res_type) = world.get::<crate::components::ResourceTypeComponent>(e.id()) {
                    res_type.0 == crate::components::ResourceType::Food
                } else {
                    false
                }
            })
            .count();
            
        let water_count = world.iter_entities()
            .filter(|e| {
                if let Some(res_type) = world.get::<crate::components::ResourceTypeComponent>(e.id()) {
                    res_type.0 == crate::components::ResourceType::Water
                } else {
                    false
                }
            })
            .count();
            
        let total_entities = world.iter_entities().count();
        
        Ok(format!(
            "World Statistics:\n  Creatures: {}\n  Food: {}\n  Water: {}\n  Total entities: {}",
            creature_count, food_count, water_count, total_entities
        ))
    }
    
    fn help(&self) -> &str {
        "Display world statistics"
    }
    
    fn usage(&self) -> &str {
        "stats"
    }
    
    fn clone_box(&self) -> Box<dyn ConsoleCommand> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
struct TimeCommand;
impl ConsoleCommand for TimeCommand {
    fn execute(&self, args: &[&str], world: &mut World) -> Result<String, String> {
        if args.is_empty() {
            if let Some(time) = world.get_resource::<Time>() {
                Ok(format!(
                    "Elapsed time: {:.1}s\nDelta time: {:.3}s\nFPS: {:.1}",
                    time.elapsed_seconds(),
                    time.delta_seconds(),
                    1.0 / time.delta_seconds()
                ))
            } else {
                Err("Time resource not found".to_string())
            }
        } else {
            match args[0] {
                "pause" => {
                    if let Some(mut control) = world.get_resource_mut::<crate::core::simulation_control::SimulationControl>() {
                        control.paused = true;
                        Ok("Time paused".to_string())
                    } else {
                        Err("SimulationControl resource not found".to_string())
                    }
                }
                "resume" => {
                    if let Some(mut control) = world.get_resource_mut::<crate::core::simulation_control::SimulationControl>() {
                        control.paused = false;
                        control.speed_multiplier = 1.0;
                        Ok("Time resumed".to_string())
                    } else {
                        Err("SimulationControl resource not found".to_string())
                    }
                }
                speed_str => {
                    if let Ok(speed) = speed_str.parse::<f32>() {
                        if let Some(mut control) = world.get_resource_mut::<crate::core::simulation_control::SimulationControl>() {
                            control.speed_multiplier = speed;
                            Ok(format!("Time scale set to {}x", speed))
                        } else {
                            Err("SimulationControl resource not found".to_string())
                        }
                    } else {
                        Err(format!("Invalid time speed: '{}'", speed_str))
                    }
                }
            }
        }
    }
    
    fn help(&self) -> &str {
        "Control simulation time"
    }
    
    fn usage(&self) -> &str {
        "time [pause|resume|<speed>]"
    }
    
    fn clone_box(&self) -> Box<dyn ConsoleCommand> {
        Box::new(self.clone())
    }
}

/// Command to execute from the console
#[derive(Event, Clone)]
pub struct ConsoleCommandEvent {
    pub command: String,
}

/// System to render the debug console UI
pub fn debug_console_ui(
    mut contexts: EguiContexts,
    mut console: ResMut<DebugConsole>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut command_events: EventWriter<ConsoleCommandEvent>,
) {
    // Toggle console with tilde key
    if keyboard.just_pressed(KeyCode::Backquote) {
        console.toggle();
    }
    
    if !console.visible {
        return;
    }
    
    let ctx = contexts.ctx_mut();
    
    egui::Window::new("Debug Console")
        .collapsible(false)
        .resizable(true)
        .default_width(600.0)
        .default_height(400.0)
        .show(ctx, |ui| {
            // Output area
            egui::ScrollArea::vertical()
                .max_height(300.0)
                .show(ui, |ui| {
                    for message in &console.output {
                        ui.colored_label(message.level.color(), &message.text);
                    }
                });
            
            ui.separator();
            
            // Input area
            let response = ui.text_edit_singleline(&mut console.input);
            
            // Focus on input when console opens
            if console.visible {
                response.request_focus();
            }
            
            // Handle input
            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                let input = console.input.clone();
                console.input.clear();
                
                // Send command event for execution
                if !input.trim().is_empty() {
                    command_events.send(ConsoleCommandEvent {
                        command: input,
                    });
                }
            }
            
            // History navigation
            if response.has_focus() {
                if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                    if !console.history.is_empty() {
                        console.history_index = Some(
                            console.history_index
                                .map(|i| i.saturating_sub(1))
                                .unwrap_or(console.history.len() - 1)
                        );
                        
                        if let Some(index) = console.history_index {
                            if let Some(cmd) = console.history.get(index) {
                                console.input = cmd.clone();
                            }
                        }
                    }
                }
                
                if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                    if let Some(index) = console.history_index {
                        if index < console.history.len() - 1 {
                            console.history_index = Some(index + 1);
                            if let Some(cmd) = console.history.get(index + 1) {
                                console.input = cmd.clone();
                            }
                        } else {
                            console.history_index = None;
                            console.input.clear();
                        }
                    }
                }
            }
        });
}

/// Resource to hold pending commands
#[derive(Resource, Default)]
pub struct PendingCommands {
    commands: Vec<String>,
}

/// System to collect console commands
pub fn collect_console_commands(
    mut pending: ResMut<PendingCommands>,
    mut command_events: EventReader<ConsoleCommandEvent>,
) {
    for event in command_events.read() {
        pending.commands.push(event.command.clone());
    }
}

/// System to execute console commands (exclusive system)
pub fn execute_console_commands(
    world: &mut World,
) {
    // Get pending commands
    let commands: Vec<String> = world.resource_mut::<PendingCommands>()
        .commands
        .drain(..)
        .collect();
    
    if commands.is_empty() {
        return;
    }
    
    let time = world.resource::<Time>().elapsed_seconds();
    
    // Process each command
    for command in commands {
        // Log and parse the command
        {
            let mut console = world.resource_mut::<DebugConsole>();
            console.log(format!("> {}", command), MessageLevel::Info, time);
            
            // Add to history
            if !command.trim().is_empty() {
                console.history.push_back(command.clone());
                while console.history.len() > MAX_HISTORY {
                    console.history.pop_front();
                }
            }
        }
        
        // Parse command
        let parts: Vec<&str> = command.trim().split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        
        let command_name = parts[0];
        let args = &parts[1..];
        
        // Get the command handler
        let command_handler = world.resource::<DebugConsole>()
            .commands
            .get(command_name)
            .cloned();
        
        // Execute command
        if let Some(handler) = command_handler {
            match handler.execute(args, world) {
                Ok(result) => {
                    world.resource_mut::<DebugConsole>()
                        .log(result, MessageLevel::Success, time);
                }
                Err(error) => {
                    world.resource_mut::<DebugConsole>()
                        .log(error, MessageLevel::Error, time);
                }
            }
        } else {
            world.resource_mut::<DebugConsole>().log(
                format!("Unknown command: '{}'. Type 'help' for available commands.", command_name),
                MessageLevel::Error,
                time
            );
        }
    }
}

/// Plugin for the debug console
pub struct DebugConsolePlugin;

impl Plugin for DebugConsolePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugConsole>()
            .init_resource::<PendingCommands>()
            .add_event::<ConsoleCommandEvent>()
            .add_systems(Update, (debug_console_ui, collect_console_commands).chain())
            .add_systems(Last, execute_console_commands);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_console_message_levels() {
        let info = MessageLevel::Info;
        let success = MessageLevel::Success;
        let warning = MessageLevel::Warning;
        let error = MessageLevel::Error;
        
        // Just verify colors are different
        assert_ne!(info.color(), success.color());
        assert_ne!(warning.color(), error.color());
    }
    
    #[test]
    fn test_console_toggle() {
        let mut console = DebugConsole::default();
        assert!(!console.visible);
        
        console.toggle();
        assert!(console.visible);
        
        console.toggle();
        assert!(!console.visible);
    }
    
    #[test]
    fn test_console_log() {
        let mut console = DebugConsole::default();
        
        console.log("Test message", MessageLevel::Info, 0.0);
        assert_eq!(console.output.len(), 1);
        assert_eq!(console.output[0].text, "Test message");
        
        // Test max output lines
        for i in 0..MAX_OUTPUT_LINES + 10 {
            console.log(format!("Message {}", i), MessageLevel::Info, i as f32);
        }
        assert_eq!(console.output.len(), MAX_OUTPUT_LINES);
    }
    
    #[test]
    fn test_help_command() {
        let mut world = World::new();
        world.init_resource::<DebugConsole>();
        
        let help = HelpCommand;
        let result = help.execute(&[], &mut world);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Available commands"));
        
        // Test help for specific command
        let result = help.execute(&["clear"], &mut world);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_clear_command() {
        let mut world = World::new();
        let mut console = DebugConsole::default();
        
        // Add some messages
        console.log("Message 1", MessageLevel::Info, 0.0);
        console.log("Message 2", MessageLevel::Info, 0.0);
        assert_eq!(console.output.len(), 2);
        
        world.insert_resource(console);
        
        let clear = ClearCommand;
        let result = clear.execute(&[], &mut world);
        assert!(result.is_ok());
        
        let console = world.resource::<DebugConsole>();
        assert_eq!(console.output.len(), 0);
    }
    
    #[test]
    fn test_command_history() {
        let mut console = DebugConsole::default();
        
        // Manually add to history like execute_command would
        console.history.push_back("help".to_string());
        console.history.push_back("clear".to_string());
        
        assert_eq!(console.history.len(), 2);
        assert_eq!(console.history[0], "help");
        assert_eq!(console.history[1], "clear");
        
        // Test max history
        for i in 0..MAX_HISTORY + 10 {
            console.history.push_back(format!("command{}", i));
        }
        
        // Trim history
        while console.history.len() > MAX_HISTORY {
            console.history.pop_front();
        }
        
        assert_eq!(console.history.len(), MAX_HISTORY);
    }
}