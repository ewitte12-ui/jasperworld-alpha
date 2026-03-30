pub mod config;
pub mod one_way;

use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_tnua_avian2d::TnuaAvian2dPlugin;

use config::PhysicsConfig;
use one_way::OneWayPlatformHooks;

/// Adds avian2d physics, the Tnua avian2d backend, gravity, and game physics config.
///
/// NOTE: `TnuaControllerPlugin<S>` is generic over the control scheme `S: TnuaScheme` and must
/// be registered separately by the module that defines the player's control scheme (e.g.
/// `PlayerPlugin`), using the same schedule as `TnuaAvian2dPlugin` (`PhysicsSchedule`):
///
/// ```ignore
/// app.add_plugins(TnuaControllerPlugin::<MyControlScheme>::new(PhysicsSchedule));
/// ```
pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app
            // Core avian2d physics — runs in FixedPostUpdate by default, which drives
            // PhysicsSchedule internally.
            .add_plugins(
                PhysicsPlugins::default()
                    .with_length_unit(18.0)
                    .with_collision_hooks::<OneWayPlatformHooks>(),
            )
            // Tnua avian2d backend — must share the same schedule as avian2d's PhysicsSchedule.
            .add_plugins(TnuaAvian2dPlugin::new(PhysicsSchedule))
            // Pixel-scale gravity: 9.8 m/s² × 100 px/m ≈ 980 units/s²
            .insert_resource(Gravity(Vec2::NEG_Y * 980.0))
            .insert_resource(PhysicsConfig::default());
    }
}
