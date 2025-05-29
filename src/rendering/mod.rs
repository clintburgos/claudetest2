pub mod cartoon;
pub mod isometric;
pub mod particles;
pub mod shadows;
pub mod speech_bubbles;
pub mod patterns;
pub mod atlas;

// Phase 4 modules
pub mod particle_system;
pub mod enhanced_speech_bubbles;
pub mod floating_ui;
pub mod audio_system;
pub mod camera_effects;

pub use cartoon::{BiomeType, CartoonRenderingPlugin};
pub use isometric::{isometric_to_world, world_to_isometric, world_to_screen, IsometricPlugin};
pub use particles::ParticleEffectsPlugin;
pub use shadows::ShadowRenderingPlugin;
pub use speech_bubbles::SpeechBubblePlugin;

// Phase 4 exports
pub use particle_system::EnhancedParticlePlugin;
pub use enhanced_speech_bubbles::EnhancedSpeechBubblePlugin;
pub use floating_ui::FloatingUIPlugin;
pub use audio_system::CartoonAudioPlugin;
pub use camera_effects::CameraEffectsPlugin;
