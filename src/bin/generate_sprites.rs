use image::{ImageBuffer, Rgba, RgbaImage};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create directories
    std::fs::create_dir_all("assets/sprites/creatures")?;
    std::fs::create_dir_all("assets/sprites/terrain")?;
    std::fs::create_dir_all("assets/sprites/particles")?;
    std::fs::create_dir_all("assets/fonts")?;

    // Generate creature atlas
    generate_creature_atlas()?;
    
    // Generate terrain atlas
    generate_terrain_atlas()?;
    
    // Generate particle sprites
    generate_particle_sprites()?;
    
    // Create a dummy font file (we'll use system fonts in production)
    std::fs::write("assets/fonts/FiraMono-Medium.ttf", b"dummy font file")?;
    
    println!("All sprite assets generated successfully!");
    Ok(())
}

fn generate_creature_atlas() -> Result<(), Box<dyn std::error::Error>> {
    // 8x8 grid of 48x48 sprites = 384x384 image
    let mut atlas = RgbaImage::new(384, 384);
    
    let animations = [
        ("Idle", [255, 200, 200, 255]),      // Pink
        ("Walk", [200, 255, 200, 255]),      // Light green
        ("Run", [200, 200, 255, 255]),       // Light blue
        ("Eat", [255, 255, 200, 255]),       // Yellow
        ("Sleep", [200, 200, 200, 255]),     // Gray
        ("Talk", [255, 200, 255, 255]),      // Magenta
        ("Attack", [255, 100, 100, 255]),    // Red
        ("Death", [100, 100, 100, 255]),     // Dark gray
    ];
    
    for (anim_idx, (_name, color)) in animations.iter().enumerate() {
        let y_offset = (anim_idx * 48) as u32;
        
        // Draw 8 frames per animation
        for frame in 0..8 {
            let x_offset = (frame * 48) as u32;
            
            // Draw creature body (circle)
            draw_circle(&mut atlas, x_offset + 24, y_offset + 24, 16, *color);
            
            // Add eyes
            draw_circle(&mut atlas, x_offset + 18, y_offset + 20, 2, [0, 0, 0, 255]);
            draw_circle(&mut atlas, x_offset + 30, y_offset + 20, 2, [0, 0, 0, 255]);
            
            // Add animation variation
            if anim_idx == 1 && frame % 2 == 0 {
                // Walk bounce
                draw_circle(&mut atlas, x_offset + 24, y_offset + 38, 4, *color);
            } else if anim_idx == 3 {
                // Eating animation - mouth
                let mouth_open = frame % 4 < 2;
                if mouth_open {
                    draw_rect(&mut atlas, x_offset + 20, y_offset + 28, 8, 4, [50, 50, 50, 255]);
                }
            }
            
            // Add genetic variation indicators based on frame
            match frame {
                0..=1 => {
                    // No pattern - plain creature
                }
                2..=3 => {
                    // Spots pattern
                    draw_circle(&mut atlas, x_offset + 15, y_offset + 15, 3, darken_color(*color));
                    draw_circle(&mut atlas, x_offset + 33, y_offset + 15, 3, darken_color(*color));
                    draw_circle(&mut atlas, x_offset + 24, y_offset + 30, 3, darken_color(*color));
                    draw_circle(&mut atlas, x_offset + 18, y_offset + 28, 2, darken_color(*color));
                    draw_circle(&mut atlas, x_offset + 30, y_offset + 28, 2, darken_color(*color));
                }
                4..=5 => {
                    // Stripes pattern
                    for i in 0..3 {
                        let stripe_y = y_offset + 14 + i * 6;
                        draw_rect(&mut atlas, x_offset + 10, stripe_y, 28, 2, darken_color(*color));
                    }
                }
                6..=7 => {
                    // Size variation - larger with outer ring
                    draw_circle_outline(&mut atlas, x_offset + 24, y_offset + 24, 18, *color);
                    draw_circle_outline(&mut atlas, x_offset + 24, y_offset + 24, 19, lighten_color(*color));
                }
                _ => {}
            }
        }
    }
    
    atlas.save("assets/sprites/creatures/creature_atlas.png")?;
    println!("Created creature atlas (384x384)");
    Ok(())
}

fn generate_terrain_atlas() -> Result<(), Box<dyn std::error::Error>> {
    // 5 biomes x 4 variants = 20 tiles
    // Using 64x32 isometric tiles in a 5x4 grid = 320x128
    let mut atlas = RgbaImage::new(320, 128);
    
    let biomes = [
        ("Forest", [34, 139, 34, 255]),      // Forest green
        ("Desert", [237, 201, 175, 255]),    // Sand
        ("Grassland", [124, 252, 0, 255]),   // Grass green
        ("Tundra", [230, 230, 250, 255]),    // Light blue-white
        ("Ocean", [0, 119, 190, 255]),       // Ocean blue
    ];
    
    for (biome_idx, (biome_name, color)) in biomes.iter().enumerate() {
        let x_offset = (biome_idx * 64) as u32;
        
        // Draw 4 variants per biome
        for variant in 0..4 {
            let y_offset = (variant * 32) as u32;
            
            // Vary the color slightly
            let variant_color = [
                (color[0] as i32 + (variant as i32 - 2) * 10).clamp(0, 255) as u8,
                (color[1] as i32 + (variant as i32 - 2) * 10).clamp(0, 255) as u8,
                (color[2] as i32 + (variant as i32 - 2) * 10).clamp(0, 255) as u8,
                255,
            ];
            
            // Draw isometric tile
            draw_isometric_tile(&mut atlas, x_offset, y_offset, 64, 32, variant_color);
            
            // Add biome-specific details
            match *biome_name {
                "Forest" => {
                    // Add tree
                    draw_circle(&mut atlas, x_offset + 32, y_offset + 16, 8, [0, 100, 0, 255]);
                }
                "Desert" => {
                    // Add cactus
                    draw_rect(&mut atlas, x_offset + 30, y_offset + 12, 4, 8, [0, 150, 0, 255]);
                }
                "Ocean" => {
                    // Add wave pattern
                    for i in 0..3 {
                        let wave_x = x_offset + 20 + i * 8;
                        let wave_y = y_offset + 14 + (i % 2) * 2;
                        draw_line(&mut atlas, wave_x, wave_y, wave_x + 6, wave_y, [255, 255, 255, 100]);
                    }
                }
                _ => {}
            }
        }
    }
    
    atlas.save("assets/sprites/terrain/terrain_atlas.png")?;
    println!("Created terrain atlas (320x128)");
    Ok(())
}

fn generate_particle_sprites() -> Result<(), Box<dyn std::error::Error>> {
    let particles = [
        ("heart", [255, 100, 100, 255]),
        ("zzz", [100, 100, 255, 255]),
        ("sparkle", [255, 255, 100, 255]),
        ("sweat", [100, 200, 255, 255]),
        ("exclamation", [255, 200, 0, 255]),
        ("question", [100, 100, 255, 255]),
    ];
    
    for (name, color) in particles.iter() {
        let mut img = RgbaImage::new(24, 24);
        
        match *name {
            "heart" => {
                // Draw heart shape
                draw_circle(&mut img, 8, 9, 4, *color);
                draw_circle(&mut img, 16, 9, 4, *color);
                // Draw triangle for bottom
                for y in 0..8 {
                    for x in 0..16 - y * 2 {
                        let px = 4 + y + x;
                        let py = 13 + y;
                        if px < 24 && py < 24 {
                            img.put_pixel(px, py, Rgba(*color));
                        }
                    }
                }
            }
            "sparkle" => {
                // Draw star/sparkle
                draw_line(&mut img, 12, 2, 12, 22, *color);
                draw_line(&mut img, 2, 12, 22, 12, *color);
                draw_line(&mut img, 5, 5, 19, 19, *color);
                draw_line(&mut img, 19, 5, 5, 19, *color);
            }
            "zzz" => {
                // Draw Z shapes
                for i in 0..3 {
                    let offset = i * 6;
                    draw_line(&mut img, 6 + offset, 6, 12 + offset, 6, *color);
                    draw_line(&mut img, 12 + offset, 6, 6 + offset, 12, *color);
                    draw_line(&mut img, 6 + offset, 12, 12 + offset, 12, *color);
                }
            }
            _ => {
                // Simple circle for others
                draw_circle(&mut img, 12, 12, 10, *color);
            }
        }
        
        img.save(format!("assets/sprites/particles/{}.png", name))?;
        println!("Created particle sprite: {}", name);
    }
    
    Ok(())
}

// Helper functions
fn draw_circle(img: &mut RgbaImage, cx: u32, cy: u32, radius: u32, color: [u8; 4]) {
    let r = radius as i32;
    for y in -r..=r {
        for x in -r..=r {
            if x * x + y * y <= r * r {
                let px = (cx as i32 + x) as u32;
                let py = (cy as i32 + y) as u32;
                if px < img.width() && py < img.height() {
                    img.put_pixel(px, py, Rgba(color));
                }
            }
        }
    }
}

fn draw_circle_outline(img: &mut RgbaImage, cx: u32, cy: u32, radius: u32, color: [u8; 4]) {
    let r = radius as i32;
    for y in -r..=r {
        for x in -r..=r {
            let dist_sq = x * x + y * y;
            if dist_sq <= r * r && dist_sq >= (r - 1) * (r - 1) {
                let px = (cx as i32 + x) as u32;
                let py = (cy as i32 + y) as u32;
                if px < img.width() && py < img.height() {
                    img.put_pixel(px, py, Rgba(color));
                }
            }
        }
    }
}

fn draw_rect(img: &mut RgbaImage, x: u32, y: u32, w: u32, h: u32, color: [u8; 4]) {
    for dy in 0..h {
        for dx in 0..w {
            let px = x + dx;
            let py = y + dy;
            if px < img.width() && py < img.height() {
                img.put_pixel(px, py, Rgba(color));
            }
        }
    }
}

fn draw_line(img: &mut RgbaImage, x0: u32, y0: u32, x1: u32, y1: u32, color: [u8; 4]) {
    let dx = (x1 as i32 - x0 as i32).abs();
    let dy = (y1 as i32 - y0 as i32).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx - dy;
    let mut x = x0 as i32;
    let mut y = y0 as i32;
    
    loop {
        if x >= 0 && y >= 0 && x < img.width() as i32 && y < img.height() as i32 {
            img.put_pixel(x as u32, y as u32, Rgba(color));
        }
        
        if x == x1 as i32 && y == y1 as i32 {
            break;
        }
        
        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }
}

fn draw_isometric_tile(img: &mut RgbaImage, x: u32, y: u32, w: u32, h: u32, color: [u8; 4]) {
    let hw = w / 2;
    let hh = h / 2;
    
    // Draw filled isometric diamond
    for dy in 0..h {
        let y_ratio = dy as f32 / h as f32;
        let width = if dy < hh {
            (dy as f32 / hh as f32 * hw as f32) as u32
        } else {
            ((1.0 - (dy - hh) as f32 / hh as f32) * hw as f32) as u32
        };
        
        for dx in 0..width * 2 {
            let px = x + hw - width + dx;
            let py = y + dy;
            if px < img.width() && py < img.height() {
                img.put_pixel(px, py, Rgba(color));
            }
        }
    }
    
    // Draw outline
    draw_line(img, x + hw, y, x + w - 1, y + hh, [0, 0, 0, 255]);
    draw_line(img, x + w - 1, y + hh, x + hw, y + h - 1, [0, 0, 0, 255]);
    draw_line(img, x + hw, y + h - 1, x, y + hh, [0, 0, 0, 255]);
    draw_line(img, x, y + hh, x + hw, y, [0, 0, 0, 255]);
}

fn darken_color(color: [u8; 4]) -> [u8; 4] {
    [
        (color[0] as f32 * 0.7) as u8,
        (color[1] as f32 * 0.7) as u8,
        (color[2] as f32 * 0.7) as u8,
        color[3],
    ]
}

fn lighten_color(color: [u8; 4]) -> [u8; 4] {
    [
        ((color[0] as f32 * 1.3).min(255.0)) as u8,
        ((color[1] as f32 * 1.3).min(255.0)) as u8,
        ((color[2] as f32 * 1.3).min(255.0)) as u8,
        color[3],
    ]
}