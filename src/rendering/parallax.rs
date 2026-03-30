use bevy::prelude::*;
use bevy::platform::collections::HashMap;

use crate::level::components::Decoration;
use crate::rendering::camera::GameplayCamera;

/// Drives horizontal parallax scrolling.
/// `factor` = fraction of camera movement applied to this layer each frame.
///   0.2 = barely moves (sky, very distant)
///   0.9 = nearly tracks with world (near trees)
#[derive(Component)]
pub struct ParallaxLayer {
    pub factor: f32,
}

/// Marks level-specific parallax background entities (tree/building layers).
/// These must be despawned on level transition and respawned by `spawn_level_decorations`.
/// Permanent elements (sky, mountains, clouds) do NOT carry this marker.
#[derive(Component)]
pub struct ParallaxBackground;

/// Computes each parallax layer's absolute render position from the snapped camera.
///
/// WHY absolute, not delta-accumulating: the old `+= delta * factor` approach
/// accumulated fractional offsets into render-space transforms, violating the
/// camera-snapped pixel-perfect contract.  Now each layer's position is derived
/// from its recorded origin and the total camera displacement since the reference
/// point, then rounded to integer pixels.  No fractional positions ever reach
/// the renderer.
///
/// WHY `Local<HashMap>` for origins: avoids adding a field to `ParallaxLayer` and
/// touching ~30 spawn sites.  Each entity self-registers its spawn-time x on first
/// encounter.  The map is cleared on teleport (level transition) so stale entries
/// from despawned layers are purged and new layers re-anchor cleanly.
///
/// WHY GameplayCamera filter: `single()` would fail (MultipleEntities) whenever
/// a secondary camera exists (e.g. the TitleSceneEntity camera during TitleScreen).
/// GameplayCamera guarantees exactly one match regardless of other active cameras.
pub fn update_parallax(
    camera_query: Query<&Transform, (With<Camera3d>, With<GameplayCamera>)>,
    mut layer_query: Query<(Entity, &ParallaxLayer, &mut Transform), Without<Camera3d>>,
    mut ref_cam_x: Local<Option<f32>>,
    mut origins: Local<HashMap<Entity, f32>>,
) {
    let Ok(cam) = camera_query.single() else { return };
    let cam_x = cam.translation.x;

    // Establish or reset the reference camera position.
    let reference = if let Some(prev) = *ref_cam_x {
        if (cam_x - prev).abs() > 200.0 {
            // Teleport (level transition): reset reference and clear origins
            // so newly-spawned layers re-anchor at their design positions.
            origins.clear();
            *ref_cam_x = Some(cam_x);
            cam_x
        } else {
            prev
        }
    } else {
        *ref_cam_x = Some(cam_x);
        cam_x
    };

    let total_delta = cam_x - reference;

    for (entity, layer, mut tf) in layer_query.iter_mut() {
        // First encounter: record this entity's spawn x as its origin.
        let origin = *origins.entry(entity).or_insert(tf.translation.x);
        // Absolute position from origin + scaled displacement, rounded to integer pixels.
        tf.translation.x = (origin + total_delta * layer.factor).round();
    }
}

/// Spawn all background/parallax layers.
///
/// Depth stack — canonical Z bands per jasper_layered_z_separation.txt:
///
///   z = +10   Foreground framing trees — large trees at level edges, in front of gameplay.
///   z =   0   Gameplay (ground / platform tiles, player, enemies)
///   z =  -38  Mountain attenuation plane — 18% dark overlay; everything behind this
///             is subtly dimmed, creating contrast separation from gameplay.
///   z =  -50  Background trees — near layer (green canopy, factor 0.9).
///             WHY z=-50 not z=-10: guardrail jasper_gameplay_readability_guardrail.txt
///             [4] requires background layers ≥40 units behind gameplay.  The old
///             z=-10 value (10 units) was below the minimum, causing near trees to
///             visually blur with platforms and confuse readability.  z=-50 satisfies
///             the guardrail and also places near trees behind the attenuation plane
///             (-38), giving them a slight darkness that reinforces depth hierarchy.
///   z =  -60  Clouds — behind near trees, high-altitude distant sky
///   z =  -70  Distant mountains (stone peaks with snow caps).
///             WHY z=-70: mountains must render BEHIND near trees (z=-50) so the
///             forest frames the mountains rather than mountains occluding trees.
///   z =  -80  Dark background trees — mid parallax (darker conifers, factor 0.75).
///             WHY z=-80 not z=-25: same guardrail fix; 80 units of separation gives
///             dark trees a visually distinct depth layer well behind near trees (-50).
///   z = -100  Sky backdrop — solid sky-blue, never transparent.
/// Spawns sky, mountains, attenuation plane, and clouds — elements that appear
/// in every level but must still obey the level lifecycle (despawn on level exit).
///
/// Called once per level entry from `spawn_level_decorations`.
/// Every entity carries `Decoration` so it is despawned on level transition.
///
/// jasper_background_parallax_lifecycle_guardrail: backgrounds are level content,
/// not engine setup. Nothing here lives in Startup.
pub fn spawn_shared_background(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    asset_server: &AssetServer,
) {
    // ── Snowy mountains (z = -70) ─────────────────────────────────────────────
    // WHY z=-70: mountains must sit BEHIND near trees (z=-50) so the forest
    // frames the mountains rather than mountains occluding trees. z=-70 places
    // them between near trees (-50) and dark trees (-80).
    // Scale reduced ~20% vs old values so mountains read as distant, not dominant.
    // Factor 0.45 (was 0.6) — truly distant peaks drift very slowly.
    // Span x = -1500..+1600 (step 160) so every level start position has coverage.
    let mountain_scales = [178.0_f32, 160.0, 197.0, 167.0, 175.0, 185.0, 156.0, 192.0];
    let mountain_data: Vec<(f32, f32, f32)> = (-1500..=1600)
        .step_by(160)
        .enumerate()
        .map(|(i, x)| (x as f32, -170.0_f32, mountain_scales[i % mountain_scales.len()]))
        .collect();
    for &(mx, my, mscale) in &mountain_data {
        commands.spawn((
            SceneRoot(asset_server.load("models/stone-mountain.glb#Scene0")),
            Transform::from_xyz(mx, my, -70.0).with_scale(Vec3::new(mscale, mscale, 20.0)),
            ParallaxLayer { factor: 0.35 },
            Decoration,
        ));

        let cap_w = mscale * 0.38;
        let cap_h = mscale * 0.20;
        let cap_y = my + mscale * 0.70;
        let snow_mesh = meshes.add(Mesh::from(Rectangle::new(cap_w, cap_h)));
        let snow_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(0.93, 0.95, 1.00),
            alpha_mode: AlphaMode::Opaque,
            unlit: true,
            ..default()
        });
        commands.spawn((
            Mesh3d(snow_mesh),
            MeshMaterial3d(snow_mat),
            Transform::from_xyz(mx, cap_y, -69.9),
            ParallaxLayer { factor: 0.35 },
            Decoration,
        ));
    }

    // ── Sky backdrop (z = -100) ───────────────────────────────────────────────
    // 6400 wide: covers Sanctuary (±1440) + parallax drift at factor 0.15.
    let sky_mesh = meshes.add(Mesh::from(Rectangle::new(6400.0, 1800.0)));
    let sky_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.45, 0.72, 0.90),
        alpha_mode: AlphaMode::Opaque,
        unlit: true,
        ..default()
    });
    commands.spawn((
        Mesh3d(sky_mesh),
        MeshMaterial3d(sky_mat),
        Transform::from_translation(Vec3::new(0.0, -50.0, -100.0)),
        ParallaxLayer { factor: 0.20 },
        Decoration,
    ));

    // ── Mountain attenuation plane (z = -38) ─────────────────────────────────
    // Semi-transparent dark overlay — creates gameplay > trees > mountains contrast.
    // 5000 wide to match mountain spread.
    let attn_mesh = meshes.add(Mesh::from(Rectangle::new(5000.0, 1600.0)));
    let attn_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.0, 0.0, 0.0, 0.18),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        double_sided: true,
        cull_mode: None,
        ..default()
    });
    commands.spawn((
        Mesh3d(attn_mesh),
        MeshMaterial3d(attn_mat),
        Transform::from_xyz(0.0, -50.0, -38.0),
        ParallaxLayer { factor: 0.38 },
        Decoration,
    ));

    // ── Clouds (z = -60) ──────────────────────────────────────────────────────
    // Span x = -1500..+1600 to match mountain coverage.
    let cloud_configs: &[(&str, f32, f32, f32)] = &[
        ("clouds/cloud1.png", -1480.0, 100.0, 0.50),
        ("clouds/cloud3.png", -1300.0, 130.0, 0.60),
        ("clouds/cloud5.png", -1100.0,  90.0, 0.42),
        ("clouds/cloud2.png",  -950.0,  60.0, 0.52),
        ("clouds/cloud7.png",  -780.0, 115.0, 0.38),
        ("clouds/cloud4.png",  -630.0, 145.0, 0.55),
        ("clouds/cloud6.png",  -480.0, 120.0, 0.45),
        ("clouds/cloud8.png",  -320.0,  80.0, 0.48),
        ("clouds/cloud2.png",  -160.0, 110.0, 0.52),
        ("clouds/cloud5.png",    -20.0, 135.0, 0.44),
        ("clouds/cloud1.png",   140.0,  95.0, 0.50),
        ("clouds/cloud3.png",   300.0, 120.0, 0.58),
        ("clouds/cloud7.png",   460.0, 105.0, 0.40),
        ("clouds/cloud4.png",   620.0, 130.0, 0.54),
        ("clouds/cloud6.png",   780.0,  85.0, 0.46),
        ("clouds/cloud8.png",   940.0, 115.0, 0.50),
        ("clouds/cloud1.png",  1100.0, 100.0, 0.48),
        ("clouds/cloud3.png",  1260.0, 125.0, 0.56),
        ("clouds/cloud5.png",  1420.0,  88.0, 0.44),
        ("clouds/cloud2.png",  1580.0, 115.0, 0.50),
    ];
    for &(tex, cx, cy, scale) in cloud_configs {
        let cloud_mesh = meshes.add(Mesh::from(Rectangle::new(120.0 * scale, 60.0 * scale)));
        let cloud_mat = materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load(tex)),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            double_sided: true,
            cull_mode: None,
            ..default()
        });
        commands.spawn((
            Mesh3d(cloud_mesh),
            MeshMaterial3d(cloud_mat),
            Transform::from_xyz(cx, cy, -60.0),
            ParallaxLayer { factor: 0.28 },
            Decoration,
        ));
    }
}

// Subdivision parallax plates are now handled by the segmented panorama
// system in subdivision_panorama.rs.  See that module for depth/factor
// specs and the full composition reference.

/// Spawns the forest/nature background tree layers (dark mid + bright front).
/// Each entity is marked `Decoration` so it gets despawned on level transitions.
/// Spans x = -1500..+1600 to cover every level's starting camera position.
pub fn spawn_nature_background(commands: &mut Commands, asset_server: &AssetServer) {
    let dark_tree_models = [
        "models/tree_cone_dark.glb",
        "models/tree_tall_dark.glb",
        "models/tree_cone_dark.glb",
    ];
    let dark_tree_scales = [52.0_f32, 44.0, 58.0, 46.0, 54.0, 48.0, 56.0, 43.0];
    // Step 80 units from -1500 to +1600 → ~39 trees, good mid-layer density.
    for (i, x) in (-1500..=1600i32).step_by(80).enumerate() {
        let model = dark_tree_models[i % dark_tree_models.len()];
        let scale = dark_tree_scales[i % dark_tree_scales.len()];
        commands.spawn((
            SceneRoot(asset_server.load(format!("{}#Scene0", model))),
            Transform::from_xyz(x as f32, -160.0, -80.0).with_scale(Vec3::new(scale, scale, 12.0)),
            ParallaxLayer { factor: 0.48 },
            Decoration,
            ParallaxBackground,
        ));
    }

    let tree_models = [
        "models/tree_oak.glb",
        "models/tree_pine.glb",
        "models/tree_default.glb",
        "models/tree_fat.glb",
    ];
    let tree_scales = [93.0_f32, 82.0, 103.0, 89.0, 98.0, 84.0, 101.0, 91.0];
    // Step 60 units → ~52 trees, denser near layer.
    for (i, x) in (-1500..=1600i32).step_by(60).enumerate() {
        let model = tree_models[i % tree_models.len()];
        let scale = tree_scales[i % tree_scales.len()];
        commands.spawn((
            SceneRoot(asset_server.load(format!("{}#Scene0", model))),
            Transform::from_xyz(x as f32, -160.0, -50.0).with_scale(Vec3::new(scale, scale, 6.0)),
            ParallaxLayer { factor: 0.38 },
            Decoration,
            ParallaxBackground,
        ));
    }
}
