use bevy::prelude::*;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyType {
    Dog,      // medium speed, ground patrol
    Squirrel, // fast, erratic
    Snake,    // slow, stays low
    Rat,      // medium speed, aggressive chaser
    Possum,   // slow, plays dead (low speed)
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyAI {
    Patrol { direction: i8 }, // 1 = right, -1 = left
    Chase,
    Return,
}

#[derive(Component)]
pub struct Enemy {
    pub enemy_type: EnemyType,
    pub health: f32,
    pub speed: f32,
    pub patrol_range: f32, // max distance from spawn before turning
    pub spawn_x: f32,
}

#[derive(Component)]
pub struct ContactDamage {
    pub amount: f32,
}

/// Marks enemies that only patrol — they never transition to Chase or Return.
/// WHY Snake and Possum: their speeds (55 and 45 u/s) are 27–22% of the player's 200 u/s.
/// A chaser that cannot close distance provides zero threat.
/// Pure patrol at their speeds creates meaningful zone-denial and timing decisions
/// without the fake-chase that makes the threat appear weaker than it is.
/// Per entity_movement_contract: "movement that does not change player decisions is non-movement."
#[derive(Component)]
pub struct PatrolOnly;

/// Suppresses all enemy spawning and AI during traversal testing.
///
/// When this resource is present:
///   - Enemy entities are despawned immediately on level load (OnEnter Playing)
///   - enemy_ai and enemy_ai_state_machine run conditions return false
///   - spawn_enemy calls complete (entities are created) then wiped — no ad-hoc
///     guard inside the spawner is needed; the cleanup system is authoritative
///
/// Inserted by DebugStartPlugin when debug_start.json sets "traversal_blockout": true.
/// Never inserted in release builds. Remove the resource to restore normal gameplay.
#[derive(Resource)]
pub struct TraversalBlockoutMode;

/// Gives an enemy the ability to jump periodically.
/// `impulse` is the initial velocity.y; `cooldown` controls how often.
/// Dog uses impulse 297: v = sqrt(2 × 980 × 45) ≈ 297 reaches 2.5 tiles (45 units)
/// under gravity 980. Row-6 platforms are 4 tiles up — 1.5 tile clearance.
#[derive(Component)]
pub struct EnemyJump {
    pub impulse: f32,
    pub cooldown: Timer,
}

/// Marks enemies that cannot be killed by stomping from above.
/// A stomp still bounces the player; it just deals no damage.
/// WHY Dog only: dogs have thick fur/skulls — stomping is ineffective.
/// Damage must come from a deliberate attack (tail slap or a later mechanic).
#[derive(Component)]
pub struct StompImmune;
