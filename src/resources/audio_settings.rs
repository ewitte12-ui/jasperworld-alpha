use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Resource, Serialize, Deserialize, Clone, Debug)]
pub struct AudioSettings {
    pub master_volume: f32,
    pub music_volume: f32,
    pub sfx_volume: f32,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            music_volume: 0.8,
            sfx_volume: 0.8,
        }
    }
}

impl AudioSettings {
    pub fn effective_music_volume(&self) -> f32 {
        self.master_volume * self.music_volume
    }

    pub fn effective_sfx_volume(&self) -> f32 {
        self.master_volume * self.sfx_volume
    }

    pub fn adjust_volume(value: &mut f32, delta: f32) {
        *value = (*value + delta).clamp(0.0, 1.0);
    }
}
