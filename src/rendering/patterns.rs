//! Genetic pattern rendering system for creature visual variations

use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use crate::components::{PatternType, Genetics, BodyModifiers};

/// Component for genetic pattern rendering
#[derive(Component, Clone, Reflect)]
#[reflect(Component)]
pub struct GeneticPattern {
    pub pattern_type: PatternType,
    pub primary_color: Color,
    pub secondary_color: Color,
    pub blend_mode: PatternBlendMode,
    pub intensity: f32, // 0.0-1.0
    pub pattern_params: PatternParameters,
}

#[derive(Clone, Debug, Reflect)]
pub enum PatternBlendMode {
    Multiply,
    Overlay,
    Replace,
    Add,
}

#[derive(Clone, Debug, Reflect)]
pub struct PatternParameters {
    pub scale: f32,
    pub rotation: f32,
    pub offset: Vec2,
    pub noise_seed: u32,
}

impl Default for PatternParameters {
    fn default() -> Self {
        Self {
            scale: 1.0,
            rotation: 0.0,
            offset: Vec2::ZERO,
            noise_seed: 0,
        }
    }
}

impl GeneticPattern {
    pub fn from_genetics(genetics: &Genetics) -> Self {
        // Determine pattern type based on pattern gene
        let pattern_type = if genetics.pattern > 0.7 {
            PatternType::Stripes
        } else if genetics.pattern > 0.4 {
            PatternType::Spots
        } else if genetics.pattern > 0.2 {
            PatternType::Patches
        } else {
            PatternType::None
        };
        
        // Generate colors based on genetics
        let hue_shift = (genetics.color - 0.5) * 0.2;
        let primary_color = Color::rgb(
            0.8 + hue_shift,
            0.6,
            0.4 - hue_shift,
        );
        let secondary_color = Color::rgb(
            0.6 + hue_shift,
            0.4,
            0.3 - hue_shift,
        );
        
        // Pattern parameters from genetics
        let pattern_params = PatternParameters {
            scale: 0.5 + genetics.size * 1.5, // Larger creatures have bigger patterns
            rotation: genetics.speed * 45.0,   // Speed affects pattern angle
            offset: Vec2::new(
                (genetics.color - 0.5) * 10.0,
                (genetics.pattern - 0.5) * 10.0,
            ),
            noise_seed: (genetics.aggression * 1000.0) as u32,
        };
        
        Self {
            pattern_type,
            primary_color,
            secondary_color,
            blend_mode: PatternBlendMode::Overlay,
            intensity: 0.3 + genetics.pattern * 0.4, // Pattern visibility
            pattern_params,
        }
    }
}

/// Resource for pattern rendering configuration
#[derive(Resource)]
pub struct PatternRenderingConfig {
    pub patterns_enabled: bool,
    pub pattern_quality: PatternQuality,
    pub max_pattern_complexity: u32,
}

impl Default for PatternRenderingConfig {
    fn default() -> Self {
        Self {
            patterns_enabled: true,
            pattern_quality: PatternQuality::Medium,
            max_pattern_complexity: 100,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PatternQuality {
    Low,    // Simple patterns, no noise
    Medium, // Standard patterns with basic noise
    High,   // Complex patterns with full noise
}

/// Material for rendering genetic patterns
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct PatternMaterial {
    #[uniform(0)]
    pub pattern_type: u32,
    #[uniform(0)]
    pub primary_color: Color,
    #[uniform(0)]
    pub secondary_color: Color,
    #[uniform(0)]
    pub pattern_params: Vec4, // x: scale, y: rotation, z: intensity, w: time
    #[texture(1)]
    #[sampler(2)]
    pub base_texture: Option<Handle<Image>>,
}

impl Material for PatternMaterial {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "shaders/pattern_shader.wgsl".into()
    }
}

/// System to update genetic patterns based on creature genetics
pub fn update_genetic_patterns(
    config: Res<PatternRenderingConfig>,
    mut query: Query<
        (
            &Genetics,
            &mut GeneticPattern,
            &mut BodyModifiers,
            Option<&mut Handle<PatternMaterial>>,
        ),
        Changed<Genetics>,
    >,
) {
    if !config.patterns_enabled {
        return;
    }
    
    for (genetics, mut pattern, mut body_mods, material_handle) in query.iter_mut() {
        // Update pattern from genetics
        *pattern = GeneticPattern::from_genetics(genetics);
        
        // Update body modifiers
        body_mods.pattern_type = pattern.pattern_type;
        
        // Update material if present
        if let Some(_material) = material_handle {
            // Material update would happen here
        }
    }
}

/// System to apply pattern overlays to creature sprites
pub fn apply_pattern_overlays(
    time: Res<Time>,
    config: Res<PatternRenderingConfig>,
    mut materials: ResMut<Assets<PatternMaterial>>,
    query: Query<(
        &GeneticPattern,
        &Handle<PatternMaterial>,
    )>,
) {
    if !config.patterns_enabled {
        return;
    }
    
    for (pattern, material_handle) in query.iter() {
        if let Some(material) = materials.get_mut(material_handle) {
            // Update material uniforms
            material.pattern_type = match pattern.pattern_type {
                PatternType::None => 0,
                PatternType::Spots => 1,
                PatternType::Stripes => 2,
                PatternType::Patches => 3,
            };
            
            material.primary_color = pattern.primary_color;
            material.secondary_color = pattern.secondary_color;
            material.pattern_params = Vec4::new(
                pattern.pattern_params.scale,
                pattern.pattern_params.rotation.to_radians(),
                pattern.intensity,
                time.elapsed_seconds(),
            );
        }
    }
}

/// Generate pattern texture procedurally
pub fn generate_pattern_texture(
    pattern_type: PatternType,
    params: &PatternParameters,
    size: u32,
) -> Image {
    let mut data = vec![0u8; (size * size * 4) as usize];
    
    match pattern_type {
        PatternType::None => {
            // Transparent texture
            for i in (0..data.len()).step_by(4) {
                data[i + 3] = 0; // Alpha = 0
            }
        }
        PatternType::Spots => {
            generate_spots_pattern(&mut data, size, params);
        }
        PatternType::Stripes => {
            generate_stripes_pattern(&mut data, size, params);
        }
        PatternType::Patches => {
            generate_patches_pattern(&mut data, size, params);
        }
    }
    
    Image::new(
        bevy::render::render_resource::Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        data,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
    )
}

fn generate_spots_pattern(data: &mut [u8], size: u32, params: &PatternParameters) {
    let spot_count = (5.0 / params.scale) as u32;
    let spot_size = size as f32 * params.scale * 0.1;
    
    // Use noise seed for deterministic randomness
    let mut rng = StdRng::seed_from_u64(params.noise_seed as u64);
    
    for _ in 0..spot_count {
        let x = rng.gen::<f32>() * size as f32;
        let y = rng.gen::<f32>() * size as f32;
        
        // Draw circular spot
        for py in 0..size {
            for px in 0..size {
                let dx = px as f32 - x;
                let dy = py as f32 - y;
                let dist = (dx * dx + dy * dy).sqrt();
                
                if dist < spot_size {
                    let idx = ((py * size + px) * 4) as usize;
                    let alpha = (1.0 - dist / spot_size) * 255.0;
                    
                    data[idx] = 0;     // R
                    data[idx + 1] = 0; // G
                    data[idx + 2] = 0; // B
                    data[idx + 3] = alpha as u8; // A
                }
            }
        }
    }
}

fn generate_stripes_pattern(data: &mut [u8], size: u32, params: &PatternParameters) {
    let stripe_width = size as f32 * params.scale * 0.1;
    let angle = params.rotation.to_radians();
    let cos_a = angle.cos();
    let sin_a = angle.sin();
    
    for y in 0..size {
        for x in 0..size {
            // Rotate coordinates
            let rx = (x as f32 - size as f32 / 2.0) * cos_a - (y as f32 - size as f32 / 2.0) * sin_a;
            
            // Create stripe pattern
            let stripe = ((rx / stripe_width).floor() as i32 % 2) == 0;
            
            let idx = ((y * size + x) * 4) as usize;
            if stripe {
                data[idx] = 0;     // R
                data[idx + 1] = 0; // G
                data[idx + 2] = 0; // B
                data[idx + 3] = 200; // A
            } else {
                data[idx + 3] = 0; // Transparent
            }
        }
    }
}

fn generate_patches_pattern(data: &mut [u8], size: u32, params: &PatternParameters) {
    let patch_size = size as f32 * params.scale * 0.2;
    let mut rng = StdRng::seed_from_u64(params.noise_seed as u64);
    
    // Generate voronoi-like patches
    let cell_count = (3.0 / params.scale) as u32;
    let mut cells = Vec::new();
    
    for _ in 0..cell_count {
        cells.push(Vec2::new(
            rng.gen::<f32>() * size as f32,
            rng.gen::<f32>() * size as f32,
        ));
    }
    
    for y in 0..size {
        for x in 0..size {
            let pos = Vec2::new(x as f32, y as f32);
            
            // Find closest cell
            let mut min_dist = f32::MAX;
            let mut second_min_dist = f32::MAX;
            
            for cell in &cells {
                let dist = pos.distance(*cell);
                if dist < min_dist {
                    second_min_dist = min_dist;
                    min_dist = dist;
                } else if dist < second_min_dist {
                    second_min_dist = dist;
                }
            }
            
            // Create patch borders
            let border_width = patch_size * 0.1;
            let is_border = (second_min_dist - min_dist) < border_width;
            
            let idx = ((y * size + x) * 4) as usize;
            if is_border {
                data[idx] = 0;     // R
                data[idx + 1] = 0; // G
                data[idx + 2] = 0; // B
                data[idx + 3] = 150; // A
            } else {
                data[idx + 3] = 0; // Transparent
            }
        }
    }
}

/// Create pattern material for a creature
pub fn create_pattern_material(
    pattern: &GeneticPattern,
    base_texture: Handle<Image>,
) -> PatternMaterial {
    PatternMaterial {
        pattern_type: match pattern.pattern_type {
            PatternType::None => 0,
            PatternType::Spots => 1,
            PatternType::Stripes => 2,
            PatternType::Patches => 3,
        },
        primary_color: pattern.primary_color,
        secondary_color: pattern.secondary_color,
        pattern_params: Vec4::new(
            pattern.pattern_params.scale,
            pattern.pattern_params.rotation,
            pattern.intensity,
            0.0, // Time will be updated in shader
        ),
        base_texture: Some(base_texture),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pattern_from_genetics() {
        let mut genetics = Genetics::default();
        
        // Test stripe pattern
        genetics.pattern = 0.8;
        let pattern = GeneticPattern::from_genetics(&genetics);
        assert_eq!(pattern.pattern_type, PatternType::Stripes);
        
        // Test spot pattern
        genetics.pattern = 0.5;
        let pattern = GeneticPattern::from_genetics(&genetics);
        assert_eq!(pattern.pattern_type, PatternType::Spots);
        
        // Test no pattern
        genetics.pattern = 0.1;
        let pattern = GeneticPattern::from_genetics(&genetics);
        assert_eq!(pattern.pattern_type, PatternType::None);
    }
    
    #[test]
    fn test_pattern_intensity() {
        let mut genetics = Genetics::default();
        genetics.pattern = 1.0;
        
        let pattern = GeneticPattern::from_genetics(&genetics);
        assert!(pattern.intensity > 0.5);
        assert!(pattern.intensity <= 0.7 + f32::EPSILON); // Allow for floating point precision
    }
    
    #[test]
    fn test_pattern_material_creation() {
        let pattern = GeneticPattern {
            pattern_type: PatternType::Spots,
            primary_color: Color::RED,
            secondary_color: Color::BLUE,
            blend_mode: PatternBlendMode::Overlay,
            intensity: 0.5,
            pattern_params: PatternParameters::default(),
        };
        
        let material = create_pattern_material(&pattern, Handle::default());
        assert_eq!(material.pattern_type, 1); // Spots = 1
        assert_eq!(material.primary_color, Color::RED);
        assert_eq!(material.pattern_params.z, 0.5); // Intensity
    }
}