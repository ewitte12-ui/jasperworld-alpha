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

use crate::puzzle::components::TransitionSet;
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
                // WHY after(TransitionSet): check_level_exit (inside TransitionSet) despawns
                // enemy entities and their GlowIndicator children in the transition frame.
                // If update_proximity_glow runs BEFORE check_level_exit in the same frame, it
                // can queue a deferred "spawn GlowIndicator" on an enemy that check_level_exit
                // then despawns — the deferred spawn completes after the despawn, leaving an
                // orphaned GlowIndicator in the next level.  Running after TransitionSet
                // guarantees the enemy's existing Children are still visible to the glow query
                // (deferred despawn not yet applied), so has_glow=true and no new spawn occurs.
                update_proximity_glow
                    .run_if(in_state(AppState::Playing))
                    .after(TransitionSet),
            );
    }
}
