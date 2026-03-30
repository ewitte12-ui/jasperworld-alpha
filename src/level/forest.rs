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

// ── Layer 1: Underground Cave/Burrow (single-screen) ─────────────────────────
// 32 cols × 18 rows, origin (0, 0) — independent from surface layer.
// Fully enclosed: solid floor (rows 0-1), walls (col 0/31), ceiling (rows 16-17).
// Organic platform layout for vertical exploration.
fn forest_layer_1() -> LayerData {
    let tiles: Vec<Vec<TileType>> = {
        let solid = || vec![S; 32];
        let cave = |platforms: &[(usize, usize)]| {
            let mut row = vec![E; 32];
            row[0] = S;
            row[31] = S;
            for &(start, end) in platforms {
                row[start..=end].fill(P);
            }
            row
        };
        vec![
            solid(),                                // row 0  — floor
            solid(),                                // row 1  — floor
            cave(&[]),                              // row 2
            cave(&[]),                              // row 3
            cave(&[]),                              // row 4
            cave(&[(5, 9), (22, 26)]),              // row 5  — low platforms
            cave(&[]),                              // row 6
            cave(&[]),                              // row 7
            cave(&[]),                              // row 8
            cave(&[(12, 16)]),                      // row 9  — mid platform
            cave(&[]),                              // row 10
            cave(&[]),                              // row 11
            cave(&[(20, 24)]),                      // row 12 — high platform
            cave(&[]),                              // row 13
            cave(&[]),                              // row 14
            cave(&[]),                              // row 15
            solid(),                                // row 16 — ceiling
            solid(),                                // row 17 — ceiling
        ]
    };

    LayerData {
        id: 1,
        tiles,
        // Origin far from main level so no surface decorations/backgrounds bleed in.
        origin_x: 5000.0,
        origin_y: 5000.0,
        // Spawn on floor, near left side: col 2 = 5000 + 2*18 + 9 = 5045
        spawn: Vec2::new(5045.0, 5063.0),
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
