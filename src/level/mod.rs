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
    spawn_city_background, spawn_nature_background, spawn_sanctuary_background,
    spawn_shared_background, spawn_subdivision_background,
};
use crate::states::NewGameRequested;
use crate::tilemap::spawn::spawn_tilemap;
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
        LevelId::Sanctuary => {
            spawn_sanctuary_inner(
                commands,
                meshes,
                materials,
                asset_server,
                progress,
            );
        }
    }
}

/// Inner logic for spawning Forest level entities, callable as a free function (no Bevy system params).
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

    // Exit — transitions to Sanctuary (final level; game_complete fires there).
    commands.spawn((
        Transform::from_xyz(gate_x + 30.0, ground_y, 0.5),
        Visibility::Hidden,
        LevelExit {
            next_level: LevelId::Sanctuary,
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

/// Spawns the Sanctuary ground overlay and water quad — shared by both the
/// JSON path and the hardcoded fallback path.
///
/// WHY extracted: `spawn_sanctuary_inner` is only called on the hardcoded path.
/// When Sanctuary loads from compiled_levels.json the inner function is never
/// reached, so these decorations were silently missing. Extracting them here
/// and calling this function from the JSON path fixes that.
fn spawn_sanctuary_extras(
    commands: &mut Commands,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let ox: f32 = -432.0;
    let oy: f32 = -200.0;
    let ground_top = oy + 3.0 * 18.0; // -146.0  (top of the 3 solid rows)

    // ── Bottom 2 ground rows — decorative overlay ─────────────────────────────
    // The tile pipeline uses grass-block for all solid tiles, but rows 0-1 should
    // visually appear as a thick ground slab. We overlay them with
    // ground_blocks_bottom2layers.glb which renders over the tile at z=0.1.
    //
    // WHY scale 18.6 (= 18.0 / 0.968): matches the grass-block tile scale.
    // GRASS_W = 0.968, TILE_SIZE = 18.0 → uniform = 18.0 / 0.968 ≈ 18.6.
    // ground_blocks_bottom2layers.glb is assumed to have the same native footprint;
    // if the visual is off, adjust this value to match the actual native dimensions.
    //
    // WHY z=0.1: slightly in front of the gameplay-plane tiles (z=0) so this model
    // draws on top of them without z-fighting, and stays behind the player (z≈1).
    let bottom_model: Handle<Scene> = asset_server.load("models/sanctuary/ground_blocks_bottom2layers.glb#Scene0");
    for row in 0..2_usize {
        for col in 0..48_usize {
            // Skip cols 43–46: water quad replaces ground blocks here.
            if (43..=46).contains(&col) {
                continue;
            }
            let wx = ox + col as f32 * 18.0 + 9.0;
            let wy = oy + row as f32 * 18.0 + 9.0;
            commands.spawn((
                SceneRoot(bottom_model.clone()),
                Transform::from_xyz(wx, wy, 0.1)
                    .with_scale(Vec3::splat(18.6)),
                components::Decoration,
            ));
        }
    }

    // Water wall at the end of the level — visual endpoint the player walks into.
    // WHY: the old door model was removed; the water PNG acts as the finishing
    // landmark. Placed at col 45 (just right of the LevelExit trigger at col 44)
    // so the player sees the water and the exit fires as they step into it.
    // WHY z=0.2: slightly in front of gameplay plane (z=0) so it draws over ground
    // tiles, behind player (z=1). AlphaMode::Blend preserves PNG transparency.
    let col_x = |c: f32| ox + c * 18.0 + 9.0;
    let water_texture: Handle<Image> = asset_server.load("models/sanctuary/water_at_end_oflevel.png");
    let water_mesh = meshes.add(Rectangle::new(72.0, 54.0)); // ~4 tiles wide, 3 tiles tall
    let water_material = materials.add(StandardMaterial {
        base_color_texture: Some(water_texture),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        double_sided: true,
        cull_mode: None,
        ..default()
    });
    let water_x = col_x(45.0); // near right edge, just past the LevelExit trigger
    // WHY ground_top - 27.0: centers the 54-unit-tall water quad in the stone
    // block area below the grass line. Top edge at ground_top, bottom edge at
    // ground_top - 54, placing the water flush across the lower stone wall.
    let water_y = ground_top - 27.0;
    commands.spawn((
        Mesh3d(water_mesh),
        MeshMaterial3d(water_material),
        Transform::from_xyz(water_x, water_y, 0.2),
        components::Decoration,
    ));

    // Raccoon family portrait at the water's edge — the ending scene.
    // Positioned just left of the water, standing on the ground surface.
    let family_texture: Handle<Image> =
        asset_server.load("models/sanctuary/raccoon_family.png");
    let family_mesh = meshes.add(Rectangle::new(54.0, 54.0)); // ~3 tiles square
    let family_material = materials.add(StandardMaterial {
        base_color_texture: Some(family_texture),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        double_sided: true,
        cull_mode: None,
        ..default()
    });
    // Place at col 42 (just left of the water gap), centered vertically
    // with base at ground_top.
    let family_x = col_x(42.0);
    let family_y = ground_top + 27.0; // base at ground_top, center 27 units up
    commands.spawn((
        Mesh3d(family_mesh),
        MeshMaterial3d(family_material),
        Transform::from_xyz(family_x, family_y, 1.5),
        components::Decoration,
    ));

    // Invisible wall at the left edge of the water gap (col 43).
    // WHY: ground colliders were removed at cols 43-46 for the water, so
    // without this the player falls into the void. This thin static wall
    // stops the player at the water's edge while the LevelExit trigger
    // (at col 44 + 30 units) fires as they approach.
    let wall_x = col_x(43.0) - 9.0; // left edge of col 43
    let wall_y = ground_top + 100.0; // tall enough the player can't jump over
    commands.spawn((
        Transform::from_xyz(wall_x, wall_y, 0.0),
        Visibility::Hidden,
        avian2d::prelude::RigidBody::Static,
        avian2d::prelude::Collider::rectangle(4.0, 200.0),
        components::Decoration,
    ));
}

/// Inner logic for Sanctuary level entity spawning.
///
/// Sanctuary is the final peaceful level — no enemies, no stars, no health_foods.
/// A single LevelExit at gate_col 44 auto-completes when the player reaches it
/// (stars_required = 0, so the gate is never spawned).
fn spawn_sanctuary_inner(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    asset_server: &AssetServer,
    progress: &mut CollectionProgress,
) {
    const OX: f32 = -432.0;
    const OY: f32 = -200.0;
    let col_x = |col: f32| OX + col * 18.0 + 9.0;
    let stand_y = |row: f32| OY + (row + 1.0) * 18.0 + 9.0;
    let ground_y = stand_y(2.0);

    // No stars, no apples, no enemies — this is a peaceful walk to water.
    // stars_total = 0 so check_gate never fires (no gate to despawn).
    progress.stars_total = 0;
    progress.stars_collected = 0;

    // Exit at col 44 — walk through the sanctuary to reach it.
    // next_level is a dummy value; game_complete fires when current_level_index >= 4.
    let exit_x = col_x(44.0);
    commands.spawn((
        Transform::from_xyz(exit_x + 30.0, ground_y, 0.5),
        Visibility::Visible,
        LevelExit {
            next_level: LevelId::Sanctuary, // dummy — game_complete fires
            half_extents: Vec2::new(51.0, 100.0),
        },
    ));

    // Ground overlay + water quad — shared with the JSON path.
    spawn_sanctuary_extras(commands, asset_server, meshes, materials);
}

/// Hardcoded fallback LevelData for Sanctuary.
///
/// 48 cols × 22 rows, single layer. Matches Forest height so the level
/// feels tall enough for background art and parallax depth.
/// Rows 0-2: solid ground.  Rows 3-21: empty air.
/// Used when compiled_levels.json does not yet contain a Sanctuary entry.
fn sanctuary_level() -> crate::level::level_data::LevelData {
    use crate::level::level_data::{LayerData, LevelData};
    use crate::tilemap::tilemap::TileType::{Empty as E, Solid as S};

    let solid = vec![S; 48];
    let empty = vec![E; 48];
    // 3 solid ground rows + 19 empty air rows = 22 rows total (matches Forest).
    let mut tiles = vec![
        solid.clone(), // row 0
        solid.clone(), // row 1
        solid.clone(), // row 2
    ];
    for _ in 3..22 {
        tiles.push(empty.clone());
    }

    LevelData {
        id: LevelId::Sanctuary,
        layers: vec![LayerData {
            id: 0,
            tiles,
            // origin_x = -432 gives 48 cols × 18 units = 864 units total,
            // centred roughly on x=0 for a short peaceful level.
            origin_x: -432.0,
            origin_y: -200.0,
            spawn: Vec2::new(-396.0, -128.0),
        }],
    }
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
            // Ground-level Forest props and foreground trees are now data-driven:
            // they are stored in compiled_levels.json (Forest layer 0 "props" array)
            // and spawned by spawn_entities_from_compiled via the JSON path.
            // See compiled_data.rs CompiledProp for the scale/rotation convention:
            //   Tripo rocks/flowers: rotation_y=-PI/2, scale=(sz, sxy, sxy)
            //   Kenney models: rotation_y=0, scale=(sxy, sxy, sz)
            //   Foreground trees (z=10): scale=(s, s, 1.0), foreground=true
        }
        LevelId::Subdivision => {
            spawn_subdivision_background(commands, asset_server);

            // Overcast sky overlay — parameters loaded from subdivision_bg.json.
            // Carries Decoration so it despawns on level exit, restoring blue sky.
            // WHY JSON-driven: overlay color/z/factor can be tuned without recompiling.
            if let Some(cfg) = crate::rendering::parallax_config::load_config::<
                crate::rendering::parallax_config::SubdivisionBgConfig,
            >("assets/configs/subdivision_bg.json")
            {
                if let Some(ov) = cfg.overlay {
                    let [r, g, b, a] = ov.color;
                    let ov_mesh = meshes.add(Rectangle::new(ov.width, ov.height));
                    let ov_mat = materials.add(StandardMaterial {
                        base_color: Color::srgba(r, g, b, a),
                        unlit: true,
                        alpha_mode: AlphaMode::Opaque,
                        ..default()
                    });
                    commands.spawn((
                        Mesh3d(ov_mesh),
                        MeshMaterial3d(ov_mat),
                        Transform::from_xyz(0.0, 0.0, ov.z),
                        crate::rendering::parallax::ParallaxLayer { factor: ov.factor },
                        components::Decoration,
                        crate::rendering::parallax::ParallaxBackground,
                    ));
                }
            } else {
                warn!("[SUBDIVISION_BG] could not load subdivision_bg.json — sky overlay skipped");
            }

            // Foreground framing trees are now data-driven:
            // stored in compiled_levels.json (Subdivision layer 0 "props" array)
            // and spawned by spawn_entities_from_compiled via the JSON path.
        }
        LevelId::Sanctuary => {
            spawn_sanctuary_background(commands, asset_server, meshes, materials);

            // Soft pink atmosphere overlay — parameters loaded from sanctuary_bg.json.
            // Carries Decoration so it despawns on level exit.
            // WHY JSON-driven: overlay color/z/factor can be tuned without recompiling.
            if let Some(cfg) = crate::rendering::parallax_config::load_config::<
                crate::rendering::parallax_config::SanctuaryBgConfig,
            >("assets/configs/sanctuary_bg.json")
            {
                if let Some(ov) = cfg.overlay {
                    let [r, g, b, a] = ov.color;
                    let ov_mesh = meshes.add(Rectangle::new(ov.width, ov.height));
                    let ov_mat = materials.add(StandardMaterial {
                        base_color: Color::srgba(r, g, b, a),
                        unlit: true,
                        alpha_mode: AlphaMode::Blend,
                        ..default()
                    });
                    commands.spawn((
                        Mesh3d(ov_mesh),
                        MeshMaterial3d(ov_mat),
                        Transform::from_xyz(0.0, 0.0, ov.z),
                        crate::rendering::parallax::ParallaxLayer { factor: ov.factor },
                        components::Decoration,
                        crate::rendering::parallax::ParallaxBackground,
                    ));
                }
            } else {
                warn!("[SANCTUARY_BG] could not load sanctuary_bg.json — sky overlay skipped");
            }
        }
        LevelId::City => {
            info!("[CITY] spawn_level_decorations: entering City arm");
            spawn_city_background(commands, asset_server);

            // Night sky overlay — parameters loaded from city_bg.json.
            // Dark navy rectangle at z=-99, in front of the blue sky backdrop at z=-100.
            // Hides daytime sky for night atmosphere.
            // WHY JSON-driven: overlay color/z/factor can be tuned without recompiling.
            if let Some(cfg) = crate::rendering::parallax_config::load_config::<
                crate::rendering::parallax_config::CityBgConfig,
            >("assets/configs/city_bg.json")
            {
                if let Some(ov) = cfg.overlay {
                    let [r, g, b, a] = ov.color;
                    let ov_mesh = meshes.add(Rectangle::new(ov.width, ov.height));
                    let ov_mat = materials.add(StandardMaterial {
                        base_color: Color::srgba(r, g, b, a),
                        unlit: true,
                        alpha_mode: AlphaMode::Opaque,
                        ..default()
                    });
                    commands.spawn((
                        Mesh3d(ov_mesh),
                        MeshMaterial3d(ov_mat),
                        Transform::from_xyz(0.0, 0.0, ov.z),
                        crate::rendering::parallax::ParallaxLayer { factor: ov.factor },
                        components::Decoration,
                        crate::rendering::parallax::ParallaxBackground,
                    ));
                }
            } else {
                warn!("[CITY_BG] could not load city_bg.json — sky overlay skipped");
            }

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

            // Ground-level city props (taxis, street lights, cones) and foreground
            // trees are now data-driven: stored in compiled_levels.json (City layer 0
            // "props" array) and spawned by spawn_entities_from_compiled via the JSON path.
            // Scale/rotation conventions are documented in compiled_data.rs CompiledProp.
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
    // WHY load JSON here rather than passing props/lights as parameters: the caller
    // (systems::switch_layer) does not have JSON context, and changing that
    // function signature would require threading compiled data through the ECS
    // resource graph. Loading inline matches the pattern used by handle_new_game.
    let level_id_str = match level_id {
        LevelId::Forest => "Forest",
        LevelId::Subdivision => "Subdivision",
        LevelId::City => "City",
        LevelId::Sanctuary => "Sanctuary",
    };

    // Load both props and lights from the same JSON layer 1 entry.
    let (props, lights_data): (
        Vec<compiled_data::CompiledProp>,
        Vec<compiled_data::CompiledLight>,
    ) = if let Some(root) =
        compiled_data::try_load_compiled_levels("assets/levels/compiled_levels.json")
    {
        root.levels
            .into_iter()
            .find(|l| l.id == level_id_str)
            .and_then(|level| {
                // Layer 1 is the sublevel (index 1 in the layers Vec).
                level.layers.into_iter().find(|l| l.id == 1)
            })
            .map(|layer| (layer.props, layer.lights))
            .unwrap_or_default()
    } else {
        warn!("[SUBLEVEL_DECOR] could not load compiled_levels.json — no props or lights spawned");
        (Vec::new(), Vec::new())
    };

    // Cave/Sewer: emissive glow (bioluminescent/atmospheric).
    // Subway: NO emissive — lit by point lights for proper 3D edge definition.
    let glow = match level_id {
        LevelId::Forest => Some(LinearRgba::new(1.2, 0.8, 0.4, 1.0)),
        LevelId::Subdivision => Some(LinearRgba::new(0.4, 0.7, 0.4, 1.0)),
        LevelId::City => None,      // point lights instead
        LevelId::Sanctuary => None, // no sublevels; arm required for exhaustiveness
    };

    info!(
        "[SUBLEVEL_DECOR] level={:?} origin=({origin_x}, {origin_y}) prop_count={} glow={:?}",
        level_id,
        props.len(),
        glow.is_some(),
    );
    for prop in &props {
        info!(
            "[SUBLEVEL_DECOR] spawn model={} pos=({}, {}, {})",
            prop.model_id, prop.x, prop.y, prop.z
        );
        let rotation = Quat::from_rotation_y(prop.rotation_y);
        let xform = Transform::from_xyz(prop.x, prop.y, prop.z)
            .with_rotation(rotation)
            .with_scale(Vec3::new(prop.scale_x, prop.scale_y, prop.scale_z));
        let mut entity = commands.spawn((
            SceneRoot(asset_server.load(format!("{}#Scene0", prop.model_id))),
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

    // Sublevel point lights — loaded from compiled_levels.json layer 1 "lights" array.
    // Carry TileEntity for auto-despawn on layer switch.
    // WHY JSON-driven: positions are co-authored with the sublevel layout in LDtk;
    // keeping them in JSON avoids recompiling Rust when light placement is tuned.
    info!("[SUBLEVEL_LIGHTS] spawning {} lights", lights_data.len());
    for light in &lights_data {
        info!(
            "[SUBLEVEL_LIGHTS] pos=({}, {}, {}) intensity={}",
            light.x, light.y, light.z, light.intensity
        );
        commands.spawn((
            PointLight {
                color: Color::srgb(light.color[0], light.color[1], light.color[2]),
                intensity: light.intensity,
                radius: light.radius,
                range: light.range,
                shadows_enabled: false,
                ..default()
            },
            Transform::from_xyz(light.x, light.y, light.z),
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
        (LevelId::Sanctuary, _) => (
            "models/sanctuary/ground_block_top.glb",
            "models/sanctuary/ground_block_top.glb",
        ),
        _ => ("models/grass-block.glb", "models/grass-block.glb"),
    }
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
        LevelId::Sanctuary => "Sanctuary",
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

                spawn_tilemap(
                    commands,
                    asset_server,
                    solid_model,
                    platform_model,
                    &tiles,
                    origin,
                    0.0,
                );

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

                    // Sanctuary extras (ground overlay + water quad) are not
                    // encoded in compiled_levels.json — spawn them here so the
                    // JSON path gets the same decorations as the hardcoded path.
                    if level_id == LevelId::Sanctuary {
                        spawn_sanctuary_extras(commands, asset_server, meshes, materials);
                    }

                    // JSON doors: only spawn from hardcoded path when the compiled
                    // layer has no door entries (so JSON-driven levels with explicit
                    // door positions win; levels without override fall back).
                    // WHY Sanctuary excluded: Sanctuary has no doors by design (peaceful
                    // walk-to-water level). The fallback would spawn an unwanted door.
                    if compiled_layer.doors.is_empty() && level_id != LevelId::Sanctuary {
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
                    LevelId::Sanctuary => sanctuary_level(),
                }
            }
        } else {
            // No JSON available — use hardcoded data (normal during development).
            match level_id {
                LevelId::Forest => forest_level(),
                LevelId::Subdivision => subdivision_level(),
                LevelId::City => city_level(),
                LevelId::Sanctuary => sanctuary_level(),
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
        spawn_tilemap(
            commands,
            asset_server,
            solid_model,
            platform_model,
            &tiles,
            origin,
            0.0,
        );
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
            LevelId::Sanctuary => Color::srgb(0.10, 0.08, 0.06),
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
