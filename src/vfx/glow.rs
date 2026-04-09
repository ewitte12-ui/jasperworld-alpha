use bevy::prelude::*;

use crate::player::components::Player;

/// Attach to any entity (enemy, door, etc.) that should show a yellow glow when
/// the player is within `radius` world units.
#[derive(Component)]
pub struct ProximityGlow {
    /// XY distance threshold (world units) at which the glow becomes visible.
    pub radius: f32,
    /// Size of the glow quad for non-mesh entities (doors).
    /// Ignored for entities with Mesh3d (enemies use their own mesh for contour glow).
    pub glow_size: Vec2,
}

/// Marker placed on the child entity that renders the glow.
/// Used to find and despawn the glow when the player moves away.
#[derive(Component)]
pub struct GlowIndicator;

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
                    base_color: Color::srgba(1.0, 0.85, 0.0, 0.5),
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

/// Spawns or despawns a yellow glow child on entities with [`ProximityGlow`]
/// based on XY distance to the player.
///
/// For enemies (entities with `Mesh3d` + `MeshMaterial3d`): spawns a slightly
/// scaled-up copy of the same mesh with a yellow-tinted version of the same
/// texture. Because the mesh handle is shared, animation UV updates apply to
/// the glow automatically. The texture alpha channel masks the glow to the
/// sprite's contour.
///
/// For doors (entities without `Mesh3d`): spawns a plain additive rectangle.
pub(crate) fn update_proximity_glow(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_query: Query<&Transform, With<Player>>,
    glowable_query: Query<(Entity, &Transform, &ProximityGlow, Option<&Children>)>,
    glow_child_query: Query<Entity, With<GlowIndicator>>,
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

        let existing_glow_children: Vec<Entity> = maybe_children
            .map(|children| {
                children
                    .iter()
                    .filter(|child| glow_child_query.get(*child).is_ok())
                    .collect()
            })
            .unwrap_or_default();

        let has_glow = !existing_glow_children.is_empty();

        let should_show = if has_glow {
            dist <= proximity_glow.radius + HYSTERESIS
        } else {
            dist <= proximity_glow.radius
        };

        match (should_show, has_glow) {
            (true, false) => {
                // Check if entity has a sprite mesh (enemy) or is a 3D model (door).
                if let (Ok(parent_mesh), Ok(parent_mat)) =
                    (mesh_query.get(entity), mat_query.get(entity))
                {
                    // ── Enemy contour glow ───────────────────────────────────
                    // Clone the mesh handle so animation UV changes propagate
                    // to the glow child automatically. The sprite texture's
                    // alpha channel masks the glow to the sprite's silhouette.
                    let texture = materials
                        .get(&parent_mat.0)
                        .and_then(|m| m.base_color_texture.clone());
                    info!(
                        "[GLOW] spawning contour glow for {:?}, texture={:?}",
                        entity,
                        texture.is_some()
                    );
                    let glow_mat = materials.add(StandardMaterial {
                        base_color: Color::srgba(1.0, 0.9, 0.0, 1.0),
                        base_color_texture: texture,
                        alpha_mode: AlphaMode::Blend,
                        unlit: true,
                        double_sided: true,
                        cull_mode: None,
                        ..default()
                    });
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
                    let glow_mesh = cache.door_mesh(&mut meshes, proximity_glow.glow_size);
                    let glow_mat = cache.door_material(&mut materials);
                    commands.entity(entity).with_children(|parent| {
                        parent.spawn((
                            Mesh3d(glow_mesh),
                            MeshMaterial3d(glow_mat),
                            Transform::from_xyz(0.0, 0.0, -0.5),
                            GlowIndicator,
                        ));
                    });
                }
            }
            (false, true) => {
                for child_entity in existing_glow_children {
                    commands.entity(child_entity).despawn();
                }
            }
            _ => {}
        }
    }
}
