pub mod city;
pub mod components;
pub mod doors;
pub mod forest;
pub mod level_data;
pub mod sanctuary;
pub mod subdivision;
pub mod systems;
pub mod test_level;

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::physics::config::GameLayer;

use crate::collectibles::components::{Collectible, CollectibleType, CollectionProgress};
use crate::dialogue::components::NpcDialogue;
use crate::collectibles::systems::spawn_collectible;
use crate::combat::components::Health;
use crate::enemies::components::{Enemy, EnemyType};
use crate::enemies::spawner::spawn_enemy;
use crate::player::components::Player;
use crate::puzzle::components::{GameProgress, LevelExit, LevelGate};
use crate::states::NewGameRequested;
use crate::tilemap::spawn::spawn_tilemap;
use crate::tilemap::tilemap::TILE_SIZE;
use crate::rendering::parallax::{spawn_nature_background, spawn_shared_background, ParallaxBackground, ParallaxLayer};
use crate::rendering::subdivision_panorama::{spawn_subdivision_mid, spawn_subdivision_near};
use doors::TransitionDoor;
use forest::forest_level;
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

// ── Subdivision (origin_x = -864, origin_y = -200) ───────────────────────────
// Same origin as Forest.
// Gate: col_x(91) = 783.0  |  Exit: (813.0, -137.0) → City

pub fn spawn_subdivision_entities(
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
    let ground_top = OY + 3.0 * 18.0; // -146.0 — passed to spawn_enemy

    let star_positions = [
        Vec3::new(col_x(1.0),  ground_y,      1.0), // Ground — spawn side (S1)
        Vec3::new(col_x(5.0),  stand_y(5.0),  1.0), // Plat A (row5, cols 3–7)
        Vec3::new(col_x(29.0), stand_y(8.0),  1.0), // Plat D (row8, cols 26–31) — post C→D reward
        Vec3::new(col_x(33.0), stand_y(10.0), 1.0), // Plat E chimney (row10) — high-commitment reward
        Vec3::new(col_x(46.0), stand_y(6.0),  1.0), // Plat F_mid (row6, cols 44–48) — forced-drop survivor
        Vec3::new(col_x(53.0), stand_y(10.0), 1.0), // Plat G (row10, cols 51–55)
        Vec3::new(col_x(60.0), stand_y(6.0),  1.0), // Plat H (row6, cols 58–62)
        Vec3::new(col_x(66.0), stand_y(3.0),  1.0), // Plat I (row3, cols 65–68)
        Vec3::new(col_x(73.0), stand_y(8.0),  1.0), // Plat J (row8, cols 72–75)
        Vec3::new(col_x(87.0), stand_y(8.0),  1.0), // Plat L (row8, cols 83–90) — pre-gate
        // 11th star: ground-level safety valve — makes any one platform star skippable
        Vec3::new(col_x(42.0), ground_y,      1.0), // ground — SC2 (between chimney and F_mid)
    ];
    for pos in star_positions {
        spawn_collectible(commands, meshes, materials, asset_server, pos, CollectibleType::Star);
    }
    // Gate opens at 10 of 11: one star is skippable (chimney row10 becomes optional)
    progress.stars_total = 10;
    progress.stars_collected = 0;

    let apple_positions = [
        Vec3::new(col_x(26.0), stand_y(6.0), 1.0), // Recovery R1b (row6, cols 26–27) — zig-zag reward
        Vec3::new(col_x(50.0), stand_y(7.0), 1.0), // Recovery R2b (row7, cols 49–51) — Screen 2 zig-zag
        Vec3::new(col_x(88.0), stand_y(8.0), 1.0), // Plat L pre-gate (row8, cols 83–90)
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

    // End-of-level trigger region: AABB spanning the final section past the gate.
    // Horizontal: 51u right of center reaches level_right; 51u left extends past gate.
    // Vertical: 100u above/below ground_y covers floor-to-jump-apex.
    commands.spawn((
        Transform::from_xyz(gate_x + 30.0, ground_y, 0.5),
        Visibility::Hidden,
        LevelExit { next_level: LevelId::City, half_extents: Vec2::new(51.0, 100.0) },
    ));

    // End-zone landmark — visual destination cue (purely decorative, no collider).
    // Open door suggests passage; placed behind tiles (z=-1) to avoid occluding gameplay.
    commands.spawn((
        SceneRoot(asset_server.load("models/door-open.glb#Scene0")),
        Transform::from_xyz(gate_x + 40.0, ground_top, -1.0)
            .with_scale(Vec3::new(60.0, 54.0, 7.0)),
        components::Decoration,
    ));

    if !skip_enemies {
        // Platform surface tops for elevated enemies.
        // Spawner contract: position.y = top of the surface tile; spawner adds COLLIDER_H/2.
        let plat_top = |row: f32| OY + (row + 1.0) * 18.0;

        // ── Ground enemies (y = ground_top) ──────────────────────────────────────
        //
        // E1 — Possum — seg-2 (cols 10–20)  col 11
        //   Role: Ground Denial / Screen 1 first threat.
        //   Patrol [col 10 – col 12].  B(row8) fall zone starts col 12: boundary case;
        //   flag for patrol-range tuning pass.
        //
        // E2 — Snake — seg-4 (cols 26–42)  col 34
        //   Role: Ground Denial / Screen 2 chimney zone.
        //   Patrol [col 32 – col 36].  D(row8) fall zone ends col 31: one col clear. ✓
        //
        // ── Elevated enemies (y = plat_top(row)) ─────────────────────────────────
        //
        // E5 — Possum — A (row5, cols 3–7)  spawn col 5
        //   Role: Horizontal Patrol / Screen 1 gentle elevated intro.
        //   Patrol [col 4 – col 6].  Edges col 3 (landing) and col 7 (exit) clear. ✓
        //
        // E6 — Snake — F_mid (row6, cols 44–48)  spawn col 47
        //   Role: Horizontal Patrol / Screen 2 post-forced-drop.
        //   Patrol [col 46 – col 48].  Arrival side cols 44–45 clear. ✓
        //
        // E7 — Possum — H (row6, cols 58–62)  spawn col 60
        //   Role: Horizontal Patrol / Screen 2 exit platform.
        //   Patrol [col 59 – col 61].  Arrival col 58 and exit col 62 clear. ✓
        //   NB2→H descent lands col 58: one tile from patrol boundary — monitor in playtesting.
        //
        // Screen 3 (cols 64–95): zero enemies.  VRT approach zone uncontested by design.
        let enemies = [
            (EnemyType::Possum, Vec2::new(col_x(11.0), ground_top),    27.0_f32), // E1 — ±1.5 tiles; ground timing
            (EnemyType::Snake,  Vec2::new(col_x(34.0), ground_top),    45.0_f32), // E2 — ±2.5 tiles; chimney drop must wait
            (EnemyType::Possum, Vec2::new(col_x(5.0),  plat_top(5.0)), 27.0_f32), // E5 — 3/5 tiles; landing edges clear
            (EnemyType::Snake,  Vec2::new(col_x(47.0), plat_top(6.0)), 27.0_f32), // E6 — 3/5 tiles; arrival cols clear
            (EnemyType::Possum, Vec2::new(col_x(60.0), plat_top(6.0)), 27.0_f32), // E7 — 3/5 tiles; arrival col clear
        ];
        for (etype, pos, patrol) in enemies {
            spawn_enemy(commands, meshes, materials, asset_server, etype, pos, patrol);
        }
    }
}

// ── City (origin_x = -1152, origin_y = -200) ─────────────────────────────────
// Gate: col_x(123) = 1071.0  |  Exit: (1101.0, -137.0) → Sanctuary

pub fn spawn_city_entities(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    asset_server: &AssetServer,
    progress: &mut CollectionProgress,
    skip_enemies: bool,
) {
    const OX: f32 = -1152.0;
    const OY: f32 = -200.0;
    let col_x = |col: f32| OX + col * 18.0 + 9.0;
    let stand_y = |row: f32| OY + (row + 1.0) * 18.0 + 9.0;
    let ground_y = stand_y(2.0);
    let ground_top = OY + 3.0 * 18.0; // -146.0

    let star_positions = [
        Vec3::new(col_x(6.0),   stand_y(4.0),  1.0), // A(row4,4-8): dumpster SC1
        Vec3::new(col_x(14.0),  stand_y(7.0),  1.0), // B(row7,12-17): scaffold SC1
        Vec3::new(col_x(21.0),  stand_y(4.0),  1.0), // C(row4,19-23): step SC1
        Vec3::new(col_x(28.0),  stand_y(7.0),  1.0), // D(row7,26-31): branch point — all routes
        Vec3::new(col_x(46.0),  stand_y(7.0),  1.0), // F-lo(row7,44-49): safe-route reward
        Vec3::new(col_x(63.0),  stand_y(10.0), 1.0), // G-hi(row10,62-66): committed-route reward
        Vec3::new(col_x(65.0),  stand_y(4.0),  1.0), // H(row4,63-68): Branch2 diverge — all routes
        Vec3::new(col_x(81.0),  stand_y(10.0), 1.0), // J(row10,79-84): tower route reward
        Vec3::new(col_x(90.0),  stand_y(4.0),  1.0), // LB2(row4,88-93): bypass route reward
        Vec3::new(col_x(118.0), stand_y(7.0),  1.0), // P(row7,116-121): pre-gate
        // 11th star: ground-level safety valve — makes any one platform star skippable
        Vec3::new(col_x(100.0), ground_y,       1.0), // ground — SC4 (safe gap between Rat and Dog zones)
    ];
    for pos in star_positions {
        spawn_collectible(commands, meshes, materials, asset_server, pos, CollectibleType::Star);
    }
    // Gate opens at 10 of 11: one star is skippable (G-hi or J tower becomes optional)
    progress.stars_total = 10;
    progress.stars_collected = 0;

    let apple_positions = [
        Vec3::new(col_x(13.0),  stand_y(7.0),  1.0), // B(row7,12-17): intro scaffold
        Vec3::new(col_x(74.0),  stand_y(4.0),  1.0), // R-SC3(row4,72-76): tower-fall recovery
        Vec3::new(col_x(110.0), stand_y(7.0),  1.0), // O(row7,108-113): post-tower descent
    ];
    for pos in apple_positions {
        spawn_collectible(commands, meshes, materials, asset_server, pos, CollectibleType::HealthFood);
    }

    let gate_x = col_x(123.0);
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
        LevelExit { next_level: LevelId::Sanctuary, half_extents: Vec2::new(51.0, 100.0) },
    ));

    // End-zone landmark — sign post as suburban directional cue.
    commands.spawn((
        SceneRoot(asset_server.load("models/sign.glb#Scene0")),
        Transform::from_xyz(gate_x + 40.0, ground_top, -1.0)
            .with_scale(Vec3::new(50.0, 50.0, 7.0)),
        components::Decoration,
    ));

    if !skip_enemies {
    // SC1 (1): Dog between A and B — motivates A as stepping stone, doesn't block platforms
    // SC2 (2): PatrolOnly only — entire SC2 ground is under Category A fall zones;
    //          PatrolOnly gives player agency to time falls without compound punishment
    // SC3 (2): Squirrel at H→R-SC3 gap creates urgency at Branch2 decision;
    //          Rat in LB1→LB2 gap adds bypass-route challenge (low-consequence falls)
    // SC4 (1): Dog wide patrol covers O→P approach — final pressure before gate
    let enemies = [
        (EnemyType::Dog,      Vec2::new(col_x(10.0),  ground_top), 54.0_f32), // SC1: A–B gap
        (EnemyType::Snake,    Vec2::new(col_x(50.0),  ground_top), 45.0_f32), // SC2: F-lo→G gap; ±2.5 tiles
        (EnemyType::Possum,   Vec2::new(col_x(58.0),  ground_top), 45.0_f32), // SC2: G-lo convergence; ±2.5 tiles
        (EnemyType::Squirrel, Vec2::new(col_x(70.0),  ground_top), 45.0_f32), // SC3: H→R-SC3 gap; wider arc readable before chase
        (EnemyType::Rat,      Vec2::new(col_x(84.0),  ground_top), 45.0_f32), // SC3: LB1→LB2 gap; pressures bypass route
        (EnemyType::Dog,      Vec2::new(col_x(110.0), ground_top), 72.0_f32), // SC4: O–P approach
    ];
    for (etype, pos, patrol) in enemies {
        spawn_enemy(commands, meshes, materials, asset_server, etype, pos, patrol);
    }
    } // skip_enemies
}

// ── Sanctuary (origin_x = -1440, origin_y = -200) ────────────────────────────
// Gate: col_x(155) = 1359.0  |  Exit: (1389.0, -137.0) → game complete

pub fn spawn_sanctuary_entities(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    asset_server: &AssetServer,
    progress: &mut CollectionProgress,
    skip_enemies: bool,
) {
    const OX: f32 = -1440.0;
    const OY: f32 = -200.0;
    let col_x = |col: f32| OX + col * 18.0 + 9.0;
    let stand_y = |row: f32| OY + (row + 1.0) * 18.0 + 9.0;
    let ground_y = stand_y(2.0);
    let ground_top = OY + 3.0 * 18.0; // -146.0

    let star_positions = [
        Vec3::new(col_x(7.0),   stand_y(4.0),  1.0), // Plat A lily pad
        Vec3::new(col_x(16.0),  stand_y(4.0),  1.0), // Plat B lily pad
        Vec3::new(col_x(3.0),   ground_y,      1.0), // Ground S1
        Vec3::new(col_x(36.0),  stand_y(5.0),  1.0), // Plat D short pillar
        Vec3::new(col_x(41.0),  stand_y(9.0),  1.0), // Plat E tall pillar
        Vec3::new(col_x(57.0),  stand_y(10.0), 1.0), // Plat G high arch
        Vec3::new(col_x(69.0),  stand_y(5.0),  1.0), // Plat H safe entry
        Vec3::new(col_x(76.0),  stand_y(8.0),  1.0), // Plat I mid zone
        Vec3::new(col_x(72.0),  stand_y(12.0), 1.0), // Plat J high vantage
        Vec3::new(col_x(108.0), stand_y(14.0), 1.0), // Plat P tower top
    ];
    for pos in star_positions {
        spawn_collectible(commands, meshes, materials, asset_server, pos, CollectibleType::Star);
    }
    progress.stars_total = star_positions.len() as u32;
    progress.stars_collected = 0;

    let apple_positions = [
        Vec3::new(col_x(23.0),  stand_y(7.0),  1.0), // Plat C stone step
        Vec3::new(col_x(46.0),  stand_y(5.0),  1.0), // Recovery R1
        Vec3::new(col_x(77.0),  stand_y(5.0),  1.0), // Recovery R2
        Vec3::new(col_x(104.0), stand_y(4.0),  1.0), // Recovery R3
        Vec3::new(col_x(112.0), stand_y(8.0),  1.0), // Plat O descent (double compensation)
        Vec3::new(col_x(149.0), stand_y(8.0),  1.0), // Plat S pre-gate
    ];
    for pos in apple_positions {
        spawn_collectible(commands, meshes, materials, asset_server, pos, CollectibleType::HealthFood);
    }

    let gate_x = col_x(155.0);
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

    // Exit triggers game complete (level_index will reach 4 in puzzle/systems.rs).
    commands.spawn((
        Transform::from_xyz(gate_x + 30.0, ground_y, 0.5),
        Visibility::Hidden,
        LevelExit { next_level: LevelId::Forest, half_extents: Vec2::new(51.0, 100.0) }, // next_level irrelevant; game_complete fires
    ));

    // End-zone landmark — treasure chest as journey's-end symbol.
    commands.spawn((
        SceneRoot(asset_server.load("models/chest.glb#Scene0")),
        Transform::from_xyz(gate_x + 40.0, ground_top, -1.0)
            .with_scale(Vec3::new(50.0, 50.0, 7.0)),
        components::Decoration,
    ));

    if !skip_enemies {
    let enemies = [
        (EnemyType::Possum,   Vec2::new(col_x(20.0),  ground_top), 54.0_f32),
        (EnemyType::Rat,      Vec2::new(col_x(50.0),  ground_top), 72.0_f32),
        (EnemyType::Squirrel, Vec2::new(col_x(59.0),  ground_top), 90.0_f32),
        (EnemyType::Rat,      Vec2::new(col_x(70.0),  ground_top), 54.0_f32),
        (EnemyType::Dog,      Vec2::new(col_x(79.0),  ground_top), 72.0_f32),
        (EnemyType::Squirrel, Vec2::new(col_x(88.0),  ground_top), 90.0_f32),
        (EnemyType::Rat,      Vec2::new(col_x(111.0), ground_top), 54.0_f32),
        (EnemyType::Possum,   Vec2::new(col_x(122.0), ground_top), 54.0_f32),
        (EnemyType::Possum,   Vec2::new(col_x(136.0), ground_top), 36.0_f32),
        (EnemyType::Snake,    Vec2::new(col_x(150.0), ground_top), 36.0_f32),
    ];
    for (etype, pos, patrol) in enemies {
        spawn_enemy(commands, meshes, materials, asset_server, etype, pos, patrol);
    }
    } // skip_enemies

    // ── Family reunion ────────────────────────────────────────────────────────
    // Jasper's family waits near the sanctuary gate (cols 135–148, on the ground).
    // Each family member has unique dialogue. Press E within 40 units to talk.
    // Marked Decoration so they despawn on level transitions / new-game.
    let family: &[(&str, f32, &[&str])] = &[
        (
            "models/character-oodi.glb",
            col_x(136.0),
            &[
                "Jasper! You made it!",
                "We've been waiting so long…",
                "I knew you'd find your way home.",
            ],
        ),
        (
            "models/character-ooli.glb",
            col_x(140.0),
            &[
                "Big bro! You came back!",
                "I collected ALL my acorns while you were gone.",
                "Race you to the big oak?",
            ],
        ),
        (
            "models/character-oopi.glb",
            col_x(144.0),
            &[
                "Oh sweetheart, I'm so relieved.",
                "I made your favourite berry pie.",
                "Come inside — you must be exhausted.",
            ],
        ),
        (
            "models/character-oobi.glb",
            col_x(148.0),
            &[
                "Took you long enough, little sibling.",
                "I only had to fight off three dogs while you were away.",
                "Glad you're safe. Don't tell Mom I said that.",
            ],
        ),
    ];

    for &(model, x, lines) in family {
        commands.spawn((
            SceneRoot(asset_server.load(format!("{}#Scene0", model))),
            Transform::from_xyz(x, ground_y, 1.0).with_scale(Vec3::splat(22.0)),
            NpcDialogue::new(lines.to_vec()),
            components::Decoration,
        ));
    }
}

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
            spawn_subdivision_entities(commands, meshes, materials, asset_server, progress, skip_enemies);
        }
        LevelId::City => {
            spawn_city_entities(commands, meshes, materials, asset_server, progress, skip_enemies);
        }
        LevelId::Sanctuary => {
            spawn_sanctuary_entities(commands, meshes, materials, asset_server, progress, skip_enemies);
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
    // Sky, mountains, attenuation plane, clouds — nature biomes only.
    // Subdivision and City have their own complete background systems;
    // calling this for them would produce two conflicting skies, wrong
    // mood elements, and a parallax spread that exceeds the ≤ 0.38 limit.
    if matches!(level_id, LevelId::Forest | LevelId::Sanctuary) {
        spawn_shared_background(commands, meshes, materials, asset_server);
    }

    let col_x = |origin_x: f32, col: f32| origin_x + col * 18.0 + 9.0;
    let ground_top = -146.0_f32;

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
        LevelId::Sanctuary => {
            // Sanctuary spans x=-1440..1440. Spawn at -1395.
            // Far/near backgrounds must start near spawn and spread to cover the full level.
            // Far (factor 0.75): dense dark trees, spaced 360 units from -1800 to 1800.
            let far_xs: &[(f32, &str, f32)] = &[
                (-1800.0, "models/tree_tall_dark.glb", 62.0),
                (-1440.0, "models/tree_cone_dark.glb", 55.0),
                (-1080.0, "models/tree_tall_dark.glb", 68.0),
                ( -720.0, "models/tree_cone_dark.glb", 58.0),
                ( -360.0, "models/tree_tall_dark.glb", 70.0),
                (    0.0, "models/tree_cone_dark.glb", 60.0),
                (  360.0, "models/tree_tall_dark.glb", 65.0),
                (  720.0, "models/tree_cone_dark.glb", 55.0),
                ( 1080.0, "models/tree_tall_dark.glb", 62.0),
                ( 1440.0, "models/tree_cone_dark.glb", 58.0),
                ( 1800.0, "models/tree_tall_dark.glb", 65.0),
            ];
            for &(x, model, scale) in far_xs {
                commands.spawn((
                    SceneRoot(asset_server.load(format!("{}#Scene0", model))),
                    Transform::from_xyz(x, -160.0, -80.0).with_scale(Vec3::new(scale, scale, 12.0)),
                    ParallaxLayer { factor: 0.75 },
                    components::Decoration,
                    ParallaxBackground,
                ));
            }
            // Near (factor 0.9): lush fat trees + mushrooms, spaced 240 units from -1800 to 1800.
            let near_models = ["models/tree_fat.glb", "models/mushroom_red.glb", "models/tree_oak.glb", "models/tree_fat.glb"];
            let near_scales = [105.0_f32, 22.0, 98.0, 110.0];
            let mut nx = -1800.0_f32;
            let mut ni = 0usize;
            while nx <= 1800.0 {
                let model = near_models[ni % near_models.len()];
                let scale = near_scales[ni % near_scales.len()];
                commands.spawn((
                    SceneRoot(asset_server.load(format!("{}#Scene0", model))),
                    Transform::from_xyz(nx, -160.0, -50.0).with_scale(Vec3::new(scale, scale, 6.0)),
                    ParallaxLayer { factor: 0.9 },
                    components::Decoration,
                    ParallaxBackground,
                ));
                nx += 240.0;
                ni += 1;
            }
        }
        LevelId::Subdivision => {
            // ── Sky (z = -100, factor 0.30) ───────────────────────────────────
            // Dull blue-grey: overcast suburban mood, deliberately less vibrant
            // than Forest. 6400 wide covers worst-case camera range for all levels.
            let sub_sky_mesh = meshes.add(Mesh::from(Rectangle::new(6400.0, 1800.0)));
            let sub_sky_mat = materials.add(StandardMaterial {
                base_color: Color::srgb(0.52, 0.60, 0.68),
                alpha_mode: AlphaMode::Opaque,
                unlit: true,
                ..default()
            });
            commands.spawn((
                Mesh3d(sub_sky_mesh),
                MeshMaterial3d(sub_sky_mat),
                Transform::from_xyz(0.0, -50.0, -100.0),
                ParallaxLayer { factor: 0.30 },
                components::Decoration,
            ));

            // ── FAR panorama (ARCHIVED — not spawned) ────────────────────
            // MID fully covers the camera envelope; FAR is never visible.
            // Asset preserved at: backgrounds/subdivision/plates/subdivision_far.png
            // To re-enable: uncomment spawn_subdivision_far() in subdivision_panorama.rs
            //               and import + call it here.
            // spawn_subdivision_far(commands, meshes, materials, asset_server);

            // ── MID panorama (single plate, z=-55) ──────────────────────────
            spawn_subdivision_mid(commands, meshes, materials, asset_server);

            // ── NEAR shadow panels (z=+8, Hollow Knight–style) ──────────────
            spawn_subdivision_near(commands, meshes, materials, asset_server);

            // ── Static parallax background (evaluation) ────────────────────────
            //
            // Replaces ~257 individual procedural entities with 4 mass planes
            // (one per depth tier).  Same Z positions, same parallax factors,
            // same color palette.  Individual architectural detail (roofline
            // segments, chimneys, poles, facades, siding, trees, dumpsters)
            // is collapsed into flat tonal bands.
            //
            // Trade-off: loss of per-element silhouette variation.
            // Gain: ~253 fewer entities, simpler scene graph, faster iteration.
            //
            // Depth tiers preserved:
            //   Far   (z=-75, factor 0.42): dark blue-grey roofline band
            //   Mid   (z=-55, factor 0.52): warm grey-beige facade band
            //   Near  (z=-12, factor 0.60): muted clutter band
            //   Fence (z=-53, factor 0.52): ground-level divider strip

            // Far roofline band — represents the collapsed silhouette of
            // rooflines, chimneys, and power poles as a single mass.
            // Vertical extent covers the old roofline range (y≈-160 to y≈-45).
            let far_band_mat = materials.add(StandardMaterial {
                base_color: Color::srgb(0.27, 0.30, 0.35),
                alpha_mode: AlphaMode::Opaque,
                unlit: true,
                ..default()
            });
            let far_band = meshes.add(Mesh::from(Rectangle::new(4400.0, 120.0)));
            commands.spawn((
                Mesh3d(far_band),
                MeshMaterial3d(far_band_mat.clone()),
                Transform::from_xyz(0.0, -100.0, -75.0),
                ParallaxLayer { factor: 0.42 },
                components::Decoration,
                ParallaxBackground,
            ));
            // Power line wire — retained as a single entity (already was one).
            let wire_mesh = meshes.add(Mesh::from(Rectangle::new(4400.0, 2.0)));
            commands.spawn((
                Mesh3d(wire_mesh),
                MeshMaterial3d(far_band_mat.clone()),
                Transform::from_xyz(0.0, -32.0, -74.2),
                ParallaxLayer { factor: 0.42 },
                components::Decoration,
                ParallaxBackground,
            ));

            // Mid-field facade band — warm grey-beige, represents house
            // facades and garages as a continuous horizontal mass.
            let mid_band_mat = materials.add(StandardMaterial {
                base_color: Color::srgb(0.50, 0.48, 0.44),
                alpha_mode: AlphaMode::Opaque,
                unlit: true,
                ..default()
            });
            let mid_band = meshes.add(Mesh::from(Rectangle::new(4400.0, 70.0)));
            commands.spawn((
                Mesh3d(mid_band),
                MeshMaterial3d(mid_band_mat.clone()),
                Transform::from_xyz(0.0, -130.0, -55.0),
                ParallaxLayer { factor: 0.52 },
                components::Decoration,
                ParallaxBackground,
            ));
            // Siding accent — darker horizontal stripe across the facade band.
            let siding_band_mat = materials.add(StandardMaterial {
                base_color: Color::srgb(0.42, 0.39, 0.36),
                alpha_mode: AlphaMode::Opaque,
                unlit: true,
                ..default()
            });
            let siding_band = meshes.add(Mesh::from(Rectangle::new(4400.0, 4.0)));
            commands.spawn((
                Mesh3d(siding_band),
                MeshMaterial3d(siding_band_mat),
                Transform::from_xyz(0.0, -122.0, -54.8),
                ParallaxLayer { factor: 0.52 },
                components::Decoration,
                ParallaxBackground,
            ));

            // Fence strip + cap — retained as two entities (already were two).
            let fence_mat = materials.add(StandardMaterial {
                base_color: Color::srgb(0.45, 0.43, 0.40),
                alpha_mode: AlphaMode::Opaque,
                unlit: true,
                ..default()
            });
            let fence_strip = meshes.add(Mesh::from(Rectangle::new(4400.0, 20.0)));
            commands.spawn((
                Mesh3d(fence_strip),
                MeshMaterial3d(fence_mat.clone()),
                Transform::from_xyz(0.0, -133.0, -53.0),
                ParallaxLayer { factor: 0.52 },
                components::Decoration,
                ParallaxBackground,
            ));
            let fence_cap_mat = materials.add(StandardMaterial {
                base_color: Color::srgb(0.50, 0.48, 0.44),
                alpha_mode: AlphaMode::Opaque,
                unlit: true,
                ..default()
            });
            let fence_cap = meshes.add(Mesh::from(Rectangle::new(4400.0, 4.0)));
            commands.spawn((
                Mesh3d(fence_cap),
                MeshMaterial3d(fence_cap_mat),
                Transform::from_xyz(0.0, -121.0, -52.8),
                ParallaxLayer { factor: 0.52 },
                components::Decoration,
                ParallaxBackground,
            ));

            // Near clutter band — represents utility poles, trees, dumpsters,
            // broken fencing as a continuous muted band behind gameplay.
            let near_band_mat = materials.add(StandardMaterial {
                base_color: Color::srgb(0.32, 0.34, 0.30),
                alpha_mode: AlphaMode::Opaque,
                unlit: true,
                ..default()
            });
            let near_band = meshes.add(Mesh::from(Rectangle::new(4400.0, 80.0)));
            commands.spawn((
                Mesh3d(near_band),
                MeshMaterial3d(near_band_mat),
                Transform::from_xyz(0.0, -120.0, -12.0),
                ParallaxLayer { factor: 0.60 },
                components::Decoration,
                ParallaxBackground,
            ));
            // ── Foreground house facades (z = +10) ────────────────────────────
            // Cropped rectangular house sides at level edges — the suburban
            // equivalent of Forest's framing trees and City's fire escapes.
            // z=+10: in front of player (z=5), frames viewport edges.
            //
            // Each facade: wide rectangle (house wall) + smaller rectangle
            // (roofline cap). Extends from below ground to above viewport —
            // cropped top and bottom, implying the house continues.
            //
            // Placed at far-left (near spawn) and far-right (near gate).
            // Positions avoid doors (-369, -27) and platform approaches.
            let facade_fg_mat = materials.add(StandardMaterial {
                base_color: Color::srgb(0.38, 0.36, 0.33),
                alpha_mode: AlphaMode::Opaque,
                unlit: true,
                ..default()
            });
            let roof_fg_mat = materials.add(StandardMaterial {
                base_color: Color::srgb(0.30, 0.28, 0.26),
                alpha_mode: AlphaMode::Opaque,
                unlit: true,
                ..default()
            });

            let sub_ox = -864.0_f32;
            // Left facade: 10u past level left edge — left half always off-screen.
            // Right facade: 30u past level right edge — right half always off-screen.
            // Both are horizontally cropped by camera clamp at their respective edges.
            // Scale reference (without adding doors):
            //   Player sprite: 28×32u.  Door ≈ 32×17u (player height × 0.6w).
            //   Wall 60u wide ≈ 3.5 doors across → two-story residential.
            //   Wall 120u tall ≈ 3.75× player height → human, not monumental.
            //   Roofline at ground_top + 105 = jump apex (-56) + 15u margin.
            //   Roofline sits just above what the player can reach.
            let facade_xs = [sub_ox - 10.0, sub_ox + 96.0 * 18.0 + 30.0];
            for &fx in &facade_xs {
                // House wall — extends from below ground to just below roofline
                let wall = meshes.add(Mesh::from(Rectangle::new(60.0, 120.0)));
                commands.spawn((
                    Mesh3d(wall),
                    MeshMaterial3d(facade_fg_mat.clone()),
                    Transform::from_xyz(fx, ground_top + 40.0, 10.0),
                    components::Decoration,
                    components::ForegroundDecoration,
                ));
                // Roofline cap — slightly wider, short, sits just above jump apex
                let roof = meshes.add(Mesh::from(Rectangle::new(68.0, 12.0)));
                commands.spawn((
                    Mesh3d(roof),
                    MeshMaterial3d(roof_fg_mat.clone()),
                    Transform::from_xyz(fx, ground_top + 105.0, 10.0),
                    components::Decoration,
                    components::ForegroundDecoration,
                ));
            }

            // ── Foreground ground props (z = +3) ────────────────────────────
            // Suburban ground dressing: mailboxes, fence posts, garden stones.
            // z=+3: behind player (z=5), in front of tiles (z=0).
            // Small items at ground level, placed between platform columns.
            let mailbox_mat = materials.add(StandardMaterial {
                base_color: Color::srgb(0.32, 0.30, 0.28),
                alpha_mode: AlphaMode::Opaque,
                unlit: true,
                ..default()
            });
            let post_mat = materials.add(StandardMaterial {
                base_color: Color::srgb(0.36, 0.32, 0.27),
                alpha_mode: AlphaMode::Opaque,
                unlit: true,
                ..default()
            });

            let sub_col_x = |col: f32| sub_ox + col * 18.0 + 9.0;

            // Mailboxes — squat rectangles, scattered.
            // Each position verified against layer 0 platform columns.
            let mailbox_cols = [1.0_f32, 38.0, 66.0, 92.0];
            for &col in &mailbox_cols {
                let mesh = meshes.add(Mesh::from(Rectangle::new(5.0, 12.0)));
                commands.spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(mailbox_mat.clone()),
                    Transform::from_xyz(sub_col_x(col), ground_top + 6.0, 3.0),
                    components::Decoration,
                    components::ForegroundDecoration,
                ));
            }

            // Fence posts — thin verticals.
            // Each position verified against layer 0 platform columns.
            let post_cols = [17.0_f32, 40.0, 56.0, 78.0];
            for &col in &post_cols {
                let mesh = meshes.add(Mesh::from(Rectangle::new(3.0, 18.0)));
                commands.spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(post_mat.clone()),
                    Transform::from_xyz(sub_col_x(col), ground_top + 9.0, 3.0),
                    components::Decoration,
                    components::ForegroundDecoration,
                ));
            }
        }
        LevelId::City => {
            let ox = -1152.0_f32;

            // ── Sky (z = -100, factor 0.20) ─────────────────────────────────
            // Cool blue-grey: evening/overcast city mood, matches City lighting
            // theme (directional RGB 0.8/0.85/1.0). Without this, the gap behind
            // far skyscrapers shows raw ClearColor (forest sky blue) which breaks
            // the biome's cool palette.
            let city_sky_mesh = meshes.add(Mesh::from(Rectangle::new(6400.0, 1800.0)));
            let city_sky_mat = materials.add(StandardMaterial {
                base_color: Color::srgb(0.35, 0.40, 0.52),
                alpha_mode: AlphaMode::Opaque,
                unlit: true,
                ..default()
            });
            commands.spawn((
                Mesh3d(city_sky_mesh),
                MeshMaterial3d(city_sky_mat),
                Transform::from_xyz(0.0, -50.0, -100.0),
                ParallaxLayer { factor: 0.20 },
                components::Decoration,
            ));

            // ── Parallax city skyline (z = -80, factor 0.75) ─────────────────
            // City spans x=-1152..1152, spawn at -1107.
            // Coverage: -1700 to +1700 at 90u step (~38 buildings).
            //
            // Base y=140: positions building bases in the upper third of the
            // viewport at ground-level camera.  Tops extend off-screen.
            // Tall buildings intrude downward toward the play space; short
            // buildings sit higher — the downward-pointing sawtooth encloses
            // the scene from above.
            //
            // Building base positions (bottom = y_center - scale/2):
            //   Scale 160: base=60  (upper third, minimal intrusion)
            //   Scale 200: base=40  (upper-third boundary)
            //   Scale 230: base=25  (upper-mid, moderate intrusion)
            //   Scale 260: base=10  (mid viewport)
            //   Scale 300: base=-10 (approaching gameplay band)
            //   Scale 340: base=-30 (strong intrusion, still above ground=-146)
            //
            // All bases on-screen at every camera position:
            //   Ground camera (bottom=-200): base=-30 > -200 ✓
            //   Max clamp   (bottom=-166): base=-30 > -166 ✓
            //
            // All tops off-screen at ground camera (viewport_top=162):
            //   Scale 160: top=220 > 162 ✓  (implies continuation)
            //   Scale 340: top=310 > 162 ✓
            //
            // Sequence [200, 340, 160, 300, 230, 260]:
            //   mid → tall → low → tall → mid → mid.
            // Dedicated skyline silhouette material — flat, unlit, dark.
            // Does NOT reuse any existing building/street GLB.
            // Pure rectangular mass with zero surface detail.
            let skyline_mat = materials.add(StandardMaterial {
                base_color: Color::srgb(0.18, 0.20, 0.25),
                alpha_mode: AlphaMode::Opaque,
                unlit: true,
                ..default()
            });

            // 8 silhouette profiles.  4 flat-top + 4 stepped (base + upper setback).
            // Widths ≥ 95u at 90u step → guaranteed overlap, no sky gaps.
            // Stepped profiles spawn two rectangles: a wide base block and a
            // narrower upper block offset upward, producing an L-shaped contour
            // that breaks flat-top monotony without adding surface detail.
            //
            // Profile format: (base_w, base_h, Option<(upper_w, upper_h, upper_y_offset)>)
            //   None           = flat-top rectangle
            //   Some(w, h, dy) = second rectangle centered at y+dy above base center
            struct SkyProfile {
                base_w: f32,
                base_h: f32,
                upper: Option<(f32, f32, f32)>, // (width, height, y_offset from base center)
            }
            let profiles: &[SkyProfile] = &[
                // 1. Low flat — short wide block
                SkyProfile { base_w: 110.0, base_h: 160.0, upper: None },
                // 2. Tall stepped — wide base with narrow tower
                SkyProfile { base_w: 120.0, base_h: 260.0, upper: Some((60.0, 80.0, 170.0)) },
                // 3. Mid flat — standard block
                SkyProfile { base_w: 100.0, base_h: 200.0, upper: None },
                // 4. Extra-tall stepped — dominant mass with setback
                SkyProfile { base_w: 130.0, base_h: 300.0, upper: Some((70.0, 80.0, 190.0)) },
                // 5. Mid stepped — moderate with small upper
                SkyProfile { base_w: 95.0,  base_h: 230.0, upper: Some((50.0, 60.0, 145.0)) },
                // 6. Tall flat — simple tall slab
                SkyProfile { base_w: 115.0, base_h: 300.0, upper: None },
                // 7. Low stepped — short with rooftop bump
                SkyProfile { base_w: 105.0, base_h: 180.0, upper: Some((45.0, 50.0, 115.0)) },
                // 8. Extra-tall flat — tallest simple block
                SkyProfile { base_w: 120.0, base_h: 340.0, upper: None },
            ];
            let mut fx = -1700.0_f32;
            let mut fi = 0usize;
            while fx <= 1700.0 {
                let p = &profiles[fi % profiles.len()];
                // Base block
                let base_mesh = meshes.add(Mesh::from(Rectangle::new(p.base_w, p.base_h)));
                commands.spawn((
                    Mesh3d(base_mesh),
                    MeshMaterial3d(skyline_mat.clone()),
                    Transform::from_xyz(fx, 140.0, -80.0),
                    ParallaxLayer { factor: 0.50 },
                    components::Decoration,
                    ParallaxBackground,
                ));
                // Upper setback (stepped profiles only)
                if let Some((uw, uh, dy)) = p.upper {
                    let upper_mesh = meshes.add(Mesh::from(Rectangle::new(uw, uh)));
                    commands.spawn((
                        Mesh3d(upper_mesh),
                        MeshMaterial3d(skyline_mat.clone()),
                        Transform::from_xyz(fx, 140.0 + dy, -80.0),
                        ParallaxLayer { factor: 0.50 },
                        components::Decoration,
                        ParallaxBackground,
                    ));
                }
                fx += 90.0;
                fi += 1;
            }

            // ── Rooftop details (z = -79, factor 0.75) ──────────────────────
            // Antennas, water towers, HVAC units — thin shapes above the skyline.
            // Placed every 200u (offset +50 from buildings) so they don't all
            // sit on the same building. z=-79 renders just in front of buildings.
            let detail_mat = materials.add(StandardMaterial {
                base_color: Color::srgb(0.22, 0.24, 0.28),
                alpha_mode: AlphaMode::Opaque,
                unlit: true,
                ..default()
            });

            // Details sit near mid-tier building bases (y≈25 for scale 230).
            // Placed on the LOWER edges of buildings where they intrude into
            // the scene — this is where the player's eye is, so details are
            // visible rather than lost off-screen above.
            let detail_configs: &[(f32, f32, f32, f32)] = &[
                // (width, height, x_offset_from_step, y_base)
                // Antennas — tall thin verticals hanging from building edge
                (3.0, 45.0, 0.0,  35.0),
                // Water towers — squat rectangles on building face
                (18.0, 14.0, 8.0, 20.0),
                // HVAC boxes — small squares near base
                (14.0, 10.0, -5.0, 8.0),
            ];
            let mut dx = -1650.0_f32;
            let mut di = 0usize;
            while dx <= 1700.0 {
                let (dw, dh, x_off, y_base) = detail_configs[di % detail_configs.len()];
                let mesh = meshes.add(Mesh::from(Rectangle::new(dw, dh)));
                commands.spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(detail_mat.clone()),
                    Transform::from_xyz(dx + x_off, y_base + dh * 0.5, -79.0),
                    ParallaxLayer { factor: 0.50 },
                    components::Decoration,
                    ParallaxBackground,
                ));
                dx += 200.0;
                di += 1;
            }
            // ── Mid-ground mass planes (z = -52, factor 0.65) ───────────────
            // Continuous dark rectangles behind the GLB building faces.
            // Fills the gaps between individual buildings (160u step, 75-110u
            // wide models = 50-85u gaps) so the mid-ground reads as a solid
            // urban wall rather than spaced buildings with skyline showing
            // through.  z=-52 sits behind the GLB buildings (z=-50) but in
            // front of the skyline (z=-80).
            //
            // Factor 0.65: between skyline (0.50) and mid-ground details
            // (0.90). The 0.25 gap from mid-ground creates subtle depth —
            // building details scroll faster over the mass behind them.
            // The mass plane is featureless (no edges, no texture within
            // the viewport) so the relative drift is invisible — it only
            // produces a depth cue, not a visible sliding artifact.
            //
            // Two tiers: a tall backdrop spanning the full building band,
            // and a shorter ground-level fill closing the bottom.
            let mass_mat = materials.add(StandardMaterial {
                base_color: Color::srgb(0.22, 0.24, 0.28),
                alpha_mode: AlphaMode::Opaque,
                unlit: true,
                ..default()
            });

            // Tall mass band: continuous wall from below ground to above
            // mid-ground building tops.  4400u wide: accounts for parallax
            // drift differential (mass at 0.65, windows at 0.90).  At the
            // extreme camera position (x≈900), the drift difference is
            // 900*(0.90-0.65) = 225u.  4400u provides 300u+ margin on each
            // side after drift, ensuring no window floats outside the mass.
            let tall_mass = meshes.add(Mesh::from(Rectangle::new(4400.0, 260.0)));
            commands.spawn((
                Mesh3d(tall_mass),
                MeshMaterial3d(mass_mat.clone()),
                Transform::from_xyz(0.0, -160.0 + 130.0, -52.0),
                ParallaxLayer { factor: 0.65 },
                components::Decoration,
                ParallaxBackground,
            ));

            // Ground-level fill: shorter, slightly brighter rectangle
            // closing the gap between building bases and the ground plane.
            // Prevents skyline from peeking through at the building/ground seam.
            let ground_fill_mat = materials.add(StandardMaterial {
                base_color: Color::srgb(0.24, 0.26, 0.30),
                alpha_mode: AlphaMode::Opaque,
                unlit: true,
                ..default()
            });
            let ground_fill = meshes.add(Mesh::from(Rectangle::new(4400.0, 60.0)));
            commands.spawn((
                Mesh3d(ground_fill),
                MeshMaterial3d(ground_fill_mat),
                Transform::from_xyz(0.0, -190.0, -51.0),
                ParallaxLayer { factor: 0.65 },
                components::Decoration,
                ParallaxBackground,
            ));

            // ── Mid-ground building faces (z = -50, factor 0.9) ────────────
            // Large building masses providing scale. 3 GLB models cycle at
            // 160u spacing with 6 height tiers.
            let near_models = ["models/city/building-a.glb", "models/city/building-c.glb", "models/city/building-e.glb"];
            let near_scales = [85.0_f32, 100.0, 75.0, 110.0, 88.0, 95.0];
            let mut nx = -1700.0_f32;
            let mut ni = 0usize;
            while nx <= 1700.0 {
                let model = near_models[ni % near_models.len()];
                let scale = near_scales[ni % near_scales.len()];
                commands.spawn((
                    SceneRoot(asset_server.load(format!("{}#Scene0", model))),
                    Transform::from_xyz(nx, -160.0, -50.0).with_scale(Vec3::new(scale, scale, 6.0)),
                    ParallaxLayer { factor: 0.9 },
                    components::Decoration,
                    ParallaxBackground,
                ));
                nx += 160.0;
                ni += 1;
            }

            // ── Building face details (z = -49, factor 0.9) ─────────────────
            // Window grids, fire escapes, signage — flat rectangles on the
            // building surfaces. Same parallax factor so they track with the
            // buildings. Purely cosmetic scale cues, no doors or entrances.
            let window_mat = materials.add(StandardMaterial {
                base_color: Color::srgb(0.25, 0.28, 0.35),
                alpha_mode: AlphaMode::Opaque,
                unlit: true,
                ..default()
            });
            let escape_mat = materials.add(StandardMaterial {
                base_color: Color::srgb(0.20, 0.20, 0.22),
                alpha_mode: AlphaMode::Opaque,
                unlit: true,
                ..default()
            });
            let sign_mat = materials.add(StandardMaterial {
                base_color: Color::srgb(0.45, 0.30, 0.25),
                alpha_mode: AlphaMode::Opaque,
                unlit: true,
                ..default()
            });

            // Window grids — 4×3 block of small rectangles per building face.
            // Placed every 160u (matching building step) offset +20.
            let mut wx = -1680.0_f32;
            let mut wi = 0usize;
            let grid_y_bases = [-130.0_f32, -120.0, -125.0, -115.0, -128.0, -118.0];
            while wx <= 1700.0 {
                let base_y = grid_y_bases[wi % grid_y_bases.len()];
                for row in 0..3 {
                    for col in 0..4 {
                        let wm = meshes.add(Mesh::from(Rectangle::new(8.0, 6.0)));
                        commands.spawn((
                            Mesh3d(wm),
                            MeshMaterial3d(window_mat.clone()),
                            Transform::from_xyz(
                                wx + col as f32 * 12.0 - 18.0,
                                base_y + row as f32 * 10.0,
                                -49.0,
                            ),
                            ParallaxLayer { factor: 0.9 },
                            components::Decoration,
                            ParallaxBackground,
                        ));
                    }
                }
                wx += 160.0;
                wi += 1;
            }

            // ── Lit window ambience (z = -48.9, factor 0.9) ─────────────────
            // Static warm rectangles overlaid on a subset of dark windows.
            // ~30% of windows lit per building face — suggests evening
            // occupancy without uniform brightness. Placed at z=-48.9
            // (just in front of dark windows at z=-49) so they overdraw
            // the dark fill cleanly. Same 8×6u size, same grid positions.
            //
            // Value: 0.45 avg — brighter than dark windows (0.29) and
            // mid-ground buildings (~0.35), but well below tiles (0.55)
            // and player (0.85). Warm amber tint reinforces artificial
            // interior light against the City's cool exterior palette.
            // Warm off-white: R-B spread of 0.10 — just enough warmth to
            // read as interior light against the cool building palette,
            // without theatrical amber or neon saturation.
            //
            // 3 alpha tiers for brightness micro-variance (±8%):
            //   dim=0.75, standard=0.85, bright=0.95
            // Same RGB — no color shift. Blended values:
            //   dim ~0.39, standard ~0.41, bright ~0.42
            // Difference is 0.03 (perceptible only on close inspection).
            let mut lit_base = |alpha: f32| {
                materials.add(StandardMaterial {
                    base_color: Color::srgba(0.48, 0.44, 0.38, alpha),
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    double_sided: true,
                    cull_mode: None,
                    ..default()
                })
            };
            let lit_mats = [
                lit_base(0.75),  // dim
                lit_base(0.85),  // standard
                lit_base(0.95),  // bright
            ];

            // Deterministic lit-window pattern: 8 masks (one per building
            // in each 8-building cycle). Each mask is a 12-bit pattern
            // (4 cols × 3 rows, row-major) where 1 = lit.
            //
            // Lit counts range 2–7 (17–58%) for organic occupancy variance.
            // Some faces are nearly dark (late-night empty office), some
            // are half-lit (occupied residential), creating the uneven
            // glow pattern of a real city block.
            //
            // 8-mask cycle at 160u step = 1280u repeat (> 2× viewport).
            // No two adjacent faces share a mask.
            //
            // Grid reference (bit = row*4 + col):
            //   row 0: bits 0,1,2,3
            //   row 1: bits 4,5,6,7
            //   row 2: bits 8,9,10,11
            // Lit counts: 5,5,6,7,5,6,5,5 = 44/96 = 45.8% average.
            // Range: 5–7 (42–58%). All within 35–60%.
            // No face fully dark or fully lit.
            // Variance: 3 tiers (5=standard, 6=active, 7=busy).
            let lit_masks: &[u16] = &[
                //  col: 0 1 2 3       row0        row1        row2
                0b_0010_0101_1001, //  A: ■ . . ■ | ■ . ■ . | . ■ . .  = 5/12  scattered
                0b_1001_1010_0100, //  B: . . ■ . | . ■ . ■ | ■ . . ■  = 5/12  edges
                0b_0101_0011_1010, //  C: . ■ . ■ | ■ ■ . . | ■ . ■ .  = 6/12  left-heavy
                0b_1110_0110_1001, //  D: ■ . . ■ | . ■ ■ . | . ■ ■ ■  = 7/12  busy floor
                0b_1001_1000_0110, //  E: . ■ ■ . | . . . ■ | ■ . . ■  = 5/12  bottom-right
                0b_1010_0101_0101, //  F: ■ . ■ . | ■ . ■ . | . ■ . ■  = 6/12  checkerboard
                0b_0110_1001_1000, //  G: . . . ■ | ■ . . ■ | . ■ ■ .  = 5/12  right-weighted
                0b_0011_0110_1001, //  H: ■ . . ■ | . ■ ■ . | ■ ■ . .  = 5/12  cluster left
            ];
            let mut lx = -1680.0_f32;
            let mut li = 0usize;
            let lit_y_bases = &grid_y_bases;
            while lx <= 1700.0 {
                let base_y = lit_y_bases[li % lit_y_bases.len()];
                let mask = lit_masks[li % lit_masks.len()];
                for row in 0..3 {
                    for col in 0..4 {
                        let bit = row * 4 + col;
                        if mask & (1 << bit) != 0 {
                            // Brightness tier: deterministic from grid position
                            // + building index. Produces irregular dim/standard/bright
                            // distribution with no visible pattern.
                            let tier = (bit + li) % 3;
                            let lm = meshes.add(Mesh::from(Rectangle::new(8.0, 6.0)));
                            commands.spawn((
                                Mesh3d(lm),
                                MeshMaterial3d(lit_mats[tier].clone()),
                                Transform::from_xyz(
                                    lx + col as f32 * 12.0 - 18.0,
                                    base_y + row as f32 * 10.0,
                                    -48.9,
                                ),
                                ParallaxLayer { factor: 0.9 },
                                components::Decoration,
                                ParallaxBackground,
                            ));
                        }
                    }
                }
                lx += 160.0;
                li += 1;
            }

            // Fire escapes — narrow vertical zigzag strips, every 320u.
            // Two segments per escape: vertical rail + diagonal brace.
            let mut ex = -1600.0_f32;
            while ex <= 1700.0 {
                // Vertical rail
                let rail = meshes.add(Mesh::from(Rectangle::new(3.0, 50.0)));
                commands.spawn((
                    Mesh3d(rail),
                    MeshMaterial3d(escape_mat.clone()),
                    Transform::from_xyz(ex, -130.0, -48.8),
                    ParallaxLayer { factor: 0.9 },
                    components::Decoration,
                    ParallaxBackground,
                ));
                // Landing platforms — 3 short horizontals
                for &ly in &[-145.0_f32, -130.0, -115.0] {
                    let landing = meshes.add(Mesh::from(Rectangle::new(14.0, 2.5)));
                    commands.spawn((
                        Mesh3d(landing),
                        MeshMaterial3d(escape_mat.clone()),
                        Transform::from_xyz(ex + 7.0, ly, -48.8),
                        ParallaxLayer { factor: 0.9 },
                        components::Decoration,
                        ParallaxBackground,
                    ));
                }
                ex += 320.0;
            }

            // Signage blocks — wide flat rectangles at street level, every 480u.
            // Muted warm tone — reads as shop fronts or billboards, not doors.
            let mut sx = -1500.0_f32;
            let sign_widths = [36.0_f32, 28.0, 42.0, 32.0];
            let mut si = 0usize;
            while sx <= 1700.0 {
                let sw = sign_widths[si % sign_widths.len()];
                let sm = meshes.add(Mesh::from(Rectangle::new(sw, 10.0)));
                commands.spawn((
                    Mesh3d(sm),
                    MeshMaterial3d(sign_mat.clone()),
                    Transform::from_xyz(sx, -148.0, -48.5),
                    ParallaxLayer { factor: 0.9 },
                    components::Decoration,
                    ParallaxBackground,
                ));
                sx += 480.0;
                si += 1;
            }

            // ── Foreground ground-level props (z = +3) ─────────────────────────
            // Urban ground dressing: lamp posts, bollards, grate covers.
            // z=+3: in front of tiles (z=0) and enemies (z=0.5) but behind
            // player (z=5). Small items at ground level — no vertical extent
            // that could overlap enemy sprites or obscure platform edges.
            // World-space static (no ParallaxLayer) — moves with gameplay.
            // Placed in ground gaps between platform columns.
            let lamp_mat = materials.add(StandardMaterial {
                base_color: Color::srgb(0.25, 0.25, 0.28),
                alpha_mode: AlphaMode::Opaque,
                unlit: true,
                ..default()
            });
            let bollard_mat = materials.add(StandardMaterial {
                base_color: Color::srgb(0.35, 0.32, 0.28),
                alpha_mode: AlphaMode::Opaque,
                unlit: true,
                ..default()
            });
            let grate_mat = materials.add(StandardMaterial {
                base_color: Color::srgb(0.22, 0.22, 0.24),
                alpha_mode: AlphaMode::Opaque,
                unlit: true,
                ..default()
            });

            // Lamp posts — thin vertical, 4×28u, ground-anchored.
            // Placed between platform clusters. Each position verified
            // against row 4/7/10 platform columns to avoid vertical tangents.
            let lamp_cols = [10.0_f32, 24.0, 42.0, 70.0, 86.0, 115.0];
            for &col in &lamp_cols {
                let mesh = meshes.add(Mesh::from(Rectangle::new(4.0, 28.0)));
                commands.spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(lamp_mat.clone()),
                    Transform::from_xyz(col_x(ox, col), ground_top + 14.0, 3.0),
                    components::Decoration,
                    components::ForegroundDecoration,
                ));
            }

            // Bollards — short thick vertical, 6×10u.
            // Clusters of 2-3, placed in open ground zones between platforms.
            // Each position verified against row 4/7 platform columns.
            let bollard_positions: &[(f32, f32)] = &[
                (col_x(ox, 1.0),  ground_top + 5.0),  // left of row 4 plat (4-8)
                (col_x(ox, 2.0),  ground_top + 5.0),
                (col_x(ox, 41.0), ground_top + 5.0),  // left of row 7 plat (44-49)
                (col_x(ox, 42.0), ground_top + 5.0),
                (col_x(ox, 98.0), ground_top + 5.0),  // right of row 7 plat (92-97)
                (col_x(ox, 99.0), ground_top + 5.0),
                (col_x(ox, 100.0),ground_top + 5.0),
            ];
            for &(bx, by) in bollard_positions {
                let mesh = meshes.add(Mesh::from(Rectangle::new(6.0, 10.0)));
                commands.spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(bollard_mat.clone()),
                    Transform::from_xyz(bx, by, 3.0),
                    components::Decoration,
                    components::ForegroundDecoration,
                ));
            }

            // Grate covers — short wide horizontal, 14×3u, flush with ground.
            // Positioned between platform columns to avoid vertical tangents.
            let grate_cols = [11.0_f32, 50.0, 69.0, 107.0, 114.0];
            for &col in &grate_cols {
                let mesh = meshes.add(Mesh::from(Rectangle::new(14.0, 3.0)));
                commands.spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(grate_mat.clone()),
                    Transform::from_xyz(col_x(ox, col), ground_top + 1.5, 3.0),
                    components::Decoration,
                    components::ForegroundDecoration,
                ));
            }

            // ── Foreground fire-escape motif (z = +10) ─────────────────────
            // Iconic City motif: cropped fire-escape structures at level
            // edges, urban counterpart to Forest's framing trees.  z=+10
            // places them in front of the player (z=5) — they bookend the
            // viewport where the player rarely stands.
            //
            // Each fire escape: two vertical rails + 4 staggered landings
            // + 3 diagonal braces connecting landings.  Extends from below
            // ground to above viewport — cropped top and bottom to imply a
            // larger building the structure is attached to.
            //
            // Left pair near spawn, right pair near gate area.
            // Positions avoid doors (-639, -153) and platform approaches.
            let escape_fg_mat = materials.add(StandardMaterial {
                base_color: Color::srgb(0.18, 0.18, 0.22),
                alpha_mode: AlphaMode::Opaque,
                unlit: true,
                ..default()
            });

            // Left escape at x=-1130 (col ~0.7, far from all platforms).
            // Right escape at x=1060 (col ~123, right of last platform
            // cluster at cols 116-121 by 2+ cols, near gate at col 123).
            //
            // Landing heights deliberately offset from platform stand_y:
            //   row 4=-101, row 7=-47, row 9=-11, row 10=7
            // Landings at -120, -75, -32, +15 — minimum 13u gap from
            // nearest platform height. No landing reads as reachable.
            let escape_xs = [-1130.0_f32, 1060.0];
            for &ex in &escape_xs {
                let rail_h = 200.0_f32;
                let rail_w = 5.0_f32;
                let rail_spacing = 28.0_f32;
                let rail_cy = ground_top + rail_h * 0.5 - 20.0;

                // Two vertical rails — thick industrial verticals
                for &rx in &[ex - rail_spacing * 0.5, ex + rail_spacing * 0.5] {
                    let mesh = meshes.add(Mesh::from(Rectangle::new(rail_w, rail_h)));
                    commands.spawn((
                        Mesh3d(mesh),
                        MeshMaterial3d(escape_fg_mat.clone()),
                        Transform::from_xyz(rx, rail_cy, 10.0),
                        components::Decoration,
                        components::ForegroundDecoration,
                    ));
                }

                // 4 staggered landings — chunky horizontal grating
                // Heights offset from all platform stand_y values.
                let landing_ys = [-120.0_f32, -75.0, -32.0, 15.0];
                for &ly in &landing_ys {
                    let mesh = meshes.add(Mesh::from(Rectangle::new(rail_spacing + 8.0, 5.0)));
                    commands.spawn((
                        Mesh3d(mesh),
                        MeshMaterial3d(escape_fg_mat.clone()),
                        Transform::from_xyz(ex, ly, 10.0),
                        components::Decoration,
                        components::ForegroundDecoration,
                    ));
                }

                // 3 diagonal braces — thick zigzag connectors
                for i in 0..3 {
                    let brace_cy = (landing_ys[i] + landing_ys[i + 1]) * 0.5;
                    let brace_h = 48.0_f32;
                    let mesh = meshes.add(Mesh::from(Rectangle::new(4.0, brace_h)));
                    let angle = if i % 2 == 0 { 0.6_f32 } else { -0.6 };
                    commands.spawn((
                        Mesh3d(mesh),
                        MeshMaterial3d(escape_fg_mat.clone()),
                        Transform::from_xyz(ex, brace_cy, 10.0)
                            .with_rotation(Quat::from_rotation_z(angle)),
                        components::Decoration,
                        components::ForegroundDecoration,
                    ));
                }
            }

            // ── Street-level city props ────────────────────────────────────────
            // Static buildings aligned to level columns, thin Z so they stay behind gameplay.
            let buildings: &[(&str, f32, f32)] = &[
                ("models/city/building-skyscraper-a.glb", col_x(ox, 5.0),   90.0),
                ("models/city/building-c.glb",            col_x(ox, 18.0),  60.0),
                ("models/city/building-skyscraper-c.glb", col_x(ox, 32.0),  95.0),
                ("models/city/building-e.glb",            col_x(ox, 48.0),  62.0),
                ("models/city/building-skyscraper-a.glb", col_x(ox, 62.0),  85.0),
                ("models/city/building-a.glb",            col_x(ox, 78.0),  58.0),
                ("models/city/building-skyscraper-c.glb", col_x(ox, 92.0),  90.0),
                ("models/city/building-e.glb",            col_x(ox, 108.0), 62.0),
                ("models/city/building-skyscraper-a.glb", col_x(ox, 120.0), 88.0),
            ];
            for &(model, x, scale) in buildings {
                commands.spawn((
                    SceneRoot(asset_server.load(format!("{}#Scene0", model))),
                    Transform::from_xyz(x, ground_top, -20.0).with_scale(Vec3::new(scale, scale, 1.0)),
                    components::Decoration,
                ));
            }
            // Street props: awnings, parasols, taxis.
            let props: &[(&str, f32, f32, f32)] = &[
                ("models/city/detail-awning.glb",    col_x(ox, 15.0),  ground_top, 18.0),
                ("models/city/detail-parasol-a.glb", col_x(ox, 35.0),  ground_top, 16.0),
                ("models/city/detail-awning.glb",    col_x(ox, 60.0),  ground_top, 18.0),
                ("models/city/detail-parasol-a.glb", col_x(ox, 85.0),  ground_top, 16.0),
                ("models/city/detail-awning.glb",    col_x(ox, 108.0), ground_top, 18.0),
                ("models/city/taxi.glb",             col_x(ox, 25.0),  ground_top, 14.0),
                ("models/city/taxi.glb",             col_x(ox, 72.0),  ground_top, 14.0),
                ("models/city/taxi.glb",             col_x(ox, 105.0), ground_top, 14.0),
            ];
            for &(model, x, y, scale) in props {
                commands.spawn((
                    SceneRoot(asset_server.load(format!("{}#Scene0", model))),
                    Transform::from_xyz(x, y, -5.0).with_scale(Vec3::new(scale, scale, 1.0)),
                    components::Decoration,
                ));
            }
        }
    }
}

/// Resets the entire game back to Forest level layer 0 when NewGameRequested is set.
#[allow(clippy::too_many_arguments)]
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
        LevelId::Forest      => forest_level(),
        LevelId::Subdivision => crate::level::subdivision::subdivision_level(),
        LevelId::City        => crate::level::city::city_level(),
        LevelId::Sanctuary   => crate::level::sanctuary::sanctuary_level(),
    };

    let layer_index = layer_index.min(level_data.layers.len().saturating_sub(1));
    let layer = &level_data.layers[layer_index];
    let origin = Vec2::new(
        layer.origin_x + TILE_SIZE * 0.5,
        layer.origin_y + TILE_SIZE * 0.5,
    );
    let spawn = layer.spawn;
    let tiles = layer.tiles.clone();

    let solid_model = match level_id {
        LevelId::Subdivision | LevelId::City => "models/brick.glb",
        _ => "models/block-grass-large.glb",
    };

    current_level.level_id    = Some(level_id);
    current_level.layer_index = layer_index;

    spawn_tilemap(commands, asset_server, solid_model, &tiles, origin, 0.0);
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
