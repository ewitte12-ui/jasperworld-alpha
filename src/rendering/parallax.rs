use bevy::platform::collections::HashMap;
use bevy::prelude::*;

use crate::level::components::Decoration;
use crate::rendering::camera::GameplayCamera;

/// Drives horizontal parallax scrolling.
/// `factor` = fraction of camera movement applied to this layer each frame.
///   factor → 1.0 = layer tracks fully with camera = appears infinitely far (sky/backdrop)
///   factor → 0.0 = layer stays fixed in world = appears nearest to player
/// Convention: DISTANT layers get HIGH factors; NEAR layers get LOW factors.
#[derive(Component)]
pub struct ParallaxLayer {
    pub factor: f32,
}

/// Pending color tint for a SceneRoot model. A one-shot system finds loaded
/// children with StandardMaterial, clones the material, applies the tint,
/// then removes the component.
///
/// `Multiply` — multiplies existing base_color channels by the tint (preserves
///   material variation; used for house color washes).
/// `Replace` — overwrites base_color entirely (used for grey platforms, etc.).
#[derive(Component, Debug)]
pub enum SceneTint {
    Multiply(Color),
    Replace(Color),
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
///
/// PARALLAX FACTOR CONVENTION (corrected):
///   factor → 1.0 = layer tracks fully with camera = appears infinitely far away (sky)
///   factor → 0.0 = layer stays fixed in world space = appears closest to player
/// This means DISTANT layers have HIGH factors and NEAR layers have LOW factors.
pub fn update_parallax(
    camera_query: Query<&Transform, (With<Camera3d>, With<GameplayCamera>)>,
    mut layer_query: Query<(Entity, &ParallaxLayer, &mut Transform), Without<Camera3d>>,
    mut ref_cam_x: Local<Option<f32>>,
    mut origins: Local<HashMap<Entity, f32>>,
) {
    let Ok(cam) = camera_query.single() else {
        return;
    };
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
///   z =  -50  Background trees — near layer (green canopy, factor 0.45).
///             WHY z=-50 not z=-10: guardrail jasper_gameplay_readability_guardrail.txt
///             [4] requires background layers ≥40 units behind gameplay.  The old
///             z=-10 value (10 units) was below the minimum, causing near trees to
///             visually blur with platforms and confuse readability.  z=-50 satisfies
///             the guardrail and also places near trees behind the attenuation plane
///             (-38), giving them a slight darkness that reinforces depth hierarchy.
///   z =  -60  Clouds — behind near trees, high-altitude distant sky (factor 0.80)
///   z =  -70  Distant mountains (stone peaks with snow caps, factor 0.85).
///             WHY z=-70: mountains must render BEHIND near trees (z=-50) so the
///             forest frames the mountains rather than mountains occluding trees.
///   z =  -80  Dark background trees — mid parallax (darker conifers, factor 0.70).
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
    level_id: crate::level::level_data::LevelId,
) {
    info!(
        "[SHARED_BG] spawn_shared_background called for {:?}",
        level_id
    );
    // ── Snowy mountains (z = -70) — Forest only ───────────────────────────────
    // Subdivision has houses, City has skyscrapers; only Forest gets mountains.
    if level_id == crate::level::level_data::LevelId::Forest {
        let mountain_scales = [178.0_f32, 160.0, 197.0, 167.0, 175.0, 185.0, 156.0, 192.0];
        let mountain_data: Vec<(f32, f32, f32)> = (-1500..=1600)
            .step_by(160)
            .enumerate()
            .map(|(i, x)| {
                (
                    x as f32,
                    -170.0_f32,
                    mountain_scales[i % mountain_scales.len()],
                )
            })
            .collect();
        for &(mx, my, mscale) in &mountain_data {
            commands.spawn((
                SceneRoot(asset_server.load("models/stone-mountain.glb#Scene0")),
                Transform::from_xyz(mx, my, -70.0).with_scale(Vec3::new(mscale, mscale, 20.0)),
                // WHY 0.85: mountains at z=-70 are very distant. High factor = tracks camera
                // closely = reads as far away. 0.85 sits between clouds (0.80) and sky (0.95).
                ParallaxLayer { factor: 0.85 },
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
                // WHY 0.85: snow cap must match mountain body factor exactly so they
                // scroll as a unit and the cap never drifts off the mountain peak.
                ParallaxLayer { factor: 0.85 },
                Decoration,
            ));
        }
    }

    // ── Sky backdrop (z = -100) ───────────────────────────────────────────────
    // 6400 wide: covers all levels + parallax drift.
    // WHY factor 0.95 for all levels: the sky is the most distant element in the scene.
    // A factor near 1.0 means the layer tracks closely with the camera, which is the
    // correct behavior for an "infinitely distant" backdrop — it barely moves relative
    // to the player, reinforcing the illusion of vast depth.
    // Previously Forest/Subdivision used 0.20 (incorrect: low factor = near, not far).
    let sky_factor = 0.95; // All levels: sky is maximally distant, nearly camera-anchored
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
        ParallaxLayer { factor: sky_factor },
        Decoration,
    ));

    // ── Attenuation plane(s) ────────────────────────────────────────────────
    // Forest: single plane at z=-38 (18% dark) covers all background equally.
    // Subdivision: two planes — light (12%) for near houses, heavy (35%) for far
    //   houses — so the far row reads as visibly more distant.
    if matches!(
        level_id,
        crate::level::level_data::LevelId::Subdivision | crate::level::level_data::LevelId::City
    ) {
        let is_city = matches!(level_id, crate::level::level_data::LevelId::City);
        // Near attenuation (z=-38): light overlay for near buildings at z=-50.
        // WHY City factor=0.92: this overlay doubles as the star-field/sky-tint for
        // the night scene. At 0.92 it stays nearly anchored with the camera so the
        // night sky reads as stationary — consistent with the sky backdrop (0.95).
        // Subdivision keeps 0.38 (original near-attenuation parallax behavior).
        let near_attn_factor = if is_city { 0.92 } else { 0.38 };
        let near_attn_mesh = meshes.add(Mesh::from(Rectangle::new(5000.0, 1600.0)));
        let near_attn_mat = materials.add(StandardMaterial {
            base_color: Color::srgba(0.0, 0.0, 0.0, 0.12),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            double_sided: true,
            cull_mode: None,
            ..default()
        });
        commands.spawn((
            Mesh3d(near_attn_mesh),
            MeshMaterial3d(near_attn_mat),
            Transform::from_xyz(0.0, -50.0, -38.0),
            ParallaxLayer { factor: near_attn_factor },
            Decoration,
        ));
        // Far attenuation (z=-75): heavier overlay for far houses at z=-80.
        // City uses near-black so it doesn't wash the night sky grey;
        // Subdivision keeps the grey-blue for overcast depth.
        let far_color = if is_city {
            Color::srgba(0.03, 0.03, 0.06, 0.50)
        } else {
            Color::srgba(0.45, 0.50, 0.58, 0.50)
        };
        let far_attn_mesh = meshes.add(Mesh::from(Rectangle::new(5000.0, 1600.0)));
        let far_attn_mat = materials.add(StandardMaterial {
            base_color: far_color,
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            double_sided: true,
            cull_mode: None,
            ..default()
        });
        commands.spawn((
            Mesh3d(far_attn_mesh),
            MeshMaterial3d(far_attn_mat),
            Transform::from_xyz(0.0, -50.0, -75.0),
            ParallaxLayer { factor: 0.48 },
            Decoration,
        ));
    } else {
        // Forest: single shared attenuation at z=-38 (18% dark)
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
    }

    // ── Clouds (z = -60) ──────────────────────────────────────────────────────
    // Span x = -1500..+1600 to match mountain coverage.
    // City: clear night sky — no clouds.
    if level_id == crate::level::level_data::LevelId::City {
        return;
    }
    let cloud_configs: &[(&str, f32, f32, f32)] = &[
        ("clouds/cloud1.png", -1480.0, 100.0, 0.50),
        ("clouds/cloud3.png", -1300.0, 130.0, 0.60),
        ("clouds/cloud5.png", -1100.0, 90.0, 0.42),
        ("clouds/cloud2.png", -950.0, 60.0, 0.52),
        ("clouds/cloud7.png", -780.0, 115.0, 0.38),
        ("clouds/cloud4.png", -630.0, 145.0, 0.55),
        ("clouds/cloud6.png", -480.0, 120.0, 0.45),
        ("clouds/cloud8.png", -320.0, 80.0, 0.48),
        ("clouds/cloud2.png", -160.0, 110.0, 0.52),
        ("clouds/cloud5.png", -20.0, 135.0, 0.44),
        ("clouds/cloud1.png", 140.0, 95.0, 0.50),
        ("clouds/cloud3.png", 300.0, 120.0, 0.58),
        ("clouds/cloud7.png", 460.0, 105.0, 0.40),
        ("clouds/cloud4.png", 620.0, 130.0, 0.54),
        ("clouds/cloud6.png", 780.0, 85.0, 0.46),
        ("clouds/cloud8.png", 940.0, 115.0, 0.50),
        ("clouds/cloud1.png", 1100.0, 100.0, 0.48),
        ("clouds/cloud3.png", 1260.0, 125.0, 0.56),
        ("clouds/cloud5.png", 1420.0, 88.0, 0.44),
        ("clouds/cloud2.png", 1580.0, 115.0, 0.50),
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
            // WHY 0.80: clouds at z=-60 are distant background. High factor = tracks
            // camera = reads as far. 0.80 sits between mountains (0.85) and the mid
            // background trees (0.70), giving a natural depth ordering.
            ParallaxLayer { factor: 0.80 },
            Decoration,
        ));
    }
}

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
    // Step 100 units → ~31 trees, evenly spaced with no overlap.
    for (i, x) in (-1500..=1600i32).step_by(100).enumerate() {
        let model = dark_tree_models[i % dark_tree_models.len()];
        let scale = dark_tree_scales[i % dark_tree_scales.len()];
        commands.spawn((
            SceneRoot(asset_server.load(format!("{}#Scene0", model))),
            Transform::from_xyz(x as f32, -160.0, -80.0).with_scale(Vec3::new(scale, scale, 12.0)),
            // WHY 0.70: dark background trees at z=-80 are the mid-distance layer.
            // High factor = tracks camera = reads as far. 0.70 sits between near
            // trees (0.45) and clouds (0.80), giving a natural three-layer depth stack.
            ParallaxLayer { factor: 0.70 },
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
    // Step 163 units → ~20 trees, evenly spaced with no overlap.
    for (i, x) in (-1500..=1600i32).step_by(163).enumerate() {
        let model = tree_models[i % tree_models.len()];
        let scale = tree_scales[i % tree_scales.len()];
        // Center-anchored Trellis models need +scale/2 to ground their base.
        let y_base = -160.0;
        let center_anchored = model.contains("tree_oak")
            || model.contains("tree_pine")
            || model.contains("tree_fat")
            || model.contains("tree_default");
        let y = if center_anchored {
            y_base + scale * 0.5
        } else {
            y_base
        };
        commands.spawn((
            SceneRoot(asset_server.load(format!("{}#Scene0", model))),
            Transform::from_xyz(x as f32, y, -50.0).with_scale(Vec3::new(scale, scale, 6.0)),
            // WHY 0.45: near trees at z=-50 are the closest background layer.
            // Lower factor = less camera tracking = reads as nearer to player.
            // 0.45 gives clear separation from dark mid trees (0.70) and sky (0.95).
            ParallaxLayer { factor: 0.45 },
            Decoration,
            ParallaxBackground,
        ));
    }
}

/// Spawns subdivision/neighborhood background layers — houses, suburban trees, fences.
/// Each entity is marked `Decoration` so it gets despawned on level transitions.
/// Spans x = -1500..+1600 to cover every level's starting camera position.
pub fn spawn_subdivision_background(commands: &mut Commands, asset_server: &AssetServer) {
    // Near houses (z=-50, factor 0.45): 5-7 Jasper units tall (90-126 world units)
    let house_models = [
        "models/suburban/building-type-a.glb",
        "models/suburban/building-type-b.glb",
        "models/suburban/building-type-c.glb",
        "models/suburban/building-type-d.glb",
        "models/suburban/building-type-e.glb",
        "models/suburban/building-type-f.glb",
        "models/suburban/building-type-g.glb",
        "models/suburban/building-type-h.glb",
    ];
    let house_scales = [119.0_f32, 150.0, 113.0, 158.0, 125.0, 138.0, 115.0, 148.0];
    for (i, x) in (-1500..=1600i32).step_by(240).enumerate() {
        let model = house_models[i % house_models.len()];
        let scale = house_scales[i % house_scales.len()];
        let tint = HOUSE_TINTS[i % HOUSE_TINTS.len()];
        commands.spawn((
            SceneRoot(asset_server.load(format!("{}#Scene0", model))),
            Transform::from_xyz(x as f32, -160.0, -50.0).with_scale(Vec3::new(
                scale,
                scale,
                scale * 0.35,
            )),
            // WHY 0.45: near houses at z=-50 are the closest background layer.
            // Lower factor = less camera tracking = reads as nearer to player.
            // 0.45 gives clear separation from far houses (0.70) and sky (0.95).
            ParallaxLayer { factor: 0.45 },
            Decoration,
            ParallaxBackground,
            SceneTint::Multiply(tint),
        ));
    }

    // Far houses (z=-80, factor 0.70): slightly smaller distant row (still 5-7 range)
    let far_house_models = [
        "models/suburban/building-type-i.glb",
        "models/suburban/building-type-j.glb",
        "models/suburban/building-type-f.glb",
        "models/suburban/building-type-g.glb",
        "models/suburban/building-type-h.glb",
    ];
    let far_house_scales = [59.0_f32, 68.0, 62.0, 75.0, 64.0, 60.0, 70.0, 62.0];
    for (i, x) in (-1500..=1600i32).step_by(200).enumerate() {
        let model = far_house_models[i % far_house_models.len()];
        let scale = far_house_scales[i % far_house_scales.len()];
        // Offset by 2 so far row doesn't repeat the same color sequence as near row
        let tint = HOUSE_TINTS[(i + 2) % HOUSE_TINTS.len()];
        commands.spawn((
            SceneRoot(asset_server.load(format!("{}#Scene0", model))),
            Transform::from_xyz(x as f32, -160.0, -80.0).with_scale(Vec3::new(
                scale,
                scale,
                scale * 0.30,
            )),
            // WHY 0.70: far houses at z=-80 are the mid-distance background layer.
            // High factor = tracks camera closely = reads as farther away.
            // 0.70 sits between near houses (0.45) and clouds (0.80).
            ParallaxLayer { factor: 0.70 },
            Decoration,
            ParallaxBackground,
            SceneTint::Multiply(tint),
        ));
    }

    // Suburban trees interspersed between houses (z=-50, factor 0.45)
    let tree_models = [
        "models/suburban/tree-suburban-large.glb",
        "models/suburban/tree-suburban-small.glb",
    ];
    let tree_scales = [160.0_f32, 120.0, 180.0, 110.0, 150.0, 130.0, 170.0, 140.0];
    for (i, x) in (-1500..=1600i32).step_by(180).enumerate() {
        let model = tree_models[i % tree_models.len()];
        let scale = tree_scales[i % tree_scales.len()];
        commands.spawn((
            SceneRoot(asset_server.load(format!("{}#Scene0", model))),
            Transform::from_xyz(x as f32 + 60.0, -160.0, -50.0)
                .with_scale(Vec3::new(scale, scale, 8.0)),
            // WHY 0.45: suburban trees at z=-50 are in the near background layer.
            // Matches near houses (0.45) so the entire z=-50 layer scrolls uniformly.
            ParallaxLayer { factor: 0.45 },
            Decoration,
            ParallaxBackground,
        ));
    }

    // Fences between houses at ground level (z=-50, factor 0.45)
    for x in (-1500..=1600i32).step_by(80) {
        commands.spawn((
            SceneRoot(asset_server.load("models/suburban/fence-suburban.glb#Scene0")),
            Transform::from_xyz(x as f32, -155.0, -50.0).with_scale(Vec3::new(40.0, 30.0, 6.0)),
            // WHY 0.45: fences at z=-50 are in the near background layer.
            // Matches near houses (0.45) so the entire z=-50 layer scrolls uniformly.
            ParallaxLayer { factor: 0.45 },
            Decoration,
            ParallaxBackground,
        ));
    }
}

/// Spawns city background layers — tall skyscrapers (near) and commercial buildings (far).
/// Each entity is marked `Decoration` so it gets despawned on level transitions.
/// Spans x = -1500..+1600 to cover every level's starting camera position.
pub fn spawn_city_background(commands: &mut Commands, asset_server: &AssetServer) {
    info!("[CITY_BG] spawn_city_background called");

    // Near skyscrapers (z=-50, factor 0.38): tall, sparse — player can see through to far layer.
    // Uniform XY scaling preserves model's natural proportions (skyscrapers are narrow by design).
    // Flat Z (8.0) prevents 3D depth from showing as width under the -28° camera tilt.
    //
    // Scale sizing: camera at ground (min_y=34) sees y=-200 to y=162.
    // Building base at ground_top (-146). Max visible height = 162-(-146) = 308 units ≈ 17 tiles.
    // Min required height = 10 Jasper units = 180 world units.
    // Scales assume Kenney skyscraper models are ~4 units tall:
    //   scale 50 × 4 = 200 units (11 tiles), scale 77 × 4 = 308 units (17 tiles, fills screen).
    let skyscraper_models = [
        "models/city/building-skyscraper-a.glb",
        "models/city/building-skyscraper-b.glb",
        "models/city/building-skyscraper-c.glb",
        "models/city/building-skyscraper-d.glb",
        "models/city/building-skyscraper-e.glb",
    ];
    let skyscraper_scales = [114.0_f32, 96.0, 135.0, 88.0, 126.0, 105.0];
    // Step 280 units → 12 buildings, sparse enough to see far layer through gaps.
    // Slight Y rotation (~10°) reveals a sliver of the left face for 3D depth.
    // 30% darker via SceneTint::Multiply so they read as mid-ground, not foreground.
    let near_rotation = Quat::from_rotation_y(0.175);
    let near_tint = Color::srgb(0.7, 0.7, 0.7);
    let near_count = (-1500..=1600i32).step_by(280).count();
    info!(
        "[CITY_BG] spawning {} near skyscrapers at z=-50",
        near_count
    );
    for (i, x) in (-1500..=1600i32).step_by(280).enumerate() {
        let model = skyscraper_models[i % skyscraper_models.len()];
        let s = skyscraper_scales[i % skyscraper_scales.len()];
        info!("[CITY_BG] near[{}] model={} x={} scale={}", i, model, x, s);
        commands.spawn((
            SceneRoot(asset_server.load(format!("{}#Scene0", model))),
            Transform::from_xyz(x as f32, -146.0, -50.0)
                .with_rotation(near_rotation)
                .with_scale(Vec3::new(s, s, s * 0.3)),
            // WHY 0.45: near skyscrapers at z=-50 are the closest background layer.
            // Lower factor = more camera tracking = feels "near". 0.45 gives clear
            // separation from the far buildings (0.70) and sky (0.95).
            ParallaxLayer { factor: 0.45 },
            Decoration,
            ParallaxBackground,
            SceneTint::Multiply(near_tint),
        ));
    }

    // Far commercial buildings (z=-80, factor 0.48): shorter, denser.
    // Uniform XY scaling preserves model proportions. Flat Z (6.0).
    // Scales assume commercial models are ~2.5 units tall:
    //   scale 72 × 2.5 = 180 units (10 tiles, minimum), scale 100 × 2.5 = 250 units.
    // Tinted darker via SceneTint::Multiply to convey distance at night.
    let far_building_models = [
        "models/city/building-a.glb",
        "models/city/building-b.glb",
        "models/city/building-c.glb",
        "models/city/building-d.glb",
        "models/city/building-e.glb",
        "models/city/building-f.glb",
        "models/city/low-detail-building-a.glb",
        "models/city/low-detail-building-b.glb",
        "models/city/low-detail-building-c.glb",
    ];
    let far_scales = [
        119.0_f32, 140.0, 109.0, 133.0, 115.0, 129.0, 112.0, 123.0, 118.0,
    ];
    // Darker tint for night-time depth — buildings appear as silhouettes.
    let night_tint = Color::srgb(0.4, 0.45, 0.55);
    let far_count = (-1500..=1600i32).step_by(160).count();
    info!("[CITY_BG] spawning {} far buildings at z=-80", far_count);
    for (i, x) in (-1500..=1600i32).step_by(160).enumerate() {
        let model = far_building_models[i % far_building_models.len()];
        let s = far_scales[i % far_scales.len()];
        commands.spawn((
            SceneRoot(asset_server.load(format!("{}#Scene0", model))),
            Transform::from_xyz(x as f32, -146.0, -80.0).with_scale(Vec3::new(s, s * 1.2, 6.0)),
            // WHY 0.70: far commercial buildings at z=-80 are distant background.
            // Higher factor = more anchored to camera = reads as farther away.
            // 0.70 sits between near skyscrapers (0.45) and the sky (0.95), giving
            // a natural three-layer depth stack for the City night scene.
            ParallaxLayer { factor: 0.70 },
            Decoration,
            ParallaxBackground,
            SceneTint::Multiply(night_tint),
        ));
    }
}

// ── House tinting system ────────────────────────────────────────────────────

/// 4 house color tints — cycled across spawned houses.
const HOUSE_TINTS: [Color; 4] = [
    Color::srgb(0.96, 0.92, 0.82), // beige
    Color::srgb(0.97, 0.97, 0.95), // white
    Color::srgb(0.55, 0.73, 0.87), // azul blue
    Color::srgb(0.60, 0.76, 0.65), // cypress green
];

/// One-shot system: finds entities with `SceneTint` whose SceneRoot children
/// have loaded, clones each child's `StandardMaterial`, applies the tint,
/// and removes the `SceneTint` component so it only runs once.
pub fn apply_scene_tints(
    mut commands: Commands,
    tint_query: Query<(Entity, &SceneTint, &Children)>,
    child_query: Query<&Children>,
    mat_query: Query<&MeshMaterial3d<StandardMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, tint, top_children) in &tint_query {
        // SceneRoot children may be nested — collect all descendants.
        let mut found_any = false;
        let mut stack: Vec<Entity> = top_children.iter().collect();
        while let Some(child) = stack.pop() {
            if let Ok(mat_handle) = mat_query.get(child)
                && let Some(original) = materials.get(&mat_handle.0)
            {
                let mut modified = original.clone();
                match tint {
                    SceneTint::Multiply(color) => {
                        let [r, g, b, a] = modified.base_color.to_srgba().to_f32_array();
                        let [tr, tg, tb, _] = color.to_srgba().to_f32_array();
                        modified.base_color = Color::srgba(r * tr, g * tg, b * tb, a);
                    }
                    SceneTint::Replace(color) => {
                        modified.base_color = *color;
                        // Clear texture so the flat color is visible instead
                        // of being multiplied with the model's baked texture.
                        modified.base_color_texture = None;
                    }
                }
                let new_handle = materials.add(modified);
                commands.entity(child).insert(MeshMaterial3d(new_handle));
                found_any = true;
            }
            if let Ok(grandchildren) = child_query.get(child) {
                stack.extend(grandchildren.iter());
            }
        }
        // Only remove component once children have loaded and been tinted.
        if found_any {
            commands.entity(entity).remove::<SceneTint>();
        }
    }
}
