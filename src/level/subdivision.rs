use bevy::prelude::*;
use crate::tilemap::tilemap::TileType::{self, Empty as E, Platform as P, Solid as S};
use super::level_data::{LayerData, LevelData, LevelId};

pub fn subdivision_level() -> LevelData {
    LevelData {
        id: LevelId::Subdivision,
        layers: vec![
            subdivision_layer_0(),
            subdivision_layer_1(),
            subdivision_layer_2(),
        ],
    }
}

// ── Layer 0: Street ──────────────────────────────────────────────────────────
// 96 cols × 22 rows, origin_x = -864.0, origin_y = -200.0
// rows 0-2: solid ground (sidewalk/road)
// row 6: platforms representing fences, mailboxes, car roofs, awnings
// row 10: higher obstacles — porch roofs, truck tops
// row 14: high awnings
fn subdivision_layer_0() -> LayerData {
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
            plat(&[(5, 9), (20, 25), (38, 43), (56, 61), (72, 77), (85, 89)]), // row 6
            empty(), // row 7
            empty(), // row 8
            empty(), // row 9
            plat(&[(12, 17), (28, 33), (45, 50), (63, 68), (76, 81)]), // row 10
            empty(), // row 11
            empty(), // row 12
            empty(), // row 13
            plat(&[(30, 34), (65, 69)]), // row 14
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

// ── Layer 1: Sewers (single-screen) ──────────────────────────────────────────
// 32 cols × 18 rows, origin (0, 0) — independent from surface layer.
// Fully enclosed: solid floor, walls, ceiling.
// Wider platforms for horizontal traversal.
fn subdivision_layer_1() -> LayerData {
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
            cave(&[(4, 8), (20, 24)]),              // row 5  — low platforms
            cave(&[]),                              // row 6
            cave(&[]),                              // row 7
            cave(&[]),                              // row 8
            cave(&[(10, 14), (25, 29)]),            // row 9  — mid platforms
            cave(&[]),                              // row 10
            cave(&[]),                              // row 11
            cave(&[(15, 19)]),                      // row 12 — high platform
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
        origin_x: 5000.0,
        origin_y: 5000.0,
        spawn: Vec2::new(5045.0, 5063.0),
    }
}

// ── Layer 2: Rooftop ─────────────────────────────────────────────────────────
// 96 cols × 22 rows, origin_x = -864.0, origin_y = -200.0
// rows 0-2: solid roof surface
// row 6: chimneys, vents
// row 10: satellite dishes, skylights
// row 14: antenna masts
fn subdivision_layer_2() -> LayerData {
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
            plat(&[(8, 12), (30, 34), (60, 64), (86, 90)]), // row 6
            empty(), // row 7
            empty(), // row 8
            empty(), // row 9
            plat(&[(18, 22), (24, 29), (50, 54), (66, 71), (80, 84)]), // row 10
            empty(), // row 11
            empty(), // row 12
            empty(), // row 13
            plat(&[(26, 30), (68, 72)]), // row 14
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
