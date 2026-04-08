pub mod city;
pub mod compiled_data;
pub mod components;
pub mod doors;
pub mod forest;
pub mod level_data;
pub mod subdivision;
pub mod systems;
pub mod test_level;

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::physics::config::GameLayer;

use crate::collectibles::components::{Collectible, CollectibleType, CollectionProgress};
use crate::collectibles::systems::spawn_collectible;
use crate::combat::components::Health;
use crate::enemies::components::{Enemy, EnemyType};
use crate::enemies::spawner::spawn_enemy;
use crate::player::components::Player;
use crate::puzzle::components::{GameProgress, LevelExit, LevelGate};
use crate::rendering::parallax::{
    spawn_city_background, spawn_nature_background, spawn_shared_background,
    spawn_subdivision_background,
};
use crate::states::NewGameRequested;
use crate::tilemap::spawn::{spawn_tilemap, spawn_tilemap_tinted};
use crate::tilemap::tilemap::TILE_SIZE;
use city::city_level;
use doors::TransitionDoor;
use forest::forest_level;
use level_data::{CurrentLevel, LevelId};
use subdivision::subdivision_level;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CurrentLevel::default())
            .add_systems(
                Startup,
                // Only non-gameplay infrastructure runs at Startup so the world is
                // empty during TitleScreen. All tile/enemy/collectible/door spawning
                // is deferred to handle_new_game (OnEnter Playing).
                (load_forest_level, spawn_floor_boundary).chain(),
            )
            .add_systems(OnEnter(crate::states::AppState::Playing), handle_new_game)
            .add_systems(
                Update,
                (
                    systems::switch_layer
                        .in_set(crate::puzzle::components::TransitionSet)
                        .after(crate::puzzle::systems::check_level_exit),
                    systems::camera_clamp.in_set(crate::rendering::camera::CameraPipeline::Clamp),
                )
                    .run_if(in_state(crate::states::AppState::Playing)),
            );
    }
}

fn load_forest_level(mut commands: Commands) {
    commands.insert_resource(forest_level());
}

/// Spawns an invisible static floor collider below the level to catch falling players/enemies.
fn spawn_floor_boundary(mut commands: Commands) {
    commands.spawn((
        Transform::from_xyz(0.0, -220.0, 0.0),
        RigidBody::Static,
        Collider::rectangle(4000.0, 10.0),
        CollisionLayers::new(GameLayer::Ground, [GameLayer::Player, GameLayer::Enemy]),
    ));
}

// ── Coordinate helpers ────────────────────────────────────────────────────────
// TILE_SIZE = 18.0
// col_x(col, origin_x)   = origin_x + col * 18 + 9
// stand_y(row, origin_y) = origin_y + (row+1) * 18 + 9
// ground_y               = stand_y(2, origin_y)
// ground_top             = origin_y + 3 * 18   (top surface of row-2 ground)
// gate_center_y          = ground_top + 200     (makes gate 400 units tall)

// ── Forest (origin_x = -864, origin_y = -200) ────────────────────────────────
// col_x(col) = -864 + col*18 + 9
// stand_y(row) = -200 + (row+1)*18 + 9
// ground_y = -137, ground_top = -146, spawn = (-819, -128)
// Gate: col_x(91) = 783.0  |  Exit: (813.0, -137.0)

/// Dispatches to the correct per-level entity spawn function.
pub fn spawn_entities_for_level(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    asset_server: &AssetServer,
    progress: &mut CollectionProgress,
    level_id: LevelId,
    skip_enemies: bool,
) {
    match level_id {
        LevelId::Forest => {
            spawn_forest_inner(
                commands,
                meshes,
                materials,
                asset_server,
                progress,
                skip_enemies,
            );
        }
        LevelId::Subdivision => {
            spawn_subdivision_inner(
                commands,
                meshes,
                materials,
                asset_server,
                progress,
                skip_enemies,
            );
        }
        LevelId::City => {
            spawn_city_inner(
                commands,
                meshes,
                materials,
                asset_server,
                progress,
                skip_enemies,
            );
        }
    }
}

/// Inner logic of spawn_forest_entities, callable as a free function (no Bevy system params).
fn spawn_forest_inner(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    asset_server: &AssetServer,
    progress: &mut CollectionProgress,
    skip_enemies: bool,
) {
    const OX: f32 = -864.0;
    const OY: f32 = -200.0;
    let col_x = |col: f32| OX + col * 18.0 + 9.0;
    let stand_y = |row: f32| OY + (row + 1.0) * 18.0 + 9.0;
    let ground_y = stand_y(2.0);
    let ground_top = OY + 3.0 * 18.0; // -146.0

    let star_positions = [
        Vec3::new(col_x(7.0), ground_y, 1.0),       // ground — Screen 1
        Vec3::new(col_x(20.0), ground_y, 1.0),      // ground — Screen 1
        Vec3::new(col_x(5.0), stand_y(6.0), 1.0),   // Platform A (row 6)
        Vec3::new(col_x(15.0), stand_y(10.0), 1.0), // Platform C (row 10)
        Vec3::new(col_x(37.0), stand_y(6.0), 1.0),  // Platform D (row 6)
        Vec3::new(col_x(46.0), stand_y(10.0), 1.0), // Platform E (row 10)
        // WHY row 14 bonus: this is the optional micro-objective star.
        // Gate opens at 10 of 11 — the player can skip this star and still
        // finish the level.  From Platform E (row 10, cols 44-49) the row 14
        // platform (cols 48-52) is visible one jump above, making the detour
        // a genuine choice rather than a required stop.  Taking the detour
        // teaches the scan-then-commit pattern at maximum height; skipping it
        // keeps the route on the standard ground→row6→row10 path.
        Vec3::new(col_x(50.0), stand_y(14.0), 1.0), // Row 14 — optional micro-objective
        Vec3::new(col_x(58.0), ground_y, 1.0),      // ground — Screen 2
        Vec3::new(col_x(69.0), stand_y(6.0), 1.0),  // Platform H (row 6)
        Vec3::new(col_x(77.0), stand_y(10.0), 1.0), // Platform I (row 10)
        Vec3::new(col_x(85.0), ground_y, 1.0),      // ground — Screen 3
    ];
    for pos in star_positions {
        spawn_collectible(
            commands,
            meshes,
            materials,
            asset_server,
            pos,
            CollectibleType::Star,
            false,
        );
    }
    // WHY stars_total = 10 with 11 spawned: the row 14 star (index 6) is the
    // optional micro-objective.  Gate opens when 10 are collected so the
    // player can reach the exit without the vertical detour.  Collecting all
    // 11 is possible and rewards curiosity without changing the gate rule.
    progress.stars_total = 10;
    progress.stars_collected = 0;

    let apple_positions = [
        Vec3::new(col_x(3.0), ground_y, 1.0),
        Vec3::new(col_x(25.0), stand_y(6.0), 1.0),
        // WHY two apples in the Dog zone section (cols 52 and 57):
        // These form the risk-vs-safety choice required by the Level Engagement
        // Guardrail.  From Platform E (row 10, cols 44-49) the player looks
        // right-downward and sees BOTH the Dog patrol and the ground apple at
        // col 52 before making any commitment.
        //
        // Safe route  — jump Platform E → Platform F (row 6, elevated above Dog
        //               reach) → collect apple at col 57.  One apple, zero Dog
        //               contact risk.
        //
        // Risky route — drop to ground at col 52, collect apple there, sprint
        //               3 cols (54 units) right and jump up to Platform F while
        //               Dog is in chase.  Two apples if executed cleanly, but
        //               contact damage is possible.
        //
        // The apple at col 52 is the ONLY reward the safe route cannot reach
        // without entering the Dog zone.  It restores health that changes how
        // cautiously the player must play Screen 3 (Snake + Possum encounters).
        Vec3::new(col_x(52.0), ground_y, 1.0), // Dog zone — risky route reward
        Vec3::new(col_x(57.0), stand_y(6.0), 1.0), // Platform F — safe route reward
        Vec3::new(col_x(79.0), stand_y(10.0), 1.0),
    ];
    for pos in apple_positions {
        spawn_collectible(
            commands,
            meshes,
            materials,
            asset_server,
            pos,
            CollectibleType::HealthFood,
            false,
        );
    }

    let gate_x = col_x(91.0);
    let gate_center_y = OY + 3.0 * 18.0 + 200.0;
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
                Transform::from_xyz(0.0, -200.0, 0.0).with_scale(Vec3::new(18.0, 80.0, 7.0)),
            ));
        });

    commands.spawn((
        Transform::from_xyz(gate_x + 30.0, ground_y, 0.5),
        Visibility::Hidden,
        LevelExit {
            next_level: LevelId::Subdivision,
            half_extents: Vec2::new(51.0, 100.0),
        },
    ));

    // End-zone landmark — open door as forest exit cue.
    commands.spawn((
        SceneRoot(asset_server.load("models/door-open.glb#Scene0")),
        Transform::from_xyz(gate_x + 40.0, ground_top, -1.0).with_scale(Vec3::new(60.0, 54.0, 7.0)),
        components::Decoration,
    ));

    if !skip_enemies {
        let enemies = [
            // Dog envelope: spawn at col 47 (x=-9), patrol_range = 108 (6 × TILE_SIZE).
            // Zone boundaries: Platform D right edge (col 39, x=-153) to
            //                  Platform F left edge (col 55, x=135) = 288 units wide.
            // Left patrol bound:  -9 - 108 = -117  (1.5 tiles inside Platform D — buffer preserved)
            // Right patrol bound: -9 + 108 =   99  (2 tiles clear of Platform F — Dog never goes under)
            // Ground coverage: 216 / 288 = 75% — forces the player to time a crossing or take the
            // elevated Platform E → Platform F route rather than sprinting past freely.
            // WHY not spawn at col 52 (x=81): col 52 is 3 tiles left of Platform F.  Even a moderate
            // range from there makes the right bound enter Platform F territory; shifting spawn to the
            // zone midpoint (col 47) gives symmetric reach with 2-tile clearance on both sides.
            (
                EnemyType::Dog,
                Vec2::new(col_x(47.0), ground_top),
                72.0_f32,
                150.0,
            ), // 3 stomps to kill
            (
                EnemyType::Snake,
                Vec2::new(col_x(74.0), ground_top),
                54.0_f32,
                50.0,
            ),
            (
                EnemyType::Possum,
                Vec2::new(col_x(82.0), ground_top),
                54.0_f32,
                50.0,
            ),
        ];
        for (etype, pos, patrol, hp) in enemies {
            spawn_enemy(
                commands,
                meshes,
                materials,
                asset_server,
                etype,
                pos,
                patrol,
                hp,
                None,
            );
        }
    } // skip_enemies
}

/// Subdivision level entity spawning — stars, apples, enemies, gate, exit.
fn spawn_subdivision_inner(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    asset_server: &AssetServer,
    progress: &mut CollectionProgress,
    skip_enemies: bool,
) {
    const OX: f32 = -864.0;
    const OY: f32 = -200.0;
    let col_x = |col: f32| OX + col * 18.0 + 9.0;
    let stand_y = |row: f32| OY + (row + 1.0) * 18.0 + 9.0;
    let ground_y = stand_y(2.0);
    let ground_top = OY + 3.0 * 18.0; // -146.0

    // Stars (11 spawned, 10 required — same rule as Forest)
    let star_positions = [
        Vec3::new(col_x(8.0), ground_y, 1.0),
        Vec3::new(col_x(22.0), ground_y, 1.0),
        Vec3::new(col_x(6.0), stand_y(6.0), 1.0),
        Vec3::new(col_x(16.0), stand_y(10.0), 1.0),
        Vec3::new(col_x(40.0), stand_y(6.0), 1.0),
        Vec3::new(col_x(48.0), stand_y(10.0), 1.0),
        Vec3::new(col_x(32.0), stand_y(14.0), 1.0), // optional high platform
        Vec3::new(col_x(60.0), ground_y, 1.0),
        Vec3::new(col_x(70.0), stand_y(6.0), 1.0),
        Vec3::new(col_x(78.0), stand_y(10.0), 1.0),
        Vec3::new(col_x(87.0), ground_y, 1.0),
    ];
    for pos in star_positions {
        spawn_collectible(
            commands,
            meshes,
            materials,
            asset_server,
            pos,
            CollectibleType::Star,
            false,
        );
    }
    progress.stars_total = 10;
    progress.stars_collected = 0;

    // Apples (5)
    let apple_positions = [
        Vec3::new(col_x(4.0), ground_y, 1.0),
        Vec3::new(col_x(26.0), stand_y(6.0), 1.0),
        Vec3::new(col_x(53.0), ground_y, 1.0), // Dog zone risk reward
        Vec3::new(col_x(58.0), stand_y(6.0), 1.0),
        Vec3::new(col_x(80.0), stand_y(10.0), 1.0),
    ];
    for pos in apple_positions {
        spawn_collectible(
            commands,
            meshes,
            materials,
            asset_server,
            pos,
            CollectibleType::HealthFood,
            false,
        );
    }

    // Gate at col 91
    let gate_x = col_x(91.0);
    let gate_center_y = OY + 3.0 * 18.0 + 200.0;
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
                Transform::from_xyz(0.0, -200.0, 0.0).with_scale(Vec3::new(18.0, 80.0, 7.0)),
            ));
        });

    // Exit — transitions to City.
    commands.spawn((
        Transform::from_xyz(gate_x + 30.0, ground_y, 0.5),
        Visibility::Hidden,
        LevelExit {
            next_level: LevelId::City,
            half_extents: Vec2::new(51.0, 100.0),
        },
    ));

    // End-zone landmark
    commands.spawn((
        SceneRoot(asset_server.load("models/door-open.glb#Scene0")),
        Transform::from_xyz(gate_x + 40.0, ground_top, -1.0).with_scale(Vec3::new(60.0, 54.0, 7.0)),
        components::Decoration,
    ));

    if !skip_enemies {
        // Dog: wider patrol range (108 vs Forest's 72) for harder encounter
        let enemies = [
            (
                EnemyType::Dog,
                Vec2::new(col_x(50.0), ground_top),
                108.0_f32,
                250.0,
            ), // 5 stomps to kill
            (
                EnemyType::Snake,
                Vec2::new(col_x(75.0), ground_top),
                54.0_f32,
                50.0,
            ),
            (
                EnemyType::Possum,
                Vec2::new(col_x(84.0), ground_top),
                54.0_f32,
                50.0,
            ),
        ];
        for (etype, pos, patrol, hp) in enemies {
            spawn_enemy(
                commands,
                meshes,
                materials,
                asset_server,
                etype,
                pos,
                patrol,
                hp,
                None,
            );
        }
    } // skip_enemies
}

/// Inner logic for City level entity spawning.
fn spawn_city_inner(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    asset_server: &AssetServer,
    progress: &mut CollectionProgress,
    skip_enemies: bool,
) {
    const OX: f32 = -864.0;
    const OY: f32 = -200.0;
    let col_x = |col: f32| OX + col * 18.0 + 9.0;
    let stand_y = |row: f32| OY + (row + 1.0) * 18.0 + 9.0;
    let ground_y = stand_y(2.0);
    let ground_top = OY + 3.0 * 18.0; // -146.0

    // 11 stars, 10 required — same rule as Forest/Subdivision.
    // Distributed across multiple heights to exploit the 44-row space.
    let star_positions = [
        Vec3::new(col_x(8.0), ground_y, 1.0),       // ground — early
        Vec3::new(col_x(22.0), stand_y(6.0), 1.0),  // low platform
        Vec3::new(col_x(35.0), stand_y(10.0), 1.0), // mid platform
        Vec3::new(col_x(45.0), stand_y(14.0), 1.0), // high platform
        Vec3::new(col_x(55.0), ground_y, 1.0),      // ground — midlevel
        Vec3::new(col_x(63.0), stand_y(8.0), 1.0),  // mid-low platform
        Vec3::new(col_x(72.0), stand_y(18.0), 1.0), // very high
        Vec3::new(col_x(32.0), stand_y(26.0), 1.0), // near-top — optional micro-objective
        Vec3::new(col_x(52.0), stand_y(22.0), 1.0), // upper platform
        Vec3::new(col_x(80.0), ground_y, 1.0),      // ground — late
        Vec3::new(col_x(87.0), ground_y, 1.0),      // ground — near exit
    ];

    progress.stars_total = 10;
    progress.stars_collected = 0;

    for pos in &star_positions {
        spawn_collectible(
            commands,
            meshes,
            materials,
            asset_server,
            *pos,
            CollectibleType::Star,
            true,
        );
    }

    // 5 apples — mix of ground and platform placements.
    let apple_positions = [
        Vec3::new(col_x(15.0), ground_y, 1.0),
        Vec3::new(col_x(30.0), stand_y(4.0), 1.0),
        Vec3::new(col_x(50.0), stand_y(10.0), 1.0),
        Vec3::new(col_x(70.0), ground_y, 1.0),
        Vec3::new(col_x(85.0), stand_y(6.0), 1.0),
    ];
    for pos in &apple_positions {
        spawn_collectible(
            commands,
            meshes,
            materials,
            asset_server,
            *pos,
            CollectibleType::HealthFood,
            true,
        );
    }

    // Gate at col 91 (same position as other levels).
    let gate_x = col_x(91.0);
    let gate_center_y = ground_top + 200.0;
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
                Transform::from_xyz(0.0, -200.0, 0.0).with_scale(Vec3::new(18.0, 80.0, 7.0)),
            ));
        });

    // Exit — game_complete fires at level_index >= 3.
    commands.spawn((
        Transform::from_xyz(gate_x + 30.0, ground_y, 0.5),
        Visibility::Hidden,
        LevelExit {
            next_level: LevelId::City,
            half_extents: Vec2::new(51.0, 100.0),
        },
    ));

    // End-zone landmark
    commands.spawn((
        SceneRoot(asset_server.load("models/door-open.glb#Scene0")),
        Transform::from_xyz(gate_x + 40.0, ground_top, -1.0).with_scale(Vec3::new(60.0, 54.0, 7.0)),
        components::Decoration,
    ));

    if !skip_enemies {
        // City enemies — more enemies, harder Dog, all non-Dog take 2 stomps.
        // Dog: 25% faster (150 vs 120), 10 stomps (500 HP), wider patrol.
        // Snake/Possum/Rat: 100 HP (2 stomps each).
        let enemies: &[(EnemyType, Vec2, f32, f32, Option<f32>)] = &[
            (
                EnemyType::Dog,
                Vec2::new(col_x(50.0), ground_top),
                144.0,
                500.0,
                Some(150.0),
            ),
            (
                EnemyType::Snake,
                Vec2::new(col_x(25.0), ground_top),
                54.0,
                100.0,
                None,
            ),
            (
                EnemyType::Snake,
                Vec2::new(col_x(75.0), ground_top),
                54.0,
                100.0,
                None,
            ),
            (
                EnemyType::Possum,
                Vec2::new(col_x(40.0), ground_top),
                54.0,
                100.0,
                None,
            ),
            (
                EnemyType::Possum,
                Vec2::new(col_x(84.0), ground_top),
                54.0,
                100.0,
                None,
            ),
            (
                EnemyType::Rat,
                Vec2::new(col_x(60.0), ground_top),
                72.0,
                100.0,
                None,
            ),
            (
                EnemyType::Rat,
                Vec2::new(col_x(88.0), ground_top),
                72.0,
                100.0,
                None,
            ),
        ];
        for &(etype, pos, patrol, hp, spd) in enemies {
            spawn_enemy(
                commands,
                meshes,
                materials,
                asset_server,
                etype,
                pos,
                patrol,
                hp,
                spd,
            );
        }
    } // skip_enemies
}

/// Spawns ALL background and decoration entities for `level_id`.
///
/// Called on every level entry (new game and level transitions).
/// Every entity spawned here carries `Decoration` so it is fully despawned on
/// level exit before this function is called again for the next level.
///
/// jasper_background_parallax_lifecycle_guardrail: this is the SINGLE AUTHORITY
/// for all biome-specific background art. Nothing is spawned in Startup.
pub fn spawn_level_decorations(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    asset_server: &AssetServer,
    level_id: LevelId,
) {
    spawn_shared_background(commands, meshes, materials, asset_server, level_id);

    match level_id {
        LevelId::Forest => {
            // Nature tree background.
            spawn_nature_background(commands, asset_server);
            // Ground-level Forest props (rocks, flowers, bushes).
            let ox = -864.0_f32;
            let col_x_f = |col: f32| ox + col * 18.0 + 9.0;
            // (model, x, y, scale_xy, scale_z)
            // z=-1: props sit just behind the tile surface, immediately visible.
            // Non-uniform scale: thin Z breaks the boxy look of GLB props placed flat.
            // Scale reference: 1J = Jasper sprite height (32 world units), halved.
            // sxy = target_J × 32.0 / native_model_Y / 2;  sz keeps original ratio.
            // All items placed on ground (y=-146) or valid platforms only.
            // Bottom-anchored Y = ground_top formula: OY + (row+1)*18
            //   Ground (row 2 top): -146.0
            //   Row 6 platforms:    -74.0
            //   Row 10 platforms:   -2.0
            //   Row 14 platforms:   70.0
            // Center-anchored rocks add half-height offset above these values.
            let decor: &[(&str, f32, f32, f32, f32)] = &[
                // ── y = -146 ground ─────────────────────────────────────────
                ("models/small_rock.glb", col_x_f(3.0), -142.8, 13.0, 5.0),
                ("models/grass_large.glb", col_x_f(7.0), -146.0, 38.0, 12.0),
                ("models/plant_bush.glb", col_x_f(9.0), -146.0, 49.0, 27.0),
                ("models/large_rock.glb", col_x_f(10.0), -134.0, 28.0, 9.0),
                (
                    "models/plant_bushLarge.glb",
                    col_x_f(15.0),
                    -146.0,
                    79.0,
                    43.0,
                ),
                ("models/flower_redA.glb", col_x_f(18.0), -146.0, 66.0, 17.0),
                ("models/yellow_flower.glb", col_x_f(22.0), -136.0, 20.0, 5.0),
                ("models/large_rock.glb", col_x_f(30.0), -134.0, 28.0, 9.0),
                ("models/small_rock.glb", col_x_f(35.0), -142.8, 13.0, 5.0),
                (
                    "models/plant_bushLarge.glb",
                    col_x_f(43.0),
                    -146.0,
                    79.0,
                    43.0,
                ),
                ("models/large_rock.glb", col_x_f(55.0), -134.0, 28.0, 9.0),
                ("models/yellow_flower.glb", col_x_f(58.0), -136.0, 20.0, 5.0),
                ("models/small_rock.glb", col_x_f(85.0), -142.8, 13.0, 5.0),
                // ── y = -74 row 6 platforms ──────────────────────────────────
                ("models/yellow_flower.glb", col_x_f(5.0), -64.0, 20.0, 5.0),
                ("models/small_rock.glb", col_x_f(8.0), -70.8, 13.0, 5.0),
                ("models/small_rock.glb", col_x_f(23.0), -70.8, 13.0, 5.0),
                ("models/plant_bush.glb", col_x_f(24.0), -74.0, 49.0, 27.0),
                ("models/flower_redA.glb", col_x_f(25.0), -74.0, 66.0, 17.0),
                ("models/grass_large.glb", col_x_f(36.0), -74.0, 38.0, 12.0),
                ("models/plant_bush.glb", col_x_f(37.0), -74.0, 49.0, 27.0),
                ("models/flower_redA.glb", col_x_f(57.0), -74.0, 66.0, 17.0),
                ("models/small_rock.glb", col_x_f(84.0), -70.8, 13.0, 5.0),
                // ── y = -2 row 10 platforms ──────────────────────────────────
                ("models/small_rock.glb", col_x_f(14.0), 1.2, 13.0, 5.0),
                ("models/grass_large.glb", col_x_f(16.0), -2.0, 38.0, 12.0),
                ("models/flower_redA.glb", col_x_f(17.0), -2.0, 66.0, 17.0),
                ("models/small_rock.glb", col_x_f(45.0), 1.2, 13.0, 5.0),
                ("models/yellow_flower.glb", col_x_f(46.0), 8.0, 20.0, 5.0),
                ("models/grass_large.glb", col_x_f(76.0), -2.0, 38.0, 12.0),
                ("models/small_rock.glb", col_x_f(78.0), 1.2, 13.0, 5.0),
                ("models/plant_bushLarge.glb", col_x_f(80.0), -2.0, 79.0, 43.0),
                // ── y = 70 row 14 platforms ──────────────────────────────────
                (
                    "models/plant_bushLarge.glb",
                    col_x_f(49.0),
                    70.0,
                    79.0,
                    43.0,
                ),
                ("models/small_rock.glb", col_x_f(52.0), 73.2, 13.0, 5.0),
                ("models/plant_bush.glb", col_x_f(68.0), 70.0, 49.0, 27.0),
                ("models/flower_redA.glb", col_x_f(69.0), 70.0, 66.0, 17.0),
                ("models/grass_large.glb", col_x_f(70.0), 70.0, 38.0, 12.0),
                ("models/plant_bush.glb", col_x_f(71.0), 70.0, 49.0, 27.0),
            ];
            // Tripo rock models face +X by default; rotate -90° Y so front faces camera.
            // After rotation local Z→world X, local X→world Z, so swap X/Z scales.
            let rock_rot = Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2);
            for &(model, x, y, sxy, sz) in decor {
                let is_tripo = model.contains("large_rock") || model.contains("small_rock") || model.contains("yellow_flower");
                let xform = if is_tripo {
                    Transform::from_xyz(x, y, -15.0)
                        .with_rotation(rock_rot)
                        .with_scale(Vec3::new(sz, sxy, sxy))
                } else {
                    Transform::from_xyz(x, y, -15.0)
                        .with_scale(Vec3::new(sxy, sxy, sz))
                };
                commands.spawn((
                    SceneRoot(asset_server.load(format!("{}#Scene0", model))),
                    // z=-15: pushed behind Jasper's z=5 plane so the 3D volume of
                    // these props does not protrude into Jasper's depth layer.
                    xform,
                    components::Decoration,
                    components::ForegroundDecoration,
                ));
            }

            // Foreground framing trees (z = +10) — bookend the left/right level edges.
            // Must live here (not Startup) per jasper_background_parallax_lifecycle_guardrail:
            // biome-specific art is level content, not engine setup.
            // WHY left trees at -525/-440: door 1 is at x=-351 (footprint ≈ -381 to -321).
            // oak half-width = 95*0.4 = 38; fat half-width = 80*0.46 = 36.8.
            // -525 oak right edge ≈ -487; -440 fat left edge ≈ -477 → 10-unit gap between them.
            // -440 fat right edge ≈ -403; door left edge = -381 → 22-unit clearance from door.
            // WHY right trees at 240/330: pine half-width = 90*0.31 = 27.9; oak half-width = 85*0.4 = 34.
            // pine right edge ≈ 268; oak left edge ≈ 296 → 28-unit gap between them.
            let fg_trees: &[(&str, f32, f32, f32)] = &[
                // center-anchored model: y raised by scale/2 to ground the base
                ("models/tree_oak.glb", -525.0, -98.5, 95.0), // moved left; gap from fat = 10 units
                ("models/tree_fat.glb", -440.0, -106.0, 80.0), // moved left; gap from door 1 = 22 units
                ("models/tree_pine.glb", 240.0, -101.0, 90.0), // moved left from 270
                ("models/tree_oak.glb", 330.0, -103.5, 85.0), // moved right from 295; gap from pine = 28 units
            ];
            for &(model, tx, ty, scale) in fg_trees {
                commands.spawn((
                    SceneRoot(asset_server.load(format!("{}#Scene0", model))),
                    Transform::from_xyz(tx, ty, 10.0).with_scale(Vec3::new(scale, scale, 1.0)),
                    components::Decoration,
                    components::ForegroundDecoration,
                ));
            }
        }
        LevelId::Subdivision => {
            spawn_subdivision_background(commands, asset_server);

            // Overcast sky overlay — grey-blue rectangle at z=-99, just in front of the
            // blue sky backdrop at z=-100. Carries Decoration so it despawns on level exit,
            // restoring the blue sky for other levels.
            let overcast_mesh = meshes.add(Rectangle::new(6400.0, 1800.0));
            let overcast_mat = materials.add(StandardMaterial {
                base_color: Color::srgb(0.55, 0.58, 0.65),
                unlit: true,
                alpha_mode: AlphaMode::Opaque,
                ..default()
            });
            commands.spawn((
                Mesh3d(overcast_mesh),
                MeshMaterial3d(overcast_mat),
                Transform::from_xyz(0.0, 0.0, -99.0),
                crate::rendering::parallax::ParallaxLayer { factor: 0.20 },
                components::Decoration,
                crate::rendering::parallax::ParallaxBackground,
            ));

            // Foreground framing — suburban trees at level edges (z=+10)
            let fg_trees: &[(&str, f32, f32, f32)] = &[
                (
                    "models/suburban/tree-suburban-large.glb",
                    -450.0,
                    -146.0,
                    180.0,
                ),
                (
                    "models/suburban/tree-suburban-small.glb",
                    -420.0,
                    -146.0,
                    140.0,
                ),
                (
                    "models/suburban/tree-suburban-large.glb",
                    270.0,
                    -146.0,
                    170.0,
                ),
                (
                    "models/suburban/tree-suburban-small.glb",
                    295.0,
                    -146.0,
                    150.0,
                ),
            ];
            for &(model, tx, ty, scale) in fg_trees {
                commands.spawn((
                    SceneRoot(asset_server.load(format!("{}#Scene0", model))),
                    Transform::from_xyz(tx, ty, 10.0).with_scale(Vec3::new(scale, scale, 1.0)),
                    components::Decoration,
                    components::ForegroundDecoration,
                ));
            }
        }
        LevelId::City => {
            info!("[CITY] spawn_level_decorations: entering City arm");
            spawn_city_background(commands, asset_server);

            // Night sky overlay — dark navy rectangle at z=-99, in front of the
            // blue sky backdrop at z=-100. Hides daytime sky for night atmosphere.
            let night_mesh = meshes.add(Rectangle::new(6400.0, 1800.0));
            let night_mat = materials.add(StandardMaterial {
                base_color: Color::srgb(0.02, 0.02, 0.04),
                unlit: true,
                alpha_mode: AlphaMode::Opaque,
                ..default()
            });
            commands.spawn((
                Mesh3d(night_mesh),
                MeshMaterial3d(night_mat),
                Transform::from_xyz(0.0, 0.0, -99.0),
                crate::rendering::parallax::ParallaxLayer { factor: 0.20 },
                components::Decoration,
                crate::rendering::parallax::ParallaxBackground,
            ));

            // Decorative stars — 200 bright dots across the full night sky.
            // z=-98: just in front of the night overlay (-99) but BEHIND both
            // building layers (far=-80, near=-50) so buildings naturally occlude
            // stars — no manual position avoidance needed.
            // WHY 5×5 base: attenuation planes at z=-75 (50% near-black) and
            // z=-38 (12% dark) sit in front, dimming stars ~44%. Larger size +
            // full brightness compensates.
            let star_mesh = meshes.add(Rectangle::new(5.0, 5.0));
            for i in 0..200 {
                let fi = i as f32;
                // Golden-ratio-based scatter — covers x=-1700..1700, y=50..750.
                let sx = ((fi * 137.508).sin() * 1600.0).rem_euclid(3400.0) - 1700.0;
                let sy = ((fi * 251.317 + 3.0).sin() * 350.0).rem_euclid(700.0) + 50.0;
                let brightness = 0.92 + ((fi * 317.432).sin() * 0.5 + 0.5) * 0.08;
                // Slight size variation for natural feel (0.6× to 1.4×).
                let size_var = 0.6 + ((fi * 193.271).sin() * 0.5 + 0.5) * 0.8;
                let star_mat = materials.add(StandardMaterial {
                    base_color: Color::srgb(brightness, brightness, brightness * 0.95),
                    unlit: true,
                    alpha_mode: AlphaMode::Opaque,
                    ..default()
                });
                commands.spawn((
                    Mesh3d(star_mesh.clone()),
                    MeshMaterial3d(star_mat),
                    Transform::from_xyz(sx, sy, -98.0).with_scale(Vec3::splat(size_var)),
                    crate::rendering::parallax::ParallaxLayer { factor: 0.22 },
                    components::Decoration,
                    crate::rendering::parallax::ParallaxBackground,
                ));
            }

            info!("[CITY] spawned night sky + {} decorative stars", 200);

            // Ground-level city props (z=-15, behind Jasper's z=5 plane to avoid depth clipping)
            let ox = -864.0_f32;
            let col_x_f = |col: f32| ox + col * 18.0 + 9.0;
            // Taxis — 3 parked cars across the level. No Y rotation needed because the
            // new model's longest axis is already X, so the side profile is naturally
            // visible from the camera (which looks down -Z onto the XY plane).
            // Trellis model native dims: X=1.000, Y=0.4821, Z=0.4734 (center-anchored).
            // Uniform scale 90:
            //   Visible length (world X) = 1.0    × 90 = 90 units (~5.0 tiles).
            //   Visible height (world Y) = 0.4821 × 90 = 43.4 units (~2.4 tiles).
            //   Depth (world Z)          = 0.4734 × 90 = 42.6 units.
            // Y = -124.3: ground(-146) + half_height(0.4821*90/2 ≈ 21.7) ≈ -124.
            let taxi_positions = [col_x_f(15.0), col_x_f(50.0), col_x_f(80.0)];
            for tx in taxi_positions {
                commands.spawn((
                    SceneRoot(asset_server.load("models/city/taxi.glb#Scene0")),
                    Transform::from_xyz(tx, -124.3, -15.0).with_scale(Vec3::splat(90.0)),
                    components::Decoration,
                    components::ForegroundDecoration,
                ));
            }

            // Street lights — every ~250 units along the ground.
            // Scale 70: native H=0.675 → 0.675*70/18 = 2.6 Jasper units tall.
            for x in (-1200..=1200i32).step_by(250) {
                commands.spawn((
                    SceneRoot(asset_server.load("models/city/light-curved.glb#Scene0")),
                    Transform::from_xyz(x as f32, -141.0, -15.0)
                        .with_scale(Vec3::new(70.0, 70.0, 16.0)),
                    components::Decoration,
                    components::ForegroundDecoration,
                ));
            }

            // Construction cones — scattered urban clutter.
            // New model: X=0.703, Y=1.0, Z=0.679, center-anchored (Y: -0.5→0.5).
            // Target height = 0.7 Jasper units = 0.7*18 = 12.6 world units → scale = 12.6/1.0 = 12.6.
            // Y position: base at -141, center-anchored → Y = -141 + (1.0*12.6)/2 = -134.7.
            let cone_positions = [col_x_f(10.0), col_x_f(45.0), col_x_f(70.0)];
            for cx in cone_positions {
                commands.spawn((
                    SceneRoot(asset_server.load("models/city/construction-cone.glb#Scene0")),
                    Transform::from_xyz(cx, -134.7, -15.0).with_scale(Vec3::splat(12.6)),
                    components::Decoration,
                    components::ForegroundDecoration,
                ));
            }

            // Foreground trees — sparse, at level edges (it's a city).
            // WHY left fat tree at -500: door 1 is at x=-351 (footprint ≈ -381 to -321).
            // fat half-width = 80*0.46 = 36.8; right edge ≈ -463 → 82-unit clearance from door.
            // WHY oak/default pair at 230/340: oak half-width = 75*0.4 = 30; default half-width = 70*0.5 = 35.
            // oak right edge ≈ 260; default left edge ≈ 305 → 45-unit gap between them.
            let fg_trees: &[(&str, f32, f32, f32)] = &[
                ("models/tree_fat.glb", -500.0, -106.0, 80.0), // moved left; gap from door 1 = 82 units
                ("models/tree_oak.glb", 230.0, -108.5, 75.0),  // moved left from 270
                ("models/tree_default.glb", 340.0, -111.0, 70.0), // moved right from 295; gap from oak = 45 units; Y adjusted from -146.0 to -111.0 (+35 = scale/2) for center-anchored Trellis model
                ("models/tree_fat.glb", -700.0, -108.5, 75.0),    // unchanged
                ("models/tree_oak.glb", 550.0, -111.0, 70.0),     // unchanged
            ];
            for &(model, tx, ty, scale) in fg_trees {
                commands.spawn((
                    SceneRoot(asset_server.load(format!("{}#Scene0", model))),
                    Transform::from_xyz(tx, ty, 10.0).with_scale(Vec3::new(scale, scale, 1.0)),
                    components::Decoration,
                    components::ForegroundDecoration,
                ));
            }
            info!(
                "[CITY] spawned 3 taxis, {} street lights, 3 cones, 5 trees",
                (-1200..=1200i32).step_by(250).count()
            );
        }
    }
}

/// Spawns a solar panel canopy on the Subdivision Rooftop layer (layer 2).
///
/// Layout (bottom to top):
///   z = +4  Opaque dark backdrop — hides parallax houses behind the panel
///   z = +5  Semi-transparent dark-blue panel strip — the visible "solar panel"
///
/// Spawns themed 3D decoration props inside a layer 1 sublevel.
/// All entities carry `TileEntity` so they despawn on layer switch.
/// Grid is 32 cols × 18 rows at origin (0,0). TILE_SIZE = 18.
pub fn spawn_sublevel_decorations(
    commands: &mut Commands,
    asset_server: &AssetServer,
    level_id: LevelId,
    origin_x: f32,
    origin_y: f32,
) {
    let col_x = |col: f32| origin_x + col * TILE_SIZE + TILE_SIZE * 0.5;
    let row_y = |row: f32| origin_y + row * TILE_SIZE;

    // (model_path, x, y, scale_xy, scale_z)
    let decor: &[(&str, f32, f32, f32, f32)] = match level_id {
        // ── Forest Cave: stalactites on ceiling, mushrooms, rocks ─────────
        LevelId::Forest => &[
            // Stalactites hanging from ceiling (row 16 bottom = y=288)
            (
                "models/cave/cliff_cave_stone.glb",
                col_x(6.0),
                row_y(15.0),
                12.0,
                4.0,
            ),
            (
                "models/cave/cliff_cave_rock.glb",
                col_x(15.0),
                row_y(15.0),
                10.0,
                3.0,
            ),
            (
                "models/cave/cliff_cave_stone.glb",
                col_x(25.0),
                row_y(15.0),
                14.0,
                5.0,
            ),
            // Mushrooms on ground (row 2 top = y=36)
            (
                "models/mushroom_red.glb",
                col_x(4.0),
                row_y(2.0),
                30.0,
                12.0,
            ),
            (
                "models/mushroom_tan.glb",
                col_x(18.0),
                row_y(2.0),
                25.0,
                10.0,
            ),
            ("models/mushrooms.glb", col_x(26.0), row_y(2.0), 28.0, 11.0),
            // Rocks on ground
            ("models/large_rock.glb", col_x(10.0), row_y(2.0) + 12.0, 28.0, 9.0),
            (
                "models/small_rock.glb",
                col_x(21.0),
                row_y(2.0) + 3.2,
                13.0,
                5.0,
            ),
        ],
        // ── Subdivision Sewer: columns, iron fences, wall segments ────────
        LevelId::Subdivision => &[
            // Columns along the floor
            (
                "models/sewer/column-large.glb",
                col_x(8.0),
                row_y(2.0),
                18.0,
                6.0,
            ),
            (
                "models/sewer/column-large.glb",
                col_x(24.0),
                row_y(2.0),
                18.0,
                6.0,
            ),
            (
                "models/sewer/stone-wall-column.glb",
                col_x(16.0),
                row_y(2.0),
                18.0,
                6.0,
            ),
            // Iron fences
            (
                "models/sewer/iron-fence.glb",
                col_x(3.0),
                row_y(2.0),
                20.0,
                5.0,
            ),
            (
                "models/sewer/iron-fence.glb",
                col_x(28.0),
                row_y(2.0),
                20.0,
                5.0,
            ),
            // Wall decorations along ceiling
            (
                "models/sewer/brick-wall.glb",
                col_x(10.0),
                row_y(15.0),
                16.0,
                4.0,
            ),
            (
                "models/sewer/brick-wall.glb",
                col_x(22.0),
                row_y(15.0),
                16.0,
                4.0,
            ),
        ],
        // ── City Subway: structural columns and wall segments ─────────────
        LevelId::City => &[
            // Stone columns — subway support pillars
            (
                "models/sewer/column-large.glb",
                col_x(8.0),
                row_y(2.0),
                18.0,
                6.0,
            ),
            (
                "models/sewer/column-large.glb",
                col_x(16.0),
                row_y(2.0),
                18.0,
                6.0,
            ),
            (
                "models/sewer/column-large.glb",
                col_x(24.0),
                row_y(2.0),
                18.0,
                6.0,
            ),
            // Wall segments along ceiling
            (
                "models/sewer/brick-wall.glb",
                col_x(6.0),
                row_y(15.0),
                16.0,
                4.0,
            ),
            (
                "models/sewer/brick-wall.glb",
                col_x(16.0),
                row_y(15.0),
                16.0,
                4.0,
            ),
            (
                "models/sewer/brick-wall.glb",
                col_x(26.0),
                row_y(15.0),
                16.0,
                4.0,
            ),
        ],
    };

    // Cave/Sewer: emissive glow (bioluminescent/atmospheric).
    // Subway: NO emissive — lit by point lights for proper 3D edge definition.
    let glow = match level_id {
        LevelId::Forest => Some(LinearRgba::new(1.2, 0.8, 0.4, 1.0)),
        LevelId::Subdivision => Some(LinearRgba::new(0.4, 0.7, 0.4, 1.0)),
        LevelId::City => None, // point lights instead
    };

    info!(
        "[SUBLEVEL_DECOR] level={:?} origin=({origin_x}, {origin_y}) decor_count={} glow={:?}",
        level_id,
        decor.len(),
        glow.is_some(),
    );
    for &(model, x, y, sxy, sz) in decor {
        info!(
            "[SUBLEVEL_DECOR] spawn model={model} pos=({x}, {y}, 3.0) scale=({sxy}, {sxy}, {sz})"
        );
        let is_tripo_rock = model.contains("large_rock") || model.contains("small_rock");
        let xform = if is_tripo_rock {
            Transform::from_xyz(x, y, 3.0)
                .with_rotation(Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2))
                .with_scale(Vec3::new(sz, sxy, sxy))
        } else {
            Transform::from_xyz(x, y, 3.0).with_scale(Vec3::new(sxy, sxy, sz))
        };
        let mut entity = commands.spawn((
            SceneRoot(asset_server.load(format!("{}#Scene0", model))),
            xform,
            components::TileEntity,
            components::ForegroundDecoration,
        ));
        if let Some(color) = glow {
            entity.insert(crate::collectibles::components::MakeEmissive {
                color,
                keep_lit: true,
            });
        }
    }

    // Sublevel point lights — carry TileEntity for auto-despawn.
    let lights: &[(f32, f32, Color, f32)] = match level_id {
        // Cave: warm amber torches — high intensity to illuminate vertex-color models
        LevelId::Forest => &[
            (col_x(8.0), row_y(8.0), Color::srgb(1.0, 0.6, 0.2), 200000.0),
            (
                col_x(22.0),
                row_y(8.0),
                Color::srgb(1.0, 0.6, 0.2),
                200000.0,
            ),
            (
                col_x(15.0),
                row_y(4.0),
                Color::srgb(1.0, 0.6, 0.2),
                150000.0,
            ),
        ],
        // Sewer: green-tinted industrial lighting
        LevelId::Subdivision => &[
            (col_x(8.0), row_y(8.0), Color::srgb(0.5, 0.8, 0.4), 150000.0),
            (
                col_x(16.0),
                row_y(4.0),
                Color::srgb(0.6, 0.8, 0.5),
                120000.0,
            ),
            (
                col_x(24.0),
                row_y(8.0),
                Color::srgb(0.5, 0.8, 0.4),
                150000.0,
            ),
        ],
        // Subway: cool fluorescent station lighting — multiple fixtures
        LevelId::City => &[
            (
                col_x(5.0),
                row_y(13.0),
                Color::srgb(0.9, 0.9, 1.0),
                120000.0,
            ),
            (
                col_x(12.0),
                row_y(13.0),
                Color::srgb(0.9, 0.9, 1.0),
                120000.0,
            ),
            (
                col_x(20.0),
                row_y(13.0),
                Color::srgb(0.9, 0.9, 1.0),
                120000.0,
            ),
            (
                col_x(27.0),
                row_y(13.0),
                Color::srgb(0.9, 0.9, 1.0),
                120000.0,
            ),
        ],
    };

    info!("[SUBLEVEL_LIGHTS] spawning {} lights", lights.len());
    for &(lx, ly, color, intensity) in lights {
        info!("[SUBLEVEL_LIGHTS] pos=({lx}, {ly}, 10.0) intensity={intensity}");
        commands.spawn((
            PointLight {
                color,
                intensity,
                radius: 0.5,
                range: 200.0,
                shadows_enabled: false,
                ..default()
            },
            Transform::from_xyz(lx, ly, 10.0),
            components::TileEntity,
        ));
    }
}

/// Rain (z=+20) renders in front of both; clouds (z=-60) are dimly visible
/// through the semi-transparent panel, giving the "rain above, player below" feel.
///
/// Entities carry `TileEntity` so they despawn automatically on layer switch.
/// Only call when level_id == Subdivision && layer_index == 2.
pub fn spawn_solar_panel_canopy(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    // Row 14 top = 70.  Panel bottom sits ~2 tiles above that = 70 + 36 = 106.
    // Panel is 18 units tall (1 tile thick) — feels like a low overhead structure.
    let panel_bottom = 106.0;
    let panel_height = 18.0;
    let panel_center_y = panel_bottom + panel_height * 0.5; // 115
    let level_width = 2000.0; // wider than the 1728-unit level for edge coverage

    // Opaque backdrop: covers from panel bottom to well above camera top.
    // Blocks parallax houses (z=-50/-80) and sky (z=-100) from showing
    // above the panel. Uses a dark grey-blue matching the overcast sky overlay
    // so the transition is seamless.
    let backdrop_height = 500.0;
    let backdrop_y = panel_bottom + backdrop_height * 0.5;
    let backdrop_mesh = meshes.add(Rectangle::new(level_width, backdrop_height));
    let backdrop_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.42, 0.45, 0.52),
        unlit: true,
        alpha_mode: AlphaMode::Opaque,
        ..default()
    });
    commands.spawn((
        components::TileEntity,
        Mesh3d(backdrop_mesh),
        MeshMaterial3d(backdrop_mat),
        Transform::from_xyz(0.0, backdrop_y, 4.0),
    ));

    // Solar panel: dark blue-grey, semi-transparent. Full level width.
    let panel_mesh = meshes.add(Rectangle::new(level_width, panel_height));
    let panel_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.10, 0.12, 0.25, 0.65),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    commands.spawn((
        components::TileEntity,
        Mesh3d(panel_mesh),
        MeshMaterial3d(panel_mat),
        Transform::from_xyz(0.0, panel_center_y, 5.0),
    ));
}

/// Canonical level spawn entry point — shared by handle_new_game, apply_debug_start,
/// and any future code that needs to load a level from scratch.
///
/// Performs, in order:
///   1. Build LevelData for the requested level
///   2. Update CurrentLevel resource (id + layer)
///   3. Spawn tilemap for the requested layer
///   4. Spawn all gameplay entities (stars, enemies, gate, exit)
///   5. Spawn transition doors
///   6. Spawn background and decorations
///   7. Insert LevelData as a resource
///
/// Returns the spawn point for the loaded layer so the caller can teleport
/// Returns (solid_model, platform_model) paths for a given level + layer.
/// Layer 1 sublevels use unique themed tile models; all others use surface defaults.
pub fn tile_models_for_layer(
    level_id: LevelId,
    layer_index: usize,
) -> (&'static str, &'static str) {
    match (level_id, layer_index) {
        (LevelId::Forest, 1) => ("models/grass-block.glb", "models/grass-block.glb"),
        (LevelId::Subdivision, 1) => ("models/redbricks.glb", "models/redbricks.glb"),
        (LevelId::City, 1) => ("models/cement-platform.glb", "models/cement-platform.glb"),
        (LevelId::Subdivision, _) => ("models/redbricks.glb", "models/redbricks.glb"),
        (LevelId::City, _) => ("models/cement-platform.glb", "models/cement-platform.glb"),
        _ => ("models/grass-block.glb", "models/grass-block.glb"),
    }
}

/// Returns an optional tint color for tiles in a given level.
/// Currently no levels use a tint (Subdivision formerly used a brick tint
/// but now uses the redbricks.glb model which has its own texture).
pub fn tile_tint_for_layer(level_id: LevelId) -> Option<Color> {
    match level_id {
        _ => None,
    }
}

/// the player — each call site has a different query type for the player,
/// so teleportation is the caller's responsibility.
///
/// Callers are responsible for despawning old entities and resetting resources
/// before calling this function.
#[allow(clippy::too_many_arguments)]
pub fn spawn_level_full(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    asset_server: &AssetServer,
    progress: &mut CollectionProgress,
    current_level: &mut CurrentLevel,
    level_id: LevelId,
    layer_index: usize,
    skip_enemies: bool,
) -> Vec2 {
    // ── Try compiled JSON first; fall back to hardcoded data on any error ────
    //
    // The JSON path provides both the tile grid (LevelData) and entity
    // positions (stars, enemies, gate, exit, doors).  The hardcoded path
    // provides the same information via the static *_level() functions.
    //
    // Everything AFTER this block (decorations, sublevel setup, resource
    // insertion) is shared and uses the `level_data` / `layer` locals set here.
    let level_id_str = match level_id {
        LevelId::Forest => "Forest",
        LevelId::Subdivision => "Subdivision",
        LevelId::City => "City",
    };

    // `json_entities_spawned` tracks whether the JSON path handled entity
    // spawning so the fallback does not double-spawn.
    let mut json_entities_spawned = false;

    // Attempt to load compiled JSON.  `try_load_compiled_levels` returns None
    // on any file / parse / version error and logs a warning internally.
    let level_data =
        if let Some(compiled_root) =
            compiled_data::try_load_compiled_levels("assets/levels/compiled_levels.json")
        {
            if let Some(compiled_level) = compiled_root
                .levels
                .iter()
                .find(|l| l.id == level_id_str)
            {
                info!(
                    "[compiled_data] using JSON data for level {level_id_str}"
                );

                // Convert JSON → LevelData for the tile grid.
                let data =
                    compiled_data::compiled_to_level_data(compiled_level, level_id);

                // Clamp layer_index before we borrow.
                let clamped = layer_index.min(data.layers.len().saturating_sub(1));
                let layer = &data.layers[clamped];
                let origin = Vec2::new(
                    layer.origin_x + TILE_SIZE * 0.5,
                    layer.origin_y + TILE_SIZE * 0.5,
                );
                let tiles = layer.tiles.clone();
                let (solid_model, platform_model) =
                    tile_models_for_layer(level_id, clamped);

                if let Some(tint) = tile_tint_for_layer(level_id) {
                    spawn_tilemap_tinted(
                        commands,
                        asset_server,
                        solid_model,
                        platform_model,
                        &tiles,
                        origin,
                        0.0,
                        tint,
                    );
                } else {
                    spawn_tilemap(
                        commands,
                        asset_server,
                        solid_model,
                        platform_model,
                        &tiles,
                        origin,
                        0.0,
                    );
                }

                // Tilemap is already spawned above from converted LevelData.
                // Mark true so the hardcoded path does not re-spawn tiles.
                json_entities_spawned = true;

                // Spawn entities from JSON data for this layer.
                // Bounds-check against the raw JSON layers (should match converted
                // data, but guard against divergence to prevent panics).
                if let Some(compiled_layer) = compiled_level.layers.get(clamped) {
                    compiled_data::spawn_entities_from_compiled(
                        commands,
                        meshes,
                        materials,
                        asset_server,
                        progress,
                        compiled_layer,
                        level_id,
                        skip_enemies,
                    );

                    // JSON doors: only spawn from hardcoded path when the compiled
                    // layer has no door entries (so JSON-driven levels with explicit
                    // door positions win; levels without override fall back).
                    if compiled_layer.doors.is_empty() {
                        doors::spawn_doors_for_level(commands, asset_server, level_id);
                    }
                } else {
                    warn!("[compiled_data] layer index {clamped} out of range in JSON; spawning hardcoded entities only");
                    // Tiles are from JSON but entities fall back to hardcoded.
                    spawn_entities_for_level(
                        commands, meshes, materials, asset_server, progress,
                        level_id, skip_enemies,
                    );
                    doors::spawn_doors_for_level(commands, asset_server, level_id);
                }
                data
            } else {
                // JSON loaded but this level is not present — use hardcoded.
                warn!(
                    "[compiled_data] level {level_id_str} not found in JSON; using hardcoded data"
                );
                match level_id {
                    LevelId::Forest => forest_level(),
                    LevelId::Subdivision => subdivision_level(),
                    LevelId::City => city_level(),
                }
            }
        } else {
            // No JSON available — use hardcoded data (normal during development).
            match level_id {
                LevelId::Forest => forest_level(),
                LevelId::Subdivision => subdivision_level(),
                LevelId::City => city_level(),
            }
        };

    // Clamp layer_index against the actual (possibly hardcoded) data.
    let layer_index = layer_index.min(level_data.layers.len().saturating_sub(1));
    let layer = &level_data.layers[layer_index];
    let origin = Vec2::new(
        layer.origin_x + TILE_SIZE * 0.5,
        layer.origin_y + TILE_SIZE * 0.5,
    );
    let spawn = layer.spawn;
    let tiles = layer.tiles.clone();

    let (solid_model, platform_model) = tile_models_for_layer(level_id, layer_index);

    current_level.level_id = Some(level_id);
    current_level.layer_index = layer_index;

    // Hardcoded fallback: spawn tilemap + entities + doors only when the JSON
    // path did not already handle them.
    if !json_entities_spawned {
        if let Some(tint) = tile_tint_for_layer(level_id) {
            spawn_tilemap_tinted(
                commands,
                asset_server,
                solid_model,
                platform_model,
                &tiles,
                origin,
                0.0,
                tint,
            );
        } else {
            spawn_tilemap(
                commands,
                asset_server,
                solid_model,
                platform_model,
                &tiles,
                origin,
                0.0,
            );
        }
        spawn_entities_for_level(
            commands,
            meshes,
            materials,
            asset_server,
            progress,
            level_id,
            skip_enemies,
        );
        doors::spawn_doors_for_level(commands, asset_server, level_id);
    }
    spawn_level_decorations(commands, meshes, materials, asset_server, level_id);

    // Solar panel canopy on Subdivision Rooftop layer only.
    if level_id == LevelId::Subdivision && layer_index == 2 {
        spawn_solar_panel_canopy(commands, meshes, materials);
    }

    // Sublevel setup: dark background, decorations, return door.
    if layer_index == 1 {
        let ox = layer.origin_x;
        let oy = layer.origin_y;
        let center_x = ox + 16.0 * TILE_SIZE;
        let center_y = oy + 9.0 * TILE_SIZE;

        let bg_color = match level_id {
            LevelId::Forest => Color::srgb(0.12, 0.10, 0.07),
            LevelId::Subdivision => Color::srgb(0.08, 0.10, 0.07),
            LevelId::City => Color::srgb(0.10, 0.10, 0.15),
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
            components::TileEntity,
        ));

        spawn_sublevel_decorations(commands, asset_server, level_id, ox, oy);

        let door_x = ox + 28.0 * TILE_SIZE + TILE_SIZE * 0.5;
        let door_y = oy + 2.0 * TILE_SIZE;
        commands.spawn((
            SceneRoot(asset_server.load("models/door-rotate.glb#Scene0")),
            Transform::from_xyz(door_x, door_y, 1.0).with_scale(Vec3::new(60.0, 54.0, 7.0)),
            doors::TransitionDoor { target_layer: 0 },
            components::TileEntity,
        ));
    }

    commands.insert_resource(level_data);

    spawn
}

#[allow(clippy::too_many_arguments)]
fn handle_new_game(
    mut commands: Commands,
    mut new_game: ResMut<NewGameRequested>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut progress: ResMut<CollectionProgress>,
    mut game_progress: ResMut<GameProgress>,
    mut current_level: ResMut<CurrentLevel>,
    tiles: Query<Entity, With<components::TileEntity>>,
    enemies: Query<Entity, With<Enemy>>,
    collectibles: Query<Entity, With<Collectible>>,
    gates: Query<Entity, With<LevelGate>>,
    exits: Query<Entity, With<LevelExit>>,
    doors: Query<Entity, With<TransitionDoor>>,
    // Decoration covers all level-specific parallax (ParallaxBackground entities also
    // carry Decoration), so no separate parallax query needed here.
    decorations: Query<Entity, With<components::Decoration>>,
    mut player_query: Query<(&mut Transform, &mut Health, &mut LinearVelocity), With<Player>>,
) {
    // Always clear transition state on entry to Playing — regardless of
    // whether this is a new game or a resume/load.  Transition flags must
    // never persist across a state boundary; they are only meaningful
    // within an active Playing session.
    game_progress.transition_in_progress = false;
    game_progress.transition_cooldown = 0;

    if !new_game.0 {
        return;
    }
    new_game.0 = false;

    // Despawn all gameplay entities.
    // All ParallaxBackground entities also carry Decoration, so they are covered here.
    for entity in tiles
        .iter()
        .chain(enemies.iter())
        .chain(collectibles.iter())
        .chain(gates.iter())
        .chain(exits.iter())
        .chain(doors.iter())
        .chain(decorations.iter())
    {
        commands.entity(entity).despawn();
    }

    // Reset resources.
    *game_progress = GameProgress::default();
    *current_level = CurrentLevel::default();

    // Spawn Forest via the canonical shared path.
    let spawn = spawn_level_full(
        &mut commands,
        &mut meshes,
        &mut materials,
        &asset_server,
        &mut progress,
        &mut current_level,
        LevelId::Forest,
        0,
        false, // skip_enemies — normal gameplay always spawns enemies
    );

    // Teleport player and reset health (new game only).
    if let Ok((mut player_tf, mut health, mut player_vel)) = player_query.single_mut() {
        player_tf.translation.x = spawn.x;
        player_tf.translation.y = spawn.y;
        *health = Health::new(100.0);
        *player_vel = LinearVelocity::ZERO;
    }
}
