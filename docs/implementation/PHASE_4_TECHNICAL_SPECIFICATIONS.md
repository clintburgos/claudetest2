# Phase 4: Technical Specifications

## Overview

This document provides detailed technical specifications, performance budgets, memory constraints, and optimization strategies for Phase 4 implementation.

## Performance Budgets

### Frame Time Budget (60 FPS Target)

Total frame time: 16.67ms

| System | Budget | Priority | Notes |
|--------|--------|----------|-------|
| Particle Update | 2.0ms | High | GPU compute preferred |
| Particle Rendering | 1.5ms | High | Instanced rendering required |
| Weather Simulation | 1.0ms | Medium | Can run at 30Hz |
| Weather Rendering | 1.5ms | Medium | Shader-based effects |
| UI Update | 1.0ms | High | Immediate mode UI |
| UI Rendering | 2.0ms | High | Batched draw calls |
| Audio Processing | 0.5ms | High | Spatial audio mixing |
| Camera Updates | 0.5ms | High | Smooth transitions |
| **Total Phase 4** | **10.0ms** | - | 60% of frame budget |
| **Remaining** | **6.67ms** | - | For core simulation |

### Memory Budget

Total Phase 4 Memory: 150MB

| Component | Budget | Details |
|-----------|--------|---------|
| Particle Pools | 20MB | 5000 particles @ 4KB each |
| Weather Textures | 32MB | Rain, snow, fog textures |
| UI Textures | 16MB | Speech bubbles, indicators |
| Audio Buffers | 40MB | Spatial audio, variations |
| Render Targets | 20MB | PiP windows, effects |
| System Data | 22MB | State, caches, buffers |
| **Total** | **150MB** | - |

### Draw Call Budget

| Render Pass | Budget | Strategy |
|------------|--------|----------|
| Particles | 10 | Instanced by texture |
| Weather | 5 | Combined mesh passes |
| UI Elements | 20 | Batched by type |
| PiP Windows | 4 | One per window |
| **Total** | **39** | Under 50 target |

## System Specifications

### Particle System Specs

```rust
pub const PARTICLE_SPECS: ParticleSpecs = ParticleSpecs {
    // Global limits
    max_total_particles: 5000,
    max_emitters: 200,
    max_active_effects: 50,
    
    // Per-effect limits
    emotion_particles: ParticleLimit {
        max_per_creature: 20,
        max_total: 500,
        spawn_rate: 10.0, // per second
        lifetime_range: (0.5, 3.0),
    },
    
    weather_particles: ParticleLimit {
        max_per_emitter: 500,
        max_total: 2000,
        spawn_rate: 100.0,
        lifetime_range: (1.0, 5.0),
    },
    
    action_particles: ParticleLimit {
        max_per_action: 50,
        max_total: 300,
        spawn_rate: 50.0,
        lifetime_range: (0.2, 1.0),
    },
    
    // Performance scaling
    lod_distances: [20.0, 50.0, 100.0],
    lod_multipliers: [1.0, 0.5, 0.1],
    
    // GPU settings
    instance_buffer_size: 1024 * 1024, // 1MB
    vertex_buffer_size: 512 * 1024,    // 512KB
    update_frequency: UpdateFrequency::Hz60,
};
```

### Weather System Specs

```rust
pub const WEATHER_SPECS: WeatherSpecs = WeatherSpecs {
    // State machine
    min_weather_duration: 300.0,  // 5 minutes
    max_weather_duration: 1800.0, // 30 minutes
    transition_duration: 60.0,    // 1 minute
    
    // Precipitation
    rain_particles_per_m2: 10.0,
    snow_particles_per_m2: 15.0,
    max_precipitation_area: 10000.0, // m²
    
    // Accumulation
    puddle_formation_rate: 0.1,    // meters/hour
    snow_accumulation_rate: 0.05,  // meters/hour
    max_accumulation_depth: 0.5,   // meters
    
    // Performance
    weather_update_rate: UpdateFrequency::Hz30,
    accumulation_grid_size: 128,   // 128x128 heightmap
    fog_volume_resolution: 64,     // 64³ voxels
};
```

### UI System Specs

```rust
pub const UI_SPECS: UISpecs = UISpecs {
    // Speech bubbles
    max_concurrent_bubbles: 10,
    bubble_fade_duration: 0.5,
    bubble_lifetime_range: (2.0, 10.0),
    max_text_length: 100,
    text_wrap_width: 200.0,
    
    // Floating UI
    max_health_bars: 50,
    health_bar_timeout: 3.0,
    max_need_bars: 20,
    ui_update_rate: UpdateFrequency::Hz30,
    
    // Comic indicators
    max_indicators: 20,
    indicator_lifetime: 2.0,
    indicator_animation_fps: 30,
    
    // Picture-in-Picture
    max_pip_windows: 4,
    pip_resolution: (320, 240),
    pip_framerate: 30,
    pip_auto_close: 10.0,
    
    // Performance
    ui_batch_size: 100,
    text_cache_size: 1000,
};
```

### Audio System Specs

```rust
pub const AUDIO_SPECS: AudioSpecs = AudioSpecs {
    // Spatial audio
    max_simultaneous_sounds: 32,
    max_distance: 100.0,
    reference_distance: 1.0,
    rolloff_factor: 1.0,
    
    // Sound variations
    pitch_variation: 0.1,      // ±10%
    volume_variation: 0.05,    // ±5%
    
    // Performance
    audio_update_rate: UpdateFrequency::Hz60,
    spatial_resolution: 0.5,   // meters
    occlusion_rays: 4,        // for obstacle detection
    
    // Memory
    max_loaded_sounds: 100,
    sound_buffer_size: 64 * 1024, // 64KB per sound
    streaming_threshold: 1024 * 1024, // 1MB
};
```

## Quality Settings

### Low Quality

```rust
pub const LOW_QUALITY: QualitySettings = QualitySettings {
    particles: ParticleQuality {
        max_particles: 1000,
        update_rate: UpdateFrequency::Hz30,
        lod_bias: -1, // More aggressive LOD
        gpu_compute: false,
    },
    
    weather: WeatherQuality {
        precipitation_density: 0.3,
        accumulation_enabled: false,
        fog_quality: FogQuality::Simple,
        transition_steps: 10,
    },
    
    ui: UIQuality {
        max_speech_bubbles: 5,
        text_quality: TextQuality::Low,
        animations_enabled: false,
        pip_enabled: false,
    },
    
    audio: AudioQuality {
        max_sounds: 16,
        spatial_audio: false,
        variations_enabled: false,
    },
};
```

### Medium Quality

```rust
pub const MEDIUM_QUALITY: QualitySettings = QualitySettings {
    particles: ParticleQuality {
        max_particles: 3000,
        update_rate: UpdateFrequency::Hz60,
        lod_bias: 0,
        gpu_compute: false,
    },
    
    weather: WeatherQuality {
        precipitation_density: 0.7,
        accumulation_enabled: true,
        fog_quality: FogQuality::Volumetric,
        transition_steps: 30,
    },
    
    ui: UIQuality {
        max_speech_bubbles: 8,
        text_quality: TextQuality::Medium,
        animations_enabled: true,
        pip_enabled: true,
    },
    
    audio: AudioQuality {
        max_sounds: 24,
        spatial_audio: true,
        variations_enabled: true,
    },
};
```

### High Quality

```rust
pub const HIGH_QUALITY: QualitySettings = QualitySettings {
    particles: ParticleQuality {
        max_particles: 5000,
        update_rate: UpdateFrequency::Hz60,
        lod_bias: 1, // Less aggressive LOD
        gpu_compute: true,
    },
    
    weather: WeatherQuality {
        precipitation_density: 1.0,
        accumulation_enabled: true,
        fog_quality: FogQuality::VolumetricScattering,
        transition_steps: 60,
    },
    
    ui: UIQuality {
        max_speech_bubbles: 10,
        text_quality: TextQuality::High,
        animations_enabled: true,
        pip_enabled: true,
    },
    
    audio: AudioQuality {
        max_sounds: 32,
        spatial_audio: true,
        variations_enabled: true,
    },
};
```

## Optimization Strategies

### Particle Optimization

```rust
// Particle culling strategy
pub fn cull_particles(
    particles: &mut Vec<ParticleInstance>,
    camera_pos: Vec3,
    frustum: &Frustum,
    quality: &ParticleQuality,
) -> Vec<ParticleInstance> {
    let mut visible = Vec::with_capacity(particles.len());
    
    for particle in particles.iter() {
        // Distance culling
        let distance = (particle.position - camera_pos).length();
        if distance > PARTICLE_MAX_DISTANCE {
            continue;
        }
        
        // Frustum culling
        if !frustum.contains_point(particle.position) {
            continue;
        }
        
        // LOD culling
        let lod = calculate_lod(distance, quality.lod_bias);
        if random::<f32>() < lod {
            visible.push(*particle);
        }
        
        // Early exit if budget exceeded
        if visible.len() >= quality.max_particles {
            break;
        }
    }
    
    visible
}

// Particle batching
pub struct ParticleBatcher {
    batches: HashMap<TextureId, ParticleBatch>,
    instance_data: Vec<ParticleInstanceData>,
    vertex_buffer: Buffer,
    instance_buffer: Buffer,
}

impl ParticleBatcher {
    pub fn add_particle(&mut self, particle: &Particle) {
        let batch = self.batches.entry(particle.texture_id)
            .or_insert_with(|| ParticleBatch::new());
        
        batch.instances.push(ParticleInstanceData {
            transform: particle.transform.compute_matrix(),
            color: particle.color.as_linear_rgba_f32(),
            uv_offset_scale: particle.uv_rect.as_vec4(),
        });
    }
    
    pub fn flush(&mut self, render_pass: &mut RenderPass) {
        // Update instance buffer
        self.instance_buffer.write(&self.instance_data);
        
        // Render each batch with single draw call
        for (texture_id, batch) in &self.batches {
            render_pass.set_bind_group(0, &batch.bind_group, &[]);
            render_pass.draw_indexed(
                0..6, // Quad indices
                0,
                0..batch.instances.len() as u32,
            );
        }
        
        self.clear();
    }
}
```

### Weather Optimization

```rust
// Weather LOD system
pub fn optimize_weather_rendering(
    weather_state: &WeatherState,
    camera: &Camera,
    quality: &WeatherQuality,
) -> WeatherRenderParams {
    let base_params = weather_state.get_render_params();
    
    match quality.fog_quality {
        FogQuality::Simple => WeatherRenderParams {
            fog_samples: 4,
            fog_march_steps: 8,
            volumetric_enabled: false,
            ..base_params
        },
        FogQuality::Volumetric => WeatherRenderParams {
            fog_samples: 8,
            fog_march_steps: 16,
            volumetric_enabled: true,
            ..base_params
        },
        FogQuality::VolumetricScattering => WeatherRenderParams {
            fog_samples: 16,
            fog_march_steps: 32,
            volumetric_enabled: true,
            scattering_enabled: true,
            ..base_params
        },
    }
}

// Precipitation optimization
pub fn optimize_precipitation(
    area: Rect,
    density: f32,
    camera_pos: Vec3,
) -> Vec<PrecipitationEmitter> {
    // Divide area into cells
    let cell_size = 10.0;
    let mut emitters = Vec::new();
    
    for x in (area.min.x as i32..area.max.x as i32).step_by(cell_size as usize) {
        for y in (area.min.y as i32..area.max.y as i32).step_by(cell_size as usize) {
            let cell_center = Vec3::new(
                x as f32 + cell_size * 0.5,
                0.0,
                y as f32 + cell_size * 0.5,
            );
            
            // Only create emitters near camera
            let distance = (cell_center - camera_pos).length();
            if distance < PRECIPITATION_RENDER_DISTANCE {
                let lod_density = density * (1.0 - distance / PRECIPITATION_RENDER_DISTANCE);
                
                emitters.push(PrecipitationEmitter {
                    position: cell_center,
                    area: Vec2::splat(cell_size),
                    density: lod_density,
                });
            }
        }
    }
    
    emitters
}
```

### UI Optimization

```rust
// UI element pooling
pub struct UIElementPool<T> {
    available: Vec<T>,
    in_use: HashMap<Entity, T>,
    create_fn: Box<dyn Fn() -> T>,
}

impl<T> UIElementPool<T> {
    pub fn acquire(&mut self, entity: Entity) -> &mut T {
        let element = self.available.pop()
            .unwrap_or_else(|| (self.create_fn)());
        
        self.in_use.insert(entity, element);
        self.in_use.get_mut(&entity).unwrap()
    }
    
    pub fn release(&mut self, entity: Entity) {
        if let Some(element) = self.in_use.remove(&entity) {
            self.available.push(element);
        }
    }
}

// Text rendering cache
pub struct TextCache {
    rendered_texts: HashMap<TextKey, RenderedText>,
    lru_order: VecDeque<TextKey>,
    max_size: usize,
}

#[derive(Hash, Eq, PartialEq)]
struct TextKey {
    text: String,
    font_size: OrderedFloat<f32>,
    max_width: OrderedFloat<f32>,
}

impl TextCache {
    pub fn get_or_render(
        &mut self,
        text: &str,
        font_size: f32,
        max_width: f32,
    ) -> &RenderedText {
        let key = TextKey {
            text: text.to_string(),
            font_size: OrderedFloat(font_size),
            max_width: OrderedFloat(max_width),
        };
        
        // LRU update
        if let Some(pos) = self.lru_order.iter().position(|k| k == &key) {
            self.lru_order.remove(pos);
        }
        self.lru_order.push_front(key.clone());
        
        // Evict if over capacity
        while self.lru_order.len() > self.max_size {
            if let Some(old_key) = self.lru_order.pop_back() {
                self.rendered_texts.remove(&old_key);
            }
        }
        
        // Render if not cached
        self.rendered_texts.entry(key)
            .or_insert_with(|| render_text(text, font_size, max_width))
    }
}
```

### Audio Optimization

```rust
// Spatial audio culling
pub struct SpatialAudioSystem {
    active_sounds: Vec<ActiveSound>,
    sound_pool: SoundPool,
    listener_position: Vec3,
}

impl SpatialAudioSystem {
    pub fn update(&mut self, dt: f32) {
        // Sort by priority (distance + importance)
        self.active_sounds.sort_by_key(|sound| {
            let distance = (sound.position - self.listener_position).length();
            OrderedFloat(distance / sound.importance)
        });
        
        // Keep only top N sounds
        self.active_sounds.truncate(AUDIO_SPECS.max_simultaneous_sounds);
        
        // Update active sounds
        for sound in &mut self.active_sounds {
            let distance = (sound.position - self.listener_position).length();
            
            // Calculate attenuation
            let attenuation = calculate_attenuation(
                distance,
                AUDIO_SPECS.reference_distance,
                AUDIO_SPECS.max_distance,
                AUDIO_SPECS.rolloff_factor,
            );
            
            // Apply volume and panning
            sound.volume = sound.base_volume * attenuation;
            sound.pan = calculate_stereo_pan(
                sound.position,
                self.listener_position,
            );
            
            // Skip if too quiet
            if sound.volume < 0.01 {
                sound.active = false;
            }
        }
        
        // Remove inactive sounds
        self.active_sounds.retain(|s| s.active);
    }
}
```

## Profiling Markers

```rust
// Performance profiling integration
pub fn profile_phase4_systems(world: &mut World) {
    // Particle systems
    world.resource_scope(|world, mut profiler: Mut<Profiler>| {
        profiler.scope("Phase4::Particles::Update", || {
            update_particle_system(world);
        });
        
        profiler.scope("Phase4::Particles::Render", || {
            render_particles(world);
        });
    });
    
    // Weather systems
    world.resource_scope(|world, mut profiler: Mut<Profiler>| {
        profiler.scope("Phase4::Weather::Update", || {
            update_weather_system(world);
        });
        
        profiler.scope("Phase4::Weather::Render", || {
            render_weather_effects(world);
        });
    });
    
    // UI systems
    world.resource_scope(|world, mut profiler: Mut<Profiler>| {
        profiler.scope("Phase4::UI::Update", || {
            update_ui_elements(world);
        });
        
        profiler.scope("Phase4::UI::Render", || {
            render_ui_elements(world);
        });
    });
    
    // Audio systems
    world.resource_scope(|world, mut profiler: Mut<Profiler>| {
        profiler.scope("Phase4::Audio::Update", || {
            update_audio_system(world);
        });
    });
}
```

## Debug Visualization

```rust
// Debug overlays for Phase 4 systems
pub fn render_phase4_debug(
    mut gizmos: Gizmos,
    debug_settings: Res<Phase4DebugSettings>,
    particles: Query<&ParticleEmitter>,
    weather: Res<WeatherState>,
    ui_elements: Query<&Transform, With<UIElement>>,
) {
    if debug_settings.show_particle_bounds {
        for emitter in particles.iter() {
            gizmos.cube(
                emitter.transform.translation,
                emitter.transform.rotation,
                Vec3::splat(emitter.bounds_radius * 2.0),
                Color::YELLOW,
            );
        }
    }
    
    if debug_settings.show_weather_grid {
        // Draw weather accumulation grid
        for x in 0..WEATHER_SPECS.accumulation_grid_size {
            for y in 0..WEATHER_SPECS.accumulation_grid_size {
                let height = weather.get_accumulation(x, y);
                if height > 0.0 {
                    gizmos.line(
                        Vec3::new(x as f32, 0.0, y as f32),
                        Vec3::new(x as f32, height, y as f32),
                        Color::BLUE,
                    );
                }
            }
        }
    }
    
    if debug_settings.show_ui_bounds {
        for transform in ui_elements.iter() {
            gizmos.rect(
                transform.translation,
                transform.rotation,
                Vec2::new(100.0, 50.0), // Approximate UI size
                Color::GREEN,
            );
        }
    }
}
```

## Validation Criteria

### Performance Validation

```rust
#[cfg(test)]
mod performance_tests {
    use super::*;
    
    #[test]
    fn validate_particle_performance() {
        let mut system = ParticleSystem::new();
        
        // Add maximum particles
        for _ in 0..PARTICLE_SPECS.max_total_particles {
            system.spawn_particle(ParticleType::Rain);
        }
        
        // Measure update time
        let start = Instant::now();
        system.update(0.016);
        let elapsed = start.elapsed();
        
        assert!(elapsed.as_millis() < 2, "Particle update exceeded 2ms budget");
    }
    
    #[test]
    fn validate_memory_usage() {
        let particle_memory = PARTICLE_SPECS.max_total_particles * size_of::<ParticleInstance>();
        let weather_memory = WEATHER_SPECS.accumulation_grid_size.pow(2) * size_of::<f32>();
        let ui_memory = UI_SPECS.max_concurrent_bubbles * size_of::<SpeechBubble>();
        
        let total = particle_memory + weather_memory + ui_memory;
        
        assert!(total < 150 * 1024 * 1024, "Memory usage exceeds 150MB budget");
    }
}
```

### Quality Validation

```rust
pub fn validate_quality_settings(settings: &QualitySettings) -> Result<(), String> {
    // Check particle limits
    if settings.particles.max_particles > PARTICLE_SPECS.max_total_particles {
        return Err("Particle count exceeds maximum".to_string());
    }
    
    // Check UI limits
    if settings.ui.max_speech_bubbles > UI_SPECS.max_concurrent_bubbles {
        return Err("Speech bubble count exceeds maximum".to_string());
    }
    
    // Check audio limits
    if settings.audio.max_sounds > AUDIO_SPECS.max_simultaneous_sounds {
        return Err("Sound count exceeds maximum".to_string());
    }
    
    Ok(())
}
```