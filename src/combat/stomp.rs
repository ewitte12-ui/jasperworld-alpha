use avian2d::prelude::*;
use bevy::prelude::*;

use crate::enemies::components::{Enemy, StompImmune};
use crate::player::components::Player;

use super::components::EnemyKillEvent;

/// Detects when the player stomps an enemy from above.
pub fn check_stomp(
    mut player_query: Query<(&Transform, &mut LinearVelocity), With<Player>>,
    enemy_query: Query<(Entity, &Transform, Has<StompImmune>), With<Enemy>>,
    mut kill_events: MessageWriter<EnemyKillEvent>,
) {
    let Ok((player_transform, mut player_vel)) = player_query.single_mut() else {
        return;
    };

    // Player must be falling
    if player_vel.y >= -50.0 {
        return;
    }

    let player_pos = player_transform.translation.truncate();
    // Player bottom = center - 12 units
    let player_bottom = player_pos.y - 12.0;

    for (enemy_entity, enemy_transform, stomp_immune) in enemy_query.iter() {
        let enemy_pos = enemy_transform.translation.truncate();
        // Enemy top = center + 24 units
        let enemy_top = enemy_pos.y + 24.0;

        // Check vertical proximity
        let vert_diff = (player_bottom - enemy_top).abs();
        // Check horizontal overlap
        let horiz_diff = (player_pos.x - enemy_pos.x).abs();

        if vert_diff < 16.0 && horiz_diff < 32.0 {
            // Always bounce the player so the landing has physical weight.
            player_vel.y = 300.0;

            // Stomp-immune enemies (e.g. Dog) cannot be killed this way.
            // Damage must come from a deliberate attack (tail slap or later mechanic).
            if !stomp_immune {
                kill_events.write(EnemyKillEvent {
                    enemy: enemy_entity,
                    stomp: true,
                });
            }

            return; // One stomp per frame
        }
    }
}
