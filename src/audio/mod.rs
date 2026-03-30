pub mod components;
pub mod systems;

use bevy::prelude::*;

use systems::{
    load_audio, play_hurt_sfx, play_jump_sfx, play_kill_sfx, play_pickup_sfx, update_bgm,
};

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_audio).add_systems(
            Update,
            (
                play_jump_sfx,
                play_pickup_sfx,
                play_hurt_sfx,
                play_kill_sfx,
                update_bgm,
            ),
        );
    }
}
