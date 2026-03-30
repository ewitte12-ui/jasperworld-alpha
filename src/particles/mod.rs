pub mod components;
pub mod systems;

use bevy::prelude::*;

use systems::{spawn_collect_burst, spawn_kill_burst, tick_particles};

pub struct ParticlesPlugin;

impl Plugin for ParticlesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (spawn_collect_burst, spawn_kill_burst, tick_particles),
        );
    }
}
