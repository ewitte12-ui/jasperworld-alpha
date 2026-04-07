use avian2d::prelude::*;
// NOTE on Friction::ZERO: enemies must carry Friction::ZERO (CombineRule::Min) for the same
// reason the player does.  Ground tiles also have Friction::ZERO, but their combine rule is
// Min (priority 0).  avian2d resolves combined friction by the higher-priority rule: enemy
// default friction uses Average (priority 1), which wins over the tile's Min, yielding a
// non-zero coefficient (~0.15) that bleeds velocity.x on every physics substep.  Because
// enemy_ai runs in Update (after FixedPostUpdate physics), it cannot correct mid-substep
// velocity loss — enemies appear to barely move despite their speed being set correctly.
// Friction::ZERO on enemies forces Min on both bodies: min(0, 0) = 0, eliminating friction
// impulses entirely and preserving velocity.x through all physics substeps.
use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;

use crate::physics::config::GameLayer;

use crate::animation::components::{AtlasLayout, EnemyAnimState, SpriteAnimation};

use super::components::{ContactDamage, Enemy, EnemyAI, EnemyJump, EnemyType, PatrolOnly};

fn speed_for_type(enemy_type: EnemyType) -> f32 {
    match enemy_type {
        EnemyType::Dog => 120.0,
        EnemyType::Squirrel => 160.0,
        EnemyType::Snake => 55.0,
        EnemyType::Rat => 130.0,
        EnemyType::Possum => 45.0,
    }
}

fn texture_path(enemy_type: EnemyType) -> &'static str {
    match enemy_type {
        EnemyType::Dog => "enemies/dog.png",
        EnemyType::Squirrel => "enemies/squirrel.png",
        EnemyType::Snake => "enemies/snake.png",
        EnemyType::Rat => "enemies/rat.png",
        EnemyType::Possum => "enemies/possum.png",
    }
}

/// Atlas grid dimensions (cols, rows) per enemy type.
/// Types using a single-frame PNG return (1, 1) — full-texture UV.
fn atlas_grid(enemy_type: EnemyType) -> (f32, f32) {
    match enemy_type {
        EnemyType::Dog => (4.0, 2.0),      // 512×256, 4×2 grid
        EnemyType::Squirrel => (4.0, 2.0), // 512×256, 4×2 grid
        EnemyType::Snake => (4.0, 2.0),    // 512×256, 4×2 grid
        EnemyType::Possum => (4.0, 2.0),   // 512×256, 4×2 grid
        EnemyType::Rat => (4.0, 2.0),      // 512×256, 4×2 grid
    }
}

/// Collider dimensions for all enemies (NOT visual size — collider only).
const COLLIDER_W: f32 = 16.0;
const COLLIDER_H: f32 = 20.0;

/// Visual quad size per enemy type. Decoupled from collider so sprite scale
/// can match the player (28×32) without changing hitboxes or physics.
fn quad_size(enemy_type: EnemyType) -> (f32, f32) {
    match enemy_type {
        EnemyType::Dog => (28.0, 32.0),      // match player quad
        EnemyType::Squirrel => (28.0, 32.0), // match player quad
        EnemyType::Snake => (28.0, 32.0),    // match player quad
        EnemyType::Possum => (28.0, 32.0),   // match player quad
        EnemyType::Rat => (28.0, 32.0),      // match player quad
    }
}

/// Spawns an enemy.
///
/// **Contract:** `position.y` is the ground surface (top of the tile the enemy
/// stands on, i.e. `ground_top`). The spawner adds `COLLIDER_H / 2` so the
/// collider sits flush on the surface without penetrating it.
#[allow(clippy::too_many_arguments)]
pub fn spawn_enemy(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    asset_server: &AssetServer,
    enemy_type: EnemyType,
    position: Vec2,
    patrol_range: f32,
    health: f32,
    speed_override: Option<f32>,
) -> Entity {
    let speed = speed_override.unwrap_or_else(|| speed_for_type(enemy_type));
    let texture: Handle<Image> = asset_server.load(texture_path(enemy_type));

    // Visual quad size may differ from collider (e.g. Dog uses 28×32 to match player).
    // Collider is unchanged — visual size is presentation only.
    //
    // Pivot correction: shift mesh vertices up so the quad bottom aligns with the
    // collider bottom (ground surface). Without this, the 28×32 quad on a 16×20
    // collider extends 6 units below ground. This is the same technique the player
    // uses (y_offset=5.0 in controller.rs). The offset is baked into mesh vertices,
    // not the transform — it's shared by all enemies of this type, not per-entity.
    //
    // y_offset = sprite_h/2 - collider_h/2 = 16 - 10 = 6.0
    // With this offset, art baseline = 4px (identical to player).
    let (sprite_w, sprite_h) = quad_size(enemy_type);
    let hw = sprite_w / 2.0;
    let hh = sprite_h / 2.0;
    let y_offset = hh - COLLIDER_H / 2.0;
    let (grid_cols, grid_rows) = atlas_grid(enemy_type);
    let mut mesh_data = Mesh::from(Rectangle::new(sprite_w, sprite_h));

    // Shift vertex positions up by y_offset so pivot corresponds to feet baseline.
    // Bevy Rectangle vertex order: TR(0), TL(1), BL(2), BR(3)
    mesh_data.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float32x3(vec![
            [hw, hh + y_offset, 0.0],   // TR
            [-hw, hh + y_offset, 0.0],  // TL
            [-hw, -hh + y_offset, 0.0], // BL
            [hw, -hh + y_offset, 0.0],  // BR
        ]),
    );

    if grid_cols > 1.0 || grid_rows > 1.0 {
        // UV inset to prevent atlas bleeding at cell boundaries.
        let eps_u = 0.002;
        let eps_v = 0.002;
        let u_min = eps_u;
        let v_min = eps_v;
        let u_max = 1.0 / grid_cols - eps_u;
        let v_max = 1.0 / grid_rows - eps_v;
        let uvs = vec![
            [u_max, v_min], // TR
            [u_min, v_min], // TL
            [u_min, v_max], // BL
            [u_max, v_max], // BR
        ];
        mesh_data.insert_attribute(Mesh::ATTRIBUTE_UV_0, VertexAttributeValues::Float32x2(uvs));
    }
    let mesh = meshes.add(mesh_data);
    // double_sided + cull_mode None: sprite flips left/right by negating
    // transform.scale.x (see update_enemy_anim_state).  A negative scale
    // inverts winding; without double_sided the flipped sprite is invisible.
    let material = materials.add(StandardMaterial {
        base_color_texture: Some(texture),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        double_sided: true,
        cull_mode: None,
        ..default()
    });

    // position.y is the ground surface; offset up by half the collider height
    // so the collider bottom aligns with the surface.
    let center_y = position.y + COLLIDER_H * 0.5;

    let mut entity = commands.spawn((
        Enemy {
            enemy_type,
            health,
            speed,
            patrol_range,
            spawn_x: position.x,
        },
        EnemyAI::Patrol { direction: 1 },
        ContactDamage { amount: 25.0 },
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_xyz(position.x, center_y, 5.0).with_scale(Vec3::splat(1.0)),
        RigidBody::Dynamic,
        // Capsule instead of rectangle: rounded bottom/top slide over individual tile
        // seam edges without catching.  Rectangle colliders catch on the vertical face
        // of each adjacent tile's collider, producing a horizontal stopping impulse
        // (velocity.x → 0) mid-substep that enemy_ai (running after physics) cannot
        // correct until the next frame — enemies appear to barely move.
        // Dimensions match the original rectangle: radius = COLLIDER_W/2 = 8,
        // cylindrical section length = COLLIDER_H - 2*radius = 20 - 16 = 4.
        Collider::capsule(COLLIDER_W * 0.5, COLLIDER_H - COLLIDER_W),
        Friction::ZERO,
        LockedAxes::ROTATION_LOCKED,
        CollisionLayers::new(
            GameLayer::Enemy,
            [GameLayer::Default, GameLayer::Ground, GameLayer::Platform],
        ),
    ));

    entity.insert((
        SpriteAnimation {
            frames: vec![0],
            current_frame: 0,
            timer: Timer::from_seconds(0.15, TimerMode::Repeating),
            looping: true,
            last_written_frame: usize::MAX,
            atlas: AtlasLayout::ENEMY,
        },
        EnemyAnimState::Idle,
    ));

    if enemy_type == EnemyType::Dog {
        entity.insert(EnemyJump {
            // v = sqrt(2 × 980 × 45) ≈ 297 → 2.5 tile jump height.
            // Row 6 platforms are 4 tiles up — 1.5 tile clearance.
            impulse: 297.0,
            cooldown: Timer::from_seconds(2.5, TimerMode::Repeating),
        });
    }

    if matches!(enemy_type, EnemyType::Snake | EnemyType::Possum) {
        entity.insert(PatrolOnly);
    }

    entity.id()
}

/// Spawns initial Forest enemies at startup (visible during menu screens).
/// Positions use Forest origin_x = -864.0.
pub fn spawn_enemies(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Forest enemies: col_x(col) = -864 + col*18 + 9
    // Y = ground_top = -146.0 (spawner adds COLLIDER_H/2 = 10 → center at -136).
    let enemies = [
        (EnemyType::Dog, Vec2::new(81.0, -146.0), 90.0_f32, 150.0),
        (EnemyType::Snake, Vec2::new(477.0, -146.0), 54.0_f32, 50.0),
        (EnemyType::Possum, Vec2::new(621.0, -146.0), 54.0_f32, 50.0),
    ];

    for (enemy_type, position, patrol_range, hp) in enemies {
        spawn_enemy(
            &mut commands,
            &mut meshes,
            &mut materials,
            &asset_server,
            enemy_type,
            position,
            patrol_range,
            hp,
            None,
        );
    }
}
