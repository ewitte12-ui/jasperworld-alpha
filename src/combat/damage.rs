use avian2d::prelude::*;
use bevy::prelude::*;

use crate::enemies::components::ContactDamage;
use crate::player::components::Player;

use super::components::{Health, Invulnerable, Knockback, PlayerDamageEvent};

type PlayerNotInvulnerable = (With<Player>, Without<Invulnerable>);

/// Detects proximity between player and enemies; fires PlayerDamageEvent on contact.
pub fn contact_damage(
    player_query: Query<(Entity, &Transform), PlayerNotInvulnerable>,
    enemy_query: Query<(&Transform, &ContactDamage)>,
    mut damage_events: MessageWriter<PlayerDamageEvent>,
) {
    let Ok((_, player_transform)) = player_query.single() else {
        return;
    };

    let player_pos = player_transform.translation.truncate();

    for (enemy_transform, contact_damage) in enemy_query.iter() {
        let enemy_pos = enemy_transform.translation.truncate();
        let distance = player_pos.distance(enemy_pos);

        // Combined half-extents: 32 + 32 = 64, but use a tighter value for feel
        if distance < 32.0 {
            let knockback_dir = (player_pos - enemy_pos).normalize_or_zero();
            damage_events.write(PlayerDamageEvent {
                amount: contact_damage.amount,
                knockback: knockback_dir * 250.0,
            });
            // Only one damage event per frame
            return;
        }
    }
}

/// Listens to PlayerDamageEvent and applies damage + invulnerability + knockback.
pub fn apply_player_damage(
    mut commands: Commands,
    mut player_query: Query<(Entity, &mut Health), With<Player>>,
    mut damage_events: MessageReader<PlayerDamageEvent>,
) {
    let Ok((player_entity, mut health)) = player_query.single_mut() else {
        return;
    };

    for event in damage_events.read() {
        health.current = (health.current - event.amount).max(0.0);

        commands.entity(player_entity).insert((
            Invulnerable {
                timer: Timer::from_seconds(1.2, TimerMode::Once),
            },
            Knockback {
                velocity: event.knockback,
                timer: Timer::from_seconds(0.15, TimerMode::Once),
            },
        ));
    }
}

/// Ticks Invulnerable timers and removes the component when expired.
pub fn tick_invulnerability(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Invulnerable)>,
    time: Res<Time>,
) {
    for (entity, mut inv) in query.iter_mut() {
        inv.timer.tick(time.delta());
        if inv.timer.just_finished() {
            commands.entity(entity).remove::<Invulnerable>();
        }
    }
}

/// Applies knockback velocity to the player and removes the component when expired.
pub fn tick_knockback(
    mut commands: Commands,
    mut query: Query<(Entity, &mut LinearVelocity, &mut Knockback)>,
    time: Res<Time>,
) {
    for (entity, mut velocity, mut knockback) in query.iter_mut() {
        knockback.timer.tick(time.delta());
        if knockback.timer.just_finished() {
            commands.entity(entity).remove::<Knockback>();
        } else {
            // Apply knockback by setting velocity
            velocity.x = knockback.velocity.x;
            velocity.y = knockback.velocity.y;
        }
    }
}
