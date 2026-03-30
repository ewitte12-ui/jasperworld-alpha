use bevy::prelude::*;

use crate::resources::GameAction;

/// Marker for the root HUD entity.
#[derive(Component)]
pub struct HudRoot;

/// Marker for the health display text node.
#[derive(Component)]
pub struct HealthDisplay;

/// Marker for the star counter text node.
#[derive(Component)]
pub struct StarCounter;

/// Marker for the level name display text node.
#[derive(Component)]
pub struct LevelNameDisplay;

// ── Menu button actions ──────────────────────────────────────────────────────

#[derive(Component, Clone, Copy, Debug)]
pub enum MenuButtonAction {
    // Main menu
    NewGame,
    LoadGame,
    Settings,
    Quit,
    // Pause menu
    Resume,
    SaveGame,
    MainMenu,
    // Navigation
    Back,
    // Settings tabs
    TabGraphics,
    TabAudio,
    TabControls,
    // Graphics settings
    ToggleFullscreen,
    ToggleVsync,
    ResolutionNext,
    ResolutionPrev,
    // Audio settings
    VolumeUp(VolumeChannel),
    VolumeDown(VolumeChannel),
    // Control rebinding
    RebindAction(GameAction),
    // Save slots
    SaveSlot(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VolumeChannel {
    Master,
    Music,
    Sfx,
}

// ── Settings tab content markers ─────────────────────────────────────────────

#[derive(Component)]
pub struct GraphicsTabContent;

#[derive(Component)]
pub struct AudioTabContent;

#[derive(Component)]
pub struct ControlsTabContent;

// ── Display markers for settings values ──────────────────────────────────────

#[derive(Component)]
pub struct VolumeDisplay(pub VolumeChannel);

#[derive(Component)]
pub struct ResolutionDisplay;

#[derive(Component)]
pub struct FullscreenDisplay;

#[derive(Component)]
pub struct VsyncDisplay;

#[derive(Component)]
pub struct KeyBindingDisplay(pub GameAction);

#[derive(Component)]
pub struct SaveSlotDisplay(pub usize);
