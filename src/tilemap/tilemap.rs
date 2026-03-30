use bevy::prelude::*;

pub const TILE_SIZE: f32 = 18.0;

/// Type of tile — determines atlas index, collision, and behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileType {
    Empty,
    Solid,    // full collision
    Platform, // one-way platform (solid on top only)
}

/// Convert grid (col, row) to world XY position (center of tile).
/// Grid origin (0,0) is at world origin. Positive y is up.
pub fn grid_to_world(col: i32, row: i32) -> Vec2 {
    Vec2::new(col as f32 * TILE_SIZE, row as f32 * TILE_SIZE)
}
