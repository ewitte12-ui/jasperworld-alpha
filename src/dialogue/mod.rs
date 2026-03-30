pub mod components;
pub mod systems;

use bevy::prelude::*;

use components::DialogueState;

pub struct DialoguePlugin;

impl Plugin for DialoguePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DialogueState::default())
            // spawn_test_npc removed — character-oobi.glb is not forest-appropriate
            .add_systems(
                Update,
                (
                    systems::check_dialogue_trigger,
                    systems::advance_dialogue,
                    systems::render_dialogue_box,
                )
                    .chain(),
            );
    }
}
