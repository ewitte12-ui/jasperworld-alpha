use crate::ldtk_schema::{LdtkEntityInstance, LdtkRoot};

/// Validates a parsed [`LdtkRoot`] and returns all discovered errors.
///
/// Uses collect-all semantics — every violation is recorded before returning
/// so callers see the complete picture in a single pass.
pub fn validate(root: &LdtkRoot) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    for level in &root.levels {
        // 1. Level must have layerInstances (not externally stored)
        let layers = match &level.layer_instances {
            Some(l) => l,
            None => {
                errors.push(format!(
                    "Level '{}': layerInstances is null (external files not supported)",
                    level.identifier
                ));
                continue;
            }
        };

        // 2. Must have at least one IntGrid layer
        let has_intgrid = layers.iter().any(|l| l.layer_type == "IntGrid");
        if !has_intgrid {
            errors.push(format!(
                "Level '{}': no IntGrid layer found",
                level.identifier
            ));
        }

        // 3. Must have at least one Entities layer
        let has_entities = layers.iter().any(|l| l.layer_type == "Entities");
        if !has_entities {
            errors.push(format!(
                "Level '{}': no Entities layer found",
                level.identifier
            ));
        }

        for layer in layers {
            // 4. Grid size must be 18 (TILE_SIZE)
            if layer.grid_size != 18 {
                errors.push(format!(
                    "Level '{}', layer '{}': grid_size is {} (expected 18)",
                    level.identifier, layer.identifier, layer.grid_size
                ));
            }

            // 5. IntGrid CSV length must match dimensions (empty is OK — new/unpainted level)
            if layer.layer_type == "IntGrid" {
                let expected = (layer.c_wid * layer.c_hei) as usize;
                if !layer.int_grid_csv.is_empty() && layer.int_grid_csv.len() != expected {
                    errors.push(format!(
                        "Level '{}', layer '{}': intGridCsv length {} != {}×{} = {}",
                        level.identifier,
                        layer.identifier,
                        layer.int_grid_csv.len(),
                        layer.c_wid,
                        layer.c_hei,
                        expected
                    ));
                }
            }

            // 6. Validate entity types and required fields
            for entity in &layer.entity_instances {
                match entity.identifier.as_str() {
                    "Enemy" => {
                        // Must have: enemy_type, patrol_range, health
                        validate_entity_field(entity, "enemy_type", &level.identifier, &mut errors);
                        validate_entity_field(
                            entity,
                            "patrol_range",
                            &level.identifier,
                            &mut errors,
                        );
                        validate_entity_field(entity, "health", &level.identifier, &mut errors);

                        // enemy_type value must be one of the known types
                        if let Some(val) = get_entity_field_str(entity, "enemy_type") {
                            match val.as_str() {
                                "Dog" | "Squirrel" | "Snake" | "Rat" | "Possum" => {}
                                other => errors.push(format!(
                                    "Level '{}': Enemy has unknown enemy_type '{}'",
                                    level.identifier, other
                                )),
                            }
                        }
                    }
                    "Door" => {
                        validate_entity_field(
                            entity,
                            "target_layer",
                            &level.identifier,
                            &mut errors,
                        );
                    }
                    // These entities need no special fields beyond position
                    "Star" | "HealthFood" | "Spawn" | "Gate" | "Exit" => {}
                    other => {
                        // Unknown entity type — warn but don't error
                        errors.push(format!(
                            "Level '{}': unknown entity type '{}'",
                            level.identifier, other
                        ));
                    }
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Checks that `field_name` exists on `entity` and has a non-null value.
/// Pushes a descriptive error to `errors` if the check fails.
fn validate_entity_field(
    entity: &LdtkEntityInstance,
    field_name: &str,
    level_name: &str,
    errors: &mut Vec<String>,
) {
    match entity.field_instances.iter().find(|f| f.identifier == field_name) {
        None => errors.push(format!(
            "Level '{}': entity '{}' at {:?} is missing required field '{}'",
            level_name, entity.identifier, entity.px, field_name
        )),
        Some(f) if f.value.is_none() => errors.push(format!(
            "Level '{}': entity '{}' at {:?} has null value for required field '{}'",
            level_name, entity.identifier, entity.px, field_name
        )),
        _ => {}
    }
}

/// Returns the string value of `field_name` on `entity`, if present and non-null.
///
/// Returns `None` when the field is absent, null, or not a JSON string.
fn get_entity_field_str(entity: &LdtkEntityInstance, field_name: &str) -> Option<String> {
    entity
        .field_instances
        .iter()
        .find(|f| f.identifier == field_name)
        .and_then(|f| f.value.as_ref())
        .and_then(|v| v.as_str())
        .map(|s| s.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ldtk_schema::LdtkRoot;

    fn minimal_valid_json() -> &'static str {
        r#"{
            "levels": [{
                "identifier": "Level1",
                "layerInstances": [
                    {
                        "__identifier": "Collision",
                        "__type": "IntGrid",
                        "__gridSize": 18,
                        "__cWid": 2,
                        "__cHei": 2,
                        "intGridCsv": [0, 0, 1, 1],
                        "entityInstances": []
                    },
                    {
                        "__identifier": "Entities",
                        "__type": "Entities",
                        "__gridSize": 18,
                        "__cWid": 2,
                        "__cHei": 2,
                        "intGridCsv": [],
                        "entityInstances": []
                    }
                ],
                "fieldInstances": []
            }]
        }"#
    }

    #[test]
    fn valid_level_passes() {
        let root: LdtkRoot = serde_json::from_str(minimal_valid_json()).unwrap();
        assert!(validate(&root).is_ok());
    }

    #[test]
    fn missing_layer_instances_is_error() {
        let json = r#"{
            "levels": [{"identifier": "L1", "layerInstances": null, "fieldInstances": []}]
        }"#;
        let root: LdtkRoot = serde_json::from_str(json).unwrap();
        let errs = validate(&root).unwrap_err();
        assert!(errs.iter().any(|e| e.contains("layerInstances is null")));
    }

    #[test]
    fn missing_intgrid_layer_is_error() {
        let json = r#"{
            "levels": [{
                "identifier": "L1",
                "layerInstances": [{
                    "__identifier": "Entities",
                    "__type": "Entities",
                    "__gridSize": 18,
                    "__cWid": 1,
                    "__cHei": 1,
                    "intGridCsv": [],
                    "entityInstances": []
                }],
                "fieldInstances": []
            }]
        }"#;
        let root: LdtkRoot = serde_json::from_str(json).unwrap();
        let errs = validate(&root).unwrap_err();
        assert!(errs.iter().any(|e| e.contains("no IntGrid layer found")));
    }

    #[test]
    fn missing_entities_layer_is_error() {
        let json = r#"{
            "levels": [{
                "identifier": "L1",
                "layerInstances": [{
                    "__identifier": "Collision",
                    "__type": "IntGrid",
                    "__gridSize": 18,
                    "__cWid": 1,
                    "__cHei": 1,
                    "intGridCsv": [0],
                    "entityInstances": []
                }],
                "fieldInstances": []
            }]
        }"#;
        let root: LdtkRoot = serde_json::from_str(json).unwrap();
        let errs = validate(&root).unwrap_err();
        assert!(errs.iter().any(|e| e.contains("no Entities layer found")));
    }

    #[test]
    fn wrong_grid_size_is_error() {
        let json = r#"{
            "levels": [{
                "identifier": "L1",
                "layerInstances": [
                    {
                        "__identifier": "Collision",
                        "__type": "IntGrid",
                        "__gridSize": 16,
                        "__cWid": 1,
                        "__cHei": 1,
                        "intGridCsv": [0],
                        "entityInstances": []
                    },
                    {
                        "__identifier": "Entities",
                        "__type": "Entities",
                        "__gridSize": 18,
                        "__cWid": 1,
                        "__cHei": 1,
                        "intGridCsv": [],
                        "entityInstances": []
                    }
                ],
                "fieldInstances": []
            }]
        }"#;
        let root: LdtkRoot = serde_json::from_str(json).unwrap();
        let errs = validate(&root).unwrap_err();
        assert!(errs.iter().any(|e| e.contains("grid_size is 16 (expected 18)")));
    }

    #[test]
    fn intgrid_csv_length_mismatch_is_error() {
        let json = r#"{
            "levels": [{
                "identifier": "L1",
                "layerInstances": [
                    {
                        "__identifier": "Collision",
                        "__type": "IntGrid",
                        "__gridSize": 18,
                        "__cWid": 3,
                        "__cHei": 3,
                        "intGridCsv": [0, 1],
                        "entityInstances": []
                    },
                    {
                        "__identifier": "Entities",
                        "__type": "Entities",
                        "__gridSize": 18,
                        "__cWid": 3,
                        "__cHei": 3,
                        "intGridCsv": [],
                        "entityInstances": []
                    }
                ],
                "fieldInstances": []
            }]
        }"#;
        let root: LdtkRoot = serde_json::from_str(json).unwrap();
        let errs = validate(&root).unwrap_err();
        assert!(errs.iter().any(|e| e.contains("intGridCsv length 2 != 3×3 = 9")));
    }

    #[test]
    fn enemy_missing_fields_is_error() {
        let json = r#"{
            "levels": [{
                "identifier": "L1",
                "layerInstances": [
                    {
                        "__identifier": "Collision",
                        "__type": "IntGrid",
                        "__gridSize": 18,
                        "__cWid": 1,
                        "__cHei": 1,
                        "intGridCsv": [0],
                        "entityInstances": []
                    },
                    {
                        "__identifier": "Entities",
                        "__type": "Entities",
                        "__gridSize": 18,
                        "__cWid": 1,
                        "__cHei": 1,
                        "intGridCsv": [],
                        "entityInstances": [{
                            "__identifier": "Enemy",
                            "px": [10, 20],
                            "fieldInstances": []
                        }]
                    }
                ],
                "fieldInstances": []
            }]
        }"#;
        let root: LdtkRoot = serde_json::from_str(json).unwrap();
        let errs = validate(&root).unwrap_err();
        assert!(errs.iter().any(|e| e.contains("missing required field 'enemy_type'")));
        assert!(errs.iter().any(|e| e.contains("missing required field 'patrol_range'")));
        assert!(errs.iter().any(|e| e.contains("missing required field 'health'")));
    }

    #[test]
    fn enemy_unknown_type_is_error() {
        let json = r#"{
            "levels": [{
                "identifier": "L1",
                "layerInstances": [
                    {
                        "__identifier": "Collision",
                        "__type": "IntGrid",
                        "__gridSize": 18,
                        "__cWid": 1,
                        "__cHei": 1,
                        "intGridCsv": [0],
                        "entityInstances": []
                    },
                    {
                        "__identifier": "Entities",
                        "__type": "Entities",
                        "__gridSize": 18,
                        "__cWid": 1,
                        "__cHei": 1,
                        "intGridCsv": [],
                        "entityInstances": [{
                            "__identifier": "Enemy",
                            "px": [0, 0],
                            "fieldInstances": [
                                {"__identifier": "enemy_type", "__value": "Dragon"},
                                {"__identifier": "patrol_range", "__value": 100},
                                {"__identifier": "health", "__value": 3}
                            ]
                        }]
                    }
                ],
                "fieldInstances": []
            }]
        }"#;
        let root: LdtkRoot = serde_json::from_str(json).unwrap();
        let errs = validate(&root).unwrap_err();
        assert!(errs.iter().any(|e| e.contains("unknown enemy_type 'Dragon'")));
    }

    #[test]
    fn known_enemy_types_pass() {
        for enemy_type in &["Dog", "Squirrel", "Snake", "Rat", "Possum"] {
            let json = format!(
                r#"{{
                "levels": [{{
                    "identifier": "L1",
                    "layerInstances": [
                        {{
                            "__identifier": "Collision",
                            "__type": "IntGrid",
                            "__gridSize": 18,
                            "__cWid": 1,
                            "__cHei": 1,
                            "intGridCsv": [0],
                            "entityInstances": []
                        }},
                        {{
                            "__identifier": "Entities",
                            "__type": "Entities",
                            "__gridSize": 18,
                            "__cWid": 1,
                            "__cHei": 1,
                            "intGridCsv": [],
                            "entityInstances": [{{
                                "__identifier": "Enemy",
                                "px": [0, 0],
                                "fieldInstances": [
                                    {{"__identifier": "enemy_type", "__value": "{}"}},
                                    {{"__identifier": "patrol_range", "__value": 100}},
                                    {{"__identifier": "health", "__value": 3}}
                                ]
                            }}]
                        }}
                    ],
                    "fieldInstances": []
                }}]
            }}"#,
                enemy_type
            );
            let root: LdtkRoot = serde_json::from_str(&json).unwrap();
            assert!(
                validate(&root).is_ok(),
                "enemy_type '{}' should be valid",
                enemy_type
            );
        }
    }

    #[test]
    fn door_missing_target_layer_is_error() {
        let json = r#"{
            "levels": [{
                "identifier": "L1",
                "layerInstances": [
                    {
                        "__identifier": "Collision",
                        "__type": "IntGrid",
                        "__gridSize": 18,
                        "__cWid": 1,
                        "__cHei": 1,
                        "intGridCsv": [0],
                        "entityInstances": []
                    },
                    {
                        "__identifier": "Entities",
                        "__type": "Entities",
                        "__gridSize": 18,
                        "__cWid": 1,
                        "__cHei": 1,
                        "intGridCsv": [],
                        "entityInstances": [{
                            "__identifier": "Door",
                            "px": [5, 5],
                            "fieldInstances": []
                        }]
                    }
                ],
                "fieldInstances": []
            }]
        }"#;
        let root: LdtkRoot = serde_json::from_str(json).unwrap();
        let errs = validate(&root).unwrap_err();
        assert!(errs.iter().any(|e| e.contains("missing required field 'target_layer'")));
    }

    #[test]
    fn unknown_entity_type_is_error() {
        let json = r#"{
            "levels": [{
                "identifier": "L1",
                "layerInstances": [
                    {
                        "__identifier": "Collision",
                        "__type": "IntGrid",
                        "__gridSize": 18,
                        "__cWid": 1,
                        "__cHei": 1,
                        "intGridCsv": [0],
                        "entityInstances": []
                    },
                    {
                        "__identifier": "Entities",
                        "__type": "Entities",
                        "__gridSize": 18,
                        "__cWid": 1,
                        "__cHei": 1,
                        "intGridCsv": [],
                        "entityInstances": [{
                            "__identifier": "Treasure",
                            "px": [0, 0],
                            "fieldInstances": []
                        }]
                    }
                ],
                "fieldInstances": []
            }]
        }"#;
        let root: LdtkRoot = serde_json::from_str(json).unwrap();
        let errs = validate(&root).unwrap_err();
        assert!(errs.iter().any(|e| e.contains("unknown entity type 'Treasure'")));
    }

    #[test]
    fn multiple_errors_all_reported() {
        // Level with wrong grid size AND missing Entities layer — both must appear
        let json = r#"{
            "levels": [{
                "identifier": "BadLevel",
                "layerInstances": [{
                    "__identifier": "Collision",
                    "__type": "IntGrid",
                    "__gridSize": 32,
                    "__cWid": 2,
                    "__cHei": 2,
                    "intGridCsv": [0, 0, 0],
                    "entityInstances": []
                }],
                "fieldInstances": []
            }]
        }"#;
        let root: LdtkRoot = serde_json::from_str(json).unwrap();
        let errs = validate(&root).unwrap_err();
        assert!(errs.iter().any(|e| e.contains("no Entities layer found")));
        assert!(errs.iter().any(|e| e.contains("grid_size is 32")));
        assert!(errs.iter().any(|e| e.contains("intGridCsv length 3 != 2×2 = 4")));
        assert!(errs.len() >= 3);
    }
}
