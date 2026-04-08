use serde::Serialize;

use crate::converter::{ConvertedDoor, ConvertedEnemy, ConvertedLayer, ConvertedLevel, ConvertedProp};

// ---------------------------------------------------------------------------
// Output schema types — serialization contract for the compiled JSON
// ---------------------------------------------------------------------------

/// Root of the compiled output JSON.
///
/// `schema_version` is always 1 for this format and must be incremented if the
/// shape of the output changes in a backwards-incompatible way.
#[derive(Debug, Serialize)]
pub struct OutputRoot {
    pub schema_version: u32,
    pub levels: Vec<OutputLevel>,
}

/// Compiled data for a single LDtk level.
#[derive(Debug, Serialize)]
pub struct OutputLevel {
    /// Matches the LDtk level identifier string.
    pub id: String,
    pub layers: Vec<OutputLayer>,
}

/// Compiled data for one layer (sublevel) within a level.
///
/// `id` is the 0-based position of this layer in its parent level's layer list.
#[derive(Debug, Serialize)]
pub struct OutputLayer {
    /// 0-based index of this layer within its parent level.
    pub id: usize,
    /// Grid width in tiles.
    pub cols: i32,
    /// Grid height in tiles.
    pub rows: i32,
    /// World-space X coordinate of the layer's left edge.
    pub origin_x: f32,
    /// World-space Y coordinate of the layer's bottom edge.
    pub origin_y: f32,
    /// Player spawn position `[x, y]` in world coordinates, or `null` if absent.
    pub spawn: Option<[f32; 2]>,
    /// 2-D tile array; `tiles[game_row][col]` where `game_row = 0` is the bottom row.
    pub tiles: Vec<Vec<u8>>,
    pub enemies: Vec<OutputEnemy>,
    /// Star collectible positions `[x, y, z]` in world coordinates.
    pub stars: Vec<[f32; 3]>,
    /// HealthFood collectible positions `[x, y, z]` in world coordinates.
    pub health_foods: Vec<[f32; 3]>,
    pub doors: Vec<OutputDoor>,
    /// Decorative prop entities placed visually in LDtk.
    pub props: Vec<OutputProp>,
    /// Column index of the Gate entity, or `null` if no Gate is present.
    pub gate_col: Option<i32>,
    /// Identifier of the next level to transition to via the Exit, or `null`.
    pub exit_next_level: Option<String>,
    /// Stars the player must collect before the Gate opens, or `null`.
    pub stars_required: Option<i32>,
}

/// Compiled enemy entity.
#[derive(Debug, Serialize)]
pub struct OutputEnemy {
    /// One of: "Dog", "Squirrel", "Snake", "Rat", "Possum".
    pub enemy_type: String,
    /// World-space X centre.
    pub x: f32,
    /// World-space Y centre (ground_top; spawner adds COLLIDER_H/2 internally).
    pub y: f32,
    /// Half-width of the patrol route in world units.
    pub patrol_range: f32,
    /// Maximum hit-points.
    pub health: f32,
    /// Per-instance movement speed override; omitted from JSON when absent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed_override: Option<f32>,
}

/// Compiled prop entity (decorative model placed via LDtk).
#[derive(Debug, Serialize)]
pub struct OutputProp {
    /// Asset path for the GLB model, e.g. "models/small_rock.glb".
    pub model_id: String,
    /// World-space X centre (tile centre).
    pub x: f32,
    /// World-space Y centre (tile centre).
    pub y: f32,
    /// Z depth in world space.
    pub z: f32,
    /// Uniform XY scale.
    pub scale_xy: f32,
    /// Z (depth) scale.
    pub scale_z: f32,
    /// Y-axis rotation in radians.
    pub rotation_y: f32,
}

/// Compiled door entity.
#[derive(Debug, Serialize)]
pub struct OutputDoor {
    /// Index of the sublevel/layer this door leads to.
    pub target_layer: i32,
    /// World-space X centre.
    pub x: f32,
    /// World-space Y centre.
    pub y: f32,
}

// ---------------------------------------------------------------------------
// Conversions from intermediate converter types
// ---------------------------------------------------------------------------

impl OutputRoot {
    /// Build an [`OutputRoot`] from the Phase-4 converter's output.
    pub fn from_converted(levels: Vec<ConvertedLevel>) -> Self {
        Self {
            // Version 1 is the initial stable format.  Increment if the schema
            // shape changes in a backwards-incompatible way.
            schema_version: 1,
            levels: levels.into_iter().map(OutputLevel::from_converted).collect(),
        }
    }
}

impl OutputLevel {
    fn from_converted(level: ConvertedLevel) -> Self {
        Self {
            id: level.identifier,
            layers: level
                .layers
                .into_iter()
                .enumerate()
                .map(|(idx, layer)| OutputLayer::from_converted(idx, layer))
                .collect(),
        }
    }
}

impl OutputLayer {
    fn from_converted(id: usize, layer: ConvertedLayer) -> Self {
        Self {
            id,
            cols: layer.cols,
            rows: layer.rows,
            origin_x: layer.origin_x,
            origin_y: layer.origin_y,
            spawn: layer.spawn,
            tiles: layer.tiles,
            enemies: layer.enemies.into_iter().map(OutputEnemy::from_converted).collect(),
            stars: layer.stars,
            health_foods: layer.health_foods,
            doors: layer.doors.into_iter().map(OutputDoor::from_converted).collect(),
            props: layer.props.into_iter().map(OutputProp::from_converted).collect(),
            gate_col: layer.gate_col,
            exit_next_level: layer.exit_next_level,
            stars_required: layer.stars_required,
        }
    }
}

impl OutputEnemy {
    fn from_converted(enemy: ConvertedEnemy) -> Self {
        Self {
            enemy_type: enemy.enemy_type,
            x: enemy.x,
            y: enemy.y,
            patrol_range: enemy.patrol_range,
            health: enemy.health,
            speed_override: enemy.speed_override,
        }
    }
}

impl OutputDoor {
    fn from_converted(door: ConvertedDoor) -> Self {
        Self {
            target_layer: door.target_layer,
            x: door.x,
            y: door.y,
        }
    }
}

impl OutputProp {
    fn from_converted(prop: ConvertedProp) -> Self {
        Self {
            model_id: prop.model_id,
            x: prop.x,
            y: prop.y,
            z: prop.z,
            scale_xy: prop.scale_xy,
            scale_z: prop.scale_z,
            rotation_y: prop.rotation_y,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::converter::{ConvertedDoor, ConvertedEnemy, ConvertedLayer, ConvertedLevel};

    // ------------------------------------------------------------------
    // Helpers
    // ------------------------------------------------------------------

    fn minimal_converted_level() -> ConvertedLevel {
        ConvertedLevel {
            identifier: "Level_0".to_string(),
            layers: vec![ConvertedLayer {
                cols: 4,
                rows: 3,
                origin_x: -100.0,
                origin_y: -50.0,
                spawn: Some([10.0, 20.0]),
                tiles: vec![
                    vec![1, 0, 0, 1],
                    vec![0, 0, 0, 0],
                    vec![1, 1, 1, 1],
                ],
                enemies: vec![ConvertedEnemy {
                    enemy_type: "Dog".to_string(),
                    x: 5.0,
                    y: 15.0,
                    patrol_range: 36.0,
                    health: 2.0,
                    speed_override: None,
                }],
                stars: vec![[30.0, 40.0, 1.0]],
                health_foods: vec![],
                doors: vec![ConvertedDoor {
                    target_layer: 1,
                    x: 50.0,
                    y: 10.0,
                }],
                props: vec![],
                gate_col: Some(7),
                exit_next_level: Some("Level_1".to_string()),
                stars_required: Some(3),
            }],
        }
    }

    // ------------------------------------------------------------------
    // schema_version_is_1
    // ------------------------------------------------------------------

    /// schema_version must always be 1 for the initial stable format.
    #[test]
    fn schema_version_is_1() {
        let root = OutputRoot::from_converted(vec![]);
        assert_eq!(root.schema_version, 1);
    }

    // ------------------------------------------------------------------
    // from_converted_maps_fields
    // ------------------------------------------------------------------

    /// Every field from the intermediate ConvertedLevel must transfer correctly
    /// to the corresponding OutputRoot / OutputLevel / OutputLayer fields.
    #[test]
    fn from_converted_maps_fields() {
        let converted = minimal_converted_level();
        let root = OutputRoot::from_converted(vec![converted]);

        assert_eq!(root.schema_version, 1);
        assert_eq!(root.levels.len(), 1);

        let level = &root.levels[0];
        assert_eq!(level.id, "Level_0");
        assert_eq!(level.layers.len(), 1);

        let layer = &level.layers[0];
        // Layer index
        assert_eq!(layer.id, 0);
        // Dimensions
        assert_eq!(layer.cols, 4);
        assert_eq!(layer.rows, 3);
        // Origin
        assert!((layer.origin_x - (-100.0)).abs() < 1e-4);
        assert!((layer.origin_y - (-50.0)).abs() < 1e-4);
        // Spawn
        let spawn = layer.spawn.expect("spawn must be set");
        assert!((spawn[0] - 10.0).abs() < 1e-4);
        assert!((spawn[1] - 20.0).abs() < 1e-4);
        // Tiles
        assert_eq!(layer.tiles.len(), 3);
        assert_eq!(layer.tiles[2], vec![1u8, 1, 1, 1]);
        // Enemy
        assert_eq!(layer.enemies.len(), 1);
        let enemy = &layer.enemies[0];
        assert_eq!(enemy.enemy_type, "Dog");
        assert!((enemy.x - 5.0).abs() < 1e-4);
        assert!((enemy.y - 15.0).abs() < 1e-4);
        assert!((enemy.patrol_range - 36.0).abs() < 1e-4);
        assert!((enemy.health - 2.0).abs() < 1e-4);
        assert!(enemy.speed_override.is_none());
        // Stars
        assert_eq!(layer.stars.len(), 1);
        assert!((layer.stars[0][2] - 1.0).abs() < 1e-4);
        // Doors
        assert_eq!(layer.doors.len(), 1);
        assert_eq!(layer.doors[0].target_layer, 1);
        // Gate / exit
        assert_eq!(layer.gate_col, Some(7));
        assert_eq!(layer.exit_next_level.as_deref(), Some("Level_1"));
        assert_eq!(layer.stars_required, Some(3));
    }

    // ------------------------------------------------------------------
    // serializes_to_expected_json
    // ------------------------------------------------------------------

    /// Serialise a minimal OutputRoot, then round-trip through serde_json::Value
    /// and verify key fields are present and have the expected values.
    #[test]
    fn serializes_to_expected_json() {
        let root = OutputRoot::from_converted(vec![minimal_converted_level()]);
        let json_str = serde_json::to_string_pretty(&root).expect("serialisation must succeed");

        let value: serde_json::Value =
            serde_json::from_str(&json_str).expect("deserialisation of own output must succeed");

        // Top-level fields
        assert_eq!(value["schema_version"], 1u32);
        assert!(value["levels"].is_array());
        assert_eq!(value["levels"].as_array().unwrap().len(), 1);

        let level = &value["levels"][0];
        assert_eq!(level["id"], "Level_0");

        let layer = &level["layers"][0];
        assert_eq!(layer["id"], 0u64);
        assert_eq!(layer["cols"], 4i32);
        assert_eq!(layer["rows"], 3i32);
        assert_eq!(layer["gate_col"], 7i32);
        assert_eq!(layer["stars_required"], 3i32);
        assert_eq!(layer["exit_next_level"], "Level_1");

        // Verify the enemy round-trips and speed_override is absent (None → omitted)
        let enemy = &layer["enemies"][0];
        assert_eq!(enemy["enemy_type"], "Dog");
        assert!(enemy["speed_override"].is_null(), "speed_override should be absent");

        // Verify star array
        let star = &layer["stars"][0];
        assert!((star[2].as_f64().unwrap() - 1.0).abs() < 1e-4);
    }
}
