pub mod ai;
pub mod components;
pub mod spawner;

use bevy::prelude::*;

use crate::puzzle::components::GameProgress;
use crate::states::AppState;
use components::TraversalBlockoutMode;

pub struct EnemiesPlugin;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        // spawn_enemies removed from Startup — it duplicated LevelPlugin's spawn_forest_inner
        // and violated the TitleScreen spec (no gameplay entities during TitleScreen).
        // Forest enemies are now spawned exclusively by handle_new_game via spawn_forest_inner.
        //
        // When TraversalBlockoutMode is active, skip_enemies=true is passed directly to
        // spawn_level_full → spawn_entities_for_level, so enemy entities are never created.
        // No cleanup system is needed.
        app.add_systems(
            Update,
            (ai::enemy_ai, ai::enemy_ai_state_machine, ai::enemy_jump)
                .run_if(in_state(AppState::Playing))
                .run_if(not(resource_exists::<TraversalBlockoutMode>))
                .run_if(|gp: Res<GameProgress>| !gp.transition_in_progress),
        );
    }
}
