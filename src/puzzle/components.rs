use bevy::prelude::*;

/// System set for all transition-related checks and state updates.
///
/// Ordered before `CameraPipeline::Follow` and `TnuaUserControlsSystems` so
/// that `transition_in_progress` is set before any player input or camera
/// movement runs in the same frame.  This guarantees:
///   1. `player_input` sees the flag and suppresses intent
///   2. `camera_follow` reads the post-teleport position, not the pre-teleport one
///   3. No system can observe a half-updated transition state
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TransitionSet;

/// The level gate — blocks exit until all stars collected
#[derive(Component)]
pub struct LevelGate;

/// The level exit trigger region.
///
/// Detection is an axis-aligned bounding box (AABB) centered on the entity's
/// `Transform.translation` (world XY).  `half_extents` defines the half-width
/// and half-height of the box.  The player triggers the exit when their
/// world-space position falls inside this box.
///
/// Region-based rather than radial: the rectangular area aligns with platform
/// geometry, making the trigger boundary predictable and entry-vector agnostic.
#[derive(Component)]
pub struct LevelExit {
    pub next_level: crate::level::level_data::LevelId,
    /// Half-size of the trigger AABB in world units (x = half-width, y = half-height).
    pub half_extents: Vec2,
}

/// Resource tracking game completion
#[derive(Resource, Default)]
pub struct GameProgress {
    pub current_level_index: usize, // 0=Forest
    pub game_complete: bool,
    /// Guard against re-entrant transitions.
    ///
    /// Set to `true` at the start of any transition (level exit or layer switch).
    /// Cleared by [`tick_transition_cooldown`] after `transition_cooldown` reaches
    /// zero — guaranteeing at least one full frame elapses (so deferred commands
    /// despawning the old trigger entity have applied) before re-entry is allowed.
    pub transition_in_progress: bool,
    /// Frames remaining before `transition_in_progress` is cleared.
    /// Set to 1 by transition systems; decremented by [`tick_transition_cooldown`].
    /// WHY 1 frame: deferred despawn commands apply at the end of the frame that
    /// issued them.  Holding the lock for 1 extra frame ensures the old
    /// trigger entity no longer exists when the guard opens.
    pub transition_cooldown: u8,
}
