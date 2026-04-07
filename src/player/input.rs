use bevy::prelude::*;
use bevy_tnua::TnuaController;
use bevy_tnua::builtins::{TnuaBuiltinJump, TnuaBuiltinWalk};

use super::components::{FacingDirection, Player, PlayerControlScheme};
use crate::puzzle::components::GameProgress;

/// Reads keyboard input and drives the Tnua character controller.
///
/// - Arrow keys / WASD for horizontal movement.
/// - Space / ArrowUp / W to jump.
///
/// During a level/layer transition (`transition_in_progress`), all player
/// intent is suppressed — Tnua receives zero motion and no jump action.
/// Physics (gravity, momentum) still runs; only player intent is blocked.
///
/// Jump buffering (press early before landing) is handled natively by
/// [`TnuaBuiltinJumpConfig::input_buffer_time`].  Coyote time (jump shortly after
/// leaving a ledge) is handled natively by [`TnuaBuiltinWalkConfig::coyote_time`].
///
/// This system must run inside [`TnuaUserControlsSystems`] (which is in `PostUpdate`).
pub fn player_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    game_progress: Option<Res<GameProgress>>,
    mut query: Query<
        (
            &mut TnuaController<PlayerControlScheme>,
            &mut FacingDirection,
        ),
        With<Player>,
    >,
) {
    let Ok((mut controller, mut facing)) = query.single_mut() else {
        return;
    };

    // ── Transition lockout ────────────────────────────────────────────────────
    // When a transition is active, feed zero intent so player input cannot
    // fight the teleport. Physics (gravity) continues normally.
    let locked = game_progress
        .as_ref()
        .is_some_and(|gp| gp.transition_in_progress);

    // ── Horizontal input ──────────────────────────────────────────────────────
    let mut move_x = 0.0_f32;
    if !locked {
        if keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyA) {
            move_x -= 1.0;
        }
        if keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyD) {
            move_x += 1.0;
        }
    }

    // ── Facing direction ──────────────────────────────────────────────────────
    // Only update direction on input; the visual rotation is applied by
    // `animate_player_procedural` which runs separately on the child entity.
    if move_x < 0.0 {
        *facing = FacingDirection::Left;
    } else if move_x > 0.0 {
        *facing = FacingDirection::Right;
    }

    // ── Feed Tnua basis (walk) ────────────────────────────────────────────────
    // desired_motion is a direction vector; Tnua multiplies it by config.speed.
    // X axis is horizontal, Z is depth — we stay in the XY plane.
    controller.basis = TnuaBuiltinWalk {
        desired_motion: Vec3::new(move_x, 0.0, 0.0),
        desired_forward: None,
    };

    // ── Feed Tnua action (jump) ───────────────────────────────────────────────
    // Jump must be fed every frame the button is held so Tnua can apply
    // variable-height jump physics (release early = shorter jump).
    // Jump buffering and coyote time are configured in the TnuaBuiltinJumpConfig /
    // TnuaBuiltinWalkConfig assets respectively — no manual timers needed.
    let jump_held = !locked
        && (keyboard.pressed(KeyCode::Space)
            || keyboard.pressed(KeyCode::ArrowUp)
            || keyboard.pressed(KeyCode::KeyW));

    controller.initiate_action_feeding();
    if jump_held {
        controller.action(PlayerControlScheme::Jump(TnuaBuiltinJump::default()));
    }
}