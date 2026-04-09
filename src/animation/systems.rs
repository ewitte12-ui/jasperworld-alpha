use avian2d::prelude::LinearVelocity;
use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;

use crate::combat::components::Invulnerable;
use crate::enemies::components::Enemy;
use crate::player::components::Player;

use super::components::{EnemyAnimState, PlayerAnimState, SpriteAnimation};

/// Determines the player's animation state based on physics velocity and status components.
pub fn update_player_anim_state(
    mut query: Query<
        (
            &LinearVelocity,
            &mut PlayerAnimState,
            &mut SpriteAnimation,
            Option<&Invulnerable>,
        ),
        With<Player>,
    >,
) {
    let Ok((velocity, mut anim_state, mut anim, invulnerable)) = query.single_mut() else {
        return;
    };

    // Hysteresis: enter Walking at >30, leave it below 8 — prevents flickering
    // when velocity hovers near the threshold on start/stop.
    let walk_speed = velocity.x.abs();
    let enter_walk = walk_speed > 30.0;
    let leave_walk = walk_speed < 8.0;

    let new_state = if invulnerable.is_some() {
        PlayerAnimState::Hurt
    } else if velocity.y.abs() > 50.0 {
        PlayerAnimState::Jumping
    } else if enter_walk || (*anim_state == PlayerAnimState::Walking && !leave_walk) {
        PlayerAnimState::Walking
    } else {
        PlayerAnimState::Idle
    };

    if new_state != *anim_state {
        *anim_state = new_state;
        // Reset animation to first frame on state change
        anim.current_frame = 0;

        // Update frames for the new state (raccoon.png 4×4 grid)
        // Row 0 (0-3): walk cycle, Row 1 (4-7): idle
        // Row 2 (8-11): jump/action, Row 3 (12): hurt/dizzy
        anim.frames = match new_state {
            PlayerAnimState::Idle => vec![4],                // standing upright (row 1, col 0)
            PlayerAnimState::Walking => vec![4, 5, 6, 7],   // upright standing variants (row 1)
            PlayerAnimState::Jumping => vec![8],             // jump pose (row 2, col 0)
            PlayerAnimState::Hurt => vec![12],               // dizzy with stars (row 3, col 0)
        };
    }
}

/// Determines enemy animation state from velocity and flips sprite to face
/// movement direction. Cosmetic only — does not modify AI, physics, or movement.
pub fn update_enemy_anim_state(
    mut query: Query<
        (
            &LinearVelocity,
            &mut EnemyAnimState,
            &mut SpriteAnimation,
            &mut Transform,
        ),
        With<Enemy>,
    >,
) {
    for (velocity, mut anim_state, mut anim, mut transform) in query.iter_mut() {
        // Flip sprite to face movement direction.
        // Sprites face RIGHT in the sheet; negative scale.x flips to left.
        if velocity.x > 1.0 {
            transform.scale.x = transform.scale.x.abs();
        } else if velocity.x < -1.0 {
            transform.scale.x = -transform.scale.x.abs();
        }

        let new_state = if velocity.x.abs() > 5.0 {
            EnemyAnimState::Walking
        } else {
            EnemyAnimState::Idle
        };

        if new_state != *anim_state {
            *anim_state = new_state;
            anim.current_frame = 0;

            // Enemy sheets: 4×2 grid (512×256, 128px cells)
            // Row 0 (0-3): walk/motion cycle
            // Row 1 (4-7): idle/alert variants
            anim.frames = match new_state {
                EnemyAnimState::Idle => vec![0],
                EnemyAnimState::Walking => vec![0, 1, 2, 3],
            };
        }
    }
}

/// Ticks sprite animation timers and updates mesh UVs.
/// Operates on ALL entities with SpriteAnimation + Mesh3d (player and enemies).
pub fn animate_sprites(
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<(&mut SpriteAnimation, &Mesh3d)>,
) {
    for (mut anim, mesh3d) in query.iter_mut() {
        anim.timer.tick(time.delta());

        if anim.timer.just_finished() {
            if anim.looping {
                anim.current_frame = (anim.current_frame + 1) % anim.frames.len();
            } else if anim.current_frame + 1 < anim.frames.len() {
                anim.current_frame += 1;
            }
        }

        let frame_index = anim.frames[anim.current_frame];

        // Only rewrite mesh UVs when the frame actually changes.
        if frame_index == anim.last_written_frame {
            continue;
        }
        anim.last_written_frame = frame_index;

        let [u_min, v_min, u_max, v_max] = anim.atlas.uv_for_index(frame_index);

        // Rectangle vertex order: TR(0), TL(1), BL(2), BR(3)
        let new_uvs: Vec<[f32; 2]> = vec![
            [u_max, v_min],  // 0 = TR
            [u_min, v_min],  // 1 = TL
            [u_min, v_max],  // 2 = BL
            [u_max, v_max],  // 3 = BR
        ];

        if let Some(mesh) = meshes.get_mut(&mesh3d.0) {
            mesh.insert_attribute(
                Mesh::ATTRIBUTE_UV_0,
                VertexAttributeValues::Float32x2(new_uvs),
            );
        }
    }
}
