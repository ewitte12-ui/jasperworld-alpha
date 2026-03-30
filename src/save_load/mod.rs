pub mod save_data;
pub mod systems;

use bevy::prelude::*;

use crate::states::AppState;

pub struct SaveLoadPlugin;

impl Plugin for SaveLoadPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (systems::save_game, systems::load_game)
                .run_if(in_state(AppState::Playing)),
        );
    }
}
