pub mod components;
pub mod systems;

use bevy::prelude::*;

use components::{CollectedEvent, CollectionProgress};

pub struct CollectiblesPlugin;

impl Plugin for CollectiblesPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CollectionProgress::default())
            .add_message::<CollectedEvent>()
            .add_systems(
                Update,
                (
                    systems::pickup_collectibles,
                    systems::spin_collectibles,
                    systems::apply_emissive_to_collectibles,
                ),
            );
    }
}
