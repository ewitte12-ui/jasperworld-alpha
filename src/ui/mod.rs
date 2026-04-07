pub mod components;
pub mod helpers;
pub mod pause;
pub mod styles;
pub mod systems;

use bevy::prelude::*;

use crate::states::AppState;
use components::HudRoot;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, systems::spawn_hud).add_systems(
            Update,
            (
                systems::update_health_display.run_if(in_state(AppState::Playing)),
                systems::update_star_counter.run_if(in_state(AppState::Playing)),
                systems::update_level_name.run_if(in_state(AppState::Playing)),
                pause::toggle_pause
                    .run_if(in_state(AppState::Playing).or(in_state(AppState::Paused))),
                manage_hud_visibility,
            ),
        );
    }
}

/// Shows the HUD only when in Playing or Paused states.
fn manage_hud_visibility(
    state: Res<State<AppState>>,
    mut query: Query<&mut Visibility, With<HudRoot>>,
) {
    let visible = matches!(state.get(), AppState::Playing | AppState::Paused);
    for mut vis in &mut query {
        *vis = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}
