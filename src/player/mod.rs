pub mod components;
pub mod controller;
pub mod input;
pub mod systems;

use avian2d::prelude::PhysicsSchedule;
use bevy::prelude::*;
use bevy_tnua::{TnuaControllerPlugin, TnuaUserControlsSystems};

use crate::rendering::camera::CameraPipeline;
use components::PlayerControlScheme;
use controller::{neutralize_player_material, setup_player_physics};
use input::player_input;
use systems::{camera_follow, player_clamp};

use crate::states::AppState;

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
                player_input.in_set(TnuaUserControlsSystems),
                player_clamp.after(player_input),
                camera_follow
                    .after(player_clamp)
                    .in_set(CameraPipeline::Follow),
            )
                .run_if(in_state(AppState::Playing)),
        );

        // Runs unconditionally (not gated by AppState) because GLB mesh entities
        // spawn asynchronously and may resolve during any state. The
        // `PlayerMaterialNeutralized` guard ensures each mesh is processed once.
        app.add_systems(Update, neutralize_player_material);
    }
}
