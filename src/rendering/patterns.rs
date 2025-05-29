//! Genetic pattern rendering system for creature visual variations
//!
//! This module provides procedural pattern generation based on creature genetics,
//! allowing for unique visual appearances that reflect genetic diversity.
//!
//! # Pattern Types
//! - **Spots**: Circular patterns with random distribution
//! - **Stripes**: Linear patterns with configurable angle and width
//! - **Patches**: Voronoi-like cell patterns for organic appearance
//! - **None**: Solid color with no pattern overlay
//!
//! # Genetic Mapping
//! Creature genetics (0.0-1.0 values) are mapped to visual properties:
//! - Pattern gene → Pattern type and intensity
//! - Size gene → Pattern scale
//! - Speed gene → Pattern rotation/angle
//! - Color gene → Hue shift for pattern colors
//! - Aggression gene → Random seed for pattern variation
//!
//! # Rendering Pipeline
//! 1. Genetics are converted to GeneticPattern components
//! 2. Patterns are rendered as overlay textures or shader uniforms
//! 3. Blend modes control how patterns combine with base colors

use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use crate::components::{PatternType, Genetics, BodyModifiers};

/// Component for genetic pattern rendering
///
/// Stores all information needed to render a creature's genetic pattern,
/// including colors, blend mode, and pattern-specific parameters.
#[derive(Component, Clone, Reflect)]
#[reflect(Component)]
pub struct GeneticPattern {
    /// The type of pattern to render
    pub pattern_type: PatternType,
    
    /// Primary color for the pattern (usually darker)
    pub primary_color: Color,
    
    /// Secondary color for pattern highlights or borders
    pub secondary_color: Color,
    
    /// How the pattern blends with the base creature color
    pub blend_mode: PatternBlendMode,
    
    /// Pattern visibility (0.0 = invisible, 1.0 = fully opaque)
    pub intensity: f32,
    
    /// Pattern-specific parameters (scale, rotation, etc.)
    pub pattern_params: PatternParameters,
}

/// Blend modes for combining patterns with base colors
///
/// Different blend modes create different visual effects:
/// - Multiply: Darkens the base color (good for shadows)
/// - Overlay: Preserves base color highlights (most natural)
/// - Replace: Completely replaces base color (strong patterns)
/// - Add: Brightens the base color (glowing effects)
#[derive(Clone, Debug, Reflect)]
pub enum PatternBlendMode {
    /// Darkens base color by multiplying values
    Multiply,
    /// Combines pattern and base while preserving contrast
    Overlay,
    /// Completely replaces base color with pattern
    Replace,
    /// Brightens base color by adding pattern values
    Add,
}

/// Parameters controlling pattern appearance
///
/// These parameters allow fine-tuning of pattern generation
/// to create unique appearances for each creature.
#[derive(Clone, Debug, Reflect)]
pub struct PatternParameters {
    /// Pattern size multiplier (0.5 = half size, 2.0 = double size)
    pub scale: f32,
    
    /// Pattern rotation in degrees (for stripes and directional patterns)
    pub rotation: f32,
    
    /// Pattern position offset in texture space
    pub offset: Vec2,
    
    /// Random seed for deterministic pattern generation
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
    /// Creates a genetic pattern from creature genetics
    ///
    /// This function maps continuous genetic values to discrete visual properties,
    /// creating a unique but consistent appearance for each genetic combination.
    ///
    /// # Genetic Mappings:
    /// - Pattern gene (0.0-1.0):
    ///   - 0.0-0.2: No pattern (solid color)
    ///   - 0.2-0.4: Patches (large organic shapes)
    ///   - 0.4-0.7: Spots (circular patterns)
    ///   - 0.7-1.0: Stripes (linear patterns)
    /// - Size gene: Affects pattern scale (larger creatures = larger patterns)
    /// - Speed gene: Affects pattern angle (faster creatures = diagonal patterns)
    /// - Color gene: Creates hue shifts in pattern colors
    /// - Aggression gene: Seeds random number generation for unique variations
    pub fn from_genetics(genetics: &Genetics) -> Self {
        // Determine pattern type based on pattern gene thresholds
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
        // Color gene creates a hue shift from brown (0.0) to orange/red (1.0)
        let hue_shift = (genetics.color - 0.5) * 0.2; // ±0.1 hue variation
        let primary_color = Color::rgb(
            0.8 + hue_shift,    // Red channel increases with color gene
            0.6,                // Green channel stays constant
            0.4 - hue_shift,    // Blue channel decreases with color gene
        );
        let secondary_color = Color::rgb(
            0.6 + hue_shift,
            0.4,
            0.3 - hue_shift,
        );
        
        // Pattern parameters derived from multiple genetic traits
        let pattern_params = PatternParameters {
            // Scale: 0.5-2.0 based on creature size (larger = bigger patterns)
            scale: 0.5 + genetics.size * 1.5,
            
            // Rotation: 0-45 degrees based on speed (faster = more diagonal)
            rotation: genetics.speed * 45.0,
            
            // Offset: Creates pattern position variation based on color/pattern genes
            offset: Vec2::new(
                (genetics.color - 0.5) * 10.0,
                (genetics.pattern - 0.5) * 10.0,
            ),
            
            // Seed: Ensures each creature has unique pattern details
            noise_seed: (genetics.aggression * 1000.0) as u32,
        };
        
        Self {
            pattern_type,
            primary_color,
            secondary_color,
            blend_mode: PatternBlendMode::Overlay, // Natural blending for most patterns
            intensity: 0.3 + genetics.pattern * 0.4, // 0.3-0.7 visibility range
            pattern_params,
        }
    }
}

/// Resource for pattern rendering configuration
///
/// Global settings that control pattern rendering quality and performance.
/// Can be adjusted based on system capabilities or user preferences.
#[derive(Resource)]
pub struct PatternRenderingConfig {
    /// Master toggle for pattern rendering (false = solid colors only)
    pub patterns_enabled: bool,
    
    /// Quality level affecting pattern detail and noise complexity
    pub pattern_quality: PatternQuality,
    
    /// Maximum number of pattern elements (spots, stripes, etc.)
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

/// Pattern rendering quality levels
///
/// Higher quality levels provide more detail but require more GPU processing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PatternQuality {
    /// Simple patterns without noise or gradients (best performance)
    Low,
    /// Standard patterns with basic noise and smooth edges (balanced)
    Medium,
    /// Complex patterns with full noise, gradients, and details (best quality)
    High,
}

/// Material for rendering genetic patterns using GPU shaders
///
/// This material is used with custom WGSL shaders to render patterns
/// efficiently on the GPU. All parameters are passed as uniforms.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct PatternMaterial {
    /// Pattern type as integer (0=None, 1=Spots, 2=Stripes, 3=Patches)
    #[uniform(0)]
    pub pattern_type: u32,
    
    /// Primary pattern color (typically darker)
    #[uniform(0)]
    pub primary_color: Color,
    
    /// Secondary pattern color (for highlights/borders)
    #[uniform(0)]
    pub secondary_color: Color,
    
    /// Packed parameters: x=scale, y=rotation(radians), z=intensity, w=time
    #[uniform(0)]
    pub pattern_params: Vec4,
    
    /// Base creature texture to apply pattern over
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
///
/// This system runs whenever a creature's genetics change, updating their
/// visual pattern accordingly. It ensures patterns stay synchronized with
/// genetic traits throughout the creature's lifetime.
///
/// # Performance Note
/// Only processes entities with changed Genetics components to minimize
/// unnecessary recalculations.
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
        return; // Skip pattern updates when disabled for performance
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
///
/// Updates pattern material uniforms each frame for animated effects.
/// This allows patterns to have subtle animations like pulsing or shifting.
///
/// # Animation Effects:
/// - Time uniform enables shader-based animations
/// - Pattern parameters can be interpolated for smooth transitions
/// - Material updates are batched for GPU efficiency
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
            // Convert enum to integer for shader compatibility
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
///
/// Creates a texture containing the pattern data that can be used as an
/// overlay or mask. Patterns are generated algorithmically for infinite variety.
///
/// # Arguments
/// * `pattern_type` - The type of pattern to generate
/// * `params` - Parameters controlling pattern appearance
/// * `size` - Texture size in pixels (square textures only)
///
/// # Returns
/// A Bevy Image with RGBA data containing the pattern
///
/// # Performance Considerations
/// Texture generation is CPU-intensive. Consider caching generated textures
/// and using smaller sizes (32x32 or 64x64) for better performance.
pub fn generate_pattern_texture(
    pattern_type: PatternType,
    params: &PatternParameters,
    size: u32,
) -> Image {
    // Allocate RGBA buffer (4 bytes per pixel)
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

/// Generates a spots pattern with circular elements
///
/// Creates randomly distributed circular spots with soft edges.
/// The number and size of spots are controlled by the scale parameter.
///
/// # Algorithm:
/// 1. Calculate spot count inversely proportional to scale
/// 2. Randomly position spots using seeded RNG
/// 3. Draw circles with distance-based alpha falloff
/// 4. Overlapping spots create natural clustering
fn generate_spots_pattern(data: &mut [u8], size: u32, params: &PatternParameters) {
    // Fewer, larger spots for higher scale values
    let spot_count = (5.0 / params.scale) as u32;
    let spot_size = size as f32 * params.scale * 0.1;
    
    // Use noise seed for deterministic randomness (same seed = same pattern)
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
                    // Soft edge falloff for natural appearance
                    let alpha = (1.0 - dist / spot_size) * 255.0;
                    
                    // Black spots with variable alpha
                    data[idx] = 0;     // R
                    data[idx + 1] = 0; // G
                    data[idx + 2] = 0; // B
                    data[idx + 3] = alpha as u8; // A
                }
            }
        }
    }
}

/// Generates a stripes pattern with parallel lines
///
/// Creates evenly spaced stripes at a specified angle. Stripe width
/// and spacing are controlled by the scale parameter.
///
/// # Algorithm:
/// 1. Calculate stripe width based on scale
/// 2. Rotate coordinate system by specified angle
/// 3. Create alternating bands using modulo arithmetic
/// 4. Apply consistent alpha for sharp edges
fn generate_stripes_pattern(data: &mut [u8], size: u32, params: &PatternParameters) {
    let stripe_width = size as f32 * params.scale * 0.1;
    let angle = params.rotation.to_radians();
    // Pre-calculate trig values for rotation
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

/// Generates a patches pattern using Voronoi cells
///
/// Creates organic-looking patches similar to giraffe spots or cell structures.
/// Uses a Voronoi diagram approach to partition space into regions.
///
/// # Algorithm:
/// 1. Generate random cell centers (fewer cells for larger patches)
/// 2. For each pixel, find the two closest cell centers
/// 3. If near the border between cells, draw the pattern
/// 4. Creates natural, organic-looking boundaries
fn generate_patches_pattern(data: &mut [u8], size: u32, params: &PatternParameters) {
    let patch_size = size as f32 * params.scale * 0.2;
    let mut rng = StdRng::seed_from_u64(params.noise_seed as u64);
    
    // Generate voronoi cell centers
    let cell_count = (3.0 / params.scale) as u32; // Fewer cells = larger patches
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
            
            // Create patch borders where cells meet
            // Border width proportional to patch size
            let border_width = patch_size * 0.1;
            let is_border = (second_min_dist - min_dist) < border_width;
            
            let idx = ((y * size + x) * 4) as usize;
            if is_border {
                // Semi-transparent black borders
                data[idx] = 0;     // R
                data[idx + 1] = 0; // G
                data[idx + 2] = 0; // B
                data[idx + 3] = 150; // A (semi-transparent)
            } else {
                data[idx + 3] = 0; // Fully transparent inside patches
            }
        }
    }
}

/// Create pattern material for a creature
///
/// Converts a GeneticPattern component into a GPU material for rendering.
/// This material is used with custom shaders to apply patterns efficiently.
///
/// # Shader Integration
/// The material uniforms are designed to work with the pattern shader at
/// "shaders/pattern_shader.wgsl", which handles the actual pattern rendering.
pub fn create_pattern_material(
    pattern: &GeneticPattern,
    base_texture: Handle<Image>,
) -> PatternMaterial {
    PatternMaterial {
        // Convert enum to shader-compatible integer
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