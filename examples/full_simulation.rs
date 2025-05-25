//! Full simulation example with predator/prey dynamics and resource management

use bevy::prelude::*;
use bevy::app::AppExit;
use creature_simulation::{
    components::*,
    plugins::*,
    simulation::ResourceType,
};

fn main() {
    App::new()
        // Bevy default plugins (headless for performance)
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: None,
                    ..default()
                })
                .build()
                .disable::<bevy::render::RenderPlugin>()
                .disable::<bevy::winit::WinitPlugin>()
                .disable::<bevy::a11y::AccessibilityPlugin>()
        )
        // Our simulation plugin
        .add_plugins(CreatureSimulationPlugin)
        // Debug plugin for logging
        .add_plugins(DebugPlugin)
        // Setup system
        .add_systems(Startup, setup)
        // Monitor system
        .add_systems(Update, (
            monitor_simulation,
            check_stop_condition,
        ))
        .run();
}

fn setup(mut commands: Commands) {
    // Spawn herbivores
    for i in 0..20 {
        let angle = i as f32 * std::f32::consts::TAU / 20.0;
        let radius = 200.0;
        let x = angle.cos() * radius;
        let y = angle.sin() * radius;
        
        let mut bundle = CreatureBundle::new(Vec2::new(x, y), 0.8);
        bundle.creature_type = CreatureType::Herbivore;
        bundle.max_speed = MaxSpeed(40.0);
        commands.spawn(bundle);
    }
    
    // Spawn carnivores
    for i in 0..5 {
        let angle = i as f32 * std::f32::consts::TAU / 5.0;
        let radius = 100.0;
        let x = angle.cos() * radius;
        let y = angle.sin() * radius;
        
        let mut bundle = CreatureBundle::new(Vec2::new(x, y), 1.2);
        bundle.creature_type = CreatureType::Carnivore;
        bundle.max_speed = MaxSpeed(60.0);
        bundle.health = Health::new(150.0);
        commands.spawn(bundle);
    }
    
    // Spawn omnivores
    for i in 0..10 {
        let x = (i as f32 - 5.0) * 50.0;
        let y = 0.0;
        
        let mut bundle = CreatureBundle::new(Vec2::new(x, y), 1.0);
        bundle.creature_type = CreatureType::Omnivore;
        bundle.max_speed = MaxSpeed(50.0);
        commands.spawn(bundle);
    }
    
    // Spawn food resources in clusters
    for cluster in 0..8 {
        let cluster_x = (cluster as f32 % 4.0 - 1.5) * 200.0;
        let cluster_y = (cluster as f32 / 4.0 - 0.5) * 200.0;
        
        for i in 0..5 {
            let offset_x = (i as f32 % 3.0 - 1.0) * 30.0;
            let offset_y = (i as f32 / 3.0 - 0.5) * 30.0;
            
            commands.spawn(ResourceBundle::new(
                Vec2::new(cluster_x + offset_x, cluster_y + offset_y),
                ResourceType::Food,
                100.0,
            ));
        }
    }
    
    // Spawn water resources
    for i in 0..10 {
        let angle = i as f32 * std::f32::consts::TAU / 10.0;
        let radius = 300.0;
        let x = angle.cos() * radius;
        let y = angle.sin() * radius;
        
        commands.spawn(ResourceBundle::new(
            Vec2::new(x, y),
            ResourceType::Water,
            200.0,
        ));
    }
    
    info!("Simulation started:");
    info!("- 20 herbivores (prey)");
    info!("- 5 carnivores (predators)");
    info!("- 10 omnivores");
    info!("- 40 food resources");
    info!("- 10 water resources");
}

fn monitor_simulation(
    creatures: Query<(&CreatureType, &Health, &Needs, &CreatureState), With<Creature>>,
    resources: Query<&ResourceAmount, With<ResourceMarker>>,
    time: Res<Time>,
    mut timer: Local<Timer>,
) {
    // Initialize timer
    if timer.duration() == std::time::Duration::ZERO {
        *timer = Timer::from_seconds(2.0, TimerMode::Repeating);
    }
    
    timer.tick(time.delta());
    if !timer.just_finished() {
        return;
    }
    
    // Count creatures by type and state
    let mut herbivores = 0;
    let mut carnivores = 0;
    let mut omnivores = 0;
    let mut eating = 0;
    let mut drinking = 0;
    let mut resting = 0;
    let mut fleeing = 0;
    
    for (creature_type, health, needs, state) in creatures.iter() {
        match creature_type {
            CreatureType::Herbivore => herbivores += 1,
            CreatureType::Carnivore => carnivores += 1,
            CreatureType::Omnivore => omnivores += 1,
        }
        
        match state {
            CreatureState::Eating => eating += 1,
            CreatureState::Drinking => drinking += 1,
            CreatureState::Resting => resting += 1,
            CreatureState::Moving { .. } => {
                // Check if fleeing (simplified check)
                if needs.hunger < 0.7 && needs.thirst < 0.7 {
                    fleeing += 1;
                }
            }
            _ => {}
        }
    }
    
    // Count resources
    let total_resources = resources.iter().filter(|r| !r.is_depleted()).count();
    
    info!("=== Simulation Status ===");
    info!("Time: {:.1}s", time.elapsed_seconds());
    info!("Creatures: {} herbivores, {} carnivores, {} omnivores", 
        herbivores, carnivores, omnivores);
    info!("Activities: {} eating, {} drinking, {} resting, {} fleeing",
        eating, drinking, resting, fleeing);
    info!("Resources remaining: {}", total_resources);
}

fn check_stop_condition(
    time: Res<Time>,
    mut exit: EventWriter<AppExit>,
    creatures: Query<Entity, With<Creature>>,
) {
    // Stop after 30 seconds or if all creatures die
    if time.elapsed_seconds() > 30.0 || creatures.is_empty() {
        info!("Simulation complete after {:.1} seconds", time.elapsed_seconds());
        info!("Final creature count: {}", creatures.iter().count());
        exit.send(AppExit);
    }
}