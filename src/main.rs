use bevy::prelude::*;
use bevy::window::WindowPosition;
use jaspersworld::animation::AnimationPlugin;
use jaspersworld::audio::AudioPlugin;
use jaspersworld::collectibles::CollectiblesPlugin;
use jaspersworld::combat::CombatPlugin;
use jaspersworld::dialogue::DialoguePlugin;
use jaspersworld::enemies::EnemiesPlugin;
use jaspersworld::level::LevelPlugin;
use jaspersworld::lighting::LightingPlugin;
use jaspersworld::menu::MenuPlugin;
use jaspersworld::menu::{apply_audio_settings, apply_graphics_settings, load_settings};
use jaspersworld::particles::ParticlesPlugin;
use jaspersworld::physics::PhysicsPlugin;
use jaspersworld::player::PlayerPlugin;
use jaspersworld::puzzle::PuzzlePlugin;
use jaspersworld::rendering::RenderingPlugin;
use jaspersworld::resources::{
    AudioSettings, ControlBindings, GraphicsSettings, PendingLoadSlot, PendingSaveSlot,
    RebindingState, SaveSlots,
};
use jaspersworld::save_load::SaveLoadPlugin;
use jaspersworld::states::{AppState, NewGameRequested, QuitRequested, SaveLoadMode, SaveLoadReturnState, SettingsReturnState, SettingsTab};
use jaspersworld::tilemap::TilemapPlugin;
use jaspersworld::title::TitleBackgroundPlugin;
use jaspersworld::ui::UiPlugin;
use jaspersworld::vfx::VfxPlugin;
use jaspersworld::window_geometry::{load_window_geometry, persist_window_geometry};

fn main() {
    // Read saved window geometry before App::new() so the OS window is
    // created at the correct position and size on the first frame.
    // This must happen here — by the time any Startup system runs,
    // the window already exists and has been shown to the user.
    let saved_geometry = load_window_geometry();

    let initial_window = {
        let mut w = Window {
            title: "Jasper's World".to_string(),
            // Fallback size: overridden by apply_graphics_settings on frame 1
            // if a settings file exists. Chosen to match the smallest resolution
            // preset so the flash (if any) is visually minimal.
            resolution: (1280u32, 720u32).into(),
            ..default()
        };
        if let Some(geom) = saved_geometry {
            w.position = WindowPosition::At(IVec2::new(geom.x, geom.y));
            w.resolution = (geom.width, geom.height).into(); // u32, u32
        }
        w
    };

    let mut app = App::new();

    app.add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(initial_window),
                    ..default()
                }),
        )
        // States
        .init_state::<AppState>()
        .add_sub_state::<SettingsTab>()
        // Menu/settings resources
        .init_resource::<GraphicsSettings>()
        .init_resource::<AudioSettings>()
        .init_resource::<ControlBindings>()
        .init_resource::<RebindingState>()
        .init_resource::<SaveSlots>()
        .init_resource::<SettingsReturnState>()
        .init_resource::<SaveLoadMode>()
        .init_resource::<SaveLoadReturnState>()
        .init_resource::<PendingSaveSlot>()
        .init_resource::<PendingLoadSlot>()
        .init_resource::<NewGameRequested>()
        .init_resource::<QuitRequested>()
        // Gameplay plugins
        .add_plugins(PhysicsPlugin)
        .add_plugins(RenderingPlugin)
        .add_plugins(TilemapPlugin)
        .add_plugins(LevelPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(EnemiesPlugin)
        .add_plugins(CombatPlugin)
        .add_plugins(CollectiblesPlugin)
        .add_plugins(PuzzlePlugin)
        .add_plugins(UiPlugin)
        .add_plugins(DialoguePlugin)
        .add_plugins(SaveLoadPlugin)
        .add_plugins(AudioPlugin)
        .add_plugins(AnimationPlugin)
        .add_plugins(ParticlesPlugin)
        .add_plugins(LightingPlugin)
        .add_plugins(VfxPlugin)
        // Menu plugin (title, main menu, pause, settings, save/load)
        .add_plugins(MenuPlugin)
        .add_plugins(TitleBackgroundPlugin)
        // Global systems
        .add_systems(Startup, load_settings)
        .add_systems(Update, (
            apply_audio_settings,
            apply_graphics_settings,
            persist_window_geometry,
        ));

    // DEBUG ONLY — compiled out in release builds (`cargo build --release`)
    #[cfg(debug_assertions)]
    app.add_plugins(jaspersworld::debug::DebugStartPlugin);

    app.run();
}
