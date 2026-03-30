pub mod components;
pub mod systems;

use bevy::prelude::*;

use systems::{animate_sprites, update_enemy_anim_state, update_player_anim_state};

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_player_anim_state,
                update_enemy_anim_state,
                animate_sprites,
            )
                .chain(),
        );
    }
}
