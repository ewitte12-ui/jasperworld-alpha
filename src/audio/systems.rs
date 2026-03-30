use bevy::prelude::*;

use crate::collectibles::components::CollectedEvent;
use crate::combat::components::{EnemyKillEvent, PlayerDamageEvent};
use crate::level::level_data::CurrentLevel;

use super::components::{AudioHandles, BackgroundMusic};

/// Startup system: loads all audio assets and inserts the AudioHandles resource.
pub fn load_audio(mut commands: Commands, asset_server: Res<AssetServer>) {
    let handles = AudioHandles {
        jump: asset_server.load("audio/jump1.ogg"),
        pickup: asset_server.load("audio/pickup1.ogg"),
        enemy_hit: asset_server.load("audio/creature1.ogg"),
        player_hurt: asset_server.load("audio/lose1.ogg"),
        bgm: asset_server.load("audio/music-forest.ogg"),
    };
    commands.insert_resource(handles);
}

/// Plays jump SFX when the player jumps.
/// Detects jump via Space/ArrowUp/W key just-pressed.
pub fn play_jump_sfx(
    keyboard: Res<ButtonInput<KeyCode>>,
    audio_handles: Option<Res<AudioHandles>>,
    mut commands: Commands,
) {
    let Some(handles) = audio_handles else {
        return;
    };

    let just_jumped = keyboard.just_pressed(KeyCode::Space)
        || keyboard.just_pressed(KeyCode::ArrowUp)
        || keyboard.just_pressed(KeyCode::KeyW);

    if just_jumped {
        commands.spawn((
            AudioPlayer::new(handles.jump.clone()),
            PlaybackSettings::DESPAWN,
        ));
    }
}

/// Plays pickup SFX when a collectible is collected.
pub fn play_pickup_sfx(
    mut events: MessageReader<CollectedEvent>,
    audio_handles: Option<Res<AudioHandles>>,
    mut commands: Commands,
) {
    let Some(handles) = audio_handles else {
        return;
    };

    for _ in events.read() {
        commands.spawn((
            AudioPlayer::new(handles.pickup.clone()),
            PlaybackSettings::DESPAWN,
        ));
    }
}

/// Plays hurt SFX when the player takes damage.
pub fn play_hurt_sfx(
    mut events: MessageReader<PlayerDamageEvent>,
    audio_handles: Option<Res<AudioHandles>>,
    mut commands: Commands,
) {
    let Some(handles) = audio_handles else {
        return;
    };

    for _ in events.read() {
        commands.spawn((
            AudioPlayer::new(handles.player_hurt.clone()),
            PlaybackSettings::DESPAWN,
        ));
    }
}

/// Plays enemy-hit/kill SFX when an enemy is killed.
pub fn play_kill_sfx(
    mut events: MessageReader<EnemyKillEvent>,
    audio_handles: Option<Res<AudioHandles>>,
    mut commands: Commands,
) {
    let Some(handles) = audio_handles else {
        return;
    };

    for _ in events.read() {
        commands.spawn((
            AudioPlayer::new(handles.enemy_hit.clone()),
            PlaybackSettings::DESPAWN,
        ));
    }
}

/// Detects CurrentLevel changes and swaps BGM accordingly.
pub fn update_bgm(
    current_level: Res<CurrentLevel>,
    audio_handles: Option<Res<AudioHandles>>,
    mut commands: Commands,
    bgm_query: Query<Entity, With<BackgroundMusic>>,
) {
    if !current_level.is_changed() {
        return;
    }

    let Some(handles) = audio_handles else {
        return;
    };

    if current_level.level_id.is_none() {
        return;
    }

    // Despawn existing BGM entities.
    for entity in bgm_query.iter() {
        commands.entity(entity).despawn();
    }

    // Spawn new BGM.
    commands.spawn((
        BackgroundMusic,
        AudioPlayer::new(handles.bgm.clone()),
        PlaybackSettings::LOOP,
    ));
}
