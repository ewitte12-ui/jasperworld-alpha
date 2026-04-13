use bevy::{
    mesh::MeshVertexBufferLayoutRef,
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    render::render_resource::{AsBindGroup, RenderPipelineDescriptor, SpecializedMeshPipelineError},
    shader::ShaderRef,
};

use crate::player::components::Player;

/// Custom material that discards sprite RGB and outputs a solid HDR yellow,
/// masked by the sprite texture's alpha channel.  This avoids the dark-brown
/// multiply artefact produced when a yellow tint is blended over a
/// sprite using standard `AlphaMode::Blend`.
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct AlphaSilhouetteMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Option<Handle<Image>>,
}

impl Material for AlphaSilhouetteMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/alpha_silhouette.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

    /// Disable backface culling so the glow remains visible when the parent
    /// sprite is horizontally flipped (scale.x < 0), which reverses winding order.
    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}

/// Attach to any entity (enemy, door, etc.) that should show a yellow glow when
/// the player is within `radius` world units.
#[derive(Component)]
pub struct ProximityGlow {
    /// XY distance threshold (world units) at which the glow becomes visible.
    pub radius: f32,
    /// For non-sprite entities (doors): scale factor applied to the door's own
    /// world scale to compute the glow rect dimensions.
    /// e.g. Vec2::new(1.2, 1.2) → glow is 20% wider/taller than the door.
    /// Ignored for entities with Mesh3d (enemies use their own mesh).
    pub glow_size: Vec2,
}

/// Marker placed on the child/independent entity that renders the glow.
/// Used by level-transition cleanup queries to despawn glow entities.
#[derive(Component)]
pub struct GlowIndicator;

/// Placed on independent (non-child) glow entities to link them back to their
/// owning glowable entity.  Used for has-glow detection and despawn when the
/// owning entity leaves proximity, since these glows are not children of the
/// door and cannot be found via the parent's Children component.
#[derive(Component)]
pub struct GlowIndicatorFor(pub Entity);

/// Cached rectangle mesh and material for door glow (non-sprite entities).
/// Enemy glow materials are created per-entity since each has a different texture.
#[derive(Default)]
pub(crate) struct GlowCache {
    door_material: Option<Handle<StandardMaterial>>,
    door_meshes: bevy::platform::collections::HashMap<(u32, u32), Handle<Mesh>>,
}

impl GlowCache {
    fn door_material(&mut self, materials: &mut Assets<StandardMaterial>) -> Handle<StandardMaterial> {
        self.door_material
            .get_or_insert_with(|| {
                materials.add(StandardMaterial {
                    // Linear HDR values (>1.0) give the same luminance boost that
                    // the enemy AlphaSilhouetteMaterial shader uses (2.0/1.7).
                    base_color: Color::linear_rgba(2.0, 1.7, 0.0, 1.0),
                    alpha_mode: AlphaMode::Add,
                    unlit: true,
                    double_sided: true,
                    cull_mode: None,
                    ..default()
                })
            })
            .clone()
    }

    fn door_mesh(&mut self, meshes: &mut Assets<Mesh>, size: Vec2) -> Handle<Mesh> {
        let key = (size.x.to_bits(), size.y.to_bits());
        self.door_meshes
            .entry(key)
            .or_insert_with(|| meshes.add(Rectangle::new(size.x, size.y)))
            .clone()
    }
}

/// Hysteresis band (world units) to prevent flicker at the radius boundary.
const HYSTERESIS: f32 = 5.0;

/// Scale factor for the contour glow behind sprites.
/// 1.15 = 15% larger than the sprite, creating a visible border.
const CONTOUR_SCALE: f32 = 1.15;

/// Spawns or despawns a yellow glow on entities with [`ProximityGlow`]
/// based on XY distance to the player.
///
/// **Enemies** (entities with `Mesh3d` + `MeshMaterial3d`): spawns a
/// slightly scaled-up copy of the same mesh as a **child** entity using
/// [`AlphaSilhouetteMaterial`].  The shared mesh handle means animation UV
/// updates propagate to the glow automatically.
///
/// **Doors** (entities without `Mesh3d`): spawns an **independent world-space
/// entity** with a plain additive rectangle.  The glow is NOT a child of the
/// door so the door's non-uniform scale (60, 54, 7) and any rotation are
/// handled explicitly rather than inherited.  The glow position is computed
/// using `transform.rotation * Vec3::Z * 0.5` so it stays correctly placed
/// in front of the door regardless of how the door entity is oriented.
///
/// WHY independent entity for doors: a child of a SceneRoot with scale
/// (60, 54, 7) inherits that non-uniform scale on all axes.  Computing a
/// local-space transform that undoes the parent scale *and* rotation is
/// fragile and breaks when either changes.  World-space spawning is simpler
/// and more robust.
// WHY clippy::too_many_arguments allowed: this system needs player/glowable/child/mesh/material
// queries plus a local cache and a GlowIndicatorFor query; splitting would obscure the
// single-pass glow update logic.
#[allow(clippy::too_many_arguments)]
pub(crate) fn update_proximity_glow(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut std_materials: ResMut<Assets<StandardMaterial>>,
    mut sil_materials: ResMut<Assets<AlphaSilhouetteMaterial>>,
    player_query: Query<&Transform, With<Player>>,
    glowable_query: Query<(Entity, &Transform, &ProximityGlow, Option<&Children>)>,
    glow_child_query: Query<Entity, With<GlowIndicator>>,
    glow_for_query: Query<(Entity, &GlowIndicatorFor)>,
    mesh_query: Query<&Mesh3d>,
    mat_query: Query<&MeshMaterial3d<StandardMaterial>>,
    mut cache: Local<GlowCache>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (entity, transform, proximity_glow, maybe_children) in glowable_query.iter() {
        let entity_pos = transform.translation.truncate();
        let dist = player_pos.distance(entity_pos);

        let is_sprite = mesh_query.get(entity).is_ok() && mat_query.get(entity).is_ok();

        // ── has_glow detection ───────────────────────────────────────────────
        // Enemies: glow is a child — look in Children.
        // Doors:   glow is an independent entity — look in GlowIndicatorFor.
        let has_glow = if is_sprite {
            maybe_children
                .map(|children| children.iter().any(|c| glow_child_query.get(c).is_ok()))
                .unwrap_or(false)
        } else {
            glow_for_query.iter().any(|(_, g)| g.0 == entity)
        };

        let should_show = if has_glow {
            dist <= proximity_glow.radius + HYSTERESIS
        } else {
            dist <= proximity_glow.radius
        };

        match (should_show, has_glow) {
            (true, false) => {
                if is_sprite {
                    // ── Enemy contour glow ───────────────────────────────────
                    // Clone the mesh handle so animation UV changes propagate
                    // to the glow child automatically.  AlphaSilhouetteMaterial
                    // discards sprite RGB entirely and outputs pure HDR yellow,
                    // masked by the texture alpha — no colour-multiply artefact.
                    let parent_mesh = mesh_query.get(entity).unwrap();
                    let parent_mat  = mat_query.get(entity).unwrap();
                    let texture = std_materials
                        .get(&parent_mat.0)
                        .and_then(|m| m.base_color_texture.clone());
                    info!(
                        "[GLOW] spawning contour glow for {:?}, texture={:?}",
                        entity,
                        texture.is_some()
                    );
                    let glow_mat = sil_materials.add(AlphaSilhouetteMaterial { texture });
                    commands.entity(entity).with_children(|parent| {
                        parent.spawn((
                            Mesh3d(parent_mesh.0.clone()),
                            MeshMaterial3d(glow_mat),
                            Transform::from_xyz(0.0, 0.0, -0.5)
                                .with_scale(Vec3::splat(CONTOUR_SCALE)),
                            GlowIndicator,
                        ));
                    });
                } else {
                    // ── Door rectangle glow ──────────────────────────────────
                    // Spawned as an independent world-space entity (NOT a child)
                    // to avoid inheriting the door's non-uniform scale (60,54,7)
                    // and any rotation.
                    //
                    // World dimensions: door_scale * glow_size_factor.
                    let world_w = transform.scale.x * proximity_glow.glow_size.x;
                    let world_h = transform.scale.y * proximity_glow.glow_size.y;
                    //
                    // World position: center the glow on the door's body.
                    //
                    // Doors are bottom-anchored (.glb origin at feet), so the
                    // door body spans [origin.y, origin.y + scale.y].  Raise the
                    // glow by scale.y*0.5 (in the door's local up direction) so
                    // the Rectangle is centered on the door rather than its floor.
                    //
                    // Also offset along local +Z by (scale.z*0.5 + 2.0) to clear
                    // the door's deepest face plus any frames/handles, accounting
                    // for the camera's ~28° downward tilt.
                    let y_up = transform.rotation * Vec3::new(0.0, transform.scale.y * 0.5, 0.0);
                    let z_clearance = transform.scale.z * 0.5 + 2.0;
                    let z_forward = transform.rotation * Vec3::new(0.0, 0.0, z_clearance);
                    let glow_pos = transform.translation + y_up + z_forward;
                    //
                    // Rotation: same as door so rect faces the same direction.
                    let glow_mesh = cache.door_mesh(&mut meshes, Vec2::new(world_w, world_h));
                    let glow_mat  = cache.door_material(&mut std_materials);
                    commands.spawn((
                        Mesh3d(glow_mesh),
                        MeshMaterial3d(glow_mat),
                        Transform::from_translation(glow_pos)
                            .with_rotation(transform.rotation),
                        Visibility::Visible,
                        InheritedVisibility::VISIBLE,
                        GlowIndicator,
                        GlowIndicatorFor(entity),
                    ));
                }
            }
            (false, true) => {
                if is_sprite {
                    // Enemy: despawn GlowIndicator children.
                    if let Some(children) = maybe_children {
                        for child in children.iter() {
                            if glow_child_query.get(child).is_ok() {
                                commands.entity(child).despawn();
                            }
                        }
                    }
                } else {
                    // Door: despawn the independent GlowIndicatorFor entity.
                    for (glow_entity, _) in glow_for_query.iter().filter(|(_, g)| g.0 == entity) {
                        commands.entity(glow_entity).despawn();
                    }
                }
            }
            _ => {}
        }
    }
}
