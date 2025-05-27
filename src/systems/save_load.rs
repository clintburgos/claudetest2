//! Save/Load system for Phase 1
//!
//! This implements basic save/load functionality for the simulation state.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Save game data structure
#[derive(Serialize, Deserialize)]
pub struct SaveGame {
    pub version: u32,
    pub timestamp: f64,
    pub seed: u64,
    pub frame: u64,
    pub creatures: Vec<CreatureSaveData>,
    pub resources: Vec<ResourceSaveData>,
    pub camera: CameraSaveData,
    pub simulation_speed: f32,
}

#[derive(Serialize, Deserialize)]
pub struct CreatureSaveData {
    pub position: (f32, f32),
    pub velocity: (f32, f32),
    pub health: f32,
    pub max_health: f32,
    pub hunger: f32,
    pub thirst: f32,
    pub energy: f32,
    pub social: f32,
    pub age: f32,
    pub creature_type: String,
    pub state: String,
    pub size: f32,
    pub max_speed: f32,
}

#[derive(Serialize, Deserialize)]
pub struct ResourceSaveData {
    pub position: (f32, f32),
    pub resource_type: String,
    pub amount: f32,
}

#[derive(Serialize, Deserialize)]
pub struct CameraSaveData {
    pub position: (f32, f32, f32),
    pub zoom: f32,
}

/// Resource for managing save/load operations
#[derive(Resource, Default)]
pub struct SaveLoadState {
    pub save_requested: bool,
    pub load_requested: bool,
    pub save_path: Option<PathBuf>,
    pub last_save_time: Option<f64>,
    pub auto_save_timer: f32,
    pub auto_save_interval: f32,
}

impl SaveLoadState {
    pub fn new() -> Self {
        Self {
            auto_save_interval: 300.0, // Auto-save every 5 minutes
            ..default()
        }
    }
}

/// Plugin for save/load functionality
pub struct SaveLoadPlugin;

impl Plugin for SaveLoadPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SaveLoadState::new())
            .add_systems(Update, (
                handle_save_input,
                handle_load_input,
                auto_save_system,
                process_save_request,
                process_load_request,
            ).chain());
    }
}

/// Handle save hotkey (F5)
fn handle_save_input(
    keyboard: Option<Res<ButtonInput<KeyCode>>>,
    mut save_state: ResMut<SaveLoadState>,
) {
    let Some(keyboard) = keyboard else { return };
    
    if keyboard.just_pressed(KeyCode::F5) {
        save_state.save_requested = true;
        save_state.save_path = Some(get_save_path("quicksave.json"));
        info!("Save requested");
    }
}

/// Handle load hotkey (F8)
fn handle_load_input(
    keyboard: Option<Res<ButtonInput<KeyCode>>>,
    mut save_state: ResMut<SaveLoadState>,
) {
    let Some(keyboard) = keyboard else { return };
    
    if keyboard.just_pressed(KeyCode::F8) {
        save_state.load_requested = true;
        save_state.save_path = Some(get_save_path("quicksave.json"));
        info!("Load requested");
    }
}

/// Auto-save system
fn auto_save_system(
    time: Res<Time>,
    mut save_state: ResMut<SaveLoadState>,
) {
    save_state.auto_save_timer += time.delta_seconds();
    
    if save_state.auto_save_timer >= save_state.auto_save_interval {
        save_state.auto_save_timer = 0.0;
        save_state.save_requested = true;
        save_state.save_path = Some(get_save_path("autosave.json"));
        info!("Auto-save triggered");
    }
}

/// Process save request
fn process_save_request(
    mut save_state: ResMut<SaveLoadState>,
    time: Res<Time>,
    rng: Res<crate::core::determinism::DeterministicRng>,
    control: Res<crate::core::simulation_control::SimulationControl>,
    creatures: Query<(
        &crate::components::Position,
        &crate::components::Velocity,
        &crate::components::Health,
        &crate::components::Needs,
        &crate::components::Age,
        &crate::components::CreatureType,
        &crate::components::CreatureState,
        &crate::components::Size,
        &crate::components::MaxSpeed,
    ), With<crate::components::Creature>>,
    resources: Query<(
        &crate::components::Position,
        &crate::components::ResourceTypeComponent,
        &crate::components::ResourceAmount,
    ), With<crate::components::ResourceMarker>>,
    camera: Query<&Transform, With<crate::plugins::MainCamera>>,
    camera_state: Res<crate::plugins::CameraState>,
) {
    if !save_state.save_requested {
        return;
    }
    
    let Some(path) = &save_state.save_path else {
        save_state.save_requested = false;
        return;
    };
    
    // Collect creature data
    let creatures_data: Vec<CreatureSaveData> = creatures.iter()
        .map(|(pos, vel, health, needs, age, c_type, state, size, max_speed)| {
            CreatureSaveData {
                position: (pos.0.x, pos.0.y),
                velocity: (vel.0.x, vel.0.y),
                health: health.current,
                max_health: health.max,
                hunger: needs.hunger,
                thirst: needs.thirst,
                energy: needs.energy,
                social: needs.social,
                age: age.0,
                creature_type: format!("{:?}", c_type),
                state: format!("{:?}", state),
                size: size.0,
                max_speed: max_speed.0,
            }
        })
        .collect();
    
    // Collect resource data
    let resources_data: Vec<ResourceSaveData> = resources.iter()
        .map(|(pos, r_type, amount)| {
            ResourceSaveData {
                position: (pos.0.x, pos.0.y),
                resource_type: format!("{:?}", r_type.0),
                amount: amount.current,
            }
        })
        .collect();
    
    // Get camera data
    let camera_data = camera.get_single()
        .map(|transform| CameraSaveData {
            position: (transform.translation.x, transform.translation.y, transform.translation.z),
            zoom: camera_state.zoom,
        })
        .unwrap_or(CameraSaveData {
            position: (0.0, 0.0, 999.9),
            zoom: 1.0,
        });
    
    // Create save game
    let save_game = SaveGame {
        version: 1,
        timestamp: time.elapsed_seconds_f64(),
        seed: rng.seed(),
        frame: rng.frame_count(),
        creatures: creatures_data,
        resources: resources_data,
        camera: camera_data,
        simulation_speed: control.speed_multiplier,
    };
    
    // Serialize and save
    match serde_json::to_string_pretty(&save_game) {
        Ok(json) => {
            match fs::write(path, json) {
                Ok(_) => {
                    info!("Game saved to {:?}", path);
                    save_state.last_save_time = Some(time.elapsed_seconds_f64());
                }
                Err(e) => {
                    error!("Failed to write save file: {}", e);
                }
            }
        }
        Err(e) => {
            error!("Failed to serialize save data: {}", e);
        }
    }
    
    save_state.save_requested = false;
}

/// Process load request
fn process_load_request(
    mut save_state: ResMut<SaveLoadState>,
    mut commands: Commands,
    creatures: Query<Entity, With<crate::components::Creature>>,
    resources: Query<Entity, With<crate::components::ResourceMarker>>,
    mut camera: Query<&mut Transform, With<crate::plugins::MainCamera>>,
    mut camera_state: ResMut<crate::plugins::CameraState>,
    mut control: ResMut<crate::core::simulation_control::SimulationControl>,
    mut rng: ResMut<crate::core::determinism::DeterministicRng>,
) {
    if !save_state.load_requested {
        return;
    }
    
    let Some(path) = &save_state.save_path else {
        save_state.load_requested = false;
        return;
    };
    
    // Read save file
    let json = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => {
            error!("Failed to read save file: {}", e);
            save_state.load_requested = false;
            return;
        }
    };
    
    // Deserialize
    let save_game: SaveGame = match serde_json::from_str(&json) {
        Ok(save) => save,
        Err(e) => {
            error!("Failed to deserialize save data: {}", e);
            save_state.load_requested = false;
            return;
        }
    };
    
    // Clear existing entities
    for entity in creatures.iter() {
        commands.entity(entity).despawn();
    }
    for entity in resources.iter() {
        commands.entity(entity).despawn();
    }
    
    // Spawn creatures from save data
    for creature_data in save_game.creatures {
        use crate::components::*;
        
        let creature_type = match creature_data.creature_type.as_str() {
            "Herbivore" => CreatureType::Herbivore,
            "Carnivore" => CreatureType::Carnivore,
            _ => CreatureType::Omnivore,
        };
        
        commands.spawn((
            CreatureBundle {
                creature: Creature,
                creature_type,
                position: Position(Vec2::new(creature_data.position.0, creature_data.position.1)),
                velocity: Velocity(Vec2::new(creature_data.velocity.0, creature_data.velocity.1)),
                health: Health { current: creature_data.health, max: creature_data.max_health },
                needs: Needs {
                    hunger: creature_data.hunger,
                    thirst: creature_data.thirst,
                    energy: creature_data.energy,
                    social: creature_data.social,
                },
                age: Age(creature_data.age),
                size: Size(creature_data.size),
                genetics: Genetics::default(), // TODO: Save/load genetics data
                max_speed: MaxSpeed(creature_data.max_speed),
                state: CreatureState::Idle, // Simplified for now
                decision_timer: DecisionTimer::default(),
                current_target: CurrentTarget::None,
            },
            SpriteBundle::default(),
            crate::plugins::CreatureSprite,
        ));
    }
    
    // Spawn resources from save data
    for resource_data in save_game.resources {
        use crate::components::*;
        
        let resource_type = if resource_data.resource_type.contains("Food") {
            ResourceType::Food
        } else {
            ResourceType::Water
        };
        
        commands.spawn((
            ResourceBundle {
                resource: ResourceMarker,
                position: Position(Vec2::new(resource_data.position.0, resource_data.position.1)),
                resource_type: ResourceTypeComponent(resource_type),
                amount: ResourceAmount::new(resource_data.amount),
            },
            SpriteBundle::default(),
            crate::plugins::ResourceSprite {
                resource_type,
            },
        ));
    }
    
    // Restore camera
    if let Ok(mut transform) = camera.get_single_mut() {
        transform.translation = Vec3::new(
            save_game.camera.position.0,
            save_game.camera.position.1,
            save_game.camera.position.2,
        );
        camera_state.zoom = save_game.camera.zoom;
    }
    
    // Restore simulation state
    control.speed_multiplier = save_game.simulation_speed;
    // Note: Can't change seed after creation, would need to recreate RNG
    rng.set_frame_count(save_game.frame);
    
    info!("Game loaded from {:?}", path);
    save_state.load_requested = false;
}

/// Get the save file path
fn get_save_path(filename: &str) -> PathBuf {
    // Create saves directory if it doesn't exist
    let saves_dir = PathBuf::from("saves");
    if !saves_dir.exists() {
        let _ = fs::create_dir(&saves_dir);
    }
    
    saves_dir.join(filename)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_save_load_state_creation() {
        let state = SaveLoadState::new();
        assert!(!state.save_requested);
        assert!(!state.load_requested);
        assert_eq!(state.auto_save_interval, 300.0);
        assert_eq!(state.auto_save_timer, 0.0);
    }
    
    #[test]
    fn test_save_game_serialization() {
        let save_game = SaveGame {
            version: 1,
            timestamp: 100.0,
            seed: 12345,
            frame: 1000,
            creatures: vec![
                CreatureSaveData {
                    position: (10.0, 20.0),
                    velocity: (1.0, 2.0),
                    health: 80.0,
                    max_health: 100.0,
                    hunger: 0.3,
                    thirst: 0.4,
                    energy: 0.7,
                    social: 0.5,
                    age: 50.0,
                    creature_type: "Herbivore".to_string(),
                    state: "Idle".to_string(),
                    size: 1.0,
                    max_speed: 50.0,
                }
            ],
            resources: vec![
                ResourceSaveData {
                    position: (30.0, 40.0),
                    resource_type: "Food".to_string(),
                    amount: 75.0,
                }
            ],
            camera: CameraSaveData {
                position: (0.0, 0.0, 999.9),
                zoom: 1.5,
            },
            simulation_speed: 2.0,
        };
        
        // Test serialization
        let json = serde_json::to_string(&save_game).unwrap();
        assert!(json.contains("\"version\":1"));
        assert!(json.contains("\"seed\":12345"));
        
        // Test deserialization
        let loaded: SaveGame = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.version, save_game.version);
        assert_eq!(loaded.seed, save_game.seed);
        assert_eq!(loaded.creatures.len(), 1);
        assert_eq!(loaded.resources.len(), 1);
    }
    
    #[test]
    fn test_auto_save_timer() {
        let mut save_state = SaveLoadState::new();
        
        // Simulate time passing
        save_state.auto_save_timer = 299.0;
        
        // One more second should trigger auto-save
        save_state.auto_save_timer += 1.0;
        
        // Check if timer exceeded interval
        assert!(save_state.auto_save_timer >= save_state.auto_save_interval);
    }
    
    #[test]
    fn test_save_path_creation() {
        let path = get_save_path("test_save.json");
        assert_eq!(path, PathBuf::from("saves/test_save.json"));
    }
    
    #[test]
    fn test_creature_save_data_conversion() {
        let creature_data = CreatureSaveData {
            position: (100.0, 200.0),
            velocity: (5.0, -5.0),
            health: 90.0,
            max_health: 100.0,
            hunger: 0.2,
            thirst: 0.3,
            energy: 0.8,
            social: 0.6,
            age: 100.0,
            creature_type: "Carnivore".to_string(),
            state: "Hunting".to_string(),
            size: 1.2,
            max_speed: 60.0,
        };
        
        // Test that all fields are properly set
        assert_eq!(creature_data.position.0, 100.0);
        assert_eq!(creature_data.position.1, 200.0);
        assert_eq!(creature_data.health, 90.0);
        assert_eq!(creature_data.creature_type, "Carnivore");
    }
    
    #[test]
    fn test_resource_save_data_conversion() {
        let resource_data = ResourceSaveData {
            position: (50.0, 75.0),
            resource_type: "Water".to_string(),
            amount: 100.0,
        };
        
        assert_eq!(resource_data.position.0, 50.0);
        assert_eq!(resource_data.position.1, 75.0);
        assert_eq!(resource_data.resource_type, "Water");
        assert_eq!(resource_data.amount, 100.0);
    }
    
    #[test]
    fn test_save_game_with_multiple_entities() {
        let save_game = SaveGame {
            version: 1,
            timestamp: 0.0,
            seed: 999,
            frame: 0,
            creatures: vec![
                CreatureSaveData {
                    position: (0.0, 0.0),
                    velocity: (0.0, 0.0),
                    health: 100.0,
                    max_health: 100.0,
                    hunger: 0.0,
                    thirst: 0.0,
                    energy: 1.0,
                    social: 0.5,
                    age: 0.0,
                    creature_type: "Herbivore".to_string(),
                    state: "Idle".to_string(),
                    size: 1.0,
                    max_speed: 50.0,
                },
                CreatureSaveData {
                    position: (10.0, 10.0),
                    velocity: (1.0, 1.0),
                    health: 80.0,
                    max_health: 100.0,
                    hunger: 0.5,
                    thirst: 0.5,
                    energy: 0.5,
                    social: 0.5,
                    age: 10.0,
                    creature_type: "Carnivore".to_string(),
                    state: "Moving".to_string(),
                    size: 1.5,
                    max_speed: 70.0,
                },
            ],
            resources: vec![
                ResourceSaveData {
                    position: (20.0, 20.0),
                    resource_type: "Food".to_string(),
                    amount: 50.0,
                },
                ResourceSaveData {
                    position: (30.0, 30.0),
                    resource_type: "Water".to_string(),
                    amount: 75.0,
                },
            ],
            camera: CameraSaveData {
                position: (0.0, 0.0, 999.9),
                zoom: 1.0,
            },
            simulation_speed: 1.0,
        };
        
        assert_eq!(save_game.creatures.len(), 2);
        assert_eq!(save_game.resources.len(), 2);
        
        // Verify different creature types
        assert_eq!(save_game.creatures[0].creature_type, "Herbivore");
        assert_eq!(save_game.creatures[1].creature_type, "Carnivore");
        
        // Verify different resource types
        assert_eq!(save_game.resources[0].resource_type, "Food");
        assert_eq!(save_game.resources[1].resource_type, "Water");
    }
}