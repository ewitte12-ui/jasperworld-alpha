use bevy::prelude::*;

pub struct LightingTheme {
    pub directional_color: Color,
    pub directional_illuminance: f32,
    pub ambient_color: Color,
    pub ambient_brightness: f32,
    pub fill_color: Color,
    pub fill_illuminance: f32,
}

impl LightingTheme {
    pub const FOREST: Self = Self {
        directional_color: Color::srgb(1.0, 0.95, 0.8), // warm daylight
        directional_illuminance: 12000.0,
        ambient_color: Color::srgb(0.4, 0.5, 0.4),
        ambient_brightness: 300.0,
        fill_color: Color::srgb(0.75, 0.85, 1.0),
        fill_illuminance: 3500.0,
    };

    /// Overcast rainy daylight — cooler tones, reduced illuminance.
    pub const SUBDIVISION: Self = Self {
        directional_color: Color::srgb(0.75, 0.80, 0.88), // cool grey-blue
        directional_illuminance: 7000.0,
        ambient_color: Color::srgb(0.35, 0.40, 0.50),
        ambient_brightness: 250.0,
        fill_color: Color::srgb(0.70, 0.75, 0.85),
        fill_illuminance: 3000.0,
    };

    /// Night-time moonlight — deep blue tones, very dim.
    /// Collectibles use emissive materials so they glow independent of scene lighting.
    pub const CITY: Self = Self {
        directional_color: Color::srgb(0.40, 0.45, 0.60),
        directional_illuminance: 4000.0,
        ambient_color: Color::srgb(0.20, 0.22, 0.35),
        ambient_brightness: 250.0,
        fill_color: Color::srgb(0.85, 0.75, 0.55),
        fill_illuminance: 5500.0,
    };
}
