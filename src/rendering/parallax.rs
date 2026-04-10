use bevy::platform::collections::HashMap;
use bevy::prelude::*;

use crate::level::components::Decoration;
use crate::rendering::camera::GameplayCamera;
use crate::rendering::parallax_config::{
    load_config, CityBgConfig, ForestBgConfig, SanctuaryBgConfig, SubdivisionBgConfig,
};

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
///   z =  -70  Distant mountains (Tripo mountain1/mountain2 models, factor 0.85).
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

    // ── Mountains (z = -70) — Forest only ────────────────────────────────────
    // mountain1 (X=1.0, Y=0.723, Z=1.0) — 8 tall peaks.
    // mountain2 (X=0.909, Y=0.398, Z=1.0) — 3 wider/lower variety.
    // Both center-anchored; Y offset = -176 + native_h * scale * 0.5 (base at -176).
    // Base lowered 6 units below ground (-170 → -176) to avoid floating appearance.
    if level_id == crate::level::level_data::LevelId::Forest {
        let cfg: ForestBgConfig = load_config("assets/configs/forest_bg.json")
            .expect("[SHARED_BG] failed to load assets/configs/forest_bg.json");
        for m in &cfg.mountains {
            // Center-anchored: shift up by half scaled height so base sits at -176.
            let y = -176.0 + m.native_h * m.scale * 0.5;
            commands.spawn((
                SceneRoot(asset_server.load(format!("{}#Scene0", m.model))),
                Transform::from_xyz(m.x, y, -70.0)
                    .with_scale(Vec3::new(m.scale, m.scale, 20.0)),
                // WHY 0.85: mountains at z=-70 are very distant. High factor = tracks camera
                // closely = reads as far away. 0.85 sits between clouds (0.80) and sky (0.95).
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

    // ── Attenuation plane(s) — loaded from per-level JSON config ─────────────
    // Forest: single plane at z=-38 (18% dark) covers all background equally.
    // Subdivision: two planes — light (12%) for near houses, heavy (50%) for far
    //   houses — so the far row reads as visibly more distant.
    // City: two planes — near at z=-38 (night sky tint), far at z=-75 (deep shadow).
    let attn_entries: Vec<crate::rendering::parallax_config::AttenuationEntry> = match level_id {
        crate::level::level_data::LevelId::Forest => {
            let cfg: ForestBgConfig = load_config("assets/configs/forest_bg.json")
                .expect("[SHARED_BG] failed to load assets/configs/forest_bg.json");
            cfg.attenuation
        }
        crate::level::level_data::LevelId::Subdivision => {
            let cfg: SubdivisionBgConfig = load_config("assets/configs/subdivision_bg.json")
                .expect("[SHARED_BG] failed to load assets/configs/subdivision_bg.json");
            cfg.attenuation
        }
        crate::level::level_data::LevelId::City => {
            let cfg: CityBgConfig = load_config("assets/configs/city_bg.json")
                .expect("[SHARED_BG] failed to load assets/configs/city_bg.json");
            cfg.attenuation
        }
        crate::level::level_data::LevelId::Sanctuary => {
            let cfg: SanctuaryBgConfig = load_config("assets/configs/sanctuary_bg.json")
                .expect("[SHARED_BG] failed to load assets/configs/sanctuary_bg.json");
            cfg.attenuation
        }
    };
    for entry in &attn_entries {
        let [r, g, b, a] = entry.color;
        let attn_mesh = meshes.add(Mesh::from(Rectangle::new(5000.0, 1600.0)));
        let attn_mat = materials.add(StandardMaterial {
            base_color: Color::srgba(r, g, b, a),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            double_sided: true,
            cull_mode: None,
            ..default()
        });
        commands.spawn((
            Mesh3d(attn_mesh),
            MeshMaterial3d(attn_mat),
            Transform::from_xyz(0.0, -50.0, entry.z),
            ParallaxLayer { factor: entry.factor },
            Decoration,
        ));
    }

    // ── Clouds — loaded from forest_bg.json or subdivision_bg.json ───────────
    // City: clear night sky — no clouds.
    // Sanctuary: cherry blossom scene uses its own background; no generic forest clouds.
    // Each CloudEntry carries its own z and factor so cloud depth can vary per entry.
    if matches!(
        level_id,
        crate::level::level_data::LevelId::City
            | crate::level::level_data::LevelId::Sanctuary
    ) {
        return;
    }
    // Both Forest and Subdivision share the same cloud list from forest_bg.json.
    // WHY forest_bg.json for both: clouds are sky-layer elements present in any
    // non-city/non-sanctuary level; subdivision has no separate cloud list in its JSON.
    let cfg: ForestBgConfig = load_config("assets/configs/forest_bg.json")
        .expect("[SHARED_BG] failed to load assets/configs/forest_bg.json");
    for cloud in &cfg.clouds {
        let cloud_mesh = meshes.add(Mesh::from(Rectangle::new(
            120.0 * cloud.scale,
            60.0 * cloud.scale,
        )));
        let cloud_mat = materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load(cloud.texture.clone())),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            double_sided: true,
            cull_mode: None,
            ..default()
        });
        commands.spawn((
            Mesh3d(cloud_mesh),
            MeshMaterial3d(cloud_mat),
            Transform::from_xyz(cloud.x, cloud.y, cloud.z),
            // WHY 0.80: clouds at z=-60 are distant background. High factor = tracks
            // camera = reads as far. 0.80 sits between mountains (0.85) and the mid
            // background trees (0.70), giving a natural depth ordering.
            ParallaxLayer { factor: cloud.factor },
            Decoration,
        ));
    }
}

/// Spawns the forest/nature background tree layers (dark mid + bright front).
/// Each entity is marked `Decoration` so it gets despawned on level transitions.
/// Layer bounds and model lists are driven by `assets/configs/forest_bg.json`.
pub fn spawn_nature_background(commands: &mut Commands, asset_server: &AssetServer) {
    let cfg: ForestBgConfig = load_config("assets/configs/forest_bg.json")
        .expect("[NATURE_BG] failed to load assets/configs/forest_bg.json");

    // ── Far trees (dark mid-distance canopy, z=-80) ───────────────────────────
    // Dense step (63 units) → ~50 trees. All are non-center-anchored in the config
    // so base sits directly at y with no extra offset.
    let ft = &cfg.far_trees;
    for (i, x) in (ft.x_start..=ft.x_end).step_by(ft.step).enumerate() {
        let model = &ft.models[i % ft.models.len()];
        let scale = ft.scales[i % ft.scales.len()];
        let y = if ft.center_anchored {
            ft.y + scale * 0.5
        } else {
            ft.y
        };
        commands.spawn((
            SceneRoot(asset_server.load(format!("{}#Scene0", model))),
            Transform::from_xyz(x as f32 + ft.x_offset, y, ft.z)
                .with_scale(Vec3::new(scale, scale, ft.scale_z)),
            // WHY 0.70: dark background trees at z=-80 are the mid-distance layer.
            // High factor = tracks camera = reads as far. 0.70 sits between near
            // trees (0.45) and clouds (0.80), giving a natural three-layer depth stack.
            ParallaxLayer { factor: ft.factor },
            Decoration,
            ParallaxBackground,
        ));
    }

    // ── Near trees (bright canopy framing, z=-50) ─────────────────────────────
    // Sparser step (163 units) → ~20 trees, evenly spaced with no overlap.
    // center_anchored=true: Trellis models need +scale/2 to ground their base.
    let nt = &cfg.near_trees;
    for (i, x) in (nt.x_start..=nt.x_end).step_by(nt.step).enumerate() {
        let model = &nt.models[i % nt.models.len()];
        let scale = nt.scales[i % nt.scales.len()];
        let y = if nt.center_anchored {
            nt.y + scale * 0.5
        } else {
            nt.y
        };
        commands.spawn((
            SceneRoot(asset_server.load(format!("{}#Scene0", model))),
            Transform::from_xyz(x as f32 + nt.x_offset, y, nt.z)
                .with_scale(Vec3::new(scale, scale, nt.scale_z)),
            // WHY 0.45: near trees at z=-50 are the closest background layer.
            // Lower factor = less camera tracking = reads as nearer to player.
            // 0.45 gives clear separation from dark mid trees (0.70) and sky (0.95).
            ParallaxLayer { factor: nt.factor },
            Decoration,
            ParallaxBackground,
        ));
    }
}

/// Spawns subdivision/neighborhood background layers — houses, suburban trees.
/// Each entity is marked `Decoration` so it gets despawned on level transitions.
/// All positioning data is loaded from `assets/configs/subdivision_bg.json`.
pub fn spawn_subdivision_background(commands: &mut Commands, asset_server: &AssetServer) {
    let cfg: SubdivisionBgConfig = load_config("assets/configs/subdivision_bg.json")
        .expect("[SUBDIV_BG] failed to load assets/configs/subdivision_bg.json");

    // ── Near houses (z=-50, factor 0.45) ─────────────────────────────────────
    // Tripo house models with own textures. Mixed anchoring: a-d center-anchored
    // (Y min < 0), e-k bottom-anchored (Y min = 0).
    // WHY native_h * scale * 0.5: center-anchored models sit with origin at their
    // midpoint, so we shift up by half their world-space height to align the base.
    // WHY depth_scale on X: after -90° Y rotation, local X → world Z (depth).
    // Flattening X prevents 3D depth from reading as width under the camera tilt.
    let nh = &cfg.near_houses;
    let near_rotation = Quat::from_rotation_y(nh.rotation_y);
    for (i, x) in (nh.x_start..=nh.x_end).step_by(nh.step).enumerate() {
        // Deterministic pseudo-random model selection for visual variety.
        // Uses a simple hash: multiply index by a large prime, XOR with another,
        // then mod by model count. Produces consistent but varied-looking layout.
        let model_idx = ((i.wrapping_mul(2654435761)) ^ 0xDEADBEEF) % nh.models.len();
        let entry = &nh.models[model_idx];
        let scale = nh.scales[i % nh.scales.len()];
        let y = if entry.center_anchored {
            nh.y + entry.native_h * scale * 0.5
        } else {
            nh.y
        };
        let mut entity = commands.spawn((
            SceneRoot(asset_server.load(format!("{}#Scene0", entry.path))),
            Transform::from_xyz(x as f32 + nh.x_offset, y, nh.z)
                .with_rotation(near_rotation)
                // After -90° Y rotation: local Z→world X (visible width),
                // local X→world Z (depth). Flatten X (depth), keep Z (width) full.
                .with_scale(Vec3::new(scale * nh.depth_scale, scale, scale)),
            ParallaxLayer { factor: nh.factor },
            Decoration,
            ParallaxBackground,
        ));
        if let Some([r, g, b]) = nh.tint {
            entity.insert(SceneTint::Multiply(Color::srgb(r, g, b)));
        }
    }

    // ── Far houses (z=-80, factor 0.70) ──────────────────────────────────────
    // Smaller distant row using a subset of house models.
    // Muted tint simulates aerial perspective (atmospheric fog on distant objects).
    // first_x_override: first far house shifted left so it's half off-screen at
    // the level edge; subsequent houses use normal step spacing from x_start.
    let fh = &cfg.far_houses;
    let far_rotation = Quat::from_rotation_y(fh.rotation_y);
    for (i, x) in (fh.x_start..=fh.x_end).step_by(fh.step).enumerate() {
        let final_x = if i == 0 {
            fh.first_x_override.unwrap_or(x) as f32
        } else {
            x as f32
        };
        // Deterministic pseudo-random model selection for visual variety.
        // Uses a simple hash: multiply index by a large prime, XOR with another,
        // then mod by model count. Produces consistent but varied-looking layout.
        let model_idx = ((i.wrapping_mul(2654435761)) ^ 0xDEADBEEF) % fh.models.len();
        let entry = &fh.models[model_idx];
        let scale = fh.scales[i % fh.scales.len()];
        let y = if entry.center_anchored {
            fh.y + entry.native_h * scale * 0.5
        } else {
            fh.y
        };
        let mut entity = commands.spawn((
            SceneRoot(asset_server.load(format!("{}#Scene0", entry.path))),
            Transform::from_xyz(final_x + fh.x_offset, y, fh.z)
                .with_rotation(far_rotation)
                .with_scale(Vec3::new(scale * fh.depth_scale, scale, scale)),
            ParallaxLayer { factor: fh.factor },
            Decoration,
            ParallaxBackground,
        ));
        if let Some([r, g, b]) = fh.tint {
            entity.insert(SceneTint::Multiply(Color::srgb(r, g, b)));
        }
    }

    // ── Suburban trees (z=-50, factor 0.45) ──────────────────────────────────
    // Interspersed between houses. x_offset=60.0 staggers trees to avoid aligning
    // with house spawn positions at the same x step multiples.
    let tr = &cfg.trees;
    for (i, x) in (tr.x_start..=tr.x_end).step_by(tr.step).enumerate() {
        let model = &tr.models[i % tr.models.len()];
        let scale = tr.scales[i % tr.scales.len()];
        let y = if tr.center_anchored {
            tr.y + scale * 0.5
        } else {
            tr.y
        };
        commands.spawn((
            SceneRoot(asset_server.load(format!("{}#Scene0", model))),
            Transform::from_xyz(x as f32 + tr.x_offset, y, tr.z)
                .with_scale(Vec3::new(scale, scale, tr.scale_z)),
            // WHY 0.45: suburban trees at z=-50 are in the near background layer.
            // Matches near houses (0.45) so the entire z=-50 layer scrolls uniformly.
            ParallaxLayer { factor: tr.factor },
            Decoration,
            ParallaxBackground,
        ));
    }
}

/// Spawns city background layers — tall skyscrapers (near) and commercial buildings (far).
/// Each entity is marked `Decoration` so it gets despawned on level transitions.
/// All positioning data is loaded from `assets/configs/city_bg.json`.
pub fn spawn_city_background(commands: &mut Commands, asset_server: &AssetServer) {
    info!("[CITY_BG] spawn_city_background called");

    let cfg: CityBgConfig = load_config("assets/configs/city_bg.json")
        .expect("[CITY_BG] failed to load assets/configs/city_bg.json");

    // ── Near skyscrapers (z=-50) ──────────────────────────────────────────────
    // Tall, sparse — player can see through to far layer.
    // Uniform XY scaling preserves model's natural proportions (skyscrapers are narrow).
    // depth_scale_factor on Z prevents 3D depth from showing as width under camera tilt.
    //
    // native_h_mult is a per-model scale normalizer: models that are taller or shorter
    // than the standard game-unit height use this multiplier so the base `scales` values
    // remain consistent across models. center_anchored models get +scale*0.5 Y shift.
    // Scale sizing: scale 114 × 2.88 = 328 (fills ~17 tiles). scale 88 × 4.08 = 359.
    let nb = &cfg.near_buildings;
    let near_rotation = Quat::from_rotation_y(nb.rotation_y);
    let near_count = (nb.x_start..=nb.x_end).step_by(nb.step).count();
    info!("[CITY_BG] spawning {} near skyscrapers at z={}", near_count, nb.z);
    for (i, x) in (nb.x_start..=nb.x_end).step_by(nb.step).enumerate() {
        let entry = &nb.models[i % nb.models.len()];
        let base_s = nb.scales[i % nb.scales.len()];
        // Apply native_h_mult to normalize this model's height to the standard game-unit scale.
        let s = base_s * entry.native_h_mult;
        let y_base = if entry.center_anchored {
            nb.y + s * 0.5
        } else {
            nb.y
        };
        info!("[CITY_BG] near[{}] model={} x={} scale={} y_base={}", i, entry.path, x, s, y_base);
        let mut entity = commands.spawn((
            SceneRoot(asset_server.load(format!("{}#Scene0", entry.path))),
            Transform::from_xyz(x as f32, y_base, nb.z)
                .with_rotation(near_rotation)
                .with_scale(Vec3::new(s, s, s * nb.depth_scale_factor)),
            // WHY 0.45: near skyscrapers at z=-50 are the closest background layer.
            // Lower factor = more camera tracking = feels "near". 0.45 gives clear
            // separation from the far buildings (0.70) and sky (0.95).
            ParallaxLayer { factor: nb.factor },
            Decoration,
            ParallaxBackground,
        ));
        if let Some([r, g, b]) = nb.tint {
            entity.insert(SceneTint::Multiply(Color::srgb(r, g, b)));
        }
    }

    // ── Far commercial buildings (z=-80) ─────────────────────────────────────
    // Shorter, denser. y_stretch stretches buildings vertically. scale_z is a fixed
    // flat depth value to prevent 3D depth showing as width.
    // Darker tint conveys distance at night — buildings appear as silhouettes.
    let fb = &cfg.far_buildings;
    let far_count = (fb.x_start..=fb.x_end).step_by(fb.step).count();
    info!("[CITY_BG] spawning {} far buildings at z={}", far_count, fb.z);
    for (i, x) in (fb.x_start..=fb.x_end).step_by(fb.step).enumerate() {
        let model = &fb.models[i % fb.models.len()];
        let s = fb.scales[i % fb.scales.len()];
        let mut entity = commands.spawn((
            SceneRoot(asset_server.load(format!("{}#Scene0", model))),
            Transform::from_xyz(x as f32, fb.y, fb.z)
                .with_scale(Vec3::new(s, s * fb.y_stretch, fb.scale_z)),
            // WHY 0.70: far commercial buildings at z=-80 are distant background.
            // Higher factor = more anchored to camera = reads as farther away.
            // 0.70 sits between near skyscrapers (0.45) and the sky (0.95), giving
            // a natural three-layer depth stack for the City night scene.
            ParallaxLayer { factor: fb.factor },
            Decoration,
            ParallaxBackground,
        ));
        if let Some([r, g, b]) = fb.tint {
            entity.insert(SceneTint::Multiply(Color::srgb(r, g, b)));
        }
    }
}

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

/// Spawns the Sanctuary level background layers — temple island, cherry blossom trees.
///
/// Layer stack:
///   z = -80   Temple island (far_background): distant centerpiece, factor 0.70.
///   z = -60   Far cherry blossom trees: mid-distance canopy, factor 0.55.
///   z = -30   Near cherry blossom trees: foreground framing, factor 0.35.
///
/// All positioning and model data is loaded from `assets/configs/sanctuary_bg.json`.
/// Every entity carries `Decoration` so it is despawned on level transition.
pub fn spawn_sanctuary_background(
    commands: &mut Commands,
    asset_server: &AssetServer,
    _meshes: &mut Assets<Mesh>,
    _materials: &mut Assets<StandardMaterial>,
) {
    let cfg: SanctuaryBgConfig = load_config("assets/configs/sanctuary_bg.json")
        .expect("[SANCTUARY_BG] failed to load assets/configs/sanctuary_bg.json");

    // ── Temple island (z = -80, factor 0.70) ─────────────────────────────────
    // Single wide model centered in the level. center_anchored Y: model origin is at
    // midpoint so no extra Y offset needed — y is the visual center position.
    let fb = &cfg.far_background;
    commands.spawn((
        SceneRoot(asset_server.load(format!("{}#Scene0", fb.model))),
        Transform::from_xyz(fb.x, fb.y, fb.z)
            // WHY -100°: the model is oriented along X; -100° Y rotation brings the
            // facade forward to face the camera with a slight leftward angle.
            .with_rotation(Quat::from_rotation_y(-100_f32.to_radians()))
            // WHY non-uniform scale (0.25 on X): after the Y rotation, local X
            // maps to world -Z (depth toward camera). Uniform scale causes the island's
            // depth to bleed forward over the ground plane. Crushing local X by 0.25
            // collapses that depth to a thin slab while preserving visible width
            // (local Z → world X) and height (local Y → world Y).
            .with_scale(Vec3::new(fb.scale * 0.25, fb.scale, fb.scale)),
        // WHY 0.70: temple at z=-80 is the deepest background element.
        // High factor = closely tracks camera = reads as very far away.
        // 0.70 matches the far-tree layer convention from forest/subdivision.
        ParallaxLayer { factor: fb.factor },
        Decoration,
        ParallaxBackground,
    ));

    // ── Far cherry blossom trees (z = -60, factor 0.55) ──────────────────────
    // Dense layer behind near trees — fills mid-distance with pink blossoms.
    // center_anchored=true: cherry blossom origin is at model midpoint; shift Y up
    // by scale*0.5 so the base sits at the config y position.
    let ft = &cfg.far_trees;
    for (i, x) in (ft.x_start..=ft.x_end).step_by(ft.step).enumerate() {
        let model = &ft.models[i % ft.models.len()];
        let scale = ft.scales[i % ft.scales.len()];
        let y = if ft.center_anchored {
            ft.y + scale * 0.5
        } else {
            ft.y
        };
        commands.spawn((
            SceneRoot(asset_server.load(format!("{}#Scene0", model))),
            Transform::from_xyz(x as f32 + ft.x_offset, y, ft.z)
                .with_scale(Vec3::new(scale, scale, ft.scale_z)),
            // WHY 0.55: far cherry blossom trees at z=-60 are mid-distance.
            // Sits between near trees (0.35) and temple (0.70), giving natural depth.
            ParallaxLayer { factor: ft.factor },
            Decoration,
            ParallaxBackground,
        ));
    }

    // ── Near cherry blossom trees (z = -30, factor 0.35) ─────────────────────
    // Sparse, larger — frame the gameplay area from close background.
    // center_anchored=true: same model; same Y correction applies.
    let nt = &cfg.near_trees;
    for (i, x) in (nt.x_start..=nt.x_end).step_by(nt.step).enumerate() {
        let model = &nt.models[i % nt.models.len()];
        let scale = nt.scales[i % nt.scales.len()];
        let y = if nt.center_anchored {
            nt.y + scale * 0.5
        } else {
            nt.y
        };
        commands.spawn((
            SceneRoot(asset_server.load(format!("{}#Scene0", model))),
            Transform::from_xyz(x as f32 + nt.x_offset, y, nt.z)
                .with_scale(Vec3::new(scale, scale, nt.scale_z)),
            // WHY 0.35: near trees at z=-30 are the closest background layer.
            // Low factor = more world-fixed = reads as near player.
            ParallaxLayer { factor: nt.factor },
            Decoration,
            ParallaxBackground,
        ));
    }

    // NOTE: The sky overlay (cfg.overlay) is spawned by the caller (level/mod.rs
    // Sanctuary arm) so the mesh/material assets stay in the calling scope.
    // The _meshes and _materials params are kept for signature parity with other
    // background spawn functions in case future layers need direct mesh creation here.
}
