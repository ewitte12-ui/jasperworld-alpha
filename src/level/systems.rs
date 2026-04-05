use avian2d::prelude::LinearVelocity;
use bevy::prelude::*;

use crate::level::components::TileEntity;
use crate::level::level_data::{CurrentLevel, LevelData, LevelId};
use crate::player::components::Player;
use crate::rendering::camera::GameplayCamera;
use crate::tilemap::spawn::{spawn_tilemap, spawn_tilemap_tinted};
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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

    // Find nearest door within interaction range.
    let player_pos = player_query
        .single()
        .map(|(t, _)| t.translation.truncate())
        .unwrap_or_default();
    let nearest_door = door_query
        .iter()
        .filter(|(door_tf, _)| door_tf.translation.truncate().distance(player_pos) < 60.0)
        .min_by(|(a_tf, _), (b_tf, _)| {
            let da = a_tf.translation.truncate().distance(player_pos);
            let db = b_tf.translation.truncate().distance(player_pos);
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        });
    let Some((_, door)) = nearest_door else {
        return;
    };

    // Use the door's explicit target layer instead of blind cycling.
    // WHY: sublevel return doors need target_layer=0, which the old
    // `(index + 1) % num_layers` formula cannot express.
    let target = door.target_layer;
    if target >= num_layers {
        warn!("[TRANSITION] door target_layer={target} out of range (num_layers={num_layers})");
        return;
    }

    info!(
        "[TRANSITION] switch_layer: player={player_pos}, from layer {} -> {}",
        current_level.layer_index, target,
    );

    // Lock player input for the duration of the layer switch.
    game_progress.transition_in_progress = true;

    current_level.layer_index = target;

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

    // Pick tile models based on level + layer (sublevels use themed tiles).
    let (solid_model, platform_model) = match current_level.level_id {
        Some(lid) => crate::level::tile_models_for_layer(lid, current_level.layer_index),
        None => ("models/grass-block.glb", "models/grass-block.glb"),
    };

    // Spawn new layer tiles.
    // spawn_tilemap expects origin = world position of tile center (col=0, row=0).
    let origin = Vec2::new(
        layer.origin_x + TILE_SIZE * 0.5,
        layer.origin_y + TILE_SIZE * 0.5,
    );
    let tile_tint = match current_level.level_id {
        Some(lid) => crate::level::tile_tint_for_layer(lid),
        None => None,
    };
    if let Some(tint) = tile_tint {
        spawn_tilemap_tinted(&mut commands, &asset_server, solid_model, platform_model, &layer.tiles, origin, 0.0, tint);
    } else {
        spawn_tilemap(&mut commands, &asset_server, solid_model, platform_model, &layer.tiles, origin, 0.0);
    }

    // Solar panel canopy on Subdivision Rooftop layer only.
    if current_level.level_id == Some(LevelId::Subdivision) && current_level.layer_index == 2 {
        crate::level::spawn_solar_panel_canopy(&mut commands, &mut meshes, &mut materials);
    }

    // Sublevel setup: dark background, decorations, return door.
    // All carry TileEntity so they auto-despawn on layer switch.
    // Uses the layer's origin so positions work regardless of where the grid is.
    info!(
        "[SUBLEVEL] layer_index={} origin=({}, {}) spawn=({}, {}) grid={}x{}",
        current_level.layer_index,
        layer.origin_x, layer.origin_y,
        layer.spawn.x, layer.spawn.y,
        layer.cols(), layer.rows(),
    );
    if current_level.layer_index == 1 {
        let ox = layer.origin_x;
        let oy = layer.origin_y;
        // Center of 32×18 grid: (ox + 16*18, oy + 9*18)
        let center_x = ox + 16.0 * TILE_SIZE;
        let center_y = oy + 9.0 * TILE_SIZE;
        info!(
            "[SUBLEVEL] entering sublevel: ox={ox} oy={oy} center=({center_x}, {center_y})"
        );

        // Dark background at z=-5: in front of ALL parallax backgrounds
        // (mountains at z=-50, clouds z=-60, sky z=-100) but behind tiles (z=0).
        let bg_color = match current_level.level_id {
            Some(LevelId::Forest)      => Color::srgb(0.12, 0.10, 0.07),
            Some(LevelId::Subdivision) => Color::srgb(0.08, 0.10, 0.07),
            Some(LevelId::City)        => Color::srgb(0.10, 0.10, 0.15),
            _                          => Color::srgb(0.05, 0.05, 0.05),
        };
        let bg_mesh = meshes.add(Rectangle::new(2000.0, 1000.0));
        let bg_mat = materials.add(StandardMaterial {
            base_color: bg_color,
            unlit: true,
            alpha_mode: AlphaMode::Opaque,
            ..default()
        });
        commands.spawn((
            Mesh3d(bg_mesh),
            MeshMaterial3d(bg_mat),
            Transform::from_xyz(center_x, center_y, -5.0),
            TileEntity,
        ));

        // Themed decorations
        if let Some(lid) = current_level.level_id {
            crate::level::spawn_sublevel_decorations(&mut commands, &asset_server, lid, ox, oy);
        }

        // Return door at col 28, ground level
        let door_x = ox + 28.0 * TILE_SIZE + TILE_SIZE * 0.5;
        let door_y = oy + 2.0 * TILE_SIZE;
        commands.spawn((
            SceneRoot(asset_server.load("models/door-rotate.glb#Scene0")),
            Transform::from_xyz(door_x, door_y, 1.0)
                .with_scale(Vec3::new(60.0, 54.0, 7.0)),
            super::doors::TransitionDoor { target_layer: 0 },
            TileEntity,
        ));
    }

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
