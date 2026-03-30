use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub const NUM_SAVE_SLOTS: usize = 3;

#[derive(Resource, Default, Clone)]
pub struct SaveSlots {
    pub slots: [Option<SaveMetadata>; NUM_SAVE_SLOTS],
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SaveMetadata {
    pub slot_index: usize,
    pub timestamp: String,
    pub playtime_secs: f64,
    pub chapter: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameSaveData {
    pub metadata: SaveMetadata,
    pub player_position: (f32, f32),
    pub health: f32,
    pub inventory: Vec<String>,
    pub flags: HashMap<String, bool>,
    #[serde(default)]
    pub current_level: String,
    #[serde(default)]
    pub current_layer: String,
    #[serde(default)]
    pub puzzle_progress: HashMap<String, Vec<String>>,
}

/// Resource flag: a save was requested from the UI for this slot.
#[derive(Resource, Default)]
pub struct PendingSaveSlot(pub Option<usize>);

/// Resource flag: a load was requested from the UI for this slot.
#[derive(Resource, Default)]
pub struct PendingLoadSlot(pub Option<usize>);

impl GameSaveData {
    pub fn new_game(slot_index: usize) -> Self {
        Self {
            metadata: SaveMetadata {
                slot_index,
                timestamp: String::new(),
                playtime_secs: 0.0,
                chapter: "Chapter 1".to_string(),
            },
            player_position: (0.0, 0.0),
            health: 100.0,
            inventory: Vec::new(),
            flags: HashMap::new(),
            current_level: "Forest".to_string(),
            current_layer: "Street".to_string(),
            puzzle_progress: HashMap::new(),
        }
    }
}

fn saves_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|p| p.join("jaspersworld_test2").join("saves"))
}

fn slot_path(slot: usize) -> Option<PathBuf> {
    saves_dir().map(|p| p.join(format!("slot_{slot}.json")))
}

pub fn write_menu_save(slot: usize, data: &GameSaveData) {
    let Some(path) = slot_path(slot) else { return };
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(data) {
        let _ = fs::write(&path, json);
    }
}

pub fn read_menu_save(slot: usize) -> Option<GameSaveData> {
    let path = slot_path(slot)?;
    let data = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&data).ok()
}

/// Loads save slot metadata from disk into SaveSlots resource.
pub fn load_save_slots(mut save_slots: ResMut<SaveSlots>) {
    for i in 0..NUM_SAVE_SLOTS {
        if let Some(save) = read_menu_save(i) {
            save_slots.slots[i] = Some(save.metadata);
        }
    }
}
