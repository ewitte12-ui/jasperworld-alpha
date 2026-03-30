use bevy::prelude::*;
use crate::tilemap::tilemap::TileType::{self, Empty as E, Platform as P, Solid as S};
use super::level_data::{LayerData, LevelData, LevelId};

pub fn city_level() -> LevelData {
    LevelData {
        id: LevelId::City,
        layers: vec![
            city_layer_0(),
            city_layer_1(),
            city_layer_2(),
        ],
    }
}

// ── Layer 0: Street ──────────────────────────────────────────────────────────
// 96 cols × 44 rows, origin_x = -864.0, origin_y = -200.0
// rows 0-2: solid ground (sidewalk)
// Platforms at rows 4,6,8,10,14,16,18,20,22,24,26,28,30
// Rows 16/20/24/28 are stepping-stone bridges ensuring every platform is
// reachable with jump_height=90 (5 tiles) and run_speed=200 (~144 unit
// max horizontal gap during a 4-row jump).
fn city_layer_0() -> LayerData {
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
            empty(), // row 4  — (fire escapes removed)
            empty(), // row 5
            plat(&[(5, 9), (20, 25), (38, 43), (56, 61), (72, 77), (85, 89)]), // row 6  — awnings
            empty(), // row 7
            plat(&[(12, 17), (42, 47), (68, 73)]),                    // row 8  — mid scaffolding
            empty(), // row 9
            plat(&[(10, 15), (28, 33), (50, 55), (70, 75), (85, 90)]), // row 10 — mid platforms
            empty(), // row 11
            empty(), // row 12
            empty(), // row 13
            plat(&[(18, 23), (40, 45), (62, 67), (82, 87)]),          // row 14 — high platforms
            empty(), // row 15
            plat(&[(48, 53)]),                                         // row 16 — bridge: 14(40-45)→18(55-60)
            empty(), // row 17
            plat(&[(25, 30), (55, 60), (78, 83)]),                    // row 18 — fire escapes + right bridge for 14(82-87)
            empty(), // row 19
            plat(&[(65, 70)]),                                         // row 20 — bridge: 18(55-60)→22(75-80)
            empty(), // row 21
            plat(&[(15, 20), (45, 50), (75, 80)]),                    // row 22 — upper scaffolding
            empty(), // row 23
            plat(&[(22, 27), (52, 57), (70, 75)]),                    // row 24 — bridges: 22→26 (all three paths)
            empty(), // row 25
            plat(&[(30, 35), (60, 65), (78, 83)]),                    // row 26 — near-top ledges + right bridge
            empty(), // row 27
            empty(), // row 28
            empty(), // row 29
            plat(&[(20, 25), (50, 55), (80, 85)]),                    // row 30 — top platforms
            empty(), // row 31
            empty(), // row 32
            empty(), // row 33
            empty(), // row 34
            empty(), // row 35
            empty(), // row 36
            empty(), // row 37
            empty(), // row 38
            empty(), // row 39
            empty(), // row 40
            empty(), // row 41
            empty(), // row 42
            empty(), // row 43
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

// ── Layer 1: Subway ──────────────────────────────────────────────────────────
// 96 cols × 44 rows, origin_x = -864.0, origin_y = -200.0
// rows 0-1: solid floor
// rows 2-34: walls at cols 0/95, interior with platforms
// rows 35-43: solid ceiling
fn city_layer_1() -> LayerData {
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
            cave(&[]),                                         // row 2
            cave(&[]),                                         // row 3
            cave(&[(10, 15), (35, 40), (60, 65)]),             // row 4
            cave(&[]),                                         // row 5
            cave(&[(5, 10), (20, 25), (48, 53), (70, 75)]),    // row 6
            cave(&[]),                                         // row 7
            cave(&[(15, 20), (40, 45), (65, 70)]),             // row 8
            cave(&[]),                                         // row 9
            cave(&[(22, 27), (55, 60), (80, 85)]),             // row 10
            cave(&[]),                                         // row 11
            cave(&[]),                                         // row 12
            cave(&[]),                                         // row 13
            cave(&[(12, 17), (45, 50), (72, 77)]),             // row 14
            cave(&[]),                                         // row 15
            cave(&[(22, 27), (52, 57)]),                       // row 16 — bridges: 14(12-17)→18(30-35), 14(45-50)→18(60-65)
            cave(&[]),                                         // row 17
            cave(&[(30, 35), (60, 65)]),                       // row 18
            cave(&[]),                                         // row 19
            cave(&[]),                                         // row 20
            cave(&[]),                                         // row 21
            cave(&[]),                                         // row 22
            cave(&[]),                                         // row 23
            cave(&[]),                                         // row 24
            cave(&[]),                                         // row 25
            cave(&[]),                                         // row 26
            cave(&[]),                                         // row 27
            cave(&[]),                                         // row 28
            cave(&[]),                                         // row 29
            cave(&[]),                                         // row 30
            cave(&[]),                                         // row 31
            cave(&[]),                                         // row 32
            cave(&[]),                                         // row 33
            cave(&[]),                                         // row 34
            solid(), // row 35
            solid(), // row 36
            solid(), // row 37
            solid(), // row 38
            solid(), // row 39
            solid(), // row 40
            solid(), // row 41
            solid(), // row 42
            solid(), // row 43
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

// ── Layer 2: Rooftop ─────────────────────────────────────────────────────────
// 96 cols × 44 rows, origin_x = -864.0, origin_y = -200.0
// rows 0-2: solid roof surface
// Platforms at rows 4,6,8,10,14,18,22,26,30 — AC units, antenna mounts, water towers
fn city_layer_2() -> LayerData {
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
            plat(&[(6, 11), (28, 33), (52, 57), (78, 83)]),           // row 4  — AC units
            empty(), // row 5
            plat(&[(10, 14), (35, 39), (60, 64), (86, 90)]),          // row 6  — vents
            empty(), // row 7
            plat(&[(20, 25), (48, 53), (72, 77)]),                    // row 8  — antenna bases
            empty(), // row 9
            plat(&[(15, 20), (38, 43), (65, 70), (85, 90)]),          // row 10 — water towers
            empty(), // row 11
            empty(), // row 12
            empty(), // row 13
            plat(&[(25, 30), (55, 60)]),                              // row 14 — antenna masts
            empty(), // row 15
            plat(&[(65, 70)]),                                         // row 16 — bridge: 14(55-60)→18(72-77)
            empty(), // row 17
            plat(&[(12, 17), (42, 47), (72, 77)]),                    // row 18 — upper structures
            empty(), // row 19
            plat(&[(20, 25)]),                                         // row 20 — bridge: 18(12-17)→22(30-35)
            empty(), // row 21
            plat(&[(30, 35), (60, 65)]),                              // row 22 — high platforms
            empty(), // row 23
            empty(), // row 24
            empty(), // row 25
            plat(&[(20, 25), (50, 55)]),                              // row 26 — near-top
            empty(), // row 27
            plat(&[(32, 37), (60, 65)]),                              // row 28 — bridges: 26(20-25)→30(40-45), 26(50-55)→30(70-75)
            empty(), // row 29
            plat(&[(40, 45), (70, 75)]),                              // row 30 — top platforms
            empty(), // row 31
            empty(), // row 32
            empty(), // row 33
            empty(), // row 34
            empty(), // row 35
            empty(), // row 36
            empty(), // row 37
            empty(), // row 38
            empty(), // row 39
            empty(), // row 40
            empty(), // row 41
            empty(), // row 42
            empty(), // row 43
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
