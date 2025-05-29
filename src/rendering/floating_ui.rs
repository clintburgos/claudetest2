use bevy::prelude::*;
use crate::components::{Health, Needs};
use crate::plugins::Selected;

/// Floating UI plugin for Phase 4
/// 
/// Displays health bars, need indicators, and status icons above creatures
pub struct FloatingUIPlugin;

impl Plugin for FloatingUIPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(FloatingUISettings::default())
            .insert_resource(UIAssets::default())
            .add_systems(Startup, load_ui_assets)
            .add_systems(Update, (
                spawn_floating_ui,
                update_health_bars,
                update_need_indicators,
                update_ui_positions,
                update_ui_visibility,
                animate_ui_elements,
                cleanup_orphaned_ui,
            ).chain());
    }
}

/// Settings for floating UI behavior
#[derive(Resource)]
pub struct FloatingUISettings {
    pub show_health_always: bool,
    pub show_health_when_damaged: bool,
    pub show_health_when_selected: bool,
    pub show_needs_when_critical: bool,
    pub show_needs_when_selected: bool,
    pub fade_distance: f32,
    pub ui_scale: f32,
    pub animation_speed: f32,
}

impl Default for FloatingUISettings {
    fn default() -> Self {
        Self {
            show_health_always: false,
            show_health_when_damaged: true,
            show_health_when_selected: true,
            show_needs_when_critical: true,
            show_needs_when_selected: true,
            fade_distance: 200.0,
            ui_scale: 1.0,
            animation_speed: 1.0,
        }
    }
}

/// UI assets for floating elements
#[derive(Resource, Default)]
pub struct UIAssets {
    pub health_bar_bg: Handle<Image>,
    pub health_bar_fill: Handle<Image>,
    pub health_bar_frame: Handle<Image>,
    pub need_icons: NeedIconHandles,
    pub status_icons: StatusIconHandles,
}

#[derive(Default)]
pub struct NeedIconHandles {
    pub hunger: Handle<Image>,
    pub thirst: Handle<Image>,
    pub energy: Handle<Image>,
    pub social: Handle<Image>,
}

#[derive(Default)]
pub struct StatusIconHandles {
    pub sleeping: Handle<Image>,
    pub eating: Handle<Image>,
    pub talking: Handle<Image>,
    pub working: Handle<Image>,
    pub alert: Handle<Image>,
}

/// Component for floating UI container
#[derive(Component)]
pub struct FloatingUI {
    pub owner: Entity,
    pub offset: Vec3,
    pub visibility_state: UIVisibilityState,
    pub fade_timer: Timer,
}

/// Visibility states for UI elements
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UIVisibilityState {
    Hidden,
    FadingIn,
    Visible,
    FadingOut,
}

/// Component for health bar
#[derive(Component)]
pub struct HealthBar {
    pub max_health: f32,
    pub current_health: f32,
    pub previous_health: f32,
    pub damage_flash_timer: Timer,
    pub heal_flash_timer: Timer,
}

/// Component for need indicator
#[derive(Component)]
pub struct NeedIndicator {
    pub need_type: NeedType,
    pub value: f32,
    pub is_critical: bool,
    pub pulse_timer: Timer,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NeedType {
    Hunger,
    Thirst,
    Energy,
    Social,
}

/// Component for status icon
#[derive(Component)]
pub struct StatusIcon {
    pub status_type: StatusType,
    pub duration: Option<Timer>,
    pub bounce_timer: Timer,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StatusType {
    Sleeping,
    Eating,
    Talking,
    Working,
    Alert,
}

/// System to load UI assets
fn load_ui_assets(
    mut ui_assets: ResMut<UIAssets>,
    asset_server: Res<AssetServer>,
) {
    // Health bar assets
    ui_assets.health_bar_bg = asset_server.load("sprites/ui/health_bar_bg.png");
    ui_assets.health_bar_fill = asset_server.load("sprites/ui/health_bar_fill.png");
    ui_assets.health_bar_frame = asset_server.load("sprites/ui/health_bar_frame.png");
    
    // Need icons
    ui_assets.need_icons.hunger = asset_server.load("sprites/ui/icons/hunger.png");
    ui_assets.need_icons.thirst = asset_server.load("sprites/ui/icons/thirst.png");
    ui_assets.need_icons.energy = asset_server.load("sprites/ui/icons/energy.png");
    ui_assets.need_icons.social = asset_server.load("sprites/ui/icons/social.png");
    
    // Status icons
    ui_assets.status_icons.sleeping = asset_server.load("sprites/ui/icons/sleeping.png");
    ui_assets.status_icons.eating = asset_server.load("sprites/ui/icons/eating.png");
    ui_assets.status_icons.talking = asset_server.load("sprites/ui/icons/talking.png");
    ui_assets.status_icons.working = asset_server.load("sprites/ui/icons/working.png");
    ui_assets.status_icons.alert = asset_server.load("sprites/ui/icons/alert.png");
}

/// System to spawn floating UI for creatures
fn spawn_floating_ui(
    mut commands: Commands,
    ui_assets: Res<UIAssets>,
    settings: Res<FloatingUISettings>,
    creatures: Query<(Entity, &Health, &Needs), (Without<FloatingUI>, With<Transform>)>,
    existing_ui: Query<&FloatingUI>,
) {
    for (entity, health, needs) in creatures.iter() {
        // Skip if already has UI
        if existing_ui.iter().any(|ui| ui.owner == entity) {
            continue;
        }
        
        // Create UI container
        let ui_offset = Vec3::new(0.0, 35.0, 5.0); // Above creature
        
        let ui_entity = commands.spawn((
            SpatialBundle {
                transform: Transform::from_translation(ui_offset),
                visibility: Visibility::Hidden,
                ..default()
            },
            FloatingUI {
                owner: entity,
                offset: ui_offset,
                visibility_state: UIVisibilityState::Hidden,
                fade_timer: Timer::from_seconds(0.3, TimerMode::Once),
            },
            Name::new("FloatingUI"),
        )).id();
        
        // Spawn health bar
        let health_bar = commands.spawn((
            SpriteBundle {
                texture: ui_assets.health_bar_bg.clone(),
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
                sprite: Sprite {
                    custom_size: Some(Vec2::new(40.0, 6.0)),
                    color: Color::rgba(0.2, 0.2, 0.2, 0.8),
                    ..default()
                },
                ..default()
            },
            HealthBar {
                max_health: health.max,
                current_health: health.current,
                previous_health: health.current,
                damage_flash_timer: Timer::from_seconds(0.2, TimerMode::Once),
                heal_flash_timer: Timer::from_seconds(0.2, TimerMode::Once),
            },
            Name::new("HealthBar"),
        )).id();
        
        // Health bar fill
        let fill_width = (health.current / health.max * 40.0).max(0.0);
        commands.spawn((
            SpriteBundle {
                texture: ui_assets.health_bar_fill.clone(),
                transform: Transform::from_translation(Vec3::new(-20.0 + fill_width / 2.0, 0.0, 0.1)),
                sprite: Sprite {
                    custom_size: Some(Vec2::new(fill_width, 6.0)),
                    color: health_color(health.current / health.max),
                    ..default()
                },
                ..default()
            },
            Name::new("HealthBarFill"),
        )).set_parent(health_bar);
        
        // Health bar frame
        commands.spawn((
            SpriteBundle {
                texture: ui_assets.health_bar_frame.clone(),
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.2)),
                sprite: Sprite {
                    custom_size: Some(Vec2::new(42.0, 8.0)),
                    ..default()
                },
                ..default()
            },
            Name::new("HealthBarFrame"),
        )).set_parent(health_bar);
        
        commands.entity(health_bar).set_parent(ui_entity);
        
        // Spawn need indicators
        let mut x_offset = -30.0;
        for (need_type, value) in [
            (NeedType::Hunger, needs.hunger),
            (NeedType::Thirst, needs.thirst),
            (NeedType::Energy, needs.energy),
            (NeedType::Social, needs.social),
        ] {
            if value < 0.3 { // Only show critical needs
                let icon_texture = match need_type {
                    NeedType::Hunger => ui_assets.need_icons.hunger.clone(),
                    NeedType::Thirst => ui_assets.need_icons.thirst.clone(),
                    NeedType::Energy => ui_assets.need_icons.energy.clone(),
                    NeedType::Social => ui_assets.need_icons.social.clone(),
                };
                
                commands.spawn((
                    SpriteBundle {
                        texture: icon_texture,
                        transform: Transform::from_translation(Vec3::new(x_offset, -15.0, 0.0))
                            .with_scale(Vec3::splat(0.5)),
                        sprite: Sprite {
                            color: need_color(value),
                            ..default()
                        },
                        ..default()
                    },
                    NeedIndicator {
                        need_type,
                        value,
                        is_critical: value < 0.2,
                        pulse_timer: Timer::from_seconds(1.0, TimerMode::Repeating),
                    },
                    Name::new(format!("NeedIndicator_{:?}", need_type)),
                )).set_parent(ui_entity);
                
                x_offset += 12.0;
            }
        }
        
        // Attach UI to creature
        commands.entity(entity).add_child(ui_entity);
    }
}

/// System to update health bars
fn update_health_bars(
    mut health_bars: Query<(&mut HealthBar, &Children)>,
    mut fills: Query<&mut Sprite, Without<HealthBar>>,
    creatures: Query<&Health>,
    floating_ui: Query<&FloatingUI>,
) {
    for (mut health_bar, children) in health_bars.iter_mut() {
        // Get owner's current health
        if let Some(ui) = floating_ui.iter().find(|ui| {
            children.iter().any(|&child| child == ui.owner)
        }) {
            if let Ok(health) = creatures.get(ui.owner) {
                health_bar.current_health = health.current;
                
                // Check for damage/healing
                if health_bar.current_health < health_bar.previous_health {
                    health_bar.damage_flash_timer.reset();
                } else if health_bar.current_health > health_bar.previous_health {
                    health_bar.heal_flash_timer.reset();
                }
                
                health_bar.previous_health = health_bar.current_health;
                
                // Update fill sprite
                for &child in children.iter() {
                    if let Ok(mut sprite) = fills.get_mut(child) {
                        let health_ratio = health_bar.current_health / health_bar.max_health;
                        let fill_width = (health_ratio * 40.0).max(0.0);
                        sprite.custom_size = Some(Vec2::new(fill_width, 6.0));
                        sprite.color = health_color(health_ratio);
                        
                        // Flash effects
                        if !health_bar.damage_flash_timer.finished() {
                            sprite.color = Color::rgb(1.0, 0.3, 0.3);
                        } else if !health_bar.heal_flash_timer.finished() {
                            sprite.color = Color::rgb(0.3, 1.0, 0.3);
                        }
                    }
                }
            }
        }
    }
}

/// System to update need indicators
fn update_need_indicators(
    time: Res<Time>,
    mut indicators: Query<(&mut NeedIndicator, &mut Transform, &mut Sprite)>,
    creatures: Query<&Needs>,
    floating_ui: Query<&FloatingUI>,
) {
    for (mut indicator, mut transform, mut sprite) in indicators.iter_mut() {
        indicator.pulse_timer.tick(time.delta());
        
        // Update value from creature
        // TODO: Link to specific creature needs
        
        // Pulse animation for critical needs
        if indicator.is_critical {
            let pulse = (indicator.pulse_timer.fraction() * std::f32::consts::TAU).sin();
            transform.scale = Vec3::splat(0.5 + pulse * 0.1);
            sprite.color = need_color(indicator.value) * (0.8 + pulse * 0.2);
        }
    }
}

/// System to update UI positions
fn update_ui_positions(
    mut ui_query: Query<(&mut Transform, &FloatingUI)>,
    creature_transforms: Query<&Transform, Without<FloatingUI>>,
) {
    for (mut ui_transform, floating_ui) in ui_query.iter_mut() {
        if let Ok(creature_transform) = creature_transforms.get(floating_ui.owner) {
            let target_pos = creature_transform.translation + floating_ui.offset;
            ui_transform.translation = ui_transform.translation.lerp(target_pos, 0.3);
        }
    }
}

/// System to update UI visibility based on settings
fn update_ui_visibility(
    time: Res<Time>,
    settings: Res<FloatingUISettings>,
    camera: Query<&Transform, With<Camera>>,
    mut ui_query: Query<(&mut FloatingUI, &mut Visibility, &Transform)>,
    creatures: Query<(&Health, Option<&Selected>)>,
) {
    let camera_pos = camera.single().translation;
    
    for (mut ui, mut visibility, transform) in ui_query.iter_mut() {
        if let Ok((health, selected)) = creatures.get(ui.owner) {
            // Determine if UI should be visible
            let should_show = settings.show_health_always ||
                (settings.show_health_when_damaged && health.current < health.max * 0.8) ||
                (settings.show_health_when_selected && selected.is_some());
            
            // Distance-based fading
            let distance = (transform.translation - camera_pos).length();
            let distance_visibility = if distance < settings.fade_distance {
                1.0
            } else if distance < settings.fade_distance * 1.5 {
                1.0 - (distance - settings.fade_distance) / (settings.fade_distance * 0.5)
            } else {
                0.0
            };
            
            // Update visibility state
            match ui.visibility_state {
                UIVisibilityState::Hidden if should_show && distance_visibility > 0.0 => {
                    ui.visibility_state = UIVisibilityState::FadingIn;
                    ui.fade_timer.reset();
                }
                UIVisibilityState::Visible if !should_show || distance_visibility == 0.0 => {
                    ui.visibility_state = UIVisibilityState::FadingOut;
                    ui.fade_timer.reset();
                }
                _ => {}
            }
            
            // Update fade timer
            ui.fade_timer.tick(time.delta());
            
            // Apply visibility
            match ui.visibility_state {
                UIVisibilityState::Hidden => {
                    *visibility = Visibility::Hidden;
                }
                UIVisibilityState::FadingIn => {
                    *visibility = Visibility::Visible;
                    if ui.fade_timer.finished() {
                        ui.visibility_state = UIVisibilityState::Visible;
                    }
                }
                UIVisibilityState::Visible => {
                    *visibility = Visibility::Visible;
                }
                UIVisibilityState::FadingOut => {
                    if ui.fade_timer.finished() {
                        ui.visibility_state = UIVisibilityState::Hidden;
                        *visibility = Visibility::Hidden;
                    }
                }
            }
        }
    }
}

/// System to animate UI elements
fn animate_ui_elements(
    time: Res<Time>,
    settings: Res<FloatingUISettings>,
    mut health_bars: Query<&mut HealthBar>,
    mut status_icons: Query<(&mut StatusIcon, &mut Transform)>,
) {
    // Update health bar timers
    for mut health_bar in health_bars.iter_mut() {
        health_bar.damage_flash_timer.tick(time.delta());
        health_bar.heal_flash_timer.tick(time.delta());
    }
    
    // Animate status icons
    for (mut icon, mut transform) in status_icons.iter_mut() {
        icon.bounce_timer.tick(time.delta());
        
        if let Some(ref mut duration) = icon.duration {
            duration.tick(time.delta());
        }
        
        // Bounce animation
        let bounce = (icon.bounce_timer.fraction() * std::f32::consts::TAU).sin() * 0.1;
        transform.translation.y += bounce * settings.animation_speed;
    }
}

/// System to cleanup orphaned UI
fn cleanup_orphaned_ui(
    mut commands: Commands,
    ui_query: Query<(Entity, &FloatingUI)>,
    creatures: Query<Entity>,
) {
    for (ui_entity, floating_ui) in ui_query.iter() {
        if creatures.get(floating_ui.owner).is_err() {
            commands.entity(ui_entity).despawn_recursive();
        }
    }
}

// Helper functions

fn health_color(ratio: f32) -> Color {
    if ratio > 0.7 {
        Color::rgb(0.2, 0.8, 0.2) // Green
    } else if ratio > 0.3 {
        Color::rgb(0.8, 0.8, 0.2) // Yellow
    } else {
        Color::rgb(0.8, 0.2, 0.2) // Red
    }
}

fn need_color(value: f32) -> Color {
    if value > 0.5 {
        Color::WHITE
    } else if value > 0.2 {
        Color::rgb(1.0, 0.8, 0.2) // Warning
    } else {
        Color::rgb(1.0, 0.2, 0.2) // Critical
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_health_color_calculation() {
        assert_eq!(health_color(1.0), Color::rgb(0.2, 0.8, 0.2));
        assert_eq!(health_color(0.5), Color::rgb(0.8, 0.8, 0.2));
        assert_eq!(health_color(0.1), Color::rgb(0.8, 0.2, 0.2));
    }
    
    #[test]
    fn test_need_color_calculation() {
        assert_eq!(need_color(1.0), Color::WHITE);
        assert_eq!(need_color(0.3), Color::rgb(1.0, 0.8, 0.2));
        assert_eq!(need_color(0.1), Color::rgb(1.0, 0.2, 0.2));
    }
}