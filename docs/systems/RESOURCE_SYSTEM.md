# Resource System

The resource system manages the distribution, consumption, and regeneration of all resources that creatures need to survive and thrive.

## Resource Types

### Food Resources

#### Plant-Based
- **Berries** - Seasonal, high energy, found in forests/grasslands
- **Nuts** - Autumn abundance, high protein, require cracking
- **Seeds** - Year-round, small portions, ground foraging
- **Grass** - Abundant, low nutrition, herbivore staple
- **Leaves** - Tree/shrub browsing, seasonal quality
- **Fruits** - Rare, high value, tree climbing needed
- **Roots/Tubers** - Hidden, require digging, high starch

#### Animal-Based
- **Insects** - Year-round, small prey, high protein
- **Fish** - Water-adjacent, seasonal, requires skill
- **Eggs** - Seasonal, high nutrition, nest raiding
- **Carrion** - Opportunistic, disease risk

#### Water Sources
- **Streams** - Flowing, clean, reliable
- **Ponds** - Standing, quality varies, may dry up
- **Puddles** - Temporary, after rain
- **Dew** - Morning only, small amounts
- **Snow** - Winter, requires melting (energy cost)

### Shelter Resources

- **Caves** - Permanent, limited, highly contested
- **Tree hollows** - Species-specific sizes
- **Dense vegetation** - Temporary, weather protection
- **Rock overhangs** - Partial shelter
- **Burrows** - Self-made or abandoned

### Nesting Materials

- **Twigs** - Basic structure
- **Leaves** - Soft lining
- **Moss** - Insulation
- **Feathers** - Warmth (if available)
- **Mud** - Binding agent

## Resource Distribution

### Spatial Distribution

Resources are distributed based on:

1. **Biome type** - Each biome has characteristic resources
2. **Terrain features** - Water near low points, berries at forest edges
3. **Clustering** - Resources often group naturally
4. **Competition** - High-value resources are rarer
5. **Accessibility** - Some resources require special abilities

### Temporal Availability

Resources change over time:

- **Seasonal cycles** - Spring growth, autumn abundance, winter scarcity
- **Daily cycles** - Dew in morning, some flowers open/close
- **Weather effects** - Rain increases mushrooms, drought reduces plants
- **Random events** - Masting years, fish runs

## Resource Regeneration

### Plant Regeneration

- **Continuous growth** - Grass regrows steadily if not overgrazed
- **Seasonal fruiting** - Berries/nuts on annual cycles
- **Depletion effects** - Overuse slows regeneration
- **Weather modifiers** - Rain speeds growth, drought slows

### Population Dynamics

Prey animals as resources:
- **Reproduction rates** - Based on food availability
- **Predation pressure** - Affects population balance
- **Migration** - Seasonal movement patterns
- **Carrying capacity** - Environment limits

## Resource Quality

Resources vary in quality affecting:

- **Nutritional value** - Energy and nutrient content
- **Spoilage** - How quickly quality degrades
- **Effort required** - Energy cost to obtain/process
- **Risk level** - Danger in obtaining resource

Quality factors:
- **Ripeness** - For fruits/berries
- **Freshness** - For carrion/caught prey
- **Size** - Larger items provide more nutrition
- **Competition** - Contested resources may be partially depleted

## Resource Detection

Creatures find resources through:

1. **Visual scanning** - Seeing resources directly
2. **Scent tracking** - Following smell gradients
3. **Memory** - Returning to known locations
4. **Social information** - Learning from others
5. **Exploration** - Random searching

Detection affected by:
- **Sensory abilities** - Species-specific ranges
- **Environmental conditions** - Weather, time of day
- **Resource characteristics** - Size, smell, visibility

## Resource Competition

### Competition Types

- **Exploitation** - Using resource before others
- **Interference** - Preventing others' access
- **Territorial exclusion** - Claiming resource areas
- **Temporal partitioning** - Using resources at different times

### Competition Resolution

- **Dominance** - Stronger individuals get priority
- **Scramble** - First come, first served
- **Group sharing** - Within social groups
- **Avoidance** - Using different resources

## Resource Management Strategies

Creatures can:

- **Cache/hoard** - Store food for later
- **Defend** - Protect resource patches
- **Share** - Within family/social groups
- **Trade** - Exchange different resources (rare)
- **Cultivate** - Protect resource sources (very rare)

## Economic Balance

The resource system maintains ecological balance:

- **Energy pyramid** - More plant resources than prey
- **Nutrient cycling** - Death returns nutrients
- **Patch dynamics** - Resources shift locations
- **Diversity promotion** - Different species use different resources

## Performance Optimization

- Resources grouped in spatial chunks
- Only simulate nearby resource regeneration
- Batch resource detection queries
- Cache resource locations in creature memory
- Statistical modeling for distant areas