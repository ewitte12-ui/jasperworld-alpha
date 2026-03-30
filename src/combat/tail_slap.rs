use bevy::prelude::*;

use crate::enemies::components::Enemy;
use crate::player::components::{FacingDirection, Player};

use super::components::EnemyKillEvent;

/// Detects a tail slap attack when the player presses F.
pub fn check_tail_slap(
    keyboard: Res<ButtonInput<KeyCode>>,
    player_query: Query<(&Transform, &FacingDirection), With<Player>>,
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,
    mut kill_events: MessageWriter<EnemyKillEvent>,
) {
    if !keyboard.just_pressed(KeyCode::KeyF) {
        return;
    }

    let Ok((player_transform, facing)) = player_query.single() else {
        return;
    };

    let player_pos = player_transform.translation.truncate();

    // Hitbox center: 28 units in front of the player
    let hitbox_offset = match facing {
        FacingDirection::Right => Vec2::new(28.0, 0.0),
        FacingDirection::Left => Vec2::new(-28.0, 0.0),
    };
    let hitbox_center = player_pos + hitbox_offset;

    // Hitbox half-extents: 28 wide, 24 tall
    let hitbox_half = Vec2::new(28.0, 24.0);

    for (enemy_entity, enemy_transform) in enemy_query.iter() {
        let enemy_pos = enemy_transform.translation.truncate();
        let diff = (enemy_pos - hitbox_center).abs();

        if diff.x < hitbox_half.x + 24.0 && diff.y < hitbox_half.y + 24.0 {
            kill_events.write(EnemyKillEvent {
                enemy: enemy_entity,
                stomp: false,
            });
        }
    }
}
