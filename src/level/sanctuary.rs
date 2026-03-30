use bevy::prelude::*;

use crate::tilemap::tilemap::TileType::{self, Empty as E, Platform as P, Solid as S};

use super::level_data::{LayerData, LevelData, LevelId};

pub fn sanctuary_level() -> LevelData {
    LevelData {
        id: LevelId::Sanctuary,
        layers: vec![
            sanctuary_layer_0(), // meadow
            sanctuary_layer_1(), // underground roots
            sanctuary_layer_2(), // hilltop/ruins
        ],
    }
}

// ── Layer 0: Sanctuary Meadow ─────────────────────────────────────────────────
//
// 160 cols × 22 rows
// Row 0-2:  solid ground
// Row 3:    empty
// Row 4:    P at 6..=9, 14..=18, 98..=101, 103..=105
// Row 5:    P at 35..=37, 67..=72, 76..=78, 80..=85, 138..=143
// Row 6:    P at 48..=53
// Row 7:    P at 21..=26, 104..=108
// Row 8:    P at 74..=78, 110..=115, 130..=134, 146..=151
// Row 9:    P at 40..=43
// Row 10:   P at 55..=59
// Row 11:   empty
// Row 12:   P at 70..=74, 100..=104
// Row 13:   empty
// Row 14:   P at 106..=110
// Rows 15-21: empty
fn sanctuary_layer_0() -> LayerData {
    let solid = || vec![S; 160];
    let empty = || vec![E; 160];
    let plat = |platforms: &[(usize, usize)]| {
        let mut row = vec![E; 160];
        for &(start, end) in platforms {
            row[start..=end].fill(P);
        }
        row
    };

    let tiles: Vec<Vec<TileType>> = vec![
        solid(),                                                                     // row 0
        solid(),                                                                     // row 1
        solid(),                                                                     // row 2
        empty(),                                                                     // row 3
        plat(&[(6, 9), (14, 18), (98, 101), (103, 105)]),                           // row 4
        plat(&[(35, 37), (67, 72), (76, 78), (80, 85), (138, 143)]),                // row 5
        plat(&[(48, 53)]),                                                           // row 6
        plat(&[(21, 26), (104, 108)]),                                               // row 7
        plat(&[(74, 78), (110, 115), (130, 134), (146, 151)]),                      // row 8
        plat(&[(40, 43)]),                                                           // row 9
        plat(&[(55, 59)]),                                                           // row 10
        empty(),                                                                     // row 11
        plat(&[(70, 74), (100, 104)]),                                               // row 12
        empty(),                                                                     // row 13
        plat(&[(106, 110)]),                                                         // row 14
        empty(),                                                                     // row 15
        empty(),                                                                     // row 16
        empty(),                                                                     // row 17
        empty(),                                                                     // row 18
        empty(),                                                                     // row 19
        empty(),                                                                     // row 20
        empty(),                                                                     // row 21
    ];

    LayerData {
        id: 0,
        tiles,
        origin_x: -1440.0,
        origin_y: -200.0,
        spawn: Vec2::new(-1395.0, -128.0),
    }
}

// ── Layer 1: Underground Roots ────────────────────────────────────────────────
//
// 160 cols × 22 rows
// Rows 0-1:   solid floor
// Rows 2-12:  cave (S col 0, E interior, S col 159)
//   row 6:  P at 5..=9, 40..=44, 75..=79, 125..=129
//   row 10: P at 19..=23, 64..=68, 105..=109
// Rows 13-21: solid ceiling
fn sanctuary_layer_1() -> LayerData {
    let solid = || vec![S; 160];
    let cave = |platforms: &[(usize, usize)]| {
        let mut row = vec![E; 160];
        row[0] = S;
        row[159] = S;
        for &(start, end) in platforms {
            row[start..=end].fill(P);
        }
        row
    };

    let tiles: Vec<Vec<TileType>> = vec![
        solid(),                                                                     // row 0
        solid(),                                                                     // row 1
        cave(&[]),                                                                   // row 2
        cave(&[]),                                                                   // row 3
        cave(&[]),                                                                   // row 4
        cave(&[]),                                                                   // row 5
        cave(&[(5, 9), (40, 44), (75, 79), (125, 129)]),                            // row 6
        cave(&[]),                                                                   // row 7
        cave(&[]),                                                                   // row 8
        cave(&[]),                                                                   // row 9
        cave(&[(19, 23), (64, 68), (105, 109)]),                                    // row 10
        cave(&[]),                                                                   // row 11
        cave(&[]),                                                                   // row 12
        solid(),                                                                     // row 13
        solid(),                                                                     // row 14
        solid(),                                                                     // row 15
        solid(),                                                                     // row 16
        solid(),                                                                     // row 17
        solid(),                                                                     // row 18
        solid(),                                                                     // row 19
        solid(),                                                                     // row 20
        solid(),                                                                     // row 21
    ];

    LayerData {
        id: 1,
        tiles,
        origin_x: -1440.0,
        origin_y: -200.0,
        spawn: Vec2::new(-1395.0, -155.0),
    }
}

// ── Layer 2: Hilltop / Ruins ──────────────────────────────────────────────────
//
// 160 cols × 22 rows
// Rows 0-2:   solid
// Rows 3-5:   empty
// Row 6:      P at 10..=14, 42..=46, 77..=81, 117..=121
// Rows 7-9:   empty
// Row 10:     P at 25..=29, 70..=74, 111..=115
// Rows 11-13: empty
// Row 14:     P at 45..=49, 105..=109
// Rows 15-21: empty
fn sanctuary_layer_2() -> LayerData {
    let solid = || vec![S; 160];
    let empty = || vec![E; 160];
    let plat = |platforms: &[(usize, usize)]| {
        let mut row = vec![E; 160];
        for &(start, end) in platforms {
            row[start..=end].fill(P);
        }
        row
    };

    let tiles: Vec<Vec<TileType>> = vec![
        solid(),                                                                     // row 0
        solid(),                                                                     // row 1
        solid(),                                                                     // row 2
        empty(),                                                                     // row 3
        empty(),                                                                     // row 4
        empty(),                                                                     // row 5
        plat(&[(10, 14), (42, 46), (77, 81), (117, 121)]),                          // row 6
        empty(),                                                                     // row 7
        empty(),                                                                     // row 8
        empty(),                                                                     // row 9
        plat(&[(25, 29), (70, 74), (111, 115)]),                                    // row 10
        empty(),                                                                     // row 11
        empty(),                                                                     // row 12
        empty(),                                                                     // row 13
        plat(&[(45, 49), (105, 109)]),                                               // row 14
        empty(),                                                                     // row 15
        empty(),                                                                     // row 16
        empty(),                                                                     // row 17
        empty(),                                                                     // row 18
        empty(),                                                                     // row 19
        empty(),                                                                     // row 20
        empty(),                                                                     // row 21
    ];

    LayerData {
        id: 2,
        tiles,
        origin_x: -1440.0,
        origin_y: -200.0,
        spawn: Vec2::new(-1395.0, -128.0),
    }
}
