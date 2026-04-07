pub mod config;
pub mod systems;

use bevy::prelude::*;
use systems::update_lighting;

pub struct LightingPlugin;

impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_lighting);
    }
}
