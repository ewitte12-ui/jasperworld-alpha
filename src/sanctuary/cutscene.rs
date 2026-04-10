use std::f32::consts::FRAC_PI_2;

use bevy::camera::ScalingMode;
use bevy::prelude::*;
use serde::Deserialize;

use avian2d::prelude::LinearVelocity;

use crate::level::components::Decoration;
use crate::player::components::Player;
use crate::puzzle::components::GameProgress;
use crate::rendering::camera::GameplayCamera;
use crate::rendering::parallax_config::load_config;
use crate::states::AppState;

// ── Config structs ────────────────────────────────────────────────────────────

#[derive(Deserialize, Clone)]
pub struct SanctuaryCutsceneConfig {
    pub trigger_distance: f32,
    pub zoom_height: f32,
    pub zoom_duration: f32,
    pub lines: Vec<DialogLine>,
    pub finish_prompt: String,
    /// Optional override for the camera's X target during the cutscene.
    /// Defaults to the trigger entity's X (raccoon family position) if omitted.
    #[serde(default)]
    pub camera_target_x: Option<f32>,
}

#[derive(Deserialize, Clone)]
pub struct DialogLine {
    pub speaker: String,
    pub text: String,
}

// ── Components ────────────────────────────────────────────────────────────────

/// Invisible marker entity placed at the raccoon family position in world space.
/// Used by `check_sanctuary_trigger` to compute XY distance to the player.
#[derive(Component)]
pub struct SanctuaryCutsceneTrigger;

/// Marker for the cutscene dialog box UI node (used for cleanup queries).
#[derive(Component)]
pub struct SanctuaryCutsceneBox;

/// Static GLB stand-in for Jasper spawned at the moment the cutscene triggers.
/// The real player entity is hidden for the duration; this provides the visual.
/// Tagged with `Decoration` so it is automatically despawned on any level transition.
#[derive(Component)]
pub struct SanctuaryJasperStand;

// ── Resources ─────────────────────────────────────────────────────────────────

/// Lifecycle state for the Sanctuary ending cutscene.
/// Reset on every `OnEnter(AppState::Playing)` while preserving the loaded config.
#[derive(Resource, Default)]
pub struct SanctuaryCutsceneState {
    /// Loaded from `assets/configs/sanctuary_cutscene.json` at Startup.
    pub config: Option<SanctuaryCutsceneConfig>,
    /// True once proximity trigger has fired — never fires again.
    pub triggered: bool,
    /// Index into `config.lines` for the currently displayed dialog line.
    pub current_line: usize,
    /// True after all lines are exhausted — shows the finish/end-game prompt.
    pub awaiting_finish: bool,
    /// True after the player confirms finish — prevents any further processing.
    pub finished: bool,
}

/// Inserted when the cutscene triggers; its presence suppresses `camera_follow`
/// so the cutscene camera systems take over.  Removed when the cutscene ends.
///
/// WHY resource instead of component: the camera entity is not level-scoped
/// (it persists across states), so a resource avoids coupling to entity lifetime.
#[derive(Resource)]
pub struct CutsceneCameraOverride {
    /// World X the camera should pan toward during the cutscene.
    pub target_x: f32,
    /// World Y the camera should pan toward (pre-computed with CAMERA_Y_OFFSET).
    pub target_y: f32,
    /// Target `viewport_height` for the zoom-in (e.g. 180.0 = tighter framing).
    pub zoom_height: f32,
    /// Original `viewport_height` to restore on cutscene exit (320.0).
    pub normal_height: f32,
    /// Duration in seconds of the zoom animation.
    pub zoom_duration: f32,
    /// Elapsed seconds since the zoom started.
    pub elapsed: f32,
}

// ── Systems ───────────────────────────────────────────────────────────────────

/// Reads `sanctuary_cutscene.json` at Startup and caches the config in
/// `SanctuaryCutsceneState`.  Logs a warning if the file is missing or malformed;
/// all cutscene systems early-return gracefully when config is `None`.
pub fn load_cutscene_config(mut state: ResMut<SanctuaryCutsceneState>) {
    state.config =
        load_config::<SanctuaryCutsceneConfig>("assets/configs/sanctuary_cutscene.json");
    if state.config.is_none() {
        warn!("[sanctuary] could not load sanctuary_cutscene.json — cutscene will be skipped");
    }
}

/// Cleans up the camera override and resets zoom when leaving Playing state.
/// Handles the case where the player quits mid-cutscene (e.g. pause → main menu).
pub fn cleanup_cutscene_on_exit(
    mut commands: Commands,
    mut camera_query: Query<&mut Projection, (With<Camera3d>, With<GameplayCamera>)>,
    player_query: Query<Entity, With<Player>>,
) {
    commands.remove_resource::<CutsceneCameraOverride>();
    // Restore player visibility in case the cutscene was interrupted mid-play.
    if let Ok(player) = player_query.single() {
        commands.entity(player).insert(Visibility::Inherited);
    }
    if let Ok(mut projection) = camera_query.single_mut()
        && let Projection::Orthographic(proj) = projection.as_mut()
    {
        proj.scaling_mode = ScalingMode::FixedVertical { viewport_height: 320.0 };
    }
}

/// Resets cutscene runtime state when gameplay begins (new game or reload).
/// Preserves the already-loaded config so we don't re-read the file.
/// Also resets viewport_height to 320.0 in case a previous run left it zoomed.
pub fn reset_cutscene_state(
    mut state: ResMut<SanctuaryCutsceneState>,
    mut commands: Commands,
    mut camera_query: Query<&mut Projection, (With<Camera3d>, With<GameplayCamera>)>,
    player_query: Query<Entity, With<Player>>,
) {
    let cfg = state.config.take();
    *state = SanctuaryCutsceneState::default();
    state.config = cfg;

    // Remove any leftover camera override from a previous run.
    commands.remove_resource::<CutsceneCameraOverride>();
    // Restore player visibility in case a previous run left it hidden.
    if let Ok(player) = player_query.single() {
        commands.entity(player).insert(Visibility::Inherited);
    }

    // Reset zoom to the standard gameplay viewport height.
    // WHY 320.0: matches the value in `setup_camera` (src/rendering/camera.rs).
    if let Ok(mut projection) = camera_query.single_mut()
        && let Projection::Orthographic(proj) = projection.as_mut()
    {
        proj.scaling_mode = ScalingMode::FixedVertical { viewport_height: 320.0 };
    }
}

/// Fires when the player crosses the trigger entity's X position (left → right).
/// Zeroes the player's velocity so Jasper stops immediately, then starts the
/// camera zoom-in by inserting `CutsceneCameraOverride`.
pub fn check_sanctuary_trigger(
    mut state: ResMut<SanctuaryCutsceneState>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player_query: Query<(Entity, &Transform), With<Player>>,
    mut player_vel_query: Query<&mut LinearVelocity, With<Player>>,
    trigger_query: Query<&Transform, With<SanctuaryCutsceneTrigger>>,
) {
    if state.triggered || state.finished {
        return;
    }
    // Clone config values before any mutable borrow of `state`.
    let (zoom_height, zoom_duration, cfg_target_x) = {
        let Some(cfg) = &state.config else {
            return;
        };
        (cfg.zoom_height, cfg.zoom_duration, cfg.camera_target_x)
    };
    let Ok((player_entity, player_tf)) = player_query.single() else {
        return;
    };
    let Ok(trigger_tf) = trigger_query.single() else {
        return;
    };

    // Trigger fires when Jasper crosses the trigger entity's X from the left.
    // WHY X-threshold: the trigger entity is placed at LDtk pixel x=585
    // (world x = origin_x + 585 = −432 + 585 = 153) per level design request.
    if player_tf.translation.x < trigger_tf.translation.x {
        return;
    }

    // Zero velocity so physics doesn't carry Jasper forward past the trigger point.
    if let Ok(mut vel) = player_vel_query.single_mut() {
        vel.x = 0.0;
        vel.y = 0.0;
    }

    // Hide the real player and replace with a static GLB for the duration of the
    // cutscene. Suppressing player_input alone is not enough — Tnua continues
    // applying forces and Jasper drifts. The static copy solves this cleanly.
    // WHY Decoration: auto-despawned alongside other level props on any transition.
    // WHY -FRAC_PI_2: Tripo convention — rotates the model to face the camera (-Z).
    // WHY scale 28.0: matches PLAYER_SPRITE_W so the stand-in is the same width.
    let pos = player_tf.translation;
    commands.entity(player_entity).insert(Visibility::Hidden);
    commands.spawn((
        SceneRoot(asset_server.load("models/sanctuary/jasper3D.glb#Scene0")),
        Transform::from_xyz(pos.x, pos.y, pos.z)
            // WHY -0.4: native model forward is +X. rotation_y=0 faces family (+X);
            // rotation_y=-PI/2 faces camera (+Z). -0.4 rad (~23°) gives a mostly
            // right-facing pose with a slight angle toward the viewer.
            .with_rotation(Quat::from_rotation_y(-0.6))
            .with_scale(Vec3::splat(28.0)),
        SanctuaryJasperStand,
        Decoration,
    ));

    state.triggered = true;
    state.current_line = 0;

    // Camera pan target X: use JSON override if set, otherwise fall back to the
    // trigger entity's X (raccoon family position).
    // WHY override: level design may want to frame a nearby landmark (e.g. the
    // cherry blossom tree at x=9.0) rather than the family sprite directly.
    let target_x = cfg_target_x.unwrap_or(trigger_tf.translation.x);
    // Camera Y: trigger Y + CAMERA_Y_OFFSET (80).
    // camera_clamp is suppressed during the cutscene so the camera can reach
    // this Y, which is below the normal level clamp minimum of ~34.0.
    let target_y = trigger_tf.translation.y + 80.0;

    commands.insert_resource(CutsceneCameraOverride {
        target_x,
        target_y,
        zoom_height,
        normal_height: 320.0,
        zoom_duration,
        elapsed: 0.0,
    });
}

/// Smooth camera pan toward the cutscene focus point.
///
/// Runs in `CameraPipeline::Follow` (same stage as `camera_follow`) so it feeds
/// into the existing Clamp → Snap → Parallax pipeline unchanged.
/// Only active while `CutsceneCameraOverride` exists.
pub fn cutscene_camera_follow(
    mut camera_query: Query<&mut Transform, (With<Camera3d>, With<GameplayCamera>)>,
    override_res: Res<CutsceneCameraOverride>,
    time: Res<Time>,
) {
    let Ok(mut cam) = camera_query.single_mut() else {
        return;
    };
    let target = Vec3::new(override_res.target_x, override_res.target_y, 100.0);
    // Lerp at CAMERA_LERP_SPEED = 5.0 — same rate as normal camera_follow.
    let factor = (5.0 * time.delta_secs()).clamp(0.0, 1.0);
    cam.translation = cam.translation.lerp(target, factor);
}

/// Animates `viewport_height` from 320.0 → `zoom_height` using smooth-step easing.
///
/// Runs in `CameraPipeline::Follow` alongside `cutscene_camera_follow`.
/// Writing to `Projection` (separate from `Transform`) means Bevy can run both
/// camera systems in parallel within the Follow set — no ordering needed.
pub fn cutscene_camera_zoom(
    mut camera_query: Query<&mut Projection, (With<Camera3d>, With<GameplayCamera>)>,
    mut override_res: ResMut<CutsceneCameraOverride>,
    time: Res<Time>,
) {
    let Ok(mut projection) = camera_query.single_mut() else {
        return;
    };
    override_res.elapsed += time.delta_secs();
    let t = (override_res.elapsed / override_res.zoom_duration).clamp(0.0, 1.0);
    // Smooth-step easing: t² × (3 − 2t)  — gentle ease-in, ease-out.
    let t_smooth = t * t * (3.0 - 2.0 * t);
    let height = override_res.normal_height
        + (override_res.zoom_height - override_res.normal_height) * t_smooth;

    if let Projection::Orthographic(proj) = projection.as_mut() {
        proj.scaling_mode = ScalingMode::FixedVertical { viewport_height: height };
    }
}

/// Advances the cutscene dialog on Space or E press.
/// When `awaiting_finish` is true, transitions the game to the main menu instead.
pub fn advance_cutscene(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<SanctuaryCutsceneState>,
    mut commands: Commands,
    mut game_progress: ResMut<GameProgress>,
    mut next_state: ResMut<NextState<AppState>>,
    box_query: Query<Entity, With<SanctuaryCutsceneBox>>,
) {
    if !state.triggered || state.finished {
        return;
    }
    if !keyboard.just_pressed(KeyCode::Space) && !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }
    let Some(cfg) = state.config.clone() else {
        return;
    };

    if state.awaiting_finish {
        // Player confirmed the ending — clean up and transition to main menu.
        state.finished = true;
        game_progress.game_complete = true;
        for entity in &box_query {
            commands.entity(entity).despawn();
        }
        commands.remove_resource::<CutsceneCameraOverride>();
        next_state.set(AppState::MainMenu);
        return;
    }

    state.current_line += 1;
    if state.current_line >= cfg.lines.len() {
        state.awaiting_finish = true;
    }

    // Despawn the existing box.  `render_cutscene_box` (chained after this system)
    // sees an empty box_query next frame and spawns a fresh box at the correct
    // speaker-side position for the new line.
    for entity in &box_query {
        commands.entity(entity).despawn();
    }
}

/// Spawns the cutscene dialog box UI panel when none exists, or despawns it
/// when the cutscene ends.  Re-spawned fresh on each line change (after the
/// prior despawn flushes) so the speaker-side positioning updates correctly.
pub fn render_cutscene_box(
    mut commands: Commands,
    state: Res<SanctuaryCutsceneState>,
    box_query: Query<Entity, With<SanctuaryCutsceneBox>>,
) {
    let active = state.triggered && !state.finished;

    if !active {
        for entity in &box_query {
            commands.entity(entity).despawn();
        }
        return;
    }

    let Some(cfg) = &state.config else {
        return;
    };

    // Box already exists — wait for the next frame after a despawn to re-spawn.
    if !box_query.is_empty() {
        return;
    }

    // ── Determine content and layout based on current state ───────────────────

    let (speaker_label, dialog_text, side_left, full_width) = if state.awaiting_finish {
        // Finish prompt: centered full-width box, no speaker label.
        ("".to_string(), cfg.finish_prompt.clone(), true, true)
    } else {
        let line = &cfg.lines[state.current_line];
        let label = match line.speaker.as_str() {
            "jasper" => "Jasper:".to_string(),
            "family" => "Family:".to_string(),
            other => format!("{other}:"),
        };
        // Jasper speaks from the left (he approaches from left); family from right.
        let is_left = line.speaker == "jasper";
        (label, line.text.clone(), is_left, false)
    };

    let (left_val, right_val, max_w) = if full_width {
        (Val::Px(16.0), Val::Px(16.0), Val::Auto)
    } else if side_left {
        (Val::Px(16.0), Val::Auto, Val::Px(660.0))
    } else {
        (Val::Auto, Val::Px(16.0), Val::Px(660.0))
    };

    let hint = if state.awaiting_finish {
        "[Space / E] to finish"
    } else {
        "[Space / E] to continue"
    };

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(40.0),
                left: left_val,
                right: right_val,
                width: Val::Px(480.0),
                max_width: max_w,
                padding: UiRect::all(Val::Px(16.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.15, 0.88)),
            SanctuaryCutsceneBox,
        ))
        .with_children(|parent| {
            // Speaker label (not shown on finish prompt).
            if !speaker_label.is_empty() {
                parent.spawn((
                    Text::new(speaker_label),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::srgb(1.0, 0.85, 0.4)),
                ));
            }

            parent.spawn((
                Text::new(dialog_text),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            parent.spawn((
                Text::new(hint),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));
        });
}
