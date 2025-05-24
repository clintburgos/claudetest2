# Genetics System Design

## Overview
The genetics system provides inheritable traits through a gene-based model supporting sexual reproduction, mutations, and gene expression.

## DNA Structure

### Gene Representation
```rust
struct Gene {
    id: GeneId,
    value: f32,        // 0.0 to 1.0
    dominance: f32,    // 0.0 to 1.0
    mutation_rate: f32 // 0.0001 to 0.01
}

struct Chromosome {
    genes: Vec<Gene>,
    crossover_points: Vec<usize>
}

struct DNA {
    maternal: Vec<Chromosome>,
    paternal: Vec<Chromosome>
}
```

### Gene Categories

#### Physical Genes
- SIZE: Body size multiplier
- SPEED: Movement speed multiplier  
- STRENGTH: Physical power
- ENDURANCE: Stamina and health
- FERTILITY: Reproduction rate
- LONGEVITY: Lifespan modifier
- METABOLISM: Energy efficiency

#### Mental Genes
- INTELLIGENCE: Learning and decision-making
- MEMORY: Information retention
- PERCEPTION: Environmental awareness
- SOCIABILITY: Group behavior tendency
- AGGRESSION: Competitive behavior
- CURIOSITY: Exploration drive
- CAUTION: Risk assessment

#### Appearance Genes
- COLOR_R, COLOR_G, COLOR_B: Visual appearance
- PATTERN: Visual pattern type
- SHAPE: Body shape modifier

## Gene Expression

### Expression Rules
1. For each gene pair (maternal/paternal):
   - If dominance difference > 0.3: dominant gene expressed
   - Otherwise: blend based on dominance weights
   
2. Expression formula:
   ```
   expressed_value = (maternal_value * maternal_dominance + 
                     paternal_value * paternal_dominance) /
                     (maternal_dominance + paternal_dominance)
   ```

3. Some genes have threshold effects:
   - If expressed_value > threshold: trait activated
   - Otherwise: trait dormant

## Reproduction Mechanics

### Mating Requirements
- Both creatures adult stage
- Reproduction need > 70
- Genetic compatibility check
- Energy > 50%

### Inheritance Process
1. **Crossover**: Random chromosome segments from each parent
2. **Recombination**: Mix genes at crossover points  
3. **Mutation**: Apply random changes
4. **Expression**: Calculate expressed traits

### Mutation System
- Point mutations: Single gene value changes
- Duplication: Gene copied within chromosome
- Deletion: Gene removed (rare)
- Inversion: Gene sequence reversed

Mutation probability factors:
- Base rate: 0.001 per gene
- Environmental stress: up to 5x
- Parent age: up to 2x
- Radiation sources: up to 10x

---
*Last Updated: 2024-12-XX*
