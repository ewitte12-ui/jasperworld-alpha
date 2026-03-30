use bevy::prelude::*;

/// Player health.
#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }

    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }
}

/// Invulnerability timer (set on player after taking damage).
#[derive(Component)]
pub struct Invulnerable {
    pub timer: Timer,
}

/// Knockback velocity to apply.
#[derive(Component)]
pub struct Knockback {
    pub velocity: Vec2,
    pub timer: Timer,
}

/// Messages (Bevy 0.18 uses Message instead of Event for queue-based messaging)
#[derive(Message)]
pub struct PlayerDamageEvent {
    pub amount: f32,
    pub knockback: Vec2,
}

#[derive(Message)]
pub struct EnemyKillEvent {
    pub enemy: Entity,
    pub stomp: bool, // was it a stomp?
}
