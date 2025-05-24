# Controls & Interface Design

## Overview
The control system enables seamless navigation between macro-level population trends and micro-level creature interactions, with intuitive camera controls and time manipulation in an isometric world.

## Camera Controls

### Movement
- **WASD / Arrow Keys**: Pan camera across the world
  - Smooth acceleration/deceleration
  - Speed scales with zoom level (move faster when zoomed out)
- **Middle Mouse Drag**: Alternative pan method
- **Edge Scrolling**: Move mouse to screen edges (optional, togglable)
- **Minimap Click**: Jump to world location

### Zoom
- **Mouse Wheel**: Smooth zoom in/out
  - Exponential scaling for natural feel
  - 5 zoom levels:
    - **Macro** (1:100): See entire world, population heatmaps
    - **Regional** (1:50): Biome overview, migration patterns  
    - **Local** (1:20): Multiple creatures, resource distribution
    - **Close** (1:5): Individual creatures with names/stats
    - **Intimate** (1:1): Single creature detail, emotions visible
- **Z/X Keys**: Alternative zoom in/out
- **Double-click**: Smart zoom to creature or point of interest

### Camera Modes
1. **Free Camera** (default)
   - Manual control as described above
   
2. **Follow Mode** (F key)
   - Lock camera to selected creature
   - Maintains current zoom level
   - Smooth tracking with slight lag
   - Auto-disables on manual movement
   
3. **Cinematic Mode** (C key)
   - Slow automatic pan across world
   - Randomly focuses on interesting events
   - Great for passive observation

## Time Controls

### Speed Control Panel
Located in bottom-right, always visible:
```
[⏸️] [▶️] [⏩] [⏩⏩] [⏩⏩⏩] [📅]
 0x   1x   5x   20x   100x  1000x
```

- **Pause** (Spacebar): Freeze simulation
- **Normal** (1): Real-time speed
- **Fast** (2): 5x speed - watch daily routines
- **Faster** (3): 20x speed - see weekly patterns
- **Rapid** (4): 100x speed - monthly changes
- **Generational** (5): 1000x speed - evolutionary time

### Time Display
- **Current Time**: Day/Night indicator, season, year
- **Generation Counter**: Current average generation
- **Speed Indicator**: Current simulation speed
- **Events Timeline**: Scrollable history of major events

## Selection & Focus System

### Creature Selection
- **Left Click**: Select creature
  - Highlights with glowing outline
  - Opens creature info panel
  - Shows relationship lines to nearby creatures
  
- **Shift+Click**: Multi-select creatures
  - Compare multiple creatures
  - Track family groups
  
- **Double Click**: Focus camera and follow
  
- **Right Click**: Context menu
  - Follow this creature
  - View family tree
  - View relationships
  - View history
  - Pin to tracking list

### Selection Visualization
- **Selected Creature**: Bright pulsing outline
- **Family Members**: Matching color dim outline
- **Friends**: Green connection lines
- **Rivals**: Red connection lines
- **Recent Interactions**: Fading speech bubbles

## UI Layout

### Main HUD Elements

#### Top Bar
```
[🏠 Overview] [👥 Population] [🧬 Genetics] [📊 Trends] [⚙️ Settings]
```

#### Left Panel (Collapsible)
**World Overview**
- Minimap with creature dots
- Biome distribution
- Resource availability
- Weather status

**Quick Stats**
- Total population
- Birth/Death rate
- Average happiness
- Food availability

#### Right Panel (Context-Sensitive)
Changes based on selection:

**No Selection**: World stats
**Creature Selected**: 
- Name, age, generation
- Needs bars (hunger, thirst, etc.)
- Traits list
- Current action/thought
- Relationship summary

**Multiple Selected**: Comparison view

#### Bottom Panel
- Time controls (right)
- Event log (center)
- Notification area (left)
## Data Visualization Modes

### Overview Mode (Tab 1)
Real-time world statistics:
- **Population Graph**: Line chart showing population over time
- **Biome Distribution**: Pie chart of creature locations
- **Resource Map**: Heatmap overlay showing food/water
- **Activity Monitor**: What creatures are doing (eating, socializing, etc.)

### Population Mode (Tab 2)
Demographics and trends:
- **Age Distribution**: Histogram
- **Generation Tree**: Visual family trees
- **Trait Distribution**: Which traits are common/rare
- **Social Network**: Connection web visualization

### Genetics Mode (Tab 3)
Evolutionary tracking:
- **Trait Evolution**: How traits change over generations
- **Mutation History**: Timeline of significant mutations
- **Fitness Tracking**: Which traits improve survival
- **Gene Pool Diversity**: Genetic variation metrics

### Trends Mode (Tab 4)
Historical analysis:
- **Custom Graphs**: User-defined data plotting
- **Pattern Recognition**: Recurring behavioral patterns
- **Predictive Models**: Population projections
- **Event Correlation**: How events affect population

## Interaction Patterns

### Zoom-Dependent UI
UI elements adapt to zoom level:

**Macro View**:
- Hide individual creature details
- Show population density clouds
- Display migration arrows
- Emphasize biome boundaries

**Close View**:
- Show creature names and health bars
- Display emotion particles
- Reveal conversation bubbles
- Hide population-level visualizations

### Smart Information Display
- **Proximity-based Details**: More info as you zoom in
- **Importance Filtering**: Critical events always visible
- **Adaptive Legends**: Change based on current view
- **Progressive Disclosure**: Advanced stats in expandable sections

## Keyboard Shortcuts

### Camera
- **WASD/Arrows**: Pan camera
- **Q/E**: Rotate camera (if implemented)
- **Z/X**: Zoom in/out
- **F**: Follow selected creature
- **C**: Cinematic mode
- **Home**: Center on world

### Time
- **Space**: Pause/Resume
- **1-5**: Set speed preset
- **,/.**: Decrease/Increase speed
- **T**: Toggle time display format

### Selection
- **Tab**: Cycle through nearby creatures
- **Shift+Tab**: Reverse cycle
- **Ctrl+A**: Select all visible
- **Esc**: Clear selection
- **P**: Pin selected to tracking

### UI
- **F1-F4**: Switch between data tabs
- **H**: Toggle HUD
- **L**: Toggle creature labels
- **M**: Toggle minimap
- **N**: Toggle notifications
- **V**: Cycle visualization overlays

## Mouse Gestures

### Advanced Controls
- **Right-click Drag**: Measure distance
- **Ctrl+Scroll**: Adjust time speed
- **Alt+Click**: Quick info tooltip
- **Middle-click**: Reset camera
## Drill-Down Features

### Creature Inspector
Double-clicking a creature opens detailed view:
- **Biography**: Life history with major events
- **Family Tree**: Interactive genealogy visualization
- **Relationships**: Detailed social connections
- **Conversations**: Recent communication log
- **Journey Map**: Path traveled over lifetime
- **Genetic Analysis**: Trait inheritance breakdown

### Interaction Viewer
When creatures interact:
- **Conversation Bubbles**: Show actual dialogue
- **Influence Indicators**: How conversation affected decisions
- **Relationship Changes**: +/- indicators for bond strength
- **Action Predictions**: What they might do next

### Historical Playback
- **Time Scrubber**: Replay past events
- **Bookmarks**: Save interesting moments
- **Speed Control**: Slow-motion for interactions
- **Ghost Mode**: See deceased ancestors

## Responsive Design

### Zoom Level Adaptations

**1:100 (Macro)**
- Show only colored dots for creatures
- Biome-tinted fog for atmosphere
- Population density heat clouds
- No individual UI elements

**1:50 (Regional)**  
- Creatures as simple sprites
- Group behavior indicators
- Resource node icons
- Migration path arrows

**1:20 (Local)**
- Basic creature animations
- Simple need indicators
- Group labels
- Territorial boundaries

**1:5 (Close)**
- Full creature animations
- Detailed need bars
- Name labels
- Emotion particles
- Relationship lines

**1:1 (Intimate)**
- All visual features
- Thought bubbles
- Detailed expressions
- UI panels auto-position

## Performance Optimizations

### Adaptive Quality
Based on creature count and zoom:
- **Auto-adjust**: Particle effects, animation quality
- **Culling**: Hide off-screen UI elements
- **LOD UI**: Simpler displays at distance
- **Batch Updates**: Group UI refreshes

### User Preferences
Customizable performance/quality:
- Animation quality slider
- Particle density control
- UI update frequency
- Label display distance
- Maximum visible creatures

## Accessibility Features

### Visual Accessibility
- **High Contrast Mode**: Enhanced UI visibility
- **Colorblind Modes**: Alternative color schemes
- **Text Size Options**: Scalable UI text
- **Icon Alternatives**: Text labels for all icons
- **Focus Indicators**: Clear selection highlights

### Control Accessibility
- **One-Handed Mode**: All controls on one side
- **Hold-to-Pan**: Alternative to drag
- **Customizable Keys**: Rebind any control
- **Reduced Motion**: Minimize animations
- **Sticky Keys**: Toggle modes instead of hold

---
*Last Updated: 2024-12-XX*
*See also: [UI_DESIGN.md](./UI_DESIGN.md) for visual implementation details*