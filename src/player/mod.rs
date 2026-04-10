pub mod components;
pub mod controller;
pub mod input;
pub mod systems;

use avian2d::prelude::PhysicsSchedule;
use bevy::prelude::*;
use bevy_tnua::{TnuaControllerPlugin, TnuaUserControlsSystems};

use crate::rendering::camera::CameraPipeline;
use crate::sanctuary::cutscene::CutsceneCameraOverride;
use crate::states::AppState;
use components::PlayerControlScheme;
use controller::setup_player_physics;
use input::player_input;
use systems::{camera_follow, player_clamp};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        // TnuaControllerPlugin is generic over the control scheme and must share the physics
        // schedule with TnuaAvian2dPlugin (both registered here with PhysicsSchedule).
        app.add_plugins(TnuaControllerPlugin::<PlayerControlScheme>::new(
            PhysicsSchedule,
        ));

        app.add_systems(Startup, setup_player_physics).add_systems(
            Update,
            (
                // player_input feeds Tnua every frame; must be in TnuaUserControlsSystems.
                // Suppressed during cutscene so Jasper can't move while dialog is active.
                player_input
                    .in_set(TnuaUserControlsSystems)
                    .run_if(not(resource_exists::<CutsceneCameraOverride>)),
                player_clamp.after(player_input),
                camera_follow
                    .after(player_clamp)
                    .in_set(CameraPipeline::Follow)
                    // Suppressed when the sanctuary cutscene is active so the
                    // cutscene camera systems take over without fighting for control.
                    .run_if(not(resource_exists::<CutsceneCameraOverride>)),
            )
                .run_if(in_state(AppState::Playing)),
        );
    }
}
