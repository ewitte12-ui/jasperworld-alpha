// ============================================================================
// DEBUG ONLY — This entire module is excluded from release builds.
//
// To enable:  set "enabled": true in debug_start.json at the project root.
// To disable: set "enabled": false  OR  delete debug_start.json.
// To remove entirely:
//   1. Delete src/debug/
//   2. Remove `#[cfg(debug_assertions)] pub mod debug;` from lib.rs
//   3. Remove `#[cfg(debug_assertions)] app.add_plugins(...)` from main.rs
// ============================================================================

use bevy::prelude::*;
use serde::Deserialize;
use std::fs;

use crate::collectibles::components::CollectionProgress;
use crate::enemies::components::TraversalBlockoutMode;
use crate::level::{
    level_data::{CurrentLevel, LevelId},
    spawn_level_full,
};
use crate::player::components::Player;
use crate::states::{AppState, NewGameRequested};

// ── Config file format (debug_start.json) ────────────────────────────────────

#[derive(Deserialize)]
struct DebugStartFile {
    enabled: bool,
    level: String,
    layer: usize,
    /// When true, all enemies are suppressed for clean traversal testing.
    /// Defaults to false when the field is absent from the JSON.
    #[serde(default)]
    traversal_blockout: bool,
}

// ── Resource — present only when debug start is active ───────────────────────

#[derive(Resource)]
pub struct DebugStartConfig {
    pub level_id: LevelId,
    pub layer_index: usize,
    pub traversal_blockout: bool,
}

// ── Plugin ───────────────────────────────────────────────────────────────────

/// Debug layer visibility mode.  Cycles with F9 (debug builds only).
/// Does not modify Z values, spawning, parallax, or camera — only
/// toggles `Visibility` on existing entities.
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugViewMode {
    #[default]
    Normal,
    ForegroundOnly,
    BackgroundOnly,
    GameplayOnly,
    ForegroundPlusGameplay,
}

pub struct DebugStartPlugin;

impl Plugin for DebugStartPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugViewMode>()
            .add_systems(Startup, read_debug_config)
            .add_systems(OnEnter(AppState::Playing), apply_debug_start)
            .add_systems(
                Update,
                (toggle_layer_visibility, log_decoration_counts)
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

// ── Startup: read config file ─────────────────────────────────────────────────

fn read_debug_config(
    mut commands: Commands,
    mut new_game: ResMut<NewGameRequested>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let Ok(text) = fs::read_to_string("debug_start.json") else {
        return; // no file — debug mode is off, normal boot continues
    };

    let cfg: DebugStartFile = match serde_json::from_str(&text) {
        Ok(c) => c,
        Err(e) => {
            warn!("[DEBUG] debug_start.json parse error: {e} — ignoring");
            return;
        }
    };

    if !cfg.enabled {
        return;
    }

    let level_id = match cfg.level.as_str() {
        "Forest" => LevelId::Forest,
        "Subdivision" => LevelId::Subdivision,
        "City" => LevelId::City,
        "Sanctuary" => LevelId::Sanctuary,
        other => {
            warn!("[DEBUG] unknown level \"{other}\" — defaulting to Forest");
            LevelId::Forest
        }
    };

    // Prevent handle_new_game from spawning Forest over the debug level.
    new_game.0 = false;

    if cfg.traversal_blockout {
        commands.insert_resource(TraversalBlockoutMode);
        info!("[DEBUG] traversal_blockout active — enemies suppressed");
    }

    commands.insert_resource(DebugStartConfig {
        level_id,
        layer_index: cfg.layer,
        traversal_blockout: cfg.traversal_blockout,
    });

    // Skip TitleScreen and all menus.
    next_state.set(AppState::Playing);

    info!(
        "[DEBUG] debug_start active → level={:?}  layer={}",
        level_id, cfg.layer
    );
}

// ── OnEnter(Playing): spawn target level ─────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn apply_debug_start(
    debug_cfg: Option<Res<DebugStartConfig>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut progress: ResMut<CollectionProgress>,
    mut current_level: ResMut<CurrentLevel>,
    mut player_query: Query<&mut Transform, With<Player>>,
    mut game_progress: ResMut<crate::puzzle::components::GameProgress>,
) {
    let Some(cfg) = debug_cfg else { return };

    // Clear any stale transition state — debug_start bypasses handle_new_game,
    // which normally resets GameProgress.  Without this, a lingering
    // transition_in_progress from a prior session would permanently lock input.
    *game_progress = crate::puzzle::components::GameProgress::default();

    // Route through the canonical spawn path — same tilemap, entity, door,
    // decoration, and resource initialization used by handle_new_game and
    // check_level_exit. No ad-hoc spawning here.
    let spawn = spawn_level_full(
        &mut commands,
        &mut meshes,
        &mut materials,
        &asset_server,
        &mut progress,
        &mut current_level,
        cfg.level_id,
        cfg.layer_index,
        cfg.traversal_blockout, // skip_enemies when blockout is active
    );

    if let Ok(mut tf) = player_query.single_mut() {
        tf.translation.x = spawn.x;
        tf.translation.y = spawn.y;
    }

    // Remove config so re-entering Playing (e.g. from Pause) does not
    // re-spawn the level a second time.
    commands.remove_resource::<DebugStartConfig>();

    info!(
        "[DEBUG] spawned level={:?}  layer={}  player_spawn={:?}",
        cfg.level_id, cfg.layer_index, spawn
    );
}

// ── Layer visibility toggle (F9) ────────────────────────────────────────────
//
// Cycles: Normal → ForegroundOnly → BackgroundOnly → GameplayOnly
//         → ForegroundPlusGameplay → Normal
//
// Layer classification by Z:
//   Background: z < -5   (sky, skyline, mass planes, mid-ground, street buildings)
//   Gameplay:   -5 ≤ z ≤ 2  (tiles, enemies, collectibles, street props)
//   Foreground: z > 2    (ground props, framing elements, weather, player)
//
// Player and camera are always visible regardless of mode.

impl DebugViewMode {
    fn next(self) -> Self {
        match self {
            Self::Normal => Self::ForegroundOnly,
            Self::ForegroundOnly => Self::BackgroundOnly,
            Self::BackgroundOnly => Self::GameplayOnly,
            Self::GameplayOnly => Self::ForegroundPlusGameplay,
            Self::ForegroundPlusGameplay => Self::Normal,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Normal => "Normal (all visible)",
            Self::ForegroundOnly => "Foreground only",
            Self::BackgroundOnly => "Background only",
            Self::GameplayOnly => "Gameplay only",
            Self::ForegroundPlusGameplay => "Foreground + Gameplay",
        }
    }
}

#[allow(clippy::type_complexity)]
fn toggle_layer_visibility(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut mode: ResMut<DebugViewMode>,
    // WHY GlobalTransform: SceneRoot children have local Transform at z≈0.
    // Only GlobalTransform reflects the parent's z position, which is the
    // actual rendering depth used for layer classification.
    mut query: Query<
        (
            &GlobalTransform,
            &mut Visibility,
            Has<crate::level::components::ForegroundDecoration>,
            Has<crate::rendering::parallax::ParallaxBackground>,
            Has<crate::rendering::parallax::ParallaxLayer>,
            Has<crate::level::components::Decoration>,
            Has<crate::level::components::TileEntity>,
            Has<crate::enemies::components::Enemy>,
            Has<crate::collectibles::components::Collectible>,
        ),
        (
            Without<crate::player::components::Player>,
            Without<Camera3d>,
        ),
    >,
) {
    // Direct mode selection: F1–F5.  F9 cycles sequentially.
    let new_mode = if keyboard.just_pressed(KeyCode::F1) {
        Some(DebugViewMode::Normal)
    } else if keyboard.just_pressed(KeyCode::F2) {
        Some(DebugViewMode::ForegroundOnly)
    } else if keyboard.just_pressed(KeyCode::F3) {
        Some(DebugViewMode::BackgroundOnly)
    } else if keyboard.just_pressed(KeyCode::F4) {
        Some(DebugViewMode::GameplayOnly)
    } else if keyboard.just_pressed(KeyCode::F5) {
        Some(DebugViewMode::ForegroundPlusGameplay)
    } else if keyboard.just_pressed(KeyCode::F9) {
        Some(mode.next())
    } else {
        None
    };

    let Some(m) = new_mode else { return };
    if *mode == m {
        return;
    }
    *mode = m;

    info!("[DEBUG] Layer isolation: {}", m.label());

    for (
        global_tf,
        mut vis,
        is_fg_deco,
        is_parallax_bg,
        is_parallax,
        is_deco,
        is_tile,
        is_enemy,
        is_collectible,
    ) in query.iter_mut()
    {
        // Classify by marker first, Z-range fallback for untagged entities.
        //
        // Foreground: ForegroundDecoration marker, or z > 2 (ground props,
        //             framing elements without the marker, weather particles).
        // Background: ParallaxBackground or ParallaxLayer marker, or
        //             Decoration without ForegroundDecoration at z < -5.
        // Gameplay:   TileEntity, Enemy, Collectible, or -5 ≤ z ≤ 2.
        let z = global_tf.translation().z;

        let is_foreground = is_fg_deco || z > 2.0;
        let is_background = is_parallax_bg
            || is_parallax
            || (is_deco && !is_fg_deco && z < -5.0)
            || (!is_fg_deco && !is_tile && !is_enemy && !is_collectible && z < -5.0);
        let is_gameplay =
            is_tile || is_enemy || is_collectible || ((-5.0..=2.0).contains(&z) && !is_foreground);

        let visible = match m {
            DebugViewMode::Normal => true,
            DebugViewMode::ForegroundOnly => is_foreground,
            DebugViewMode::BackgroundOnly => is_background,
            DebugViewMode::GameplayOnly => is_gameplay,
            DebugViewMode::ForegroundPlusGameplay => is_foreground || is_gameplay,
        };

        *vis = if visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

// ── Decoration count logging ─────────────────────────────────────────────────
// Logs entity counts by marker when the current level changes.
// Fires once per level load, not every frame.

fn log_decoration_counts(
    current_level: Res<crate::level::level_data::CurrentLevel>,
    fg_query: Query<Entity, With<crate::level::components::ForegroundDecoration>>,
    deco_query: Query<Entity, With<crate::level::components::Decoration>>,
    parallax_bg_query: Query<Entity, With<crate::rendering::parallax::ParallaxBackground>>,
    parallax_query: Query<Entity, With<crate::rendering::parallax::ParallaxLayer>>,
    tile_query: Query<Entity, With<crate::level::components::TileEntity>>,
) {
    if !current_level.is_changed() {
        return;
    }

    let level_name = match current_level.level_id {
        Some(LevelId::Forest) => "Forest",
        Some(LevelId::Subdivision) => "Subdivision",
        Some(LevelId::City) => "City",
        Some(LevelId::Sanctuary) => "Sanctuary",
        None => "None",
    };

    info!(
        "[DEBUG] Decoration counts for {level_name}:\n\
         \x20 ForegroundDecoration: {}\n\
         \x20 Decoration (all):     {}\n\
         \x20 ParallaxBackground:   {}\n\
         \x20 ParallaxLayer:        {}\n\
         \x20 TileEntity:           {}",
        fg_query.iter().count(),
        deco_query.iter().count(),
        parallax_bg_query.iter().count(),
        parallax_query.iter().count(),
        tile_query.iter().count(),
    );
}
