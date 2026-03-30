use bevy::prelude::*;

use crate::puzzle::components::GameProgress;
use crate::states::AppState;

/// Toggles between Playing and Paused on Escape.
/// The full pause menu UI is handled by MenuPlugin (menu/mod.rs).
///
/// Blocked during transitions: `toggle_pause` and transition systems
/// (`check_level_exit`, `switch_layer`) all run unordered in `Update`.
/// Both may call `next_state.set()` — with `NextState` the last writer
/// wins.  Checking `transition_in_progress` prevents a pause request
/// from overwriting a pending state change (e.g. `AppState::MainMenu`
/// on game complete).
pub fn toggle_pause(
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    game_progress: Option<Res<GameProgress>>,
) {
    if game_progress
        .as_ref()
        .is_some_and(|gp| gp.transition_in_progress)
    {
        return;
    }

    if keyboard.just_pressed(KeyCode::Escape) {
        match state.get() {
            AppState::Playing => next_state.set(AppState::Paused),
            AppState::Paused => next_state.set(AppState::Playing),
            _ => {}
        }
    }
}
