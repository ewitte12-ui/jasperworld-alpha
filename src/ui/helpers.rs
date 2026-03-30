use super::components::{MenuButtonAction, VolumeChannel};
use super::styles::*;
use bevy::prelude::*;

/// Spawn a standard menu button with text and an action component.
pub fn spawn_button(
    parent: &mut ChildSpawnerCommands,
    text: &str,
    action: MenuButtonAction,
) -> Entity {
    parent
        .spawn((
            Button,
            Node {
                width: BUTTON_WIDTH,
                height: BUTTON_HEIGHT,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(COLOR_BUTTON_NORMAL),
            action,
        ))
        .with_children(|btn: &mut ChildSpawnerCommands| {
            btn.spawn((
                Text::new(text),
                TextFont {
                    font_size: FONT_SIZE_BUTTON,
                    ..default()
                },
                TextColor(COLOR_TEXT),
            ));
        })
        .id()
}

/// Spawn a small square button (for +/- controls).
pub fn spawn_small_button(
    parent: &mut ChildSpawnerCommands,
    text: &str,
    action: MenuButtonAction,
) -> Entity {
    parent
        .spawn((
            Button,
            Node {
                width: BUTTON_SMALL_WIDTH,
                height: BUTTON_SMALL_HEIGHT,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(COLOR_BUTTON_NORMAL),
            action,
        ))
        .with_children(|btn: &mut ChildSpawnerCommands| {
            btn.spawn((
                Text::new(text),
                TextFont {
                    font_size: FONT_SIZE_BODY,
                    ..default()
                },
                TextColor(COLOR_TEXT),
            ));
        })
        .id()
}

/// Spawn a tab button.
pub fn spawn_tab_button(
    parent: &mut ChildSpawnerCommands,
    text: &str,
    action: MenuButtonAction,
    active: bool,
) -> Entity {
    parent
        .spawn((
            Button,
            Node {
                width: BUTTON_TAB_WIDTH,
                height: BUTTON_TAB_HEIGHT,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(if active {
                COLOR_BUTTON_ACTIVE_TAB
            } else {
                COLOR_BUTTON_NORMAL
            }),
            action,
        ))
        .with_children(|btn: &mut ChildSpawnerCommands| {
            btn.spawn((
                Text::new(text),
                TextFont {
                    font_size: FONT_SIZE_BODY,
                    ..default()
                },
                TextColor(COLOR_TEXT),
            ));
        })
        .id()
}

/// Spawn a volume control row: Label [-] value [+]
pub fn spawn_volume_row(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    value: f32,
    channel: VolumeChannel,
    display_marker: impl Component,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(10.0),
            ..default()
        })
        .with_children(|row: &mut ChildSpawnerCommands| {
            row.spawn((
                Text::new(label),
                TextFont {
                    font_size: FONT_SIZE_BODY,
                    ..default()
                },
                TextColor(COLOR_TEXT),
                Node {
                    width: Val::Px(120.0),
                    ..default()
                },
            ));

            spawn_small_button(row, "-", MenuButtonAction::VolumeDown(channel));

            row.spawn((
                Text::new(format!("{}%", (value * 100.0) as i32)),
                TextFont {
                    font_size: FONT_SIZE_BODY,
                    ..default()
                },
                TextColor(COLOR_TEXT),
                Node {
                    width: Val::Px(60.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                display_marker,
            ));

            spawn_small_button(row, "+", MenuButtonAction::VolumeUp(channel));
        });
}

/// Spawn a toggle row: Label [ON/OFF button]
pub fn spawn_toggle_row(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    value: bool,
    action: MenuButtonAction,
    display_marker: impl Component,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(10.0),
            ..default()
        })
        .with_children(|row: &mut ChildSpawnerCommands| {
            row.spawn((
                Text::new(label),
                TextFont {
                    font_size: FONT_SIZE_BODY,
                    ..default()
                },
                TextColor(COLOR_TEXT),
                Node {
                    width: Val::Px(120.0),
                    ..default()
                },
            ));

            row.spawn((
                Button,
                Node {
                    width: Val::Px(100.0),
                    height: BUTTON_SMALL_HEIGHT,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(COLOR_BUTTON_NORMAL),
                action,
            ))
            .with_children(|btn: &mut ChildSpawnerCommands| {
                btn.spawn((
                    Text::new(if value { "ON" } else { "OFF" }),
                    TextFont {
                        font_size: FONT_SIZE_BODY,
                        ..default()
                    },
                    TextColor(COLOR_TEXT),
                    display_marker,
                ));
            });
        });
}

