use bevy::prelude::*;

/// Screen flash effect (for level transitions, taking damage).
#[derive(Component)]
pub struct ScreenFlash {
    pub timer: Timer,
    pub color: Color,
}

/// Level name display that fades out.
#[derive(Component)]
pub struct LevelNameFlash {
    pub timer: Timer,
}

/// Marker for camera-relative VFX particles (rain, leaves, dust).
/// Per jasper_camera_world_anchor_guardrail_v2: Category 4 exception.
/// Spawn position is computed from camera position to maintain viewport density,
/// but once spawned each particle moves independently in world-space.
#[derive(Component)]
pub struct CameraRelativeVfx;

/// Weather particle emitter.
#[derive(Component)]
pub struct WeatherEmitter {
    pub spawn_timer: Timer,
    pub particle_type: WeatherType,
}

#[derive(Clone, Copy)]
pub enum WeatherType {
    Leaves, // Forest
    Rain,   // City
    Dust,   // Sanctuary
}
