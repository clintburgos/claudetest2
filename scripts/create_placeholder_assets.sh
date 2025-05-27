#!/bin/bash
# Create placeholder sprite assets for testing

# Create directories
mkdir -p assets/sprites/creatures
mkdir -p assets/sprites/terrain
mkdir -p assets/sprites/particles

# Create placeholder images using ImageMagick convert (if available) or touch

# Check if ImageMagick is available
if command -v convert &> /dev/null; then
    echo "Creating placeholder sprites with ImageMagick..."
    
    # Creature atlas (384x384 - 8x8 grid of 48x48 sprites)
    convert -size 384x384 xc:lightgreen \
        -fill darkgreen -draw "circle 192,192 240,192" \
        -fill black -draw "circle 170,180 175,180" \
        -fill black -draw "circle 210,180 215,180" \
        assets/sprites/creatures/creature_atlas.png
    
    # Terrain atlas (320x128 - 5x4 grid of 64x32 tiles)  
    convert -size 320x128 xc:brown \
        -fill green -draw "rectangle 0,0 64,32" \
        -fill yellow -draw "rectangle 64,0 128,32" \
        -fill lightblue -draw "rectangle 128,0 192,32" \
        -fill white -draw "rectangle 192,0 256,32" \
        -fill blue -draw "rectangle 256,0 320,32" \
        assets/sprites/terrain/terrain_atlas.png
    
    # Particle sprites
    convert -size 24x24 xc:transparent -fill red -draw "circle 12,12 20,12" assets/sprites/particles/heart.png
    convert -size 24x24 xc:transparent -fill blue -draw "circle 12,12 20,12" assets/sprites/particles/zzz.png
    convert -size 24x24 xc:transparent -fill yellow -draw "circle 12,12 20,12" assets/sprites/particles/sparkle.png
    convert -size 24x24 xc:transparent -fill lightblue -draw "circle 12,12 20,12" assets/sprites/particles/sweat.png
    convert -size 24x24 xc:transparent -fill orange -draw "circle 12,12 20,12" assets/sprites/particles/exclamation.png
    convert -size 24x24 xc:transparent -fill purple -draw "circle 12,12 20,12" assets/sprites/particles/question.png
    
else
    echo "ImageMagick not found. Creating empty placeholder files..."
    
    # Just create empty files as placeholders
    touch assets/sprites/creatures/creature_atlas.png
    touch assets/sprites/terrain/terrain_atlas.png
    touch assets/sprites/particles/heart.png
    touch assets/sprites/particles/zzz.png
    touch assets/sprites/particles/sparkle.png
    touch assets/sprites/particles/sweat.png
    touch assets/sprites/particles/exclamation.png
    touch assets/sprites/particles/question.png
fi

echo "Placeholder assets created in assets/sprites/"
echo "Note: These are just empty or simple colored files for testing."
echo "Replace with actual sprite artwork for production."