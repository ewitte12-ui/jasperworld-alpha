use bevy::prelude::*;

use crate::combat::components::Health;
use crate::level::level_data::CurrentLevel;
use crate::player::components::Player;

use super::components::{
    CollectedEvent, Collectible, CollectibleType, CollectionProgress, MakeEmissive,
};

/// Marker component for collectibles that spin in place.
#[derive(Component)]
pub struct Spinning {
    pub speed: f32,
}

/// Rotates all Spinning entities around the Y axis.
pub fn spin_collectibles(time: Res<Time>, mut query: Query<(&mut Transform, &Spinning)>) {
    for (mut transform, spinning) in &mut query {
        transform.rotate_y(spinning.speed * time.delta_secs());
    }
}

/// Pickup collectibles when the player is within 24.0 units.
pub fn pickup_collectibles(
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
    collectible_query: Query<(Entity, &Transform, &Collectible)>,
    mut progress: ResMut<CollectionProgress>,
    current_level: Res<CurrentLevel>,
    mut health_query: Query<&mut Health, With<Player>>,
    mut collected_writer: MessageWriter<CollectedEvent>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (entity, collectible_transform, collectible) in &collectible_query {
        let collectible_pos = collectible_transform.translation.truncate();
        let distance = player_pos.distance(collectible_pos);

        if distance < 64.0 {
            commands.entity(entity).despawn();

            // Build a key for the collected_by_layer map. level_id is always Some
            // during normal gameplay (set before any collectibles exist), but the
            // guard prevents panics in debug/editor scenarios.
            let layer_key = current_level
                .level_id
                .map(|lid| (lid, current_level.layer_index));
            // Use logical_pos (the pre-offset position from compiled JSON) as the
            // persistence key. The spawn functions add a +6 Y visual offset to the
            // Transform, so collectible_transform.translation would be [x, y+6, z]
            // while spawn_entities_from_compiled uses the raw [x, y, z] as its key.
            // logical_pos stores the raw position so both sides match.
            let pos_key = CollectionProgress::pos_key(collectible.logical_pos);

            match collectible.collectible_type {
                CollectibleType::Star => {
                    if let Some(key) = layer_key {
                        progress
                            .collected_by_layer
                            .entry(key)
                            .or_default()
                            .stars
                            .insert(pos_key);
                    }
                    progress.stars_collected += 1;
                    info!(
                        "Star collected! {}/{}",
                        progress.stars_collected, progress.stars_total
                    );
                }
                CollectibleType::HealthFood => {
                    if let Some(key) = layer_key {
                        progress
                            .collected_by_layer
                            .entry(key)
                            .or_default()
                            .health_foods
                            .insert(pos_key);
                    }
                    if let Ok(mut health) = health_query.single_mut() {
                        health.current = (health.current + 20.0).min(health.max);
                    }
                }
            }

            collected_writer.write(CollectedEvent {
                collectible_type: collectible.collectible_type,
            });
        }
    }
}

/// Walks SceneRoot descendants of entities with `MakeEmissive`, clones their
/// `StandardMaterial`, sets emissive + unlit, then removes the marker.
/// Same pattern as `apply_scene_tints` in parallax.rs.
pub fn apply_emissive_to_collectibles(
    mut commands: Commands,
    query: Query<(Entity, &MakeEmissive, &Children)>,
    child_query: Query<&Children>,
    mat_query: Query<&MeshMaterial3d<StandardMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, emissive, top_children) in &query {
        let mut found_any = false;
        let mut stack: Vec<Entity> = top_children.iter().collect();
        while let Some(child) = stack.pop() {
            if let Ok(mat_handle) = mat_query.get(child)
                && let Some(original) = materials.get(&mat_handle.0)
            {
                let mut modified = original.clone();
                modified.emissive = emissive.color;
                if emissive.keep_lit {
                    // Neutral white base so PBR lighting gives clean 3D shading
                    // without the original texture being desaturated by cool
                    // City ambient light. The emissive provides the color.
                    modified.base_color = Color::WHITE;
                    modified.base_color_texture = None;
                } else {
                    modified.unlit = true;
                }
                modified.double_sided = true;
                modified.cull_mode = None;
                let new_handle = materials.add(modified);
                commands.entity(child).insert(MeshMaterial3d(new_handle));
                found_any = true;
            }
            if let Ok(grandchildren) = child_query.get(child) {
                stack.extend(grandchildren.iter());
            }
        }
        if found_any {
            info!(
                "[EMISSIVE] applied to entity {entity:?} keep_lit={}",
                emissive.keep_lit
            );
            commands.entity(entity).remove::<MakeEmissive>();
        }
    }
}

/// Spawns a collectible at the given world position.
///
/// Star      → star_collectible.glb (Kenney Platformer Kit, colormap.png texture)
/// HealthFood → apple.glb (Kenney Food Kit, vertex colors)
pub fn spawn_collectible(
    commands: &mut Commands,
    asset_server: &AssetServer,
    position: Vec3,
    collectible_type: CollectibleType,
    emissive: bool,
) {
    match collectible_type {
        CollectibleType::Star => spawn_star_3d(commands, asset_server, position, emissive),
        CollectibleType::HealthFood => {
            spawn_health_food_3d(commands, asset_server, position, emissive)
        }
    }
}

/// Spawns star_collectible.glb (gold star from Kenney Platformer Kit) for level progression.
///
/// Visual Y offset (+6): lifts the star model above platform surfaces so the
/// bottom of the 3D mesh does not clip into tile geometry. The collectible's
/// logical position (used by pickup_collectibles distance check) is the
/// Transform translation, which includes this offset — but pickup_radius is
/// 64 units, so 6 units of vertical shift has zero gameplay impact.
///
/// `position` is the raw pre-offset position from the compiled JSON, stored as
/// `Collectible::logical_pos` so that `pickup_collectibles` can use a key that
/// matches what `spawn_entities_from_compiled` uses for skipping.
fn spawn_star_3d(
    commands: &mut Commands,
    asset_server: &AssetServer,
    position: Vec3,
    emissive: bool,
) {
    let visual_offset = Vec3::new(0.0, 6.0, 0.0);
    let mut entity = commands.spawn((
        SceneRoot(asset_server.load("models/star_collectible.glb#Scene0")),
        Transform::from_translation(position + visual_offset).with_scale(Vec3::splat(50.0)),
        Collectible {
            collectible_type: CollectibleType::Star,
            logical_pos: position,
        },
        Spinning { speed: 1.5 },
    ));
    if emissive {
        entity.insert(MakeEmissive {
            color: LinearRgba::new(8.0, 6.8, 1.6, 1.0),
            keep_lit: false,
        });
        // Warm point light illuminates surrounding geometry in dark City scenes.
        entity.with_children(|parent| {
            parent.spawn((
                PointLight {
                    color: Color::srgb(1.0, 0.85, 0.3),
                    intensity: 50000.0,
                    range: 120.0,
                    shadows_enabled: false,
                    ..default()
                },
                Transform::default(),
            ));
        });
    }
}

/// Spawns apple.glb as a health pickup.
///
/// Same visual Y offset as stars — prevents clipping into platform surfaces.
/// `position` is stored as `Collectible::logical_pos` (pre-offset) for the
/// same persistence-key reason as `spawn_star_3d`.
fn spawn_health_food_3d(
    commands: &mut Commands,
    asset_server: &AssetServer,
    position: Vec3,
    emissive: bool,
) {
    let visual_offset = Vec3::new(0.0, 6.0, 0.0);
    let mut entity = commands.spawn((
        SceneRoot(asset_server.load("models/apple.glb#Scene0")),
        Transform::from_translation(position + visual_offset).with_scale(Vec3::splat(80.0)),
        Collectible {
            collectible_type: CollectibleType::HealthFood,
            logical_pos: position,
        },
        Spinning { speed: 1.5 },
    ));
    if emissive {
        entity.insert(MakeEmissive {
            color: LinearRgba::new(6.0, 2.0, 1.2, 1.0),
            keep_lit: false,
        });
        // Warm point light illuminates surrounding geometry in dark City scenes.
        entity.with_children(|parent| {
            parent.spawn((
                PointLight {
                    color: Color::srgb(1.0, 0.5, 0.2),
                    intensity: 40000.0,
                    range: 120.0,
                    shadows_enabled: false,
                    ..default()
                },
                Transform::default(),
            ));
        });
    }
}
