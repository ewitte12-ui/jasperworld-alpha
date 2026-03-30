use bevy::math::Vec2;

use crate::tilemap::tilemap::TileType::{self, Empty as E, Platform as P, Solid as S};

use super::level_data::{LayerData, LevelData, LevelId};

pub fn subdivision_level() -> LevelData {
    LevelData {
        id: LevelId::Subdivision,
        layers: vec![
            subdivision_layer_0(), // street level
            subdivision_layer_1(), // rooftops
            subdivision_layer_2(), // basement / sewers
        ],
    }
}

// ── Layer 0: Street Level — SPINE + GROUND TRAVERSAL ─────────────────────────
//
// 96 cols × 22 rows  |  origin_x = -864.0  |  origin_y = -200.0
// Spawn: (-819.0, -128.0)
//
// Design intent: "Space built for someone else, not for the player."
// No symmetric ladder patterns. Each jump requires aim, not just timing.
// Gaps and height changes vary independently — long gap / small step,
// short gap / tall step, same height / long gap, etc.
//
// Height bands:
//   LOW  = rows 3–5  (1–3 rows above ground)
//   MID  = rows 6–7  (4–5 rows above ground)
//   HIGH = rows 8–10 (6–8 rows above ground)
//
// ── Spine chain ──────────────────────────────────────────────────────────────
//
//   Ground(row2) → A(row5) → B(row8) → C(row5) → D(row8)
//               → E(row10) ──[DROP 108u]──► F(row4) → F_mid(row6)
//               → G(row9)  → H(row6) → I(row3) → J(row7)
//               → [jump entry +36u]  → VRT-Base(row10) → VRT-T1(row12) → VRT-T2(row14) → VRT-Peak(row16)
//               → L(row7)  → Gate
//
// ── Screen 1 (cols 0–31): asymmetric staircase ───────────────────────────────
//   Ground  cols  0–2    Two-tile intro — decision visible immediately
//   A row5  cols  3–7    UP 3 rows from ground, wide (5 tiles)
//   B row8  cols 12–15   UP 3 rows from A, GAP 4 cols — long horizontal, narrow (4 tiles)
//   C row5  cols 18–20   DOWN 3 rows from B, GAP 2 cols — short gap, SAME HEIGHT as A (3 tiles)
//                         W2 wall immediately right of C forces the jump launch to D.
//   D row8  cols 26–31   UP 3 rows from C, GAP 5 cols — demanding diagonal, wide landing (6 tiles)
//                         Same height as B — subtle echo, not staircase.
//
// Jump pattern Screen 1: (UP3,GAP4) → (DOWN3,GAP2) → (UP3,GAP5)
//   Long-horizontal-small-step, short-horizontal-tall-fall, demanding-diagonal.
//   Each transition has a different ratio of vertical to horizontal distance.
//
// ── Screen 2 (cols 32–63): chimney → FORCED DROP → recovery ─────────────────
//   E row10 cols 32–35   UP 2 rows from D, adjacent — chimney presents immediately (4 tiles)
//                         Drop shaft cols 36–37 is empty — fall is uninterrupted.
//   F row4  cols 38–42   ★ FORCED DROP LANDING — 6 rows below E = 108 u.
//                         IRREVERSIBLE: max jump = 90 u. No path back to E.
//                         W4 wall at col43 immediately right of F: "you must climb from here."
//   F_mid row6 cols44–48  UP 2 rows from F, over W4 (5 tiles)
//   G row9  cols 51–55   UP 3 rows from F_mid, GAP 2 (5 tiles)
//   H row6  cols 58–62   DOWN 3 rows from G, GAP 2 (5 tiles)
//
// ── Screen 3 (cols 64–95): near-ground entry → VERTICAL RECOVERY TOWER → gate ─
//   I row3  cols 65–68   DOWN 3 from H — near-ground pause (4 tiles)
//                         W5 (3-tile wall) blocks leftward reversal from I.
//   J row8  cols 72–75   UP 5 rows from I, GAP 3 — demanding diagonal, at jump envelope limit (4 tiles)
//                         Height matches B/D (all row8): Screen 3 spine echoes Screen 1–2 height.
//
// ── VERTICAL RECOVERY TOWER (VRT) — cols 75–84 ───────────────────────────────
//   Entry: walk right off J(row8) — no platform at row8 past col75; Base(row10) is 2 rows above.
//          Jump entry (+36 u). Tower climb begins at Base.
//
//   VRT-Base row10 cols 75–77  ENTRY — 2 rows above J (3 tiles)
//   VRT-T1   row12 cols 79–80  UP 2 rows, RIGHT 2 cols = 36u+36u (2 tiles)  zig →
//   VRT-T2   row14 cols 76–77  UP 2 rows, LEFT  3 cols = 36u+54u (2 tiles)  ← zag
//   VRT-Peak row16 cols 79–81  UP 2 rows, RIGHT 2 cols = 36u+36u (3 tiles)  zig →
//
//   Zig-zag pattern: RIGHT / LEFT / RIGHT — no two tiers share a vertical axis.
//   Total climb: row10 → row16 = 6 rows = 108 u.
//   Each step: ≤36u vertical, ≤54u horizontal — within jump envelope, requires aim.
//
//   Falling consequence:
//     Fall from VRT-Base(row10): ground below (8 rows/144 u); reclimb via J(row8) → Base(row10).
//     Fall from VRT-T1(row12, cols79–80): ground below; reclimb via J → Base.
//     Fall from VRT-T2(row14, cols76–77): lands on VRT-Base(row10, cols75–77); reclimb from Base.
//     Fall from VRT-Peak(row16, cols79–80): may land on VRT-T1(row12); 2-step reclimb.
//     Recovery path is always accessible, never instant, never automatic.
//
//   W7 wall (cols81–82, rows3–5): VRT zone boundary — entirely below all tower tiers.
//   W7 top (row5) is 1 row below Base (row6). No clearance concern from Base to T1.
//
//   L row7  cols 83–90   exit from VRT-Peak — DROP 3 rows, GAP 2 cols (8 tiles)
//                         Wide landing; gate visible from VRT-Peak.
//   Gate:   col  91      Unjumpable — LevelExit beyond
//
// ── Ground-level blockers ─────────────────────────────────────────────────────
//
//   FENCE (rows 3–4, 36 u): hop from ground — slows pace, doesn't stop progress
//   WALL  (rows 3–5, 54 u): near-impassable from ground; trivial from spine platform
//
//   W1 fence cols  8– 9  (row3–4): first interruption after spawn
//   W2 wall  cols 21–22  (row3–5): forces jump launch from C toward D
//   W3 fence cols 24–25  (row3–4): second obstacle before D — double-fence zone
//   W4 wall  cols 43–44  (row3–5): drop-recovery signal; F(row4) must jump over
//   W5 wall  cols 63–64  (row3–5): blocks I→H reversal path
//   W6 fence cols 69–70  (row3–4): interrupts ground approach to J
//   W7 wall  cols 81–82  (row3–5): VRT zone boundary; separates Base from T1 approach
//
// Doors:
//   Door → Layer 1 (Rooftops): col_x(27) ≈ -369.0  (platform D, wide landing)
//   Door → Layer 2 (Sewers):   col_x(46) ≈  -27.0  (platform F_mid)
fn subdivision_layer_0() -> LayerData {
    let solid = || vec![S; 96];
    let empty = || vec![E; 96];
    let plat = |platforms: &[(usize, usize)]| {
        let mut row = vec![E; 96];
        for &(start, end) in platforms {
            row[start..=end].fill(P);
        }
        row
    };
    // Mixed row: Solid walls at `walls`, Platform spine at `plats`.
    // No two ranges within the same row may overlap.
    let mixed = |walls: &[(usize, usize)], plats: &[(usize, usize)]| {
        let mut row = vec![E; 96];
        for &(start, end) in walls { row[start..=end].fill(S); }
        for &(start, end) in plats { row[start..=end].fill(P); }
        row
    };

    // ── Platform index ────────────────────────────────────────────────────────
    //
    //  SPINE (mandatory traversal chain):
    //    row4:  F(38–42)  I(65–68)   [forced-drop landing | collectible re-entry (2 rows from ground)]
    //    row5:  A(3–7)  C(18–20)
    //    row6:  F_mid(44–48)  H(58–62)  R3b(74–75)  R4b(82–83)
    //    row7:  R1b(26–27)
    //    row8:  B(12–15)  D(26–31)  R2b(49–51)  NB2(57)  J(72–75)  L(83–90)
    //    row10: OA(8–10)  NB1(23)  E(32–35) [chimney]  G(51–55)  VRT-Base(75–77) [tower entry]
    //    row12: VRT-T1(79–80) [tower zig]
    //    row14: VRT-T2(76–77) [tower zag]
    //    row16: VRT-Peak(79–81) [tower summit]
    //
    //  OPTIONAL ROUTE (one navigational branch, no reward):
    //    OA row9 (8–10) — high bypass between A and B.
    //      From A(row5): +4 rows = 72 u, 1-col gap — deliberate upward aim required.
    //      From OA:      -1 row  = 18 u, 2-col gap to B(row8,12–15) — trivial drop.
    //      Visible from A: sits above W1 fence (rows3–4), clearly higher than B.
    //      Reversible: drop back to A's zone or proceed to B.
    //
    //  NARROW COMMITMENT GEOMETRY (no reward — teaches precision landing):
    //    NB1 row7  (23)     — 1 tile between W2/W3 walls; optional C→D waypoint.
    //                         From C(row5): +2 rows = 36 u, 3-col gap.
    //                         From NB1 to D(row8): +1 row = 18 u, 3-col gap.
    //    NB2 row8  (57)     — 1 tile just left of H; descent step between G and H.
    //                         From G(row9): -1 row = 18 u, 2-col gap.
    //                         From NB2 to H(row6): -2 rows = 36 u, 1-col gap.
    //    NB3 row6  (69–70)  — 2 rows above I(row4); I→J mid-perch.
    //                         From I(row4): +2 rows = 36 u, 1–2-col gap.
    //                         From NB3 to J(row8): +2 rows = 36 u, 2-col gap.
    //
    //  RECOVERY CLUSTERS (zig-zag ascent from ground back to spine):
    //    Each cluster = low step (row4) + mid step (row6 or row7, offset L or R) → spine.
    //    Offset direction alternates so no cluster reads as a straight ladder.
    //
    //    R1 — Screen 1, approach to D (player fell past C):
    //      R1a row4 (28–29)  ← ground hop, 2 rows = 36 u
    //      R1b row7 (26–27)  ← offset LEFT  3 rows = 54 u  zig
    //      → D  row8 (26–31)   1 row  = 18 u  ← CASCADE (see post-fix audit)
    //
    //    R2 — Screen 2, approach to G (player missed F_mid or fell from G area):
    //      R2a row4 (47–49)  ← ground hop, 2 rows = 36 u
    //      R2b row8 (49–51)  ← offset RIGHT 4 rows = 72 u  zag
    //      → G  row10 (51–55)  2 rows = 36 u
    //
    //    R3 — Screen 3, approach to J (player fell past I/W6 zone):
    //      R3a row4 (71–72)  ← ground hop just right of W6, 2 rows = 36 u
    //      R3b row6 (74–75)  ← offset RIGHT 2 rows = 36 u  zag
    //      → J  row7 (72–75)   1 row  = 18 u  (final step)
    //
    //    R4 — Screen 3, approach to L (player fell off VRT or missed L):
    //      R4a row4 (83–84)  ← ground hop just right of W7, 2 rows = 36 u
    //      R4b row6 (82–83)  ← offset LEFT  2 rows = 36 u  zig
    //      → L  row7 (83–90)   1 row  = 18 u  (final step)
    //      NOTE: VRT fallers at cols75–81 reclimb via J(row8) → Base(row10,75–77).
    //
    //  Walls (Solid tiles above ground = visual blockers):
    //    rows3–4: W1(8–9)  W3(24–25)  W6(69–70)                [fence, 36 u]
    //    rows3–5: W2(21–22)  W4(43–44)  W5(63–64)  W7(81–82)   [wall,  54 u]

    let tiles: Vec<Vec<TileType>> = vec![
        solid(),                                                                              // row 0  ground
        solid(),                                                                              // row 1  ground
        solid(),                                                                              // row 2  ground (stand_y = −137)
        mixed(&[(8,9),(21,22),(24,25),(43,44),(63,64),(69,70),(81,82)], &[]),                  // row 3  walls W1–W7 (I → row4)
        mixed(&[(8,9),(21,22),(24,25),(43,44),(63,64),(69,70),(81,82)],                        // row 4  walls W1–W7
              &[(28,29),(38,42),(47,49),(65,68),(71,72),(83,84)]),                             //        R1a  F  R2a  I  R3a  R4a
        mixed(&[(21,22),(43,44),(63,64),(81,82)],                       &[(3,7),(18,20)]),                // row 5  W2/W4/W5/W7 | A  C  (NB3 → row6)
        plat( &[(44,48),(58,62),(69,70),(74,75),(82,83)]),                                    // row 6  F_mid  H  NB3  R3b  R4b
        plat( &[(26,27)]),                                                                    // row 7  R1b  (R2b → row8)
        plat( &[(12,15),(26,31),(49,51),(57,57),(72,75),(83,90)]),                            // row 8  B  D  R2b  NB2  J  L
        empty(),                                                                              // row 9  (OA→row10  G→row10)
        plat( &[(8,10),(23,23),(32,35),(51,55),(75,77)]),                                    // row 10 OA  NB1  E(chimney)  G  VRT-Base
        empty(),                                                                              // row 11
        plat( &[(79,80)]),                                                                    // row 12 VRT-T1
        empty(),                                                                              // row 13
        plat( &[(76,77)]),                                                                    // row 14 VRT-T2
        empty(),                                                                              // row 15
        plat( &[(79,81)]),                                                                    // row 16 VRT-Peak (tower summit)
        empty(),                                                                              // row 17
        empty(),                                                                              // row 18
        empty(),                                                                              // row 19
        empty(),                                                                              // row 20
        empty(),                                                                              // row 21
    ];

    LayerData {
        id: 0,
        tiles,
        origin_x: -864.0,
        origin_y: -200.0,
        spawn: Vec2::new(-819.0, -128.0),
    }
}

// ── Layer 1: Rooftops ──────────────────────────────────────────────────────────
//
// 96 cols × 22 rows
// origin_x = -864.0, origin_y = -200.0
// Spawn: (-819.0, -128.0)
//
// rows 0-2   : solid
// rows 3-5   : empty
// row  6     : P at 8..=12, 32..=37, 58..=62, 84..=87
// rows 7-9   : empty
// row  10    : P at 18..=22, 50..=54, 82..=85
// rows 11-13 : empty
// row  14    : P at 28..=32, 68..=72
// rows 15-21 : empty
fn subdivision_layer_1() -> LayerData {
    let solid = || vec![S; 96];
    let empty = || vec![E; 96];
    let plat = |platforms: &[(usize, usize)]| {
        let mut row = vec![E; 96];
        for &(start, end) in platforms {
            row[start..=end].fill(P);
        }
        row
    };

    let tiles: Vec<Vec<TileType>> = vec![
        solid(),                                               // row 0
        solid(),                                               // row 1
        solid(),                                               // row 2
        empty(),                                               // row 3
        empty(),                                               // row 4
        empty(),                                               // row 5
        plat(&[(8, 12), (32, 37), (58, 62), (84, 87)]),       // row 6
        empty(),                                               // row 7
        empty(),                                               // row 8
        empty(),                                               // row 9
        plat(&[(18, 22), (50, 54), (82, 85)]),                 // row 10
        empty(),                                               // row 11
        empty(),                                               // row 12
        empty(),                                               // row 13
        plat(&[(28, 32), (68, 72)]),                           // row 14
        empty(),                                               // row 15
        empty(),                                               // row 16
        empty(),                                               // row 17
        empty(),                                               // row 18
        empty(),                                               // row 19
        empty(),                                               // row 20
        empty(),                                               // row 21
    ];

    LayerData {
        id: 1,
        tiles,
        origin_x: -864.0,
        origin_y: -200.0,
        spawn: Vec2::new(-819.0, -128.0),
    }
}

// ── Layer 2: Basement / Sewers ─────────────────────────────────────────────────
//
// 96 cols × 22 rows
// origin_x = -864.0, origin_y = -200.0
// Spawn: (-819.0, -155.0)
//
// rows 0-1   : solid floor
// rows 2-12  : cave walls (S at col 0, S at col 95, E elsewhere)
//   row 6 special  : S|E×4|P×5|E×35|P×5|E×39|P×4|E×2|S
//   row 10 special : S|E×22|P×5|E×38|P×5|E×24|S
// rows 13-21 : solid ceiling
fn subdivision_layer_2() -> LayerData {
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

    let tiles: Vec<Vec<TileType>> = vec![
        solid(),                                               // row 0
        solid(),                                               // row 1
        cave(&[]),                                             // row 2
        cave(&[]),                                             // row 3
        cave(&[]),                                             // row 4
        cave(&[]),                                             // row 5
        cave(&[(5, 9), (45, 49), (89, 92)]),                   // row 6
        cave(&[]),                                             // row 7
        cave(&[]),                                             // row 8
        cave(&[]),                                             // row 9
        cave(&[(23, 27), (66, 70)]),                           // row 10
        cave(&[]),                                             // row 11
        cave(&[]),                                             // row 12
        solid(),                                               // row 13
        solid(),                                               // row 14
        solid(),                                               // row 15
        solid(),                                               // row 16
        solid(),                                               // row 17
        solid(),                                               // row 18
        solid(),                                               // row 19
        solid(),                                               // row 20
        solid(),                                               // row 21
    ];

    LayerData {
        id: 2,
        tiles,
        origin_x: -864.0,
        origin_y: -200.0,
        spawn: Vec2::new(-819.0, -155.0),
    }
}
