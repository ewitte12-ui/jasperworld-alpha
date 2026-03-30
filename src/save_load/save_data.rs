use serde::{Deserialize, Serialize};

/// Data persisted for a single save slot.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SaveSlot {
    pub slot: u8, // 1, 2, or 3
    pub level_index: usize,
    pub layer_index: usize,
    pub player_x: f32,
    pub player_y: f32,
    pub health: f32,
    pub stars_collected: u32,
}
