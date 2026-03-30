use bevy::prelude::*;

/// Top-level application state.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    TitleScreen,
    MainMenu,
    Settings,
    SaveLoadMenu,
    Playing,
    Paused,
}

/// Sub-state for which settings tab is active (only valid in Settings state).
#[derive(SubStates, Clone, PartialEq, Eq, Hash, Debug, Default)]
#[source(AppState = AppState::Settings)]
pub enum SettingsTab {
    #[default]
    Graphics,
    Audio,
    Controls,
}

/// Remembers which state to return to after closing Settings.
#[derive(Resource, Clone)]
pub struct SettingsReturnState(pub AppState);

impl Default for SettingsReturnState {
    fn default() -> Self {
        Self(AppState::MainMenu)
    }
}

/// Whether the save/load menu is in save or load mode.
#[derive(Resource, Default, Clone, PartialEq, Eq)]
pub enum SaveLoadMode {
    #[default]
    Load,
    Save,
}

/// Remembers which state to return to after closing the save/load menu.
#[derive(Resource, Clone)]
pub struct SaveLoadReturnState(pub AppState);

impl Default for SaveLoadReturnState {
    fn default() -> Self {
        Self(AppState::MainMenu)
    }
}

/// Flag resource: when true, entering Playing state triggers a full game reset.
#[derive(Resource, Default)]
pub struct NewGameRequested(pub bool);

/// One-shot quit guard.  Set to `true` the moment an exit is requested.
/// Once set, all subsequent exit inputs are ignored (idempotent) and no
/// system may initiate new work.  The `AppExit` event is written exactly
/// once, on the frame this flag transitions from `false` to `true`.
#[derive(Resource, Default)]
pub struct QuitRequested(pub bool);
