use crate::ldtk_schema::{LdtkEntityInstance, LdtkLayerInstance, LdtkRoot};

/// Game-space tile size in world units.
const TILE_SIZE: f32 = 18.0;

/// Default world-space origin X applied when the level has no OriginX field.
const DEFAULT_ORIGIN_X: f32 = -864.0;

/// Default world-space origin Y applied when the level has no OriginY field.
const DEFAULT_ORIGIN_Y: f32 = -200.0;

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
    /// Column index of the Gate entity, if present.
    pub gate_col: Option<i32>,
    /// Identifier of the next level to load, extracted from the Exit entity.
    pub exit_next_level: Option<String>,
    /// Stars required to pass through the Gate, extracted from the Gate entity.
    pub stars_required: Option<i32>,
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
    root.levels.iter().map(convert_level).collect()
}

// ---------------------------------------------------------------------------
// Level conversion
// ---------------------------------------------------------------------------

fn convert_level(level: &crate::ldtk_schema::LdtkLevel) -> ConvertedLevel {
    // Extract world-space origin from level field instances, using defaults when absent.
    let origin_x = get_level_field_f32(level, "OriginX").unwrap_or(DEFAULT_ORIGIN_X);
    let origin_y = get_level_field_f32(level, "OriginY").unwrap_or(DEFAULT_ORIGIN_Y);

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
        gate_col,
        exit_next_level,
        stars_required,
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

/// Converts an LDtk entity pixel position to game-world coordinates (tile centre).
///
/// LDtk pixel coordinates have the origin at the **top-left** of the level.
/// The game has origin at the **bottom-left**, so the row index is flipped.
///
/// Use this for entities that float or are not position-sensitive with respect
/// to a ground surface (e.g. Star, HealthFood, Gate, Exit).
///
/// ```text
/// col        = px[0] / TILE_SIZE          (fractional col)
/// ldtk_row   = px[1] / TILE_SIZE          (fractional row, 0 = top)
/// game_row   = (c_hei - 1) - ldtk_row    (flipped, 0 = bottom)
/// world_x    = origin_x + col * TILE_SIZE + TILE_SIZE/2   (tile centre)
/// world_y    = origin_y + game_row * TILE_SIZE + TILE_SIZE/2
/// ```
fn px_to_world(px: [i32; 2], c_hei: i32, origin_x: f32, origin_y: f32) -> (f32, f32) {
    let col = px[0] as f32 / TILE_SIZE;
    let ldtk_row = px[1] as f32 / TILE_SIZE;
    // Flip from LDtk top-down to game bottom-up.
    let game_row = (c_hei as f32 - 1.0) - ldtk_row;
    let world_x = origin_x + col * TILE_SIZE + TILE_SIZE / 2.0;
    let world_y = origin_y + game_row * TILE_SIZE + TILE_SIZE / 2.0;
    (world_x, world_y)
}

/// Converts an LDtk entity pixel position to the **ground-top** surface Y in
/// game-world coordinates.
///
/// Use this for entities that stand on the ground surface (Enemy, Spawn, Door).
/// The enemy spawner adds `COLLIDER_H/2` internally and therefore expects
/// `ground_top` — the top edge of the tile the entity occupies — rather than
/// the tile centre returned by [`px_to_world`].
///
/// ```text
/// col        = px[0] / TILE_SIZE
/// ldtk_row   = px[1] / TILE_SIZE
/// game_row   = (c_hei - 1) - ldtk_row
/// world_x    = origin_x + col * TILE_SIZE + TILE_SIZE/2   (tile centre X)
/// world_y    = origin_y + (game_row + 1) * TILE_SIZE      (top surface of tile)
/// ```
fn px_to_world_surface(px: [i32; 2], c_hei: i32, origin_x: f32, origin_y: f32) -> (f32, f32) {
    let col = px[0] as f32 / TILE_SIZE;
    let ldtk_row = px[1] as f32 / TILE_SIZE;
    // Flip from LDtk top-down to game bottom-up.
    let game_row = (c_hei as f32 - 1.0) - ldtk_row;
    let world_x = origin_x + col * TILE_SIZE + TILE_SIZE / 2.0;
    // ground_top: top surface of the tile at game_row (not tile centre).
    // Enemies/spawn/doors sit on this surface; the spawner offsets by COLLIDER_H/2 itself.
    let world_y = origin_y + (game_row + 1.0) * TILE_SIZE;
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

    /// With origin (0, 0), a 10×10 grid, and px = [18, 18] (one tile in, one
    /// tile down from the LDtk top-left), we expect:
    ///
    ///   col        = 18 / 18 = 1
    ///   ldtk_row   = 18 / 18 = 1
    ///   game_row   = (10 - 1) - 1 = 8
    ///   world_x    = 0 + 1 * 18 + 9 = 27
    ///   world_y    = 0 + 8 * 18 + 9 = 153
    #[test]
    fn px_to_world_converts_correctly() {
        let (wx, wy) = px_to_world([18, 18], 10, 0.0, 0.0);
        assert!(
            (wx - 27.0).abs() < 1e-4,
            "world_x should be 27.0, got {wx}"
        );
        assert!(
            (wy - 153.0).abs() < 1e-4,
            "world_y should be 153.0, got {wy}"
        );
    }

    // ------------------------------------------------------------------
    // convert_level_extracts_enemies
    // ------------------------------------------------------------------

    #[test]
    fn convert_level_extracts_enemies() {
        // 2×2 grid, enemy at px [0, 0] (top-left in LDtk).
        // c_hei = 2, so game_row = (2-1) - 0 = 1.
        // world_x = -864 + 0 + 9 = -855
        // world_y (ground_top) = -200 + (1+1)*18 = -200 + 36 = -164
        //   (tile centre would have been -200 + 1*18 + 9 = -173)
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

        // Verify world position (ground_top via px_to_world_surface).
        assert!(
            (enemy.x - (-855.0)).abs() < 1e-4,
            "expected x=-855, got {}",
            enemy.x
        );
        assert!(
            (enemy.y - (-164.0)).abs() < 1e-4,
            "expected y=-164 (ground_top), got {}",
            enemy.y
        );
    }

    // ------------------------------------------------------------------
    // convert_level_extracts_collectibles
    // ------------------------------------------------------------------

    #[test]
    fn convert_level_extracts_collectibles() {
        // 4×4 grid. Star at px [18, 0], HealthFood at px [36, 18].
        //
        // Star:
        //   col = 1, ldtk_row = 0, game_row = 3
        //   wx = -864 + 18 + 9 = -837, wy = -200 + 3*18 + 9 = -137
        //
        // HealthFood:
        //   col = 2, ldtk_row = 1, game_row = 2
        //   wx = -864 + 36 + 9 = -819, wy = -200 + 2*18 + 9 = -155
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
        assert!((star[0] - (-837.0)).abs() < 1e-4, "star x got {}", star[0]);
        assert!((star[1] - (-137.0)).abs() < 1e-4, "star y got {}", star[1]);
        assert!((star[2] - 1.0).abs() < 1e-4, "star z should be 1.0");

        let hf = layer.health_foods[0];
        assert!((hf[0] - (-819.0)).abs() < 1e-4, "hf x got {}", hf[0]);
        assert!((hf[1] - (-155.0)).abs() < 1e-4, "hf y got {}", hf[1]);
        assert!((hf[2] - 1.0).abs() < 1e-4, "hf z should be 1.0");
    }

    // ------------------------------------------------------------------
    // convert_level_extracts_doors
    // ------------------------------------------------------------------

    #[test]
    fn convert_level_extracts_doors() {
        // 3×3 grid. Door at px [0, 36], target_layer = 2.
        //   col = 0, ldtk_row = 2, game_row = (3-1) - 2 = 0
        //   wx = -864 + 0 + 9 = -855
        //   wy (ground_top) = -200 + (0+1)*18 = -200 + 18 = -182
        //     (tile centre would have been -200 + 0*18 + 9 = -191)
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
        assert!((door.x - (-855.0)).abs() < 1e-4, "door x got {}", door.x);
        assert!(
            (door.y - (-182.0)).abs() < 1e-4,
            "expected y=-182 (ground_top), got {}",
            door.y
        );
    }

    // ------------------------------------------------------------------
    // convert_level_extracts_spawn
    // ------------------------------------------------------------------

    #[test]
    fn convert_level_extracts_spawn() {
        // 5×5 grid. Spawn at px [36, 54] (col=2, ldtk_row=3, game_row=1).
        //   wx = -864 + 36 + 9 = -819
        //   wy (ground_top) = -200 + (1+1)*18 = -200 + 36 = -164
        //     (tile centre would have been -200 + 1*18 + 9 = -173)
        let intgrid = make_intgrid_layer(5, 5, vec![0; 25]);
        let spawn_entity = make_entity("Spawn", [36, 54], vec![]);
        let entities = make_entities_layer(5, 5, vec![spawn_entity]);
        let root = minimal_root_with_layers(intgrid, entities, vec![]);

        let levels = convert(&root);
        let layer = &levels[0].layers[0];

        let spawn = layer.spawn.expect("spawn should be set");
        assert!((spawn[0] - (-819.0)).abs() < 1e-4, "spawn x got {}", spawn[0]);
        assert!(
            (spawn[1] - (-164.0)).abs() < 1e-4,
            "expected y=-164 (ground_top), got {}",
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
}
