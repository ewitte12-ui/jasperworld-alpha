use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LevelId {
    Forest,
    Subdivision,
}

/// A single layer within a level.
#[derive(Debug, Clone)]
pub struct LayerData {
    pub id: usize,
    /// Tile grid: grid[row][col], row 0 = bottom, row increases upward.
    /// Uses TileType::Empty, Solid, Platform.
    pub tiles: Vec<Vec<crate::tilemap::tilemap::TileType>>,
    /// World x of grid column 0
    pub origin_x: f32,
    /// World y of grid row 0
    pub origin_y: f32,
    /// Player spawn point for this layer
    pub spawn: Vec2,
}

impl LayerData {
    pub fn rows(&self) -> usize {
        self.tiles.len()
    }

    pub fn cols(&self) -> usize {
        self.tiles.first().map(|r| r.len()).unwrap_or(0)
    }
}

#[derive(Debug, Clone, Resource)]
pub struct LevelData {
    pub id: LevelId,
    pub layers: Vec<LayerData>,
}

/// Resource tracking current level + layer state.
#[derive(Resource, Default)]
pub struct CurrentLevel {
    pub level_id: Option<LevelId>,
    pub layer_index: usize,
}
