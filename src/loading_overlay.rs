use bevy::prelude::*;

use crate::level::level_data::{CurrentLevel, LevelId};
use crate::states::AppState;

/// Marker for the loading-overlay UI root.
#[derive(Component)]
struct LoadingOverlayRoot;

/// Timer that drives when the overlay is visible. The duration is a heuristic
/// for how long async asset loads typically take after a level transition in
/// the WASM build — long enough to hide most of the visible "popping in"
/// without lingering past the playable state.
#[derive(Resource)]
struct LoadingOverlayState {
    timer: Timer,
    active: bool,
}

impl Default for LoadingOverlayState {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(2.0, TimerMode::Once),
            active: false,
        }
    }
}

pub struct LoadingOverlayPlugin;

impl Plugin for LoadingOverlayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LoadingOverlayState>()
            .add_systems(OnEnter(AppState::Playing), show_overlay)
            .add_systems(
                Update,
                (detect_level_change, tick_overlay).run_if(in_state(AppState::Playing)),
            )
            .add_systems(OnExit(AppState::Playing), despawn_overlay);
    }
}

fn spawn_overlay_entity(commands: &mut Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            ZIndex(i32::MAX),
            LoadingOverlayRoot,
        ))
        .with_children(|parent: &mut ChildSpawnerCommands| {
            parent.spawn((
                Text::new("Loading…"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn show_overlay(
    mut commands: Commands,
    mut state: ResMut<LoadingOverlayState>,
    existing: Query<&LoadingOverlayRoot>,
) {
    if existing.is_empty() {
        spawn_overlay_entity(&mut commands);
    }
    state.timer = Timer::from_seconds(2.0, TimerMode::Once);
    state.active = true;
}

fn detect_level_change(
    current: Res<CurrentLevel>,
    mut last: Local<Option<LevelId>>,
    mut commands: Commands,
    mut state: ResMut<LoadingOverlayState>,
    existing: Query<&LoadingOverlayRoot>,
) {
    if current.level_id != *last {
        *last = current.level_id;
        if current.level_id.is_some() {
            if existing.is_empty() {
                spawn_overlay_entity(&mut commands);
            }
            state.timer = Timer::from_seconds(2.0, TimerMode::Once);
            state.active = true;
        }
    }
}

fn tick_overlay(
    time: Res<Time>,
    mut state: ResMut<LoadingOverlayState>,
    mut commands: Commands,
    existing: Query<Entity, With<LoadingOverlayRoot>>,
) {
    if !state.active {
        return;
    }
    state.timer.tick(time.delta());
    if state.timer.just_finished() {
        state.active = false;
        for e in &existing {
            commands.entity(e).despawn();
        }
    }
}

fn despawn_overlay(
    mut commands: Commands,
    existing: Query<Entity, With<LoadingOverlayRoot>>,
    mut state: ResMut<LoadingOverlayState>,
) {
    for e in &existing {
        commands.entity(e).despawn();
    }
    state.active = false;
}
