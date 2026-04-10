pub mod cutscene;

use bevy::prelude::*;

use crate::rendering::camera::CameraPipeline;
use crate::states::AppState;
use cutscene::{
    advance_cutscene, check_sanctuary_trigger, cleanup_cutscene_on_exit,
    cutscene_camera_follow, cutscene_camera_zoom, load_cutscene_config, render_cutscene_box,
    reset_cutscene_state, CutsceneCameraOverride, SanctuaryCutsceneState,
};

pub struct SanctuaryPlugin;

impl Plugin for SanctuaryPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resource holds config + runtime state; init_resource uses Default (config = None).
            .init_resource::<SanctuaryCutsceneState>()
            // Load dialog config once at startup.
            .add_systems(Startup, load_cutscene_config)
            // Reset runtime state each time gameplay begins (new game / restart).
            .add_systems(OnEnter(AppState::Playing), reset_cutscene_state)
            // Ensure override + zoom are cleaned up if the player exits mid-cutscene.
            .add_systems(OnExit(AppState::Playing), cleanup_cutscene_on_exit)
            // Cutscene dialog systems: chained so trigger → advance → render in order.
            .add_systems(
                Update,
                (check_sanctuary_trigger, advance_cutscene, render_cutscene_box)
                    .chain()
                    .run_if(in_state(AppState::Playing)),
            )
            // Camera override systems run in CameraPipeline::Follow, only while
            // CutsceneCameraOverride exists.  They feed into the existing
            // Clamp → Snap → Parallax pipeline so pixel-snapping is preserved.
            .add_systems(
                Update,
                cutscene_camera_follow
                    .in_set(CameraPipeline::Follow)
                    .run_if(in_state(AppState::Playing))
                    .run_if(resource_exists::<CutsceneCameraOverride>),
            )
            .add_systems(
                Update,
                cutscene_camera_zoom
                    .in_set(CameraPipeline::Follow)
                    .run_if(in_state(AppState::Playing))
                    .run_if(resource_exists::<CutsceneCameraOverride>),
            );
    }
}
