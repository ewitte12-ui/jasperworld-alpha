use bevy::prelude::*;
use bevy::window::{PresentMode, WindowMode};

use crate::resources::control_bindings::keycode_display_name;
use crate::resources::graphics_settings::RESOLUTIONS;
use crate::resources::{
    AudioSettings, ControlBindings, GameAction, GameSaveData, GraphicsSettings, PendingLoadSlot,
    PendingSaveSlot, RebindingState, SaveSlots, load_save_slots, read_menu_save, write_menu_save,
};
use crate::states::{
    AppState, SaveLoadMode, SaveLoadReturnState, SettingsReturnState, SettingsTab,
};
use crate::ui::components::*;
use crate::ui::helpers::*;
use crate::ui::styles::*;

// ── Marker components for each menu screen ───────────────────────────────────

#[derive(Component)]
struct TitleScreenRoot;

#[derive(Component)]
struct MainMenuRoot;

#[derive(Component)]
struct PauseMenuRoot;

#[derive(Component)]
struct SettingsMenuRoot;

#[derive(Component)]
struct SaveLoadMenuRoot;

// ── Plugin ───────────────────────────────────────────────────────────────────

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        // Title screen
        app.add_systems(OnEnter(AppState::TitleScreen), setup_title_screen)
            .add_systems(
                OnExit(AppState::TitleScreen),
                despawn_with::<TitleScreenRoot>,
            )
            .add_systems(
                Update,
                handle_title_input.run_if(in_state(AppState::TitleScreen)),
            );

        // Main menu
        app.add_systems(OnEnter(AppState::MainMenu), setup_main_menu)
            .add_systems(OnExit(AppState::MainMenu), despawn_with::<MainMenuRoot>)
            .add_systems(
                Update,
                (
                    handle_navigation_click.run_if(in_state(AppState::MainMenu)),
                    handle_button_interaction_colors.run_if(in_state(AppState::MainMenu)),
                ),
            );

        // Pause menu
        app.add_systems(OnEnter(AppState::Paused), setup_pause_menu)
            .add_systems(OnExit(AppState::Paused), despawn_with::<PauseMenuRoot>)
            .add_systems(
                Update,
                (
                    handle_navigation_click.run_if(in_state(AppState::Paused)),
                    handle_button_interaction_colors.run_if(in_state(AppState::Paused)),
                ),
            );

        // Settings
        app.add_systems(OnEnter(AppState::Settings), setup_settings_menu)
            .add_systems(
                OnExit(AppState::Settings),
                (despawn_with::<SettingsMenuRoot>, save_settings).chain(),
            )
            .add_systems(
                Update,
                (
                    handle_navigation_click.run_if(in_state(AppState::Settings)),
                    handle_settings_adjustment.run_if(in_state(AppState::Settings)),
                    handle_button_interaction_colors.run_if(in_state(AppState::Settings)),
                    update_tab_visibility.run_if(in_state(AppState::Settings)),
                    capture_rebind_key.run_if(in_state(AppState::Settings)),
                ),
            );

        // Save/load menu
        app.add_systems(Startup, load_save_slots)
            .add_systems(OnEnter(AppState::SaveLoadMenu), setup_save_load_menu)
            .add_systems(
                OnExit(AppState::SaveLoadMenu),
                despawn_with::<SaveLoadMenuRoot>,
            )
            .add_systems(
                Update,
                (
                    handle_navigation_click.run_if(in_state(AppState::SaveLoadMenu)),
                    handle_button_interaction_colors.run_if(in_state(AppState::SaveLoadMenu)),
                    handle_pending_save_load.run_if(in_state(AppState::Playing)),
                ),
            );

        // Safety-net: on macOS the app hangs if window entities aren't despawned
        // on the main thread before the render thread drops its Arc<Window>.
        // Re-send AppExit every frame and despawn all windows once quit is set.
        app.add_systems(Last, enforce_quit_exit);
    }
}

// ── Generic despawn helper ────────────────────────────────────────────────────

fn despawn_with<T: Component>(mut commands: Commands, q: Query<Entity, With<T>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

/// Safety-net for clean macOS shutdown.
///
/// On macOS, bevy_winit's `despawn_windows` must drop window surfaces on the
/// main thread.  If the window entity is never despawned, the render thread
/// holds the last Arc and the app hangs on exit.  This system runs in `Last`
/// and, once `QuitRequested` is set, re-sends `AppExit` every frame and
/// despawns all window entities so `despawn_windows` can do its cleanup.
fn enforce_quit_exit(
    quit: Res<crate::states::QuitRequested>,
    mut exit: MessageWriter<AppExit>,
    windows: Query<Entity, With<Window>>,
    mut commands: Commands,
) {
    if !quit.0 {
        return;
    }
    exit.write(AppExit::Success);
    for entity in &windows {
        commands.entity(entity).despawn();
    }
}

// ── Title Screen ─────────────────────────────────────────────────────────────

fn setup_title_screen(mut commands: Commands) {
    // Transparent background so the 3D forest scene (rendered by the title camera
    // at order: 1) shows through.  Text is anchored to the bottom of the screen.
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexEnd,
                align_items: AlignItems::Center,
                padding: UiRect::bottom(Val::Px(48.0)),
                row_gap: SECTION_GAP,
                ..default()
            },
            BackgroundColor(Color::NONE),
            TitleScreenRoot,
        ))
        .with_children(|parent: &mut ChildSpawnerCommands| {
            parent.spawn((
                Text::new("JASPER'S WORLD"),
                TextFont {
                    font_size: FONT_SIZE_TITLE,
                    ..default()
                },
                TextColor(COLOR_TEXT_TITLE),
            ));

            parent.spawn((
                Text::new("Press any key to continue"),
                TextFont {
                    font_size: FONT_SIZE_BODY,
                    ..default()
                },
                TextColor(COLOR_TEXT_SUBTITLE),
            ));
        });
}

fn handle_title_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keys.get_just_pressed().next().is_some() || mouse.get_just_pressed().next().is_some() {
        next_state.set(AppState::MainMenu);
    }
}

// ── Main Menu ─────────────────────────────────────────────────────────────────

fn setup_main_menu(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: MENU_GAP,
                ..default()
            },
            BackgroundColor(COLOR_BACKGROUND),
            MainMenuRoot,
        ))
        .with_children(|parent: &mut ChildSpawnerCommands| {
            parent.spawn((
                Text::new("JASPER'S WORLD"),
                TextFont {
                    font_size: FONT_SIZE_TITLE,
                    ..default()
                },
                TextColor(COLOR_TEXT_TITLE),
                Node {
                    margin: UiRect::bottom(SECTION_GAP),
                    ..default()
                },
            ));

            spawn_button(parent, "New Game", MenuButtonAction::NewGame);
            spawn_button(parent, "Load Game", MenuButtonAction::LoadGame);
            spawn_button(parent, "Settings", MenuButtonAction::Settings);
            spawn_button(parent, "Quit", MenuButtonAction::Quit);
        });
}

// ── Pause Menu ────────────────────────────────────────────────────────────────

fn setup_pause_menu(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: MENU_GAP,
                ..default()
            },
            BackgroundColor(COLOR_OVERLAY),
            PauseMenuRoot,
        ))
        .with_children(|parent: &mut ChildSpawnerCommands| {
            parent.spawn((
                Text::new("PAUSED"),
                TextFont {
                    font_size: FONT_SIZE_HEADING,
                    ..default()
                },
                TextColor(COLOR_TEXT_TITLE),
                Node {
                    margin: UiRect::bottom(SECTION_GAP),
                    ..default()
                },
            ));

            spawn_button(parent, "Resume", MenuButtonAction::Resume);
            spawn_button(parent, "Settings", MenuButtonAction::Settings);
            spawn_button(parent, "Save Game", MenuButtonAction::SaveGame);
            spawn_button(parent, "Main Menu", MenuButtonAction::MainMenu);
        });
}

// ── Settings Menu ─────────────────────────────────────────────────────────────

fn setup_settings_menu(
    mut commands: Commands,
    graphics: Res<GraphicsSettings>,
    audio: Res<AudioSettings>,
    bindings: Res<ControlBindings>,
) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(40.0)),
                row_gap: SECTION_GAP,
                ..default()
            },
            BackgroundColor(COLOR_BACKGROUND),
            SettingsMenuRoot,
        ))
        .with_children(|parent: &mut ChildSpawnerCommands| {
            parent.spawn((
                Text::new("Settings"),
                TextFont {
                    font_size: FONT_SIZE_HEADING,
                    ..default()
                },
                TextColor(COLOR_TEXT_TITLE),
            ));

            // Tab row
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(8.0),
                    ..default()
                })
                .with_children(|tabs: &mut ChildSpawnerCommands| {
                    spawn_tab_button(tabs, "Graphics", MenuButtonAction::TabGraphics, true);
                    spawn_tab_button(tabs, "Audio", MenuButtonAction::TabAudio, false);
                    spawn_tab_button(tabs, "Controls", MenuButtonAction::TabControls, false);
                });

            // Tab content panels
            spawn_graphics_content(parent, &graphics);
            spawn_audio_content(parent, &audio);
            spawn_controls_content(parent, &bindings);

            spawn_button(parent, "Back", MenuButtonAction::Back);
        });
}

fn spawn_graphics_content(parent: &mut ChildSpawnerCommands, graphics: &GraphicsSettings) {
    let res = RESOLUTIONS[graphics.resolution_index];
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: MENU_GAP,
                ..default()
            },
            GraphicsTabContent,
        ))
        .with_children(|content: &mut ChildSpawnerCommands| {
            // Resolution row
            content
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(10.0),
                    ..default()
                })
                .with_children(|row: &mut ChildSpawnerCommands| {
                    row.spawn((
                        Text::new("Resolution"),
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

                    spawn_small_button(row, "<", MenuButtonAction::ResolutionPrev);

                    row.spawn((
                        Text::new(format!("{}x{}", res.0 as i32, res.1 as i32)),
                        TextFont {
                            font_size: FONT_SIZE_BODY,
                            ..default()
                        },
                        TextColor(COLOR_TEXT),
                        Node {
                            width: Val::Px(140.0),
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        ResolutionDisplay,
                    ));

                    spawn_small_button(row, ">", MenuButtonAction::ResolutionNext);
                });

            spawn_toggle_row(
                content,
                "Fullscreen",
                graphics.fullscreen,
                MenuButtonAction::ToggleFullscreen,
                FullscreenDisplay,
            );

            spawn_toggle_row(
                content,
                "VSync",
                graphics.vsync,
                MenuButtonAction::ToggleVsync,
                VsyncDisplay,
            );
        });
}

fn spawn_audio_content(parent: &mut ChildSpawnerCommands, audio: &AudioSettings) {
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: MENU_GAP,
                display: Display::None,
                ..default()
            },
            AudioTabContent,
        ))
        .with_children(|content: &mut ChildSpawnerCommands| {
            spawn_volume_row(
                content,
                "Master",
                audio.master_volume,
                VolumeChannel::Master,
                VolumeDisplay(VolumeChannel::Master),
            );
            spawn_volume_row(
                content,
                "Music",
                audio.music_volume,
                VolumeChannel::Music,
                VolumeDisplay(VolumeChannel::Music),
            );
            spawn_volume_row(
                content,
                "SFX",
                audio.sfx_volume,
                VolumeChannel::Sfx,
                VolumeDisplay(VolumeChannel::Sfx),
            );
        });
}

fn spawn_controls_content(parent: &mut ChildSpawnerCommands, bindings: &ControlBindings) {
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: MENU_GAP,
                display: Display::None,
                ..default()
            },
            ControlsTabContent,
        ))
        .with_children(|content: &mut ChildSpawnerCommands| {
            for action in GameAction::ALL {
                let key = bindings.key_for(action);
                content
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(10.0),
                        ..default()
                    })
                    .with_children(|row: &mut ChildSpawnerCommands| {
                        row.spawn((
                            Text::new(action.display_name()),
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
                                width: Val::Px(120.0),
                                height: BUTTON_SMALL_HEIGHT,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(COLOR_BUTTON_NORMAL),
                            MenuButtonAction::RebindAction(action),
                        ))
                        .with_children(
                            |btn: &mut ChildSpawnerCommands| {
                                btn.spawn((
                                    Text::new(keycode_display_name(key)),
                                    TextFont {
                                        font_size: FONT_SIZE_BODY,
                                        ..default()
                                    },
                                    TextColor(COLOR_TEXT),
                                    KeyBindingDisplay(action),
                                ));
                            },
                        );
                    });
            }
        });
}

// ── Save/Load Menu ────────────────────────────────────────────────────────────

fn setup_save_load_menu(
    mut commands: Commands,
    mode: Res<SaveLoadMode>,
    save_slots: Res<SaveSlots>,
) {
    use crate::resources::save_data::NUM_SAVE_SLOTS;

    let title_text = match *mode {
        SaveLoadMode::Save => "Save Game",
        SaveLoadMode::Load => "Load Game",
    };

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: MENU_GAP,
                ..default()
            },
            BackgroundColor(COLOR_BACKGROUND),
            SaveLoadMenuRoot,
        ))
        .with_children(|parent: &mut ChildSpawnerCommands| {
            parent.spawn((
                Text::new(title_text),
                TextFont {
                    font_size: FONT_SIZE_HEADING,
                    ..default()
                },
                TextColor(COLOR_TEXT_TITLE),
                Node {
                    margin: UiRect::bottom(SECTION_GAP),
                    ..default()
                },
            ));

            for i in 0..NUM_SAVE_SLOTS {
                let slot_text = match &save_slots.slots[i] {
                    Some(meta) => format!(
                        "Slot {} - {} ({}s)",
                        i + 1,
                        meta.chapter,
                        meta.playtime_secs as i32
                    ),
                    None => format!("Slot {} - Empty", i + 1),
                };

                let bg = if save_slots.slots[i].is_some() {
                    COLOR_SLOT_OCCUPIED
                } else {
                    COLOR_SLOT_EMPTY
                };

                parent
                    .spawn((
                        Button,
                        Node {
                            width: BUTTON_WIDTH,
                            height: Val::Px(65.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(bg),
                        MenuButtonAction::SaveSlot(i),
                    ))
                    .with_children(|btn: &mut ChildSpawnerCommands| {
                        btn.spawn((
                            Text::new(slot_text),
                            TextFont {
                                font_size: FONT_SIZE_BODY,
                                ..default()
                            },
                            TextColor(COLOR_TEXT),
                            SaveSlotDisplay(i),
                        ));
                    });
            }

            parent.spawn(Node {
                height: Val::Px(8.0),
                ..default()
            });
            spawn_button(parent, "Back", MenuButtonAction::Back);
        });
}

// ── Button Interaction Color ──────────────────────────────────────────────────

#[allow(clippy::type_complexity)]
pub fn handle_button_interaction_colors(
    mut query: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, mut bg) in &mut query {
        *bg = match interaction {
            Interaction::Pressed => BackgroundColor(COLOR_BUTTON_PRESSED),
            Interaction::Hovered => BackgroundColor(COLOR_BUTTON_HOVERED),
            Interaction::None => BackgroundColor(COLOR_BUTTON_NORMAL),
        };
    }
}

// ── Navigation Click Handler ──────────────────────────────────────────────────

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn handle_navigation_click(
    query: Query<(&Interaction, &MenuButtonAction), (Changed<Interaction>, With<Button>)>,
    current_state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut next_tab: ResMut<NextState<SettingsTab>>,
    mut settings_return: ResMut<SettingsReturnState>,
    mut save_load_mode: ResMut<SaveLoadMode>,
    mut save_load_return: ResMut<SaveLoadReturnState>,
    mut exit: MessageWriter<AppExit>,
    save_slots: Res<SaveSlots>,
    mut pending_save: ResMut<PendingSaveSlot>,
    mut pending_load: ResMut<PendingLoadSlot>,
    mut new_game: ResMut<crate::states::NewGameRequested>,
    mut quit_requested: ResMut<crate::states::QuitRequested>,
) {
    // Once quit is requested, reject all navigation — the app is shutting down.
    if quit_requested.0 {
        return;
    }

    for (interaction, action) in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match *action {
            MenuButtonAction::NewGame => {
                new_game.0 = true;
                next_state.set(AppState::Playing);
            }
            MenuButtonAction::LoadGame => {
                *save_load_mode = SaveLoadMode::Load;
                save_load_return.0 = AppState::MainMenu;
                next_state.set(AppState::SaveLoadMenu);
            }
            MenuButtonAction::Settings => {
                let current = current_state.get();
                settings_return.0 = if *current == AppState::Paused {
                    AppState::Paused
                } else {
                    AppState::MainMenu
                };
                next_state.set(AppState::Settings);
            }
            MenuButtonAction::Quit => {
                quit_requested.0 = true;
                exit.write(AppExit::Success);
            }
            MenuButtonAction::Resume => {
                next_state.set(AppState::Playing);
            }
            MenuButtonAction::SaveGame => {
                *save_load_mode = SaveLoadMode::Save;
                save_load_return.0 = AppState::Paused;
                next_state.set(AppState::SaveLoadMenu);
            }
            MenuButtonAction::MainMenu => {
                next_state.set(AppState::MainMenu);
            }
            MenuButtonAction::Back => match current_state.get() {
                AppState::Settings => {
                    next_state.set(settings_return.0);
                }
                AppState::SaveLoadMenu => {
                    next_state.set(save_load_return.0);
                }
                _ => {
                    next_state.set(AppState::MainMenu);
                }
            },
            MenuButtonAction::TabGraphics => {
                next_tab.set(SettingsTab::Graphics);
            }
            MenuButtonAction::TabAudio => {
                next_tab.set(SettingsTab::Audio);
            }
            MenuButtonAction::TabControls => {
                next_tab.set(SettingsTab::Controls);
            }
            MenuButtonAction::SaveSlot(slot) => {
                if *save_load_mode == SaveLoadMode::Save {
                    pending_save.0 = Some(slot);
                    next_state.set(save_load_return.0);
                } else if save_slots.slots[slot].is_some() {
                    pending_load.0 = Some(slot);
                    next_state.set(AppState::Playing);
                }
            }
            _ => {}
        }
    }
}

// ── Settings Adjustment ───────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn handle_settings_adjustment(
    query: Query<(&Interaction, &MenuButtonAction), (Changed<Interaction>, With<Button>)>,
    mut graphics: ResMut<GraphicsSettings>,
    mut audio: ResMut<AudioSettings>,
    mut rebinding: ResMut<RebindingState>,
    mut resolution_display: Query<
        &mut Text,
        (
            With<ResolutionDisplay>,
            Without<FullscreenDisplay>,
            Without<VsyncDisplay>,
            Without<VolumeDisplay>,
            Without<KeyBindingDisplay>,
        ),
    >,
    mut fullscreen_display: Query<
        &mut Text,
        (
            With<FullscreenDisplay>,
            Without<ResolutionDisplay>,
            Without<VsyncDisplay>,
            Without<VolumeDisplay>,
            Without<KeyBindingDisplay>,
        ),
    >,
    mut vsync_display: Query<
        &mut Text,
        (
            With<VsyncDisplay>,
            Without<ResolutionDisplay>,
            Without<FullscreenDisplay>,
            Without<VolumeDisplay>,
            Without<KeyBindingDisplay>,
        ),
    >,
    mut volume_display: Query<
        (&VolumeDisplay, &mut Text),
        (
            Without<ResolutionDisplay>,
            Without<FullscreenDisplay>,
            Without<VsyncDisplay>,
            Without<KeyBindingDisplay>,
        ),
    >,
    mut key_display: Query<
        (&KeyBindingDisplay, &mut Text),
        (
            Without<ResolutionDisplay>,
            Without<FullscreenDisplay>,
            Without<VsyncDisplay>,
            Without<VolumeDisplay>,
        ),
    >,
) {
    for (interaction, action) in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match *action {
            MenuButtonAction::ResolutionNext => {
                graphics.cycle_resolution_forward();
                let res = RESOLUTIONS[graphics.resolution_index];
                for mut text in &mut resolution_display {
                    **text = format!("{}x{}", res.0 as i32, res.1 as i32);
                }
            }
            MenuButtonAction::ResolutionPrev => {
                graphics.cycle_resolution_backward();
                let res = RESOLUTIONS[graphics.resolution_index];
                for mut text in &mut resolution_display {
                    **text = format!("{}x{}", res.0 as i32, res.1 as i32);
                }
            }
            MenuButtonAction::ToggleFullscreen => {
                graphics.fullscreen = !graphics.fullscreen;
                for mut text in &mut fullscreen_display {
                    **text = if graphics.fullscreen { "ON" } else { "OFF" }.to_string();
                }
            }
            MenuButtonAction::ToggleVsync => {
                graphics.vsync = !graphics.vsync;
                for mut text in &mut vsync_display {
                    **text = if graphics.vsync { "ON" } else { "OFF" }.to_string();
                }
            }
            MenuButtonAction::VolumeUp(channel) => {
                let vol = match channel {
                    VolumeChannel::Master => &mut audio.master_volume,
                    VolumeChannel::Music => &mut audio.music_volume,
                    VolumeChannel::Sfx => &mut audio.sfx_volume,
                };
                AudioSettings::adjust_volume(vol, 0.05);
                let new_val = *vol;
                for (vd, mut text) in &mut volume_display {
                    if vd.0 == channel {
                        **text = format!("{}%", (new_val * 100.0) as i32);
                    }
                }
            }
            MenuButtonAction::VolumeDown(channel) => {
                let vol = match channel {
                    VolumeChannel::Master => &mut audio.master_volume,
                    VolumeChannel::Music => &mut audio.music_volume,
                    VolumeChannel::Sfx => &mut audio.sfx_volume,
                };
                AudioSettings::adjust_volume(vol, -0.05);
                let new_val = *vol;
                for (vd, mut text) in &mut volume_display {
                    if vd.0 == channel {
                        **text = format!("{}%", (new_val * 100.0) as i32);
                    }
                }
            }
            MenuButtonAction::RebindAction(action) => {
                rebinding.awaiting = Some(action);
                for (kbd, mut text) in &mut key_display {
                    if kbd.0 == action {
                        **text = "Press a key...".to_string();
                    }
                }
            }
            _ => {}
        }
    }
}

// ── Tab Visibility ────────────────────────────────────────────────────────────

#[allow(clippy::type_complexity)]
pub fn update_tab_visibility(
    tab: Res<State<SettingsTab>>,
    mut graphics_q: Query<
        &mut Node,
        (
            With<GraphicsTabContent>,
            Without<AudioTabContent>,
            Without<ControlsTabContent>,
        ),
    >,
    mut audio_q: Query<
        &mut Node,
        (
            With<AudioTabContent>,
            Without<GraphicsTabContent>,
            Without<ControlsTabContent>,
        ),
    >,
    mut controls_q: Query<
        &mut Node,
        (
            With<ControlsTabContent>,
            Without<GraphicsTabContent>,
            Without<AudioTabContent>,
        ),
    >,
) {
    let current = tab.get();
    for mut node in &mut graphics_q {
        node.display = if *current == SettingsTab::Graphics {
            Display::Flex
        } else {
            Display::None
        };
    }
    for mut node in &mut audio_q {
        node.display = if *current == SettingsTab::Audio {
            Display::Flex
        } else {
            Display::None
        };
    }
    for mut node in &mut controls_q {
        node.display = if *current == SettingsTab::Controls {
            Display::Flex
        } else {
            Display::None
        };
    }
}

// ── Key Rebinding ─────────────────────────────────────────────────────────────

pub fn capture_rebind_key(
    keys: Res<ButtonInput<KeyCode>>,
    mut rebinding: ResMut<RebindingState>,
    mut bindings: ResMut<ControlBindings>,
    mut display_q: Query<(&KeyBindingDisplay, &mut Text)>,
) {
    let Some(action) = rebinding.awaiting else {
        return;
    };

    let Some(&key) = keys.get_just_pressed().next() else {
        return;
    };

    if key == KeyCode::Escape {
        rebinding.awaiting = None;
        for (kbd, mut text) in &mut display_q {
            if kbd.0 == action {
                **text = keycode_display_name(bindings.key_for(action)).to_string();
            }
        }
        return;
    }

    bindings.rebind(action, key);
    rebinding.awaiting = None;

    // Update all displays since a swap may have occurred.
    for (kbd, mut text) in &mut display_q {
        **text = keycode_display_name(bindings.key_for(kbd.0)).to_string();
    }
}

// ── Graphics Settings Application ────────────────────────────────────────────

pub fn apply_graphics_settings(graphics: Res<GraphicsSettings>, mut windows: Query<&mut Window>) {
    if !graphics.is_changed() {
        return;
    }
    let Ok(mut window) = windows.single_mut() else {
        return;
    };

    let (w, h) = graphics.resolution();
    window.resolution.set(w, h);
    window.mode = if graphics.fullscreen {
        WindowMode::BorderlessFullscreen(MonitorSelection::Current)
    } else {
        WindowMode::Windowed
    };
    window.present_mode = if graphics.vsync {
        PresentMode::AutoVsync
    } else {
        PresentMode::AutoNoVsync
    };
}

// ── Audio Settings Application ────────────────────────────────────────────────

pub fn apply_audio_settings(audio: Res<AudioSettings>, mut global_volume: ResMut<GlobalVolume>) {
    if !audio.is_changed() {
        return;
    }
    global_volume.volume = bevy::audio::Volume::Linear(audio.master_volume);
}

// ── Settings Persistence ──────────────────────────────────────────────────────

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
struct SettingsFile {
    graphics: GraphicsSettings,
    audio: AudioSettings,
}

fn settings_path() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("jaspersworld_test2").join("settings.json"))
}

pub fn load_settings(mut graphics: ResMut<GraphicsSettings>, mut audio: ResMut<AudioSettings>) {
    let Some(path) = settings_path() else { return };
    let Ok(data) = fs::read_to_string(&path) else {
        return;
    };
    let Ok(settings) = serde_json::from_str::<SettingsFile>(&data) else {
        return;
    };
    *graphics = settings.graphics;
    *audio = settings.audio;
}

pub fn save_settings(graphics: Res<GraphicsSettings>, audio: Res<AudioSettings>) {
    let Some(path) = settings_path() else { return };
    let settings = SettingsFile {
        graphics: graphics.clone(),
        audio: audio.clone(),
    };
    let Ok(json) = serde_json::to_string_pretty(&settings) else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(&path, json);
}

// ── Pending Save/Load Handler ─────────────────────────────────────────────────

use crate::collectibles::components::CollectionProgress;
use crate::combat::components::Health;
use crate::level::level_data::CurrentLevel;
use crate::player::components::Player;
use crate::puzzle::components::GameProgress;

#[allow(clippy::too_many_arguments)]
pub fn handle_pending_save_load(
    mut pending_save: ResMut<PendingSaveSlot>,
    mut pending_load: ResMut<PendingLoadSlot>,
    mut save_slots: ResMut<SaveSlots>,
    mut player_query: Query<(&mut Transform, &mut Health), With<Player>>,
    current_level: Res<CurrentLevel>,
    mut progress_mut: ResMut<CollectionProgress>,
    game_progress: Res<GameProgress>,
) {
    // Block save/load while a transition is active — the world is in an
    // inconsistent state between despawn and spawn.
    if game_progress.transition_in_progress {
        return;
    }
    // Handle save
    if let Some(slot) = pending_save.0.take()
        && let Ok((player_tf, health)) = player_query.single()
    {
        let level_name = match current_level.level_id {
            Some(crate::level::level_data::LevelId::Forest) => "Forest",
            Some(crate::level::level_data::LevelId::Subdivision) => "Subdivision",
            Some(crate::level::level_data::LevelId::City) => "City",
            Some(crate::level::level_data::LevelId::Sanctuary) => "Sanctuary",
            None => "Forest",
        };

        let mut data = GameSaveData::new_game(slot);
        data.player_position = (player_tf.translation.x, player_tf.translation.y);
        data.health = health.current;
        data.current_level = level_name.to_string();
        data.metadata.chapter = format!("{level_name} L{}", current_level.layer_index + 1);

        write_menu_save(slot, &data);
        save_slots.slots[slot] = Some(data.metadata);
    }

    // Handle load
    if let Some(slot) = pending_load.0.take()
        && let Some(data) = read_menu_save(slot)
    {
        if let Ok((mut tf, mut health)) = player_query.single_mut() {
            tf.translation.x = data.player_position.0;
            tf.translation.y = data.player_position.1;
            health.current = data.health;
        }
        progress_mut.stars_collected = 0; // reset progress on load
    }
}
