use bevy::prelude::*;
use crate::tilemap::tilemap::TileType::{self, Empty as E, Platform as P, Solid as S};
use super::level_data::{LayerData, LevelData, LevelId};

pub fn forest_level() -> LevelData {
    LevelData {
        id: LevelId::Forest,
        layers: vec![
            forest_layer_0(),
            forest_layer_1(),
            forest_layer_2(),
        ],
    }
}

// ── Layer 0: Surface Forest ───────────────────────────────────────────────────
// 96 cols × 22 rows, origin_x = -864.0, origin_y = -200.0
// rows 0-2: solid ground
// rows 3-5: open air
// row 6: platforms at 4..=8, 22..=26, 35..=39, 55..=60, 67..=71, 83..=87
// rows 7-9: open air
// row 10: platforms at 13..=18, 44..=46, 75..=80
//   Platform E trimmed to cols 44–46 (was 44–49).
//   WHY: cols 47–49 removed to break the elevated bypass over the Dog zone.
//   Old route: Plat D (row6, 35–39) → Plat E (row10, 44–49) → Plat F (row6, 55–60)
//   With the gap at cols 47–54 (row 10), the player lands on Plat E but cannot
//   reach Plat F without dropping to the ground (col ~46 → Dog zone col 41–53).
//   Star at col 46 is still on the trimmed platform.
//   Row-14 detour (cols 48–52, 4 rows up, 2 cols right from col 46) still reachable.
// rows 11-13: open air
// row 14: platforms at 48..=52, 68..=72
// rows 15-21: open sky
fn forest_layer_0() -> LayerData {
    let tiles: Vec<Vec<TileType>> = {
        let solid = || vec![S; 96];
        let empty = || vec![E; 96];
        let plat = |platforms: &[(usize, usize)]| {
            let mut row = vec![E; 96];
            for &(start, end) in platforms {
                row[start..=end].fill(P);
            }
            row
        };
        vec![
            solid(), // row 0
            solid(), // row 1
            solid(), // row 2
            empty(), // row 3
            empty(), // row 4
            empty(), // row 5
            plat(&[(4, 8), (22, 26), (35, 39), (55, 60), (67, 71), (83, 87)]), // row 6
            empty(), // row 7
            empty(), // row 8
            empty(), // row 9
            plat(&[(13, 18), (44, 46), (75, 80)]), // row 10 — Plat E trimmed; gap 47–54 forces ground drop into Dog zone
            empty(), // row 11
            empty(), // row 12
            empty(), // row 13
            plat(&[(48, 52), (68, 72)]), // row 14
            empty(), // row 15
            empty(), // row 16
            empty(), // row 17
            empty(), // row 18
            empty(), // row 19
            empty(), // row 20
            empty(), // row 21
        ]
    };

    LayerData {
        id: 0,
        tiles,
        origin_x: -864.0,
        origin_y: -200.0,
        spawn: Vec2::new(-819.0, -128.0),
    }
}

// ── Layer 1: Underground Burrow ───────────────────────────────────────────────
// 96 cols × 22 rows, origin_x = -864.0, origin_y = -200.0
// rows 0-1: solid floor
// rows 2-12: walls (S at col 0 and col 95), interior E
//   row 6 special: S col 0, E cols 1-2, P cols 3-7, E cols 8-44, P cols 45-49, E cols 50-94, S col 95
//   row 10 special: S col 0, E cols 1-20, P cols 21-25, E cols 26-69, P cols 70-74, E cols 75-94, S col 95
// rows 13-21: solid ceiling
fn forest_layer_1() -> LayerData {
    let tiles: Vec<Vec<TileType>> = {
        let solid = || vec![S; 96];
        let cave = |platforms: &[(usize, usize)]| {
            let mut row = vec![E; 96];
            row[0] = S;
            row[95] = S;
            for &(start, end) in platforms {
                row[start..=end].fill(P);
            }
            row
        };
        vec![
            solid(), // row 0
            solid(), // row 1
            cave(&[]),                              // row 2
            cave(&[]),                              // row 3
            cave(&[]),                              // row 4
            cave(&[]),                              // row 5
            cave(&[(3, 7), (45, 49)]),              // row 6 — platforms at 3..=7 and 45..=49
            cave(&[]),                              // row 7
            cave(&[]),                              // row 8
            cave(&[]),                              // row 9
            cave(&[(21, 25), (70, 74)]),            // row 10 — platforms at 21..=25 and 70..=74
            cave(&[]),                              // row 11
            cave(&[]),                              // row 12
            solid(), // row 13
            solid(), // row 14
            solid(), // row 15
            solid(), // row 16
            solid(), // row 17
            solid(), // row 18
            solid(), // row 19
            solid(), // row 20
            solid(), // row 21
        ]
    };

    LayerData {
        id: 1,
        tiles,
        origin_x: -864.0,
        origin_y: -200.0,
        spawn: Vec2::new(-819.0, -155.0),
    }
}

// ── Layer 2: Treetop Canopy ───────────────────────────────────────────────────
// 96 cols × 22 rows, origin_x = -864.0, origin_y = -200.0
// rows 0-2: solid ground
// rows 3-5: open air
// row 6: platforms at 6..=10, 32..=36, 62..=66, 88..=91
// rows 7-9: open air
// row 10: platforms at 16..=20, 48..=52, 78..=82
// rows 11-13: open air
// row 14: platforms at 25..=29, 65..=69
// rows 15-21: open sky
fn forest_layer_2() -> LayerData {
    let tiles: Vec<Vec<TileType>> = {
        let solid = || vec![S; 96];
        let empty = || vec![E; 96];
        let plat = |platforms: &[(usize, usize)]| {
            let mut row = vec![E; 96];
            for &(start, end) in platforms {
                row[start..=end].fill(P);
            }
            row
        };
        vec![
            solid(), // row 0
            solid(), // row 1
            solid(), // row 2
            empty(), // row 3
            empty(), // row 4
            empty(), // row 5
            plat(&[(6, 10), (32, 36), (62, 66), (88, 91)]), // row 6
            empty(), // row 7
            empty(), // row 8
            empty(), // row 9
            plat(&[(16, 20), (48, 52), (78, 82)]), // row 10
            empty(), // row 11
            empty(), // row 12
            empty(), // row 13
            plat(&[(25, 29), (65, 69)]), // row 14
            empty(), // row 15
            empty(), // row 16
            empty(), // row 17
            empty(), // row 18
            empty(), // row 19
            empty(), // row 20
            empty(), // row 21
        ]
    };

    LayerData {
        id: 2,
        tiles,
        origin_x: -864.0,
        origin_y: -200.0,
        spawn: Vec2::new(-783.0, -128.0),
    }
}
