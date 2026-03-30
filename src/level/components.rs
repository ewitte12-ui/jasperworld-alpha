use bevy::prelude::*;

/// Marker for all tile entities — used for cleanup on level switch.
#[derive(Component)]
pub struct TileEntity;

/// Marks a tile as a one-way platform (passable from below).
#[derive(Component)]
pub struct OneWayPlatform;

/// A transition zone that switches the player to another layer.
#[derive(Component)]
pub struct LayerTransition {
    pub target_layer: usize,
    pub spawn_x: f32,
    pub spawn_y: f32,
}

/// Marker for decorative props that should be despawned on level reset.
#[derive(Component)]
pub struct Decoration;

/// Marker for foreground decoration entities (z > 0).
/// Used by debug layer isolation and any system that needs to query
/// foreground visual elements independently from background decorations.
#[derive(Component)]
pub struct ForegroundDecoration;

/// Marker for Subdivision-biome-only entities (static parallax plates).
/// Allows targeted despawn without touching shared or City-specific entities.
#[derive(Component)]
pub struct SubdivisionOnly;
