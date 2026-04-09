pub mod components;
pub mod glow;
pub mod systems;
pub mod vignette;

use bevy::prelude::*;
use glow::update_proximity_glow;
use systems::{
    emit_weather_particles, flash_level_name, flash_on_damage, tick_level_name_flash,
    tick_screen_flash, update_weather,
};
use vignette::spawn_vignette;

use crate::states::AppState;

pub struct VfxPlugin;

impl Plugin for VfxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_vignette)
            .add_systems(
                Update,
                (
                    update_weather,
                    emit_weather_particles,
                    flash_on_damage,
                    tick_screen_flash,
                    flash_level_name,
                    tick_level_name_flash,
                ),
            )
            .add_systems(
                Update,
                update_proximity_glow.run_if(in_state(AppState::Playing)),
            );
    }
}
