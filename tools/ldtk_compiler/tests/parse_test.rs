//! Integration test: parse a minimal LDtk fixture, validate, convert, and
//! verify the output JSON matches expectations.

use std::fs;
use std::process::Command;

/// Build and run the compiler against a hand-crafted LDtk fixture.
#[test]
fn end_to_end_compile() {
    let dir = tempfile::tempdir().expect("temp dir");
    let input_path = dir.path().join("test.ldtk");
    let output_path = dir.path().join("compiled_levels.json");

    // Minimal LDtk project with one level, one IntGrid layer, one Entities layer
    let ldtk_json = serde_json::json!({
        "levels": [{
            "identifier": "TestLevel",
            "layerInstances": [
                {
                    "__identifier": "Tiles",
                    "__type": "IntGrid",
                    "__gridSize": 18,
                    "__cWid": 4,
                    "__cHei": 3,
                    "intGridCsv": [0,0,0,0, 0,0,0,0, 1,1,1,1],
                    "entityInstances": []
                },
                {
                    "__identifier": "Entities",
                    "__type": "Entities",
                    "__gridSize": 18,
                    "__cWid": 4,
                    "__cHei": 3,
                    "intGridCsv": [],
                    "entityInstances": [
                        {
                            "__identifier": "Spawn",
                            "px": [18, 18],
                            "fieldInstances": []
                        },
                        {
                            "__identifier": "Enemy",
                            "px": [54, 18],
                            "fieldInstances": [
                                {"__identifier": "enemy_type", "__value": "Dog"},
                                {"__identifier": "patrol_range", "__value": 72.0},
                                {"__identifier": "health", "__value": 150.0}
                            ]
                        },
                        {
                            "__identifier": "Star",
                            "px": [36, 18],
                            "fieldInstances": []
                        }
                    ]
                }
            ],
            "fieldInstances": [
                {"__identifier": "OriginX", "__value": -864.0},
                {"__identifier": "OriginY", "__value": -200.0}
            ]
        }]
    });

    fs::write(&input_path, ldtk_json.to_string()).expect("write fixture");

    // Run the compiler binary
    let output = Command::new(env!("CARGO_BIN_EXE_ldtk_compiler"))
        .arg("--input")
        .arg(input_path.to_str().unwrap())
        .arg("--output")
        .arg(output_path.to_str().unwrap())
        .output()
        .expect("run compiler");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "Compiler failed:\nstdout: {stdout}\nstderr: {stderr}"
    );

    // Verify output exists and is valid JSON
    let contents = fs::read_to_string(&output_path).expect("read output");
    let root: serde_json::Value = serde_json::from_str(&contents).expect("parse output JSON");

    assert_eq!(root["schema_version"], 1);
    assert_eq!(root["levels"].as_array().unwrap().len(), 1);

    let level = &root["levels"][0];
    assert_eq!(level["id"], "TestLevel");
    assert_eq!(level["layers"].as_array().unwrap().len(), 1);

    let layer = &level["layers"][0];
    // Tiles should be Y-flipped: LDtk bottom row [1,1,1,1] → game row 0
    let tiles = layer["tiles"].as_array().unwrap();
    assert_eq!(tiles[0].as_array().unwrap(), &[1, 1, 1, 1]); // game bottom = LDtk top-2
    assert_eq!(tiles[2].as_array().unwrap(), &[0, 0, 0, 0]); // game top = LDtk top-0

    // Should have 1 enemy
    assert_eq!(layer["enemies"].as_array().unwrap().len(), 1);
    assert_eq!(layer["enemies"][0]["enemy_type"], "Dog");

    // Should have 1 star
    assert_eq!(layer["stars"].as_array().unwrap().len(), 1);

    // Should have a spawn point
    assert!(layer["spawn"].is_array());
}
