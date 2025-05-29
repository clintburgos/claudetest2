#!/usr/bin/env python3
"""Generate placeholder UI assets for Phase 4."""

from PIL import Image, ImageDraw
import os

def create_health_bars():
    """Create health bar UI components."""
    # Background
    bg = Image.new('RGBA', (100, 20), (50, 50, 50, 255))
    bg.save('assets/sprites/ui/health_bar_bg.png')
    
    # Fill (green gradient)
    fill = Image.new('RGBA', (100, 20), (0, 0, 0, 0))
    draw = ImageDraw.Draw(fill)
    for i in range(100):
        green = int(255 - (i * 1.5))
        red = int(i * 1.5)
        draw.line([(i, 0), (i, 20)], fill=(red, green, 0, 255))
    fill.save('assets/sprites/ui/health_bar_fill.png')
    
    # Frame
    frame = Image.new('RGBA', (100, 20), (0, 0, 0, 0))
    draw = ImageDraw.Draw(frame)
    draw.rectangle((0, 0, 99, 19), outline=(200, 200, 200, 255), width=2)
    frame.save('assets/sprites/ui/health_bar_frame.png')
    
    print("Created health bar components")

def create_speech_bubbles():
    """Create speech bubble variations."""
    bubbles = [
        ("speech_bubble", (255, 255, 255), "rounded"),
        ("thought_bubble", (240, 240, 255), "cloud"),
        ("shout_bubble", (255, 240, 240), "jagged"),
    ]
    
    for name, color, style in bubbles:
        img = Image.new('RGBA', (128, 96), (0, 0, 0, 0))
        draw = ImageDraw.Draw(img)
        
        if style == "rounded":
            # Regular speech bubble
            draw.rounded_rectangle((10, 10, 118, 70), radius=15, 
                                 fill=color + (255,), outline=(0, 0, 0, 255), width=2)
            # Tail
            draw.polygon([(30, 65), (40, 65), (25, 85)], 
                       fill=color + (255,), outline=(0, 0, 0, 255))
        
        elif style == "cloud":
            # Thought bubble with cloud edges
            # Main bubble
            draw.ellipse((15, 15, 113, 65), fill=color + (255,), outline=(0, 0, 0, 255))
            # Cloud puffs
            for x, y, size in [(20, 20, 15), (30, 15, 18), (50, 12, 20), 
                             (70, 15, 18), (90, 20, 15), (100, 30, 15),
                             (95, 45, 15), (85, 55, 15), (65, 60, 15),
                             (45, 60, 15), (25, 55, 15), (15, 45, 15)]:
                draw.ellipse((x-size//2, y-size//2, x+size//2, y+size//2), 
                           fill=color + (255,), outline=(0, 0, 0, 255))
            # Thought dots
            for i, size in enumerate([8, 6, 4]):
                y = 75 + i * 10
                draw.ellipse((30-size//2, y-size//2, 30+size//2, y+size//2), 
                           fill=color + (255,), outline=(0, 0, 0, 255))
        
        elif style == "jagged":
            # Shout bubble with jagged edges
            points = []
            import math
            center_x, center_y = 64, 40
            for i in range(20):
                angle = (i / 20) * 2 * math.pi
                radius = 45 if i % 2 == 0 else 35
                x = center_x + radius * math.cos(angle)
                y = center_y + radius * math.sin(angle) * 0.7
                points.append((x, y))
            draw.polygon(points, fill=color + (255,), outline=(0, 0, 0, 255), width=2)
            # Tail
            draw.polygon([(50, 60), (60, 60), (45, 85)], 
                       fill=color + (255,), outline=(0, 0, 0, 255))
        
        img.save(f'assets/sprites/ui/{name}.png')
    
    print("Created speech bubbles")

def create_emoji_atlas():
    """Create emoji atlas for creature expressions."""
    # 4x4 grid of 32x32 emojis = 128x128 atlas
    atlas = Image.new('RGBA', (128, 128), (0, 0, 0, 0))
    draw = ImageDraw.Draw(atlas)
    
    emojis = [
        # Row 1: Basic emotions
        ("happy", (255, 220, 0)),
        ("sad", (100, 150, 255)),
        ("angry", (255, 100, 100)),
        ("neutral", (200, 200, 200)),
        # Row 2: States
        ("sleepy", (180, 180, 255)),
        ("hungry", (255, 200, 100)),
        ("love", (255, 150, 200)),
        ("sick", (150, 255, 150)),
        # Row 3: Actions
        ("eating", (255, 180, 100)),
        ("drinking", (100, 200, 255)),
        ("working", (200, 150, 100)),
        ("playing", (255, 100, 255)),
        # Row 4: Reactions
        ("surprised", (255, 255, 100)),
        ("confused", (200, 100, 255)),
        ("excited", (255, 200, 0)),
        ("tired", (150, 150, 150)),
    ]
    
    for idx, (name, color) in enumerate(emojis):
        row = idx // 4
        col = idx % 4
        x = col * 32
        y = row * 32
        
        # Draw face circle
        draw.ellipse((x + 4, y + 4, x + 28, y + 28), 
                   fill=color + (255,), outline=(0, 0, 0, 255))
        
        # Draw eyes
        if name == "sleepy":
            draw.line((x + 10, y + 12, x + 14, y + 12), fill=(0, 0, 0, 255), width=2)
            draw.line((x + 18, y + 12, x + 22, y + 12), fill=(0, 0, 0, 255), width=2)
        elif name in ["happy", "excited"]:
            draw.arc((x + 9, y + 10, x + 15, y + 16), 0, 180, fill=(0, 0, 0, 255), width=2)
            draw.arc((x + 17, y + 10, x + 23, y + 16), 0, 180, fill=(0, 0, 0, 255), width=2)
        else:
            draw.ellipse((x + 10, y + 11, x + 14, y + 15), fill=(0, 0, 0, 255))
            draw.ellipse((x + 18, y + 11, x + 22, y + 15), fill=(0, 0, 0, 255))
        
        # Draw mouth
        if name == "happy":
            draw.arc((x + 10, y + 16, x + 22, y + 24), 0, 180, fill=(0, 0, 0, 255), width=2)
        elif name == "sad":
            draw.arc((x + 10, y + 20, x + 22, y + 28), 180, 360, fill=(0, 0, 0, 255), width=2)
        elif name == "angry":
            draw.line((x + 10, y + 20, x + 22, y + 20), fill=(0, 0, 0, 255), width=2)
        elif name == "surprised":
            draw.ellipse((x + 14, y + 18, x + 18, y + 24), fill=(0, 0, 0, 255))
        
    atlas.save('assets/sprites/ui/emoji_atlas.png')
    print("Created emoji atlas")

def create_ui_icons():
    """Create UI icons for needs and states."""
    icons = [
        # Needs
        ("hunger", (255, 150, 50), "food"),
        ("thirst", (100, 200, 255), "droplet"),
        ("energy", (255, 255, 100), "lightning"),
        ("social", (255, 100, 255), "people"),
        # States
        ("sleeping", (150, 150, 255), "moon"),
        ("eating", (200, 150, 100), "food"),
        ("talking", (100, 255, 100), "speech"),
        ("working", (255, 200, 100), "gear"),
        ("alert", (255, 100, 100), "exclamation"),
    ]
    
    for name, color, icon_type in icons:
        img = Image.new('RGBA', (32, 32), (0, 0, 0, 0))
        draw = ImageDraw.Draw(img)
        
        if icon_type == "food":
            # Apple shape
            draw.ellipse((8, 10, 24, 26), fill=color + (255,), outline=(0, 0, 0, 255))
            draw.rectangle((14, 6, 18, 12), fill=(139, 69, 19, 255))
        
        elif icon_type == "droplet":
            # Water drop
            draw.polygon([(16, 6), (24, 18), (16, 26), (8, 18)], 
                       fill=color + (255,), outline=(0, 0, 0, 255))
        
        elif icon_type == "lightning":
            # Lightning bolt
            draw.polygon([(12, 6), (20, 14), (16, 14), (20, 26), (12, 18), (16, 18)], 
                       fill=color + (255,), outline=(0, 0, 0, 255))
        
        elif icon_type == "people":
            # Two simple figures
            draw.ellipse((6, 8, 12, 14), fill=color + (255,), outline=(0, 0, 0, 255))
            draw.ellipse((20, 8, 26, 14), fill=color + (255,), outline=(0, 0, 0, 255))
            draw.rectangle((7, 14, 11, 24), fill=color + (255,), outline=(0, 0, 0, 255))
            draw.rectangle((21, 14, 25, 24), fill=color + (255,), outline=(0, 0, 0, 255))
        
        elif icon_type == "moon":
            # Crescent moon
            draw.ellipse((8, 8, 24, 24), fill=color + (255,), outline=(0, 0, 0, 255))
            draw.ellipse((12, 6, 28, 22), fill=(0, 0, 0, 0))
        
        elif icon_type == "speech":
            # Speech bubble
            draw.ellipse((6, 6, 26, 20), fill=color + (255,), outline=(0, 0, 0, 255))
            draw.polygon([(12, 18), (16, 18), (10, 26)], 
                       fill=color + (255,), outline=(0, 0, 0, 255))
        
        elif icon_type == "gear":
            # Simple gear
            draw.ellipse((10, 10, 22, 22), fill=color + (255,), outline=(0, 0, 0, 255))
            for angle in range(0, 360, 45):
                import math
                rad = math.radians(angle)
                x1 = 16 + 6 * math.cos(rad)
                y1 = 16 + 6 * math.sin(rad)
                x2 = 16 + 10 * math.cos(rad)
                y2 = 16 + 10 * math.sin(rad)
                draw.rectangle((x1-2, y1-2, x2+2, y2+2), fill=color + (255,))
        
        elif icon_type == "exclamation":
            # Exclamation mark
            draw.rectangle((14, 6, 18, 20), fill=color + (255,), outline=(0, 0, 0, 255))
            draw.ellipse((14, 24, 18, 28), fill=color + (255,), outline=(0, 0, 0, 255))
        
        img.save(f'assets/sprites/ui/icons/{name}.png')
    
    print("Created UI icons")

def create_particle_atlas():
    """Create particle effect atlas."""
    # 4x4 grid of 32x32 particles = 128x128 atlas
    atlas = Image.new('RGBA', (128, 128), (0, 0, 0, 0))
    draw = ImageDraw.Draw(atlas)
    
    particles = [
        # Row 1: Basic particles
        ("spark", (255, 255, 100)),
        ("smoke", (150, 150, 150)),
        ("bubble", (100, 200, 255)),
        ("leaf", (100, 200, 50)),
        # Row 2: Effects
        ("star", (255, 255, 255)),
        ("heart", (255, 100, 150)),
        ("music", (200, 100, 255)),
        ("dust", (200, 180, 150)),
        # Row 3: Status effects
        ("poison", (100, 255, 100)),
        ("ice", (200, 230, 255)),
        ("fire", (255, 150, 50)),
        ("electric", (255, 255, 150)),
        # Row 4: Misc
        ("sweat", (150, 200, 255)),
        ("anger", (255, 100, 100)),
        ("confetti", (255, 100, 255)),
        ("glow", (255, 255, 200)),
    ]
    
    for idx, (name, color) in enumerate(particles):
        row = idx // 4
        col = idx % 4
        x = col * 32 + 16
        y = row * 32 + 16
        
        if name == "spark":
            # Star burst
            for i in range(8):
                import math
                angle = i * math.pi / 4
                x2 = x + 12 * math.cos(angle)
                y2 = y + 12 * math.sin(angle)
                draw.line((x, y, x2, y2), fill=color + (255,), width=2)
        
        elif name in ["smoke", "dust"]:
            # Puffy cloud
            for dx, dy, size in [(0, 0, 12), (-5, -5, 8), (5, -5, 8), (-5, 5, 8), (5, 5, 8)]:
                draw.ellipse((x + dx - size//2, y + dy - size//2, 
                            x + dx + size//2, y + dy + size//2), 
                           fill=color + (100,))
        
        elif name == "bubble":
            # Transparent bubble
            draw.ellipse((x - 10, y - 10, x + 10, y + 10), 
                       fill=color + (50,), outline=color + (200,), width=2)
            draw.ellipse((x - 5, y - 8, x - 2, y - 5), fill=(255, 255, 255, 200))
        
        elif name == "star":
            # Five-pointed star
            import math
            points = []
            for i in range(10):
                angle = math.pi * 2 * i / 10 - math.pi / 2
                radius = 10 if i % 2 == 0 else 5
                px = x + radius * math.cos(angle)
                py = y + radius * math.sin(angle)
                points.append((px, py))
            draw.polygon(points, fill=color + (255,))
        
        elif name == "heart":
            # Heart shape
            draw.ellipse((x - 8, y - 6, x - 2, y + 2), fill=color + (255,))
            draw.ellipse((x + 2, y - 6, x + 8, y + 2), fill=color + (255,))
            draw.polygon([(x, y + 8), (x - 8, y), (x + 8, y)], fill=color + (255,))
        
        else:
            # Default circular particle
            draw.ellipse((x - 8, y - 8, x + 8, y + 8), fill=color + (180,))
    
    atlas.save('assets/sprites/particles/particle_atlas.png')
    print("Created particle atlas")

def main():
    """Generate all UI assets."""
    # Ensure directories exist
    os.makedirs('assets/sprites/ui', exist_ok=True)
    os.makedirs('assets/sprites/ui/icons', exist_ok=True)
    os.makedirs('assets/sprites/particles', exist_ok=True)
    
    create_health_bars()
    create_speech_bubbles()
    create_emoji_atlas()
    create_ui_icons()
    create_particle_atlas()
    
    print("\nUI assets created successfully!")
    print("These are placeholder assets for testing.")
    print("Replace with actual artwork for production.")

if __name__ == "__main__":
    main()