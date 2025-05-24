# Genetics System

Genetics form the foundation of creature diversity and evolution in the simulation. Each creature inherits traits from its parents through a realistic genetic system that models both Mendelian inheritance and continuous trait variation.

## Core Concepts

### Genetic Structure

Each creature carries a unique genome consisting of:

- **Discrete genes** - Following Mendelian inheritance (dominant/recessive)
- **Polygenic traits** - Continuous variation affected by multiple genes
- **Epigenetic factors** - Environmental influences on gene expression

### Key Genetic Traits

#### Physical Traits
- Size (polygenic)
- Speed (polygenic)
- Strength (discrete + modifiers)
- Color patterns (discrete)
- Fur/scale type (discrete)

#### Behavioral Traits
- Aggression level (polygenic)
- Curiosity (polygenic)
- Social tendency (polygenic)
- Risk tolerance (polygenic)

#### Health Traits
- Lifespan modifier
- Disease resistance
- Fertility rate
- Metabolic efficiency

## Inheritance Mechanics

### Mendelian Inheritance

Discrete traits follow classic dominant/recessive patterns:

```
Parent 1: Aa (brown fur)
Parent 2: aa (white fur)

Possible offspring:
- 50% Aa (brown fur)
- 50% aa (white fur)
```

### Polygenic Inheritance

Continuous traits are influenced by multiple genes:

```
Size = Base + (Gene1 + Gene2 + Gene3) × Modifiers
```

Children inherit a blend of parental values with small random variations.

### Mutations

- **Mutation rate**: 1% per gene per generation
- **Effect size**: Usually small (±10% for continuous traits)
- **Beneficial mutations**: Rare but possible
- **Lethal mutations**: Filtered out (non-viable offspring)

## Genetic Fitness

Fitness is calculated based on:

1. **Survival traits** - Health, disease resistance
2. **Reproductive success** - Fertility, attractiveness
3. **Environmental adaptation** - Suited to current biome
4. **Social success** - Ability to form beneficial relationships

## Evolution Mechanics

### Natural Selection

- Creatures with higher fitness are more likely to:
  - Survive to reproductive age
  - Find mates
  - Produce more offspring
  - Pass on successful traits

### Sexual Selection

- Mate choice based on:
  - Visible traits (size, colors)
  - Health indicators
  - Social status
  - Resource provision ability

### Genetic Drift

In small populations, random events can fix or eliminate traits regardless of fitness.

### Gene Flow

Migration between populations introduces new genetic variation and prevents inbreeding.

## Population Genetics

### Genetic Diversity Metrics

- **Heterozygosity** - Genetic variation within individuals
- **Allelic richness** - Number of different gene variants
- **Inbreeding coefficient** - Measure of population isolation

### Evolutionary Pressures

Different biomes create different selection pressures:

- **Desert** - Selects for water efficiency, heat tolerance
- **Tundra** - Selects for cold resistance, fat storage
- **Forest** - Selects for climbing ability, camouflage

## Visual Representation

Genetics influence creature appearance:

- **Size variations** - 0.5x to 2.0x species baseline
- **Color morphs** - Multiple variants per species
- **Body proportions** - Leg length, body shape
- **Special features** - Horns, crests, patterns

## Breeding System Integration

The genetics system deeply integrates with reproduction:

- **Mate selection** - Creatures assess genetic compatibility
- **Hybrid vigor** - Outbred offspring have fitness bonuses
- **Inbreeding depression** - Related parents produce weaker offspring
- **Assortative mating** - Tendency to choose similar mates

## Performance Considerations

- Genetic calculations are cached and only updated on:
  - Birth (full genome generation)
  - Mutation events
  - Fitness recalculation (daily)
- Simplified genetics for distant creatures
- Batch processing for population statistics