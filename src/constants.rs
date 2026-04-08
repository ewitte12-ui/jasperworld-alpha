//! Centralized depth and sizing constants.
//!
//! Every Z-value and tile measurement in the game should reference this file.
//! Changing a value here changes it everywhere — no grep required.

// ── Tile sizing ──────────────────────────────────────────────────────
/// Side length of one square tile in world units.
pub const TILE_SIZE: f32 = 18.0;

// ── Z-depth stack (back → front) ────────────────────────────────────
// Camera sits at Z = 100 looking down -Z.  Objects are layered:
pub const Z_SKY: f32 = -100.0;
pub const Z_SKY_OVERLAY: f32 = -99.0;
pub const Z_SKY_STARS: f32 = -98.0;
pub const Z_FAR_PARALLAX: f32 = -80.0;
pub const Z_FAR_ATTEN: f32 = -75.0;
pub const Z_MOUNTAINS: f32 = -70.0;
pub const Z_CLOUDS: f32 = -60.0;
pub const Z_NEAR_PARALLAX: f32 = -50.0;
pub const Z_NEAR_ATTEN: f32 = -38.0;
pub const Z_DECORATION: f32 = -15.0;
pub const Z_SUBLEVEL_BG: f32 = -5.0;
pub const Z_DOOR_PROP: f32 = -1.0;
pub const Z_TILES: f32 = 0.0;
pub const Z_EXIT: f32 = 0.5;
pub const Z_GATE: f32 = 1.0;
pub const Z_SUBLEVEL_PROP: f32 = 3.0;
pub const Z_GAMEPLAY: f32 = 5.0;
pub const Z_FOREGROUND: f32 = 10.0;
pub const Z_VFX: f32 = 20.0;
pub const Z_CAMERA: f32 = 100.0;
