use bevy::prelude::*;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use serde::{Deserialize, Serialize};

/// Quality settings plugin for Phase 4
/// 
/// Provides performance presets and dynamic quality adjustment
pub struct QualitySettingsPlugin;

impl Plugin for QualitySettingsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(QualitySettings::default())
            .insert_resource(PerformanceMetrics::default())
            .insert_resource(QualityAutoAdjust::default())
            .add_systems(Update, (
                monitor_performance,
                auto_adjust_quality,
                apply_quality_settings,
            ).chain())
            .add_systems(PostUpdate, update_performance_metrics);
    }
}

/// Quality preset levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QualityPreset {
    Low,
    Medium,
    High,
    Ultra,
    Custom,
}

/// Comprehensive quality settings
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct QualitySettings {
    pub preset: QualityPreset,
    
    // Rendering settings
    pub render_distance: f32,
    pub shadow_quality: ShadowQuality,
    pub texture_quality: TextureQuality,
    pub particle_density: f32,
    pub animation_quality: AnimationQuality,
    
    // Effect settings
    pub weather_effects: bool,
    pub particle_effects: bool,
    pub floating_ui: bool,
    pub speech_bubbles: bool,
    pub environmental_effects: bool,
    
    // Performance settings
    pub max_particles: u32,
    pub max_creatures_rendered: u32,
    pub update_frequency: UpdateFrequency,
    pub lod_bias: f32,
    
    // UI settings
    pub ui_scale: f32,
    pub ui_animations: bool,
    pub ui_transparency: bool,
}

impl Default for QualitySettings {
    fn default() -> Self {
        Self::from_preset(QualityPreset::Medium)
    }
}

impl QualitySettings {
    pub fn from_preset(preset: QualityPreset) -> Self {
        match preset {
            QualityPreset::Low => Self {
                preset,
                render_distance: 200.0,
                shadow_quality: ShadowQuality::None,
                texture_quality: TextureQuality::Low,
                particle_density: 0.25,
                animation_quality: AnimationQuality::Basic,
                weather_effects: false,
                particle_effects: true,
                floating_ui: true,
                speech_bubbles: true,
                environmental_effects: false,
                max_particles: 100,
                max_creatures_rendered: 50,
                update_frequency: UpdateFrequency::Low,
                lod_bias: 0.5,
                ui_scale: 1.0,
                ui_animations: false,
                ui_transparency: false,
            },
            QualityPreset::Medium => Self {
                preset,
                render_distance: 400.0,
                shadow_quality: ShadowQuality::Low,
                texture_quality: TextureQuality::Medium,
                particle_density: 0.5,
                animation_quality: AnimationQuality::Normal,
                weather_effects: true,
                particle_effects: true,
                floating_ui: true,
                speech_bubbles: true,
                environmental_effects: true,
                max_particles: 500,
                max_creatures_rendered: 200,
                update_frequency: UpdateFrequency::Normal,
                lod_bias: 0.75,
                ui_scale: 1.0,
                ui_animations: true,
                ui_transparency: true,
            },
            QualityPreset::High => Self {
                preset,
                render_distance: 600.0,
                shadow_quality: ShadowQuality::Medium,
                texture_quality: TextureQuality::High,
                particle_density: 0.75,
                animation_quality: AnimationQuality::High,
                weather_effects: true,
                particle_effects: true,
                floating_ui: true,
                speech_bubbles: true,
                environmental_effects: true,
                max_particles: 1000,
                max_creatures_rendered: 500,
                update_frequency: UpdateFrequency::High,
                lod_bias: 1.0,
                ui_scale: 1.0,
                ui_animations: true,
                ui_transparency: true,
            },
            QualityPreset::Ultra => Self {
                preset,
                render_distance: 1000.0,
                shadow_quality: ShadowQuality::High,
                texture_quality: TextureQuality::Ultra,
                particle_density: 1.0,
                animation_quality: AnimationQuality::Ultra,
                weather_effects: true,
                particle_effects: true,
                floating_ui: true,
                speech_bubbles: true,
                environmental_effects: true,
                max_particles: 2000,
                max_creatures_rendered: 1000,
                update_frequency: UpdateFrequency::Ultra,
                lod_bias: 1.2,
                ui_scale: 1.0,
                ui_animations: true,
                ui_transparency: true,
            },
            QualityPreset::Custom => Self::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShadowQuality {
    None,
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextureQuality {
    Low,    // 512x512 max
    Medium, // 1024x1024 max
    High,   // 2048x2048 max
    Ultra,  // 4096x4096 max
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimationQuality {
    Basic,  // Key frames only
    Normal, // 30 FPS animations
    High,   // 60 FPS animations
    Ultra,  // Full interpolation
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpdateFrequency {
    Low,    // 15 Hz updates
    Normal, // 30 Hz updates
    High,   // 60 Hz updates
    Ultra,  // Uncapped
}

/// Performance metrics tracking
#[derive(Resource, Default)]
pub struct PerformanceMetrics {
    pub fps: f32,
    pub frame_time: f32,
    pub particle_count: u32,
    pub creature_count: u32,
    pub draw_calls: u32,
    pub memory_usage: f32,
    
    // Historical data for averaging
    pub fps_history: Vec<f32>,
    pub frame_time_history: Vec<f32>,
}

impl PerformanceMetrics {
    pub fn update(&mut self, diagnostics: &DiagnosticsStore) {
        // Update FPS
        if let Some(fps_diagnostic) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(fps) = fps_diagnostic.smoothed() {
                self.fps = fps as f32;
                self.fps_history.push(fps as f32);
                if self.fps_history.len() > 60 {
                    self.fps_history.remove(0);
                }
            }
        }
        
        // Update frame time
        if let Some(frame_time_diagnostic) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FRAME_TIME) {
            if let Some(frame_time) = frame_time_diagnostic.smoothed() {
                self.frame_time = frame_time as f32;
                self.frame_time_history.push(frame_time as f32);
                if self.frame_time_history.len() > 60 {
                    self.frame_time_history.remove(0);
                }
            }
        }
    }
    
    pub fn average_fps(&self) -> f32 {
        if self.fps_history.is_empty() {
            self.fps
        } else {
            self.fps_history.iter().sum::<f32>() / self.fps_history.len() as f32
        }
    }
    
    pub fn average_frame_time(&self) -> f32 {
        if self.frame_time_history.is_empty() {
            self.frame_time
        } else {
            self.frame_time_history.iter().sum::<f32>() / self.frame_time_history.len() as f32
        }
    }
}

/// Auto-adjustment settings
#[derive(Resource)]
pub struct QualityAutoAdjust {
    pub enabled: bool,
    pub target_fps: f32,
    pub adjustment_interval: Timer,
    pub min_preset: QualityPreset,
    pub max_preset: QualityPreset,
    pub last_adjustment: Option<std::time::Instant>,
}

impl Default for QualityAutoAdjust {
    fn default() -> Self {
        Self {
            enabled: true,
            target_fps: 60.0,
            adjustment_interval: Timer::from_seconds(5.0, TimerMode::Repeating),
            min_preset: QualityPreset::Low,
            max_preset: QualityPreset::High,
            last_adjustment: None,
        }
    }
}

/// System to monitor performance
fn monitor_performance(
    diagnostics: Res<DiagnosticsStore>,
    mut metrics: ResMut<PerformanceMetrics>,
    particles: Query<&crate::rendering::particle_system::ParticleInstance>,
    creatures: Query<Entity, With<crate::components::Creature>>,
) {
    metrics.update(&diagnostics);
    metrics.particle_count = particles.iter().count() as u32;
    metrics.creature_count = creatures.iter().count() as u32;
}

/// System to auto-adjust quality based on performance
fn auto_adjust_quality(
    time: Res<Time>,
    metrics: Res<PerformanceMetrics>,
    mut quality: ResMut<QualitySettings>,
    mut auto_adjust: ResMut<QualityAutoAdjust>,
) {
    if !auto_adjust.enabled {
        return;
    }
    
    auto_adjust.adjustment_interval.tick(time.delta());
    
    if auto_adjust.adjustment_interval.just_finished() {
        let avg_fps = metrics.average_fps();
        let target = auto_adjust.target_fps;
        
        // Determine if adjustment is needed
        if avg_fps < target * 0.9 {
            // Performance too low, decrease quality
            match quality.preset {
                QualityPreset::Ultra if auto_adjust.max_preset as u8 >= QualityPreset::Ultra as u8 => {
                    *quality = QualitySettings::from_preset(QualityPreset::High);
                    info!("Auto-adjusting quality to High (FPS: {:.1})", avg_fps);
                }
                QualityPreset::High if auto_adjust.max_preset as u8 >= QualityPreset::High as u8 => {
                    *quality = QualitySettings::from_preset(QualityPreset::Medium);
                    info!("Auto-adjusting quality to Medium (FPS: {:.1})", avg_fps);
                }
                QualityPreset::Medium if auto_adjust.min_preset as u8 <= QualityPreset::Low as u8 => {
                    *quality = QualitySettings::from_preset(QualityPreset::Low);
                    info!("Auto-adjusting quality to Low (FPS: {:.1})", avg_fps);
                }
                _ => {
                    // Can't decrease further, adjust individual settings
                    quality.particle_density *= 0.8;
                    quality.max_particles = (quality.max_particles as f32 * 0.8) as u32;
                    quality.render_distance *= 0.9;
                }
            }
        } else if avg_fps > target * 1.2 {
            // Performance good, can increase quality
            match quality.preset {
                QualityPreset::Low if auto_adjust.max_preset as u8 >= QualityPreset::Medium as u8 => {
                    *quality = QualitySettings::from_preset(QualityPreset::Medium);
                    info!("Auto-adjusting quality to Medium (FPS: {:.1})", avg_fps);
                }
                QualityPreset::Medium if auto_adjust.max_preset as u8 >= QualityPreset::High as u8 => {
                    *quality = QualitySettings::from_preset(QualityPreset::High);
                    info!("Auto-adjusting quality to High (FPS: {:.1})", avg_fps);
                }
                QualityPreset::High if auto_adjust.max_preset as u8 >= QualityPreset::Ultra as u8 => {
                    *quality = QualitySettings::from_preset(QualityPreset::Ultra);
                    info!("Auto-adjusting quality to Ultra (FPS: {:.1})", avg_fps);
                }
                _ => {}
            }
        }
        
        auto_adjust.last_adjustment = Some(std::time::Instant::now());
    }
}

/// System to apply quality settings to various systems
fn apply_quality_settings(
    quality: Res<QualitySettings>,
    mut particle_pool: ResMut<crate::rendering::particle_system::ParticlePool>,
    mut weather_emitters: Query<&mut crate::rendering::particle_system::ParticleEmitter>,
    mut floating_ui_settings: ResMut<crate::rendering::floating_ui::FloatingUISettings>,
    // TODO: Add fog settings when available
    // mut fog_settings: ResMut<FogSettings>,
) {
    if !quality.is_changed() {
        return;
    }
    
    // Apply particle settings
    // particle_pool.max_particles = quality.max_particles;
    
    for mut emitter in weather_emitters.iter_mut() {
        emitter.lod_bias = quality.lod_bias;
        emitter.active = quality.weather_effects;
    }
    
    // Apply UI settings
    floating_ui_settings.ui_scale = quality.ui_scale;
    
    // TODO: Apply fog distance when fog settings are available
    // match quality.preset {
    //     QualityPreset::Low => {
    //         fog_settings.falloff = FogFalloff::Linear {
    //             start: 100.0,
    //             end: quality.render_distance,
    //         };
    //     }
    //     _ => {
    //         fog_settings.falloff = FogFalloff::Linear {
    //             start: quality.render_distance * 0.5,
    //             end: quality.render_distance,
    //         };
    //     }
    // }
}

/// System to update performance metrics
fn update_performance_metrics(
    diagnostics: Res<DiagnosticsStore>,
    mut metrics: ResMut<PerformanceMetrics>,
) {
    metrics.update(&diagnostics);
}

/// Commands for changing quality settings
pub trait QualityCommands {
    fn set_quality_preset(&mut self, preset: QualityPreset);
    fn enable_auto_adjust(&mut self, enabled: bool);
    fn set_target_fps(&mut self, fps: f32);
}

impl QualityCommands for Commands<'_, '_> {
    fn set_quality_preset(&mut self, preset: QualityPreset) {
        self.insert_resource(QualitySettings::from_preset(preset));
    }
    
    fn enable_auto_adjust(&mut self, enabled: bool) {
        self.insert_resource(QualityAutoAdjust {
            enabled,
            ..Default::default()
        });
    }
    
    fn set_target_fps(&mut self, fps: f32) {
        self.insert_resource(QualityAutoAdjust {
            target_fps: fps,
            ..Default::default()
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_quality_presets() {
        let low = QualitySettings::from_preset(QualityPreset::Low);
        assert_eq!(low.max_particles, 100);
        assert_eq!(low.particle_density, 0.25);
        
        let ultra = QualitySettings::from_preset(QualityPreset::Ultra);
        assert_eq!(ultra.max_particles, 2000);
        assert_eq!(ultra.particle_density, 1.0);
    }
    
    #[test]
    fn test_performance_metrics() {
        let mut metrics = PerformanceMetrics::default();
        metrics.fps_history = vec![58.0, 60.0, 62.0];
        assert_eq!(metrics.average_fps(), 60.0);
    }
}