use bevy::prelude::*;
use std::fs;
use std::path::PathBuf;

use crate::collectibles::components::CollectionProgress;
use crate::combat::components::Health;
use crate::level::level_data::CurrentLevel;
use crate::player::components::Player;
use crate::puzzle::components::GameProgress;

use super::save_data::SaveSlot;

fn save_path() -> PathBuf {
    let base = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("jaspersworld").join("save1.json")
}

/// Saves the game on Ctrl+S.
///
/// Blocked during transitions: saving mid-transition would capture an
/// inconsistent snapshot (old level despawned, new level not yet applied).
pub fn save_game(
    keyboard: Res<ButtonInput<KeyCode>>,
    player_query: Query<(&Transform, &Health), With<Player>>,
    current_level: Res<CurrentLevel>,
    progress: Res<CollectionProgress>,
    game_progress: Res<GameProgress>,
) {
    if game_progress.transition_in_progress {
        return;
    }

    let ctrl = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);

    if !(ctrl && keyboard.just_pressed(KeyCode::KeyS)) {
        return;
    }

    let (player_tf, health) = match player_query.single() {
        Ok(v) => v,
        Err(_) => {
            warn!("save_game: no player found");
            return;
        }
    };

    let slot = SaveSlot {
        slot: 1,
        level_index: game_progress.current_level_index,
        layer_index: current_level.layer_index,
        player_x: player_tf.translation.x,
        player_y: player_tf.translation.y,
        health: health.current,
        stars_collected: progress.stars_collected,
    };

    let path = save_path();
    if let Some(parent) = path.parent()
        && let Err(e) = fs::create_dir_all(parent)
    {
        error!("save_game: could not create directory: {e}");
        return;
    }

    match serde_json::to_string_pretty(&slot) {
        Ok(json) => match fs::write(&path, json) {
            Ok(_) => info!("Game saved to {}", path.display()),
            Err(e) => error!("save_game: write failed: {e}"),
        },
        Err(e) => error!("save_game: serialise failed: {e}"),
    }
}

/// Loads the game on Ctrl+L and restores what can be restored without a full
/// level reload (player position, health, star count).
///
/// Blocked during transitions: loading mid-transition could teleport the
/// player into a zone where the old trigger entity has not yet despawned.
pub fn load_game(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&mut Transform, &mut Health), With<Player>>,
    mut progress: ResMut<CollectionProgress>,
    game_progress: Res<GameProgress>,
) {
    if game_progress.transition_in_progress {
        return;
    }

    let ctrl = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);

    if !(ctrl && keyboard.just_pressed(KeyCode::KeyL)) {
        return;
    }

    let path = save_path();
    let json = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            warn!("load_game: could not read save file: {e}");
            return;
        }
    };

    let slot: SaveSlot = match serde_json::from_str(&json) {
        Ok(s) => s,
        Err(e) => {
            error!("load_game: deserialise failed: {e}");
            return;
        }
    };

    match player_query.single_mut() {
        Ok((mut tf, mut health)) => {
            tf.translation.x = slot.player_x;
            tf.translation.y = slot.player_y;
            health.current = slot.health;
        }
        Err(_) => {
            warn!("load_game: no player found");
        }
    }

    progress.stars_collected = slot.stars_collected;

    info!("Game loaded!");
}
