#!/bin/bash
# Create UI sprite assets for Phase 4

# Create directories
mkdir -p assets/sprites/ui/icons
mkdir -p assets/sprites/particles

echo "Creating UI assets with ImageMagick..."

# Health bar components
convert -size 100x20 xc:'#323232' assets/sprites/ui/health_bar_bg.png
convert -size 100x20 gradient:'#00ff00-#ff0000' -rotate 180 assets/sprites/ui/health_bar_fill.png
convert -size 100x20 xc:transparent -stroke '#c8c8c8' -strokewidth 2 -fill none -draw "rectangle 1,1 98,18" assets/sprites/ui/health_bar_frame.png

# Speech bubbles
# Regular speech bubble
convert -size 128x96 xc:transparent \
    -fill white -stroke black -strokewidth 2 \
    -draw "roundrectangle 10,10 118,70 15,15" \
    -draw "polygon 30,65 40,65 25,85" \
    assets/sprites/ui/speech_bubble.png

# Thought bubble
convert -size 128x96 xc:transparent \
    -fill '#f0f0ff' -stroke black -strokewidth 1 \
    -draw "ellipse 64,40 50,25" \
    -draw "circle 30,30 40,30" \
    -draw "circle 40,25 50,25" \
    -draw "circle 60,22 70,22" \
    -draw "circle 80,25 90,25" \
    -draw "circle 90,35 100,35" \
    -draw "circle 85,50 95,50" \
    -draw "circle 70,55 80,55" \
    -draw "circle 50,55 60,55" \
    -draw "circle 35,50 45,50" \
    -draw "circle 25,40 35,40" \
    -draw "circle 30,75 34,75" \
    -draw "circle 30,85 32,85" \
    assets/sprites/ui/thought_bubble.png

# Shout bubble
convert -size 128x96 xc:transparent \
    -fill '#fff0f0' -stroke black -strokewidth 2 \
    -draw "polygon 64,10 80,25 75,40 60,35 45,45 30,35 25,20 40,15 50,5 70,15" \
    -draw "polygon 50,60 60,60 45,85" \
    assets/sprites/ui/shout_bubble.png

# Emoji atlas (128x128 - 4x4 grid of 32x32 emojis)
convert -size 128x128 xc:transparent \
    -fill yellow -stroke black -strokewidth 1 \
    -draw "circle 16,16 28,16" \
    -fill black -draw "circle 10,12 12,12" -draw "circle 22,12 24,12" \
    -stroke black -strokewidth 2 -fill none -draw "arc 10,16 22,24 0,180" \
    -fill '#6496ff' -stroke black -strokewidth 1 \
    -draw "circle 48,16 60,16" \
    -fill black -draw "circle 42,12 44,12" -draw "circle 54,12 56,12" \
    -stroke black -strokewidth 2 -fill none -draw "arc 42,20 54,28 180,360" \
    -fill '#ff6464' -stroke black -strokewidth 1 \
    -draw "circle 80,16 92,16" \
    -fill black -draw "circle 74,12 76,12" -draw "circle 86,12 88,12" \
    -stroke black -strokewidth 2 -draw "line 74,20 86,20" \
    -fill '#c8c8c8' -stroke black -strokewidth 1 \
    -draw "circle 112,16 124,16" \
    -fill black -draw "circle 106,12 108,12" -draw "circle 118,12 120,12" \
    assets/sprites/ui/emoji_atlas.png

# UI Icons
# Hunger (apple)
convert -size 32x32 xc:transparent \
    -fill '#ff9632' -stroke black -strokewidth 1 \
    -draw "ellipse 16,18 8,8" \
    -fill '#8b4513' -draw "rectangle 14,6 18,12" \
    assets/sprites/ui/icons/hunger.png

# Thirst (water drop)
convert -size 32x32 xc:transparent \
    -fill '#64c8ff' -stroke black -strokewidth 1 \
    -draw "polygon 16,6 24,18 16,26 8,18" \
    assets/sprites/ui/icons/thirst.png

# Energy (lightning bolt)
convert -size 32x32 xc:transparent \
    -fill '#ffff64' -stroke black -strokewidth 1 \
    -draw "polygon 12,6 20,14 16,14 20,26 12,18 16,18" \
    assets/sprites/ui/icons/energy.png

# Social (two figures)
convert -size 32x32 xc:transparent \
    -fill '#ff64ff' -stroke black -strokewidth 1 \
    -draw "circle 9,11 12,11" -draw "rectangle 7,14 11,24" \
    -draw "circle 23,11 26,11" -draw "rectangle 21,14 25,24" \
    assets/sprites/ui/icons/social.png

# Sleeping (moon)
convert -size 32x32 xc:transparent \
    -fill '#9696ff' -stroke black -strokewidth 1 \
    -draw "circle 16,16 24,16" \
    -fill transparent -draw "circle 20,14 28,14" \
    assets/sprites/ui/icons/sleeping.png

# Eating (food)
convert -size 32x32 xc:transparent \
    -fill '#c89664' -stroke black -strokewidth 1 \
    -draw "ellipse 16,18 8,8" \
    assets/sprites/ui/icons/eating.png

# Talking (speech bubble)
convert -size 32x32 xc:transparent \
    -fill '#64ff64' -stroke black -strokewidth 1 \
    -draw "ellipse 16,13 10,7" \
    -draw "polygon 12,18 16,18 10,26" \
    assets/sprites/ui/icons/talking.png

# Working (gear)
convert -size 32x32 xc:transparent \
    -fill '#ffc864' -stroke black -strokewidth 1 \
    -draw "circle 16,16 22,16" \
    -fill '#ffc864' \
    -draw "rectangle 14,4 18,28" \
    -draw "rectangle 4,14 28,18" \
    -draw "polygon 8,8 10,10 22,22 24,24" \
    -draw "polygon 24,8 22,10 10,22 8,24" \
    assets/sprites/ui/icons/working.png

# Alert (exclamation)
convert -size 32x32 xc:transparent \
    -fill '#ff6464' -stroke black -strokewidth 1 \
    -draw "rectangle 14,6 18,20" \
    -draw "circle 16,26 18,26" \
    assets/sprites/ui/icons/alert.png

# Particle atlas (128x128 - 4x4 grid)
convert -size 128x128 xc:transparent \
    -fill '#ffff64' \
    -draw "line 16,4 16,28" -draw "line 4,16 28,16" \
    -draw "line 8,8 24,24" -draw "line 24,8 8,24" \
    -fill '#969696' -draw "circle 48,16 56,16" \
    -fill '#64c8ff' -draw "circle 80,16 88,16" \
    -fill white -draw "circle 83,11 85,11" \
    -fill '#64c832' -draw "polygon 112,10 118,16 112,22 106,16" \
    -fill white -draw "circle 48,48 56,48" \
    -fill '#ff6496' -draw "circle 76,44 80,44" -draw "circle 84,44 88,44" \
    -draw "polygon 80,56 72,48 88,48" \
    -fill '#c864ff' -draw "circle 112,48 116,48" \
    -fill '#c8b496' -draw "circle 16,80 20,80" \
    assets/sprites/particles/particle_atlas.png

echo "UI assets created successfully!"
echo "Created:"
echo "  - Health bar components (bg, fill, frame)"
echo "  - Speech bubbles (speech, thought, shout)"
echo "  - Emoji atlas (16 expressions)"
echo "  - UI icons (9 icons for needs and states)"
echo "  - Particle atlas (16 particle types)"