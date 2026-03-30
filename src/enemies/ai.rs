use avian2d::prelude::*;
use bevy::prelude::*;

use crate::player::components::Player;

use super::components::{Enemy, EnemyAI, PatrolOnly};

pub fn enemy_ai(
    mut enemy_query: Query<(&Enemy, &mut EnemyAI, &mut LinearVelocity, &Transform)>,
    player_query: Query<&Transform, With<Player>>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (enemy, ai, mut velocity, transform) in enemy_query.iter_mut() {
        let pos = transform.translation.truncate();

        match *ai {
            EnemyAI::Patrol { direction } => {
                velocity.x = direction as f32 * enemy.speed;
            }
            EnemyAI::Chase => {
                let dx = player_pos.x - pos.x;
                // Keep moving if close — only stop when almost exactly aligned.
                // Use a 4-unit deadband to prevent oscillation.
                if dx.abs() > 4.0 {
                    velocity.x = dx.signum() * enemy.speed;
                } else {
                    velocity.x = 0.0;
                }
            }
            EnemyAI::Return => {
                let dx = enemy.spawn_x - pos.x;
                if dx.abs() > 4.0 {
                    velocity.x = dx.signum() * enemy.speed * 0.5;
                } else {
                    velocity.x = 0.0;
                }
            }
        }
    }
}

pub fn enemy_ai_state_machine(
    mut enemy_query: Query<(&Enemy, &mut EnemyAI, &Transform, Has<PatrolOnly>)>,
    player_query: Query<&Transform, With<Player>>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (enemy, mut ai, transform, patrol_only) in enemy_query.iter_mut() {
        let pos = transform.translation.truncate();
        let dist = pos.distance(player_pos);
        let return_dist = (pos.x - enemy.spawn_x).abs();

        match *ai {
            EnemyAI::Patrol { direction } => {
                let offset = pos.x - enemy.spawn_x;
                // Flip direction at patrol boundary with a small buffer to prevent jitter.
                if offset >= enemy.patrol_range && direction > 0 {
                    *ai = EnemyAI::Patrol { direction: -1 };
                } else if offset <= -enemy.patrol_range && direction < 0 {
                    *ai = EnemyAI::Patrol { direction: 1 };
                }
                // PatrolOnly enemies (Snake, Possum) never chase — their threat is
                // zone denial and timing, not pursuit.  At 55/45 u/s vs player 200 u/s,
                // a chase they cannot win is not a threat; a patrol the player must time is.
                if !patrol_only && dist < 150.0 {
                    *ai = EnemyAI::Chase;
                }
            }
            EnemyAI::Chase => {
                if dist > 250.0 {
                    *ai = EnemyAI::Return;
                }
            }
            EnemyAI::Return => {
                if return_dist < 10.0 {
                    *ai = EnemyAI::Patrol { direction: 1 };
                }
                if dist < 150.0 {
                    *ai = EnemyAI::Chase;
                }
            }
        }
    }
}
