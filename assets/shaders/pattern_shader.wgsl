#import bevy_sprite::mesh2d_vertex_output

@group(1) @binding(0)
var pattern_texture: texture_2d<f32>;
@group(1) @binding(1)
var pattern_sampler: sampler;

struct PatternMaterial {
    pattern_type: u32,
    primary_color: vec4<f32>,
    secondary_color: vec4<f32>,
    pattern_params: vec4<f32>, // x: scale, y: rotation, z: intensity, w: time
}

@group(1) @binding(2)
var<uniform> material: PatternMaterial;

@fragment
fn fragment(in: MeshVertexOutput) -> @location(0) vec4<f32> {
    let base_color = textureSample(pattern_texture, pattern_sampler, in.uv);
    var pattern_color = base_color;
    
    switch material.pattern_type {
        // Spots pattern
        case 1u: {
            let density = material.pattern_params.x;
            let size = material.pattern_params.y;
            
            // Create spot pattern using noise
            let spot_coord = in.uv * density;
            let spot_noise = noise_2d(spot_coord);
            
            if spot_noise > (1.0 - size) {
                pattern_color = mix(base_color, material.secondary_color, material.pattern_params.z);
            }
        }
        
        // Stripes pattern
        case 2u: {
            let width = material.pattern_params.x;
            let angle = material.pattern_params.y;
            
            // Rotate UV coordinates
            let cos_a = cos(angle);
            let sin_a = sin(angle);
            let rotated_uv = vec2<f32>(
                in.uv.x * cos_a - in.uv.y * sin_a,
                in.uv.x * sin_a + in.uv.y * cos_a
            );
            
            // Create stripe pattern
            let stripe = step(0.5, fract(rotated_uv.x * width));
            pattern_color = mix(base_color, material.secondary_color, stripe * material.pattern_params.z);
        }
        
        // Patches pattern
        case 3u: {
            let scale = material.pattern_params.x;
            let irregularity = material.pattern_params.y;
            
            // Voronoi-based patches
            let cell_coord = in.uv * scale;
            let voronoi = voronoi_2d(cell_coord, irregularity);
            
            pattern_color = mix(
                material.primary_color,
                material.secondary_color,
                voronoi * material.pattern_params.z
            );
        }
        
        // Gradient pattern
        case 4u: {
            let gradient = in.uv.y; // Vertical gradient
            pattern_color = mix(
                material.primary_color,
                material.secondary_color,
                gradient * material.pattern_params.z
            );
        }
        
        default: {}
    }
    
    return pattern_color;
}

// Simple 2D noise function
fn noise_2d(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    
    let a = hash_2d(i);
    let b = hash_2d(i + vec2<f32>(1.0, 0.0));
    let c = hash_2d(i + vec2<f32>(0.0, 1.0));
    let d = hash_2d(i + vec2<f32>(1.0, 1.0));
    
    let u = f * f * (3.0 - 2.0 * f);
    
    return mix(a, b, u.x) + (c - a) * u.y * (1.0 - u.x) + (d - b) * u.x * u.y;
}

fn hash_2d(p: vec2<f32>) -> f32 {
    let h = dot(p, vec2<f32>(127.1, 311.7));
    return fract(sin(h) * 43758.5453123);
}

// Simple voronoi implementation
fn voronoi_2d(p: vec2<f32>, irregularity: f32) -> f32 {
    let i = floor(p);
    let f = fract(p);
    
    var min_dist = 1.0;
    
    for (var y = -1; y <= 1; y = y + 1) {
        for (var x = -1; x <= 1; x = x + 1) {
            let neighbor = vec2<f32>(f32(x), f32(y));
            let point = hash_2d(i + neighbor) * irregularity + neighbor;
            let dist = length(point - f);
            min_dist = min(min_dist, dist);
        }
    }
    
    return min_dist;
}