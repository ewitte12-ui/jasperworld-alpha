use bevy::prelude::*;

use crate::level::level_data::{CurrentLevel, LevelId};
use crate::rendering::camera::{FillDirectionalLight, PrimaryDirectionalLight};

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
    mut fill_light_query: Query<
        &mut DirectionalLight,
        (With<FillDirectionalLight>, Without<PrimaryDirectionalLight>),
    >,
    mut ambient: ResMut<GlobalAmbientLight>,
) {
    if !current_level.is_changed() {
        return;
    }

    let theme = if current_level.layer_index == 1 {
        // Underground sublevel — use dedicated cave/subway themes.
        if current_level.level_id == Some(LevelId::City) {
            &LightingTheme::SUBWAY
        } else {
            &LightingTheme::CAVE
        }
    } else {
        match current_level.level_id {
            Some(LevelId::Forest) => &LightingTheme::FOREST,
            Some(LevelId::Subdivision) => &LightingTheme::SUBDIVISION,
            Some(LevelId::City) => &LightingTheme::CITY,
            Some(LevelId::Sanctuary) => &LightingTheme::SANCTUARY,
            None => &LightingTheme::FOREST,
        }
    };

    // Update directional light
    if let Ok(mut dir_light) = dir_light_query.single_mut() {
        dir_light.color = theme.directional_color;
        dir_light.illuminance = theme.directional_illuminance;
    }

    // Update fill light
    if let Ok(mut fill) = fill_light_query.single_mut() {
        fill.color = theme.fill_color;
        fill.illuminance = theme.fill_illuminance;
    }

    // Update global ambient light
    ambient.color = theme.ambient_color;
    ambient.brightness = theme.ambient_brightness;
}
