use avian2d::prelude::LinearVelocity;
use bevy::prelude::*;

use crate::collectibles::components::{Collectible, CollectionProgress};
use crate::enemies::components::Enemy;
use crate::level::components::{Decoration, TileEntity};
use crate::level::doors::TransitionDoor;
use crate::level::level_data::CurrentLevel;
use crate::level::spawn_level_full;
use crate::player::components::Player;
use crate::states::AppState;

use super::components::{GameProgress, LevelExit, LevelGate};

/// Despawns all LevelGate entities once all stars are collected.
pub fn check_gate(
    mut commands: Commands,
    progress: Res<CollectionProgress>,
    gate_query: Query<Entity, With<LevelGate>>,
) {
    if progress.stars_total > 0 && progress.stars_collected >= progress.stars_total {
        for entity in &gate_query {
            commands.entity(entity).despawn();
        }
    }
}

/// Single owner of level-to-level scene transitions during gameplay.
///
/// This is the ONLY system that may initiate a mid-gameplay level transition.
/// Other systems may observe state (`GameProgress`, `CurrentLevel`) but must
/// not trigger transitions themselves.
///
/// Entry-point initializers (`handle_new_game`, `apply_debug_start`) run on
/// `OnEnter(Playing)` and are not mid-gameplay transitions.
/// Layer switches (`switch_layer`) swap tiles within the same level and are
/// not level transitions.
#[allow(clippy::too_many_arguments)]
pub fn check_level_exit(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut player_query: Query<(&mut Transform, &mut LinearVelocity), With<Player>>,
    exit_query: Query<(&Transform, &LevelExit), Without<Player>>,
    tile_entities: Query<Entity, With<TileEntity>>,
    decoration_entities: Query<Entity, With<Decoration>>,
    enemy_entities: Query<Entity, With<Enemy>>,
    collectible_entities: Query<Entity, With<Collectible>>,
    door_entities: Query<Entity, With<TransitionDoor>>,
    exit_entities: Query<Entity, With<LevelExit>>,
    mut game_progress: ResMut<GameProgress>,
    mut collection_progress: ResMut<CollectionProgress>,
    mut current_level: ResMut<CurrentLevel>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    // One-shot guard: reject if a transition is already in progress or game is complete.
    if game_progress.transition_in_progress || game_progress.game_complete {
        return;
    }

    let player_pos = {
        let Ok((pt, _)) = player_query.single() else {
            return;
        };
        pt.translation.truncate()
    };

    for (exit_transform, level_exit) in &exit_query {
        let exit_pos = exit_transform.translation.truncate();
        let delta = (player_pos - exit_pos).abs();
        if delta.x < level_exit.half_extents.x && delta.y < level_exit.half_extents.y {
            // Lock immediately — all further overlap checks are rejected.
            game_progress.transition_in_progress = true;
            game_progress.current_level_index += 1;

            // Despawn all gameplay entities from the old level.
            // Gate entities are already despawned by check_gate (all stars
            // collected is required before the exit is reachable).
            for entity in tile_entities
                .iter()
                .chain(decoration_entities.iter())
                .chain(enemy_entities.iter())
                .chain(collectible_entities.iter())
                .chain(door_entities.iter())
                .chain(exit_entities.iter())
            {
                commands.entity(entity).despawn();
            }

            *collection_progress = CollectionProgress::default();

            if game_progress.current_level_index >= 3 {
                game_progress.game_complete = true;
                // Game is over — clear immediately; no next level to guard.
                game_progress.transition_in_progress = false;
                game_progress.transition_cooldown = 0;
                next_state.set(AppState::MainMenu);
                return;
            }

            // Spawn next level via the canonical shared path.
            let next_level = level_exit.next_level;
            let spawn = spawn_level_full(
                &mut commands,
                &mut meshes,
                &mut materials,
                &asset_server,
                &mut collection_progress,
                &mut current_level,
                next_level,
                0,
                false,
            );

            if let Ok((mut player_tf, mut player_vel)) = player_query.single_mut() {
                player_tf.translation.x = spawn.x;
                player_tf.translation.y = spawn.y;
                // Zero velocity so entry vector does not carry into the new level.
                *player_vel = LinearVelocity::ZERO;
            }

            // Arm cooldown — tick_transition_cooldown will clear the lock
            // after 1 frame, once deferred despawn commands have applied.
            game_progress.transition_cooldown = 1;
            break;
        }
    }
}

/// Decrements `transition_cooldown` and clears `transition_in_progress` when
/// the cooldown reaches zero.
///
/// WHY a separate system: transition systems issue deferred `commands.despawn()`
/// for old trigger entities.  Those commands apply at the end of the frame.
/// If `transition_in_progress` were cleared in the same frame, a second overlap
/// check could (in theory) match against the not-yet-despawned entity.  Holding
/// the lock for 1 extra frame guarantees the old entity is gone before re-entry
/// is allowed.
///
/// Safety net: if `transition_in_progress` is `true` but `transition_cooldown`
/// is already 0 (should not happen — indicates a missed cooldown arm), the flag
/// is cleared immediately to prevent a permanent hang.
///
/// Must run every `Update` frame while `AppState::Playing`.
pub fn tick_transition_cooldown(mut game_progress: ResMut<GameProgress>) {
    if !game_progress.transition_in_progress {
        return;
    }
    if game_progress.transition_cooldown == 0 {
        // Invariant violation: flag is set but cooldown was never armed.
        // Clear unconditionally to prevent a permanent hang.
        warn!("[TRANSITION] cooldown=0 with transition_in_progress=true — clearing stuck lock");
        game_progress.transition_in_progress = false;
        return;
    }
    game_progress.transition_cooldown -= 1;
    if game_progress.transition_cooldown == 0 {
        game_progress.transition_in_progress = false;
    }
}
