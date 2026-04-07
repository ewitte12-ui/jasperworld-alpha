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
        directional_color: Color::srgb(0.35, 0.42, 0.65),
        directional_illuminance: 8000.0,
        ambient_color: Color::srgb(0.20, 0.22, 0.35),
        ambient_brightness: 150.0,
        fill_color: Color::srgb(0.85, 0.75, 0.55),
        fill_illuminance: 7000.0,
    };

    /// Underground subway — fluorescent cool-green station lighting.
    /// Low ambient with strong overhead directional for harsh industrial feel.
    pub const SUBWAY: Self = Self {
        directional_color: Color::srgb(0.75, 0.90, 0.80), // cool fluorescent green-white
        directional_illuminance: 10000.0,
        ambient_color: Color::srgb(0.15, 0.20, 0.18),
        ambient_brightness: 100.0,
        fill_color: Color::srgb(0.60, 0.75, 0.65),
        fill_illuminance: 4000.0,
    };

    /// Underground cave — warm torch/lantern glow with low ambient.
    /// High directional illuminance preserves 3D tile contrast (top-face highlights);
    /// low ambient keeps the underground feel dark between lit areas.
    pub const CAVE: Self = Self {
        directional_color: Color::srgb(0.95, 0.65, 0.35), // warm orange firelight
        directional_illuminance: 10000.0,
        ambient_color: Color::srgb(0.12, 0.08, 0.05),
        ambient_brightness: 120.0,
        fill_color: Color::srgb(0.90, 0.55, 0.30),
        fill_illuminance: 3500.0,
    };
}
