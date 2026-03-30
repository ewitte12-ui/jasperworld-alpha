use avian2d::prelude::LinearVelocity;
use bevy::prelude::*;

use crate::level::components::TileEntity;
use crate::level::level_data::{CurrentLevel, LevelData};
use crate::player::components::Player;
use crate::rendering::camera::GameplayCamera;
use crate::tilemap::spawn::spawn_tilemap;
use crate::tilemap::tilemap::TILE_SIZE;

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

/// Cycles to the next layer when the player presses E, but only if the player
/// is within 60 units of a [`super::doors::TransitionDoor`].
/// Despawns all [`TileEntity`] entities, spawns the new layer's tiles,
/// and teleports the player to the new layer's spawn point.
///
/// Sets `transition_in_progress` for the duration of the switch so that
/// player input is suppressed and cannot fight the teleport.
#[allow(clippy::too_many_arguments)]
pub fn switch_layer(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut current_level: ResMut<CurrentLevel>,
    level_data: Option<Res<LevelData>>,
    tile_entities: Query<Entity, With<TileEntity>>,
    mut player_query: Query<(&mut Transform, &mut LinearVelocity), With<Player>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    door_query: Query<(&Transform, &super::doors::TransitionDoor), Without<Player>>,
    mut game_progress: ResMut<crate::puzzle::components::GameProgress>,
) {
    if !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }

    if game_progress.transition_in_progress || game_progress.game_complete {
        return;
    }

    let Some(level_data) = level_data else { return };

    let num_layers = level_data.layers.len();
    if num_layers == 0 {
        return;
    }

    // Only switch if player is near a door.
    let player_pos = player_query
        .single()
        .map(|(t, _)| t.translation.truncate())
        .unwrap_or_default();
    let near_door = door_query
        .iter()
        .any(|(door_tf, _)| door_tf.translation.truncate().distance(player_pos) < 60.0);
    if !near_door {
        return;
    }

    // TEMP LOG — remove after validation
    info!(
        "[TRANSITION] switch_layer TRIGGER: player={player_pos}, layer_idx={}",
        current_level.layer_index,
    );

    // Lock player input for the duration of the layer switch.
    game_progress.transition_in_progress = true;

    // Advance to next layer.
    current_level.layer_index = (current_level.layer_index + 1) % num_layers;

    // TEMP LOG — remove after validation
    info!(
        "[TRANSITION] switch_layer START: locked, advancing to layer_idx={}",
        current_level.layer_index,
    );
    let layer = &level_data.layers[current_level.layer_index];

    // Despawn all existing tile entities.
    for entity in &tile_entities {
        commands.entity(entity).despawn();
    }

    // Pick solid model based on current level theme.
    let solid_model = match current_level.level_id {
        Some(crate::level::level_data::LevelId::Subdivision)
        | Some(crate::level::level_data::LevelId::City) => "models/brick.glb",
        _ => "models/block-grass-large.glb",
    };

    // Spawn new layer tiles.
    // spawn_tilemap expects origin = world position of tile center (col=0, row=0).
    let origin = Vec2::new(
        layer.origin_x + TILE_SIZE * 0.5,
        layer.origin_y + TILE_SIZE * 0.5,
    );
    spawn_tilemap(&mut commands, &asset_server, solid_model, &layer.tiles, origin, 0.0);

    // Teleport player to the new layer's spawn point.
    if let Ok((mut player_transform, mut player_vel)) = player_query.single_mut() {
        player_transform.translation.x = layer.spawn.x;
        player_transform.translation.y = layer.spawn.y;
        // Zero velocity so entry vector does not carry into the new layer.
        *player_vel = LinearVelocity::ZERO;
    }

    // Arm cooldown — tick_transition_cooldown clears the lock after 1 frame,
    // ensuring deferred tile despawn commands have applied before re-entry.
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
}
