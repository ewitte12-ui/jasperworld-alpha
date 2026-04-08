use bevy::prelude::*;

use super::components::EnemyKillEvent;

/// Processes EnemyKillEvents and despawns enemy entities.
pub fn process_kills(mut commands: Commands, mut kill_events: MessageReader<EnemyKillEvent>) {
    for event in kill_events.read() {
        // TODO: add score tracking and death particles
        commands.entity(event.enemy).despawn();
    }
}
