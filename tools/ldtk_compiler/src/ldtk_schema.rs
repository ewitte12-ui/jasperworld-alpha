use serde::Deserialize;

/// Root of an LDtk project file.
#[derive(Debug, Deserialize)]
pub struct LdtkRoot {
    pub levels: Vec<LdtkLevel>,
}

/// A single level in the LDtk project.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LdtkLevel {
    pub identifier: String,
    /// `None` when LDtk stores layer data in external files rather than inline.
    pub layer_instances: Option<Vec<LdtkLayerInstance>>,
    pub field_instances: Vec<LdtkFieldInstance>,
}

/// A layer within a level (IntGrid, Entities, Tiles, etc.).
///
/// Computed fields use the `__`-prefix convention from LDtk and must be
/// renamed explicitly because `rename_all` cannot produce leading underscores.
#[derive(Debug, Deserialize)]
pub struct LdtkLayerInstance {
    #[serde(rename = "__identifier")]
    pub identifier: String,
    /// Layer kind: "IntGrid", "Entities", "Tiles", or "AutoLayer".
    #[serde(rename = "__type")]
    pub layer_type: String,
    /// Size of one grid cell in pixels.
    #[serde(rename = "__gridSize")]
    pub grid_size: i32,
    /// Grid width in cells.
    #[serde(rename = "__cWid")]
    pub c_wid: i32,
    /// Grid height in cells.
    #[serde(rename = "__cHei")]
    pub c_hei: i32,
    /// Flattened IntGrid values in row-major order; empty for non-IntGrid layers.
    #[serde(rename = "intGridCsv", default)]
    pub int_grid_csv: Vec<i32>,
    /// Entity instances; empty for non-Entities layers.
    #[serde(rename = "entityInstances", default)]
    pub entity_instances: Vec<LdtkEntityInstance>,
}

/// An entity placed in an Entities layer.
#[derive(Debug, Deserialize)]
pub struct LdtkEntityInstance {
    #[serde(rename = "__identifier")]
    pub identifier: String,
    /// Pixel position [x, y] within the level coordinate space.
    pub px: [i32; 2],
    #[serde(rename = "fieldInstances")]
    pub field_instances: Vec<LdtkFieldInstance>,
}

/// A custom field attached to a level or entity.
///
/// `value` is `Option<serde_json::Value>` because LDtk fields can hold any
/// JSON type (string, number, bool, array, object) and may also be null when
/// the field is unset in the editor.
#[derive(Debug, Deserialize)]
pub struct LdtkFieldInstance {
    #[serde(rename = "__identifier")]
    pub identifier: String,
    #[serde(rename = "__value")]
    pub value: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_ldtk() {
        let json = r#"{
            "levels": [{
                "identifier": "TestLevel",
                "layerInstances": [{
                    "__identifier": "Tiles",
                    "__type": "IntGrid",
                    "__gridSize": 18,
                    "__cWid": 4,
                    "__cHei": 3,
                    "intGridCsv": [0,0,0,0, 0,0,0,0, 1,1,1,1],
                    "entityInstances": []
                }],
                "fieldInstances": [{
                    "__identifier": "OriginX",
                    "__value": -864.0
                }]
            }]
        }"#;
        let root: LdtkRoot = serde_json::from_str(json).unwrap();
        assert_eq!(root.levels.len(), 1);
        assert_eq!(root.levels[0].identifier, "TestLevel");
        let layer = &root.levels[0].layer_instances.as_ref().unwrap()[0];
        assert_eq!(layer.grid_size, 18);
        assert_eq!(layer.int_grid_csv.len(), 12);
    }
}
