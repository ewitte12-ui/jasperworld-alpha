use avian2d::prelude::*;
use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;

use bevy_tnua::builtins::{TnuaBuiltinJumpConfig, TnuaBuiltinWalkConfig};
use bevy_tnua::{TnuaConfig, TnuaController};
use bevy_tnua_avian2d::TnuaAvian2dSensorShape;

use super::components::{FacingDirection, Player, PlayerControlScheme, PlayerControlSchemeConfig};
use crate::animation::components::{PlayerAnimState, SpriteAnimation};
use crate::combat::components::Health;
use crate::physics::config::{GameLayer, PhysicsConfig};

/// Raccoon spritesheet layout (assets/raccoon.png, 512×512, 4 cols × 4 rows, 128px cells).
const RACCOON_COLS: f32 = 4.0;
const RACCOON_ROWS: f32 = 4.0;
const ATLAS_W: f32 = 512.0;
const ATLAS_H: f32 = 512.0;
/// Display size of the player sprite in world units.
const PLAYER_SPRITE_W: f32 = 28.0;
const PLAYER_SPRITE_H: f32 = 32.0;

/// Spawns the raccoon player with avian2d physics and bevy-tnua character controller.
pub fn setup_player_physics(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut scheme_configs: ResMut<Assets<PlayerControlSchemeConfig>>,
    physics_config: Res<PhysicsConfig>,
) {
    // Idle frame: row 1, col 0 (standing upright) with half-texel inset.
    let col = 0.0_f32;
    let row = 1.0_f32;
    let eps_u = 0.5 / ATLAS_W;
    let eps_v = 0.5 / ATLAS_H;
    let u_min = col / RACCOON_COLS + eps_u;
    let v_min = row / RACCOON_ROWS + eps_v;
    let u_max = (col + 1.0) / RACCOON_COLS - eps_u;
    let v_max = (row + 1.0) / RACCOON_ROWS - eps_v;

    // Shift quad up because sprite content is bottom-aligned in cell (~60% down),
    // so the visible raccoon sits below the collider center without an offset.
    let y_offset = 5.0;
    let hw = PLAYER_SPRITE_W / 2.0;
    let hh = PLAYER_SPRITE_H / 2.0;
    let mut mesh = Mesh::from(Rectangle::new(PLAYER_SPRITE_W, PLAYER_SPRITE_H));
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float32x3(vec![
            [hw,  hh + y_offset, 0.0],
            [-hw, hh + y_offset, 0.0],
            [-hw, -hh + y_offset, 0.0],
            [hw,  -hh + y_offset, 0.0],
        ]),
    );
    let uvs = vec![
        [u_max, v_min],
        [u_min, v_min],
        [u_min, v_max],
        [u_max, v_max],
    ];
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, VertexAttributeValues::Float32x2(uvs));
    let mesh_handle = meshes.add(mesh);

    let texture: Handle<Image> = asset_server.load("raccoon.png");
    // WHY double_sided + cull_mode None: the sprite flips left/right by negating
    // transform.scale.x (see player/input.rs).  A negative x scale inverts the
    // mesh winding so the back face points toward the camera — with backface
    // culling ON the flipped sprite would be invisible.  double_sided disables
    // that culling so the sprite is visible in both orientations.
    let material = materials.add(StandardMaterial {
        base_color_texture: Some(texture),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        double_sided: true,
        cull_mode: None,
        ..default()
    });

    let float_height = physics_config.player_height / 2.0;

    let config_handle = scheme_configs.add(PlayerControlSchemeConfig {
        basis: TnuaBuiltinWalkConfig {
            float_height,
            speed: physics_config.run_speed,
            acceleration: 1200.0,
            air_acceleration: 600.0,
            coyote_time: 0.12,
            free_fall_extra_gravity: 980.0,
            spring_strength: 1200.0,
            spring_dampening: 1.2,
            cling_distance: 1.0,
            ..Default::default()
        },
        jump: TnuaBuiltinJumpConfig {
            height: physics_config.jump_height,
            input_buffer_time: 0.15,
            takeoff_extra_gravity: 300.0,
            fall_extra_gravity: 400.0,
            shorten_extra_gravity: 600.0,
            ..Default::default()
        },
    });

    // Spawn at y slightly higher to account for sprite center vs feet.
    // The sprite is centered on the entity; raising spawn y by 8 puts feet on ground.
    commands.spawn((
        Player,
        FacingDirection::default(),
        Health::new(100.0),
        Mesh3d(mesh_handle),
        MeshMaterial3d(material),
        Transform::from_xyz(-261.0, -120.0, 5.0),
        RigidBody::Dynamic,
        Collider::rectangle(physics_config.player_width - 2.0, physics_config.player_height),
        Friction::ZERO,
        LockedAxes::ROTATION_LOCKED,
        CollisionLayers::new(
            GameLayer::Player,
            [GameLayer::Default, GameLayer::Ground, GameLayer::Platform],
        ),
    )).insert((
        TnuaAvian2dSensorShape(Collider::rectangle(14.0, 0.0)),
        TnuaController::<PlayerControlScheme>::default(),
        TnuaConfig::<PlayerControlScheme>(config_handle),
        SpriteAnimation {
            frames: vec![4],
            current_frame: 0,
            timer: Timer::from_seconds(0.12, TimerMode::Repeating),
            looping: true,
            last_written_frame: usize::MAX,
            atlas: crate::animation::components::AtlasLayout::RACCOON,
        },
        PlayerAnimState::Idle,
    ));
}
