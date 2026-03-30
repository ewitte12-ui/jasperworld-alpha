use bevy::prelude::*;

pub struct LightingTheme {
    pub directional_color: Color,
    pub directional_illuminance: f32,
    pub ambient_color: Color,
    pub ambient_brightness: f32,
}

impl LightingTheme {
    pub const FOREST: Self = Self {
        directional_color: Color::srgb(1.0, 0.95, 0.8), // warm daylight
        directional_illuminance: 12000.0,
        ambient_color: Color::srgb(0.4, 0.5, 0.4),
        ambient_brightness: 300.0,
    };
    pub const SUBDIVISION: Self = Self {
        directional_color: Color::srgb(1.0, 0.98, 0.95), // neutral daylight
        directional_illuminance: 14000.0,
        ambient_color: Color::srgb(0.5, 0.5, 0.5),
        ambient_brightness: 400.0,
    };
    pub const CITY: Self = Self {
        directional_color: Color::srgb(0.8, 0.85, 1.0), // cool city light
        directional_illuminance: 8000.0,
        ambient_color: Color::srgb(0.3, 0.3, 0.45),
        ambient_brightness: 500.0, // more ambient from neon/screens
    };
    pub const SANCTUARY: Self = Self {
        directional_color: Color::srgb(1.0, 0.85, 0.5), // golden hour
        directional_illuminance: 10000.0,
        ambient_color: Color::srgb(0.5, 0.45, 0.3),
        ambient_brightness: 350.0,
    };
}
