use avian2d::prelude::LinearVelocity;
use bevy::prelude::*;

use crate::collectibles::components::{Collectible, CollectionProgress};
use crate::enemies::components::Enemy;
use crate::level::components::{Decoration, TileEntity};
use crate::level::level_data::{CurrentLevel, LevelData};
use crate::level::spawn_level_full;
use crate::player::components::Player;
use crate::puzzle::components::{LevelExit, LevelGate};
use crate::rendering::camera::GameplayCamera;
use crate::tilemap::tilemap::TILE_SIZE;
use crate::vfx::glow::GlowIndicator;

/// Half of the orthographic viewport height (FixedVertical 320 / 2).
const HALF_HEIGHT: f32 = 160.0;

/// Camera tilt corrections for Y-axis clamping.
///
/// The Camera3d is tilted -28° around X (looking downward), so it sees MORE
/// of the world below camera_y and LESS above:
///   view_bottom  = camera_y − 234   (large range below)
///   view_top     = camera_y + 128   (smaller range above)
///
/// Derived by tracing orthographic viewport corners (±160 in camera-local Y)
/// along the tilted forward vector to the z=0 plane.
const VIEW_BOTTOM_OFFSET: f32 = 234.0;
const VIEW_TOP_OFFSET: f32 = 128.0;

/// Transitions the player to a new layer when they press E near a
/// [`super::doors::TransitionDoor`]. Executes a full entity wipe and
/// respawns the target layer via [`spawn_level_full`] — the same canonical
/// path used by `check_level_exit` and `handle_new_game`.
///
/// This makes each layer a self-contained stage: sublevels get their own
/// fresh enemies/stars/gate/exit, and returning to layer 0 respawns all
/// layer-0 entities (including previously killed enemies).
///
/// Sets `transition_in_progress` and arms `transition_cooldown=1` so that
/// player input is suppressed until the despawn commands have applied.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn switch_layer(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut current_level: ResMut<CurrentLevel>,
    level_data: Option<Res<LevelData>>,
    mut game_progress: ResMut<crate::puzzle::components::GameProgress>,
    mut collection_progress: ResMut<CollectionProgress>,
    mut player_query: Query<(&mut Transform, &mut LinearVelocity), With<Player>>,
    // Dual-purpose: proximity check (Transform) AND cleanup despawn (Entity).
    door_query: Query<(Entity, &Transform, &super::doors::TransitionDoor), Without<Player>>,
    tile_entities: Query<Entity, With<TileEntity>>,
    decoration_entities: Query<Entity, With<Decoration>>,
    enemy_entities: Query<Entity, With<Enemy>>,
    collectible_entities: Query<Entity, With<Collectible>>,
    // WHY Or<>: Bevy 0.18 system limit is 16 parameters. Merging three cleanup-only
    // markers into one Or query keeps us at exactly 16. LevelGate is included here
    // (unlike check_level_exit) because switch_layer can fire BEFORE all stars are
    // collected, so a gate may still exist; check_level_exit relies on check_gate
    // having already cleared it.
    cleanup_bundle: Query<
        Entity,
        Or<(With<GlowIndicator>, With<LevelExit>, With<LevelGate>)>,
    >,
) {
    // ── Guard: E-key ─────────────────────────────────────────────────────
    if !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }

    // ── Guard: transition lock / game over ───────────────────────────────
    if game_progress.transition_in_progress || game_progress.game_complete {
        return;
    }

    // ── Guard: required state ────────────────────────────────────────────
    let Some(current_level_id) = current_level.level_id else {
        return;
    };
    let Some(level_data_ref) = level_data.as_ref() else {
        return;
    };
    let num_layers = level_data_ref.layers.len();
    if num_layers == 0 {
        return;
    }

    // ── Proximity: find nearest TransitionDoor within 60 units ───────────
    let Ok((player_tf, _)) = player_query.single() else {
        return;
    };
    let player_pos = player_tf.translation.truncate();

    let Some((_, _, nearest_door)) = door_query
        .iter()
        .filter(|(_, t, _)| t.translation.truncate().distance(player_pos) < 60.0)
        .min_by(|(_, a, _), (_, b, _)| {
            let da = a.translation.truncate().distance(player_pos);
            let db = b.translation.truncate().distance(player_pos);
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })
    else {
        return;
    };

    // Use the door's explicit target layer instead of blind cycling.
    // WHY: sublevel return doors need target_layer=0, which a
    // `(index + 1) % num_layers` formula cannot express.
    let target = nearest_door.target_layer;
    if target >= num_layers {
        warn!(
            "[TRANSITION] door target_layer={target} out of range (num_layers={num_layers})"
        );
        return;
    }
    if target == current_level.layer_index {
        return; // defensive: already on the target layer
    }

    info!(
        "[TRANSITION] switch_layer: level={:?} layer {} -> {}",
        current_level_id, current_level.layer_index, target
    );

    // ── Lock the transition (cleared next frame by tick_transition_cooldown) ─
    // Note: level_data_ref (a &Res<LevelData>) and level_data (Option<Res>)
    // are no longer used after this point. Commands that re-insert LevelData
    // are deferred, so the immutable borrow does not conflict with the
    // spawn_level_full call below — no explicit drop needed.
    game_progress.transition_in_progress = true;

    // ── Full wipe — mirror check_level_exit, plus doors + LevelGate ──────
    // All TransitionDoor entities (L0 transition doors + L1 return door)
    // are now JSON-spawned via spawn_entities_from_compiled and carry
    // only the TransitionDoor marker (no TileEntity), so they're cleaned
    // up here via door_query.
    for entity in tile_entities
        .iter()
        .chain(decoration_entities.iter())
        .chain(enemy_entities.iter())
        .chain(collectible_entities.iter())
        .chain(door_query.iter().map(|(e, _, _)| e))
        .chain(cleanup_bundle.iter())
    {
        commands.entity(entity).despawn();
    }

    // ── Reset per-layer progress ─────────────────────────────────────────
    // Reset per-layer counts to match check_level_exit's contract.
    // spawn_level_full → spawn_entities_from_compiled will immediately set
    // stars_total to the target layer's count.
    collection_progress.stars_collected = 0;
    collection_progress.stars_total = 0;

    // ── Respawn the target layer via the canonical shared path ──────────
    // Handles: tiles, JSON entities (enemies, stars, gate, exit, layer-0/2
    // props, doors), LevelData resource, biome decorations (parallax),
    // Subdivision layer-2 solar panel canopy, and layer-1 sublevel setup
    // (dark background, sublevel decorations + emissive props, return door).
    let spawn = spawn_level_full(
        &mut commands,
        &mut meshes,
        &mut materials,
        &asset_server,
        &mut collection_progress,
        &mut current_level,
        current_level_id,
        target,
        false, // skip_enemies = false (matches check_level_exit)
    );

    // ── Teleport player and zero velocity ────────────────────────────────
    if let Ok((mut player_tf, mut player_vel)) = player_query.single_mut() {
        player_tf.translation.x = spawn.x;
        player_tf.translation.y = spawn.y;
        // Zero velocity so entry vector does not carry into the new layer.
        *player_vel = LinearVelocity::ZERO;
    }

    // ── Arm cooldown — tick_transition_cooldown clears the lock next frame ─
    game_progress.transition_cooldown = 1;
}

/// Clamps the camera position so it never scrolls outside the current layer bounds.
/// Must run after `camera_follow`.
pub fn camera_clamp(
    current_level: Res<CurrentLevel>,
    level_data: Option<Res<LevelData>>,
    // WHY GameplayCamera: bare With<Camera3d> is invalid per camera_role_identity_guardrail.
    // single_mut() is only safe when the query is guaranteed unique by role marker.
    mut camera_query: Query<&mut Transform, (With<Camera3d>, With<GameplayCamera>)>,
    windows: Query<&Window>,
) {
    let Some(level_data) = level_data else { return };
    let Ok(mut cam) = camera_query.single_mut() else {
        return;
    };

    let layer = &level_data.layers[current_level.layer_index];
    let cols = layer.cols() as f32;
    let rows = layer.rows() as f32;

    let level_left = layer.origin_x;
    let level_right = layer.origin_x + cols * TILE_SIZE;
    let level_bottom = layer.origin_y;
    let level_top = layer.origin_y + rows * TILE_SIZE;

    // Compute half-width from actual window aspect ratio so the clamp
    // is correct regardless of window size, title-bar chrome, or DPI.
    let half_width = if let Ok(window) = windows.single() {
        HALF_HEIGHT * (window.width() / window.height())
    } else {
        HALF_HEIGHT * 1.5 // fallback: 960/640
    };

    let min_x = level_left + half_width;
    let max_x = level_right - half_width;

    // Y: corrected for the -28° camera tilt.
    // view_bottom = camera_y − VIEW_BOTTOM_OFFSET  →  camera_y ≥ level_bottom + VIEW_BOTTOM_OFFSET
    // view_top    = camera_y + VIEW_TOP_OFFSET      →  camera_y ≤ level_top   − VIEW_TOP_OFFSET
    let min_y = level_bottom + VIEW_BOTTOM_OFFSET;
    let max_y = level_top - VIEW_TOP_OFFSET;

    // Only clamp if the level is wider / taller than the viewport.
    if min_x <= max_x {
        cam.translation.x = cam.translation.x.clamp(min_x, max_x);
    } else {
        // Level narrower than viewport: centre the camera.
        cam.translation.x = (level_left + level_right) * 0.5;
    }

    if min_y <= max_y {
        cam.translation.y = cam.translation.y.clamp(min_y, max_y);
    } else {
        cam.translation.y = (level_bottom + level_top) * 0.5;
    }

    // Debug: log camera bounds after layer switch
    if current_level.is_changed() {
        info!(
            "[CAMERA_CLAMP] layer={} bounds=({level_left}..{level_right}, {level_bottom}..{level_top}) \
             cam=({:.1}, {:.1}) min_x={min_x:.1} max_x={max_x:.1} min_y={min_y:.1} max_y={max_y:.1} \
             half_width={half_width:.1}",
            current_level.layer_index, cam.translation.x, cam.translation.y,
        );
    }
}
