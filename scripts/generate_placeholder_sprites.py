#!/usr/bin/env python3
"""Generate placeholder sprites for testing the cartoon isometric UI."""

from PIL import Image, ImageDraw
import os

def create_creature_atlas():
    """Create a simple creature sprite atlas."""
    # 8x8 grid of 48x48 sprites = 384x384 image
    atlas = Image.new('RGBA', (384, 384), (0, 0, 0, 0))
    draw = ImageDraw.Draw(atlas)
    
    # Draw simple creature shapes for different animations
    animations = [
        ("Idle", (255, 200, 200)),      # Pink
        ("Walk", (200, 255, 200)),      # Light green
        ("Run", (200, 200, 255)),       # Light blue
        ("Eat", (255, 255, 200)),       # Yellow
        ("Sleep", (200, 200, 200)),     # Gray
        ("Talk", (255, 200, 255)),      # Magenta
        ("Attack", (255, 100, 100)),    # Red
        ("Death", (100, 100, 100)),     # Dark gray
    ]
    
    for anim_idx, (anim_name, color) in enumerate(animations):
        # Each animation gets a row
        y_offset = anim_idx * 48
        
        # Draw 8 frames per animation
        for frame in range(8):
            x_offset = frame * 48
            
            # Draw a simple circle creature
            bbox = (x_offset + 8, y_offset + 8, x_offset + 40, y_offset + 40)
            draw.ellipse(bbox, fill=color, outline=(0, 0, 0, 255))
            
            # Add eyes
            eye_y = y_offset + 18
            draw.ellipse((x_offset + 14, eye_y, x_offset + 18, eye_y + 4), fill=(0, 0, 0, 255))
            draw.ellipse((x_offset + 30, eye_y, x_offset + 34, eye_y + 4), fill=(0, 0, 0, 255))
            
            # Add simple animation variation
            if anim_name == "Walk" and frame % 2 == 0:
                # Bounce effect
                draw.ellipse((x_offset + 20, y_offset + 35, x_offset + 28, y_offset + 43), 
                           fill=color, outline=(0, 0, 0, 255))
    
    atlas.save('assets/sprites/creatures/creature_atlas.png')
    print("Created creature atlas")

def create_terrain_atlas():
    """Create a simple terrain tile atlas."""
    # 5 biomes x 4 variants = 20 tiles
    # Using 64x32 isometric tiles in a 5x4 grid = 320x128
    atlas = Image.new('RGBA', (320, 128), (0, 0, 0, 0))
    draw = ImageDraw.Draw(atlas)
    
    biomes = [
        ("Forest", (34, 139, 34)),      # Forest green
        ("Desert", (237, 201, 175)),    # Sand
        ("Grassland", (124, 252, 0)),   # Grass green
        ("Tundra", (230, 230, 250)),    # Light blue-white
        ("Ocean", (0, 119, 190)),       # Ocean blue
    ]
    
    for biome_idx, (biome_name, color) in enumerate(biomes):
        x_offset = biome_idx * 64
        
        # Draw 4 variants per biome
        for variant in range(4):
            y_offset = variant * 32
            
            # Draw isometric diamond tile
            points = [
                (x_offset + 32, y_offset + 0),    # Top
                (x_offset + 63, y_offset + 16),   # Right
                (x_offset + 32, y_offset + 31),   # Bottom
                (x_offset + 0, y_offset + 16),    # Left
            ]
            
            # Vary the color slightly for each variant
            variant_color = tuple(
                min(255, max(0, c + (variant - 2) * 10)) 
                for c in color
            )
            
            draw.polygon(points, fill=variant_color + (255,), outline=(0, 0, 0, 255))
            
            # Add simple texture
            if biome_name == "Forest":
                # Add tree dots
                draw.ellipse((x_offset + 28, y_offset + 12, x_offset + 36, y_offset + 20), 
                           fill=(0, 100, 0, 255))
    
    atlas.save('assets/sprites/terrain/terrain_atlas.png')
    print("Created terrain atlas")

def create_particle_sprites():
    """Create simple particle effect sprites."""
    particles = [
        ("heart", (255, 100, 100), "♥"),
        ("zzz", (100, 100, 255), "Z"),
        ("sparkle", (255, 255, 100), "✦"),
        ("sweat", (100, 200, 255), "💧"),
        ("exclamation", (255, 200, 0), "!"),
        ("question", (100, 100, 255), "?"),
    ]
    
    for name, color, symbol in particles:
        img = Image.new('RGBA', (24, 24), (0, 0, 0, 0))
        draw = ImageDraw.Draw(img)
        
        # Draw simple shape
        if name == "heart":
            # Simple heart shape
            draw.ellipse((4, 6, 12, 14), fill=color)
            draw.ellipse((12, 6, 20, 14), fill=color)
            draw.polygon([(12, 18), (4, 10), (20, 10)], fill=color)
        elif name == "sparkle":
            # Star shape
            draw.line((12, 2, 12, 22), fill=color, width=3)
            draw.line((2, 12, 22, 12), fill=color, width=3)
            draw.line((5, 5, 19, 19), fill=color, width=2)
            draw.line((19, 5, 5, 19), fill=color, width=2)
        else:
            # Simple circle with symbol
            draw.ellipse((2, 2, 22, 22), fill=color)
        
        img.save(f'assets/sprites/particles/{name}.png')
    
    print("Created particle sprites")

def main():
    """Generate all placeholder sprites."""
    # Ensure directories exist
    os.makedirs('assets/sprites/creatures', exist_ok=True)
    os.makedirs('assets/sprites/terrain', exist_ok=True)
    os.makedirs('assets/sprites/particles', exist_ok=True)
    
    create_creature_atlas()
    create_terrain_atlas()
    create_particle_sprites()
    
    print("\nPlaceholder sprites created successfully!")
    print("These are simple colored shapes for testing.")
    print("Replace with actual artwork for production.")

if __name__ == "__main__":
    main()