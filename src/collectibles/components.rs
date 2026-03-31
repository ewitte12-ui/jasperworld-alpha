use bevy::prelude::*;

#[derive(Component, Debug, Clone, Copy)]
pub enum CollectibleType {
    Star,       // regular collectible, counts toward puzzle
    HealthFood, // restores 20 HP, does NOT count toward puzzle
}

#[derive(Component)]
pub struct Collectible {
    pub collectible_type: CollectibleType,
}

/// Resource tracking collected stars per level/layer
#[derive(Resource, Default)]
pub struct CollectionProgress {
    pub stars_collected: u32,
    pub stars_total: u32,
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
