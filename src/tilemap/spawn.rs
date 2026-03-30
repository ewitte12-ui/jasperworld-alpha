use avian2d::prelude::*;
use bevy::prelude::*;

use crate::level::components::{OneWayPlatform, TileEntity};
use crate::physics::config::GameLayer;
use crate::rendering::parallax::SceneTint;
use crate::tilemap::tilemap::{TILE_SIZE, TileType};

// Kenney Platformer Kit native model bounds (from GLB GLTF accessor min/max).
// All models are bottom-anchored (Y: 0 → model_height).
// scale = TILE_SIZE / native_dimension ensures each tile fills exactly TILE_SIZE × TILE_SIZE.
const BLOCK_LARGE_W: f32 = 2.082; // block-grass-large native X width
const BLOCK_LARGE_H: f32 = 1.000; // block-grass-large native Y height
const BLOCK_LOW_W: f32 = 1.082; // block-grass-low native X width
const BLOCK_LOW_H: f32 = 0.500; // block-grass-low native Y height
const BRICK_W: f32 = 0.500; // brick native X width
const BRICK_H: f32 = 0.500; // brick native Y height
const PLATFORM_W: f32 = 1.082; // platform native X width (same as block-grass-low)
const PLATFORM_H: f32 = 0.500; // platform native Y height

/// Spawn all tiles for a 2D grid using 3D GLB models.
///
/// Two-pass approach to eliminate tile-seam ghost collisions:
///
///   Pass 1 — one **visual** entity per tile (SceneRoot child, no physics).
///             Preserves per-tile GLB model rendering unchanged.
///
///   Pass 2 — one **physics** entity per contiguous horizontal run of same-type
///             tiles (one wide `Collider::rectangle` spanning the entire run).
///             This removes all interior vertical tile edges; the physics solver
///             only ever sees the flat top surface of each run.  No seam contacts,
///             no phantom horizontal stopping impulses.
pub fn spawn_tilemap(
    commands: &mut Commands,
    asset_server: &AssetServer,
    solid_model: &str,
    platform_model: &str,
    grid: &[Vec<TileType>],
    origin: Vec2,
    z: f32,
) {
    spawn_tilemap_inner(commands, asset_server, solid_model, platform_model, grid, origin, z, None);
}

/// Same as `spawn_tilemap` but applies a color tint to platform tiles.
#[allow(clippy::too_many_arguments)]
pub fn spawn_tilemap_tinted(
    commands: &mut Commands,
    asset_server: &AssetServer,
    solid_model: &str,
    platform_model: &str,
    grid: &[Vec<TileType>],
    origin: Vec2,
    z: f32,
    platform_tint: Color,
) {
    spawn_tilemap_inner(commands, asset_server, solid_model, platform_model, grid, origin, z, Some(platform_tint));
}

#[allow(clippy::too_many_arguments)]
fn spawn_tilemap_inner(
    commands: &mut Commands,
    asset_server: &AssetServer,
    solid_model: &str,
    platform_model: &str,
    grid: &[Vec<TileType>],
    origin: Vec2,
    z: f32,
    platform_tint: Option<Color>,
) {
    // ── Pass 1: visuals (one GLB model per tile, no collider) ────────────────
    for (row_idx, row) in grid.iter().enumerate() {
        for (col_idx, &tile_type) in row.iter().enumerate() {
            if tile_type == TileType::Empty {
                continue;
            }

            let wx = origin.x + col_idx as f32 * TILE_SIZE;
            let wy = origin.y + row_idx as f32 * TILE_SIZE;

            let model = if tile_type == TileType::Platform {
                platform_model
            } else {
                solid_model
            };

            let vis_scale = if tile_type == TileType::Platform {
                model_scale_platform(model)
            } else {
                model_scale(model)
            };
            let scene_handle = asset_server.load(format!("{}#Scene0", model));

            // Visual entity: same world position as the original physics entity
            // (wx, wy+2.0) with the scene child offset down by TILE_SIZE/2 so the
            // model top aligns with the visible ground surface.
            // SceneTint must be on the SceneRoot entity (not the parent) because
            // apply_scene_tints walks Children of the tinted entity to find
            // MeshMaterial3d handles — the mesh entities are children of the
            // SceneRoot entity, not children of the TileEntity parent.
            let tint_platform = tile_type == TileType::Platform && platform_tint.is_some();
            commands
                .spawn((
                    TileEntity,
                    Transform::from_xyz(wx, wy + 2.0, z),
                    Visibility::default(),
                ))
                .with_children(|parent| {
                    let mut child = parent.spawn((
                        SceneRoot(scene_handle),
                        Transform::from_xyz(0.0, -TILE_SIZE * 0.5, 0.0).with_scale(vis_scale),
                    ));
                    if tint_platform {
                        child.insert(SceneTint::Replace(platform_tint.unwrap()));
                    }
                });
        }
    }

    // ── Pass 2: merged colliders (one per contiguous horizontal run) ─────────
    for (row_idx, row) in grid.iter().enumerate() {
        let wy = origin.y + row_idx as f32 * TILE_SIZE;
        spawn_merged_run_colliders(commands, row, origin.x, wy, z, TileType::Solid);
        spawn_merged_run_colliders(commands, row, origin.x, wy, z, TileType::Platform);
    }
}

/// Scans `row` for contiguous runs of `target` tile type and spawns one merged
/// rectangle collider per run.  Interior tile edges within a run are eliminated;
/// only the flat top surface is visible to the physics solver.
fn spawn_merged_run_colliders(
    commands: &mut Commands,
    row: &[TileType],
    origin_x: f32,
    wy: f32,
    z: f32,
    target: TileType,
) {
    let is_platform = target == TileType::Platform;
    // Collider 4 units shorter than visual tile, shifted up 2 units so the top
    // surface stays flush with the visual ground surface (same as before).
    let collider_h = TILE_SIZE - 4.0;
    let col_count = row.len();

    let mut run_start: Option<usize> = None;

    // Iterate one past the end so any open run at the right edge gets flushed.
    for col_idx in 0..=col_count {
        let continues_run = col_idx < col_count && row[col_idx] == target;

        match (run_start, continues_run) {
            (None, true) => {
                // Start of a new run.
                run_start = Some(col_idx);
            }
            (Some(start), false) => {
                // End of a run — spawn one merged collider for [start, col_idx-1].
                let end = col_idx - 1;
                let num_tiles = (end - start + 1) as f32;
                let run_w = num_tiles * TILE_SIZE;
                // Centre = average of first-tile centre and last-tile centre.
                let cx = origin_x + (start as f32 + end as f32) * 0.5 * TILE_SIZE;

                let layers = if is_platform {
                    CollisionLayers::new(
                        GameLayer::Platform,
                        [GameLayer::Player, GameLayer::Enemy, GameLayer::Default],
                    )
                } else {
                    CollisionLayers::new(
                        GameLayer::Ground,
                        [GameLayer::Player, GameLayer::Enemy, GameLayer::Default],
                    )
                };

                let mut e = commands.spawn((
                    TileEntity,
                    Transform::from_xyz(cx, wy + 2.0, z),
                    Visibility::default(),
                    RigidBody::Static,
                    Collider::rectangle(run_w, collider_h),
                    Friction::ZERO,
                    layers,
                ));

                if is_platform {
                    e.insert((OneWayPlatform, ActiveCollisionHooks::MODIFY_CONTACTS));
                }

                run_start = None;
            }
            // (None, false): no run in progress, tile doesn't match — skip.
            // (Some(_), true): run in progress, tile matches — extend implicitly.
            _ => {}
        }
    }
}

/// Returns the non-uniform scale that makes a Kenney Platformer Kit model fill
/// exactly `TILE_SIZE × TILE_SIZE` world units.
fn model_scale(model_path: &str) -> Vec3 {
    let (w, h) = if model_path.contains("block-grass-low") {
        (BLOCK_LOW_W, BLOCK_LOW_H)
    } else if model_path.contains("block-grass") {
        (BLOCK_LARGE_W, BLOCK_LARGE_H)
    } else if model_path.contains("platform") {
        (PLATFORM_W, PLATFORM_H)
    } else if model_path.contains("brick") {
        (BRICK_W, BRICK_H)
    } else {
        // Unknown model — use a reasonable uniform scale.
        return Vec3::splat(TILE_SIZE);
    };
    // Z scale gives the blocks visible depth from the -28° camera tilt,
    // making the green grass top face clearly visible.
    // Ground/brick tiles get thick Z so the grass/brick top face is clearly
    // visible from the -28° camera tilt. Platforms use a thinner Z so they
    // don't look like chunky slabs.
    let z_scale = if model_path.contains("block-grass-low") { 3.0 } else { 6.0 };
    Vec3::new(TILE_SIZE / w, TILE_SIZE / h, z_scale)
}

/// Same as `model_scale` but with extra Z depth so the top face is more
/// visible from the -28° camera tilt. Used for brick platform tiles.
fn model_scale_platform(model_path: &str) -> Vec3 {
    let mut s = model_scale(model_path);
    s.z = 10.0;
    s
}

/// Returns true if the grid cell at (col, row) is a solid or platform tile.
/// Out-of-bounds cells are treated as empty (not solid).
pub fn neighbor_solid(
    grid: &[Vec<TileType>],
    num_rows: usize,
    num_cols: usize,
    col: i32,
    row: i32,
) -> bool {
    if col < 0 || row < 0 {
        return false;
    }
    let col = col as usize;
    let row = row as usize;
    if row >= num_rows || col >= num_cols {
        return false;
    }
    grid[row][col] != TileType::Empty
}
