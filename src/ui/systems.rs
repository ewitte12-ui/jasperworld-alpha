use bevy::prelude::*;

use crate::collectibles::components::CollectionProgress;
use crate::combat::components::Health;
use crate::level::level_data::CurrentLevel;
use crate::player::components::Player;

use super::components::{HealthDisplay, HudRoot, LevelNameDisplay, StarCounter};

/// Spawns the HUD overlay (runs at Startup).
pub fn spawn_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/KenneyPixel.ttf");

    // Root node: full screen, flex column, no interaction
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            // Transparent background so the game shows through
            BackgroundColor(Color::NONE),
            HudRoot,
        ))
        .with_children(|parent| {
            // ── Top row: health (left) + level name (centre) + stars (right)
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|row| {
                    // Health display (top-left)
                    row.spawn((
                        Text::new("HP 100/100"),
                        TextFont {
                            font: font.clone(),
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.3, 0.3)),
                        HealthDisplay,
                    ));

                    // Level name (top-centre)
                    row.spawn((
                        Text::new("Forest L1"),
                        TextFont {
                            font: font.clone(),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 1.0, 0.8)),
                        LevelNameDisplay,
                    ));

                    // Star counter (top-right)
                    row.spawn((
                        Text::new("Stars 0/0"),
                        TextFont {
                            font: font.clone(),
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.9, 0.2)),
                        StarCounter,
                    ));
                });
        });
}

/// Updates the health display text from the player's Health component.
pub fn update_health_display(
    player_query: Query<&Health, With<Player>>,
    mut text_query: Query<&mut Text, With<HealthDisplay>>,
) {
    let Ok(health) = player_query.single() else {
        return;
    };
    let Ok(mut text) = text_query.single_mut() else {
        return;
    };
    **text = format!("HP {}/{}", health.current as i32, health.max as i32);
}

/// Updates the star counter text from CollectionProgress resource.
pub fn update_star_counter(
    progress: Res<CollectionProgress>,
    mut text_query: Query<&mut Text, With<StarCounter>>,
) {
    if !progress.is_changed() {
        return;
    }
    let Ok(mut text) = text_query.single_mut() else {
        return;
    };
    **text = format!(
        "Stars {}/{}",
        progress.stars_collected, progress.stars_total
    );
}

/// Updates the level name display when the level changes.
pub fn update_level_name(
    current_level: Res<CurrentLevel>,
    mut text_query: Query<&mut Text, With<LevelNameDisplay>>,
) {
    if !current_level.is_changed() {
        return;
    }
    let Ok(mut text) = text_query.single_mut() else {
        return;
    };
    let name = match current_level.level_id {
        Some(crate::level::level_data::LevelId::Forest) => "Forest",
        Some(crate::level::level_data::LevelId::Subdivision) => "Subdivision",
        Some(crate::level::level_data::LevelId::City) => "City",
        Some(crate::level::level_data::LevelId::Sanctuary) => "Sanctuary",
        None => "Forest",
    };
    let layer = current_level.layer_index + 1;
    **text = format!("{name} L{layer}");
}
