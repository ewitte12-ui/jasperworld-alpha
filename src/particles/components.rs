use bevy::prelude::*;

#[derive(Component)]
pub struct Particle {
    pub velocity: Vec2,
    pub lifetime: Timer,
    pub fade: bool, // if true, alpha decreases over time
}
