# Cartoon Graphics Implementation Design

## Overview
This document details the implementation design for integrating cartoon-style isometric graphics into the existing creature simulation, following established design patterns and architecture decisions.

## Architecture Overview

### 1. Rendering Pipeline Enhancement

```rust
// src/plugins/rendering.rs modifications
pub struct CartoonRenderingPlugin;

impl Plugin for CartoonRenderingPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .insert_resource(IsometricSettings::default())
            .insert_resource(SpriteAtlasManager::new())
            .insert_resource(ParticleSystemManager::new())
            
            // Systems - ordered by render stage
            .add_systems(Update, (
                update_sprite_animations,
                update_expression_overlays,
                update_particle_effects,
                update_terrain_chunks,
            ).chain())
            
            .add_systems(PostUpdate, (
                sort_isometric_depth,
                cull_offscreen_entities,
                batch_sprite_draws,
            ).chain());
    }
}
```

### 2. Component Architecture

```rust
// src/components/visuals.rs

#[derive(Component)]
pub struct CartoonSprite {
    pub atlas_handle: Handle<TextureAtlas>,
    pub animation_state: AnimationState,
    pub current_frame: usize,
    pub frame_timer: Timer,
    pub base_scale: Vec2,
    pub genetic_modifiers: GeneticVisuals,
}

#[derive(Component)]
pub struct GeneticVisuals {
    pub size_multiplier: f32,      // 0.7 - 1.3 based on genetics
    pub hue_shift: f32,            // Color variation
    pub pattern_type: PatternType,  // Spots, stripes, solid
    pub feature_sizes: FeatureSizes, // Ears, tail, etc.
}

#[derive(Component)]
pub struct ExpressionOverlay {
    pub emotion: EmotionType,
    pub intensity: f32,
    pub eye_state: EyeState,
    pub mouth_state: MouthState,
    pub blend_timer: Timer,
}

#[derive(Component)]
pub struct IsometricTransform {
    pub grid_position: IVec3,      // Logical position
    pub world_position: Vec3,      // Rendered position
    pub depth_layer: f32,          // For sorting
    pub elevation: f32,            // Height above ground
}
```

### 3. Asset Management System

```rust
// src/systems/asset_loader.rs

pub struct CartoonAssetLoader {
    texture_atlases: HashMap<String, Handle<TextureAtlas>>,
    loading_queue: PriorityQueue<AssetRequest>,
    cache: AssetCache,
}

impl CartoonAssetLoader {
    pub fn load_creature_assets(&mut self, species: Species) -> AssetLoadResult {
        let atlas_path = format!("sprites/creatures/{}/atlas.png", species);
        let config_path = format!("sprites/creatures/{}/config.json", species);
        
        // Priority loading based on LOD
        self.loading_queue.push(AssetRequest {
            path: atlas_path,
            priority: AssetPriority::Critical,
            lod_level: 0,
        });
        
        // Return handle for immediate use with placeholder
        AssetLoadResult::Loading(self.get_placeholder_handle())
    }
}
```

### 4. Biome Rendering System

```rust
// src/systems/biome_renderer.rs

pub struct BiomeRenderer {
    chunk_size: u32,
    loaded_chunks: HashMap<ChunkCoord, TerrainChunk>,
    tile_sets: HashMap<BiomeType, TileSet>,
    transition_blender: TransitionBlender,
}

#[derive(Component)]
pub struct TerrainChunk {
    pub tiles: Vec<TileInstance>,
    pub decorations: Vec<DecorationInstance>,
    pub mesh: Handle<Mesh>,
    pub is_dirty: bool,
}

impl BiomeRenderer {
    pub fn generate_chunk(&mut self, coord: ChunkCoord, biome_map: &BiomeMap) -> TerrainChunk {
        let mut tiles = Vec::new();
        
        for x in 0..self.chunk_size {
            for y in 0..self.chunk_size {
                let world_pos = chunk_to_world(coord, x, y);
                let biome = biome_map.sample(world_pos);
                
                // Handle biome transitions
                let neighbors = biome_map.get_neighbors(world_pos);
                let tile = if neighbors.all_same() {
                    self.tile_sets[&biome].get_random_tile()
                } else {
                    self.transition_blender.blend_tiles(biome, neighbors)
                };
                
                tiles.push(TileInstance {
                    position: world_pos,
                    tile_id: tile,
                    variation: rand::random(),
                });
            }
        }
        
        TerrainChunk {
            tiles,
            decorations: self.generate_decorations(&tiles),
            mesh: self.build_chunk_mesh(&tiles),
            is_dirty: false,
        }
    }
}
```

### 5. Animation State Machine

```rust
// src/systems/animation_controller.rs

pub struct AnimationController;

impl AnimationController {
    pub fn update_creature_animation(
        &self,
        creature: &Creature,
        sprite: &mut CartoonSprite,
        lod: LODLevel,
    ) {
        // Determine animation based on state
        let target_animation = match (creature.state, creature.action) {
            (CreatureState::Moving, _) => {
                if creature.velocity.length() > SPRINT_THRESHOLD {
                    AnimationType::Run
                } else {
                    AnimationType::Walk
                }
            }
            (_, Some(Action::Eating)) => AnimationType::Eat,
            (_, Some(Action::Drinking)) => AnimationType::Drink,
            (CreatureState::Sleeping, _) => AnimationType::Sleep,
            (CreatureState::Conversing, _) => AnimationType::Talk,
            _ => AnimationType::Idle,
        };
        
        // Apply LOD-based frame rate
        let frame_rate = match lod {
            LODLevel::Full => 60.0,
            LODLevel::High => 30.0,
            LODLevel::Medium => 15.0,
            LODLevel::Low => 10.0,
            LODLevel::Minimal => 0.0, // Static
        };
        
        sprite.frame_timer.set_duration(Duration::from_secs_f32(1.0 / frame_rate));
        
        // Smooth animation transitions
        if sprite.animation_state.current != target_animation {
            sprite.animation_state.transition_to(target_animation, 0.2);
        }
    }
}
```

### 6. Particle Effects System

```rust
// src/systems/particle_system.rs

#[derive(Component)]
pub struct ParticleEmitter {
    pub effect_type: ParticleEffect,
    pub spawn_rate: f32,
    pub lifetime: f32,
    pub position_offset: Vec3,
    pub velocity_range: (Vec3, Vec3),
    pub texture: Handle<Image>,
}

pub enum ParticleEffect {
    Emotion { emotion: EmotionType },
    Weather { weather: WeatherType },
    Action { action: ActionType },
    Environmental { env_type: EnvironmentalType },
}

impl ParticleSystem {
    pub fn spawn_emotion_particles(
        &mut self,
        commands: &mut Commands,
        entity: Entity,
        emotion: EmotionType,
        intensity: f32,
    ) {
        let (texture, spawn_pattern) = match emotion {
            EmotionType::Love => ("particles/heart.png", SpawnPattern::Burst(3)),
            EmotionType::Happy => ("particles/sparkle.png", SpawnPattern::Continuous(0.5)),
            EmotionType::Sleeping => ("particles/zzz.png", SpawnPattern::Sequential(1.0)),
            EmotionType::Angry => ("particles/steam.png", SpawnPattern::Sides),
            _ => return,
        };
        
        commands.entity(entity).insert(ParticleEmitter {
            effect_type: ParticleEffect::Emotion { emotion },
            spawn_rate: intensity,
            lifetime: 2.0,
            position_offset: Vec3::new(0.0, 20.0, 0.0),
            velocity_range: (Vec3::new(-5.0, 10.0, -5.0), Vec3::new(5.0, 20.0, 5.0)),
            texture: self.get_texture(texture),
        });
    }
}
```

### 7. UI Integration

```rust
// src/plugins/ui_cartoon.rs

pub struct CartoonUIPlugin;

impl CartoonUIPlugin {
    fn render_creature_info(
        &self,
        ui: &mut egui::Ui,
        creature: &SelectedCreature,
        visuals: &CartoonVisuals,
    ) {
        // Speech bubble for conversations
        if let Some(conversation) = &creature.active_conversation {
            self.render_speech_bubble(ui, conversation, creature.position);
        }
        
        // Health and needs bars above creature
        let screen_pos = world_to_screen(creature.position + Vec3::Y * 50.0);
        
        egui::Area::new("creature_overhead")
            .fixed_pos(screen_pos)
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    // Health bar with cartoon styling
                    self.render_cartoon_bar(ui, "health", creature.health, Color32::GREEN);
                    
                    // Need icons
                    if creature.hunger > 0.7 {
                        ui.add(egui::Image::new("icons/hungry.png").size([16.0, 16.0]));
                    }
                    if creature.thirst > 0.7 {
                        ui.add(egui::Image::new("icons/thirsty.png").size([16.0, 16.0]));
                    }
                });
            });
    }
    
    fn render_speech_bubble(
        &self,
        ui: &mut egui::Ui,
        conversation: &Conversation,
        position: Vec3,
    ) {
        let screen_pos = world_to_screen(position + Vec3::Y * 80.0);
        
        egui::Area::new("speech_bubble")
            .fixed_pos(screen_pos)
            .show(ui.ctx(), |ui| {
                Frame::none()
                    .fill(Color32::WHITE)
                    .stroke(Stroke::new(2.0, Color32::BLACK))
                    .rounding(10.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            // Show emotion icons
                            match conversation.tone {
                                Tone::Friendly => ui.label("😊"),
                                Tone::Angry => ui.label("😠"),
                                Tone::Sad => ui.label("😢"),
                                _ => {}
                            }
                            ui.label(&conversation.current_message);
                        });
                    });
            });
    }
}
```

### 8. Performance Optimization

```rust
// src/systems/performance_manager.rs

pub struct CartoonPerformanceManager {
    sprite_batcher: SpriteBatcher,
    visibility_culler: VisibilityCuller,
    lod_calculator: LODCalculator,
    quality_settings: QualitySettings,
}

impl CartoonPerformanceManager {
    pub fn optimize_frame(
        &mut self,
        camera: &Camera,
        creatures: &Query<(&Transform, &CartoonSprite)>,
        target_fps: f32,
        current_fps: f32,
    ) {
        // Dynamic quality adjustment
        if current_fps < target_fps * 0.9 {
            self.quality_settings.decrease_quality();
        } else if current_fps > target_fps * 1.1 {
            self.quality_settings.increase_quality();
        }
        
        // Frustum culling
        let visible = self.visibility_culler.cull_entities(camera, creatures);
        
        // LOD assignment based on distance
        for (entity, transform) in visible {
            let distance = camera.position.distance(transform.translation);
            let lod = self.lod_calculator.calculate_lod(distance, self.quality_settings);
            
            // Apply LOD-specific optimizations
            match lod {
                LODLevel::Minimal => {
                    // Disable animations, particles
                    commands.entity(entity).remove::<ParticleEmitter>();
                }
                LODLevel::Low => {
                    // Reduce animation framerate
                    sprite.frame_timer.set_duration(Duration::from_millis(100));
                }
                _ => {}
            }
        }
        
        // Batch sprite draws
        self.sprite_batcher.batch_sprites(visible);
    }
}
```

## Implementation Phases

### Phase 1: Core Rendering (Week 1)
1. Implement `CartoonRenderingPlugin`
2. Set up isometric coordinate system
3. Create sprite batching system
4. Implement depth sorting

### Phase 2: Asset Pipeline (Week 1-2)
1. Create texture atlas loader
2. Implement hot-reloading
3. Set up sprite sheet configuration
4. Create placeholder system

### Phase 3: Creature Visuals (Week 2-3)
1. Implement `CartoonSprite` component
2. Create animation state machine
3. Add expression overlay system
4. Implement genetic variations

### Phase 4: World Rendering (Week 3-4)
1. Create biome tile renderer
2. Implement chunk-based loading
3. Add biome transitions
4. Place decorative elements

### Phase 5: Effects & Polish (Week 4-5)
1. Implement particle system
2. Add weather effects
3. Create UI overlays
4. Add screen effects

### Phase 6: Optimization (Week 5-6)
1. Implement LOD system
2. Add performance monitoring
3. Optimize sprite batching
4. Fine-tune quality settings

## Integration Points

### With Existing Systems
- **Spatial Grid**: Use for visibility culling
- **Event System**: Trigger particles on state changes
- **Save/Load**: Serialize visual states
- **Time Scaling**: Adjust animation speeds

### With egui
- Render overlays in screen space
- Use egui for speech bubbles
- Integrate with existing panels

### With Bevy ECS
- Use Bevy's built-in sprite rendering
- Leverage transform hierarchy
- Utilize asset server

## Performance Targets
- 500 creatures at 60 FPS (release mode)
- < 1000 draw calls per frame
- < 500MB texture memory
- < 16ms frame time

## Testing Strategy
1. Unit tests for coordinate conversion
2. Integration tests for asset loading
3. Performance benchmarks
4. Visual regression tests

## Risk Mitigation
- **Asset Creation**: Use procedural generation for placeholders
- **Performance**: Implement LOD early
- **Memory**: Stream textures based on visibility
- **Complexity**: Incremental feature rollout