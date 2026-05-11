use bevy::prelude::*;

use super::level_data::LevelId;

/// Marks a door entity that transitions to a specific layer.
/// Retained for compatibility with cleanup queries; no entities carry it now
/// that sub-level doors are disabled.
#[derive(Component)]
pub struct TransitionDoor {
    pub target_layer: usize,
}

/// No-op — sub-level transition doors are disabled. Kept so the Startup system
/// registration in level/mod.rs continues to compile.
pub fn spawn_transition_doors(_commands: Commands, _asset_server: Res<AssetServer>) {}

/// Sub-level transition doors are disabled — each level is now a single layer.
/// The function is retained as a no-op so existing call sites keep compiling.
pub fn spawn_doors_for_level(
    _commands: &mut Commands,
    _asset_server: &AssetServer,
    _level_id: LevelId,
) {
}
