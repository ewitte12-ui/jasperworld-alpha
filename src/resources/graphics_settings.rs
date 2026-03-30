use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub const RESOLUTIONS: [(f32, f32); 4] = [
    (1280.0, 720.0),
    (1600.0, 900.0),
    (1920.0, 1080.0),
    (2560.0, 1440.0),
];

#[derive(Resource, Serialize, Deserialize, Clone, Debug)]
pub struct GraphicsSettings {
    pub resolution_index: usize,
    pub fullscreen: bool,
    pub vsync: bool,
}

impl Default for GraphicsSettings {
    fn default() -> Self {
        Self {
            resolution_index: 0, // 1280x720 for windowed default
            fullscreen: false,
            vsync: true,
        }
    }
}

impl GraphicsSettings {
    pub fn resolution(&self) -> (f32, f32) {
        RESOLUTIONS[self.resolution_index]
    }

    pub fn cycle_resolution_forward(&mut self) {
        self.resolution_index = (self.resolution_index + 1) % RESOLUTIONS.len();
    }

    pub fn cycle_resolution_backward(&mut self) {
        if self.resolution_index == 0 {
            self.resolution_index = RESOLUTIONS.len() - 1;
        } else {
            self.resolution_index -= 1;
        }
    }
}
