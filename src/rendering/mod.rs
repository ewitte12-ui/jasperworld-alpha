pub mod atlas;
pub mod camera;
pub mod parallax;
pub mod quad;
use bevy::prelude::*;

use camera::{CameraPipeline, CameraPlugin, camera_snap};
use parallax::{apply_scene_tints, update_parallax};
pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CameraPlugin)
            // Declare total order: Follow → Clamp → Snap → Parallax.
            // All four stages run every Update frame; .chain() enforces strict ordering.
            .configure_sets(
                Update,
                (
                    CameraPipeline::Follow,
                    CameraPipeline::Clamp,
                    CameraPipeline::Snap,
                    CameraPipeline::Parallax,
                )
                    .chain(),
            )
            .add_systems(Startup, load_atlas_resources)
            .add_systems(
                Update,
                (
                    camera_snap.in_set(CameraPipeline::Snap),
                    update_parallax.in_set(CameraPipeline::Parallax),
                    apply_scene_tints,
                ),
            );
    }
}

fn load_atlas_resources(mut commands: Commands, asset_server: Res<AssetServer>) {
    let tile_atlas = atlas::TileAtlas::new(
        asset_server.load("tiles/tilemap_packed.png"),
        atlas::AtlasConfig::TILE_TILEMAP,
    );
    commands.insert_resource(tile_atlas);

    let char_atlas = atlas::CharAtlas::new(
        asset_server.load("tiles/tilemap-characters_packed.png"),
        atlas::AtlasConfig::CHAR_TILEMAP,
    );
    commands.insert_resource(char_atlas);
}
