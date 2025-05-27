import matplotlib.pyplot as plt
import matplotlib.patches as patches
from matplotlib.patches import FancyBboxPatch, Circle, Ellipse, Polygon
import numpy as np

# Create figure and axis
fig, ax = plt.subplots(1, 1, figsize=(16, 10))
ax.set_xlim(0, 1600)
ax.set_ylim(0, 1000)
ax.set_aspect('equal')

# Hide axes
ax.axis('off')

# Background gradient (sky)
gradient = np.linspace(0.9, 0.7, 256).reshape(1, -1)
gradient = np.repeat(gradient, 256, axis=0)
ax.imshow(gradient, extent=[0, 1600, 0, 1000], aspect='auto', cmap='Blues_r', alpha=0.3)

# Function to convert world to isometric coordinates
def world_to_iso(x, y):
    iso_x = (x - y) * 0.5
    iso_y = (x + y) * 0.25
    return iso_x + 800, iso_y + 200

# Draw isometric terrain tiles
def draw_iso_tile(x, y, color, biome_type='grass'):
    ix, iy = world_to_iso(x, y)
    
    # Tile shape (diamond)
    tile = Polygon([
        (ix, iy),
        (ix + 32, iy + 16),
        (ix, iy + 32),
        (ix - 32, iy + 16)
    ], closed=True, facecolor=color, edgecolor='#2a2a2a', linewidth=0.5)
    ax.add_patch(tile)
    
    # Add texture details
    if biome_type == 'grass':
        # Grass blades
        for i in range(3):
            offset_x = np.random.randint(-10, 10)
            offset_y = np.random.randint(-5, 5)
            grass = plt.Line2D([ix + offset_x, ix + offset_x], 
                             [iy + offset_y + 16, iy + offset_y + 20],
                             color='#2d5016', linewidth=1)
            ax.add_line(grass)
    elif biome_type == 'desert':
        # Sand dots
        for i in range(5):
            dot = Circle((ix + np.random.randint(-15, 15), 
                         iy + np.random.randint(-5, 5) + 16), 
                        radius=1, color='#d4a574')
            ax.add_patch(dot)

# Draw biomes
# Forest biome (left side)
for x in range(100, 600, 64):
    for y in range(100, 600, 64):
        shade = '#3a7d44' if (x + y) % 128 == 0 else '#4a8b54'
        draw_iso_tile(x, y, shade, 'grass')

# Desert biome (right side)
for x in range(700, 1200, 64):
    for y in range(100, 600, 64):
        shade = '#f4d03f' if (x + y) % 128 == 0 else '#fad643'
        draw_iso_tile(x, y, shade, 'desert')

# Transition zone
for x in range(600, 700, 64):
    for y in range(100, 600, 64):
        draw_iso_tile(x, y, '#8b9556', 'grass')

# Draw trees in forest
def draw_tree(x, y):
    ix, iy = world_to_iso(x, y)
    
    # Tree trunk
    trunk = FancyBboxPatch((ix - 8, iy), 16, 40,
                           boxstyle="round,pad=0.1",
                           facecolor='#654321', edgecolor='#2a2a2a', linewidth=2)
    ax.add_patch(trunk)
    
    # Tree foliage (cartoon style)
    for i, (size, offset_y) in enumerate([(40, 40), (35, 55), (25, 65)]):
        foliage = Circle((ix, iy + offset_y), size,
                        facecolor='#2d5016', edgecolor='#1a3009', linewidth=2)
        ax.add_patch(foliage)

# Add some trees
tree_positions = [(200, 200), (350, 150), (450, 300), (250, 400)]
for pos in tree_positions:
    draw_tree(pos[0], pos[1])

# Draw cartoon creatures
def draw_creature(x, y, species='herbivore', emotion='happy', action=None):
    ix, iy = world_to_iso(x, y)
    
    # Shadow
    shadow = Ellipse((ix, iy - 5), 40, 20, facecolor='black', alpha=0.2)
    ax.add_patch(shadow)
    
    # Body
    if species == 'herbivore':
        body_color = '#7cb342'
        size = 30
    elif species == 'carnivore':
        body_color = '#e74c3c'
        size = 35
    else:  # omnivore
        body_color = '#8b6914'
        size = 32
    
    # Main body (round and cute)
    body = Circle((ix, iy + 20), size, facecolor=body_color, 
                  edgecolor='#2a2a2a', linewidth=3)
    ax.add_patch(body)
    
    # Head
    head = Circle((ix, iy + 50), size * 0.8, facecolor=body_color,
                  edgecolor='#2a2a2a', linewidth=3)
    ax.add_patch(head)
    
    # Large expressive eyes (40% of head)
    eye_size = size * 0.32
    left_eye = Circle((ix - 10, iy + 52), eye_size, facecolor='white',
                      edgecolor='#2a2a2a', linewidth=2)
    right_eye = Circle((ix + 10, iy + 52), eye_size, facecolor='white',
                       edgecolor='#2a2a2a', linewidth=2)
    ax.add_patch(left_eye)
    ax.add_patch(right_eye)
    
    # Pupils showing emotion
    if emotion == 'happy':
        pupil_y = iy + 52
        # Sparkles in eyes
        sparkle1 = Circle((ix - 10, iy + 55), 2, facecolor='white')
        sparkle2 = Circle((ix + 10, iy + 55), 2, facecolor='white')
        ax.add_patch(sparkle1)
        ax.add_patch(sparkle2)
    elif emotion == 'scared':
        pupil_y = iy + 50  # Looking up
    elif emotion == 'sad':
        pupil_y = iy + 49  # Looking down
    else:
        pupil_y = iy + 52
    
    left_pupil = Circle((ix - 10, pupil_y), eye_size * 0.5, facecolor='black')
    right_pupil = Circle((ix + 10, pupil_y), eye_size * 0.5, facecolor='black')
    ax.add_patch(left_pupil)
    ax.add_patch(right_pupil)
    
    # Emotion-specific features
    if emotion == 'happy':
        # Smile
        smile = patches.Arc((ix, iy + 45), 20, 15, angle=0, theta1=200, theta2=340,
                           linewidth=3, color='#2a2a2a')
        ax.add_patch(smile)
        # Blush
        blush1 = Circle((ix - 20, iy + 45), 5, facecolor='#ff6b6b', alpha=0.5)
        blush2 = Circle((ix + 20, iy + 45), 5, facecolor='#ff6b6b', alpha=0.5)
        ax.add_patch(blush1)
        ax.add_patch(blush2)
    elif emotion == 'sad':
        # Frown
        frown = patches.Arc((ix, iy + 40), 20, 15, angle=0, theta1=20, theta2=160,
                           linewidth=3, color='#2a2a2a')
        ax.add_patch(frown)
        # Tear
        tear = Ellipse((ix - 15, iy + 45), 3, 6, facecolor='#4fc3f7')
        ax.add_patch(tear)
    elif emotion == 'angry':
        # Angry eyebrows
        brow1 = plt.Line2D([ix - 15, ix - 5], [iy + 60, iy + 58], 
                          color='#2a2a2a', linewidth=4)
        brow2 = plt.Line2D([ix + 5, ix + 15], [iy + 58, iy + 60], 
                          color='#2a2a2a', linewidth=4)
        ax.add_line(brow1)
        ax.add_line(brow2)
    
    # Action-specific additions
    if action == 'eating':
        # Berry in mouth
        berry = Circle((ix, iy + 40), 5, facecolor='#e74c3c')
        ax.add_patch(berry)
    elif action == 'sleeping':
        # Closed eyes
        left_eye.set_facecolor(body_color)
        right_eye.set_facecolor(body_color)
        # Z particles
        ax.text(ix + 30, iy + 70, 'Z', fontsize=20, weight='bold', color='#4fc3f7')
        ax.text(ix + 40, iy + 80, 'z', fontsize=16, weight='bold', color='#4fc3f7')
        ax.text(ix + 48, iy + 88, 'z', fontsize=12, weight='bold', color='#4fc3f7')
    elif action == 'talking':
        # Speech bubble
        bubble = FancyBboxPatch((ix + 40, iy + 60), 80, 40,
                               boxstyle="round,pad=0.1",
                               facecolor='white', edgecolor='#2a2a2a', linewidth=2)
        ax.add_patch(bubble)
        # Tail pointing to speaker
        tail = Polygon([(ix + 40, iy + 65), (ix + 30, iy + 55), (ix + 45, iy + 60)],
                      facecolor='white', edgecolor='#2a2a2a', linewidth=2)
        ax.add_patch(tail)
        # Heart emoji in bubble
        ax.text(ix + 65, iy + 75, '❤️', fontsize=16)
        ax.text(ix + 85, iy + 75, '😊', fontsize=16)
    
    # Genetic variations
    if np.random.random() > 0.5:
        # Spots pattern
        for i in range(3):
            spot_x = ix + np.random.randint(-15, 15)
            spot_y = iy + np.random.randint(10, 30)
            spot = Circle((spot_x, spot_y), 3, facecolor='#2a2a2a', alpha=0.3)
            ax.add_patch(spot)
    
    return ix, iy

# Place creatures
creatures = [
    (300, 350, 'herbivore', 'happy', 'eating'),
    (400, 400, 'herbivore', 'happy', 'talking'),
    (500, 250, 'carnivore', 'angry', None),
    (150, 450, 'omnivore', 'sad', None),
    (350, 500, 'herbivore', 'happy', 'sleeping'),
    (900, 300, 'herbivore', 'scared', None),
    (1000, 350, 'carnivore', 'happy', None),
]

for x, y, species, emotion, action in creatures:
    draw_creature(x, y, species, emotion, action)

# Draw resources
def draw_resource(x, y, resource_type):
    ix, iy = world_to_iso(x, y)
    
    if resource_type == 'berries':
        # Berry bush
        bush = Circle((ix, iy + 10), 15, facecolor='#2d5016', 
                     edgecolor='#1a3009', linewidth=2)
        ax.add_patch(bush)
        # Berries
        for i in range(5):
            berry_x = ix + np.random.randint(-10, 10)
            berry_y = iy + np.random.randint(5, 15)
            berry = Circle((berry_x, berry_y), 3, facecolor='#e74c3c',
                          edgecolor='#8b0000', linewidth=1)
            ax.add_patch(berry)
    elif resource_type == 'water':
        # Water puddle
        water = Ellipse((ix, iy), 40, 20, facecolor='#4fc3f7', 
                       alpha=0.7, edgecolor='#1e88e5', linewidth=2)
        ax.add_patch(water)
        # Ripples
        ripple = Ellipse((ix, iy), 50, 25, fill=False,
                        edgecolor='#1e88e5', linewidth=1, alpha=0.5)
        ax.add_patch(ripple)
    elif resource_type == 'cactus':
        # Desert cactus with water
        cactus_body = FancyBboxPatch((ix - 10, iy), 20, 40,
                                    boxstyle="round,pad=0.1",
                                    facecolor='#2d5016', edgecolor='#1a3009', linewidth=2)
        ax.add_patch(cactus_body)
        # Spines
        for i in range(8):
            spine_y = iy + i * 5
            ax.plot([ix - 12, ix - 15], [spine_y, spine_y], 'k-', linewidth=1)
            ax.plot([ix + 12, ix + 15], [spine_y, spine_y], 'k-', linewidth=1)

# Place resources
draw_resource(250, 300, 'berries')
draw_resource(400, 200, 'water')
draw_resource(950, 400, 'cactus')

# Weather effect (light rain in forest)
for i in range(20):
    rain_x = np.random.randint(100, 600)
    rain_y = np.random.randint(600, 900)
    ax.plot([rain_x, rain_x - 5], [rain_y, rain_y - 20], 
           color='#4fc3f7', alpha=0.3, linewidth=1)

# UI Elements
# Health bar above a creature
creature_x, creature_y = world_to_iso(400, 400)
health_bg = FancyBboxPatch((creature_x - 20, creature_y + 80), 40, 6,
                          boxstyle="round,pad=0.1",
                          facecolor='#e74c3c', edgecolor='#2a2a2a', linewidth=1)
health_fg = FancyBboxPatch((creature_x - 20, creature_y + 80), 30, 6,
                          boxstyle="round,pad=0.1",
                          facecolor='#4caf50', edgecolor=None)
ax.add_patch(health_bg)
ax.add_patch(health_fg)

# Need icons
hunger_icon = Circle((creature_x - 30, creature_y + 70), 8, 
                    facecolor='#ff9800', edgecolor='#2a2a2a', linewidth=2)
ax.add_patch(hunger_icon)
ax.text(creature_x - 30, creature_y + 70, '🍖', fontsize=10, ha='center', va='center')

# Particle effects
# Hearts for bonding creatures
heart_x, heart_y = world_to_iso(400, 400)
for i in range(3):
    offset = i * 10
    ax.text(heart_x + 10, heart_y + 90 + offset, '❤️', 
           fontsize=12 - i*2, alpha=1 - i*0.3)

# Day/night indicator (sun)
sun = Circle((1400, 850), 50, facecolor='#ffd54f', 
            edgecolor='#ffa000', linewidth=3)
ax.add_patch(sun)
# Sun rays
for angle in range(0, 360, 45):
    rad = np.radians(angle)
    x1, y1 = 1400 + 60 * np.cos(rad), 850 + 60 * np.sin(rad)
    x2, y2 = 1400 + 80 * np.cos(rad), 850 + 80 * np.sin(rad)
    ax.plot([x1, x2], [y1, y2], color='#ffa000', linewidth=3)

# UI Panel (bottom right)
panel = FancyBboxPatch((1200, 50), 350, 200,
                      boxstyle="round,pad=0.02",
                      facecolor='white', edgecolor='#2a2a2a', 
                      linewidth=3, alpha=0.9)
ax.add_patch(panel)

# Panel content
ax.text(1220, 220, 'Selected: Happy Herbivore', fontsize=14, weight='bold')
ax.text(1220, 190, 'Age: 12 days', fontsize=12)
ax.text(1220, 170, 'Health: 75%', fontsize=12)
ax.text(1220, 150, 'Hunger: 30%', fontsize=12)
ax.text(1220, 130, 'Thirst: 45%', fontsize=12)
ax.text(1220, 110, 'Energy: 80%', fontsize=12)
ax.text(1220, 90, 'Genetics: Fast, Social', fontsize=12, style='italic')
ax.text(1220, 70, 'Current: Socializing', fontsize=12, color='#4caf50')

# Title
ax.text(800, 950, 'Creature Evolution Sim - Cartoon Isometric View', 
       fontsize=24, weight='bold', ha='center',
       bbox=dict(boxstyle="round,pad=0.3", facecolor='white', alpha=0.8))

# Mini-map (top right)
minimap = FancyBboxPatch((1400, 700), 150, 100,
                        boxstyle="round,pad=0.02",
                        facecolor='#f5f5f5', edgecolor='#2a2a2a', 
                        linewidth=2, alpha=0.9)
ax.add_patch(minimap)
# Mini biomes
mini_forest = patches.Rectangle((1410, 750), 60, 40, facecolor='#4a8b54')
mini_desert = patches.Rectangle((1480, 750), 60, 40, facecolor='#fad643')
ax.add_patch(mini_forest)
ax.add_patch(mini_desert)
# Player view indicator
view_rect = patches.Rectangle((1450, 760), 50, 20, 
                             fill=False, edgecolor='red', linewidth=2)
ax.add_patch(view_rect)

plt.tight_layout()
plt.savefig('/Users/clintonburgos/Documents/Code/claudetest2/docs/implementation/cartoon_isometric_mockup.png', 
            dpi=150, bbox_inches='tight', facecolor='#87ceeb')
plt.close()

print("Mockup saved to cartoon_isometric_mockup.png")