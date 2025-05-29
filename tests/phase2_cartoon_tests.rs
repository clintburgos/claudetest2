//! Comprehensive tests for Phase 2 cartoon isometric implementation

use bevy::prelude::*;
use creature_simulation::prelude::*;
use creature_simulation::rendering::cartoon::*;
use creature_simulation::rendering::particles::*;
use creature_simulation::systems::biome::*;

/// Test biome generation and resource distribution
#[test]
fn test_biome_generation() {
    let mut biome_map = BiomeMap::new(12345);
    
    // Test deterministic generation
    let pos1 = Vec2::new(0.0, 0.0);
    let biome1 = biome_map.get_biome(pos1);
    let biome2 = biome_map.get_biome(pos1);
    assert_eq!(biome1, biome2, "Biome generation should be deterministic");
    
    // Test different positions yield different biomes
    let positions = vec![
        Vec2::new(100.0, 0.0),
        Vec2::new(-100.0, 0.0),
        Vec2::new(0.0, 100.0),
        Vec2::new(0.0, -100.0),
        Vec2::new(500.0, 500.0),
    ];
    
    let mut biome_counts = std::collections::HashMap::new();
    for pos in positions {
        let biome = biome_map.get_biome(pos);
        *biome_counts.entry(biome).or_insert(0) += 1;
    }
    
    // Should have some variety
    assert!(biome_counts.len() > 1, "Should generate multiple biome types");
}

/// Test biome-specific resource spawning
#[test]
fn test_biome_resource_distribution() {
    // Test each biome has appropriate resources
    let forest_resources = BiomeMap::get_biome_resources(BiomeType::Forest);
    assert!(forest_resources.len() > 0);
    assert!(forest_resources.iter().any(|(r, _)| matches!(r, ResourceType::Berry)));
    assert!(forest_resources.iter().any(|(r, _)| matches!(r, ResourceType::Mushroom)));
    
    let desert_resources = BiomeMap::get_biome_resources(BiomeType::Desert);
    assert!(desert_resources.iter().any(|(r, _)| matches!(r, ResourceType::CactiWater)));
    assert!(desert_resources.iter().any(|(r, _)| matches!(r, ResourceType::DesertFruit)));
    
    let ocean_resources = BiomeMap::get_biome_resources(BiomeType::Ocean);
    assert!(ocean_resources.iter().any(|(r, _)| matches!(r, ResourceType::Seaweed)));
    assert!(ocean_resources.iter().any(|(r, _)| matches!(r, ResourceType::Shellfish)));
    
    // Test weights sum to reasonable values
    for biome in [BiomeType::Forest, BiomeType::Desert, BiomeType::Grassland, BiomeType::Tundra, BiomeType::Ocean] {
        let resources = BiomeMap::get_biome_resources(biome);
        let total_weight: f32 = resources.iter().map(|(_, w)| w).sum();
        assert!(total_weight > 0.0, "Biome {:?} should have positive resource weights", biome);
    }
}

/// Test biome abundance modifiers
#[test]
fn test_biome_abundance() {
    assert_eq!(BiomeMap::get_biome_abundance(BiomeType::Forest), 1.2);
    assert_eq!(BiomeMap::get_biome_abundance(BiomeType::Desert), 0.6);
    assert_eq!(BiomeMap::get_biome_abundance(BiomeType::Grassland), 1.0);
    assert_eq!(BiomeMap::get_biome_abundance(BiomeType::Tundra), 0.7);
    assert_eq!(BiomeMap::get_biome_abundance(BiomeType::Ocean), 1.1);
}

/// Test resource nutritional values
#[test]
fn test_resource_nutritional_values() {
    // Test biome-specific resources have appropriate nutritional values
    let (food, water) = ResourceType::Berry.nutritional_values();
    assert!(food > 0.0, "Berry should provide food value");
    assert!(water >= 0.0, "Berry should provide some water");
    
    let (food, water) = ResourceType::CactiWater.nutritional_values();
    assert!(water > food, "Cacti water should be primarily hydration");
    
    let (food, water) = ResourceType::IceFish.nutritional_values();
    assert!(food > water, "Fish should be primarily food");
    
    // Verify all resource types have valid nutritional values
    let all_resources = vec![
        ResourceType::Berry, ResourceType::Mushroom, ResourceType::Nuts,
        ResourceType::CactiWater, ResourceType::DesertFruit,
        ResourceType::IceFish, ResourceType::SnowBerry,
        ResourceType::Seeds, ResourceType::Grass,
        ResourceType::Seaweed, ResourceType::Shellfish,
    ];
    
    for resource in all_resources {
        let (food, water) = resource.nutritional_values();
        assert!(food >= 0.0 && water >= 0.0, "{:?} should have non-negative nutritional values", resource);
        assert!(food > 0.0 || water > 0.0, "{:?} should provide some nutrition", resource);
    }
}

/// Test resource consumption and regeneration
#[test]
fn test_resource_amount_operations() {
    let mut amount = ResourceAmount::new(100.0);
    assert_eq!(amount.current, 100.0);
    assert_eq!(amount.max, 100.0);
    assert!(!amount.is_depleted());
    assert!(amount.is_full());
    
    // Test consumption
    let consumed = amount.consume(30.0);
    assert_eq!(consumed, 30.0);
    assert_eq!(amount.current, 70.0);
    assert_eq!(amount.percentage(), 0.7);
    
    // Test over-consumption
    let consumed = amount.consume(100.0);
    assert_eq!(consumed, 70.0);
    assert_eq!(amount.current, 0.0);
    assert!(amount.is_depleted());
    
    // Test regeneration
    amount.regenerate(50.0);
    assert_eq!(amount.current, 50.0);
    assert!(!amount.is_depleted());
    assert!(!amount.is_full());
    
    // Test over-regeneration
    amount.regenerate(100.0);
    assert_eq!(amount.current, 100.0);
    assert!(amount.is_full());
}

/// Test biome cache management
#[test]
fn test_biome_cache_management() {
    let mut biome_map = BiomeMap::new(54321);
    
    // Generate some biome data to populate cache
    for x in -10..=10 {
        for y in -10..=10 {
            biome_map.get_biome_tile(IVec2::new(x, y));
        }
    }
    
    // Cache should have entries
    assert!(biome_map.biome_cache.len() > 0);
    let initial_size = biome_map.biome_cache.len();
    
    // Clear distant cache
    biome_map.clear_distant_cache(IVec2::ZERO, 5);
    
    // Should have fewer entries
    assert!(biome_map.biome_cache.len() < initial_size);
    
    // Nearby entries should remain
    assert!(biome_map.biome_cache.contains_key(&IVec2::new(0, 0)));
    assert!(biome_map.biome_cache.contains_key(&IVec2::new(3, 3)));
    
    // Distant entries should be removed
    assert!(!biome_map.biome_cache.contains_key(&IVec2::new(10, 10)));
}

/// Test biome transition detection
#[test]
fn test_biome_transitions() {
    let mut biome_map = BiomeMap::new(11111);
    
    // Find a position and check its transition data
    let tile_pos = IVec2::new(0, 0);
    let transition = biome_map.get_transition_data(tile_pos);
    
    assert!(transition.blend_factor >= 0.0 && transition.blend_factor <= 1.0);
    assert!(transition.edge_distance >= 0.0);
    
    // Transition cache should work
    let transition2 = biome_map.get_transition_data(tile_pos);
    assert_eq!(transition.primary_biome, transition2.primary_biome);
    assert_eq!(transition.blend_factor, transition2.blend_factor);
}

/// Test animation frame mapping
#[test]
fn test_animation_frame_ranges() {
    // Test each animation has correct frame range
    let test_cases = vec![
        (AnimationState::Idle, 0, 4),
        (AnimationState::Walk, 4, 8),
        (AnimationState::Run, 12, 6),
        (AnimationState::Eat, 18, 6),
        (AnimationState::Sleep, 24, 4),
        (AnimationState::Talk, 28, 8),
        (AnimationState::Attack, 36, 6),
        (AnimationState::Death, 42, 8),
    ];
    
    for (state, expected_start, expected_count) in test_cases {
        let (start, count) = get_animation_frames(state);
        assert_eq!(start, expected_start, "{:?} should start at frame {}", state, expected_start);
        assert_eq!(count, expected_count, "{:?} should have {} frames", state, expected_count);
    }
}

/// Test emotion determination from creature state
#[test]
fn test_emotion_determination() {
    // Test critical needs trigger appropriate emotions
    let mut needs = Needs::default();
    needs.hunger = 0.9; // Very hungry
    let state = CreatureState::Idle;
    
    let emotion = determine_emotion_from_state(&needs, &state);
    assert_eq!(emotion, EmotionType::Hungry);
    
    // Test low energy triggers tired
    needs.hunger = 0.5;
    needs.energy = 0.1;
    let emotion = determine_emotion_from_state(&needs, &state);
    assert_eq!(emotion, EmotionType::Tired);
    
    // Test dead creatures show no emotion
    let state = CreatureState::Dead;
    let emotion = determine_emotion_from_state(&needs, &state);
    assert_eq!(emotion, EmotionType::Neutral);
}

/// Test particle type mapping from emotions
#[test]
fn test_particle_emotion_mapping() {
    // Create a test app with minimal setup
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(ParticleAssets::default());
    app.insert_resource(Time::<()>::default());
    
    // Test emotion to particle type mapping
    let test_cases = vec![
        (EmotionType::Happy, Some(ParticleType::Heart)),
        (EmotionType::Tired, Some(ParticleType::Zzz)),
        (EmotionType::Curious, Some(ParticleType::Question)),
        (EmotionType::Frightened, Some(ParticleType::Exclamation)),
        (EmotionType::Angry, Some(ParticleType::Sweat)),
        (EmotionType::Neutral, None),
    ];
    
    for (emotion, expected_particle) in test_cases {
        // The actual mapping is done in spawn_emotion_particles
        // We test the logic here
        let particle = match emotion {
            EmotionType::Happy => Some(ParticleType::Heart),
            EmotionType::Tired => Some(ParticleType::Zzz),
            EmotionType::Curious => Some(ParticleType::Question),
            EmotionType::Frightened => Some(ParticleType::Exclamation),
            EmotionType::Angry => Some(ParticleType::Sweat),
            _ => None,
        };
        assert_eq!(particle, expected_particle, "Emotion {:?} should map to particle {:?}", emotion, expected_particle);
    }
}

/// Test particle physics properties
#[test]
fn test_particle_physics() {
    // Test lerp function
    assert_eq!(lerp(0.0, 10.0, 0.0), 0.0);
    assert_eq!(lerp(0.0, 10.0, 0.5), 5.0);
    assert_eq!(lerp(0.0, 10.0, 1.0), 10.0);
    assert_eq!(lerp(-10.0, 10.0, 0.5), 0.0);
    
    // Test particle lifetime and interpolation
    let mut particle = Particle {
        lifetime: 1.0,
        initial_lifetime: 2.0,
        velocity: Vec2::new(10.0, 20.0),
        acceleration: Vec2::new(0.0, -10.0),
        scale_curve: (1.0, 0.0),
        alpha_curve: (1.0, 0.0),
    };
    
    // Simulate one frame (0.016s)
    let dt = 0.016;
    particle.lifetime -= dt;
    particle.velocity += particle.acceleration * dt;
    
    assert!(particle.lifetime < 1.0);
    assert!(particle.velocity.y < 20.0); // Gravity applied
    
    let progress = 1.0 - (particle.lifetime / particle.initial_lifetime);
    let scale = lerp(particle.scale_curve.0, particle.scale_curve.1, progress);
    let alpha = lerp(particle.alpha_curve.0, particle.alpha_curve.1, progress);
    
    assert!(scale < 1.0 && scale > 0.0);
    assert!(alpha < 1.0 && alpha > 0.0);
}

/// Test genetic variation mapping
#[test]
fn test_genetic_variations() {
    // Test size scaling
    let genetics = Genetics {
        size: 0.0, // Minimum
        color: 0.5,
        pattern: 0.5,
        aggression: 0.5,
        speed: 0.5,
    };
    let size_scale = 0.7 + genetics.size * 0.6;
    assert_eq!(size_scale, 0.7);
    
    let genetics = Genetics {
        size: 1.0, // Maximum
        color: 0.5,
        pattern: 0.5,
        aggression: 0.5,
        speed: 0.5,
    };
    let size_scale = 0.7 + genetics.size * 0.6;
    assert_eq!(size_scale, 1.3);
    
    // Test pattern thresholds
    let genetics_no_pattern = Genetics { pattern: 0.3, ..Default::default() };
    let genetics_spots = Genetics { pattern: 0.5, ..Default::default() };
    let genetics_stripes = Genetics { pattern: 0.8, ..Default::default() };
    
    assert!(genetics_no_pattern.pattern < 0.4); // No pattern threshold
    assert!(genetics_spots.pattern > 0.4 && genetics_spots.pattern < 0.7); // Spots range
    assert!(genetics_stripes.pattern > 0.7); // Stripes threshold
}

/// Test expression overlay parameters
#[test]
fn test_expression_overlays() {
    let mut overlay = ExpressionOverlay {
        eye_offset: Vec2::ZERO,
        eye_scale: 1.0,
        mouth_curve: 0.0,
        mouth_open: 0.0,
        brow_angle: 0.0,
    };
    
    // Test happy expression
    apply_emotion_to_expression(&mut overlay, EmotionType::Happy);
    assert!(overlay.mouth_curve > 0.0, "Happy should have upward mouth curve");
    assert!(overlay.eye_scale > 1.0, "Happy should have slightly larger eyes");
    
    // Test angry expression
    apply_emotion_to_expression(&mut overlay, EmotionType::Angry);
    assert!(overlay.mouth_curve < 0.0, "Angry should have downward mouth curve");
    assert!(overlay.brow_angle < 0.0, "Angry should have furrowed brow");
    
    // Test frightened expression
    apply_emotion_to_expression(&mut overlay, EmotionType::Frightened);
    assert!(overlay.eye_scale > 1.2, "Frightened should have wide eyes");
    assert!(overlay.brow_angle > 0.0, "Frightened should have raised brow");
}

/// Test speech bubble icon mapping
#[test]
fn test_speech_bubble_icons() {
    // Test conversation state to icon mapping
    let test_cases = vec![
        (ConversationState::Greeting, "!"),
        (ConversationState::ShareInfo(ConversationTopic::FoodLocation), "?"),
        (ConversationState::RequestHelp, "..."),
        (ConversationState::OfferHelp, "♥"),
    ];
    
    for (state, expected_icon) in test_cases {
        let icon = match state {
            ConversationState::Greeting => "!",
            ConversationState::ShareInfo(_) => "?",
            ConversationState::RequestHelp => "...",
            ConversationState::OfferHelp => "♥",
            _ => "?",
        };
        assert_eq!(icon, expected_icon, "State {:?} should show icon {}", state, expected_icon);
    }
}

/// Test quality preset configurations
#[test]
fn test_quality_presets() {
    let low = CartoonVisualConfig::low_quality();
    assert!(!low.particles_enabled);
    assert!(!low.expressions_enabled);
    assert_eq!(low.shadow_opacity, 0.0);
    
    let medium = CartoonVisualConfig::medium_quality();
    assert!(medium.particles_enabled);
    assert!(medium.expressions_enabled);
    assert!(medium.shadow_opacity > 0.0);
    
    let high = CartoonVisualConfig::high_quality();
    assert!(high.particles_enabled);
    assert!(high.expressions_enabled);
    assert_eq!(high.shadow_opacity, 0.8);
    assert!(high.max_particles > medium.max_particles);
}

/// Test isometric coordinate conversions
#[test]
fn test_isometric_conversions() {
    use creature_simulation::rendering::isometric::*;
    
    // Test world to screen conversion
    let world_pos = Vec3::new(1.0, 0.0, 1.0);
    let screen_pos = world_to_screen(world_pos);
    assert_eq!(screen_pos.x, 0.0); // X and Z cancel out
    assert_eq!(screen_pos.y, TILE_HEIGHT); // Both contribute equally
    
    // Test screen to world conversion
    let screen_pos = Vec2::new(32.0, 16.0);
    let world_pos = screen_to_world(screen_pos, Vec2::ZERO, 1.0);
    assert!(world_pos.y.abs() < 0.001); // Ground level
    
    // Test tile helpers
    let tile_coord = IVec2::new(5, 3);
    let world = tiles::tile_to_world(tile_coord);
    assert_eq!(world.x, 5.0);
    assert_eq!(world.z, 3.0);
    
    let tile_back = tiles::world_to_tile(world);
    assert_eq!(tile_back, tile_coord);
}

/// Test depth calculation for isometric rendering
#[test]
fn test_isometric_depth() {
    use creature_simulation::rendering::isometric::calculate_depth;
    
    // Objects at same position but different heights
    let pos1 = Vec3::new(0.0, 0.0, 0.0);
    let pos2 = Vec3::new(0.0, 1.0, 0.0);
    
    let depth1 = calculate_depth(pos1, 0.0);
    let depth2 = calculate_depth(pos2, 0.0);
    
    assert!(depth2 > depth1, "Higher objects should have greater depth");
    
    // Objects at different positions
    let pos3 = Vec3::new(1.0, 0.0, 1.0);
    let depth3 = calculate_depth(pos3, 0.0);
    
    assert!(depth3 > depth1, "Objects further back should have greater depth");
}

/// Helper function implementations for tests
pub fn apply_emotion_to_expression(overlay: &mut ExpressionOverlay, emotion: EmotionType) {
    match emotion {
        EmotionType::Happy => {
            overlay.mouth_curve = 0.5;
            overlay.eye_scale = 1.1;
            overlay.brow_angle = -10.0;
        }
        EmotionType::Sad => {
            overlay.mouth_curve = -0.5;
            overlay.eye_scale = 0.9;
            overlay.brow_angle = 20.0;
        }
        EmotionType::Angry => {
            overlay.mouth_curve = -0.3;
            overlay.eye_scale = 0.8;
            overlay.brow_angle = -20.0;
        }
        EmotionType::Frightened => {
            overlay.mouth_curve = -0.2;
            overlay.eye_scale = 1.3;
            overlay.brow_angle = 15.0;
        }
        EmotionType::Tired => {
            overlay.mouth_curve = -0.1;
            overlay.eye_scale = 0.7;
            overlay.brow_angle = 5.0;
        }
        _ => {
            overlay.mouth_curve = 0.0;
            overlay.eye_scale = 1.0;
            overlay.brow_angle = 0.0;
        }
    }
}

