# Territory Marking System Design

## Overview

A comprehensive territory system that models how creatures claim, mark, defend, and negotiate territorial boundaries. The system includes scent marking, visual displays, patrol behaviors, and both individual and group territories with realistic overlap and conflict resolution.

## Core Territory Components

```rust
pub struct TerritorySystem {
    pub territories: HashMap<TerritoryId, Territory>,
    pub marking_registry: MarkingRegistry,
    pub boundary_manager: BoundaryManager,
    pub overlap_resolver: OverlapResolver,
    pub patrol_scheduler: PatrolScheduler,
}

pub struct Territory {
    pub id: TerritoryId,
    pub owner: TerritoryOwner,
    pub bounds: TerritoryBounds,
    pub markers: Vec<TerritoryMarker>,
    pub quality: TerritoryQuality,
    pub establishment_date: f64,
    pub defense_priority: f32,
    pub tolerance_zones: Vec<ToleranceZone>,
}

pub enum TerritoryOwner {
    Individual(EntityId),
    Pair(EntityId, EntityId),
    Group(GroupId),
    Contested(Vec<EntityId>),
}

pub struct TerritoryBounds {
    pub core_area: ConvexHull,
    pub buffer_zone: ConvexHull,
    pub patrol_routes: Vec<PatrolRoute>,
    pub key_resources: Vec<ResourceLocation>,
    pub area: f32,
}

pub struct TerritoryQuality {
    pub food_abundance: f32,
    pub water_access: f32,
    pub shelter_quality: f32,
    pub safety_rating: f32,
    pub overall_score: f32,
}
```

### Territory Marking

```rust
pub struct TerritoryMarker {
    pub marker_type: MarkerType,
    pub location: Vec3,
    pub strength: f32,          // 0.0 = faded, 1.0 = fresh
    pub owner_id: EntityId,
    pub timestamp: f64,
    pub message: MarkingMessage,
}

pub enum MarkerType {
    Scent {
        chemical_signature: ChemicalSignature,
        substrate: SubstrateType,
        weather_resistance: f32,
    },
    Visual {
        mark_type: VisualMarkType,
        visibility_range: f32,
        permanence: f32,
    },
    Acoustic {
        call_type: TerritorialCall,
        frequency: f32,
        range: f32,
    },
    Physical {
        modification_type: PhysicalModification,
        effort_required: f32,
    },
}

pub struct ChemicalSignature {
    pub pheromones: Vec<Pheromone>,
    pub individual_markers: Vec<f32>,  // Unique chemical fingerprint
    pub age: f32,
    pub sex: Sex,
    pub health_status: f32,
    pub reproductive_state: ReproductiveState,
}

pub enum VisualMarkType {
    ScratchMarks { depth: f32, pattern: ScratchPattern },
    RubbedBark { height: f32, intensity: f32 },
    Scat { pile_size: f32, placement: PlacementStrategy },
    NestConstruction,
    DisplayStructure,
}

pub struct MarkingMessage {
    pub ownership_claim: OwnershipStrength,
    pub warnings: Vec<TerritoryWarning>,
    pub social_info: SocialInformation,
    pub resource_claims: Vec<ResourceType>,
}

impl TerritoryMarkingSystem {
    pub fn create_marking(
        &mut self,
        creature: &mut Creature,
        location: Vec3,
        marking_type: MarkingIntention,
    ) -> Result<TerritoryMarker, MarkingError> {
        // Check if creature can mark here
        if !self.can_mark_location(creature, location) {
            return Err(MarkingError::InvalidLocation);
        }
        
        // Determine marker type based on species capabilities
        let marker_type = self.select_marker_type(creature, marking_type);
        
        // Create chemical signature for scent marks
        let signature = match &marker_type {
            MarkerType::Scent { .. } => {
                Some(self.generate_chemical_signature(creature))
            }
            _ => None,
        };
        
        // Energy cost
        let energy_cost = self.calculate_marking_cost(&marker_type);
        if creature.physical_stats.energy < energy_cost {
            return Err(MarkingError::InsufficientEnergy);
        }
        creature.physical_stats.energy -= energy_cost;
        
        // Create marker
        let marker = TerritoryMarker {
            marker_type,
            location,
            strength: 1.0,
            owner_id: creature.id,
            timestamp: current_time(),
            message: self.encode_message(creature, marking_type),
        };
        
        // Register marking
        self.marking_registry.add_marker(marker.clone());
        
        Ok(marker)
    }
    
    pub fn detect_marking(
        &self,
        detector: &Creature,
        marker: &TerritoryMarker,
    ) -> Option<MarkingDetection> {
        match &marker.marker_type {
            MarkerType::Scent { chemical_signature, .. } => {
                let distance = (detector.position - marker.location).length();
                let wind_factor = self.calculate_wind_dispersal(marker.location, detector.position);
                let detection_threshold = detector.sensory_stats.smell_sensitivity;
                
                let detectability = marker.strength * wind_factor / (1.0 + distance * 0.1);
                
                if detectability > detection_threshold {
                    Some(MarkingDetection {
                        marker_id: marker.id,
                        information_quality: detectability,
                        decoded_message: self.decode_scent_message(
                            chemical_signature,
                            detector.cognitive_stats.scent_memory
                        ),
                    })
                } else {
                    None
                }
            }
            
            MarkerType::Visual { visibility_range, .. } => {
                let distance = (detector.position - marker.location).length();
                let line_of_sight = self.check_line_of_sight(detector.position, marker.location);
                
                if distance <= *visibility_range && line_of_sight {
                    Some(MarkingDetection {
                        marker_id: marker.id,
                        information_quality: 1.0 - distance / visibility_range,
                        decoded_message: self.decode_visual_message(&marker.message),
                    })
                } else {
                    None
                }
            }
            
            _ => None
        }
    }
}
```

### Territory Establishment

```rust
pub struct TerritoryEstablishment {
    pub site_evaluation: SiteEvaluator,
    pub boundary_creation: BoundaryCreator,
    pub initial_marking: InitialMarkingStrategy,
}

pub struct SiteEvaluator {
    pub resource_requirements: ResourceRequirements,
    pub safety_criteria: SafetyCriteria,
    pub expansion_potential: ExpansionAnalyzer,
}

impl TerritoryEstablishment {
    pub fn establish_territory(
        &mut self,
        creature: &mut Creature,
        center: Vec3,
        desired_size: f32,
    ) -> Result<Territory, EstablishmentError> {
        // Evaluate site quality
        let quality = self.site_evaluation.evaluate_site(
            center,
            desired_size,
            &creature.species.habitat_preferences
        );
        
        if quality.overall_score < creature.territory_standards {
            return Err(EstablishmentError::InsufficientQuality);
        }
        
        // Check for conflicts
        let existing_territories = self.find_overlapping_territories(center, desired_size);
        if !existing_territories.is_empty() && !self.can_contest_territories(&existing_territories, creature) {
            return Err(EstablishmentError::TerritoryOccupied);
        }
        
        // Create initial boundaries
        let bounds = self.boundary_creation.create_boundaries(
            center,
            desired_size,
            &quality.key_resources,
            &existing_territories
        );
        
        // Create territory
        let territory = Territory {
            id: generate_territory_id(),
            owner: TerritoryOwner::Individual(creature.id),
            bounds,
            markers: Vec::new(),
            quality,
            establishment_date: current_time(),
            defense_priority: 1.0,
            tolerance_zones: Vec::new(),
        };
        
        // Initial marking campaign
        let marking_plan = self.initial_marking.create_marking_plan(
            &territory,
            creature.marking_rate
        );
        
        creature.current_behavior = Behavior::TerritoryMarking {
            territory_id: territory.id,
            marking_plan,
            progress: 0.0,
        };
        
        Ok(territory)
    }
    
    pub fn expand_territory(
        &mut self,
        territory: &mut Territory,
        expansion_direction: Vec3,
        expansion_amount: f32,
    ) -> Result<(), ExpansionError> {
        // Check if expansion is viable
        let new_area = territory.bounds.area + expansion_amount;
        if new_area > self.get_max_territory_size(&territory.owner) {
            return Err(ExpansionError::ExceedsMaxSize);
        }
        
        // Find new boundaries
        let new_bounds = self.boundary_creation.expand_boundaries(
            &territory.bounds,
            expansion_direction,
            expansion_amount
        );
        
        // Check for new conflicts
        let new_overlaps = self.find_new_overlaps(&territory.bounds, &new_bounds);
        for overlap in new_overlaps {
            if !self.can_claim_overlap(&territory.owner, &overlap) {
                return Err(ExpansionError::ConflictWithNeighbor(overlap.owner));
            }
        }
        
        // Update territory
        territory.bounds = new_bounds;
        territory.defense_priority = self.calculate_defense_priority(territory);
        
        Ok(())
    }
}
```

### Patrol Behavior

```rust
pub struct PatrolSystem {
    pub route_optimizer: RouteOptimizer,
    pub marking_scheduler: MarkingScheduler,
    pub intrusion_detector: IntrusionDetector,
}

pub struct PatrolRoute {
    pub waypoints: Vec<PatrolWaypoint>,
    pub total_distance: f32,
    pub estimated_duration: f32,
    pub marking_locations: Vec<usize>, // Indices into waypoints
    pub priority: PatrolPriority,
}

pub struct PatrolWaypoint {
    pub location: Vec3,
    pub action: PatrolAction,
    pub dwell_time: f32,
    pub importance: f32,
}

pub enum PatrolAction {
    Mark(MarkerType),
    Investigate,
    Rest,
    Display,
    CheckResource(ResourceType),
}

impl PatrolSystem {
    pub fn generate_patrol_route(
        &self,
        territory: &Territory,
        patrol_type: PatrolType,
    ) -> PatrolRoute {
        match patrol_type {
            PatrolType::Boundary => {
                // Follow territory perimeter
                let waypoints = self.route_optimizer.create_perimeter_route(
                    &territory.bounds,
                    20.0 // waypoint spacing
                );
                
                // Add marking locations at strategic points
                let marking_indices = self.marking_scheduler.select_marking_points(
                    &waypoints,
                    territory.markers.len() as f32 / territory.bounds.area
                );
                
                PatrolRoute {
                    waypoints,
                    total_distance: self.calculate_route_distance(&waypoints),
                    estimated_duration: self.estimate_patrol_duration(&waypoints),
                    marking_locations: marking_indices,
                    priority: PatrolPriority::High,
                }
            }
            
            PatrolType::Resource => {
                // Visit key resources
                let resource_locations = territory.bounds.key_resources.clone();
                let waypoints = self.route_optimizer.create_resource_route(
                    resource_locations,
                    territory.bounds.core_area.center()
                );
                
                PatrolRoute {
                    waypoints,
                    total_distance: self.calculate_route_distance(&waypoints),
                    estimated_duration: self.estimate_patrol_duration(&waypoints),
                    marking_locations: vec![], // Mark at resources
                    priority: PatrolPriority::Medium,
                }
            }
            
            PatrolType::Random => {
                // Random walk within territory
                let waypoints = self.route_optimizer.create_random_route(
                    &territory.bounds,
                    10 // number of points
                );
                
                PatrolRoute {
                    waypoints,
                    total_distance: self.calculate_route_distance(&waypoints),
                    estimated_duration: self.estimate_patrol_duration(&waypoints),
                    marking_locations: vec![],
                    priority: PatrolPriority::Low,
                }
            }
        }
    }
    
    pub fn execute_patrol(
        &mut self,
        creature: &mut Creature,
        patrol: &mut PatrolProgress,
        territory: &Territory,
        delta_time: f32,
    ) -> PatrolUpdate {
        let current_waypoint = &patrol.route.waypoints[patrol.current_waypoint_index];
        
        // Move toward waypoint
        let distance = (creature.position - current_waypoint.location).length();
        if distance > 1.0 {
            creature.move_toward(current_waypoint.location, creature.movement_speed * delta_time);
            return PatrolUpdate::Moving;
        }
        
        // At waypoint, perform action
        match &current_waypoint.action {
            PatrolAction::Mark(marker_type) => {
                if let Ok(marker) = self.create_patrol_marking(creature, marker_type.clone()) {
                    return PatrolUpdate::Marked(marker);
                }
            }
            
            PatrolAction::Investigate => {
                // Check for intrusions
                let intrusions = self.intrusion_detector.scan_area(
                    current_waypoint.location,
                    10.0,
                    territory.owner
                );
                
                if !intrusions.is_empty() {
                    return PatrolUpdate::IntrusionDetected(intrusions);
                }
            }
            
            PatrolAction::Display => {
                // Territorial display
                creature.perform_display(DisplayType::Territorial);
            }
            
            _ => {}
        }
        
        // Move to next waypoint
        patrol.current_waypoint_index = (patrol.current_waypoint_index + 1) % patrol.route.waypoints.len();
        
        if patrol.current_waypoint_index == 0 {
            patrol.completed_loops += 1;
            PatrolUpdate::LoopCompleted
        } else {
            PatrolUpdate::WaypointReached
        }
    }
}
```

### Territory Defense

```rust
pub struct TerritoryDefense {
    pub threat_assessment: TerritoryThreatAssessment,
    pub response_strategies: HashMap<ThreatLevel, DefenseStrategy>,
    pub escalation_manager: EscalationManager,
}

pub struct TerritoryIntrusion {
    pub intruder: EntityId,
    pub entry_point: Vec3,
    pub current_position: Vec3,
    pub intrusion_depth: f32,
    pub duration: f32,
    pub intruder_behavior: IntruderBehavior,
}

pub enum IntruderBehavior {
    Passing,
    Foraging,
    Exploring,
    Challenging,
    Sneaking,
}

pub enum DefenseStrategy {
    Monitor,
    Warn {
        warning_type: TerritoryWarning,
        intensity: f32,
    },
    Escort {
        escort_distance: f32,
    },
    Chase {
        pursuit_distance: f32,
    },
    Attack,
    CallReinforcements,
}

impl TerritoryDefense {
    pub fn respond_to_intrusion(
        &mut self,
        defender: &mut Creature,
        intrusion: &TerritoryIntrusion,
        territory: &Territory,
    ) -> DefenseResponse {
        // Assess threat level
        let threat_level = self.threat_assessment.assess_intrusion(
            intrusion,
            defender,
            territory
        );
        
        // Get appropriate strategy
        let strategy = self.response_strategies
            .get(&threat_level)
            .cloned()
            .unwrap_or(DefenseStrategy::Monitor);
        
        match strategy {
            DefenseStrategy::Warn { warning_type, intensity } => {
                // Issue territorial warning
                let warning = self.issue_warning(defender, warning_type, intensity);
                
                DefenseResponse::Warning {
                    warning,
                    next_action: if intrusion.duration > 30.0 {
                        Some(DefenseStrategy::Chase { pursuit_distance: 50.0 })
                    } else {
                        None
                    },
                }
            }
            
            DefenseStrategy::Chase { pursuit_distance } => {
                // Initiate pursuit
                defender.current_behavior = Behavior::ChaseIntruder {
                    target: intrusion.intruder,
                    max_distance: pursuit_distance,
                    territory_bounds: territory.bounds.clone(),
                };
                
                DefenseResponse::Pursuing {
                    chase_speed: defender.movement_speed * 1.5,
                    abandon_distance: pursuit_distance,
                }
            }
            
            DefenseStrategy::CallReinforcements => {
                // Alert group members
                let call = TerritorialCall::AlarmCall {
                    threat_level,
                    location: defender.position,
                };
                
                DefenseResponse::ReinforcementsRequested {
                    call,
                    expected_responders: self.count_nearby_allies(defender.position),
                }
            }
            
            _ => DefenseResponse::Monitoring
        }
    }
    
    pub fn coordinate_group_defense(
        &mut self,
        defenders: &mut [&mut Creature],
        intrusions: &[TerritoryIntrusion],
        territory: &Territory,
    ) -> GroupDefenseStrategy {
        // Assign defenders to intrusions
        let assignments = self.assign_defenders(defenders, intrusions);
        
        // Coordinate response
        let mut group_strategy = GroupDefenseStrategy::default();
        
        for (defender_indices, intrusion) in assignments {
            let sub_group: Vec<&mut Creature> = defender_indices.iter()
                .map(|&i| defenders[i])
                .collect();
            
            match sub_group.len() {
                1 => {
                    // Single defender
                    let response = self.respond_to_intrusion(sub_group[0], intrusion, territory);
                    group_strategy.individual_responses.push(response);
                }
                2..=3 => {
                    // Small group - coordinated chase
                    group_strategy.coordinated_actions.push(
                        CoordinatedAction::FlankingManeuver {
                            target: intrusion.intruder,
                            flankers: sub_group.iter().map(|c| c.id).collect(),
                        }
                    );
                }
                _ => {
                    // Large group - surround
                    group_strategy.coordinated_actions.push(
                        CoordinatedAction::Encirclement {
                            target: intrusion.intruder,
                            participants: sub_group.iter().map(|c| c.id).collect(),
                        }
                    );
                }
            }
        }
        
        group_strategy
    }
}
```

### Territory Negotiation

```rust
pub struct TerritoryNegotiation {
    pub boundary_negotiator: BoundaryNegotiator,
    pub resource_sharing: ResourceSharingAgreements,
    pub tolerance_manager: ToleranceManager,
}

pub struct TerritorialAgreement {
    pub parties: Vec<EntityId>,
    pub agreement_type: AgreementType,
    pub terms: AgreementTerms,
    pub duration: f32,
    pub violations: Vec<Violation>,
}

pub enum AgreementType {
    BoundarySettlement {
        original_overlap: Area,
        new_boundary: Vec<Vec3>,
    },
    ResourceSharing {
        resource: ResourceLocation,
        access_schedule: AccessSchedule,
    },
    MutualTolerance {
        tolerance_zones: Vec<ToleranceZone>,
        conditions: Vec<ToleranceCondition>,
    },
    NonAggression {
        buffer_zone: Area,
    },
}

impl TerritoryNegotiation {
    pub fn negotiate_boundary(
        &mut self,
        party1: &mut Creature,
        party2: &mut Creature,
        disputed_area: &Area,
    ) -> Result<TerritorialAgreement, NegotiationFailure> {
        // Assess negotiation power
        let power1 = self.assess_negotiation_power(party1, disputed_area);
        let power2 = self.assess_negotiation_power(party2, disputed_area);
        
        // Determine negotiation outcome
        let outcome = if (power1 - power2).abs() < 0.2 {
            // Roughly equal - compromise
            NegotiationOutcome::Compromise {
                split_ratio: power1 / (power1 + power2),
            }
        } else if power1 > power2 {
            // Party 1 dominates
            NegotiationOutcome::Concession {
                winner: party1.id,
                concession_amount: 0.8,
            }
        } else {
            // Party 2 dominates
            NegotiationOutcome::Concession {
                winner: party2.id,
                concession_amount: 0.8,
            }
        };
        
        // Create agreement based on outcome
        match outcome {
            NegotiationOutcome::Compromise { split_ratio } => {
                let new_boundary = self.boundary_negotiator.create_compromise_boundary(
                    disputed_area,
                    split_ratio
                );
                
                Ok(TerritorialAgreement {
                    parties: vec![party1.id, party2.id],
                    agreement_type: AgreementType::BoundarySettlement {
                        original_overlap: disputed_area.clone(),
                        new_boundary,
                    },
                    terms: AgreementTerms {
                        respect_boundary: true,
                        marking_restrictions: Some(5.0), // 5m from boundary
                        violation_penalties: vec![Penalty::Escalation],
                    },
                    duration: 365.0, // One year
                    violations: Vec::new(),
                })
            }
            
            NegotiationOutcome::Concession { winner, .. } => {
                Ok(TerritorialAgreement {
                    parties: vec![party1.id, party2.id],
                    agreement_type: AgreementType::NonAggression {
                        buffer_zone: disputed_area.shrink(0.8),
                    },
                    terms: AgreementTerms {
                        respect_boundary: true,
                        marking_restrictions: None,
                        violation_penalties: vec![Penalty::ImmediateDefense],
                    },
                    duration: 180.0, // Six months
                    violations: Vec::new(),
                })
            }
            
            _ => Err(NegotiationFailure::IrreconcilableDifferences)
        }
    }
    
    fn assess_negotiation_power(
        &self,
        creature: &Creature,
        disputed_area: &Area,
    ) -> f32 {
        let mut power = 0.0;
        
        // Physical dominance
        power += creature.physical_stats.strength * 0.3;
        
        // Territory quality and size
        if let Some(territory) = self.get_creature_territory(creature.id) {
            power += territory.quality.overall_score * 0.2;
            power += (territory.bounds.area / 1000.0).min(1.0) * 0.2;
        }
        
        // Social support
        power += creature.social_state.group_size as f32 * 0.1;
        
        // Prior claim strength
        let prior_claim = self.calculate_prior_claim(creature.id, disputed_area);
        power += prior_claim * 0.2;
        
        power
    }
}
```

### Scent Decay & Maintenance

```rust
pub struct ScentDecaySystem {
    pub environmental_factors: EnvironmentalFactors,
    pub substrate_properties: HashMap<SubstrateType, SubstrateProperties>,
    pub weather_effects: WeatherEffects,
}

pub struct SubstrateProperties {
    pub retention_factor: f32,      // How well it holds scent
    pub exposure_factor: f32,       // Exposure to elements
    pub texture_bonus: f32,         // Rough surfaces hold scent better
}

impl ScentDecaySystem {
    pub fn update_scent_markers(
        &mut self,
        markers: &mut Vec<TerritoryMarker>,
        weather: &Weather,
        delta_time: f32,
    ) {
        for marker in markers {
            if let MarkerType::Scent { substrate, weather_resistance, .. } = &marker.marker_type {
                // Base decay rate
                let mut decay_rate = 0.01; // 1% per hour base decay
                
                // Substrate effects
                let substrate_props = self.substrate_properties.get(substrate)
                    .unwrap_or(&DEFAULT_SUBSTRATE);
                decay_rate /= substrate_props.retention_factor;
                
                // Weather effects
                match weather {
                    Weather::Rain { intensity } => {
                        decay_rate *= 1.0 + intensity * (1.0 - weather_resistance);
                    }
                    Weather::Wind { speed } => {
                        decay_rate *= 1.0 + (speed / 20.0) * substrate_props.exposure_factor;
                    }
                    Weather::Sun { intensity } => {
                        decay_rate *= 1.0 + intensity * 0.5;
                    }
                    _ => {}
                }
                
                // Apply decay
                marker.strength -= decay_rate * delta_time;
                
                // Minimum threshold
                if marker.strength < 0.05 {
                    marker.strength = 0.0; // Completely faded
                }
            }
        }
        
        // Remove faded markers
        markers.retain(|m| m.strength > 0.0);
    }
    
    pub fn refresh_marking(
        &mut self,
        creature: &mut Creature,
        old_marker: &TerritoryMarker,
    ) -> Result<TerritoryMarker, MarkingError> {
        // Can only refresh own markings
        if old_marker.owner_id != creature.id {
            return Err(MarkingError::NotOwner);
        }
        
        // Create fresh marking at same location
        let mut new_marker = old_marker.clone();
        new_marker.strength = 1.0;
        new_marker.timestamp = current_time();
        
        // Update chemical signature with current state
        if let MarkerType::Scent { chemical_signature, .. } = &mut new_marker.marker_type {
            *chemical_signature = self.generate_chemical_signature(creature);
        }
        
        // Reduced energy cost for refreshing
        let energy_cost = self.calculate_marking_cost(&new_marker.marker_type) * 0.5;
        creature.physical_stats.energy -= energy_cost;
        
        Ok(new_marker)
    }
}
```

### Visual Territory Display

```rust
pub struct TerritoryVisualization {
    pub boundary_renderer: BoundaryRenderer,
    pub marker_renderer: MarkerRenderer,
    pub overlap_visualizer: OverlapVisualizer,
}

pub struct TerritoryVisuals {
    pub boundary_line: LineRenderer,
    pub territory_tint: Color,
    pub marker_icons: Vec<MarkerIcon>,
    pub patrol_paths: Vec<PathRenderer>,
}

impl TerritoryVisualization {
    pub fn render_territory(
        &self,
        territory: &Territory,
        view_mode: TerritoryViewMode,
    ) -> TerritoryVisuals {
        let mut visuals = TerritoryVisuals::default();
        
        // Boundary visualization
        match view_mode {
            TerritoryViewMode::Boundaries => {
                visuals.boundary_line = self.boundary_renderer.render_boundary(
                    &territory.bounds,
                    self.get_owner_color(&territory.owner),
                    2.0 // line width
                );
            }
            
            TerritoryViewMode::Influence => {
                // Heat map of marking density
                visuals.territory_tint = self.calculate_influence_tint(territory);
            }
            
            TerritoryViewMode::Resources => {
                // Highlight key resources
                for resource in &territory.bounds.key_resources {
                    visuals.marker_icons.push(self.create_resource_icon(resource));
                }
            }
            
            TerritoryViewMode::Patrols => {
                // Show patrol routes
                for route in &territory.bounds.patrol_routes {
                    visuals.patrol_paths.push(self.render_patrol_route(route));
                }
            }
        }
        
        // Always show active markers
        for marker in &territory.markers {
            if marker.strength > 0.2 {
                visuals.marker_icons.push(self.marker_renderer.render_marker(marker));
            }
        }
        
        visuals
    }
    
    pub fn show_territory_contest(
        &self,
        contested_area: &Area,
        claimants: &[EntityId],
    ) -> ContestVisualization {
        ContestVisualization {
            contested_boundary: self.render_contested_area(contested_area),
            claimant_colors: claimants.iter()
                .map(|id| (*id, self.get_claimant_color(*id)))
                .collect(),
            conflict_particles: self.create_conflict_particles(contested_area.center()),
        }
    }
}
```

## Integration Points

### With Movement System
- Patrol route following
- Territory boundary constraints
- Intrusion detection via spatial queries

### With Social System  
- Group territory management
- Dominance affecting territory size
- Coalition territory defense

### With Scent/Sensory System
- Chemical communication
- Marking detection
- Individual recognition

### With Decision Making
- Territory quality evaluation
- Defense priority decisions
- Patrol scheduling

## Performance Considerations

- Territory bounds use convex hulls for fast point-in-polygon tests
- Scent decay is batch processed every 10 seconds
- Patrol routes are pre-calculated and cached
- Marker detection uses spatial hashing for nearby markers only
- Visual rendering uses LOD based on zoom level

## Balance Configuration

```rust
pub struct TerritoryBalance {
    // Size limits
    pub min_territory_size: f32,          // 100.0 m²
    pub max_territory_size: f32,          // 10000.0 m²
    pub size_per_creature: f32,           // 500.0 m²
    
    // Marking parameters  
    pub marking_energy_cost: f32,         // 5.0
    pub marking_interval: f32,            // 300.0 seconds
    pub scent_decay_rate: f32,           // 0.01 per hour
    pub visual_mark_duration: f32,        // 7 days
    
    // Defense parameters
    pub intrusion_tolerance: f32,         // 10.0 meters
    pub escalation_threshold: f32,        // 30.0 seconds
    pub pursuit_distance: f32,            // 100.0 meters
    
    // Negotiation factors
    pub prior_claim_weight: f32,          // 0.3
    pub strength_weight: f32,             // 0.4  
    pub group_support_weight: f32,        // 0.3
}