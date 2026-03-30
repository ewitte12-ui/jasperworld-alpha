use bevy::prelude::*;

use crate::combat::components::Health;
use crate::player::components::Player;

use super::components::{CollectedEvent, Collectible, CollectibleType, CollectionProgress};

/// Marker component for collectibles that spin in place.
#[derive(Component)]
pub struct Spinning {
    pub speed: f32,
}

/// Rotates all Spinning entities around the Y axis.
pub fn spin_collectibles(time: Res<Time>, mut query: Query<(&mut Transform, &Spinning)>) {
    for (mut transform, spinning) in &mut query {
        transform.rotate_y(spinning.speed * time.delta_secs());
    }
}

/// Pickup collectibles when the player is within 24.0 units.
pub fn pickup_collectibles(
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
    collectible_query: Query<(Entity, &Transform, &Collectible)>,
    mut progress: ResMut<CollectionProgress>,
    mut health_query: Query<&mut Health, With<Player>>,
    mut collected_writer: MessageWriter<CollectedEvent>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (entity, collectible_transform, collectible) in &collectible_query {
        let collectible_pos = collectible_transform.translation.truncate();
        let distance = player_pos.distance(collectible_pos);

        if distance < 64.0 {
            commands.entity(entity).despawn();

            match collectible.collectible_type {
                CollectibleType::Star => {
                    progress.stars_collected += 1;
                    info!("Star collected! {}/{}", progress.stars_collected, progress.stars_total);
                }
                CollectibleType::HealthFood => {
                    if let Ok(mut health) = health_query.single_mut() {
                        health.current = (health.current + 20.0).min(health.max);
                    }
                }
            }

            collected_writer.write(CollectedEvent {
                collectible_type: collectible.collectible_type,
            });
        }
    }
}

/// Spawns a collectible at the given world position.
///
/// Star      → star_collectible.glb (Kenney Platformer Kit, colormap.png texture)
/// HealthFood → apple.glb (Kenney Food Kit, vertex colors)
pub fn spawn_collectible(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    asset_server: &AssetServer,
    position: Vec3,
    collectible_type: CollectibleType,
) {
    match collectible_type {
        CollectibleType::Star => spawn_star_3d(commands, asset_server, position),
        CollectibleType::HealthFood => spawn_health_food_3d(commands, asset_server, position),
    }
    let _ = (meshes, materials);
}

/// Spawns star_collectible.glb (gold star from Kenney Platformer Kit) for level progression.
///
/// Visual Y offset (+6): lifts the star model above platform surfaces so the
/// bottom of the 3D mesh does not clip into tile geometry. The collectible's
/// logical position (used by pickup_collectibles distance check) is the
/// Transform translation, which includes this offset — but pickup_radius is
/// 64 units, so 6 units of vertical shift has zero gameplay impact.
fn spawn_star_3d(commands: &mut Commands, asset_server: &AssetServer, position: Vec3) {
    let visual_offset = Vec3::new(0.0, 6.0, 0.0);
    commands.spawn((
        SceneRoot(asset_server.load("models/star_collectible.glb#Scene0")),
        Transform::from_translation(position + visual_offset).with_scale(Vec3::splat(50.0)),
        Collectible {
            collectible_type: CollectibleType::Star,
        },
        Spinning { speed: 1.5 },
    ));
}

/// Spawns apple.glb as a health pickup.
///
/// Same visual Y offset as stars — prevents clipping into platform surfaces.
fn spawn_health_food_3d(commands: &mut Commands, asset_server: &AssetServer, position: Vec3) {
    let visual_offset = Vec3::new(0.0, 6.0, 0.0);
    commands.spawn((
        SceneRoot(asset_server.load("models/apple.glb#Scene0")),
        Transform::from_translation(position + visual_offset).with_scale(Vec3::splat(80.0)),
        Collectible {
            collectible_type: CollectibleType::HealthFood,
        },
        Spinning { speed: 1.5 },
    ));
}
