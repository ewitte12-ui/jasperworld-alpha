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
pub fn spawn_doors_for_level(
    commands: &mut Commands,
    asset_server: &AssetServer,
    level_id: LevelId,
) {
    let ground_top: f32 = -146.0;

    let (x1, x2) = match level_id {
        // col_x(col) = origin_x + col*18 + 9
        LevelId::Forest => {
            // origin_x = -864.0
            // Door → layer 1: col 28 → -864 + 504 + 9 = -351.0
            // Door → layer 2: col 72 → -864 + 1296 + 9 = 441.0
            (-351.0_f32, 441.0_f32)
        }
        LevelId::Subdivision => {
            // origin_x = -864.0
            // Door → layer 1: col 27 → -864 + 486 + 9 = -369.0  (under platform D, row8)
            // Door → layer 2: col 46 → -864 + 828 + 9 =  -27.0  (under platform F_mid, row6)
            (-369.0_f32, -27.0_f32)
        }
        LevelId::City => {
            // origin_x = -1152.0
            // Door → layer 1: col 28 → -1152 + 504 + 9 = -639.0
            // Door → layer 2: col 55 → -1152 + 990 + 9 = -153.0
            (-639.0_f32, -153.0_f32)
        }
        LevelId::Sanctuary => {
            // origin_x = -1440.0
            // Door → layer 1: col 27 → -1440 + 486 + 9 = -945.0
            // Door → layer 2: col 60 → -1440 + 1080 + 9 = -351.0
            (-945.0_f32, -351.0_f32)
        }
    };

    commands.spawn((
        SceneRoot(asset_server.load("models/door-rotate.glb#Scene0")),
        Transform::from_xyz(x1, ground_top, 1.0)
            .with_scale(Vec3::new(60.0, 54.0, 7.0)),
        TransitionDoor { target_layer: 1 },
    ));

    commands.spawn((
        SceneRoot(asset_server.load("models/door-rotate.glb#Scene0")),
        Transform::from_xyz(x2, ground_top, 1.0)
            .with_scale(Vec3::new(60.0, 54.0, 7.0)),
        TransitionDoor { target_layer: 2 },
    ));
}
