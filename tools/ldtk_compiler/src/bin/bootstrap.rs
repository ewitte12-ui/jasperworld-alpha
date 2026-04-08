/// Bootstrap binary: generates assets/levels/compiled_levels.json from
/// hardcoded level data.  Run with:
///
///   cargo run -p ldtk_compiler --bin bootstrap
///
/// The output path is resolved at compile time from CARGO_MANIFEST_DIR,
/// walking two levels up to reach the workspace root.

use ldtk_compiler::output_schema::{OutputDoor, OutputEnemy, OutputLayer, OutputLevel, OutputRoot};
use ldtk_compiler::writer::write_atomic;

// ---------------------------------------------------------------------------
// Tile-grid helpers
// ---------------------------------------------------------------------------

fn empty_grid(cols: usize, rows: usize) -> Vec<Vec<u8>> {
    vec![vec![0u8; cols]; rows]
}

fn solid_row(cols: usize) -> Vec<u8> {
    vec![1u8; cols]
}

/// Set tiles in `row` from `col_start` to `col_end` (inclusive) to value 2
/// (platform tile).
fn set_platform(grid: &mut Vec<Vec<u8>>, row: usize, col_start: usize, col_end: usize) {
    for col in col_start..=col_end {
        grid[row][col] = 2;
    }
}

/// Set the left and right border tiles of `row` to value 1 (solid).
fn set_walls(grid: &mut Vec<Vec<u8>>, row: usize, cols: usize) {
    grid[row][0] = 1;
    grid[row][cols - 1] = 1;
}

// ---------------------------------------------------------------------------
// Forest Level
// ---------------------------------------------------------------------------

fn forest_layer0() -> OutputLayer {
    let cols: usize = 96;
    let rows: usize = 22;
    let mut grid = empty_grid(cols, rows);

    // rows 0-2: all solid
    for r in 0..=2 {
        grid[r] = solid_row(cols);
    }
    // rows 3-5: already empty from empty_grid

    // row 6: platforms at 4..=8, 22..=26, 35..=39, 55..=60, 67..=71, 83..=87
    set_platform(&mut grid, 6, 4, 8);
    set_platform(&mut grid, 6, 22, 26);
    set_platform(&mut grid, 6, 35, 39);
    set_platform(&mut grid, 6, 55, 60);
    set_platform(&mut grid, 6, 67, 71);
    set_platform(&mut grid, 6, 83, 87);

    // rows 7-9: empty

    // row 10: platforms at 13..=18, 44..=46, 75..=80
    set_platform(&mut grid, 10, 13, 18);
    set_platform(&mut grid, 10, 44, 46);
    set_platform(&mut grid, 10, 75, 80);

    // rows 11-13: empty

    // row 14: platforms at 48..=52, 68..=72
    set_platform(&mut grid, 14, 48, 52);
    set_platform(&mut grid, 14, 68, 72);

    // rows 15-21: empty

    OutputLayer {
        id: 0,
        cols: cols as i32,
        rows: rows as i32,
        origin_x: -864.0,
        origin_y: -200.0,
        spawn: Some([-819.0, -128.0]),
        tiles: grid,
        enemies: vec![
            OutputEnemy {
                enemy_type: "Dog".to_string(),
                x: -9.0,
                y: -146.0,
                patrol_range: 72.0,
                health: 150.0,
                speed_override: None,
            },
            OutputEnemy {
                enemy_type: "Snake".to_string(),
                x: 477.0,
                y: -146.0,
                patrol_range: 54.0,
                health: 50.0,
                speed_override: None,
            },
            OutputEnemy {
                enemy_type: "Possum".to_string(),
                x: 621.0,
                y: -146.0,
                patrol_range: 54.0,
                health: 50.0,
                speed_override: None,
            },
        ],
        stars: vec![
            [-729.0, -137.0, 1.0],
            [-495.0, -137.0, 1.0],
            [-765.0, -65.0, 1.0],
            [-585.0, 7.0, 1.0],
            [-189.0, -65.0, 1.0],
            [-27.0, 7.0, 1.0],
            [45.0, 79.0, 1.0],
            [189.0, -137.0, 1.0],
            [387.0, -65.0, 1.0],
            [531.0, 7.0, 1.0],
            [675.0, -137.0, 1.0],
        ],
        health_foods: vec![
            [-801.0, -137.0, 1.0],
            [-405.0, -65.0, 1.0],
            [81.0, -137.0, 1.0],
            [171.0, -65.0, 1.0],
            [567.0, 7.0, 1.0],
        ],
        doors: vec![
            OutputDoor { target_layer: 1, x: -351.0, y: -146.0 },
            OutputDoor { target_layer: 2, x: 45.0, y: 70.0 },
        ],
        props: vec![],
        lights: vec![],
        gate_col: Some(91),
        exit_next_level: Some("Subdivision".to_string()),
        stars_required: Some(10),
    }
}

fn forest_layer1() -> OutputLayer {
    // Cave sublevel: 32×18, origin (5000, 5000)
    let cols: usize = 32;
    let rows: usize = 18;
    let mut grid = empty_grid(cols, rows);

    // rows 0-1: all solid
    for r in 0..=1 {
        grid[r] = solid_row(cols);
    }

    // rows 2-4: walls only
    for r in 2..=4 {
        set_walls(&mut grid, r, cols);
    }

    // row 5: walls + platforms at 5..=9, 22..=26
    set_walls(&mut grid, 5, cols);
    set_platform(&mut grid, 5, 5, 9);
    set_platform(&mut grid, 5, 22, 26);

    // rows 6-8: walls only
    for r in 6..=8 {
        set_walls(&mut grid, r, cols);
    }

    // row 9: walls + platforms at 12..=16
    set_walls(&mut grid, 9, cols);
    set_platform(&mut grid, 9, 12, 16);

    // rows 10-11: walls only
    for r in 10..=11 {
        set_walls(&mut grid, r, cols);
    }

    // row 12: walls + platforms at 20..=24
    set_walls(&mut grid, 12, cols);
    set_platform(&mut grid, 12, 20, 24);

    // rows 13-15: walls only
    for r in 13..=15 {
        set_walls(&mut grid, r, cols);
    }

    // rows 16-17: all solid
    for r in 16..=17 {
        grid[r] = solid_row(cols);
    }

    OutputLayer {
        id: 1,
        cols: cols as i32,
        rows: rows as i32,
        origin_x: 5000.0,
        origin_y: 5000.0,
        spawn: Some([5045.0, 5063.0]),
        tiles: grid,
        enemies: vec![],
        stars: vec![],
        health_foods: vec![],
        doors: vec![],
        props: vec![],
        lights: vec![],
        gate_col: None,
        exit_next_level: None,
        stars_required: None,
    }
}

fn forest_layer2() -> OutputLayer {
    // Treetop: 96×22, origin (-864, -200)
    let cols: usize = 96;
    let rows: usize = 22;
    let mut grid = empty_grid(cols, rows);

    // rows 0-2: all solid
    for r in 0..=2 {
        grid[r] = solid_row(cols);
    }
    // rows 3-5: empty

    // row 6: platforms at 6..=10, 32..=36, 62..=66, 88..=91
    set_platform(&mut grid, 6, 6, 10);
    set_platform(&mut grid, 6, 32, 36);
    set_platform(&mut grid, 6, 62, 66);
    set_platform(&mut grid, 6, 88, 91);

    // rows 7-9: empty

    // row 10: platforms at 16..=20, 48..=52, 78..=82
    set_platform(&mut grid, 10, 16, 20);
    set_platform(&mut grid, 10, 48, 52);
    set_platform(&mut grid, 10, 78, 82);

    // rows 11-13: empty

    // row 14: platforms at 25..=29, 65..=69
    set_platform(&mut grid, 14, 25, 29);
    set_platform(&mut grid, 14, 65, 69);

    // rows 15-21: empty

    OutputLayer {
        id: 2,
        cols: cols as i32,
        rows: rows as i32,
        origin_x: -864.0,
        origin_y: -200.0,
        spawn: Some([-783.0, -128.0]),
        tiles: grid,
        enemies: vec![],
        stars: vec![],
        health_foods: vec![],
        doors: vec![],
        props: vec![],
        lights: vec![],
        gate_col: None,
        exit_next_level: None,
        stars_required: None,
    }
}

fn forest_level() -> OutputLevel {
    OutputLevel {
        id: "Forest".to_string(),
        layers: vec![forest_layer0(), forest_layer1(), forest_layer2()],
    }
}

// ---------------------------------------------------------------------------
// Subdivision Level
// ---------------------------------------------------------------------------

fn subdivision_layer0() -> OutputLayer {
    let cols: usize = 96;
    let rows: usize = 22;
    let mut grid = empty_grid(cols, rows);

    // rows 0-2: all solid
    for r in 0..=2 {
        grid[r] = solid_row(cols);
    }
    // rows 3-5: empty

    // row 6: platforms at 5..=9, 20..=25, 38..=43, 56..=61, 72..=77, 85..=89
    set_platform(&mut grid, 6, 5, 9);
    set_platform(&mut grid, 6, 20, 25);
    set_platform(&mut grid, 6, 38, 43);
    set_platform(&mut grid, 6, 56, 61);
    set_platform(&mut grid, 6, 72, 77);
    set_platform(&mut grid, 6, 85, 89);

    // rows 7-9: empty

    // row 10: platforms at 12..=17, 28..=33, 45..=50, 63..=68, 76..=81
    set_platform(&mut grid, 10, 12, 17);
    set_platform(&mut grid, 10, 28, 33);
    set_platform(&mut grid, 10, 45, 50);
    set_platform(&mut grid, 10, 63, 68);
    set_platform(&mut grid, 10, 76, 81);

    // rows 11-13: empty

    // row 14: platforms at 30..=34, 65..=69
    set_platform(&mut grid, 14, 30, 34);
    set_platform(&mut grid, 14, 65, 69);

    // rows 15-21: empty

    OutputLayer {
        id: 0,
        cols: cols as i32,
        rows: rows as i32,
        origin_x: -864.0,
        origin_y: -200.0,
        spawn: Some([-819.0, -128.0]),
        tiles: grid,
        enemies: vec![
            OutputEnemy {
                enemy_type: "Dog".to_string(),
                x: 45.0,
                y: -146.0,
                patrol_range: 108.0,
                health: 250.0,
                speed_override: None,
            },
            OutputEnemy {
                enemy_type: "Snake".to_string(),
                x: 495.0,
                y: -146.0,
                patrol_range: 54.0,
                health: 50.0,
                speed_override: None,
            },
            OutputEnemy {
                enemy_type: "Possum".to_string(),
                x: 657.0,
                y: -146.0,
                patrol_range: 54.0,
                health: 50.0,
                speed_override: None,
            },
        ],
        stars: vec![
            [-711.0, -137.0, 1.0],
            [-459.0, -137.0, 1.0],
            [-747.0, -65.0, 1.0],
            [-567.0, 7.0, 1.0],
            [-135.0, -65.0, 1.0],
            [9.0, 7.0, 1.0],
            [-279.0, 79.0, 1.0],
            [225.0, -137.0, 1.0],
            [405.0, -65.0, 1.0],
            [549.0, 7.0, 1.0],
            [711.0, -137.0, 1.0],
        ],
        health_foods: vec![
            [-783.0, -137.0, 1.0],
            [-387.0, -65.0, 1.0],
            [99.0, -137.0, 1.0],
            [189.0, -65.0, 1.0],
            [585.0, 7.0, 1.0],
        ],
        doors: vec![
            OutputDoor { target_layer: 1, x: -351.0, y: -146.0 },
            OutputDoor { target_layer: 2, x: 351.0, y: 70.0 },
        ],
        props: vec![],
        lights: vec![],
        gate_col: Some(91),
        exit_next_level: Some("City".to_string()),
        stars_required: Some(10),
    }
}

fn subdivision_layer1() -> OutputLayer {
    // Sewers: 32×18, origin (5000, 5000)
    let cols: usize = 32;
    let rows: usize = 18;
    let mut grid = empty_grid(cols, rows);

    // rows 0-1: all solid
    for r in 0..=1 {
        grid[r] = solid_row(cols);
    }

    // rows 2-4: walls only
    for r in 2..=4 {
        set_walls(&mut grid, r, cols);
    }

    // row 5: walls + platforms at 4..=8, 20..=24
    set_walls(&mut grid, 5, cols);
    set_platform(&mut grid, 5, 4, 8);
    set_platform(&mut grid, 5, 20, 24);

    // rows 6-8: walls only
    for r in 6..=8 {
        set_walls(&mut grid, r, cols);
    }

    // row 9: walls + platforms at 10..=14, 25..=29
    set_walls(&mut grid, 9, cols);
    set_platform(&mut grid, 9, 10, 14);
    set_platform(&mut grid, 9, 25, 29);

    // rows 10-11: walls only
    for r in 10..=11 {
        set_walls(&mut grid, r, cols);
    }

    // row 12: walls + platforms at 15..=19
    set_walls(&mut grid, 12, cols);
    set_platform(&mut grid, 12, 15, 19);

    // rows 13-15: walls only
    for r in 13..=15 {
        set_walls(&mut grid, r, cols);
    }

    // rows 16-17: all solid
    for r in 16..=17 {
        grid[r] = solid_row(cols);
    }

    OutputLayer {
        id: 1,
        cols: cols as i32,
        rows: rows as i32,
        origin_x: 5000.0,
        origin_y: 5000.0,
        spawn: Some([5045.0, 5063.0]),
        tiles: grid,
        enemies: vec![],
        stars: vec![],
        health_foods: vec![],
        doors: vec![],
        props: vec![],
        lights: vec![],
        gate_col: None,
        exit_next_level: None,
        stars_required: None,
    }
}

fn subdivision_layer2() -> OutputLayer {
    // Rooftop: 96×22, origin (-864, -200)
    let cols: usize = 96;
    let rows: usize = 22;
    let mut grid = empty_grid(cols, rows);

    // rows 0-2: all solid
    for r in 0..=2 {
        grid[r] = solid_row(cols);
    }
    // rows 3-5: empty

    // row 6: platforms at 8..=12, 30..=34, 60..=64, 86..=90
    set_platform(&mut grid, 6, 8, 12);
    set_platform(&mut grid, 6, 30, 34);
    set_platform(&mut grid, 6, 60, 64);
    set_platform(&mut grid, 6, 86, 90);

    // rows 7-9: empty

    // row 10: platforms at 18..=22, 24..=29, 50..=54, 66..=71, 80..=84
    set_platform(&mut grid, 10, 18, 22);
    set_platform(&mut grid, 10, 24, 29);
    set_platform(&mut grid, 10, 50, 54);
    set_platform(&mut grid, 10, 66, 71);
    set_platform(&mut grid, 10, 80, 84);

    // rows 11-13: empty

    // row 14: platforms at 26..=30, 68..=72
    set_platform(&mut grid, 14, 26, 30);
    set_platform(&mut grid, 14, 68, 72);

    // rows 15-21: empty

    OutputLayer {
        id: 2,
        cols: cols as i32,
        rows: rows as i32,
        origin_x: -864.0,
        origin_y: -200.0,
        spawn: Some([-783.0, -128.0]),
        tiles: grid,
        enemies: vec![],
        stars: vec![],
        health_foods: vec![],
        doors: vec![],
        props: vec![],
        lights: vec![],
        gate_col: None,
        exit_next_level: None,
        stars_required: None,
    }
}

fn subdivision_level() -> OutputLevel {
    OutputLevel {
        id: "Subdivision".to_string(),
        layers: vec![subdivision_layer0(), subdivision_layer1(), subdivision_layer2()],
    }
}

// ---------------------------------------------------------------------------
// City Level
// ---------------------------------------------------------------------------

fn city_layer0() -> OutputLayer {
    let cols: usize = 96;
    let rows: usize = 44;
    let mut grid = empty_grid(cols, rows);

    // rows 0-2: all solid
    for r in 0..=2 {
        grid[r] = solid_row(cols);
    }
    // rows 3-5: empty

    // row 6: platforms at 5..=9, 20..=25, 38..=43, 56..=61, 72..=77, 85..=89
    set_platform(&mut grid, 6, 5, 9);
    set_platform(&mut grid, 6, 20, 25);
    set_platform(&mut grid, 6, 38, 43);
    set_platform(&mut grid, 6, 56, 61);
    set_platform(&mut grid, 6, 72, 77);
    set_platform(&mut grid, 6, 85, 89);

    // row 7: empty

    // row 8: platforms at 12..=17, 42..=47, 68..=73
    set_platform(&mut grid, 8, 12, 17);
    set_platform(&mut grid, 8, 42, 47);
    set_platform(&mut grid, 8, 68, 73);

    // row 9: empty

    // row 10: platforms at 10..=15, 28..=33, 50..=55, 70..=75, 85..=90
    set_platform(&mut grid, 10, 10, 15);
    set_platform(&mut grid, 10, 28, 33);
    set_platform(&mut grid, 10, 50, 55);
    set_platform(&mut grid, 10, 70, 75);
    set_platform(&mut grid, 10, 85, 90);

    // rows 11-13: empty

    // row 14: platforms at 18..=23, 40..=45, 62..=67, 82..=87
    set_platform(&mut grid, 14, 18, 23);
    set_platform(&mut grid, 14, 40, 45);
    set_platform(&mut grid, 14, 62, 67);
    set_platform(&mut grid, 14, 82, 87);

    // row 15: empty

    // row 16: platforms at 48..=53
    set_platform(&mut grid, 16, 48, 53);

    // row 17: empty

    // row 18: platforms at 25..=30, 55..=60, 78..=83
    set_platform(&mut grid, 18, 25, 30);
    set_platform(&mut grid, 18, 55, 60);
    set_platform(&mut grid, 18, 78, 83);

    // row 19: empty

    // row 20: platforms at 65..=70
    set_platform(&mut grid, 20, 65, 70);

    // row 21: empty

    // row 22: platforms at 15..=20, 45..=50, 75..=80
    set_platform(&mut grid, 22, 15, 20);
    set_platform(&mut grid, 22, 45, 50);
    set_platform(&mut grid, 22, 75, 80);

    // row 23: empty

    // row 24: platforms at 22..=27, 52..=57, 70..=75
    set_platform(&mut grid, 24, 22, 27);
    set_platform(&mut grid, 24, 52, 57);
    set_platform(&mut grid, 24, 70, 75);

    // row 25: empty

    // row 26: platforms at 30..=35, 60..=65, 78..=83
    set_platform(&mut grid, 26, 30, 35);
    set_platform(&mut grid, 26, 60, 65);
    set_platform(&mut grid, 26, 78, 83);

    // rows 27-29: empty

    // row 30: platforms at 20..=25, 50..=55, 80..=85
    set_platform(&mut grid, 30, 20, 25);
    set_platform(&mut grid, 30, 50, 55);
    set_platform(&mut grid, 30, 80, 85);

    // rows 31-43: empty

    OutputLayer {
        id: 0,
        cols: cols as i32,
        rows: rows as i32,
        origin_x: -864.0,
        origin_y: -200.0,
        spawn: Some([-819.0, -128.0]),
        tiles: grid,
        enemies: vec![
            OutputEnemy {
                enemy_type: "Dog".to_string(),
                x: 45.0,
                y: -146.0,
                patrol_range: 144.0,
                health: 500.0,
                speed_override: Some(150.0),
            },
            OutputEnemy {
                enemy_type: "Snake".to_string(),
                x: -405.0,
                y: -146.0,
                patrol_range: 54.0,
                health: 100.0,
                speed_override: None,
            },
            OutputEnemy {
                enemy_type: "Snake".to_string(),
                x: 495.0,
                y: -146.0,
                patrol_range: 54.0,
                health: 100.0,
                speed_override: None,
            },
            OutputEnemy {
                enemy_type: "Possum".to_string(),
                x: -135.0,
                y: -146.0,
                patrol_range: 54.0,
                health: 100.0,
                speed_override: None,
            },
            OutputEnemy {
                enemy_type: "Possum".to_string(),
                x: 657.0,
                y: -146.0,
                patrol_range: 54.0,
                health: 100.0,
                speed_override: None,
            },
            OutputEnemy {
                enemy_type: "Rat".to_string(),
                x: 225.0,
                y: -146.0,
                patrol_range: 72.0,
                health: 100.0,
                speed_override: None,
            },
            OutputEnemy {
                enemy_type: "Rat".to_string(),
                x: 729.0,
                y: -146.0,
                patrol_range: 72.0,
                health: 100.0,
                speed_override: None,
            },
        ],
        stars: vec![
            [-711.0, -137.0, 1.0],
            [-459.0, -65.0, 1.0],
            [-225.0, 7.0, 1.0],
            [-45.0, 79.0, 1.0],
            [135.0, -137.0, 1.0],
            [279.0, -29.0, 1.0],
            [441.0, 151.0, 1.0],
            [-279.0, 295.0, 1.0],
            [81.0, 223.0, 1.0],
            [585.0, -137.0, 1.0],
            [711.0, -137.0, 1.0],
        ],
        health_foods: vec![
            [-585.0, -137.0, 1.0],
            [-315.0, -101.0, 1.0],
            [45.0, 7.0, 1.0],
            [405.0, -137.0, 1.0],
            [675.0, -65.0, 1.0],
        ],
        doors: vec![
            OutputDoor { target_layer: 1, x: -351.0, y: -146.0 },
            OutputDoor { target_layer: 2, x: 81.0, y: 358.0 },
        ],
        props: vec![],
        lights: vec![],
        gate_col: Some(91),
        exit_next_level: Some("City".to_string()),
        stars_required: Some(10),
    }
}

fn city_layer1() -> OutputLayer {
    // Subway: 32×18, origin (5000, 5000)
    let cols: usize = 32;
    let rows: usize = 18;
    let mut grid = empty_grid(cols, rows);

    // rows 0-1: all solid
    for r in 0..=1 {
        grid[r] = solid_row(cols);
    }

    // rows 2-3: walls only
    for r in 2..=3 {
        set_walls(&mut grid, r, cols);
    }

    // row 4: walls + platforms at 6..=10, 22..=26
    set_walls(&mut grid, 4, cols);
    set_platform(&mut grid, 4, 6, 10);
    set_platform(&mut grid, 4, 22, 26);

    // rows 5-7: walls only
    for r in 5..=7 {
        set_walls(&mut grid, r, cols);
    }

    // row 8: walls + platforms at 13..=17
    set_walls(&mut grid, 8, cols);
    set_platform(&mut grid, 8, 13, 17);

    // rows 9-10: walls only
    for r in 9..=10 {
        set_walls(&mut grid, r, cols);
    }

    // row 11: walls + platforms at 4..=8, 24..=28
    set_walls(&mut grid, 11, cols);
    set_platform(&mut grid, 11, 4, 8);
    set_platform(&mut grid, 11, 24, 28);

    // rows 12-15: walls only
    for r in 12..=15 {
        set_walls(&mut grid, r, cols);
    }

    // rows 16-17: all solid
    for r in 16..=17 {
        grid[r] = solid_row(cols);
    }

    OutputLayer {
        id: 1,
        cols: cols as i32,
        rows: rows as i32,
        origin_x: 5000.0,
        origin_y: 5000.0,
        spawn: Some([5045.0, 5063.0]),
        tiles: grid,
        enemies: vec![],
        stars: vec![],
        health_foods: vec![],
        doors: vec![],
        props: vec![],
        lights: vec![],
        gate_col: None,
        exit_next_level: None,
        stars_required: None,
    }
}

fn city_layer2() -> OutputLayer {
    // Rooftop: 96×44, origin (-864, -200)
    let cols: usize = 96;
    let rows: usize = 44;
    let mut grid = empty_grid(cols, rows);

    // rows 0-2: all solid
    for r in 0..=2 {
        grid[r] = solid_row(cols);
    }

    // row 3: empty

    // row 4: platforms at 6..=11, 28..=33, 52..=57, 78..=83
    set_platform(&mut grid, 4, 6, 11);
    set_platform(&mut grid, 4, 28, 33);
    set_platform(&mut grid, 4, 52, 57);
    set_platform(&mut grid, 4, 78, 83);

    // row 5: empty

    // row 6: platforms at 10..=14, 35..=39, 60..=64, 86..=90
    set_platform(&mut grid, 6, 10, 14);
    set_platform(&mut grid, 6, 35, 39);
    set_platform(&mut grid, 6, 60, 64);
    set_platform(&mut grid, 6, 86, 90);

    // row 7: empty

    // row 8: platforms at 20..=25, 48..=53, 72..=77
    set_platform(&mut grid, 8, 20, 25);
    set_platform(&mut grid, 8, 48, 53);
    set_platform(&mut grid, 8, 72, 77);

    // row 9: empty

    // row 10: platforms at 15..=20, 38..=43, 65..=70, 85..=90
    set_platform(&mut grid, 10, 15, 20);
    set_platform(&mut grid, 10, 38, 43);
    set_platform(&mut grid, 10, 65, 70);
    set_platform(&mut grid, 10, 85, 90);

    // rows 11-13: empty

    // row 14: platforms at 25..=30, 55..=60
    set_platform(&mut grid, 14, 25, 30);
    set_platform(&mut grid, 14, 55, 60);

    // row 15: empty

    // row 16: platforms at 65..=70
    set_platform(&mut grid, 16, 65, 70);

    // row 17: empty

    // row 18: platforms at 12..=17, 42..=47, 72..=77
    set_platform(&mut grid, 18, 12, 17);
    set_platform(&mut grid, 18, 42, 47);
    set_platform(&mut grid, 18, 72, 77);

    // row 19: empty

    // row 20: platforms at 20..=25
    set_platform(&mut grid, 20, 20, 25);

    // row 21: empty

    // row 22: platforms at 30..=35, 60..=65
    set_platform(&mut grid, 22, 30, 35);
    set_platform(&mut grid, 22, 60, 65);

    // rows 23-25: empty

    // row 26: platforms at 20..=25, 50..=55
    set_platform(&mut grid, 26, 20, 25);
    set_platform(&mut grid, 26, 50, 55);

    // row 27: empty

    // row 28: platforms at 32..=37, 60..=65
    set_platform(&mut grid, 28, 32, 37);
    set_platform(&mut grid, 28, 60, 65);

    // row 29: empty

    // row 30: platforms at 40..=45, 70..=75
    set_platform(&mut grid, 30, 40, 45);
    set_platform(&mut grid, 30, 70, 75);

    // rows 31-43: empty

    OutputLayer {
        id: 2,
        cols: cols as i32,
        rows: rows as i32,
        origin_x: -864.0,
        origin_y: -200.0,
        spawn: Some([-783.0, -128.0]),
        tiles: grid,
        enemies: vec![],
        stars: vec![],
        health_foods: vec![],
        doors: vec![],
        props: vec![],
        lights: vec![],
        gate_col: None,
        exit_next_level: None,
        stars_required: None,
    }
}

fn city_level() -> OutputLevel {
    OutputLevel {
        id: "City".to_string(),
        layers: vec![city_layer0(), city_layer1(), city_layer2()],
    }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() -> anyhow::Result<()> {
    // Resolve the workspace root from this binary's manifest directory.
    // CARGO_MANIFEST_DIR is set at compile time to the ldtk_compiler crate
    // directory; two levels up reaches the workspace root.
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_root = std::path::Path::new(manifest_dir)
        .parent() // tools/
        .and_then(|p| p.parent()) // workspace root
        .expect("could not resolve workspace root from CARGO_MANIFEST_DIR");

    let output_path = workspace_root
        .join("assets")
        .join("levels")
        .join("compiled_levels.json");

    println!("bootstrap: building compiled_levels.json");
    println!("  Output: {}", output_path.display());

    let root = OutputRoot {
        schema_version: 1,
        levels: vec![forest_level(), subdivision_level(), city_level()],
    };

    // Print summary before writing
    for level in &root.levels {
        println!(
            "  Level '{}': {} layer(s)",
            level.id,
            level.layers.len()
        );
        for layer in &level.layers {
            let tile_count: usize = layer
                .tiles
                .iter()
                .flat_map(|row| row.iter())
                .filter(|&&t| t != 0)
                .count();
            println!(
                "    Layer {}: {}×{} grid, {} non-zero tiles, {} enemies, {} stars, {} health_foods, {} doors",
                layer.id,
                layer.cols,
                layer.rows,
                tile_count,
                layer.enemies.len(),
                layer.stars.len(),
                layer.health_foods.len(),
                layer.doors.len(),
            );
        }
    }

    write_atomic(&root, &output_path)?;

    println!("Done. Written to {}", output_path.display());
    Ok(())
}
