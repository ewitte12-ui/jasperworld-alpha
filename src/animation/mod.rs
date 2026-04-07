pub mod components;
pub mod systems;

use bevy::prelude::*;

use systems::{
    animate_player_procedural, animate_sprites, debug_animation_state,
    drive_player_animation, finalize_player_animation, setup_player_animation,
    update_enemy_anim_state, update_player_anim_state,
};

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        // Chain ordering guarantees apply_deferred runs between each system:
        //   setup_player_animation    — discovers AnimationPlayer, starts clip loading (deferred)
        //   [apply_deferred]          — commands from setup are applied
        //   finalize_player_animation — polls clip load status, builds graph when ready (deferred)
        //   [apply_deferred]          — commands from finalize are applied
        //   update_player_anim_state  — reads velocity, writes PlayerAnimState
        //   update_enemy_anim_state   — reads velocity, writes EnemyAnimState
        //   drive_player_animation    — starts/stops skeletal clips based on state
        //   animate_player_procedural — facing rotation always; full procedural only as fallback
        //   animate_sprites           — UV ticking for enemy sprite sheets
        app.add_systems(
            Update,
            (
                setup_player_animation,
                finalize_player_animation,
                update_player_anim_state,
                update_enemy_anim_state,
                drive_player_animation,
                animate_player_procedural,
                animate_sprites,
                debug_animation_state,
            )
                .chain(),
        );
    }
}
