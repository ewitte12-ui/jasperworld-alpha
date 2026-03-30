use bevy::camera::ScalingMode;
use bevy::prelude::*;

/// Canonical camera update ordering — enforced every Update frame.
///
/// Total order: Follow → Clamp → Snap → Parallax
///
/// WHY: `update_parallax` reads the camera transform and must see the fully-settled,
/// pixel-snapped position after `camera_follow` (tracking), `camera_clamp` (bounds),
/// and `camera_snap` (integer pixel rounding) have all completed.  Without this
/// ordering, Bevy's parallel executor may run `update_parallax` against a mid-frame,
/// partially-updated camera transform, causing jitter even when the player is still.
///
/// Rule (pixel_perfect_camera_contract): `camera_snap` is the SINGLE AUTHORITY for
/// rounding camera position to integer pixels.  No other system may call `round()` on
/// camera translation.  `camera_snap` runs once per frame, after all camera mutations
/// (Follow, Clamp), before any camera reads (Parallax, Render).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum CameraPipeline {
    /// camera_follow: lerp toward player, teleport snap for large gaps.
    Follow,
    /// camera_clamp: enforce level bounds (float output).
    Clamp,
    /// camera_snap: round camera position to integer pixels (SINGLE AUTHORITY).
    /// No other system may round or snap the camera position.
    Snap,
    /// update_parallax: reads final snapped camera transform, writes layer offsets.
    Parallax,
}

/// Marker for the primary (sun) directional light.
///
/// WHY: Two DirectionalLight entities exist in the scene (primary sun + fill light).
/// Without this marker, `Query<&mut DirectionalLight>::single_mut()` returns
/// `Err(MultipleEntities)` on every call, silently preventing lighting theme
/// changes between levels.  Filtering `With<PrimaryDirectionalLight>` gives a
/// unique match so `update_lighting` can actually apply per-level themes.
#[derive(Component)]
pub struct PrimaryDirectionalLight;

/// Role marker for the single persistent gameplay camera.
///
/// WHY: Per jasper_camera_role_identity_guardrail, every camera must carry
/// exactly one role marker.  All systems that query a camera by Transform
/// MUST filter with `(With<Camera3d>, With<GameplayCamera>)` — never
/// `With<Camera3d>` alone — so they remain correct when other cameras
/// (TitleCamera, future debug/cutscene cameras) coexist in the world.
///
/// LIFECYCLE: spawned `is_active: false` at Startup.  TitleBackgroundPlugin
/// activates it on `OnExit(AppState::TitleScreen)` and deactivates it on
/// `OnEnter(AppState::TitleScreen)`.  This satisfies the title screen
/// isolation guardrail: exactly 1 active camera per AppState at all times.
///
/// WARNING: Removing this marker from the gameplay camera will cause every
/// `With<GameplayCamera>` query to return zero results, silently breaking
/// parallax, camera-follow, camera-clamp, and VFX weather placement.
#[derive(Component)]
pub struct GameplayCamera;

/// Single authority for pixel-perfect camera alignment.
/// Runs once per frame, after Follow and Clamp, before Parallax.
/// No other system may round or snap the camera position.
pub fn camera_snap(
    mut camera_query: Query<&mut Transform, (With<Camera3d>, With<GameplayCamera>)>,
) {
    let Ok(mut cam) = camera_query.single_mut() else {
        return;
    };
    cam.translation.x = cam.translation.x.round();
    cam.translation.y = cam.translation.y.round();
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        // Match sky color so ClearColor is never visible as a seam.
        app.insert_resource(ClearColor(Color::srgb(0.45, 0.72, 0.90)));
        app.add_systems(Startup, setup_camera);
    }
}

fn setup_camera(mut commands: Commands) {
    // Orthographic camera looking down -Z at the XY plane.
    commands.spawn((
        Camera3d::default(),
        Camera {
            // WHY is_active: false — starts inactive; TitleBackgroundPlugin
            // activates it when TitleScreen exits.  This guarantees exactly 1
            // active camera during TitleScreen (the title camera) with zero
            // code needed in the first-frame path.
            is_active: false,
            ..default()
        },
        GameplayCamera,
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 320.0,
            },
            near: -1000.0,
            far: 1000.0,
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_xyz(0.0, 0.0, 100.0)
            .with_rotation(Quat::from_rotation_x(-28_f32.to_radians())),
        // Disable MSAA to prevent texture atlas bleeding at sprite cell boundaries.
        Msaa::Off,
    ));

    // Primary sun: high above and slightly forward (+Z) so block TOPS are brightly lit
    // and the front face is in softer light — this is what gives the 2.5D depth.
    commands.spawn((
        DirectionalLight {
            illuminance: 15000.0,
            color: Color::srgb(1.0, 0.97, 0.88), // warm sunlight tint
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 400.0, 150.0).looking_at(Vec3::ZERO, Vec3::Y),
        PrimaryDirectionalLight,
    ));

    // Fill light: soft cool light from below-front to prevent the block fronts from
    // going completely dark, giving a pleasant gradient from bright top to lit front.
    commands.spawn((
        DirectionalLight {
            illuminance: 3500.0,
            color: Color::srgb(0.75, 0.85, 1.0), // cool sky-bounce tint
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, -200.0, 200.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
