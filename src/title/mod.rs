// ── Title Screen 3D Background ────────────────────────────────────────────────
//
// Spawns a non-interactive forest diorama behind the title text.
// All entities carry `TitleSceneEntity` (lifetime) and are despawned on exit.
// The UI text layer (Node/Text) renders above this automatically because Bevy
// always draws UI on top of 3D cameras.
//
// ── CAMERA COUNT CONTRACT ─────────────────────────────────────────────────────
//
//   Per jasper_title_screen_isolation_guardrail: exactly 1 active camera per
//   AppState is required.
//
//   During TitleScreen:  title camera (TitleCamera)        active   = 1 ✓
//                        gameplay camera (GameplayCamera)  inactive = 0 ✓
//
//   This plugin manages the gameplay camera's is_active flag:
//     OnEnter(TitleScreen) → gameplay camera is_active = false
//     OnExit(TitleScreen)  → gameplay camera is_active = true
//
//   The gameplay camera is spawned is_active=false in camera.rs so this rule
//   holds from the very first frame without an explicit Startup disable.
//
// ── DESIGN: Why `order: 1` instead of RenderLayers isolation ─────────────────
//
//   The cleanest isolation would be to assign a dedicated RenderLayers (e.g.
//   layer 2) to both the title camera and all title entities.  This was
//   rejected because SceneRoot child meshes do NOT inherit the parent entity's
//   RenderLayers component in Bevy 0.18 — only the root entity gets the
//   component.  Making layer isolation work would require a post-spawn system
//   that walks the spawned hierarchy and stamps every child with the right
//   layer, which is fragile and couples this module to Bevy's internal scene
//   loading lifecycle.
//
//   Chosen approach: title camera `order: 1` renders on top of the gameplay
//   camera (order: 0, is_active: false during TitleScreen → renders nothing).
//   The sky-blue ClearColor from the global ClearColor resource serves as the
//   backdrop; forest entities layer over it progressively as they load.
//
//   WARNING: if gameplay entities ever exist during TitleScreen again, ensure
//   the gameplay camera is still is_active=false.  The correct fix is always
//   to keep TitleScreen free of gameplay entities, not to fight with cameras.
//
// ── COORDINATE NOTES ─────────────────────────────────────────────────────────
//
//   Title-scene coordinates are self-contained within this file.
//   They share no relationship with the gameplay world origin.
//   All Y positions are derived from GROUND_Y and TILE_SIZE_WORLD below.

use bevy::camera::ScalingMode;
use bevy::prelude::*;
use bevy::prelude::ClearColorConfig;

use crate::rendering::camera::GameplayCamera;
use crate::states::AppState;

// ── Constants ─────────────────────────────────────────────────────────────────

// Tile size in world units — matches gameplay TILE_SIZE (18.0) so ground blocks
// use the same unit grid.
const TILE_SIZE_WORLD: f32 = 18.0;

// Y position of the top surface of the ground tile layer.
// Empirical: at camera y = -50 and -14° tilt, this places ground in the lower
// third of the viewport, leaving the upper two-thirds for sky and trees.
const GROUND_Y: f32 = -72.0;

// Raccoon stands on top of the ground block row.
// Derived: GROUND_Y + TILE_SIZE_WORLD = -72.0 + 18.0 = -54.0
// WARNING: if GROUND_Y changes, update this derivation too.
const RACCOON_Y: f32 = GROUND_Y + TILE_SIZE_WORLD; // -54.0

// ── Markers ───────────────────────────────────────────────────────────────────

/// Lifetime marker — every entity spawned by spawn_title_scene carries this.
/// despawn_title_scene queries it to clean up all title entities on state exit.
/// INVARIANT: any entity missing this tag will leak into MainMenu / Playing.
#[derive(Component)]
pub struct TitleSceneEntity;

/// Role marker for the title camera (per jasper_camera_role_identity_guardrail).
/// Must appear on the title camera AND be used in any query that targets it.
/// WHAT BREAKS: removing this makes the title camera unqueryable by role,
/// violating the "no bare Camera3d queries" rule.
#[derive(Component)]
pub struct TitleCamera;

// ── Plugin ────────────────────────────────────────────────────────────────────

pub struct TitleBackgroundPlugin;

impl Plugin for TitleBackgroundPlugin {
    fn build(&self, app: &mut App) {
        app
            // Deactivate gameplay camera so only 1 camera is active during TitleScreen.
            .add_systems(OnEnter(AppState::TitleScreen), (disable_gameplay_camera, spawn_title_scene))
            // Re-activate gameplay camera before any other state can render.
            .add_systems(OnExit(AppState::TitleScreen), (enable_gameplay_camera, despawn_title_scene));
    }
}

// ── Camera lifecycle ──────────────────────────────────────────────────────────

/// Sets the gameplay camera inactive while TitleScreen is displayed.
/// WHY: jasper_title_screen_isolation_guardrail requires exactly 1 active
/// camera per AppState.  Deactivating (not despawning) preserves the entity
/// so all gameplay systems that query GameplayCamera still get a valid result
/// without needing to handle its absence.
fn disable_gameplay_camera(mut q: Query<&mut Camera, With<GameplayCamera>>) {
    if let Ok(mut cam) = q.single_mut() {
        cam.is_active = false;
    }
}

/// Re-activates the gameplay camera when leaving TitleScreen.
/// Must run in OnExit so the camera is live before any other state's systems
/// that depend on it (camera_follow, camera_clamp, update_parallax).
fn enable_gameplay_camera(mut q: Query<&mut Camera, With<GameplayCamera>>) {
    if let Ok(mut cam) = q.single_mut() {
        cam.is_active = true;
    }
}

// ── Despawn ───────────────────────────────────────────────────────────────────

fn despawn_title_scene(mut commands: Commands, q: Query<Entity, With<TitleSceneEntity>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

// ── Spawn ─────────────────────────────────────────────────────────────────────

fn spawn_title_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    // ── Dedicated camera ─────────────────────────────────────────────────────
    // TitleCamera = role marker (jasper_camera_role_identity_guardrail).
    // TitleSceneEntity = lifetime marker (despawned on OnExit TitleScreen).
    //
    // order: 1 — renders AFTER the gameplay camera (order: 0, inactive).
    // See module-level comment for why RenderLayers are NOT used here.
    //
    // clear_color: None
    //   WHY: Default would clear to sky-blue before forest entities render.
    //   SceneRoot assets load asynchronously (1-3 frame delay); clearing without
    //   geometry produces a sky-blue flash.  None preserves the sky-blue from
    //   the global ClearColor; forest entities layer over it as they load.
    //   NOTE: depth buffer is still cleared by Camera3d::default() (depth_load_op:
    //   Clear), so title scene entities correctly depth-test against each other.
    //
    // viewport_height: 280.0
    //   WHY not 320 (gameplay value): slightly smaller height shows more scene
    //   width, giving a panoramic title feel rather than the tighter gameplay view.
    //
    // Transform y = -50.0
    //   Centres the camera between the raccoon (RACCOON_Y = -54) and the sky.
    //   z = 100.0 matches the gameplay camera's canonical Z (same near/far range).
    //
    // Rotation -14°
    //   WHY not -28° (gameplay tilt): shallower tilt gives a more frontal,
    //   cinematic composition for the title shot.  The gameplay -28° tilt is
    //   intentionally steeper for top-down platformer readability, which would
    //   look wrong on a static title card.
    commands.spawn((
        Camera3d::default(),
        Camera {
            order: 1,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        TitleCamera,
        TitleSceneEntity,
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 280.0,
            },
            near: -1000.0,
            far: 1000.0,
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_xyz(0.0, -50.0, 100.0)
            .with_rotation(Quat::from_rotation_x(-14_f32.to_radians())),
        Msaa::Off,
    ));

    // ── Lights ───────────────────────────────────────────────────────────────
    // Warm golden key light — late-afternoon sun from upper-right.
    // 12 000 lux matches the gameplay primary sun (15 000) minus ambient haze.
    commands.spawn((
        DirectionalLight {
            illuminance: 12000.0,
            color: Color::srgb(1.0, 0.92, 0.70),
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(80.0, 300.0, 150.0).looking_at(Vec3::ZERO, Vec3::Y),
        TitleSceneEntity,
    ));

    // Cool fill light from the opposite side — lifts shadow detail on faces.
    // 2 500 lux: low enough that the warm key dominates the mood.
    commands.spawn((
        DirectionalLight {
            illuminance: 2500.0,
            color: Color::srgb(0.65, 0.80, 1.0),
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(-80.0, -100.0, 200.0).looking_at(Vec3::ZERO, Vec3::Y),
        TitleSceneEntity,
    ));

    // ── Ground tiles (6 × 2 grass patch) ─────────────────────────────────────
    // Scale convention: Vec3::new(xy_scale, xy_scale, thin_z).
    // See canonical decision #2 — GLB models must use this form, NOT Vec3::splat.
    // thin_z (0.3) keeps the block visually thin while xy_scale (1.2) gives
    // them a slight footprint boost over the raccoon's tile-sized platform.
    //
    // Position formula:
    //   x = col * TILE_SIZE_WORLD + TILE_SIZE_WORLD/2   (centres block on column)
    //   y = GROUND_Y - row * TILE_SIZE_WORLD             (row 0 at top, row 1 below)
    //   z = -3.0  (slightly behind raccoon at z=0)
    for row in 0..2i32 {
        for col in -3..3i32 {
            commands.spawn((
                SceneRoot(asset_server.load("models/block-grass-large.glb#Scene0")),
                Transform {
                    translation: Vec3::new(
                        col as f32 * TILE_SIZE_WORLD + TILE_SIZE_WORLD * 0.5, // centre on column
                        GROUND_Y - row as f32 * TILE_SIZE_WORLD,
                        -3.0,
                    ),
                    scale: Vec3::new(1.2, 1.2, 0.3),
                    ..default()
                },
                TitleSceneEntity,
            ));
        }
    }

    // ── Raccoon (center stage) ────────────────────────────────────────────────
    // character-oozi.glb is the Kenney character with a striped tail — used as
    // the raccoon stand-in until a custom raccoon model is available.
    // y = RACCOON_Y (-54.0): derived as GROUND_Y + TILE_SIZE_WORLD, standing on
    // the top surface of the ground tile row.
    // scale z = 8.0: empirical depth thickness for a character-sized GLB.
    commands.spawn((
        SceneRoot(asset_server.load("models/character-oozi.glb#Scene0")),
        Transform {
            translation: Vec3::new(0.0, RACCOON_Y, 0.0),
            scale: Vec3::new(22.0, 22.0, 8.0),
            ..default()
        },
        TitleSceneEntity,
    ));

    // ── Framing trees (z = -6, just behind raccoon) ───────────────────────────
    // y = -44.0: RACCOON_Y + 10 — tree bases slightly above ground so their
    // roots don't clip into the grass tiles.
    let framing = [
        (-80.0_f32, "models/tree_oak.glb#Scene0"),
        (-52.0_f32, "models/tree_fat.glb#Scene0"),
        (52.0_f32, "models/tree_fat.glb#Scene0"),
        (80.0_f32, "models/tree_oak.glb#Scene0"),
    ];
    for (x, model) in &framing {
        // Center-anchored Trellis models need +scale/2 to ground their base.
        let center_anchored = model.contains("tree_oak") || model.contains("tree_fat");
        let y = if center_anchored { -44.0 + 18.0 * 0.5 } else { -44.0 };
        commands.spawn((
            SceneRoot(asset_server.load(*model)),
            Transform {
                translation: Vec3::new(*x, y, -6.0),
                scale: Vec3::new(18.0, 18.0, 6.0),
                ..default()
            },
            TitleSceneEntity,
        ));
    }

    // ── Background trees (z = -18, depth layer) ───────────────────────────────
    // y = -32.0: higher than framing trees (y=-44) so their bases read above
    // the framing trees' canopy, creating vertical depth separation.
    // Smaller scale (14) vs framing (18) reinforces the distance reading.
    let bg_trees = [
        (-130.0_f32, "models/tree_tall_dark.glb#Scene0"),
        (-100.0_f32, "models/tree_cone_dark.glb#Scene0"),
        (-65.0_f32, "models/tree_tall_dark.glb#Scene0"),
        (65.0_f32, "models/tree_tall_dark.glb#Scene0"),
        (100.0_f32, "models/tree_cone_dark.glb#Scene0"),
        (130.0_f32, "models/tree_tall_dark.glb#Scene0"),
    ];
    for (x, model) in &bg_trees {
        commands.spawn((
            SceneRoot(asset_server.load(*model)),
            Transform {
                translation: Vec3::new(*x, -32.0, -18.0),
                scale: Vec3::new(14.0, 14.0, 5.0),
                ..default()
            },
            TitleSceneEntity,
        ));
    }

    // ── Ground props (all at y = RACCOON_Y, z positive = foreground) ──────────
    // Positive z values place props in front of the raccoon.
    // Scale z is intentionally thin (2–4) — props are decorative dressing only.
    commands.spawn((
        SceneRoot(asset_server.load("models/rock_largeA.glb#Scene0")),
        Transform {
            translation: Vec3::new(-32.0, RACCOON_Y, 2.0),
            scale: Vec3::new(8.0, 8.0, 3.0),
            ..default()
        },
        TitleSceneEntity,
    ));
    commands.spawn((
        SceneRoot(asset_server.load("models/plant_bushLarge.glb#Scene0")),
        Transform {
            translation: Vec3::new(30.0, RACCOON_Y, 1.5),
            scale: Vec3::new(10.0, 10.0, 4.0),
            ..default()
        },
        TitleSceneEntity,
    ));
    commands.spawn((
        SceneRoot(asset_server.load("models/mushrooms.glb#Scene0")),
        Transform {
            translation: Vec3::new(-20.0, RACCOON_Y, 3.0),
            scale: Vec3::new(6.0, 6.0, 2.5),
            ..default()
        },
        TitleSceneEntity,
    ));
    commands.spawn((
        SceneRoot(asset_server.load("models/flower_yellowA.glb#Scene0")),
        Transform {
            translation: Vec3::new(16.0, RACCOON_Y, 4.0),
            scale: Vec3::new(7.0, 7.0, 3.0),
            ..default()
        },
        TitleSceneEntity,
    ));
}
