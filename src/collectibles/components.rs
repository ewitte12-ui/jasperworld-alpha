use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use crate::level::level_data::LevelId;

#[derive(Component, Debug, Clone, Copy)]
pub enum CollectibleType {
    Star,       // regular collectible, counts toward puzzle
    HealthFood, // restores 20 HP, does NOT count toward puzzle
}

#[derive(Component)]
pub struct Collectible {
    pub collectible_type: CollectibleType,
    /// The pre-offset logical position passed to `spawn_collectible` — i.e. the
    /// raw `[x, y, z]` from the compiled JSON, before the +6 Y visual offset is
    /// applied by the inner spawn functions.
    ///
    /// Used as the persistence key in `pickup_collectibles` so it matches the
    /// key used by `spawn_entities_from_compiled` (which never sees the offset).
    pub logical_pos: Vec3,
}

/// Per-layer record of which collectible positions the player has already
/// collected. Positions are quantized to `(i32, i32, i32)` — JSON-spawned
/// collectibles sit at LDtk tile centers (multiples of 18), so rounding to
/// int is collision-proof.
#[derive(Default, Debug)]
pub struct CollectedSet {
    pub stars: HashSet<(i32, i32, i32)>,
    pub health_foods: HashSet<(i32, i32, i32)>,
}

/// Resource tracking collected stars per level/layer
#[derive(Resource, Default)]
pub struct CollectionProgress {
    pub stars_collected: u32,
    pub stars_total: u32,
    /// Per-(level, layer) persistent record of collected positions.
    ///
    /// Lifecycle:
    /// - `pickup_collectibles` inserts on collection.
    /// - `switch_layer` preserves this across layer transitions within a level.
    /// - `check_level_exit` clears this (via `CollectionProgress::default()`)
    ///   when moving to a new level — each level is a fresh challenge.
    /// - `handle_new_game` clears this (via `CollectionProgress::default()`).
    /// - Save/load does NOT persist this map (known limitation).
    pub collected_by_layer: HashMap<(LevelId, usize), CollectedSet>,
}

impl CollectionProgress {
    /// Quantizes a world-space position to a stable integer key for the
    /// collected-by-layer map. Uses `.round()` so sub-unit JSON drift doesn't
    /// cause key mismatches.
    pub fn pos_key(pos: Vec3) -> (i32, i32, i32) {
        (pos.x.round() as i32, pos.y.round() as i32, pos.z.round() as i32)
    }
}

/// Marker: scene children haven't had emissive applied yet.
/// Removed once the system clones + modifies the loaded GLB materials.
///
/// `keep_lit`: when true, scene lighting is preserved (unlit stays false)
/// so directional light/shadow still defines 3D edges. When false, the
/// material is set to unlit (flat glow, no edge definition).
#[derive(Component)]
pub struct MakeEmissive {
    pub color: LinearRgba,
    pub keep_lit: bool,
}

#[derive(Message)]
pub struct CollectedEvent {
    pub collectible_type: CollectibleType,
}
