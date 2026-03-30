use bevy::prelude::*;

/// Spawns a screen-space UI vignette using 4 semi-transparent edge strips.
/// This is camera-independent — always stays at screen edges regardless of tilt.
pub fn spawn_vignette(mut commands: Commands) {
    let edge_color = Color::srgba(0.0, 0.0, 0.0, 0.5);

    commands.spawn(Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        position_type: PositionType::Absolute,
        ..default()
    }).with_children(|parent| {
        // Top strip
        parent.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(12.0),
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                ..default()
            },
            BackgroundColor(edge_color),
        ));

        // Bottom strip
        parent.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(12.0),
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.0),
                left: Val::Px(0.0),
                ..default()
            },
            BackgroundColor(edge_color),
        ));

        // Left strip
        parent.spawn((
            Node {
                width: Val::Px(20.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                ..default()
            },
            BackgroundColor(edge_color),
        ));

        // Right strip
        parent.spawn((
            Node {
                width: Val::Px(20.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                right: Val::Px(0.0),
                ..default()
            },
            BackgroundColor(edge_color),
        ));
    });
}
