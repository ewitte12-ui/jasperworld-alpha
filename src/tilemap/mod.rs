pub mod autotile;
pub mod spawn;
#[allow(clippy::module_inception)]
pub mod tilemap;

use bevy::prelude::*;

pub struct TilemapPlugin;

impl Plugin for TilemapPlugin {
    fn build(&self, _app: &mut App) {}
}
