use bevy::prelude::*;

use super::level_data::LevelId;

/// Marks a door entity that transitions to a specific layer.
#[derive(Component)]
pub struct TransitionDoor {
    pub target_layer: usize,
}

/// Spawns door models at layer transition points for the initial (Forest) level at startup.
///
/// Forest: origin_x = -864.0, ground_top = -146.0
///   Door to underground (layer 1): col_x(28) = -864 + 28*18 + 9 = -351.0
///   Door to treetop  (layer 2): col_x(72) = -864 + 72*18 + 9 =  441.0
pub fn spawn_transition_doors(mut commands: Commands, asset_server: Res<AssetServer>) {
    spawn_doors_for_level(&mut commands, &asset_server, LevelId::Forest);
}

/// Spawns the two layer-transition doors appropriate for a given level.
/// Call this whenever a level is loaded (new game or level transition).
///
/// Door 1 (→ underground layer 1): at ground level.
/// Door 2 (→ upper layer 2): on the highest platform (row 14).
pub fn spawn_doors_for_level(
    commands: &mut Commands,
    asset_server: &AssetServer,
    level_id: LevelId,
) {
    // origin_y = -200, TILE_SIZE = 18
    let ground_top: f32 = -146.0; // origin_y + 3 * 18
    // Row 14 top surface: origin_y + 15 * 18 = 70.0
    let row14_top: f32 = 70.0;

    // col_x(col) = origin_x + col*18 + 9, origin_x = -864
    let (x_underground, x_upper, upper_y) = match level_id {
        LevelId::Forest => (
            -351.0_f32, // col 28 — ground-level door to underground
            45.0_f32,   // col 50 — on row 14 platform (48-52)
            row14_top,
        ),
        LevelId::Subdivision => (
            -351.0_f32, // col 28 — ground-level door to sewers
            351.0_f32,  // col 67 — on row 14 platform (65-69)
            row14_top,
        ),
        LevelId::City => (
            -351.0_f32, // col 28 — ground-level door to subway
            81.0_f32,   // col 52 — on row 30 platform (50-55)
            // row 30 top = origin_y + 31*18 = -200 + 558 = 358.0
            358.0_f32,
        ),
    };

    // Door to underground — at ground level.
    commands.spawn((
        SceneRoot(asset_server.load("models/door-rotate.glb#Scene0")),
        Transform::from_xyz(x_underground, ground_top, 1.0)
            .with_scale(Vec3::new(60.0, 54.0, 7.0)),
        TransitionDoor { target_layer: 1 },
    ));

    // Door to upper layer — on the highest platform (row 14 for Forest/Subdivision, row 30 for City).
    commands.spawn((
        SceneRoot(asset_server.load("models/door-rotate.glb#Scene0")),
        Transform::from_xyz(x_upper, upper_y, 1.0)
            .with_scale(Vec3::new(60.0, 54.0, 7.0)),
        TransitionDoor { target_layer: 2 },
    ));
}
