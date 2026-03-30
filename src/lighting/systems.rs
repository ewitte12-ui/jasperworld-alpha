use bevy::prelude::*;

use crate::level::level_data::{CurrentLevel, LevelId};
use crate::rendering::camera::PrimaryDirectionalLight;

use super::config::LightingTheme;

/// Updates the DirectionalLight and GlobalAmbientLight to match the current level's theme.
///
/// WHY PrimaryDirectionalLight filter: two DirectionalLight entities exist in the scene
/// (primary sun + fill light in camera.rs).  A bare `Query<&mut DirectionalLight>`
/// returns `Err(MultipleEntities)` from `single_mut()`, silently skipping every
/// lighting theme change.  Filtering on `With<PrimaryDirectionalLight>` gives a
/// unique match so the theme actually applies.
pub fn update_lighting(
    current_level: Res<CurrentLevel>,
    mut dir_light_query: Query<&mut DirectionalLight, With<PrimaryDirectionalLight>>,
    mut ambient: ResMut<GlobalAmbientLight>,
) {
    if !current_level.is_changed() {
        return;
    }

    let theme = match current_level.level_id {
        Some(LevelId::Forest) => &LightingTheme::FOREST,
        Some(LevelId::Subdivision) => &LightingTheme::SUBDIVISION,
        Some(LevelId::City) => &LightingTheme::CITY,
        None => &LightingTheme::FOREST,
    };

    // Update directional light
    if let Ok(mut dir_light) = dir_light_query.single_mut() {
        dir_light.color = theme.directional_color;
        dir_light.illuminance = theme.directional_illuminance;
    }

    // Update global ambient light
    ambient.color = theme.ambient_color;
    ambient.brightness = theme.ambient_brightness;
}

/// Startup system — spawns a few small point lights to simulate torches on underground platforms.
pub fn spawn_point_lights(mut commands: Commands) {
    // Orange torch-like point lights for underground/lower layers
    let torch_positions = [
        Vec3::new(100.0, -90.0, 5.0),
        Vec3::new(-80.0, -90.0, 5.0),
        Vec3::new(200.0, -90.0, 5.0),
    ];

    for pos in torch_positions {
        commands.spawn((
            PointLight {
                color: Color::srgb(1.0, 0.5, 0.1),
                intensity: 50000.0,
                radius: 0.5,
                range: 60.0,
                shadows_enabled: false,
                ..default()
            },
            Transform::from_translation(pos),
        ));
    }
}
