# Observer Tools and Data Visualization

## Overview

The observer tools provide comprehensive data visualization and analysis capabilities for monitoring the simulation. These tools help users understand emergent behaviors, track population dynamics, and observe evolutionary trends.

## Core Visualization Views

### 1. Overview Mode

Real-time simulation statistics:

```rust
pub struct OverviewPanel {
    // Population metrics
    total_creatures: u32,
    births_per_minute: f32,
    deaths_per_minute: f32,
    population_trend: TrendDirection,
    
    // Resource metrics
    total_food: u32,
    total_water: u32,
    resource_consumption_rate: f32,
    resource_regeneration_rate: f32,
    
    // Social metrics
    active_groups: u32,
    active_conversations: u32,
    average_group_size: f32,
    relationship_count: u32,
    
    // Performance metrics
    fps: f32,
    creature_update_time: Duration,
    render_time: Duration,
    memory_usage: MemoryStats,
}

pub struct OverviewGraph {
    pub graph_type: GraphType,
    pub time_window: TimeWindow,
    pub data_series: Vec<DataSeries>,
    pub update_frequency: f32,
}

pub enum GraphType {
    PopulationOverTime,
    ResourceAvailability,
    BirthDeathRates,
    SpeciesDistribution,
    PerformanceMetrics,
}

pub enum TimeWindow {
    Last5Minutes,
    LastHour,
    LastDay,
    LastWeek,
    AllTime,
}
```

### 2. Population Dynamics View

Detailed population analysis:

```rust
pub struct PopulationView {
    // Age distribution
    age_histogram: Histogram<f32>,
    age_pyramid: AgePyramid,
    
    // Genetics distribution
    trait_distributions: HashMap<TraitType, Distribution>,
    genetic_diversity_index: f32,
    mutation_rate: f32,
    
    // Species tracking
    species_populations: HashMap<SpeciesId, SpeciesData>,
    species_tree: PhylogeneticTree,
    extinction_events: Vec<ExtinctionEvent>,
    
    // Population predictions
    growth_projection: PopulationProjection,
    carrying_capacity_estimate: u32,
    resource_pressure_index: f32,
}

pub struct SpeciesData {
    pub id: SpeciesId,
    pub population: u32,
    pub average_traits: HashMap<TraitType, f32>,
    pub habitat_preference: BiomeType,
    pub diet_type: DietType,
    pub social_structure: SocialStructure,
    pub emergence_time: f32,
}

pub struct AgePyramid {
    pub age_buckets: Vec<AgeBucket>,
    pub male_counts: Vec<u32>,
    pub female_counts: Vec<u32>,
}

pub struct PopulationProjection {
    pub time_points: Vec<f32>,
    pub projected_populations: Vec<u32>,
    pub confidence_intervals: Vec<(u32, u32)>,
    pub assumptions: ProjectionAssumptions,
}
```

### 3. Genetics Visualization

Evolution tracking and analysis:

```rust
pub struct GeneticsView {
    // Trait evolution
    trait_timeline: TraitTimeline,
    dominant_traits: Vec<(TraitType, f32)>,
    recessive_traits: Vec<(TraitType, f32)>,
    
    // Mutation tracking
    recent_mutations: RingBuffer<MutationEvent>,
    beneficial_mutations: Vec<MutationRecord>,
    mutation_fixation_rate: f32,
    
    // Lineage tracking
    family_trees: HashMap<Entity, FamilyTree>,
    notable_lineages: Vec<LineageData>,
    common_ancestors: HashMap<(Entity, Entity), Entity>,
    
    // Gene flow
    gene_flow_map: GeneFlowMap,
    hybridization_events: Vec<HybridizationEvent>,
    genetic_bottlenecks: Vec<BottleneckEvent>,
}

pub struct TraitTimeline {
    pub traits: HashMap<TraitType, TimeSeries<f32>>,
    pub time_resolution: f32,
    pub max_history: Duration,
}

pub struct MutationEvent {
    pub creature: Entity,
    pub trait: TraitType,
    pub old_value: f32,
    pub new_value: f32,
    pub timestamp: f32,
    pub impact: MutationImpact,
}

pub enum MutationImpact {
    Beneficial,
    Neutral,
    Harmful,
    Unknown,
}

pub struct LineageData {
    pub founder: Entity,
    pub living_descendants: u32,
    pub total_descendants: u32,
    pub average_fitness: f32,
    pub unique_traits: Vec<TraitType>,
}
```

### 4. Social Network View

Relationship and group dynamics:

```rust
pub struct SocialNetworkView {
    // Network visualization
    network_graph: ForceDirectedGraph,
    community_detection: CommunityStructure,
    centrality_measures: HashMap<Entity, CentralityScores>,
    
    // Relationship analysis
    relationship_matrix: SparseMatrix<f32>,
    relationship_types: HashMap<(Entity, Entity), RelationshipType>,
    relationship_strength_distribution: Distribution,
    
    // Group dynamics
    group_hierarchy: GroupHierarchy,
    leadership_transitions: Vec<LeadershipEvent>,
    group_stability_scores: HashMap<GroupId, f32>,
    
    // Communication patterns
    conversation_frequency_map: HeatMap,
    knowledge_diffusion_paths: Vec<DiffusionPath>,
    cultural_clusters: Vec<CulturalCluster>,
}

pub struct ForceDirectedGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub layout_algorithm: LayoutAlgorithm,
    pub interaction_strength: f32,
}

pub struct GraphNode {
    pub entity: Entity,
    pub position: Vec2,
    pub size: f32, // Based on importance
    pub color: Color, // Based on attributes
    pub label: Option<String>,
}

pub struct GraphEdge {
    pub from: Entity,
    pub to: Entity,
    pub weight: f32,
    pub edge_type: EdgeType,
    pub color: Color,
}

pub struct CommunityStructure {
    pub communities: Vec<Community>,
    pub modularity_score: f32,
    pub hierarchy_levels: Vec<HierarchyLevel>,
}
```

## Data Overlays

### Heat Maps

Spatial data visualization:

```rust
pub struct HeatMapOverlay {
    pub data_type: HeatMapType,
    pub resolution: u32,
    pub color_gradient: ColorGradient,
    pub opacity: f32,
    pub update_frequency: f32,
}

pub enum HeatMapType {
    PopulationDensity,
    ResourceAvailability,
    CreatureActivity,
    DeathLocations,
    BirthLocations,
    ConflictZones,
    MigrationPaths,
    DiseaseSpread,
}

impl HeatMapOverlay {
    pub fn generate(&self, world: &World) -> HeatMapData {
        match self.data_type {
            HeatMapType::PopulationDensity => {
                self.calculate_population_density(world)
            }
            HeatMapType::ResourceAvailability => {
                self.calculate_resource_density(world)
            }
            // ... other types
        }
    }
    
    fn calculate_population_density(&self, world: &World) -> HeatMapData {
        let mut grid = Grid2D::new(self.resolution, self.resolution);
        
        for (entity, position) in world.query::<&Position>() {
            let grid_pos = world_to_grid(position.0, self.resolution);
            grid.increment(grid_pos.x, grid_pos.y);
        }
        
        // Smooth with gaussian blur
        grid.gaussian_blur(1.5);
        
        HeatMapData {
            grid,
            min_value: 0.0,
            max_value: grid.max_value(),
            color_gradient: self.color_gradient.clone(),
        }
    }
}
```

### Flow Visualization

Movement and migration patterns:

```rust
pub struct FlowVisualization {
    pub flow_type: FlowType,
    pub particle_system: FlowParticleSystem,
    pub flow_lines: Vec<FlowLine>,
    pub aggregation_level: AggregationLevel,
}

pub enum FlowType {
    CreatureMovement,
    ResourceTransport,
    GeneFlow,
    KnowledgeSpread,
    DiseaseTransmission,
}

pub struct FlowLine {
    pub points: Vec<Vec3>,
    pub flow_rate: f32,
    pub direction: FlowDirection,
    pub color: Color,
    pub width: f32,
}

pub struct FlowParticleSystem {
    pub particles: Vec<FlowParticle>,
    pub spawn_rate: f32,
    pub lifetime: f32,
    pub speed_multiplier: f32,
}

pub struct FlowParticle {
    pub position: Vec3,
    pub velocity: Vec3,
    pub age: f32,
    pub color: Color,
    pub size: f32,
}
```

## Time-Based Analysis

### Timeline View

Historical data browser:

```rust
pub struct TimelineView {
    pub current_time: f32,
    pub playback_speed: f32,
    pub events: TimelineEventList,
    pub bookmarks: Vec<TimelineBookmark>,
    pub filters: TimelineFilters,
}

pub struct TimelineEventList {
    pub events: BTreeMap<OrderedFloat<f32>, Vec<TimelineEvent>>,
    pub event_types: HashSet<EventType>,
    pub importance_threshold: f32,
}

pub enum TimelineEvent {
    PopulationMilestone {
        time: f32,
        population: u32,
        milestone_type: MilestoneType,
    },
    MassExtinction {
        time: f32,
        casualties: u32,
        cause: ExtinctionCause,
    },
    EvolutionaryLeap {
        time: f32,
        trait: TraitType,
        change_magnitude: f32,
        affected_population: u32,
    },
    CulturalDevelopment {
        time: f32,
        development_type: CulturalDevelopment,
        originating_group: GroupId,
    },
    EnvironmentalChange {
        time: f32,
        change_type: EnvironmentalChange,
        affected_area: AABB,
    },
}

pub struct TimelineBookmark {
    pub name: String,
    pub time: f32,
    pub description: String,
    pub camera_state: CameraState,
    pub active_overlays: Vec<OverlayType>,
}
```

### Trend Analysis

Statistical trend detection:

```rust
pub struct TrendAnalyzer {
    pub metrics: HashMap<MetricType, TimeSeriesData>,
    pub trend_detectors: Vec<Box<dyn TrendDetector>>,
    pub predictions: HashMap<MetricType, Prediction>,
}

pub trait TrendDetector {
    fn detect(&self, data: &TimeSeriesData) -> Option<Trend>;
    fn confidence(&self) -> f32;
}

pub struct Trend {
    pub trend_type: TrendType,
    pub start_time: f32,
    pub strength: f32,
    pub projected_duration: Option<f32>,
}

pub enum TrendType {
    LinearGrowth { slope: f32 },
    ExponentialGrowth { rate: f32 },
    Decline { rate: f32 },
    Cyclic { period: f32, amplitude: f32 },
    Chaotic,
}

pub struct Prediction {
    pub metric_type: MetricType,
    pub future_values: Vec<(f32, f32)>, // (time, value)
    pub confidence_interval: (f32, f32),
    pub method: PredictionMethod,
}
```

## Interactive Tools

### Inspection Tool

Detailed entity examination:

```rust
pub struct InspectionTool {
    pub selected_entity: Option<Entity>,
    pub inspection_mode: InspectionMode,
    pub comparison_entities: Vec<Entity>,
}

pub enum InspectionMode {
    Detailed,
    Comparative,
    Historical,
    Genealogical,
}

pub struct CreatureInspector {
    // Basic info
    pub name: String,
    pub age: f32,
    pub generation: u32,
    
    // Vital stats
    pub health: HealthStats,
    pub needs: NeedsStats,
    pub traits: TraitList,
    
    // Relationships
    pub family: FamilyInfo,
    pub social_connections: Vec<SocialConnection>,
    pub group_membership: Option<GroupInfo>,
    
    // History
    pub life_events: Vec<LifeEvent>,
    pub movement_history: MovementTrail,
    pub conversation_log: Vec<ConversationSummary>,
    
    // Predictions
    pub survival_probability: f32,
    pub reproduction_likelihood: f32,
    pub expected_lifespan: f32,
}
```

### Query Builder

Advanced data queries:

```rust
pub struct QueryBuilder {
    pub query_type: QueryType,
    pub filters: Vec<QueryFilter>,
    pub aggregations: Vec<Aggregation>,
    pub output_format: OutputFormat,
}

pub enum QueryType {
    Creatures,
    Resources,
    Events,
    Relationships,
    Statistics,
}

pub enum QueryFilter {
    Age { min: Option<f32>, max: Option<f32> },
    Trait { trait_type: TraitType, operator: Operator, value: f32 },
    Location { bounds: AABB },
    Group { group_id: Option<GroupId> },
    Relationship { with_entity: Entity, relationship_type: RelationshipType },
    TimeRange { start: f32, end: f32 },
}

pub enum Aggregation {
    Count,
    Average(String),
    Sum(String),
    StandardDeviation(String),
    Percentile(String, f32),
    GroupBy(String),
}

pub enum OutputFormat {
    Table,
    Graph,
    HeatMap,
    Export(ExportFormat),
}
```

### Experiment Tools

Simulation manipulation:

```rust
pub struct ExperimentTools {
    pub scenarios: Vec<Scenario>,
    pub active_experiments: Vec<Experiment>,
    pub control_groups: HashMap<ExperimentId, ControlGroup>,
}

pub struct Scenario {
    pub name: String,
    pub description: String,
    pub modifications: Vec<WorldModification>,
    pub duration: Option<f32>,
    pub success_criteria: Vec<Criterion>,
}

pub enum WorldModification {
    SetResource { 
        resource_type: ResourceType, 
        availability: f32 
    },
    ModifyTrait { 
        trait_type: TraitType, 
        modifier: f32, 
        affected_creatures: CreatureFilter 
    },
    EnvironmentalChange { 
        change_type: EnvironmentalChange, 
        intensity: f32 
    },
    IntroduceDisease { 
        disease: DiseaseType, 
        patient_zero: Option<Entity> 
    },
}

pub struct Experiment {
    pub id: ExperimentId,
    pub hypothesis: String,
    pub variables: HashMap<String, Variable>,
    pub measurements: Vec<Measurement>,
    pub results: ExperimentResults,
}
```

## Performance Profiling

### System Performance View

```rust
pub struct PerformanceProfiler {
    pub frame_analyzer: FrameAnalyzer,
    pub system_timings: HashMap<SystemId, SystemProfile>,
    pub bottleneck_detector: BottleneckDetector,
    pub optimization_suggestions: Vec<OptimizationHint>,
}

pub struct FrameAnalyzer {
    pub frame_times: RingBuffer<FrameTime>,
    pub frame_breakdown: FrameBreakdown,
    pub spike_detector: SpikeDetector,
}

pub struct FrameBreakdown {
    pub simulation_time: Duration,
    pub render_time: Duration,
    pub ui_time: Duration,
    pub idle_time: Duration,
}

pub struct SystemProfile {
    pub average_time: Duration,
    pub max_time: Duration,
    pub call_count: u32,
    pub cache_hit_rate: f32,
    pub memory_allocated: usize,
}
```

## Data Export

```rust
pub struct DataExporter {
    pub export_formats: Vec<ExportFormat>,
    pub scheduled_exports: Vec<ScheduledExport>,
}

pub enum ExportFormat {
    CSV {
        delimiter: char,
        headers: bool,
    },
    JSON {
        pretty_print: bool,
        include_metadata: bool,
    },
    SQLite {
        database_path: PathBuf,
        table_structure: TableStructure,
    },
    Binary {
        compression: CompressionType,
        include_index: bool,
    },
}

pub struct ScheduledExport {
    pub name: String,
    pub query: DataQuery,
    pub format: ExportFormat,
    pub schedule: ExportSchedule,
    pub destination: ExportDestination,
}

pub enum ExportSchedule {
    Manual,
    Periodic(Duration),
    OnEvent(EventType),
    AtSimulationTime(f32),
}
```

This comprehensive set of observer tools provides deep insights into the simulation while maintaining performance through efficient data structures and smart update strategies.