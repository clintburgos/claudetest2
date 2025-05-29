# Phase 4: Additional Implementation Details

## Overview

This document provides the missing implementation details for Phase 4, including lightning effects, particle-terrain collision, font rendering pipeline, and cross-system integration patterns.

## Lightning Effects Implementation

### Lightning System Architecture

```rust
#[derive(Resource)]
pub struct LightningSystem {
    pub active_strikes: Vec<LightningStrike>,
    pub pending_strikes: VecDeque<PendingStrike>,
    pub config: LightningConfig,
    pub mesh_cache: HashMap<u64, Handle<Mesh>>,
    pub next_strike_timer: Timer,
}

#[derive(Clone)]
pub struct LightningStrike {
    pub id: u64,
    pub start_pos: Vec3,
    pub end_pos: Vec3,
    pub branches: Vec<LightningBranch>,
    pub intensity: f32,
    pub lifetime: f32,
    pub age: f32,
    pub flash_state: FlashState,
}

#[derive(Clone)]
pub struct LightningBranch {
    pub points: Vec<Vec3>,
    pub width: f32,
    pub intensity: f32,
    pub branch_level: u8,
    pub glow_radius: f32,
}

#[derive(Clone, Copy, PartialEq)]
pub enum FlashState {
    PreFlash(f32),    // Build up
    MainFlash(f32),   // Bright flash
    Afterglow(f32),   // Fade out
    Complete,
}

pub struct LightningConfig {
    pub max_branches: usize,
    pub branch_probability: f32,
    pub branch_angle_variance: f32,
    pub segment_length: f32,
    pub width_falloff: f32,
    pub detail_level: u8,
    pub flash_duration: f32,
    pub afterglow_duration: f32,
}
```

### Lightning Generation Algorithm

```rust
pub fn generate_lightning_strike(
    start: Vec3,
    end: Vec3,
    config: &LightningConfig,
) -> LightningStrike {
    let mut rng = thread_rng();
    let main_branch = generate_branch(
        start,
        end,
        config.segment_length,
        config.detail_level,
        &mut rng,
    );
    
    let mut branches = vec![main_branch.clone()];
    let mut branch_points = Vec::new();
    
    // Collect potential branch points
    for (i, point) in main_branch.points.iter().enumerate() {
        if i > 0 && i < main_branch.points.len() - 1 {
            if rng.gen::<f32>() < config.branch_probability {
                branch_points.push((i, *point));
            }
        }
    }
    
    // Generate branches
    for (index, branch_start) in branch_points.iter().take(config.max_branches) {
        let progress = *index as f32 / main_branch.points.len() as f32;
        let branch_end = calculate_branch_endpoint(
            *branch_start,
            end,
            config.branch_angle_variance,
            progress,
            &mut rng,
        );
        
        let sub_branch = generate_branch(
            *branch_start,
            branch_end,
            config.segment_length * 0.7,
            config.detail_level - 1,
            &mut rng,
        );
        
        branches.push(LightningBranch {
            points: sub_branch.points,
            width: main_branch.width * config.width_falloff * (1.0 - progress),
            intensity: main_branch.intensity * 0.6,
            branch_level: 1,
            glow_radius: main_branch.glow_radius * 0.5,
        });
    }
    
    LightningStrike {
        id: generate_unique_id(),
        start_pos: start,
        end_pos: end,
        branches,
        intensity: 1.0,
        lifetime: config.flash_duration + config.afterglow_duration,
        age: 0.0,
        flash_state: FlashState::PreFlash(0.0),
    }
}

fn generate_branch(
    start: Vec3,
    end: Vec3,
    segment_length: f32,
    detail_level: u8,
    rng: &mut ThreadRng,
) -> LightningBranch {
    let direction = (end - start).normalize();
    let distance = start.distance(end);
    let segments = (distance / segment_length).ceil() as usize;
    
    let mut points = vec![start];
    let mut current = start;
    
    for i in 1..segments {
        let progress = i as f32 / segments as f32;
        let target = start.lerp(end, progress);
        
        // Add randomness perpendicular to direction
        let offset = if detail_level > 0 {
            let perpendicular = direction.cross(Vec3::Y).normalize();
            let perpendicular2 = direction.cross(perpendicular).normalize();
            
            let displacement = segment_length * 0.3 * (1.0 - progress * 0.5);
            perpendicular * rng.gen_range(-displacement..displacement)
                + perpendicular2 * rng.gen_range(-displacement..displacement)
        } else {
            Vec3::ZERO
        };
        
        current = target + offset;
        points.push(current);
    }
    
    points.push(end);
    
    // Smooth the path
    if detail_level > 1 {
        points = smooth_lightning_path(&points, 2);
    }
    
    LightningBranch {
        points,
        width: 2.0,
        intensity: 1.0,
        branch_level: 0,
        glow_radius: 5.0,
    }
}
```

### Lightning Rendering

```rust
pub fn render_lightning(
    mut commands: Commands,
    mut lightning_system: ResMut<LightningSystem>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    
    // Update existing strikes
    for strike in &mut lightning_system.active_strikes {
        strike.age += dt;
        
        // Update flash state
        strike.flash_state = match strike.flash_state {
            FlashState::PreFlash(t) if t < 0.05 => FlashState::PreFlash(t + dt),
            FlashState::PreFlash(_) => FlashState::MainFlash(0.0),
            FlashState::MainFlash(t) if t < lightning_system.config.flash_duration => {
                FlashState::MainFlash(t + dt)
            }
            FlashState::MainFlash(_) => FlashState::Afterglow(0.0),
            FlashState::Afterglow(t) if t < lightning_system.config.afterglow_duration => {
                FlashState::Afterglow(t + dt)
            }
            _ => FlashState::Complete,
        };
        
        // Update intensity based on flash state
        strike.intensity = match strike.flash_state {
            FlashState::PreFlash(t) => t / 0.05,
            FlashState::MainFlash(t) => {
                let flash_progress = t / lightning_system.config.flash_duration;
                1.0 + (flash_progress * std::f32::consts::PI).sin() * 0.5
            }
            FlashState::Afterglow(t) => {
                let fade_progress = t / lightning_system.config.afterglow_duration;
                (1.0 - fade_progress) * 0.3
            }
            FlashState::Complete => 0.0,
        };
    }
    
    // Remove completed strikes
    lightning_system.active_strikes.retain(|s| s.flash_state != FlashState::Complete);
    
    // Spawn mesh entities for active strikes
    for strike in &lightning_system.active_strikes {
        let mesh = generate_lightning_mesh(strike);
        let mesh_handle = meshes.add(mesh);
        
        let material = materials.add(StandardMaterial {
            base_color: Color::rgba(0.8, 0.8, 1.0, strike.intensity),
            emissive: Color::rgb(2.0, 2.0, 3.0) * strike.intensity,
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        });
        
        commands.spawn((
            PbrBundle {
                mesh: mesh_handle,
                material,
                ..default()
            },
            LightningMesh {
                strike_id: strike.id,
                lifetime: Timer::from_seconds(0.1, TimerMode::Once),
            },
        ));
        
        // Spawn glow effect
        if strike.intensity > 0.5 {
            for branch in &strike.branches {
                for (i, point) in branch.points.iter().enumerate() {
                    if i % 3 == 0 {  // Reduce glow point density
                        commands.spawn(PointLightBundle {
                            point_light: PointLight {
                                intensity: 10000.0 * strike.intensity * branch.intensity,
                                radius: branch.glow_radius,
                                color: Color::rgb(0.8, 0.8, 1.0),
                                shadows_enabled: false,
                                ..default()
                            },
                            transform: Transform::from_translation(*point),
                            ..default()
                        })
                        .insert(TemporaryLight {
                            lifetime: Timer::from_seconds(0.1, TimerMode::Once),
                        });
                    }
                }
            }
        }
    }
}

fn generate_lightning_mesh(strike: &LightningStrike) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();
    
    for branch in &strike.branches {
        let branch_mesh_data = generate_branch_geometry(
            &branch.points,
            branch.width * strike.intensity,
            branch.intensity,
        );
        
        let index_offset = positions.len() as u32;
        positions.extend(branch_mesh_data.positions);
        normals.extend(branch_mesh_data.normals);
        uvs.extend(branch_mesh_data.uvs);
        
        for idx in branch_mesh_data.indices {
            indices.push(idx + index_offset);
        }
    }
    
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.set_indices(Some(Indices::U32(indices)));
    
    mesh
}

// Lightning shader
pub const LIGHTNING_SHADER: &str = r#"
#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings

struct LightningMaterial {
    color: vec4<f32>,
    glow_intensity: f32,
    pulse_frequency: f32,
    distortion_amount: f32,
    time: f32,
}

@group(1) @binding(0)
var<uniform> material: LightningMaterial;

@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
    var color = material.color;
    
    // Add electrical distortion
    let distortion = sin(in.world_position.y * 10.0 + material.time * material.pulse_frequency) 
        * material.distortion_amount;
    
    // Glow effect based on UV
    let glow = pow(1.0 - abs(in.uv.x - 0.5) * 2.0, 2.0) * material.glow_intensity;
    
    color.rgb += vec3<f32>(glow);
    color.a *= 1.0 - abs(in.uv.x - 0.5) * 2.0;
    
    // Add slight animation
    color.rgb *= 1.0 + sin(material.time * 20.0) * 0.1;
    
    return color;
}
"#;
```

## Particle-Terrain Collision System

### Collision Detection Architecture

```rust
#[derive(Component)]
pub struct ParticleCollider {
    pub radius: f32,
    pub restitution: f32,
    pub friction: f32,
    pub collision_response: CollisionResponse,
    pub terrain_interaction: TerrainInteraction,
}

#[derive(Clone, Copy)]
pub enum CollisionResponse {
    Bounce,
    Stick,
    Splash,
    Absorb,
    Slide,
}

#[derive(Clone)]
pub struct TerrainInteraction {
    pub spawn_on_impact: Option<ParticleType>,
    pub sound_on_impact: Option<SoundId>,
    pub leave_mark: bool,
    pub mark_lifetime: f32,
    pub influence_terrain: Option<TerrainInfluence>,
}

#[derive(Clone)]
pub enum TerrainInfluence {
    Wetness(f32),
    Snow(f32),
    Scorch(f32),
    Displacement(f32),
}

pub fn update_particle_terrain_collisions(
    mut particles: Query<(
        &mut Transform,
        &mut ParticleVelocity,
        &ParticleCollider,
        &mut ParticleLifetime,
    )>,
    terrain: Res<TerrainHeightMap>,
    mut terrain_marks: ResMut<TerrainMarkSystem>,
    mut commands: Commands,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    
    for (mut transform, mut velocity, collider, mut lifetime) in particles.iter_mut() {
        let pos = transform.translation;
        let terrain_height = terrain.sample_height(pos.x, pos.z);
        
        // Check collision
        if pos.y - collider.radius <= terrain_height {
            // Calculate collision normal
            let normal = terrain.calculate_normal(pos.x, pos.z);
            
            match collider.collision_response {
                CollisionResponse::Bounce => {
                    // Reflect velocity
                    let dot = velocity.0.dot(normal);
                    velocity.0 -= 2.0 * dot * normal;
                    velocity.0 *= collider.restitution;
                    
                    // Adjust position
                    transform.translation.y = terrain_height + collider.radius;
                }
                
                CollisionResponse::Stick => {
                    velocity.0 = Vec3::ZERO;
                    transform.translation.y = terrain_height + collider.radius * 0.5;
                    lifetime.remaining = lifetime.remaining.min(1.0); // Fade out
                }
                
                CollisionResponse::Splash => {
                    // Spawn splash particles
                    if let Some(splash_type) = &collider.terrain_interaction.spawn_on_impact {
                        spawn_impact_particles(
                            &mut commands,
                            pos,
                            normal,
                            *splash_type,
                            velocity.0.length(),
                        );
                    }
                    lifetime.remaining = 0.0; // Remove particle
                }
                
                CollisionResponse::Absorb => {
                    // Sink into terrain
                    velocity.0 *= 1.0 - collider.friction * dt;
                    if velocity.0.length() < 0.1 {
                        lifetime.remaining = 0.0;
                    }
                }
                
                CollisionResponse::Slide => {
                    // Project velocity onto terrain surface
                    let dot = velocity.0.dot(normal);
                    velocity.0 -= dot * normal;
                    velocity.0 *= 1.0 - collider.friction * dt;
                    
                    transform.translation.y = terrain_height + collider.radius;
                }
            }
            
            // Apply terrain interaction
            if let Some(influence) = &collider.terrain_interaction.influence_terrain {
                apply_terrain_influence(
                    &mut terrain_marks,
                    pos.x,
                    pos.z,
                    influence,
                    dt,
                );
            }
            
            // Leave mark if configured
            if collider.terrain_interaction.leave_mark {
                terrain_marks.add_mark(TerrainMark {
                    position: Vec2::new(pos.x, pos.z),
                    radius: collider.radius * 2.0,
                    intensity: 1.0,
                    lifetime: collider.terrain_interaction.mark_lifetime,
                    mark_type: get_mark_type_for_particle(&collider),
                });
            }
        }
    }
}

// Optimized terrain sampling
pub struct TerrainHeightMap {
    data: Vec<f32>,
    width: usize,
    height: usize,
    scale: f32,
    cache: RwLock<HashMap<(i32, i32), TerrainChunk>>,
}

impl TerrainHeightMap {
    pub fn sample_height(&self, x: f32, z: f32) -> f32 {
        // Convert to grid coordinates
        let grid_x = (x / self.scale) as i32;
        let grid_z = (z / self.scale) as i32;
        
        // Check cache
        let chunk_key = (grid_x / 16, grid_z / 16);
        
        if let Some(chunk) = self.cache.read().unwrap().get(&chunk_key) {
            return chunk.sample_bilinear(x, z);
        }
        
        // Load chunk if not cached
        self.load_chunk(chunk_key);
        self.cache.read().unwrap()
            .get(&chunk_key)
            .map(|c| c.sample_bilinear(x, z))
            .unwrap_or(0.0)
    }
    
    pub fn calculate_normal(&self, x: f32, z: f32) -> Vec3 {
        let epsilon = 0.1;
        
        let h_x1 = self.sample_height(x - epsilon, z);
        let h_x2 = self.sample_height(x + epsilon, z);
        let h_z1 = self.sample_height(x, z - epsilon);
        let h_z2 = self.sample_height(x, z + epsilon);
        
        let dx = (h_x2 - h_x1) / (2.0 * epsilon);
        let dz = (h_z2 - h_z1) / (2.0 * epsilon);
        
        Vec3::new(-dx, 1.0, -dz).normalize()
    }
}
```

### Particle Impact Effects

```rust
pub fn spawn_impact_particles(
    commands: &mut Commands,
    impact_pos: Vec3,
    impact_normal: Vec3,
    particle_type: ParticleType,
    impact_velocity: f32,
) {
    let particle_count = (impact_velocity * 0.5).clamp(3.0, 20.0) as usize;
    
    match particle_type {
        ParticleType::WaterSplash => {
            // Spawn water droplets
            for i in 0..particle_count {
                let angle = (i as f32 / particle_count as f32) * std::f32::consts::TAU;
                let spread = random_range(0.2, 0.8);
                let up_velocity = random_range(1.0, 3.0) * impact_velocity * 0.3;
                
                let velocity = (impact_normal * up_velocity 
                    + Vec3::new(angle.cos() * spread, 0.0, angle.sin() * spread))
                    .normalize() * impact_velocity * 0.5;
                
                commands.spawn(WaterDropletBundle {
                    particle: Particle {
                        position: impact_pos + impact_normal * 0.1,
                        velocity,
                        lifetime: random_range(0.3, 0.8),
                        size: random_range(0.5, 1.5),
                        color: Color::rgba(0.6, 0.8, 1.0, 0.8),
                    },
                    collider: ParticleCollider {
                        radius: 0.1,
                        restitution: 0.3,
                        friction: 0.1,
                        collision_response: CollisionResponse::Splash,
                        terrain_interaction: TerrainInteraction {
                            spawn_on_impact: None,
                            sound_on_impact: Some(SoundId::WaterDroplet),
                            leave_mark: true,
                            mark_lifetime: 2.0,
                            influence_terrain: Some(TerrainInfluence::Wetness(0.1)),
                        },
                    },
                    ..default()
                });
            }
        }
        
        ParticleType::DirtKickup => {
            // Spawn dust particles
            for _ in 0..particle_count / 2 {
                let velocity = impact_normal * random_range(0.5, 2.0) 
                    + random_direction() * random_range(0.2, 1.0);
                
                commands.spawn(DustParticleBundle {
                    particle: Particle {
                        position: impact_pos,
                        velocity,
                        lifetime: random_range(1.0, 2.0),
                        size: random_range(1.0, 3.0),
                        color: Color::rgba(0.6, 0.5, 0.4, 0.5),
                    },
                    physics: ParticlePhysics {
                        gravity_multiplier: 0.3,
                        drag: 2.0,
                        turbulence: 0.5,
                    },
                    ..default()
                });
            }
        }
        
        _ => {}
    }
}
```

## Font Rendering Pipeline

### Text Rendering System

```rust
pub struct CartoonFontRenderer {
    pub font_atlas: FontAtlas,
    pub glyph_cache: HashMap<GlyphKey, RenderedGlyph>,
    pub text_mesh_pool: MeshPool<TextMesh>,
    pub outline_shader: Handle<Shader>,
}

#[derive(Hash, Eq, PartialEq)]
struct GlyphKey {
    character: char,
    font_size: OrderedFloat<f32>,
    style: FontStyle,
}

pub struct RenderedGlyph {
    pub texture_coords: Rect,
    pub size: Vec2,
    pub bearing: Vec2,
    pub advance: f32,
    pub outline_coords: Option<Rect>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum FontStyle {
    Regular,
    Bold,
    Italic,
    BoldItalic,
    Comic,
    Handwritten,
}

impl CartoonFontRenderer {
    pub fn render_text(
        &mut self,
        text: &str,
        font_size: f32,
        style: FontStyle,
        max_width: Option<f32>,
        outline: Option<TextOutline>,
    ) -> TextMeshData {
        let mut mesh_data = TextMeshData::new();
        let mut cursor = Vec2::ZERO;
        let line_height = font_size * 1.2;
        
        // Process text with word wrapping
        let wrapped_text = if let Some(width) = max_width {
            self.wrap_text(text, font_size, style, width)
        } else {
            vec![text.to_string()]
        };
        
        for line in wrapped_text {
            cursor.x = 0.0;
            
            for ch in line.chars() {
                if ch == ' ' {
                    cursor.x += font_size * 0.3;
                    continue;
                }
                
                let glyph = self.get_or_render_glyph(ch, font_size, style);
                
                // Add quad for character
                let pos = cursor + glyph.bearing;
                mesh_data.add_glyph_quad(
                    pos,
                    glyph.size,
                    glyph.texture_coords,
                    Color::WHITE,
                );
                
                // Add outline if requested
                if let Some(outline) = &outline {
                    if let Some(outline_coords) = glyph.outline_coords {
                        mesh_data.add_glyph_quad(
                            pos - Vec2::splat(outline.width),
                            glyph.size + Vec2::splat(outline.width * 2.0),
                            outline_coords,
                            outline.color,
                        );
                    }
                }
                
                cursor.x += glyph.advance;
            }
            
            cursor.y -= line_height;
        }
        
        mesh_data
    }
    
    fn wrap_text(
        &self,
        text: &str,
        font_size: f32,
        style: FontStyle,
        max_width: f32,
    ) -> Vec<String> {
        let mut lines = Vec::new();
        let mut current_line = String::new();
        let mut current_width = 0.0;
        
        for word in text.split_whitespace() {
            let word_width = self.measure_text(word, font_size, style);
            
            if current_width + word_width > max_width && !current_line.is_empty() {
                lines.push(current_line.trim().to_string());
                current_line = String::new();
                current_width = 0.0;
            }
            
            if !current_line.is_empty() {
                current_line.push(' ');
                current_width += font_size * 0.3;
            }
            
            current_line.push_str(word);
            current_width += word_width;
        }
        
        if !current_line.is_empty() {
            lines.push(current_line.trim().to_string());
        }
        
        lines
    }
}

// Signed Distance Field shader for smooth text
pub const SDF_TEXT_SHADER: &str = r#"
#import bevy_sprite::mesh2d_view_bindings
#import bevy_sprite::mesh2d_bindings

struct SDFTextMaterial {
    color: vec4<f32>,
    outline_color: vec4<f32>,
    outline_width: f32,
    softness: f32,
    shadow_offset: vec2<f32>,
    shadow_color: vec4<f32>,
}

@group(1) @binding(0)
var<uniform> material: SDFTextMaterial;

@group(1) @binding(1)
var font_texture: texture_2d<f32>;
@group(1) @binding(2)
var font_sampler: sampler;

@fragment
fn fragment(
    #import bevy_sprite::mesh2d_vertex_output
) -> @location(0) vec4<f32> {
    let sdf = textureSample(font_texture, font_sampler, in.uv).r;
    
    // Calculate distances for different effects
    let outline_distance = 0.5 - material.outline_width;
    let shadow_uv = in.uv + material.shadow_offset;
    let shadow_sdf = textureSample(font_texture, font_sampler, shadow_uv).r;
    
    // Smooth step for antialiasing
    let text_alpha = smoothstep(0.5 - material.softness, 0.5 + material.softness, sdf);
    let outline_alpha = smoothstep(outline_distance - material.softness, outline_distance + material.softness, sdf);
    let shadow_alpha = smoothstep(0.5 - material.softness * 2.0, 0.5 + material.softness * 2.0, shadow_sdf);
    
    // Composite layers
    var color = material.shadow_color;
    color.a *= shadow_alpha * 0.5;
    
    // Add outline
    color = mix(color, material.outline_color, outline_alpha * material.outline_color.a);
    
    // Add text
    color = mix(color, material.color, text_alpha * material.color.a);
    
    return color;
}
"#;

// Font atlas generation
pub struct FontAtlas {
    texture: Handle<Image>,
    glyphs: HashMap<char, AtlasGlyph>,
    size: Vec2,
    padding: f32,
}

impl FontAtlas {
    pub fn generate(
        font_data: &[u8],
        charset: &str,
        sizes: &[f32],
        include_outline: bool,
    ) -> Self {
        let font = Font::try_from_bytes(font_data).unwrap();
        let mut packer = RectPacker::new(2048, 2048);
        let mut glyphs = HashMap::new();
        
        // Rasterize all glyphs
        for size in sizes {
            for ch in charset.chars() {
                let glyph_data = rasterize_glyph(&font, ch, *size, include_outline);
                
                if let Some(rect) = packer.pack(
                    glyph_data.width + 2.0,
                    glyph_data.height + 2.0,
                ) {
                    glyphs.insert(ch, AtlasGlyph {
                        character: ch,
                        size: *size,
                        bounds: rect,
                        metrics: glyph_data.metrics,
                    });
                }
            }
        }
        
        // Generate texture
        let texture_data = packer.render_to_texture(&glyphs);
        
        Self {
            texture: create_texture_from_data(texture_data),
            glyphs,
            size: Vec2::new(2048.0, 2048.0),
            padding: 1.0,
        }
    }
}
```

### Speech Bubble Text Rendering

```rust
pub struct SpeechBubbleRenderer {
    pub bubble_mesh: Handle<Mesh>,
    pub font_renderer: CartoonFontRenderer,
    pub bubble_styles: HashMap<BubbleStyle, BubbleConfig>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum BubbleStyle {
    Speech,
    Thought,
    Shout,
    Whisper,
    System,
}

pub struct BubbleConfig {
    pub border_width: f32,
    pub border_radius: f32,
    pub padding: Vec2,
    pub tail_size: f32,
    pub tail_curve: f32,
    pub base_color: Color,
    pub border_color: Color,
    pub font_style: FontStyle,
    pub animation: BubbleAnimation,
}

#[derive(Clone)]
pub enum BubbleAnimation {
    None,
    PopIn { duration: f32, overshoot: f32 },
    Wobble { frequency: f32, amplitude: f32 },
    Pulse { frequency: f32, scale_range: (f32, f32) },
}

impl SpeechBubbleRenderer {
    pub fn create_speech_bubble(
        &mut self,
        text: &str,
        style: BubbleStyle,
        max_width: f32,
        speaker_pos: Vec3,
    ) -> SpeechBubbleBundle {
        let config = self.bubble_styles.get(&style).unwrap();
        
        // Render text
        let text_mesh = self.font_renderer.render_text(
            text,
            16.0,
            config.font_style,
            Some(max_width - config.padding.x * 2.0),
            Some(TextOutline {
                width: 1.0,
                color: config.border_color,
            }),
        );
        
        // Calculate bubble size
        let text_bounds = calculate_text_bounds(&text_mesh);
        let bubble_size = text_bounds + config.padding * 2.0;
        
        // Generate bubble mesh
        let bubble_mesh = self.generate_bubble_mesh(
            bubble_size,
            config,
            speaker_pos,
        );
        
        SpeechBubbleBundle {
            bubble: SpeechBubble {
                text: text.to_string(),
                style,
                lifetime: Timer::from_seconds(
                    calculate_reading_time(text),
                    TimerMode::Once,
                ),
                animation_state: AnimationState::default(),
            },
            mesh: bubble_mesh,
            material: create_bubble_material(config),
            transform: Transform::from_translation(
                speaker_pos + Vec3::new(0.0, 30.0, 0.0)
            ),
            ..default()
        }
    }
    
    fn generate_bubble_mesh(
        &self,
        size: Vec2,
        config: &BubbleConfig,
        speaker_pos: Vec3,
    ) -> Handle<Mesh> {
        let mut mesh_builder = MeshBuilder::new();
        
        // Create rounded rectangle
        let corners = [
            Vec2::new(-size.x * 0.5, size.y * 0.5),
            Vec2::new(size.x * 0.5, size.y * 0.5),
            Vec2::new(size.x * 0.5, -size.y * 0.5),
            Vec2::new(-size.x * 0.5, -size.y * 0.5),
        ];
        
        // Build bubble body with rounded corners
        for i in 0..4 {
            let start = corners[i];
            let end = corners[(i + 1) % 4];
            
            // Add corner arc
            let corner_center = start + (end - start).normalize() * config.border_radius;
            add_arc_to_mesh(
                &mut mesh_builder,
                corner_center,
                config.border_radius,
                i as f32 * 0.5 * std::f32::consts::PI,
                (i as f32 * 0.5 + 0.5) * std::f32::consts::PI,
                8,
            );
            
            // Add edge
            mesh_builder.add_line(
                corner_center + (end - start).normalize() * config.border_radius,
                end - (end - start).normalize() * config.border_radius,
                config.border_width,
            );
        }
        
        // Add tail
        let tail_direction = (speaker_pos - Vec3::new(0.0, 30.0, 0.0)).normalize();
        let tail_base = Vec2::new(0.0, -size.y * 0.5);
        let tail_tip = tail_base + Vec2::new(tail_direction.x, tail_direction.y) * config.tail_size;
        
        mesh_builder.add_bezier_curve(
            tail_base - Vec2::new(5.0, 0.0),
            tail_base + Vec2::new(-2.0, -5.0),
            tail_tip,
            tail_tip,
            config.tail_curve,
            10,
        );
        
        mesh_builder.add_bezier_curve(
            tail_tip,
            tail_tip,
            tail_base + Vec2::new(2.0, -5.0),
            tail_base + Vec2::new(5.0, 0.0),
            config.tail_curve,
            10,
        );
        
        mesh_builder.build()
    }
}

fn calculate_reading_time(text: &str) -> f32 {
    let words = text.split_whitespace().count();
    let base_time = 2.0; // Minimum display time
    let reading_speed = 200.0; // Words per minute
    
    base_time + (words as f32 * 60.0 / reading_speed)
}
```

## Cross-System Integration Patterns

### Event-Driven Integration

```rust
// Central event system for Phase 4
#[derive(Event)]
pub enum Phase4Event {
    // Particle events
    ParticleSpawned {
        emitter: Entity,
        particle_type: ParticleType,
        count: usize,
    },
    ParticleCollision {
        particle: Entity,
        surface: SurfaceType,
        impact_velocity: f32,
    },
    
    // Weather events
    WeatherChanged {
        from: WeatherType,
        to: WeatherType,
        transition_duration: f32,
    },
    LightningStrike {
        position: Vec3,
        intensity: f32,
    },
    
    // UI events
    SpeechBubbleSpawned {
        speaker: Entity,
        text: String,
        emotion: EmotionType,
    },
    CameraFocusRequest {
        target: Entity,
        duration: f32,
        zoom_level: f32,
    },
    
    // Audio events
    SoundTriggered {
        source: Entity,
        sound_id: SoundId,
        spatial: bool,
    },
}

// Integration coordinator
pub struct Phase4Coordinator {
    pub event_queue: VecDeque<Phase4Event>,
    pub system_states: HashMap<String, SystemState>,
    pub performance_monitor: PerformanceMonitor,
}

impl Phase4Coordinator {
    pub fn process_events(
        &mut self,
        world: &mut World,
        events: EventReader<Phase4Event>,
    ) {
        for event in events.iter() {
            match event {
                Phase4Event::ParticleSpawned { emitter, particle_type, count } => {
                    // Trigger audio based on particle type
                    if let Some(sound) = get_particle_spawn_sound(*particle_type) {
                        world.send_event(Phase4Event::SoundTriggered {
                            source: *emitter,
                            sound_id: sound,
                            spatial: true,
                        });
                    }
                    
                    // Update performance metrics
                    self.performance_monitor.record_particles(*count);
                }
                
                Phase4Event::WeatherChanged { to, .. } => {
                    // Adjust particle limits based on weather
                    world.resource_scope(|world, mut particle_system: Mut<ParticleSystem>| {
                        particle_system.adjust_limits_for_weather(*to);
                    });
                    
                    // Update audio ambience
                    world.send_event(AudioEvent::ChangeAmbience {
                        weather: *to,
                    });
                }
                
                Phase4Event::LightningStrike { position, intensity } => {
                    // Camera shake
                    world.send_event(CameraEvent::Shake {
                        magnitude: *intensity * 0.5,
                        duration: 0.3,
                    });
                    
                    // Thunder sound with delay
                    let distance = calculate_distance_to_player(*position, world);
                    let delay = distance / 343.0; // Speed of sound
                    
                    world.send_event_delayed(
                        Phase4Event::SoundTriggered {
                            source: Entity::PLACEHOLDER,
                            sound_id: SoundId::Thunder,
                            spatial: false,
                        },
                        delay,
                    );
                    
                    // Scare nearby creatures
                    scare_creatures_near(*position, *intensity * 50.0, world);
                }
                
                Phase4Event::SpeechBubbleSpawned { speaker, emotion, .. } => {
                    // Trigger vocalization
                    if let Some(voice_sound) = get_emotion_voice_sound(*emotion) {
                        world.send_event(Phase4Event::SoundTriggered {
                            source: *speaker,
                            sound_id: voice_sound,
                            spatial: true,
                        });
                    }
                    
                    // Update UI focus
                    world.send_event(UIEvent::FocusElement {
                        entity: *speaker,
                    });
                }
                
                _ => {}
            }
        }
    }
}
```

### System Communication Protocol

```rust
// Shared data structures between systems
#[derive(Resource)]
pub struct Phase4SharedData {
    pub particle_density_map: DensityMap,
    pub audio_priority_queue: BinaryHeap<AudioPriority>,
    pub active_effects: HashMap<Entity, Vec<ActiveEffect>>,
    pub performance_budget: PerformanceBudget,
}

// Performance-aware scheduling
pub struct Phase4Scheduler {
    pub frame_budget: FrameBudget,
    pub system_priorities: HashMap<SystemLabel, Priority>,
    pub adaptive_quality: AdaptiveQuality,
}

impl Phase4Scheduler {
    pub fn schedule_systems(
        &mut self,
        world: &mut World,
        available_time: f32,
    ) -> Vec<SystemLabel> {
        let mut scheduled = Vec::new();
        let mut remaining_time = available_time;
        
        // Sort systems by priority
        let mut systems: Vec<_> = self.system_priorities.iter().collect();
        systems.sort_by_key(|(_, priority)| -priority.value);
        
        for (label, priority) in systems {
            let estimated_cost = self.estimate_system_cost(label, world);
            
            if remaining_time >= estimated_cost || priority.is_critical {
                scheduled.push(label.clone());
                remaining_time -= estimated_cost;
                
                if remaining_time <= 0.0 && !priority.is_critical {
                    break;
                }
            }
        }
        
        // Adjust quality if over budget
        if remaining_time < 0.0 {
            self.adaptive_quality.reduce_quality(world);
        } else if remaining_time > available_time * 0.3 {
            self.adaptive_quality.increase_quality(world);
        }
        
        scheduled
    }
}

// Dependency injection for loose coupling
pub trait Phase4System: Send + Sync {
    fn dependencies(&self) -> Vec<TypeId>;
    fn provides(&self) -> Vec<TypeId>;
    fn run(&mut self, context: &mut SystemContext);
}

pub struct SystemContext<'a> {
    pub world: &'a mut World,
    pub shared_data: &'a mut Phase4SharedData,
    pub events: &'a mut Events<Phase4Event>,
    pub performance: &'a PerformanceMonitor,
}

// Example integrated system
pub struct IntegratedParticleAudioSystem {
    particle_sounds: HashMap<ParticleType, Vec<SoundId>>,
    cooldowns: HashMap<Entity, Timer>,
}

impl Phase4System for IntegratedParticleAudioSystem {
    fn dependencies(&self) -> Vec<TypeId> {
        vec![
            TypeId::of::<ParticleSystem>(),
            TypeId::of::<AudioSystem>(),
        ]
    }
    
    fn provides(&self) -> Vec<TypeId> {
        vec![TypeId::of::<ParticleAudioSync>()]
    }
    
    fn run(&mut self, context: &mut SystemContext) {
        // Query particle spawn events
        let particle_events: Vec<_> = context.world
            .resource::<Events<ParticleSpawnEvent>>()
            .get_reader()
            .iter()
            .cloned()
            .collect();
        
        for event in particle_events {
            // Check cooldown
            if let Some(cooldown) = self.cooldowns.get_mut(&event.emitter) {
                if !cooldown.finished() {
                    continue;
                }
                cooldown.reset();
            }
            
            // Select and play sound
            if let Some(sounds) = self.particle_sounds.get(&event.particle_type) {
                if let Some(sound_id) = sounds.choose(&mut thread_rng()) {
                    context.shared_data.audio_priority_queue.push(AudioPriority {
                        sound_id: *sound_id,
                        position: event.position,
                        priority: event.importance,
                        timestamp: Instant::now(),
                    });
                }
            }
        }
    }
}
```

This completes the missing implementation details for Phase 4, including lightning effects, particle-terrain collision, font rendering pipeline, and comprehensive cross-system integration patterns.