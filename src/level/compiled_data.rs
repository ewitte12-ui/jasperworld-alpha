/// Phase 6 — Game-Side JSON Loading
///
/// Deserializes `compiled_levels.json` (produced by the LDtk compiler) and
/// converts it to the game's existing types.  All public functions are called
/// from `mod.rs`; the deserialization types are crate-private.
///
/// On any parse / version / file error this module returns `None` / falls back
/// silently so the hardcoded level data continues to work unchanged.
use bevy::prelude::*;
use serde::Deserialize;

use crate::collectibles::components::{CollectibleType, CollectionProgress};
use crate::collectibles::systems::spawn_collectible;
use crate::enemies::components::EnemyType;
use crate::enemies::spawner::spawn_enemy;
use crate::puzzle::components::{LevelExit, LevelGate};
use crate::tilemap::tilemap::TileType;
use avian2d::prelude::*;

use super::doors::TransitionDoor;
use super::level_data::{LayerData, LevelData, LevelId};

// ── Deserialization structs (match output_schema from Phase 5) ───────────────

#[derive(Deserialize)]
pub struct CompiledRoot {
    pub schema_version: u32,
    pub levels: Vec<CompiledLevel>,
}

#[derive(Deserialize)]
pub struct CompiledLevel {
    pub id: String,
    pub layers: Vec<CompiledLayer>,
}

#[derive(Deserialize)]
pub struct CompiledLayer {
    pub id: usize,
    pub cols: i32,
    pub rows: i32,
    pub origin_x: f32,
    pub origin_y: f32,
    pub spawn: Option<[f32; 2]>,
    pub tiles: Vec<Vec<u8>>,
    pub enemies: Vec<CompiledEnemy>,
    pub stars: Vec<[f32; 3]>,
    pub health_foods: Vec<[f32; 3]>,
    pub doors: Vec<CompiledDoor>,
    /// Decorative prop entities placed visually in LDtk.
    /// Uses `#[serde(default)]` so compiled JSON without this field still parses
    /// correctly — backwards compatibility with pre-Prop compiled outputs.
    #[serde(default)]
    pub props: Vec<CompiledProp>,
    /// Point lights for this layer (sublevel lighting).
    /// Uses `#[serde(default)]` so JSON without this field still parses correctly
    /// — backwards compatibility with pre-light compiled outputs.
    #[serde(default)]
    pub lights: Vec<CompiledLight>,
    pub gate_col: Option<i32>,
    pub exit_next_level: Option<String>,
    pub stars_required: Option<i32>,
}

#[derive(Deserialize)]
pub struct CompiledEnemy {
    pub enemy_type: String,
    pub x: f32,
    pub y: f32,
    pub patrol_range: f32,
    pub health: f32,
    pub speed_override: Option<f32>,
}

#[derive(Deserialize)]
pub struct CompiledDoor {
    pub target_layer: i32,
    pub x: f32,
    pub y: f32,
}

#[derive(Deserialize)]
pub struct CompiledProp {
    pub model_id: String,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub scale_z: f32,
    pub rotation_y: f32,
    /// If true, entity gets ForegroundDecoration marker (z > 0 or visible gameplay props).
    #[serde(default)]
    pub foreground: bool,
}

/// A point light entry stored in the layer JSON.
/// Uses serde defaults for radius and range so existing JSON without those
/// fields continues to parse — backwards-compatible with pre-light outputs.
#[derive(Deserialize)]
pub struct CompiledLight {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub color: [f32; 3],
    pub intensity: f32,
    #[serde(default = "default_light_radius")]
    pub radius: f32,
    #[serde(default = "default_light_range")]
    pub range: f32,
}

fn default_light_radius() -> f32 {
    0.5
}
fn default_light_range() -> f32 {
    200.0
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Reads `path` from disk, parses JSON, checks schema_version == 1.
/// Returns `None` on any error (file missing, parse failure, wrong version).
/// All errors are logged at `warn!` level so fallback is silent but diagnosable.
pub fn try_load_compiled_levels(path: &str) -> Option<CompiledRoot> {
    let text = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            warn!("[compiled_data] could not read {path}: {e}");
            return None;
        }
    };

    let root: CompiledRoot = match serde_json::from_str(&text) {
        Ok(r) => r,
        Err(e) => {
            warn!("[compiled_data] JSON parse error in {path}: {e}");
            return None;
        }
    };

    if root.schema_version != 1 {
        warn!(
            "[compiled_data] unexpected schema_version {} in {path} (expected 1)",
            root.schema_version
        );
        return None;
    }

    Some(root)
}

/// Converts a `CompiledLevel` to the game's `LevelData` struct.
///
/// Tile byte values: 0 → Empty, 1 → Solid, 2 → Platform (anything else → Empty
/// with a warning so the game never panics on unknown values from the compiler).
pub fn compiled_to_level_data(compiled: &CompiledLevel, level_id: LevelId) -> LevelData {
    let layers = compiled
        .layers
        .iter()
        .map(|cl| {
            // Convert each row of bytes to TileType.
            // Compiled grid: rows × cols, row 0 = bottom (matches LayerData convention).
            let tiles: Vec<Vec<TileType>> = cl
                .tiles
                .iter()
                .map(|row| {
                    row.iter()
                        .map(|&b| match b {
                            0 => TileType::Empty,
                            1 => TileType::Solid,
                            2 => TileType::Platform,
                            other => {
                                warn!(
                                    "[compiled_data] unknown tile byte {other} in layer {} — treating as Empty",
                                    cl.id
                                );
                                TileType::Empty
                            }
                        })
                        .collect()
                })
                .collect();

            // Spawn point: use compiled value or fall back to a safe default
            // (col 3, row 3 — well clear of the ground surface).
            let spawn = cl
                .spawn
                .map(|[x, y]| Vec2::new(x, y))
                .unwrap_or_else(|| {
                    warn!(
                        "[compiled_data] layer {} has no spawn point; using default",
                        cl.id
                    );
                    Vec2::new(cl.origin_x + 3.0 * 18.0 + 9.0, cl.origin_y + 4.0 * 18.0)
                });

            LayerData {
                id: cl.id,
                tiles,
                origin_x: cl.origin_x,
                origin_y: cl.origin_y,
                spawn,
            }
        })
        .collect();

    LevelData { id: level_id, layers }
}

/// Spawns all gameplay entities (stars, health foods, enemies, doors, gate, exit)
/// from a single compiled layer.
///
/// This mirrors the logic in `spawn_forest_inner` / `spawn_subdivision_inner` /
/// `spawn_city_inner`, but driven entirely by JSON data.
///
/// # City shimmer
/// City-level stars use `emissive: true` (shimmer) to match the hardcoded path.
/// All other levels use `false`.
#[allow(clippy::too_many_arguments)]
pub fn spawn_entities_from_compiled(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    asset_server: &AssetServer,
    progress: &mut CollectionProgress,
    layer: &CompiledLayer,
    level_id: LevelId,
    skip_enemies: bool,
) {
    // City-level collectibles use the shimmer/emissive effect.
    let shimmer = matches!(level_id, LevelId::City);

    // ── Stars ────────────────────────────────────────────────────────────────
    for &[x, y, z] in &layer.stars {
        spawn_collectible(
            commands,
            asset_server,
            Vec3::new(x, y, z),
            CollectibleType::Star,
            shimmer,
        );
    }

    // Set stars_required (default 10 to match hardcoded levels if absent).
    let required = layer.stars_required.unwrap_or(10) as u32;
    progress.stars_total = required;
    progress.stars_collected = 0;

    // ── Health foods ─────────────────────────────────────────────────────────
    for &[x, y, z] in &layer.health_foods {
        spawn_collectible(
            commands,
            asset_server,
            Vec3::new(x, y, z),
            CollectibleType::HealthFood,
            shimmer,
        );
    }

    // ── Enemies ──────────────────────────────────────────────────────────────
    if !skip_enemies {
        for enemy in &layer.enemies {
            let Some(etype) = enemy_type_from_str(&enemy.enemy_type) else {
                continue; // warning already emitted by helper
            };
            spawn_enemy(
                commands,
                meshes,
                materials,
                asset_server,
                etype,
                Vec2::new(enemy.x, enemy.y),
                enemy.patrol_range,
                enemy.health,
                enemy.speed_override,
            );
        }
    }

    // ── Transition doors (layer doors from JSON) ──────────────────────────────
    // WHY: compiled_levels.json can include door data; if present we spawn them
    // here.  Note that `spawn_doors_for_level` in mod.rs still runs afterward
    // for levels not covered by the JSON path, so we only emit the JSON doors
    // when the caller uses the JSON path.
    for door in &layer.doors {
        if door.target_layer < 0 {
            warn!(
                "[compiled_data] door with negative target_layer {} — skipping",
                door.target_layer
            );
            continue;
        }
        commands.spawn((
            SceneRoot(asset_server.load("models/door-rotate.glb#Scene0")),
            Transform::from_xyz(door.x, door.y, 1.0)
                .with_scale(Vec3::new(60.0, 54.0, 7.0)),
            TransitionDoor {
                target_layer: door.target_layer as usize,
            },
        ));
    }

    // ── Props (decorative models placed in LDtk) ─────────────────────────────
    for prop in &layer.props {
        // rotation_y is stored in radians as authored in LDtk.
        let rotation = Quat::from_rotation_y(prop.rotation_y);
        if prop.model_id.ends_with(".png") {
            // PNG flat-quad prop: scale_x/scale_y are the rectangle dimensions in world units.
            // WHY: PNG assets are spawned as Rectangle meshes with StandardMaterial, not GLB scenes.
            //
            // WHY y + scale_y/2: LDtk entities use bottom-center pivot (pivotY=1), so prop.y
            // is the bottom edge of the prop in world space.  Bevy's Rectangle is center-anchored,
            // so without the offset the center sits at the ground surface and the sprite sinks
            // halfway underground.  Adding half the height shifts the center up so the bottom
            // aligns with prop.y — same technique used by the enemy spawner.
            let texture = asset_server.load(prop.model_id.clone());
            let mesh = meshes.add(Rectangle::new(prop.scale_x, prop.scale_y));
            let material = materials.add(StandardMaterial {
                base_color_texture: Some(texture),
                unlit: true,
                alpha_mode: AlphaMode::Blend,
                double_sided: true,
                cull_mode: None,
                ..default()
            });
            commands.spawn((
                Mesh3d(mesh),
                MeshMaterial3d(material),
                Transform::from_xyz(prop.x, prop.y + prop.scale_y / 2.0, prop.z)
                    .with_rotation(rotation),
                super::components::Decoration,
            ));
        } else {
            let mut entity = commands.spawn((
                SceneRoot(asset_server.load(format!("{}#Scene0", prop.model_id))),
                Transform::from_xyz(prop.x, prop.y, prop.z)
                    .with_rotation(rotation)
                    .with_scale(Vec3::new(prop.scale_x, prop.scale_y, prop.scale_z)),
                super::components::Decoration,
            ));
            if prop.foreground {
                entity.insert(super::components::ForegroundDecoration);
            }
        }
    }

    // ── Level gate ───────────────────────────────────────────────────────────
    if let Some(gate_col) = layer.gate_col {
        // gate_x: same formula as hardcoded — origin_x + col*18 + 9
        let gate_x = layer.origin_x + gate_col as f32 * 18.0 + 9.0;
        // ground_top: top surface of row-2 ground tiles = origin_y + 3 * TILE_SIZE
        let ground_top = layer.origin_y + 3.0 * 18.0;
        // Gate collider center is 200 units above ground_top (400-unit tall gate).
        let gate_center_y = ground_top + 200.0;

        // Sanctuary uses water as its visual endpoint — no gate, exit trigger,
        // or door model. The game ending is handled separately.
        if level_id != LevelId::Sanctuary {
            commands
                .spawn((
                    Transform::from_xyz(gate_x, gate_center_y, 1.0),
                    Visibility::default(),
                    RigidBody::Static,
                    Collider::rectangle(36.0, 400.0),
                    LevelGate,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        SceneRoot(asset_server.load("models/door-rotate-large.glb#Scene0")),
                        Transform::from_xyz(0.0, -200.0, 0.0)
                            .with_scale(Vec3::new(18.0, 80.0, 7.0)),
                    ));
                });

            // ── Level exit ───────────────────────────────────────────────────
            // Position: 30 units right of gate, at stand_y(2) = origin_y + 3*18 + 9
            let ground_y = layer.origin_y + 3.0 * 18.0 + 9.0;

            let next_level = layer
                .exit_next_level
                .as_deref()
                .and_then(level_id_from_str)
                // If JSON omits exit_next_level, stay on current level
                // (City does this — game_complete fires at level_index >= 3).
                .unwrap_or(level_id);

            commands.spawn((
                Transform::from_xyz(gate_x + 30.0, ground_y, 0.5),
                Visibility::Hidden,
                LevelExit {
                    next_level,
                    half_extents: Vec2::new(51.0, 100.0),
                },
            ));

            // End-zone landmark — open door prop as visual cue.
            commands.spawn((
                SceneRoot(asset_server.load("models/door-open.glb#Scene0")),
                Transform::from_xyz(gate_x + 40.0, ground_top, -1.0)
                    .with_scale(Vec3::new(60.0, 54.0, 7.0)),
                super::components::Decoration,
            ));
        }
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

/// Maps an enemy type string from JSON to the game's `EnemyType` enum.
/// Returns `None` and emits a `warn!` for unknown strings.
fn enemy_type_from_str(s: &str) -> Option<EnemyType> {
    match s {
        "Dog" => Some(EnemyType::Dog),
        "Squirrel" => Some(EnemyType::Squirrel),
        "Snake" => Some(EnemyType::Snake),
        "Rat" => Some(EnemyType::Rat),
        "Possum" => Some(EnemyType::Possum),
        other => {
            warn!("[compiled_data] unknown enemy_type \"{other}\" in JSON — skipping");
            None
        }
    }
}

/// Maps a level ID string from JSON to the game's `LevelId` enum.
/// Returns `None` and emits a `warn!` for unknown strings.
fn level_id_from_str(s: &str) -> Option<LevelId> {
    match s {
        "Forest" => Some(LevelId::Forest),
        "Subdivision" => Some(LevelId::Subdivision),
        "City" => Some(LevelId::City),
        "Sanctuary" => Some(LevelId::Sanctuary),
        other => {
            warn!("[compiled_data] unknown level id \"{other}\" in JSON — skipping");
            None
        }
    }
}
