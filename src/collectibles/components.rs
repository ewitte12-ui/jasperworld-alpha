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

#[derive(Message)]
pub struct CollectedEvent {
    pub collectible_type: CollectibleType,
}
