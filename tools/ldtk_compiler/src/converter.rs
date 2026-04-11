use crate::ldtk_schema::{LdtkEntityInstance, LdtkLayerInstance, LdtkRoot};

/// Game-space tile size in world units.
const TILE_SIZE: f32 = 18.0;

/// Default world-space origin X applied when the level has no OriginX field.
const DEFAULT_ORIGIN_X: f32 = -864.0;

/// Default world-space origin Y applied when the level has no OriginY field.
const DEFAULT_ORIGIN_Y: f32 = -200.0;

/// Returns the normalized distance from the model's visual bottom to its 3D
/// origin for models whose origin sits at (or near) the vertical center.
/// The converter shifts these models up by `offset * scale_y` so the visual
/// bottom aligns with the LDtk editor placement.
///
/// Values were measured from the actual GLB mesh bounding boxes using trimesh.
/// A model is included here when `abs(y_min) >= 0.2` — large enough that the
/// visual sinking is noticeable in-game.
///
/// Returns `None` for bottom-anchored models (no offset needed).
fn center_anchor_half_height(model_id: &str) -> Option<f32> {
    // Strip any leading path components so both "models/city/taxi.glb"
    // and "taxi.glb" match.
    let filename = model_id.rsplit('/').next().unwrap_or(model_id);
    match filename {
        // Tripo rocks — origin at model center
        "large_rock.glb" => Some(0.429),
        // small_rock.glb: mesh is center-anchored (0.247) but visually
        // acceptable without offset — adding it overshoots by ~4 units.
        //
        // Tripo flowers — origin at model center; trimesh reports 0.500 but
        // in-game testing shows 0.300 aligns the visual base correctly.
        "yellow_flower.glb" => Some(0.300),
        // Trellis trees — origin at model center
        "tree_oak.glb" => Some(0.500),
        "tree_fat.glb" => Some(0.447),
        "tree_pine.glb" => Some(0.500),
        "tree_default.glb" => Some(0.500),
        // Sanctuary cherry blossom — trimesh: y_min=-0.5, y_max=0.5, origin at center
        "tree_cherryblossom.glb" => Some(0.500),
        // Tripo city props — origin at model center
        "taxi.glb" => Some(0.241),
        "construction-cone.glb" => Some(0.500),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Public output types
// ---------------------------------------------------------------------------

/// All converted data for a single LDtk level, ready for the Phase 5 emitter.
pub struct ConvertedLevel {
    pub identifier: String,
    pub layers: Vec<ConvertedLayer>,
}

/// The converted contents of one layer within a level.
///
/// A single LDtk level produces one `ConvertedLayer` that aggregates every
/// piece of game-ready data extracted from the IntGrid and Entities layers.
pub struct ConvertedLayer {
    /// Grid width in tiles.
    pub cols: i32,
    /// Grid height in tiles.
    pub rows: i32,
    /// World-space X coordinate of the layer's left edge.
    pub origin_x: f32,
    /// World-space Y coordinate of the layer's bottom edge.
    pub origin_y: f32,
    /// Player spawn position `[x, y]` in world coordinates, if a Spawn entity exists.
    pub spawn: Option<[f32; 2]>,
    /// 2-D tile array indexed `tiles[game_row][col]`.
    ///
    /// `game_row = 0` is the **bottom** row (opposite of LDtk's top-down CSV).
    /// Values: 0 = air, 1 = solid, 2 = platform.
    pub tiles: Vec<Vec<u8>>,
    pub enemies: Vec<ConvertedEnemy>,
    /// Star collectible positions `[x, y, z]` in world coordinates (z = 1.0).
    pub stars: Vec<[f32; 3]>,
    /// HealthFood collectible positions `[x, y, z]` in world coordinates (z = 1.0).
    pub health_foods: Vec<[f32; 3]>,
    pub doors: Vec<ConvertedDoor>,
    /// Decorative prop entities placed visually in LDtk.
    pub props: Vec<ConvertedProp>,
    /// Point-light entities placed visually in LDtk.
    pub lights: Vec<ConvertedLight>,
    /// Column index of the Gate entity, if present.
    pub gate_col: Option<i32>,
    /// Identifier of the next level to load, extracted from the Exit entity.
    pub exit_next_level: Option<String>,
    /// Stars required to pass through the Gate, extracted from the Gate entity.
    pub stars_required: Option<i32>,
    /// Sublevel (L1) dark-background color as an sRGB `[r, g, b]` triple,
    /// sourced from the LDtk level's `bg_color` custom field.
    /// Only populated on L1 levels (Forest_Cave / Subdivision_Sewer / City_Subway);
    /// `None` on surface/rooftop levels where no dark background is drawn.
    pub bg_color: Option<[f32; 3]>,
    /// Sublevel (L1) emissive glow on/off flag, from the `glow_enabled` field.
    pub glow_enabled: Option<bool>,
    /// Sublevel (L1) emissive glow color as an sRGB `[r, g, b]` triple,
    /// from the `glow_color` field. Multiplied by `glow_intensity` in linear
    /// space at spawn time to reconstruct the final HDR glow value.
    pub glow_color: Option<[f32; 3]>,
    /// Sublevel (L1) emissive glow intensity multiplier (linear space),
    /// from the `glow_intensity` field. 0.0 disables the glow.
    pub glow_intensity: Option<f32>,
    /// Solar canopy on/off flag for the rooftop layer, from `canopy_enabled`.
    /// `None` on all levels except Subdivision_Rooftop.
    pub canopy_enabled: Option<bool>,
    /// Solar canopy panel bottom Y (world units), from `canopy_panel_bottom`.
    pub canopy_panel_bottom: Option<f32>,
    /// Solar canopy panel strip thickness (world units), from `canopy_panel_height`.
    pub canopy_panel_height: Option<f32>,
    /// Solar canopy opaque backdrop height above panel (world units), from `canopy_backdrop_height`.
    pub canopy_backdrop_height: Option<f32>,
    /// Solar canopy panel strip base color as sRGB `[r, g, b]` triple,
    /// parsed from `canopy_panel_color` hex field.
    pub canopy_panel_color: Option<[f32; 3]>,
    /// Solar canopy panel strip alpha (0..1), from `canopy_panel_alpha`.
    pub canopy_panel_alpha: Option<f32>,
    /// Solar canopy opaque backdrop base color as sRGB `[r, g, b]` triple,
    /// parsed from `canopy_backdrop_color` hex field.
    pub canopy_backdrop_color: Option<[f32; 3]>,
}

/// A converted enemy entity.
pub struct ConvertedEnemy {
    /// One of: "Dog", "Squirrel", "Snake", "Rat", "Possum".
    pub enemy_type: String,
    /// World-space X centre.
    pub x: f32,
    /// World-space Y centre.
    pub y: f32,
    /// Half-width of the enemy's patrol route in world units.
    pub patrol_range: f32,
    /// Maximum hit-points for this enemy.
    pub health: f32,
    /// Optional per-instance movement speed override.
    pub speed_override: Option<f32>,
}

/// A converted prop entity (decorative model placed via LDtk).
pub struct ConvertedProp {
    /// Asset path for the GLB model, e.g. "models/small_rock.glb".
    pub model_id: String,
    /// World-space X centre (tile centre, not surface).
    pub x: f32,
    /// World-space Y centre (tile centre, not surface).
    pub y: f32,
    /// Z depth in world space; negative values push the prop behind the action plane.
    pub z: f32,
    /// X scale applied to the model (1.0 = natural width).
    pub scale_x: f32,
    /// Y scale applied to the model (1.0 = natural height).
    pub scale_y: f32,
    /// Z (depth) scale applied to the model.
    pub scale_z: f32,
    /// Y-axis rotation in radians.
    pub rotation_y: f32,
    /// When true, this prop renders in front of the player (foreground layer).
    pub foreground: bool,
}

/// A converted point-light or spot-light entity placed via LDtk.
pub struct ConvertedLight {
    /// World-space X centre (tile centre).
    pub x: f32,
    /// World-space Y centre (tile centre).
    pub y: f32,
    /// Z depth in world space.
    pub z: f32,
    /// Linear RGB colour, each component in [0.0, 1.0].
    pub color: [f32; 3],
    /// Light intensity in lumens (or engine-relative units).
    pub intensity: f32,
    /// Radius of the light source sphere; omitted when not set.
    pub radius: Option<f32>,
    /// Maximum range in world units beyond which the light has no effect; omitted when not set.
    pub range: Option<f32>,
}

/// A converted door entity that links the current layer to another.
pub struct ConvertedDoor {
    /// Index of the sublevel/layer this door leads to.
    pub target_layer: i32,
    /// World-space X centre.
    pub x: f32,
    /// World-space Y centre.
    pub y: f32,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Converts every level in `root` into game-ready [`ConvertedLevel`] structs.
///
/// This function does **not** validate the input; run [`crate::validator::validate`]
/// before calling this if you need validation guarantees.
pub fn convert(root: &LdtkRoot) -> Vec<ConvertedLevel> {
    let raw: Vec<ConvertedLevel> = root.levels.iter().map(convert_level).collect();
    merge_sublevel_layers(raw)
}

/// Merges related LDtk levels into multi-layer game levels.
///
/// LDtk stores each sublevel as a separate level (e.g. "Forest_Cave"),
/// but the game expects a single "Forest" level with layers [0, 1, 2].
///
/// Layer assignment by suffix:
///   - No suffix → layer 0 (surface)
///   - `_Cave`, `_Sewer`, `_Subway` → layer 1 (underground)
///   - `_Rooftop` → layer 2
///
/// Levels whose names don't match any known suffix are passed through
/// unchanged (single-layer).
fn merge_sublevel_layers(levels: Vec<ConvertedLevel>) -> Vec<ConvertedLevel> {
    use std::collections::BTreeMap;

    // Map each level to (base_name, layer_index).
    let sublevel_suffixes: &[(&str, usize)] = &[
        ("_Cave", 1),
        ("_Sewer", 1),
        ("_Subway", 1),
        ("_Rooftop", 2),
    ];

    // Group levels by base name, tracking their layer index.
    // BTreeMap gives deterministic ordering (alphabetical by base name).
    let mut groups: BTreeMap<String, Vec<(usize, ConvertedLevel)>> = BTreeMap::new();

    for level in levels {
        let (base, layer_idx) = match sublevel_suffixes
            .iter()
            .find(|(suffix, _)| level.identifier.ends_with(suffix))
        {
            Some((suffix, idx)) => {
                let base = level.identifier[..level.identifier.len() - suffix.len()].to_string();
                (base, *idx)
            }
            None => (level.identifier.clone(), 0),
        };
        groups.entry(base).or_default().push((layer_idx, level));
    }

    // Build merged levels: sort each group's layers by index, extract the
    // single ConvertedLayer from each ConvertedLevel, and collect into one
    // multi-layer ConvertedLevel.
    groups
        .into_iter()
        .map(|(base_name, mut entries)| {
            entries.sort_by_key(|(idx, _)| *idx);
            let layers: Vec<ConvertedLayer> = entries
                .into_iter()
                .flat_map(|(_, level)| level.layers)
                .collect();
            ConvertedLevel {
                identifier: base_name,
                layers,
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Level conversion
// ---------------------------------------------------------------------------

fn convert_level(level: &crate::ldtk_schema::LdtkLevel) -> ConvertedLevel {
    // Extract world-space origin from level field instances, using defaults when absent.
    let origin_x = get_level_field_f32(level, "OriginX").unwrap_or(DEFAULT_ORIGIN_X);
    let origin_y = get_level_field_f32(level, "OriginY").unwrap_or(DEFAULT_ORIGIN_Y);

    // Sublevel (L1) dark-background + emissive glow fields.
    // These are `None` for surface/rooftop LDtk levels that don't set them;
    // after merge_sublevel_layers runs, the values end up on layer 1 of the
    // merged output (matching where they were authored in LDtk).
    let bg_color = get_level_field_color(level, "bg_color");
    let glow_enabled = get_level_field_bool(level, "glow_enabled");
    let glow_color = get_level_field_color(level, "glow_color");
    let glow_intensity = get_level_field_f32(level, "glow_intensity");

    // Solar canopy fields (L2 rooftop only; absent / None on all other levels).
    let canopy_enabled = get_level_field_bool(level, "canopy_enabled");
    let canopy_panel_bottom = get_level_field_f32(level, "canopy_panel_bottom");
    let canopy_panel_height = get_level_field_f32(level, "canopy_panel_height");
    let canopy_backdrop_height = get_level_field_f32(level, "canopy_backdrop_height");
    let canopy_panel_color = get_level_field_color(level, "canopy_panel_color");
    let canopy_panel_alpha = get_level_field_f32(level, "canopy_panel_alpha");
    let canopy_backdrop_color = get_level_field_color(level, "canopy_backdrop_color");

    let layers = match &level.layer_instances {
        Some(l) => l,
        None => {
            // Validator should have caught this; return an empty layer set.
            return ConvertedLevel {
                identifier: level.identifier.clone(),
                layers: vec![],
            };
        }
    };

    // Locate the first IntGrid layer and the first Entities layer.
    let intgrid_layer = layers.iter().find(|l| l.layer_type == "IntGrid");
    let entities_layer = layers.iter().find(|l| l.layer_type == "Entities");

    let (cols, rows, tiles) = match intgrid_layer {
        Some(layer) => (layer.c_wid, layer.c_hei, convert_tiles(layer)),
        None => (0, 0, vec![]),
    };

    // Use the Entities layer dimensions for entity-pixel conversion; fall back
    // to the IntGrid dimensions when no Entities layer exists (both should match
    // after validation).
    let c_hei_for_entities = entities_layer.map(|l| l.c_hei).unwrap_or(rows);

    let mut spawn: Option<[f32; 2]> = None;
    let mut enemies: Vec<ConvertedEnemy> = vec![];
    let mut stars: Vec<[f32; 3]> = vec![];
    let mut health_foods: Vec<[f32; 3]> = vec![];
    let mut doors: Vec<ConvertedDoor> = vec![];
    let mut props: Vec<ConvertedProp> = vec![];
    let mut lights: Vec<ConvertedLight> = vec![];
    let mut gate_col: Option<i32> = None;
    let mut exit_next_level: Option<String> = None;
    let mut stars_required: Option<i32> = None;

    if let Some(ent_layer) = entities_layer {
        for entity in &ent_layer.entity_instances {
            match entity.identifier.as_str() {
                "Spawn" => {
                    // Spawn stands on a surface; use ground_top so the player
                    // is placed at the correct height above the tile.
                    let (wx, wy) = px_to_world_surface(
                        entity.px,
                        c_hei_for_entities,
                        origin_x,
                        origin_y,
                    );
                    spawn = Some([wx, wy]);
                }
                "Enemy" => {
                    // Enemy stands on a surface; the spawner adds COLLIDER_H/2
                    // internally and expects ground_top, not tile centre.
                    let (wx, wy) = px_to_world_surface(
                        entity.px,
                        c_hei_for_entities,
                        origin_x,
                        origin_y,
                    );
                    // Required fields — default to safe values when absent so the
                    // converter can still produce output even for partially-valid data.
                    let enemy_type =
                        get_field_str(entity, "enemy_type").unwrap_or_else(|| "Dog".to_string());
                    let patrol_range = get_field_f32(entity, "patrol_range").unwrap_or(0.0);
                    let health = get_field_f32(entity, "health").unwrap_or(1.0);
                    let speed_override = get_field_f32(entity, "speed_override");

                    enemies.push(ConvertedEnemy {
                        enemy_type,
                        x: wx,
                        y: wy,
                        patrol_range,
                        health,
                        speed_override,
                    });
                }
                "Star" => {
                    // Stars float; tile-centre Y is correct.
                    let (wx, wy) =
                        px_to_world(entity.px, c_hei_for_entities, origin_x, origin_y);
                    // Z = 1.0 places collectibles in front of the background plane.
                    stars.push([wx, wy, 1.0]);
                }
                "HealthFood" => {
                    // HealthFood floats; tile-centre Y is correct.
                    let (wx, wy) =
                        px_to_world(entity.px, c_hei_for_entities, origin_x, origin_y);
                    // Z = 1.0 places collectibles in front of the background plane.
                    health_foods.push([wx, wy, 1.0]);
                }
                "Door" => {
                    // Door stands on a surface; use ground_top for consistent
                    // placement with the player and enemies.
                    let (wx, wy) = px_to_world_surface(
                        entity.px,
                        c_hei_for_entities,
                        origin_x,
                        origin_y,
                    );
                    let target_layer = get_field_i32(entity, "target_layer").unwrap_or(0);
                    doors.push(ConvertedDoor {
                        target_layer,
                        x: wx,
                        y: wy,
                    });
                }
                id if id == "Prop" || id.starts_with("Prop_") => {
                    // Props can be anywhere — use tile centre (not ground_top).
                    let (wx, wy) =
                        px_to_world(entity.px, c_hei_for_entities, origin_x, origin_y);
                    // Required field — default to empty string so the converter
                    // still emits output even for partially-valid data.
                    let model_id = get_field_str(entity, "model_id")
                        .unwrap_or_default();
                    // Optional visual tuning fields with documented gameplay-safe defaults.
                    // scale_x = 1.0 means no X scaling (natural model width).
                    let scale_x = get_field_f32(entity, "scale_x").unwrap_or(1.0);
                    // scale_y = 1.0 means no Y scaling (natural model height).
                    let scale_y = get_field_f32(entity, "scale_y").unwrap_or(1.0);
                    // Center-anchored models need a Y offset so the visual bottom
                    // aligns with the LDtk placement (editor shows bottom, but the
                    // 3D model origin is at its center).
                    let wy = match center_anchor_half_height(&model_id) {
                        Some(half_h) => wy + half_h * scale_y,
                        None => wy,
                    };
                    // scale_z = 1.0 means no depth scaling.
                    let scale_z = get_field_f32(entity, "scale_z").unwrap_or(1.0);
                    // z_depth = -15.0 places props behind the action plane but
                    // in front of the parallax background.
                    let z = get_field_f32(entity, "z_depth").unwrap_or(-15.0);
                    // rotation_y = 0.0 means facing the camera (no rotation).
                    let rotation_y = get_field_f32(entity, "rotation_y").unwrap_or(0.0);
                    // foreground = false means the prop renders behind the player.
                    // When true, the prop renders in front of the player.
                    let foreground = get_field_bool(entity, "foreground").unwrap_or(false);
                    props.push(ConvertedProp {
                        model_id,
                        x: wx,
                        y: wy,
                        z,
                        scale_x,
                        scale_y,
                        scale_z,
                        rotation_y,
                        foreground,
                    });
                }
                "Light" => {
                    // Lights use tile centre position, same as Props.
                    let (wx, wy) =
                        px_to_world(entity.px, c_hei_for_entities, origin_x, origin_y);
                    // z_depth = 3.0 places lights in front of the action plane by default.
                    let z = get_field_f32(entity, "z_depth").unwrap_or(3.0);
                    // intensity defaults to 100000.0 (engine-relative lumens).
                    let intensity = get_field_f32(entity, "intensity").unwrap_or(100000.0);
                    // color is a hex string e.g. "#FFFFFF"; default to white.
                    let color_hex = get_field_str(entity, "color")
                        .unwrap_or_else(|| "#FFFFFF".to_string());
                    let color = parse_hex_color(&color_hex);
                    let radius = get_field_f32(entity, "radius");
                    let range = get_field_f32(entity, "range");
                    lights.push(ConvertedLight {
                        x: wx,
                        y: wy,
                        z,
                        color,
                        intensity,
                        radius,
                        range,
                    });
                }
                "Gate" => {
                    // gate_col is the column index of the gate tile, not the world position.
                    gate_col = get_field_i32(entity, "gate_col");
                    // stars_required controls how many stars the player must collect.
                    stars_required = get_field_i32(entity, "stars_required");
                }
                "Exit" => {
                    exit_next_level = get_field_str(entity, "exit_next_level");
                }
                // Unknown entity types are silently ignored here; the validator
                // already surfaced them as errors before conversion runs.
                _ => {}
            }
        }
    }

    let layer = ConvertedLayer {
        cols,
        rows,
        origin_x,
        origin_y,
        spawn,
        tiles,
        enemies,
        stars,
        health_foods,
        doors,
        props,
        lights,
        gate_col,
        exit_next_level,
        stars_required,
        bg_color,
        glow_enabled,
        glow_color,
        glow_intensity,
        canopy_enabled,
        canopy_panel_bottom,
        canopy_panel_height,
        canopy_backdrop_height,
        canopy_panel_color,
        canopy_panel_alpha,
        canopy_backdrop_color,
    };

    ConvertedLevel {
        identifier: level.identifier.clone(),
        layers: vec![layer],
    }
}

// ---------------------------------------------------------------------------
// Tile conversion — IntGrid CSV → row-major 2D array with Y-flip
// ---------------------------------------------------------------------------

/// Converts the flat IntGrid CSV from an LDtk layer into a 2-D `Vec<Vec<u8>>`
/// with `tiles[game_row][col]` where `game_row = 0` is the **bottom** row.
///
/// LDtk stores rows top-to-bottom (row 0 = top in LDtk editor).
/// The game uses row 0 = bottom, so every row index is flipped:
/// `game_row = (total_rows - 1) - ldtk_row`.
fn convert_tiles(layer: &LdtkLayerInstance) -> Vec<Vec<u8>> {
    let cols = layer.c_wid as usize;
    let rows = layer.c_hei as usize;
    let mut tiles = vec![vec![0u8; cols]; rows];

    // Iterate over every (ldtk_row, col) pair via the flat CSV index so that
    // clippy's needless_range_loop lint does not fire — we genuinely need both
    // coordinates to index two different arrays.
    for (csv_index, &raw_value) in layer.int_grid_csv.iter().enumerate() {
        let ldtk_row = csv_index / cols;
        let col = csv_index % cols;
        // Y-flip: row 0 in LDtk is the top; row 0 in the game is the bottom.
        let game_row = (rows - 1) - ldtk_row;
        tiles[game_row][col] = raw_value as u8;
    }

    tiles
}

// ---------------------------------------------------------------------------
// Coordinate conversion helpers
// ---------------------------------------------------------------------------

/// Converts an LDtk entity pixel position to game-world coordinates.
///
/// All LDtk entities use **bottom-center pivot (0.5, 1.0)**:
/// - `px[0]` is the horizontal **center** of the entity (already includes any
///   half-tile offset for grid-snapped entities — do NOT add TILE_SIZE/2 again).
/// - `px[1]` is the **bottom** of the entity (sits on a grid line).
///
/// LDtk pixel coordinates have the origin at the **top-left** of the level;
/// the game has origin at the **bottom-left**, so a Y-flip is applied using the
/// full level height in pixels (`c_hei * TILE_SIZE`).
///
/// ```text
/// level_h  = c_hei * TILE_SIZE
/// world_x  = origin_x + px[0]              (px[0] is already the centre X)
/// world_y  = origin_y + (level_h - px[1])  (Y-flip; px[1] is entity bottom)
/// ```
fn px_to_world(px: [i32; 2], c_hei: i32, origin_x: f32, origin_y: f32) -> (f32, f32) {
    let level_h = c_hei as f32 * TILE_SIZE;
    // px[0] is the horizontal center — no half-tile offset needed.
    let world_x = origin_x + px[0] as f32;
    // Y-flip: LDtk Y=0 is the top, game Y=0 is the bottom.
    // px[1] is the bottom edge of the entity.
    let world_y = origin_y + (level_h - px[1] as f32);
    (world_x, world_y)
}

/// Converts an LDtk entity pixel position to the **ground surface** Y in
/// game-world coordinates.
///
/// All LDtk entities use **bottom-center pivot (0.5, 1.0)**:
/// - `px[0]` is the horizontal **center** of the entity.
/// - `px[1]` is the **bottom** of the entity, which for ground-placed entities
///   (Enemy, Spawn, Door) IS the ground surface — no extra TILE_SIZE offset needed.
///
/// With bottom-center pivot the "surface" and "centre" distinction collapses:
/// `px[1]` always points to the bottom of the entity regardless of its height,
/// so this function applies the same Y-flip formula as [`px_to_world`].
///
/// The enemy spawner adds `COLLIDER_H/2` internally and expects the ground
/// surface coordinate — this function provides exactly that.
///
/// ```text
/// level_h  = c_hei * TILE_SIZE
/// world_x  = origin_x + px[0]              (px[0] is already the centre X)
/// world_y  = origin_y + (level_h - px[1])  (Y-flip; px[1] IS the ground surface)
/// ```
fn px_to_world_surface(px: [i32; 2], c_hei: i32, origin_x: f32, origin_y: f32) -> (f32, f32) {
    let level_h = c_hei as f32 * TILE_SIZE;
    // px[0] is the horizontal center — no half-tile offset needed.
    let world_x = origin_x + px[0] as f32;
    // Y-flip: LDtk Y=0 is the top, game Y=0 is the bottom.
    // With bottom-center pivot, px[1] IS the ground surface — no extra tile offset.
    let world_y = origin_y + (level_h - px[1] as f32);
    (world_x, world_y)
}

// ---------------------------------------------------------------------------
// Level field helpers
// ---------------------------------------------------------------------------

fn get_level_field_f32(level: &crate::ldtk_schema::LdtkLevel, name: &str) -> Option<f32> {
    level
        .field_instances
        .iter()
        .find(|f| f.identifier == name)
        .and_then(|f| f.value.as_ref())
        .and_then(|v| v.as_f64())
        .map(|n| n as f32)
}

/// Returns the bool value of a named custom field from a Level, or `None`
/// when the field is absent, null, or not a JSON boolean.
fn get_level_field_bool(level: &crate::ldtk_schema::LdtkLevel, name: &str) -> Option<bool> {
    level
        .field_instances
        .iter()
        .find(|f| f.identifier == name)
        .and_then(|f| f.value.as_ref())
        .and_then(|v| v.as_bool())
}

/// Returns the sRGB `[r, g, b]` value of a named Color custom field from a
/// Level, parsed from LDtk's hex string format (e.g. `"#1F1A12"`).
/// Returns `None` when the field is absent, null, or not a JSON string.
fn get_level_field_color(level: &crate::ldtk_schema::LdtkLevel, name: &str) -> Option<[f32; 3]> {
    level
        .field_instances
        .iter()
        .find(|f| f.identifier == name)
        .and_then(|f| f.value.as_ref())
        .and_then(|v| v.as_str())
        .map(parse_hex_color)
}

// ---------------------------------------------------------------------------
// Entity field extraction helpers
// ---------------------------------------------------------------------------

/// Returns the string value of a named custom field from an entity, or `None`
/// when the field is absent, null, or not a JSON string.
fn get_field_str(entity: &LdtkEntityInstance, name: &str) -> Option<String> {
    entity
        .field_instances
        .iter()
        .find(|f| f.identifier == name)
        .and_then(|f| f.value.as_ref())
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// Returns the f32 value of a named custom field from an entity, or `None`
/// when the field is absent, null, or not a JSON number.
fn get_field_f32(entity: &LdtkEntityInstance, name: &str) -> Option<f32> {
    entity
        .field_instances
        .iter()
        .find(|f| f.identifier == name)
        .and_then(|f| f.value.as_ref())
        .and_then(|v| v.as_f64())
        .map(|n| n as f32)
}

/// Returns the i32 value of a named custom field from an entity, or `None`
/// when the field is absent, null, or not a JSON integer.
fn get_field_i32(entity: &LdtkEntityInstance, name: &str) -> Option<i32> {
    entity
        .field_instances
        .iter()
        .find(|f| f.identifier == name)
        .and_then(|f| f.value.as_ref())
        .and_then(|v| v.as_i64())
        .map(|n| n as i32)
}

/// Returns the bool value of a named custom field from an entity, or `None`
/// when the field is absent, null, or not a JSON boolean.
fn get_field_bool(entity: &LdtkEntityInstance, name: &str) -> Option<bool> {
    entity
        .field_instances
        .iter()
        .find(|f| f.identifier == name)
        .and_then(|f| f.value.as_ref())
        .and_then(|v| v.as_bool())
}

/// Parses a CSS hex color string (e.g. `"#FF8040"` or `"FF8040"`) into a
/// linear `[r, g, b]` array with each component in `[0.0, 1.0]`.
///
/// Returns `[1.0, 1.0, 1.0]` (white) when the string is malformed or too short.
fn parse_hex_color(hex: &str) -> [f32; 3] {
    let hex = hex.trim_start_matches('#');
    if hex.len() >= 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255) as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255) as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255) as f32 / 255.0;
        [r, g, b]
    } else {
        [1.0, 1.0, 1.0]
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ldtk_schema::{LdtkFieldInstance, LdtkLayerInstance, LdtkRoot};

    // ------------------------------------------------------------------
    // Helpers for building test fixtures
    // ------------------------------------------------------------------

    fn make_intgrid_layer(c_wid: i32, c_hei: i32, csv: Vec<i32>) -> LdtkLayerInstance {
        LdtkLayerInstance {
            identifier: "Collision".to_string(),
            layer_type: "IntGrid".to_string(),
            grid_size: 18,
            c_wid,
            c_hei,
            int_grid_csv: csv,
            entity_instances: vec![],
        }
    }

    fn make_entities_layer(
        c_wid: i32,
        c_hei: i32,
        entities: Vec<crate::ldtk_schema::LdtkEntityInstance>,
    ) -> LdtkLayerInstance {
        LdtkLayerInstance {
            identifier: "Entities".to_string(),
            layer_type: "Entities".to_string(),
            grid_size: 18,
            c_wid,
            c_hei,
            int_grid_csv: vec![],
            entity_instances: entities,
        }
    }

    fn make_entity(
        id: &str,
        px: [i32; 2],
        fields: Vec<LdtkFieldInstance>,
    ) -> crate::ldtk_schema::LdtkEntityInstance {
        crate::ldtk_schema::LdtkEntityInstance {
            identifier: id.to_string(),
            px,
            field_instances: fields,
        }
    }

    fn field_str(id: &str, val: &str) -> LdtkFieldInstance {
        LdtkFieldInstance {
            identifier: id.to_string(),
            value: Some(serde_json::Value::String(val.to_string())),
        }
    }

    fn field_f64(id: &str, val: f64) -> LdtkFieldInstance {
        LdtkFieldInstance {
            identifier: id.to_string(),
            value: Some(serde_json::json!(val)),
        }
    }

    fn field_i64(id: &str, val: i64) -> LdtkFieldInstance {
        LdtkFieldInstance {
            identifier: id.to_string(),
            value: Some(serde_json::json!(val)),
        }
    }

    fn minimal_root_with_layers(
        intgrid: LdtkLayerInstance,
        entities: LdtkLayerInstance,
        level_fields: Vec<LdtkFieldInstance>,
    ) -> LdtkRoot {
        LdtkRoot {
            levels: vec![crate::ldtk_schema::LdtkLevel {
                identifier: "TestLevel".to_string(),
                layer_instances: Some(vec![intgrid, entities]),
                field_instances: level_fields,
            }],
        }
    }

    // ------------------------------------------------------------------
    // convert_tiles_flips_rows
    // ------------------------------------------------------------------

    /// A 3×3 IntGrid where each LDtk row has a unique value (1/2/3 top→bottom)
    /// should appear in reverse order after Y-flip (3/2/1 bottom→top).
    ///
    /// LDtk CSV (row-major, top-to-bottom):
    ///   row 0 (top):    [1, 1, 1]
    ///   row 1 (middle): [2, 2, 2]
    ///   row 2 (bottom): [3, 3, 3]
    ///
    /// Expected game tiles (row 0 = bottom):
    ///   game_row 0: [3, 3, 3]   ← was LDtk row 2
    ///   game_row 1: [2, 2, 2]   ← was LDtk row 1
    ///   game_row 2: [1, 1, 1]   ← was LDtk row 0
    #[test]
    fn convert_tiles_flips_rows() {
        let layer = make_intgrid_layer(
            3,
            3,
            vec![
                1, 1, 1, // LDtk row 0 (top)
                2, 2, 2, // LDtk row 1
                3, 3, 3, // LDtk row 2 (bottom)
            ],
        );
        let tiles = convert_tiles(&layer);

        assert_eq!(tiles.len(), 3, "should have 3 rows");
        assert_eq!(tiles[0], vec![3, 3, 3], "game_row 0 should be LDtk row 2");
        assert_eq!(tiles[1], vec![2, 2, 2], "game_row 1 should be LDtk row 1");
        assert_eq!(tiles[2], vec![1, 1, 1], "game_row 2 should be LDtk row 0");
    }

    // ------------------------------------------------------------------
    // px_to_world_converts_correctly
    // ------------------------------------------------------------------

    /// With origin (0, 0), a 10×10 grid, and px = [18, 18].
    /// Bottom-center pivot: px[0]=18 is the horizontal center, px[1]=18 is the
    /// entity bottom.
    ///
    ///   level_h  = 10 * 18 = 180
    ///   world_x  = 0 + 18 = 18
    ///   world_y  = 0 + (180 - 18) = 162
    #[test]
    fn px_to_world_converts_correctly() {
        let (wx, wy) = px_to_world([18, 18], 10, 0.0, 0.0);
        assert!(
            (wx - 18.0).abs() < 1e-4,
            "world_x should be 18.0, got {wx}"
        );
        assert!(
            (wy - 162.0).abs() < 1e-4,
            "world_y should be 162.0, got {wy}"
        );
    }

    // ------------------------------------------------------------------
    // convert_level_extracts_enemies
    // ------------------------------------------------------------------

    #[test]
    fn convert_level_extracts_enemies() {
        // 2×2 grid, enemy at px [0, 0] (bottom-center pivot: x=0 is centre, y=0 is bottom).
        // c_hei = 2, level_h = 2*18 = 36.
        // world_x = -864 + 0 = -864
        // world_y = -200 + (36 - 0) = -200 + 36 = -164  (ground surface)
        let intgrid = make_intgrid_layer(2, 2, vec![0, 0, 1, 1]);
        let enemy_entity = make_entity(
            "Enemy",
            [0, 0],
            vec![
                field_str("enemy_type", "Dog"),
                field_f64("patrol_range", 72.0),
                field_f64("health", 3.0),
                field_f64("speed_override", 50.0),
            ],
        );
        let entities = make_entities_layer(2, 2, vec![enemy_entity]);
        let root = minimal_root_with_layers(intgrid, entities, vec![]);

        let levels = convert(&root);
        assert_eq!(levels.len(), 1);
        let layer = &levels[0].layers[0];
        assert_eq!(layer.enemies.len(), 1);

        let enemy = &layer.enemies[0];
        assert_eq!(enemy.enemy_type, "Dog");
        assert!((enemy.patrol_range - 72.0).abs() < 1e-4);
        assert!((enemy.health - 3.0).abs() < 1e-4);
        assert_eq!(enemy.speed_override, Some(50.0));

        // Verify world position (ground surface via px_to_world_surface, bottom-center pivot).
        assert!(
            (enemy.x - (-864.0)).abs() < 1e-4,
            "expected x=-864, got {}",
            enemy.x
        );
        assert!(
            (enemy.y - (-164.0)).abs() < 1e-4,
            "expected y=-164 (ground surface), got {}",
            enemy.y
        );
    }

    // ------------------------------------------------------------------
    // convert_level_extracts_collectibles
    // ------------------------------------------------------------------

    #[test]
    fn convert_level_extracts_collectibles() {
        // 4×4 grid. Star at px [18, 0], HealthFood at px [36, 18].
        // Bottom-center pivot: px[0] is centre X, px[1] is entity bottom.
        // level_h = 4 * 18 = 72.
        //
        // Star: px=[18, 0]
        //   world_x = -864 + 18 = -846
        //   world_y = -200 + (72 - 0) = -200 + 72 = -128
        //
        // HealthFood: px=[36, 18]
        //   world_x = -864 + 36 = -828
        //   world_y = -200 + (72 - 18) = -200 + 54 = -146
        let intgrid = make_intgrid_layer(4, 4, vec![0; 16]);
        let entities = make_entities_layer(
            4,
            4,
            vec![
                make_entity("Star", [18, 0], vec![]),
                make_entity("HealthFood", [36, 18], vec![]),
            ],
        );
        let root = minimal_root_with_layers(intgrid, entities, vec![]);

        let levels = convert(&root);
        let layer = &levels[0].layers[0];

        assert_eq!(layer.stars.len(), 1, "should have one star");
        assert_eq!(layer.health_foods.len(), 1, "should have one health food");

        let star = layer.stars[0];
        assert!((star[0] - (-846.0)).abs() < 1e-4, "star x got {}", star[0]);
        assert!((star[1] - (-128.0)).abs() < 1e-4, "star y got {}", star[1]);
        assert!((star[2] - 1.0).abs() < 1e-4, "star z should be 1.0");

        let hf = layer.health_foods[0];
        assert!((hf[0] - (-828.0)).abs() < 1e-4, "hf x got {}", hf[0]);
        assert!((hf[1] - (-146.0)).abs() < 1e-4, "hf y got {}", hf[1]);
        assert!((hf[2] - 1.0).abs() < 1e-4, "hf z should be 1.0");
    }

    // ------------------------------------------------------------------
    // convert_level_extracts_doors
    // ------------------------------------------------------------------

    #[test]
    fn convert_level_extracts_doors() {
        // 3×3 grid. Door at px [0, 36].
        // Bottom-center pivot: px[0]=0 is centre X, px[1]=36 is entity bottom (ground surface).
        // level_h = 3 * 18 = 54.
        //   world_x = -864 + 0 = -864
        //   world_y = -200 + (54 - 36) = -200 + 18 = -182
        let intgrid = make_intgrid_layer(3, 3, vec![0; 9]);
        let door_entity =
            make_entity("Door", [0, 36], vec![field_i64("target_layer", 2)]);
        let entities = make_entities_layer(3, 3, vec![door_entity]);
        let root = minimal_root_with_layers(intgrid, entities, vec![]);

        let levels = convert(&root);
        let layer = &levels[0].layers[0];

        assert_eq!(layer.doors.len(), 1);
        let door = &layer.doors[0];
        assert_eq!(door.target_layer, 2);
        assert!((door.x - (-864.0)).abs() < 1e-4, "door x got {}", door.x);
        assert!(
            (door.y - (-182.0)).abs() < 1e-4,
            "expected y=-182 (ground surface), got {}",
            door.y
        );
    }

    // ------------------------------------------------------------------
    // convert_level_extracts_spawn
    // ------------------------------------------------------------------

    #[test]
    fn convert_level_extracts_spawn() {
        // 5×5 grid. Spawn at px [36, 54].
        // Bottom-center pivot: px[0]=36 is centre X, px[1]=54 is entity bottom (ground surface).
        // level_h = 5 * 18 = 90.
        //   world_x = -864 + 36 = -828
        //   world_y = -200 + (90 - 54) = -200 + 36 = -164
        let intgrid = make_intgrid_layer(5, 5, vec![0; 25]);
        let spawn_entity = make_entity("Spawn", [36, 54], vec![]);
        let entities = make_entities_layer(5, 5, vec![spawn_entity]);
        let root = minimal_root_with_layers(intgrid, entities, vec![]);

        let levels = convert(&root);
        let layer = &levels[0].layers[0];

        let spawn = layer.spawn.expect("spawn should be set");
        assert!((spawn[0] - (-828.0)).abs() < 1e-4, "spawn x got {}", spawn[0]);
        assert!(
            (spawn[1] - (-164.0)).abs() < 1e-4,
            "expected y=-164 (ground surface), got {}",
            spawn[1]
        );
    }

    // ------------------------------------------------------------------
    // convert_level_default_origin
    // ------------------------------------------------------------------

    /// When no OriginX / OriginY fields are present on the level, the converter
    /// must fall back to the canonical defaults (-864, -200).
    #[test]
    fn convert_level_default_origin() {
        let intgrid = make_intgrid_layer(2, 2, vec![0; 4]);
        let entities = make_entities_layer(2, 2, vec![]);
        // Pass empty field_instances so no OriginX/OriginY fields exist.
        let root = minimal_root_with_layers(intgrid, entities, vec![]);

        let levels = convert(&root);
        let layer = &levels[0].layers[0];

        assert!(
            (layer.origin_x - DEFAULT_ORIGIN_X).abs() < 1e-4,
            "expected origin_x = {DEFAULT_ORIGIN_X}, got {}",
            layer.origin_x
        );
        assert!(
            (layer.origin_y - DEFAULT_ORIGIN_Y).abs() < 1e-4,
            "expected origin_y = {DEFAULT_ORIGIN_Y}, got {}",
            layer.origin_y
        );
    }

    // ------------------------------------------------------------------
    // convert_level_respects_explicit_origin
    // ------------------------------------------------------------------

    /// When OriginX / OriginY are provided, they must be used instead of the defaults.
    #[test]
    fn convert_level_respects_explicit_origin() {
        let intgrid = make_intgrid_layer(2, 2, vec![0; 4]);
        let entities = make_entities_layer(2, 2, vec![]);
        let level_fields = vec![
            LdtkFieldInstance {
                identifier: "OriginX".to_string(),
                value: Some(serde_json::json!(-500.0f64)),
            },
            LdtkFieldInstance {
                identifier: "OriginY".to_string(),
                value: Some(serde_json::json!(100.0f64)),
            },
        ];
        let root = minimal_root_with_layers(intgrid, entities, level_fields);

        let levels = convert(&root);
        let layer = &levels[0].layers[0];

        assert!(
            (layer.origin_x - (-500.0)).abs() < 1e-4,
            "expected origin_x=-500, got {}",
            layer.origin_x
        );
        assert!(
            (layer.origin_y - 100.0).abs() < 1e-4,
            "expected origin_y=100, got {}",
            layer.origin_y
        );
    }

    // ------------------------------------------------------------------
    // center_anchored_prop_y_offset
    // ------------------------------------------------------------------

    /// Props whose model is center-anchored (e.g. large_rock.glb) must have
    /// their Y shifted up by `half_height * scale_y` so the visual bottom
    /// aligns with the LDtk editor placement.
    #[test]
    fn center_anchored_prop_y_offset() {
        // 4×4 grid, default origin (-864, -200), level_h = 72.
        // Prop at px [36, 36] with model_id = "models/large_rock.glb".
        // Base world_y = -200 + (72 - 36) = -164.
        // large_rock half_height = 0.429, scale_y = 28.0.
        // Adjusted world_y = -164 + 0.429 * 28.0 = -164 + 12.012 = -151.988.
        let intgrid = make_intgrid_layer(4, 4, vec![0; 16]);
        let prop_entity = make_entity(
            "Prop",
            [36, 36],
            vec![
                field_str("model_id", "models/large_rock.glb"),
                field_f64("scale_x", 9.0),
                field_f64("scale_y", 28.0),
                field_f64("scale_z", 28.0),
                field_f64("rotation_y", -1.5707963),
            ],
        );
        let entities = make_entities_layer(4, 4, vec![prop_entity]);
        let root = minimal_root_with_layers(intgrid, entities, vec![]);

        let levels = convert(&root);
        let layer = &levels[0].layers[0];

        assert_eq!(layer.props.len(), 1);
        let prop = &layer.props[0];

        // Base Y without offset would be -164.0
        let base_y = -164.0_f32;
        let expected_y = base_y + 0.429 * 28.0;
        assert!(
            (prop.y - expected_y).abs() < 1e-2,
            "expected y={expected_y:.3}, got {:.3} (center-anchor offset should apply)",
            prop.y
        );
    }

    // ------------------------------------------------------------------
    // non_center_anchored_prop_no_offset
    // ------------------------------------------------------------------

    /// Props with bottom-anchored models (e.g. plant_bush.glb) should NOT
    /// get any Y offset.
    #[test]
    fn non_center_anchored_prop_no_offset() {
        let intgrid = make_intgrid_layer(4, 4, vec![0; 16]);
        let prop_entity = make_entity(
            "Prop",
            [36, 36],
            vec![
                field_str("model_id", "models/plant_bush.glb"),
                field_f64("scale_x", 49.0),
                field_f64("scale_y", 49.0),
                field_f64("scale_z", 27.0),
            ],
        );
        let entities = make_entities_layer(4, 4, vec![prop_entity]);
        let root = minimal_root_with_layers(intgrid, entities, vec![]);

        let levels = convert(&root);
        let layer = &levels[0].layers[0];

        assert_eq!(layer.props.len(), 1);
        let prop = &layer.props[0];

        // No offset: world_y = -200 + (72 - 36) = -164
        assert!(
            (prop.y - (-164.0)).abs() < 1e-2,
            "expected y=-164.0 (no offset), got {:.3}",
            prop.y
        );
    }
}
