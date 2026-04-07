use avian2d::prelude::*;
use bevy::prelude::*;

use bevy_tnua::builtins::{TnuaBuiltinJumpConfig, TnuaBuiltinWalkConfig};
use bevy_tnua::{TnuaConfig, TnuaController};
use bevy_tnua_avian2d::TnuaAvian2dSensorShape;

use super::components::{FacingDirection, Player, PlayerControlScheme, PlayerControlSchemeConfig};
use crate::animation::components::{PlayerAnimState, PlayerModelPending, PlayerModelVisual};
use crate::combat::components::Health;
use crate::physics::config::{GameLayer, PhysicsConfig};

/// Spawns the raccoon player with avian2d physics, bevy-tnua character
/// controller, and a static 3D GLB model (assets/models/jasper.glb).
///
/// Architecture: physics lives on the PARENT entity (scale 1.0 so the
/// collider isn't scaled). The visual model is a CHILD entity with its
/// own scale and Y offset to align feet with the collider bottom.
///
/// The visual model has no skeleton — animation is handled procedurally
/// by `animate_player_procedural` in the animation module.
pub fn setup_player_physics(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut scheme_configs: ResMut<Assets<PlayerControlSchemeConfig>>,
    physics_config: Res<PhysicsConfig>,
) {
    // Enemies still need meshes/materials via their own spawn systems that
    // share the same system set. Suppress unused-variable warnings here.
    let _ = (&mut meshes, &mut materials);

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

    // Model native height ≈ 1.0 unit (bottom-anchored, Y=0 to Y≈1.0).
    // City lamp post (0.675 × 70 = 47.25 world units) should be 1.2× Jasper's
    // height → Jasper = 47.25 / 1.2 ≈ 39.4 → scale ≈ 40.
    // WHAT BREAKS: if changed, visual size mismatches collider/world geometry.
    let model_scale = 40.0;

    // The collider is centered on the entity position. Tnua floats the entity
    // so collider bottom sits at ground level. The model's Y=0 is at the
    // entity origin, so shift it down by half the collider height to align
    // the model's feet with the collider bottom (= ground).
    // Slightly less than -float_height to raise feet above ground surface.
    let model_y_offset = -float_height + 8.0;

    // Model faces +X (right). Add a -45° Y tilt for better visual read.
    // The facing system in input.rs composes this same tilt with direction changes.
    let base_rotation = Quat::from_rotation_y((-45.0_f32).to_radians());

    commands.spawn((
        // Parent entity: physics + game logic. Scale stays 1.0 so the
        // collider dimensions are in world units as specified.
        Player,
        FacingDirection::default(),
        Health::new(100.0),
        Transform::from_xyz(-261.0, -120.0, 5.0),
        Visibility::default(),
        RigidBody::Dynamic,
        Collider::rectangle(physics_config.player_width - 2.0, physics_config.player_height),
        Friction::ZERO,
        LockedAxes::ROTATION_LOCKED,
        CollisionLayers::new(
            GameLayer::Player,
            [GameLayer::Default, GameLayer::Ground, GameLayer::Platform],
        ),
        TnuaAvian2dSensorShape(Collider::rectangle(14.0, 0.0)),
        TnuaController::<PlayerControlScheme>::default(),
        TnuaConfig::<PlayerControlScheme>(config_handle),
        PlayerAnimState::Idle,
        // Signals that skeletal animation setup is pending. Removed once
        // setup_player_animation discovers the AnimationPlayer descendant.
        PlayerModelPending,
    )).with_children(|parent| {
        // Child entity: visual model with scale, offset, and rotation.
        // Rotation is updated by `animate_player_procedural` for direction + animation.
        parent.spawn((
            SceneRoot(asset_server.load("models/jasper.glb#Scene0")),
            Transform {
                translation: Vec3::new(0.0, model_y_offset, 0.0),
                rotation: base_rotation,
                scale: Vec3::splat(model_scale),
            },
            PlayerModelVisual,
        ));
    });
}
