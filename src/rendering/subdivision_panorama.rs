//! Subdivision parallax layers — NEAR active, MID active, FAR archived.
//!
//! NEAR: Hollow Knight–style edge wisps at viewport margins (z=−30).
//!       Alpha-blended silhouette masks, optional (`NEAR_ENABLED` flag).
//! MID:  LOCKED — `subdivision_mid.png` is the final MID background.
//! FAR:  LOCKED — `subdivision_far.png` is archived (not spawned).
//!
//! FAR is intentionally disabled: the opaque MID plate fully occupies the
//! vertical camera envelope at all camera positions.  The FAR layer is never
//! visible, even at maximum jump height.  Its assets, configuration, and
//! spawn function are preserved here for future use (e.g. if MID gains
//! alpha transparency or the camera envelope changes).
//!
//! Parallax handled by `ParallaxLayer` / `update_parallax`.

use bevy::prelude::*;

use crate::level::components::{Decoration, SubdivisionOnly};
use crate::rendering::parallax::{ParallaxBackground, ParallaxLayer};

// ── Component ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PanoramaLayerId {
    Far,
    Mid,
    Near,
}

#[derive(Component)]
pub struct PanoramaSegment {
    pub layer: PanoramaLayerId,
    pub anchor_x: f32,
    pub width_world: f32,
}

// ── NEAR edge wisps (Hollow Knight–style atmospheric framing) ────────
//
// Small, abstract, semi-transparent silhouette masks at left/right
// viewport edges only.  Factor 1.20 (foreground parallax) means each
// element outpaces the camera by 20%, creating slow inward *pressure*
// rather than visible scrolling.  Over the full 1160u camera travel,
// wisps drift 232u relative to the viewport (41% of viewport width).
//
// Z depth: −30 — behind all gameplay (z=0), player (z=5), enemies,
// and foreground props (z=3..10).  In front of the attenuation plane
// (z=−38) and MID (z=−55).  This placement structurally prevents NEAR
// from ever overlapping any gameplay entity regardless of parallax state.
//
// Rendering: alpha-blended, unlit.  Desaturated dark olive/charcoal
// at 24% opacity — reads as atmospheric shadow, not a visible object.
// Hollow Knight reference: foreground silhouettes are near-black,
// zero saturation, ~20–30% opacity, with organic soft edges.
//
// Placement safety (60% center exclusion, 60u width, factor 1.20):
//   Right-edge safe origins:  x0 > −379  (binding at cam_min = −580)
//   Left-edge safe origins:   x0 < −1013 (binding at cam_max = +580)
//   Coverage: ~80% of camera travel (20% mid-scroll gap with no NEAR).
//   The gap is by design — NEAR exists to subtly frame edges, not fill.
//
// Constraint invariants (compile-time assertions below):
//   Width    ∈ [48, 72]     world units
//   Height   ∈ [100, 140]   world units
//   Bottom edge ≥ y = 20    (center_y − height/2 ≥ 20)
//   Center Y ∈ [60, 120]
//   Opacity  ∈ [0.20, 0.30] (never exceed 0.30)
//   Factor   ∈ [1.20, 1.30]
//   Z        ∈ (MID_Z, 0)   (behind gameplay, in front of MID)
//   Never within center 60% of viewport at any camera position
//
// ── DISABLE CONDITIONS (set NEAR_ENABLED = false) ───────────────
//
// 1. OVERLAPS PLAYER — Structurally prevented: NEAR Z (−30) is behind
//    all gameplay entities (z ≥ 0).  If NEAR_Z is ever moved to z ≥ 0,
//    the compile-time assertion will fail.  If parallax drift somehow
//    causes a wisp to visually align with the player sprite despite
//    being behind it, disable immediately.
//
// 2. BECOMES NOTICEABLE — NEAR must be imperceptible during active play.
//    If a playtester can consciously identify a wisp as a distinct
//    element (rather than vague atmospheric darkening), disable.
//    Thresholds: opacity > 0.28 is suspect, > 0.30 is a hard violation.
//    Width > 72u or height > 140u also risks noticeability.
//
// 3. IMPACTS READABILITY — NEAR must never reduce contrast between
//    gameplay elements and their background.  At z=−30 with 24% opacity,
//    the darkening is < 3% luminance.  If readability degrades in the
//    outer 20% of viewport (where wisps appear), disable.  Specific
//    failure modes: wisp edge aligns with platform edge creating false
//    contour; wisp color competes with enemy silhouette; wisp causes
//    player to misjudge jump distance near viewport margins.

/// Master switch — set to `false` to disable NEAR without removing code.
const NEAR_ENABLED: bool = true;

const NEAR_QUAD_WIDTH: f32 = 60.0;
const NEAR_QUAD_HEIGHT: f32 = 120.0;
/// Center Y: bottom edge = 100 − 60 = 40 (≥ 20 ✓).
const NEAR_CENTER_Y: f32 = 100.0;
/// Behind gameplay (z=0) and player (z=5), in front of MID (z=−55).
/// Structurally prevents any overlap with gameplay entities.
const NEAR_Z: f32 = -30.0;
/// Foreground parallax: 20% faster than camera.  Reads as slow inward
/// pressure — the viewport edges tighten imperceptibly as camera pans.
/// 1.20 chosen over 1.30 for Hollow Knight "breathing" feel: drift is
/// 232u over full camera travel (vs 348u at 1.30), keeping motion
/// subliminal rather than trackable.
const NEAR_FACTOR: f32 = 1.20;
/// Opacity: extracted so compile-time assertion can enforce the cap.
const NEAR_OPACITY: f32 = 0.24;
/// Desaturated dark olive/charcoal — Hollow Knight reference palette.
/// G slightly dominant (olive undertone), all channels low (charcoal).
/// At 24% opacity the effective on-screen tint is < 3% luminance shift,
/// imperceptible during active play but present as atmospheric weight.
const NEAR_COLOR: Color = Color::srgba(0.09, 0.11, 0.08, NEAR_OPACITY);

// ── Compile-time constraint validation ──────────────────────────────
const _: () = assert!(NEAR_QUAD_WIDTH >= 48.0 && NEAR_QUAD_WIDTH <= 72.0,
    "NEAR width must be 48–72 world units");
const _: () = assert!(NEAR_QUAD_HEIGHT >= 100.0 && NEAR_QUAD_HEIGHT <= 140.0,
    "NEAR height must be 100–140 world units");
const _: () = assert!(NEAR_CENTER_Y >= 60.0 && NEAR_CENTER_Y <= 120.0,
    "NEAR center Y must be 60–120");
const _: () = assert!(NEAR_CENTER_Y - NEAR_QUAD_HEIGHT / 2.0 >= 20.0,
    "NEAR bottom edge must be ≥ y=20");
const _: () = assert!(NEAR_OPACITY >= 0.20 && NEAR_OPACITY <= 0.30,
    "NEAR opacity must be 0.20–0.30 (never exceed 0.30)");
const _: () = assert!(NEAR_FACTOR >= 1.20 && NEAR_FACTOR <= 1.30,
    "NEAR factor must be 1.20–1.30");
const _: () = assert!(NEAR_Z < 0.0,
    "NEAR Z must be negative (behind gameplay) to prevent player overlap");
const _: () = assert!(NEAR_Z > MID_Z,
    "NEAR Z must be in front of MID");

/// Four edge wisps: 2 right-edge, 2 left-edge.
///
/// Right-edge (origins > −379): visible when cam_x < −185.
/// Left-edge (origins < −1013): visible when cam_x > +50.
/// Gap: cam_x ∈ [−185, +50] = 20% of camera travel (no NEAR visible).
/// Masks are 128×256 px organic blobs (1:2 aspect matching 60×120 quad).
const NEAR_PANELS: [(&str, f32); 4] = [
    // Right-edge group (visible near level entry)
    ("backgrounds/subdivision/near/near_silhouette_a.png", -345.0),
    ("backgrounds/subdivision/near/near_silhouette_b.png", -370.0),
    // Left-edge group (visible near level end)
    ("backgrounds/subdivision/near/near_silhouette_a.png", -1020.0),
    ("backgrounds/subdivision/near/near_silhouette_b.png", -1050.0),
];

// ── FAR plate geometry (ARCHIVED — not spawned) ─────────────────────
// Source: subdivision_far.png — 4096×512 px, aspect 8:1.
// Asset path: backgrounds/subdivision/plates/subdivision_far.png
//
// Warm sunset treeline — would be visible below the MID plate in the
// gap between MID bottom (y=−37) and viewport bottom (y≈−200), but
// MID fully covers the camera envelope so FAR never reaches the screen.
//
// Preserved configuration (verified 2026-03-27):
//   FAR_QUAD_WIDTH:  3168.0   (396 × 8, native aspect)
//   FAR_QUAD_HEIGHT: 396.0
//   FAR_CENTER_Y:    -120.0   (tree band fills y≈−199 to y≈−41)
//   FAR_Z:           -95.0    (behind MID −55, in front of sky −100)
//   FAR_FACTOR:      0.28     (approved range 0.25–0.30)
//   FAR_BASE_X:      -162.0   (centred at camera midpoint)
//
// Edge safety (if re-enabled):
//   X: 882+ u margin at all camera positions (16:9 through 32:9).
//   Y: exposed band [−200, −37] fully covered with 115–118 u margin.
//
// To re-enable: uncomment spawn_subdivision_far() below and its call
// site in level/mod.rs (search for "FAR panorama").

// ── MID plate geometry ───────────────────────────────────────────────
// Source: subdivision_mid.png — 3328×768 px, aspect 4.333:1.
// Quad preserves native aspect at 396 world-unit height.

const MID_QUAD_WIDTH: f32 = 1716.0;
const MID_QUAD_HEIGHT: f32 = 396.0;
const MID_CENTER_Y: f32 = 161.0;
/// Must be negative — gameplay is at z=0, player at z=5.
const MID_Z: f32 = -55.0;
const MID_FACTOR: f32 = 0.55;

const _: () = assert!(MID_Z < 0.0, "MID_Z must be negative to render behind gameplay");

/// Anchor X: centres the houses at the camera travel midpoint.
///
/// Camera range (16:9): [−580, +580].  Reference = −580 (level entry).
/// At cam midpoint (x=0): quad centre = anchor + 580 × 0.55 = anchor + 319.
/// Setting anchor = −319 → houses centred at world x=0.
///
/// Edge safety (16:9, half_width ≈ 284):
///   cam min (−580): quad left = −319 − 858 = −1177, viewport left = −864 → 313 u margin
///   cam max (+580): quad right = +319 + 858 = +1177, viewport right = +864 → 313 u margin
/// Ultrawide (21:9): 190+ u margin.  Edges never visible.
const BASE_X: f32 = -319.0;

// ── Spawn ────────────────────────────────────────────────────────────

/// Spawns small NEAR edge wisps — optional, Hollow Knight–style.
///
/// Four organic silhouette masks (60×120u, 24% opacity dark olive) at
/// left/right viewport edges.  Z=−30: behind gameplay, in front of MID.
/// Parallax 1.20 — motion reads as atmospheric pressure, not scrolling.
/// Alpha-blended, unlit.  At any camera position, 0–2 wisps partially
/// visible in the outer 20% of the viewport.  Gated by `NEAR_ENABLED`.
pub fn spawn_subdivision_near(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    asset_server: &AssetServer,
) {
    if !NEAR_ENABLED {
        return;
    }

    let mesh = meshes.add(Mesh::from(Rectangle::new(NEAR_QUAD_WIDTH, NEAR_QUAD_HEIGHT)));

    for &(mask_path, origin_x) in &NEAR_PANELS {
        let mat = materials.add(StandardMaterial {
            base_color: NEAR_COLOR,
            base_color_texture: Some(asset_server.load(mask_path)),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            double_sided: true,
            cull_mode: None,
            ..default()
        });

        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(mat),
            Transform::from_xyz(origin_x, NEAR_CENTER_Y, NEAR_Z),
            ParallaxLayer { factor: NEAR_FACTOR },
            PanoramaSegment {
                layer: PanoramaLayerId::Near,
                anchor_x: origin_x,
                width_world: NEAR_QUAD_WIDTH,
            },
            Decoration,
            SubdivisionOnly,
        ));
    }
}

// /// Spawns a single FAR plate — LOCKED, finalized asset.
// /// ARCHIVED: MID fully covers the camera envelope; FAR is never visible.
// /// Uncomment this function and its call site in level/mod.rs to re-enable.
// pub fn spawn_subdivision_far(
//     commands: &mut Commands,
//     meshes: &mut Assets<Mesh>,
//     materials: &mut Assets<StandardMaterial>,
//     asset_server: &AssetServer,
// ) {
//     const FAR_QUAD_WIDTH: f32 = 3168.0;
//     const FAR_QUAD_HEIGHT: f32 = 396.0;
//     const FAR_CENTER_Y: f32 = -120.0;
//     const FAR_Z: f32 = -95.0;
//     const FAR_FACTOR: f32 = 0.28;
//     const FAR_BASE_X: f32 = -162.0;
//
//     let mesh = meshes.add(Mesh::from(Rectangle::new(FAR_QUAD_WIDTH, FAR_QUAD_HEIGHT)));
//
//     let mat = materials.add(StandardMaterial {
//         base_color_texture: Some(asset_server.load("backgrounds/subdivision/plates/subdivision_far.png")),
//         alpha_mode: AlphaMode::Opaque,
//         unlit: true,
//         double_sided: true,
//         cull_mode: None,
//         ..default()
//     });
//
//     commands.spawn((
//         Mesh3d(mesh),
//         MeshMaterial3d(mat),
//         Transform::from_xyz(FAR_BASE_X, FAR_CENTER_Y, FAR_Z),
//         ParallaxLayer { factor: FAR_FACTOR },
//         PanoramaSegment {
//             layer: PanoramaLayerId::Far,
//             anchor_x: FAR_BASE_X,
//             width_world: FAR_QUAD_WIDTH,
//         },
//         Decoration,
//         SubdivisionOnly,
//         ParallaxBackground,
//     ));
// }

/// Spawns a single MID plate — LOCKED, finalized asset.
pub fn spawn_subdivision_mid(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    asset_server: &AssetServer,
) {
    let mesh = meshes.add(Mesh::from(Rectangle::new(MID_QUAD_WIDTH, MID_QUAD_HEIGHT)));

    let mat = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load("backgrounds/subdivision/plates/subdivision_mid.png")),
        alpha_mode: AlphaMode::Opaque,
        unlit: true,
        double_sided: true,
        cull_mode: None,
        ..default()
    });

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(mat),
        Transform::from_xyz(BASE_X, MID_CENTER_Y, MID_Z),
        ParallaxLayer { factor: MID_FACTOR },
        PanoramaSegment {
            layer: PanoramaLayerId::Mid,
            anchor_x: BASE_X,
            width_world: MID_QUAD_WIDTH,
        },
        Decoration,
        SubdivisionOnly,
        ParallaxBackground,
    ));
}
