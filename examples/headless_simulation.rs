//! Truly headless simulation example

use bevy::app::{AppExit, ScheduleRunnerPlugin};
use bevy::core::TaskPoolPlugin;
use bevy::hierarchy::HierarchyPlugin;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::time::TimePlugin;
use bevy::transform::TransformPlugin;
use std::time::Duration;

use creature_simulation::{components::*, plugins::*};

fn main() {
    App::new()
        // Minimal plugins for headless operation
        .add_plugins((
            TaskPoolPlugin::default(),
            TimePlugin,
            TransformPlugin,
            HierarchyPlugin,
            LogPlugin::default(),
            ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1.0 / 60.0)),
        ))
        // Our simulation plugins
        .add_plugins(CreatureSimulationPlugin)
        .add_plugins(DebugPlugin)
        // Setup
        .add_systems(Startup, setup)
        // Monitor
        .add_systems(Update, (
            monitor_simulation,
            check_stop_condition,
        ))
        .run();
}

fn setup(mut commands: Commands) {
    // Spawn herbivores
    for i in 0..10 {
        let angle = i as f32 * std::f32::consts::TAU / 10.0;
        let x = angle.cos() * 150.0;
        let y = angle.sin() * 150.0;

        let mut bundle = CreatureBundle::new(Vec2::new(x, y), 0.8);
        bundle.creature_type = CreatureType::Herbivore;
        commands.spawn(bundle);
    }

    // Spawn carnivores
    for i in 0..3 {
        let x = (i as f32 - 1.0) * 100.0;

        let mut bundle = CreatureBundle::new(Vec2::new(x, 0.0), 1.2);
        bundle.creature_type = CreatureType::Carnivore;
        bundle.max_speed = MaxSpeed(60.0);
        commands.spawn(bundle);
    }

    // Spawn resources
    for i in 0..20 {
        let x = (i as f32 % 5.0 - 2.0) * 80.0;
        let y = (i as f32 / 5.0 - 1.5) * 80.0;

        let resource_type = if i % 2 == 0 { ResourceType::Food } else { ResourceType::Water };

        commands.spawn(ResourceBundle::new(Vec2::new(x, y), resource_type, 100.0));
    }

    info!("=== Headless Simulation Started ===");
    info!("10 herbivores, 3 carnivores, 20 resources");
}

fn monitor_simulation(
    creatures: Query<(&CreatureType, &CreatureState, &Needs), With<Creature>>,
    time: Res<Time>,
    mut timer: Local<Timer>,
) {
    if timer.duration() == std::time::Duration::ZERO {
        *timer = Timer::from_seconds(2.0, TimerMode::Repeating);
    }

    timer.tick(time.delta());
    if !timer.just_finished() {
        return;
    }

    let mut states = std::collections::HashMap::new();
    let mut critical_needs = 0;

    for (creature_type, state, needs) in creatures.iter() {
        let state_name = match state {
            CreatureState::Idle => "idle",
            CreatureState::Moving { .. } => "moving",
            CreatureState::Eating => "eating",
            CreatureState::Drinking => "drinking",
            CreatureState::Resting => "resting",
            CreatureState::Dead => "dead",
        };

        *states.entry(state_name).or_insert(0) += 1;

        if needs.has_critical_need() {
            critical_needs += 1;
        }
    }

    info!(
        "T={:.1}s: States: {:?}, Critical needs: {}",
        time.elapsed_seconds(),
        states,
        critical_needs
    );
}

fn check_stop_condition(time: Res<Time>, mut exit: EventWriter<AppExit>) {
    if time.elapsed_seconds() > 10.0 {
        info!("Simulation complete after 10 seconds");
        exit.send(AppExit);
    }
}
