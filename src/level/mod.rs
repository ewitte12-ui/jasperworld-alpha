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
use crate::states::NewGameRequested;
use crate::tilemap::spawn::spawn_tilemap;
use crate::tilemap::tilemap::TILE_SIZE;
use crate::rendering::parallax::{spawn_nature_background, spawn_shared_background, spawn_subdivision_background};
use doors::TransitionDoor;
use forest::forest_level;
use subdivision::subdivision_level;
use level_data::{CurrentLevel, LevelId};

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
            .add_systems(
                OnEnter(crate::states::AppState::Playing),
                handle_new_game,
            )
            .add_systems(
                Update,
                (
                    systems::switch_layer
                        .in_set(crate::puzzle::components::TransitionSet)
                        .after(crate::puzzle::systems::check_level_exit),
                    systems::camera_clamp
                        .in_set(crate::rendering::camera::CameraPipeline::Clamp),
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
            spawn_forest_inner(commands, meshes, materials, asset_server, progress, skip_enemies);
        }
        LevelId::Subdivision => {
            spawn_subdivision_inner(commands, meshes, materials, asset_server, progress, skip_enemies);
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
        Vec3::new(col_x(7.0),  ground_y,       1.0), // ground — Screen 1
        Vec3::new(col_x(20.0), ground_y,       1.0), // ground — Screen 1
        Vec3::new(col_x(5.0),  stand_y(6.0),  1.0), // Platform A (row 6)
        Vec3::new(col_x(15.0), stand_y(10.0), 1.0), // Platform C (row 10)
        Vec3::new(col_x(37.0), stand_y(6.0),  1.0), // Platform D (row 6)
        Vec3::new(col_x(46.0), stand_y(10.0), 1.0), // Platform E (row 10)
        // WHY row 14 bonus: this is the optional micro-objective star.
        // Gate opens at 10 of 11 — the player can skip this star and still
        // finish the level.  From Platform E (row 10, cols 44-49) the row 14
        // platform (cols 48-52) is visible one jump above, making the detour
        // a genuine choice rather than a required stop.  Taking the detour
        // teaches the scan-then-commit pattern at maximum height; skipping it
        // keeps the route on the standard ground→row6→row10 path.
        Vec3::new(col_x(50.0), stand_y(14.0), 1.0), // Row 14 — optional micro-objective
        Vec3::new(col_x(58.0), ground_y,       1.0), // ground — Screen 2
        Vec3::new(col_x(69.0), stand_y(6.0),  1.0), // Platform H (row 6)
        Vec3::new(col_x(77.0), stand_y(10.0), 1.0), // Platform I (row 10)
        Vec3::new(col_x(85.0), ground_y,       1.0), // ground — Screen 3
    ];
    for pos in star_positions {
        spawn_collectible(commands, meshes, materials, asset_server, pos, CollectibleType::Star);
    }
    // WHY stars_total = 10 with 11 spawned: the row 14 star (index 6) is the
    // optional micro-objective.  Gate opens when 10 are collected so the
    // player can reach the exit without the vertical detour.  Collecting all
    // 11 is possible and rewards curiosity without changing the gate rule.
    progress.stars_total = 10;
    progress.stars_collected = 0;

    let apple_positions = [
        Vec3::new(col_x(3.0),  ground_y,      1.0),
        Vec3::new(col_x(25.0), stand_y(6.0),  1.0),
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
        Vec3::new(col_x(52.0), ground_y,      1.0), // Dog zone — risky route reward
        Vec3::new(col_x(57.0), stand_y(6.0),  1.0), // Platform F — safe route reward
        Vec3::new(col_x(79.0), stand_y(10.0), 1.0),
    ];
    for pos in apple_positions {
        spawn_collectible(commands, meshes, materials, asset_server, pos, CollectibleType::HealthFood);
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
                Transform::from_xyz(0.0, -200.0, 0.0)
                    .with_scale(Vec3::new(18.0, 80.0, 7.0)),
            ));
        });

    commands.spawn((
        Transform::from_xyz(gate_x + 30.0, ground_y, 0.5),
        Visibility::Hidden,
        LevelExit { next_level: LevelId::Subdivision, half_extents: Vec2::new(51.0, 100.0) },
    ));

    // End-zone landmark — open door as forest exit cue.
    commands.spawn((
        SceneRoot(asset_server.load("models/door-open.glb#Scene0")),
        Transform::from_xyz(gate_x + 40.0, ground_top, -1.0)
            .with_scale(Vec3::new(60.0, 54.0, 7.0)),
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
        (EnemyType::Dog,    Vec2::new(col_x(47.0), ground_top), 72.0_f32), // ±4 tiles; readable arc from Platform E
        (EnemyType::Snake,  Vec2::new(col_x(74.0), ground_top), 54.0_f32),
        (EnemyType::Possum, Vec2::new(col_x(82.0), ground_top), 54.0_f32),
    ];
    for (etype, pos, patrol) in enemies {
        spawn_enemy(commands, meshes, materials, asset_server, etype, pos, patrol);
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
        Vec3::new(col_x(8.0),  ground_y,       1.0),
        Vec3::new(col_x(22.0), ground_y,       1.0),
        Vec3::new(col_x(6.0),  stand_y(6.0),  1.0),
        Vec3::new(col_x(16.0), stand_y(10.0), 1.0),
        Vec3::new(col_x(40.0), stand_y(6.0),  1.0),
        Vec3::new(col_x(48.0), stand_y(10.0), 1.0),
        Vec3::new(col_x(32.0), stand_y(14.0), 1.0), // optional high platform
        Vec3::new(col_x(60.0), ground_y,       1.0),
        Vec3::new(col_x(70.0), stand_y(6.0),  1.0),
        Vec3::new(col_x(78.0), stand_y(10.0), 1.0),
        Vec3::new(col_x(87.0), ground_y,       1.0),
    ];
    for pos in star_positions {
        spawn_collectible(commands, meshes, materials, asset_server, pos, CollectibleType::Star);
    }
    progress.stars_total = 10;
    progress.stars_collected = 0;

    // Apples (5)
    let apple_positions = [
        Vec3::new(col_x(4.0),  ground_y,      1.0),
        Vec3::new(col_x(26.0), stand_y(6.0),  1.0),
        Vec3::new(col_x(53.0), ground_y,      1.0), // Dog zone risk reward
        Vec3::new(col_x(58.0), stand_y(6.0),  1.0),
        Vec3::new(col_x(80.0), stand_y(10.0), 1.0),
    ];
    for pos in apple_positions {
        spawn_collectible(commands, meshes, materials, asset_server, pos, CollectibleType::HealthFood);
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
                Transform::from_xyz(0.0, -200.0, 0.0)
                    .with_scale(Vec3::new(18.0, 80.0, 7.0)),
            ));
        });

    // Exit — next_level irrelevant; game_complete fires at level_index >= 2
    commands.spawn((
        Transform::from_xyz(gate_x + 30.0, ground_y, 0.5),
        Visibility::Hidden,
        LevelExit { next_level: LevelId::Subdivision, half_extents: Vec2::new(51.0, 100.0) },
    ));

    // End-zone landmark
    commands.spawn((
        SceneRoot(asset_server.load("models/door-open.glb#Scene0")),
        Transform::from_xyz(gate_x + 40.0, ground_top, -1.0)
            .with_scale(Vec3::new(60.0, 54.0, 7.0)),
        components::Decoration,
    ));

    if !skip_enemies {
    // Dog: wider patrol range (108 vs Forest's 72) for harder encounter
    let enemies = [
        (EnemyType::Dog,    Vec2::new(col_x(50.0), ground_top), 108.0_f32),
        (EnemyType::Snake,  Vec2::new(col_x(75.0), ground_top), 54.0_f32),
        (EnemyType::Possum, Vec2::new(col_x(84.0), ground_top), 54.0_f32),
    ];
    for (etype, pos, patrol) in enemies {
        spawn_enemy(commands, meshes, materials, asset_server, etype, pos, patrol);
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
            let decor: &[(&str, f32, f32, f32, f32)] = &[
                ("models/rock_smallA.glb",     col_x_f(3.0),  -141.0, 13.0,  5.0),
                ("models/plant_bush.glb",      col_x_f(9.0),  -141.0, 26.0, 14.0),
                ("models/flower_redA.glb",     col_x_f(18.0), -141.0, 24.0,  6.0),
                ("models/rock_tallA.glb",      col_x_f(30.0), -141.0, 12.0,  4.0),
                ("models/plant_bushLarge.glb", col_x_f(43.0), -141.0, 26.0, 14.0),
                ("models/flower_yellowA.glb",  col_x_f(5.0),   -69.0, 24.0,  6.0),
                ("models/rock_smallA.glb",     col_x_f(8.0),   -69.0, 13.0,  5.0),
                ("models/plant_bush.glb",      col_x_f(24.0),  -69.0, 26.0, 14.0),
                ("models/flower_redA.glb",     col_x_f(25.0),  -69.0, 24.0,  6.0),
                ("models/rock_smallA.glb",     col_x_f(14.0),    3.0, 13.0,  5.0),
                ("models/grass_large.glb",     col_x_f(16.0),    3.0, 26.0,  8.0),
                ("models/flower_yellowA.glb",  col_x_f(46.0),    3.0, 24.0,  6.0),
                ("models/plant_bushLarge.glb", col_x_f(49.0),   75.0, 26.0, 14.0),
                ("models/rock_smallA.glb",     col_x_f(52.0),   75.0, 13.0,  5.0),
                ("models/flower_redA.glb",     col_x_f(69.0),   75.0, 24.0,  6.0),
                ("models/plant_bush.glb",      col_x_f(71.0),   75.0, 26.0, 14.0),
            ];
            for &(model, x, y, sxy, sz) in decor {
                commands.spawn((
                    SceneRoot(asset_server.load(format!("{}#Scene0", model))),
                    // z=+3: in front of tile plane (z=0) so props are not occluded
                    // by the 3D depth of ground block GLBs.
                    Transform::from_xyz(x, y, 3.0).with_scale(Vec3::new(sxy, sxy, sz)),
                    components::Decoration,
                    components::ForegroundDecoration,
                ));
            }

            // Foreground framing trees (z = +10) — bookend the left/right level edges.
            // Must live here (not Startup) per jasper_background_parallax_lifecycle_guardrail:
            // biome-specific art is level content, not engine setup.
            // WHY left trees at -367/-342: at the original -295/-270 they sat only 2.5 cols
            // from Platform D, occluding the approach.  -367/-342 gives a 117-unit clear window.
            // WHY -450/-420: door 1 is at x=-351 (footprint ≈ -381 to -321).
            // Original -367/-342 were inside that range and occluded the door.
            // -450/-420 are clearly left of the door's edge.
            let fg_trees: &[(&str, f32, f32, f32)] = &[
                ("models/tree_oak.glb",  -450.0, -146.0, 95.0),
                ("models/tree_fat.glb",  -420.0, -146.0, 80.0),
                ("models/tree_pine.glb",  270.0, -146.0, 90.0),
                ("models/tree_oak.glb",   295.0, -146.0, 85.0),
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

            // Ground-level suburban props (z=+3, in front of tile plane)
            let ox = -864.0_f32;
            let col_x_f = |col: f32| ox + col * 18.0 + 9.0;
            // (model, x, y, scale_xy, scale_z)
            let decor: &[(&str, f32, f32, f32, f32)] = &[
                ("models/suburban/planter.glb",       col_x_f(4.0),  -141.0, 20.0, 10.0),
                ("models/suburban/fence-suburban.glb", col_x_f(15.0), -141.0, 30.0,  8.0),
                ("models/suburban/planter.glb",       col_x_f(28.0), -141.0, 18.0,  9.0),
                ("models/suburban/fence-suburban.glb", col_x_f(42.0), -141.0, 30.0,  8.0),
                ("models/suburban/planter.glb",       col_x_f(55.0), -141.0, 22.0, 11.0),
                ("models/suburban/fence-suburban.glb", col_x_f(65.0), -141.0, 30.0,  8.0),
                ("models/suburban/planter.glb",       col_x_f(78.0), -141.0, 20.0, 10.0),
                ("models/suburban/fence-suburban.glb", col_x_f(88.0), -141.0, 30.0,  8.0),
            ];
            for &(model, x, y, sxy, sz) in decor {
                commands.spawn((
                    SceneRoot(asset_server.load(format!("{}#Scene0", model))),
                    Transform::from_xyz(x, y, 3.0).with_scale(Vec3::new(sxy, sxy, sz)),
                    components::Decoration,
                    components::ForegroundDecoration,
                ));
            }

            // Foreground framing — suburban trees at level edges (z=+10)
            let fg_trees: &[(&str, f32, f32, f32)] = &[
                ("models/suburban/tree-suburban-large.glb", -450.0, -146.0, 180.0),
                ("models/suburban/tree-suburban-small.glb", -420.0, -146.0, 140.0),
                ("models/suburban/tree-suburban-large.glb",  270.0, -146.0, 170.0),
                ("models/suburban/tree-suburban-small.glb",  295.0, -146.0, 150.0),
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
    let level_data = match level_id {
        LevelId::Forest => forest_level(),
        LevelId::Subdivision => subdivision_level(),
    };

    let layer_index = layer_index.min(level_data.layers.len().saturating_sub(1));
    let layer = &level_data.layers[layer_index];
    let origin = Vec2::new(
        layer.origin_x + TILE_SIZE * 0.5,
        layer.origin_y + TILE_SIZE * 0.5,
    );
    let spawn = layer.spawn;
    let tiles = layer.tiles.clone();

    let (solid_model, platform_model) = match level_id {
        LevelId::Forest => ("models/block-grass-large.glb", "models/block-grass-low.glb"),
        LevelId::Subdivision => ("models/brick.glb", "models/brick.glb"),
    };

    current_level.level_id    = Some(level_id);
    current_level.layer_index = layer_index;

    spawn_tilemap(commands, asset_server, solid_model, platform_model, &tiles, origin, 0.0);
    spawn_entities_for_level(commands, meshes, materials, asset_server, progress, level_id, skip_enemies);
    doors::spawn_doors_for_level(commands, asset_server, level_id);
    spawn_level_decorations(commands, meshes, materials, asset_server, level_id);

    commands.insert_resource(level_data);

    spawn
}

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
    for entity in tiles.iter()
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
