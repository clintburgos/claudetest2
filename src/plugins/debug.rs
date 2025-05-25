//! Debug plugin for development and testing

use bevy::prelude::*;
use crate::components::*;

/// Debug plugin with diagnostics and visualization
pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (
                debug_creature_count,
                debug_resource_count,
            ));
    }
}

fn debug_creature_count(
    query: Query<Entity, With<Creature>>,
    time: Res<Time>,
    mut timer: Local<Timer>,
) {
    if timer.duration() == std::time::Duration::ZERO {
        *timer = Timer::from_seconds(1.0, TimerMode::Repeating);
    }
    
    timer.tick(time.delta());
    if timer.just_finished() {
        let count = query.iter().count();
        info!("Active creatures: {}", count);
    }
}

fn debug_resource_count(
    query: Query<Entity, With<ResourceMarker>>,
    time: Res<Time>,
    mut timer: Local<Timer>,
) {
    if timer.duration() == std::time::Duration::ZERO {
        *timer = Timer::from_seconds(5.0, TimerMode::Repeating);
    }
    
    timer.tick(time.delta());
    if timer.just_finished() {
        let count = query.iter().count();
        info!("Active resources: {}", count);
    }
}