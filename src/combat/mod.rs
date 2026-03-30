pub mod components;
pub mod damage;
pub mod kill;
pub mod stomp;
pub mod tail_slap;

use bevy::prelude::*;
use components::{EnemyKillEvent, PlayerDamageEvent};

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<PlayerDamageEvent>()
            .add_message::<EnemyKillEvent>()
            .add_systems(
                Update,
                (
                    damage::contact_damage,
                    damage::apply_player_damage,
                    damage::tick_invulnerability,
                    damage::tick_knockback,
                    stomp::check_stomp,
                    tail_slap::check_tail_slap,
                    kill::process_kills,
                )
                    .chain(),
            );
    }
}
