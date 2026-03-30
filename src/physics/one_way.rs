use avian2d::prelude::*;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::level::components::OneWayPlatform;

/// Collision hooks that implement one-way platform behavior.
///
/// Platforms with [`OneWayPlatform`] + [`ActiveCollisionHooks::MODIFY_CONTACTS`] only
/// block entities landing from above. Side contacts and contacts from below are
/// rejected, so entities can jump through platforms and walk off edges freely.
#[derive(SystemParam)]
pub struct OneWayPlatformHooks<'w, 's> {
    platforms: Query<'w, 's, (), With<OneWayPlatform>>,
}

impl CollisionHooks for OneWayPlatformHooks<'_, '_> {
    fn modify_contacts(&self, contacts: &mut ContactPair, _commands: &mut Commands) -> bool {
        let is_platform_1 = self.platforms.contains(contacts.collider1);
        let is_platform_2 = self.platforms.contains(contacts.collider2);

        if !is_platform_1 && !is_platform_2 {
            return true; // Neither entity is a one-way platform
        }

        if contacts.manifolds.is_empty() {
            return false;
        }

        // manifold.normal points from collider1 → collider2.
        // Compute normal FROM the platform TO the other entity.
        // Only keep the collision if ALL manifold normals point upward (top-face contact).
        // This rejects side contacts (horizontal normals) and bottom contacts,
        // allowing entities to jump through from below and walk off edges.
        contacts.manifolds.iter().all(|manifold| {
            let normal_from_platform = if is_platform_1 {
                manifold.normal
            } else {
                -manifold.normal
            };
            // Threshold 0.5 ≈ 60° from horizontal — must be a clear top-face contact
            normal_from_platform.y > 0.5
        })
    }
}
