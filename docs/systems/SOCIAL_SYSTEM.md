# Social System

The social system governs relationships, group dynamics, and cooperative behaviors between creatures, creating complex emergent social structures.

## Relationship System

### Relationship Types

Creatures can form various types of relationships:

- **Friendship** - Mutual positive bonds from shared experiences
- **Family** - Parent-child and sibling bonds
- **Mating bonds** - Pair bonds between reproductive partners
- **Rivalry** - Competitive relationships over resources/mates
- **Mentorship** - Teaching/learning relationships
- **Group membership** - Belonging to social groups

### Relationship Mechanics

Each relationship tracks:
- **Bond strength** (0.0 to 1.0)
- **Trust level** (affects information sharing)
- **Interaction history** (recent positive/negative events)
- **Time since last interaction**
- **Shared experiences** (survived danger together, raised young, etc.)

### Relationship Development

Relationships form and change through:

1. **Proximity** - Being near each other increases familiarity
2. **Shared activities** - Eating, playing, grooming together
3. **Communication** - Successful information exchange
4. **Mutual aid** - Helping in times of need
5. **Conflict** - Fighting reduces relationship quality

## Group Dynamics

### Group Formation

Groups form naturally when:
- Multiple creatures with high mutual relationships cluster
- Benefits of group living outweigh costs
- Shared territory or resources encourage cooperation

### Group Structure

Groups can develop:

- **Leadership** - Dominant individuals guide group decisions
- **Roles** - Scouts, defenders, caregivers emerge
- **Hierarchies** - Social ranking affects resource access
- **Coalitions** - Subgroups form within larger groups

### Group Behaviors

Coordinated group activities:

- **Collective foraging** - Sharing food location information
- **Group defense** - Mobbing predators together
- **Communal care** - Alloparenting of young
- **Migration** - Moving together seasonally
- **Cultural transmission** - Sharing learned behaviors

### Group Decision Making

Groups make collective decisions through:

1. **Leadership** - Following high-ranking individuals
2. **Consensus** - Reaching agreement through communication
3. **Quorum sensing** - Acting when enough members agree
4. **Democratic** - Each member's preference counts

## Social Learning

Social learning mechanics are handled by the Conversation System. See [Conversation System](CONVERSATION_SYSTEM.md) for details on:
- Knowledge transfer between creatures
- Information networks and gossip chains
- Cultural transmission of behaviors
- Trust-based information sharing

## Cooperation & Competition

### Cooperative Behaviors

- **Food sharing** - Especially with kin and mates
- **Grooming** - Reduces parasites, strengthens bonds
- **Sentinel behavior** - Taking turns watching for danger
- **Collaborative hunting** - Working together for large prey
- **Territory defense** - Group members defend shared space

### Competition Management

Groups manage competition through:
- **Dominance hierarchies** - Reduce constant fighting
- **Resource partitioning** - Different members specialize
- **Conflict resolution** - Reconciliation after fights
- **Punishment** - Cheaters face social consequences

## Social Influence on Decisions

Social context affects individual choices:

- **Conformity** - Following group behavior norms
- **Peer pressure** - Modifying behavior to fit in
- **Social learning** - Adopting successful strategies
- **Reputation** - Past behavior affects current treatment

## Emotional Contagion

Emotions spread through groups:
- **Fear** - Panic can spread rapidly
- **Excitement** - Play and joy are contagious
- **Aggression** - Mob mentality in conflicts
- **Calm** - Relaxed individuals soothe others

## Communication Integration

Social bonds directly integrate with the [Conversation System](CONVERSATION_SYSTEM.md):
- Trust levels affect information quality and sharing
- Relationship strength influences conversation outcomes
- Group membership enables coordinated communication
- See Conversation System for detailed communication mechanics

## Performance Considerations

- Relationship updates only when creatures interact
- Group calculations use spatial clustering
- Social influence computed at decision time
- Simplified relationships for distant creatures