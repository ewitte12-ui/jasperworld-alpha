use bevy::prelude::*;

use super::components::Player;
use crate::level::level_data::{CurrentLevel, LevelData};
use crate::rendering::camera::GameplayCamera;
use crate::tilemap::tilemap::TILE_SIZE;

const CAMERA_LERP_SPEED: f32 = 5.0;
/// Shift the camera upward so the player appears in the lower portion of the screen,
/// keeping the ground near the bottom edge.
const CAMERA_Y_OFFSET: f32 = 80.0;

/// Lerps the camera position toward the player. Camera z stays fixed at 100.0.
#[allow(clippy::type_complexity)]
pub fn camera_follow(
    time: Res<Time>,
    player_query: Query<&Transform, (With<Player>, Without<Camera3d>)>,
    // WHY GameplayCamera: bare With<Camera3d> is invalid per camera_role_identity_guardrail.
    // Must target the gameplay camera specifically so title/debug cameras don't interfere.
    mut camera_query: Query<&mut Transform, (With<Camera3d>, With<GameplayCamera>, Without<Player>)>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };

    let dt = time.delta_secs();
    let lerp_factor = CAMERA_LERP_SPEED * dt;

    let target_x = player_transform.translation.x;
    let target_y = player_transform.translation.y + CAMERA_Y_OFFSET;

    let dx = target_x - camera_transform.translation.x;
    let dy = target_y - camera_transform.translation.y;

    // WHY snap threshold: on level transition the player teleports up to ~1700 units
    // away.  Lerping from the old position takes ~20 frames (0.33 s) during which the
    // camera shows an empty despawned scene, causing a visible flash.  Any gap larger
    // than a single screen width (~570 units) must be a teleport — snap immediately.
    if dx.abs() > 400.0 || dy.abs() > 400.0 {
        camera_transform.translation.x = target_x;
        camera_transform.translation.y = target_y;
    } else {
        camera_transform.translation.x += dx * lerp_factor;
        camera_transform.translation.y += dy * lerp_factor;
    }
    // Keep camera z fixed (pixel snap handled by camera_snap system)
    camera_transform.translation.z = 100.0;
}

/// Prevents the player from walking outside the current level bounds.
pub fn player_clamp(
    current_level: Res<CurrentLevel>,
    level_data: Option<Res<LevelData>>,
    mut player_query: Query<&mut Transform, With<Player>>,
) {
    let Some(level_data) = level_data else { return };
    let Ok(mut tf) = player_query.single_mut() else { return };

    let layer = &level_data.layers[current_level.layer_index];
    let level_left = layer.origin_x;
    let level_right = layer.origin_x + layer.cols() as f32 * TILE_SIZE;

    tf.translation.x = tf.translation.x.clamp(level_left, level_right);
}
