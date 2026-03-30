use avian2d::prelude::{Collider, RigidBody};
use bevy::prelude::*;

use crate::level::components::TileEntity;
use crate::rendering::atlas::{AtlasConfig, TileAtlas, uv_rect};
use crate::rendering::quad::spawn_textured_quad;

const TILE_SIZE: f32 = 18.0;
const GRASS_TOP_INDEX: usize = 4;
const DIRT_FILL_INDEX: usize = 24;

/// Spawns the Phase-2 test level:
/// - Ground floor: 30 tiles wide × 2 tiles deep
/// - Platform A: cols 3–5, y = 54
/// - Platform B: cols 10–12, y = 90
/// - Platform C: cols 18–20, y = 54
pub fn spawn_test_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    tile_atlas: Res<TileAtlas>,
) {
    let config = &tile_atlas.config;
    let image = tile_atlas.image.clone();

    // x origin: left edge of tile 0 is at -15*18 = -270.
    // Tile centres are at start_x + col * TILE_SIZE + TILE_SIZE * 0.5.
    let start_x = -15.0 * TILE_SIZE; // = -270.0

    // ── Ground floor ────────────────────────────────────────────────────────
    for col in 0..30usize {
        let cx = start_x + col as f32 * TILE_SIZE + TILE_SIZE * 0.5;

        // Top row: grass surface (y = 0)
        spawn_ground_tile(
            &mut commands,
            &mut meshes,
            &mut materials,
            image.clone(),
            uv_rect(config, GRASS_TOP_INDEX),
            Vec3::new(cx, 0.0, 0.0),
        );

        // Fill row: dirt (y = -18)
        spawn_ground_tile(
            &mut commands,
            &mut meshes,
            &mut materials,
            image.clone(),
            uv_rect(config, DIRT_FILL_INDEX),
            Vec3::new(cx, -TILE_SIZE, 0.0),
        );
    }

    // ── Platforms ───────────────────────────────────────────────────────────
    // Platform A: cols 3–5, y = 3*18 = 54
    spawn_platform(
        &mut commands,
        &mut meshes,
        &mut materials,
        image.clone(),
        config,
        start_x,
        3..=5,
        3.0 * TILE_SIZE,
    );

    // Platform B: cols 10–12, y = 5*18 = 90
    spawn_platform(
        &mut commands,
        &mut meshes,
        &mut materials,
        image.clone(),
        config,
        start_x,
        10..=12,
        5.0 * TILE_SIZE,
    );

    // Platform C: cols 18–20, y = 3*18 = 54
    spawn_platform(
        &mut commands,
        &mut meshes,
        &mut materials,
        image.clone(),
        config,
        start_x,
        18..=20,
        3.0 * TILE_SIZE,
    );
}

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Spawns a single static ground tile with a physics collider.
fn spawn_ground_tile(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    image: Handle<Image>,
    uv: [f32; 4],
    position: Vec3,
) {
    let entity = spawn_textured_quad(
        commands,
        meshes,
        materials,
        image,
        uv,
        position,
        Vec2::splat(TILE_SIZE),
    );
    commands.entity(entity).insert((
        TileEntity,
        RigidBody::Static,
        Collider::rectangle(TILE_SIZE, TILE_SIZE),
    ));
}

/// Spawns a row of platform tiles spanning `col_range` at the given `y`.
#[allow(clippy::too_many_arguments)]
fn spawn_platform(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    image: Handle<Image>,
    config: &AtlasConfig,
    start_x: f32,
    col_range: std::ops::RangeInclusive<usize>,
    y: f32,
) {
    let uv = uv_rect(config, GRASS_TOP_INDEX);
    for col in col_range {
        let cx = start_x + col as f32 * TILE_SIZE + TILE_SIZE * 0.5;
        let entity = spawn_textured_quad(
            commands,
            meshes,
            materials,
            image.clone(),
            uv,
            Vec3::new(cx, y, 0.0),
            Vec2::splat(TILE_SIZE),
        );
        commands.entity(entity).insert((
            TileEntity,
            RigidBody::Static,
            Collider::rectangle(TILE_SIZE, TILE_SIZE),
        ));
    }
}
