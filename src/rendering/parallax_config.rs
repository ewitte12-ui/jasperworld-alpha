use serde::Deserialize;

// ── Forest ────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct ForestBgConfig {
    pub mountains: Vec<MountainEntry>,
    pub far_trees: TreeLayerConfig,
    pub near_trees: TreeLayerConfig,
    pub clouds: Vec<CloudEntry>,
    pub attenuation: Vec<AttenuationEntry>,
    /// Optional sky overlay — Forest has none; field exists for structural parity.
    #[serde(default)]
    pub overlay: Option<OverlayEntry>,
}

#[derive(Deserialize)]
pub struct MountainEntry {
    pub model: String,
    pub x: f32,
    pub native_h: f32,
    pub scale: f32,
}

#[derive(Deserialize)]
pub struct TreeLayerConfig {
    pub x_start: i32,
    pub x_end: i32,
    pub step: usize,
    pub y: f32,
    pub z: f32,
    pub factor: f32,
    pub models: Vec<String>,
    pub scales: Vec<f32>,
    pub scale_z: f32,
    /// Whether each model in this layer is center-anchored (origin at midpoint).
    /// Center-anchored models need +scale/2 added to Y so their base sits at `y`.
    #[serde(default)]
    pub center_anchored: bool,
    /// Per-instance X offset added after the loop position (e.g. 60.0 staggers trees).
    #[serde(default)]
    pub x_offset: f32,
}

#[derive(Deserialize)]
pub struct CloudEntry {
    pub texture: String,
    pub x: f32,
    pub y: f32,
    pub scale: f32,
    /// World-space Z position of the cloud quad.
    pub z: f32,
    /// ParallaxLayer factor for this cloud entry.
    pub factor: f32,
}

#[derive(Deserialize)]
pub struct AttenuationEntry {
    /// World-space Z position of the semi-transparent attenuation plane.
    pub z: f32,
    /// ParallaxLayer factor for this plane.
    pub factor: f32,
    /// RGBA color (each component 0.0–1.0) for the overlay material.
    pub color: [f32; 4],
}

// ── Overlay ───────────────────────────────────────────────────────────────────

/// A full-screen sky overlay rectangle spawned just in front of the sky backdrop.
/// Used by Subdivision (overcast grey-blue) and City (night dark navy).
/// RGBA color components are 0.0–1.0.
#[derive(Deserialize)]
pub struct OverlayEntry {
    /// World-space Z for the overlay quad (e.g. -99.0, just in front of sky at -100).
    pub z: f32,
    /// ParallaxLayer factor — should match the sky backdrop (e.g. 0.20).
    pub factor: f32,
    /// RGBA color, each component 0.0–1.0.
    pub color: [f32; 4],
    /// World-space width of the overlay rectangle in game units.
    pub width: f32,
    /// World-space height of the overlay rectangle in game units.
    pub height: f32,
}

// ── Subdivision ───────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct SubdivisionBgConfig {
    pub near_houses: HouseLayerConfig,
    pub far_houses: HouseLayerConfig,
    pub trees: TreeLayerConfig,
    pub attenuation: Vec<AttenuationEntry>,
    /// Optional sky overlay quad (overcast effect).
    /// Uses `#[serde(default)]` so existing JSON without this field still parses.
    #[serde(default)]
    pub overlay: Option<OverlayEntry>,
}

#[derive(Deserialize)]
pub struct HouseLayerConfig {
    pub x_start: i32,
    pub x_end: i32,
    pub step: usize,
    pub y: f32,
    pub z: f32,
    pub factor: f32,
    /// Optional RGB tint applied as SceneTint::Multiply to all models in this layer.
    #[serde(default)]
    pub tint: Option<[f32; 3]>,
    pub models: Vec<HouseModelEntry>,
    pub scales: Vec<f32>,
    /// Scale multiplier for the X (depth) axis after Y-axis rotation.
    /// Flattens the model's depth so it doesn't read as 3D volume under the camera tilt.
    pub depth_scale: f32,
    /// Y-axis rotation in radians. -PI/2 faces models toward -Z (camera).
    pub rotation_y: f32,
    /// Per-instance X offset added after the loop position (e.g. 60.0 staggers trees).
    #[serde(default)]
    pub x_offset: f32,
    /// If Some, overrides the X position for the first spawned instance (i == 0).
    /// Used to shift the first far-house half off-screen at the level edge.
    #[serde(default)]
    pub first_x_override: Option<i32>,
}

#[derive(Deserialize)]
pub struct HouseModelEntry {
    pub path: String,
    /// Native model height along Y (used to compute center-anchoring Y offset).
    pub native_h: f32,
    /// If true, model origin is at its midpoint — shift Y up by native_h*scale*0.5.
    #[serde(default)]
    pub center_anchored: bool,
}

// ── City ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct CityBgConfig {
    pub near_buildings: CityBuildingLayer,
    pub far_buildings: CityFarBuildingLayer,
    pub attenuation: Vec<AttenuationEntry>,
    /// Optional sky overlay quad (night atmosphere effect).
    /// Uses `#[serde(default)]` so existing JSON without this field still parses.
    #[serde(default)]
    pub overlay: Option<OverlayEntry>,
}

#[derive(Deserialize)]
pub struct CityBuildingLayer {
    pub x_start: i32,
    pub x_end: i32,
    pub step: usize,
    pub y: f32,
    pub z: f32,
    pub factor: f32,
    /// Optional RGB tint applied as SceneTint::Multiply.
    #[serde(default)]
    pub tint: Option<[f32; 3]>,
    /// Y-axis rotation in radians.
    pub rotation_y: f32,
    pub models: Vec<CityBuildingModelEntry>,
    pub scales: Vec<f32>,
    /// Fraction of uniform scale used for the Z (depth) axis to prevent 3D depth
    /// from reading as width under the camera's downward tilt.
    pub depth_scale_factor: f32,
}

#[derive(Deserialize)]
pub struct CityBuildingModelEntry {
    pub path: String,
    /// Scale normalizer: compensates for a model's native height differing from the
    /// standard game-unit height so that the base `scales` values remain consistent
    /// across models. Applied to scale before Y-anchoring. Set to 1.0 when no
    /// normalization is needed.
    pub native_h_mult: f32,
    /// If true, model origin is at its midpoint — shift Y up by scale*0.5.
    #[serde(default)]
    pub center_anchored: bool,
}

#[derive(Deserialize)]
pub struct CityFarBuildingLayer {
    pub x_start: i32,
    pub x_end: i32,
    pub step: usize,
    pub y: f32,
    pub z: f32,
    pub factor: f32,
    /// Optional RGB tint applied as SceneTint::Multiply.
    #[serde(default)]
    pub tint: Option<[f32; 3]>,
    pub models: Vec<String>,
    pub scales: Vec<f32>,
    /// Multiplier applied to the Y component of scale (stretches buildings vertically).
    pub y_stretch: f32,
    /// Fixed Z scale (depth). Flat value prevents 3D depth from showing as width.
    pub scale_z: f32,
}

// ── Sanctuary ─────────────────────────────────────────────────────────────────

/// Top-level config for the Sanctuary background (Phase 6).
/// Fields mirror the Forest pattern for structural parity; populated by
/// assets/configs/sanctuary_bg.json when that file is created.
#[derive(Deserialize)]
pub struct SanctuaryBgConfig {
    pub far_background: SanctuaryBackgroundEntry,
    pub far_trees: TreeLayerConfig,
    pub near_trees: TreeLayerConfig,
    pub attenuation: Vec<AttenuationEntry>,
    #[serde(default)]
    pub overlay: Option<OverlayEntry>,
}

/// A single wide background model (e.g. a temple or mountain wall) used as
/// the deepest layer in the Sanctuary scene.
#[derive(Deserialize)]
pub struct SanctuaryBackgroundEntry {
    pub model: String,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub scale: f32,
    pub factor: f32,
    /// X-axis rotation in radians (e.g. -FRAC_PI_2 to stand a flat model upright).
    #[serde(default)]
    pub rotation_x: f32,
    /// Y-axis rotation in radians (0.0 = face camera directly).
    #[serde(default)]
    pub rotation_y: f32,
    /// Z-axis rotation in radians (negative = clockwise tilt from camera view).
    /// Used to level models whose geometry tilts on one side.
    #[serde(default)]
    pub rotation_z: f32,
}

// ── Loader ────────────────────────────────────────────────────────────────────

/// Load and deserialize a JSON config file at `path`.
///
/// Returns `None` with a warning log if the file cannot be read or parsed.
/// Callers should fall back to hard-coded defaults when this returns `None`
/// so a missing/malformed config file never crashes the game.
pub fn load_config<T: for<'de> Deserialize<'de>>(path: &str) -> Option<T> {
    let contents = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[parallax_config] could not read '{}': {}", path, e);
            return None;
        }
    };
    match serde_json::from_str(&contents) {
        Ok(cfg) => Some(cfg),
        Err(e) => {
            eprintln!("[parallax_config] could not parse '{}': {}", path, e);
            None
        }
    }
}
