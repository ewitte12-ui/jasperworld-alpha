pub mod components;
pub mod systems;

use bevy::prelude::*;
use bevy_tnua::TnuaUserControlsSystems;

use crate::rendering::camera::CameraPipeline;
use crate::states::AppState;
use components::{GameProgress, TransitionSet};

pub struct PuzzlePlugin;

impl Plugin for PuzzlePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameProgress::default())
            // TransitionSet must complete before player input reads the guard
            // and before the camera pipeline reads the player position.
            .configure_sets(
                Update,
                TransitionSet
                    .before(TnuaUserControlsSystems)
                    .before(CameraPipeline::Follow),
            )
            .add_systems(
                Update,
                (
                    // Total order within TransitionSet:
                    //   tick_cooldown → check_level_exit → check_gate
                    //
                    // tick_cooldown first: clears the lock one frame after it
                    // was armed, so deferred despawns have applied before any
                    // trigger re-evaluates.
                    //
                    // check_level_exit before check_gate: if check_level_exit
                    // sets the guard, switch_layer (also in TransitionSet,
                    // ordered after check_level_exit via this chain) will see
                    // it and no-op.  Prevents two transitions in one frame.
                    //
                    // check_gate is independent but ordered after
                    // check_level_exit so the gate despawn cannot race
                    // with a level transition on the same frame.
                    systems::tick_transition_cooldown,
                    systems::check_level_exit,
                    systems::check_gate,
                )
                    .chain()
                    .in_set(TransitionSet)
                    .run_if(in_state(AppState::Playing)),
            );
    }
}
