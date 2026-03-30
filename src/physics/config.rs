use avian2d::prelude::*;
use bevy::prelude::*;

/// Physics configuration constants.
#[derive(Resource, Debug, Clone)]
pub struct PhysicsConfig {
    /// Player run speed in world units per second.
    pub run_speed: f32,
    /// Jump height in world units (5 tiles × 18 units/tile).
    pub jump_height: f32,
    /// Width of player collider.
    pub player_width: f32,
    /// Height of player collider.
    pub player_height: f32,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            run_speed: 200.0,
            jump_height: 90.0, // 5 tiles × 18 units
            player_width: 16.0,
            player_height: 22.0,
        }
    }
}

/// Collision layers for the game world.
///
/// The `Default` variant (layer 0) is used by entities that do not specify a layer.
/// The `#[derive(PhysicsLayer)]` macro assigns bit positions automatically:
///   - Default    → bit 0
///   - Player     → bit 1
///   - Ground     → bit 2
///   - Platform   → bit 3  (one-way platforms)
///   - Enemy      → bit 4
///   - Collectible → bit 5
#[derive(PhysicsLayer, Default, Clone, Copy, Debug)]
pub enum GameLayer {
    #[default]
    Default,
    Player,
    Ground,
    Platform, // one-way platforms
    Enemy,
    Collectible,
}
