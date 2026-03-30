use bevy::prelude::*;

use crate::tilemap::tilemap::TileType::{self, Empty as E, Platform as P, Solid as S};

use super::level_data::{LayerData, LevelData, LevelId};

pub fn city_level() -> LevelData {
    LevelData {
        id: LevelId::City,
        layers: vec![
            city_layer_0(), // street level
            city_layer_1(), // fire escape (middle)
            city_layer_2(), // rooftop
        ],
    }
}

// ── Layer 0: Street Level ─────────────────────────────────────────────────────
//
// 128 cols × 22 rows  |  origin_x = -1152.0  |  origin_y = -200.0
// ground_top = -146.0  |  ground_y = -137.0  |  spawn = (-1107.0, -128.0)
//
// ─── BRANCH 1 — SC2 Binary Branch (diverge: D, rejoin: G) ───────────────────
//   Safe / longer:  D(row7) ──▼ E-lo(row4) ──▲ F-lo(row7) ──▶ G(row7)
//                   Lower height, forgiving falls; miss any hop → ground → retry
//   Fast / risky:   D(row7) ──▲ E-hi(row10) ──▶ F-hi(row10) ──▼ G(row7)
//                   Direct 2-hop lateral traverse at height; miss E-hi = full height
//                   loss to ground; no intermediate catch platform on this line.
//   Diverge: D(row7, 26–31) — player sees E-lo below AND E-hi above simultaneously.
//   Rejoin:  G(row7, 55–60)
//   No dead-end: both routes reach G; ground is always recoverable from either.
//
// ─── BRANCH 2 — SC3 Tower vs. Low Bypass (diverge: H, rejoin: M) ────────────
//   Safe / longer:  H(row4) ──▶ LB1(row4,78–83) ──▶ LB2(row4,88–93) ──▲ M(row7)
//                   Three same-height hops then one +3-row step to M.
//                   Miss any hop → ground → can re-approach from left.
//   Fast / risky:   H(row4) ──▲ I(row7) ──▲ J(row10) ──▲ K(row14) ──▲ K2(row17)
//                              then fall 8 rows to L(row9) ──▲ M(row7)
//                   Tower peak is off-screen from ground. Fall from K2 = 144u.
//   Diverge: H(row4, 63–68) — I visible above (tower), LB1 visible ahead at same
//            height (bypass). Player actively jumps higher to enter tower.
//   Rejoin:  M(row7, 92–97)
//   No dead-end: from any tower step, player can fall right → bypass shelf zone →
//                LB2 → M, or fall left → H → bypass. No softlock possible.
//   LB1 doubles as extended catch zone for I-misses (falls rightward from I
//   at cols 72–77 can land on LB1 at cols 78–83 at same ground level). ✓
//
// ─── DECISION TREE (route planning required) ─────────────────────────────────
//
//   Decision 1 — D(row7,26–31):  E-lo [safe/long]  OR  E-hi [committed]
//     │
//     ├─ E-lo → F-lo(row7) → G-lo(row7,55–60)
//     │    └─ Decision 2a — G-lo: jump to G-hi(row10) [costs extra +3 rows] OR proceed to H
//     │
//     └─ E-hi → F-hi → FG-hi(row10,52–57)
//          └─ Decision 2b — FG-hi: continue to G-hi(row10) [stay committed] OR drop to G-lo
//                                    (Decision 1 determines the COST of Decision 2)
//
//   Decision 3 — G-hi(row10,62–66): continue to H-hi(row10) OR drop to H(row4)
//     │
//     └─ H-hi → IJ-bridge(row10,74–77)
//          └─ Decision 4 — IJ-bridge: jump to J(row10,79–84) [skip I, tower step 2]
//                                      OR  drop to I(row7,72–77) [normal tower entry]
//                          (I is the fall-catch for IJ-bridge — same columns, 3 rows below)
//
//   Decision 5 — H(row4,63–68): tower [I→J→K→K2 risky] OR bypass [LB1→LB2 safe]
//
//   Decision 6 — M(row7,92–97): MN-hi [row9, committed] OR N [row4, safe/long]
//
//   Sequenced consequence: D→E-hi earns access to FG-hi→G-hi at zero extra cost.
//   D→E-lo can still reach G-hi but costs one additional +3 row jump at G-lo.
//   The hard route at D reduces the cost of the harder exit at G.
//
// ─── HEIGHT COMMITMENT ZONE 1 — Extended Scaffold Run + Corridor ─────────────
//   Elevation: row10  |  fall consequence: 8 rows to ground (144u) — not re-jumpable
//   Entry:  D(row7) → E-hi(row10,33–37) [+3 rows]
//   Zone:   E-hi ──▶ F-hi(44–48) ──▶ FG-hi(52–57) [HCZ-1 sustained, 3 platforms]
//   Portal: FG-hi ──▶ G-hi(62–66) [2-col gap, same height — stay committed]
//           OR FG-hi ──▼ G-lo(row7,55–60) [drop left — exit committed zone]
//   Corridor: G-hi ──▶ H-hi(69–72) ──▶ IJ-bridge(74–77) ──▶ J(79–84)
//             All same-height small-gap hops; no height change until decision at IJ-bridge
//   Recovery: D(row7)→E-hi = minimum re-entry; 8-row fall cannot be climbed directly
//
// ─── HEIGHT COMMITMENT ZONE 2 — Post-Tower Elevated Exit ─────────────────────
//   Entry:  M(row7,92–97) — arrived after tower climb (itself a 13-row commitment)
//   High:   M ──▲ MN-hi(row9,101–106) [+2 rows] ──▼ O(row7,108–113) [-2 rows]
//   Low:    M ──▼ N(row4,101–106) [-3 rows] ──▲ RP-O(row4,109–113) ──▲ O(row7) [+3r]
//   Fall from MN-hi: N(row4,101–106) directly below — same cols, 5 rows — natural catch ✓
//
// ─── MAJOR VERTICAL REGION 1 ─────────────────────────────────────────────────
//   Height span: D(row7) → row10 = 3 rows = 54u = 2.45× JasperHeight ✓
//   Row10 highway: E-hi through J = 7 platforms spanning cols 33–84 = 918u
//
// ─── MAJOR VERTICAL REGION 2 — SC3 Commitment Tower (multi-screen tall) ─────
//   Height span: H(row4) to K2(row17) = 13 rows = 234u.
//   Ground camera top = 120u. K2 stand_y = 133u → above viewport from ground. ✓
//   Zigzag (no straight vertical climb):
//     H  (row4,  63–68)  ──▲ RIGHT
//     I  (row7,  72–77)  +3 rows 54u ✓  ──▲ RIGHT
//     J  (row10, 79–84)  +3 rows 54u ✓  ──▲ LEFT  (reversal)
//     K  (row14, 74–77)  +4 rows 72u ✓  ──▲ RIGHT (reversal)
//     K2 (row17, 80–83)  +3 rows 54u ✓  ── peak ──
//     L  (row9,  80–87)  −8 rows, Category B wide recovery shelf
//
// Platform table:
//   Row  4: A(4–8)  C(19–23)  E-lo(35–40)  H(63–68)  R-SC3(72–76)
//            LB1(78–83)  LB2(88–93)  N(101–106)  RP-O(109–113)
//   Row  7: B(12–17)  D(26–31)  F-lo(44–49)  G-lo(55–60)  I(72–77)
//            M(92–97)  O(108–113)  P(116–121)
//   Row  9: L(80–87)  MN-hi(101–106)                       ← HCZ-2 elevated hop
//   Row 10: E-hi(33–37)  F-hi(44–48)  FG-hi(52–57)        ← HCZ-1 three-step zone
//            G-hi(62–66)  H-hi(69–72)  IJ-bridge(74–77)   ← committed corridor
//            J(79–84)                                       ← tower step 2
//   Row 14: K(74–77)
//   Row 17: K2(80–83)                                       ← tower peak
//
// ─── RECOVERY AUDIT — every major failure point ──────────────────────────────
//
//  FP-SC1   A/B/C/D falls  → ground → nearest row4 platform ≤2 rows away ✓
//
//  FP-Ehi   Miss E-hi      → E-lo(row4,35–40) column overlap catches fall ✓
//  FP-Fhi   Miss F-hi      → F-lo(row7,44–49) column overlap catches fall ✓
//  FP-G     Miss G         → ground → walk forward 3–8 cols → H(row4,63–68) +2 rows ✓
//                            (forward-continue: player doesn't need to re-reach G)
//
//  FP-I     Miss I (tower) → R-SC3(row4,72–76) and LB1(row4,78–83) catch zone;
//                            choose retry tower or take bypass ✓
//  FP-J     Miss J         → L(row9,80–87) is 1 row below J, cols overlap at 80–84 ✓
//  FP-K     Miss K         → falls to IJ-bridge(row10,74–77): same column range,
//                            4 rows below K (not I — IJ-bridge intercepts first) ✓
//                            from IJ-bridge: jump to J → retry K ✓
//  FP-K2u   Miss K2 under  → K catches (retry immediately) ✓
//  FP-K2m   Miss K2 mid    → J(row10,79–84) or L(row9,80–87) catches ✓
//  FP-K2o   K2 overshoot   → L(row9,80–87) at cols 84–87; far overshoot → LB2 ✓
//  FP-L     Fall from L    → LB1/LB2(row4) → bypass continues to M ✓
//
//  FP-M     Miss M         → ground → LB2(row4,88–93) +2 rows → M +3 rows ✓
//  FP-N     Miss N         → ground → N same cols +2 rows ✓
//  FP-O     Miss O         → ground → RP-O(row4,109–113) +2 rows → O +3 rows ✓
//                            [RP-O is directly below O; two-step path is visually clear]
//  FP-P     Miss P         → ground → walk to gate (ground-level passage) ✓
//
// Category B: R-SC3(72–76,row4)  LB1(78–83,row4)  LB2(88–93,row4)
//             RP-O(109–113,row4)  L(80–87,row9)
// Spine (safe default): Ground→A→B→C→D→E-lo→F-lo→G→H→LB1→LB2→M→N→RP-O→O→P→Gate
fn city_layer_0() -> LayerData {
    let solid = || vec![S; 128];
    let empty = || vec![E; 128];
    let plat = |platforms: &[(usize, usize)]| {
        let mut row = vec![E; 128];
        for &(start, end) in platforms {
            row[start..=end].fill(P);
        }
        row
    };

    // Row index 0 = bottom
    let tiles: Vec<Vec<TileType>> = vec![
        solid(), // row 0
        solid(), // row 1
        solid(), // row 2
        empty(), // row 3
        // A: SC1 entry  C: scaffold step  E-lo: Branch1 safe floor
        // H: Branch2 diverge  R-SC3: tower recovery / bypass start
        // LB1: bypass step 1 (+ catches rightward I-falls)
        // LB2: bypass step 2 (→ M +3 rows)
        // N: SC4 descent step
        // RP-O: recovery shelf directly below O(row7,108–113) — FP-O fix
        plat(&[(4, 8), (19, 23), (35, 40), (63, 68), (72, 76), (78, 83), (88, 93), (101, 106), (109, 113)]), // row 4
        empty(), // row 5
        empty(), // row 6
        // B: scaffold | D: branch point (E-lo below / E-hi above) | F-lo: safe branch
        // G: convergence | I: SC3 climb step | M: SC4 entry | O, P: pre-gate descent
        plat(&[(12, 17), (26, 31), (44, 49), (55, 60), (72, 77), (92, 97), (108, 113), (116, 121)]), // row 7
        empty(), // row 8
        // L: tower recovery shelf (Category B)
        // MN-hi: HCZ-2 elevated hop from M; N(row4,101-106) is fall-catch below same cols
        plat(&[(80, 87), (101, 106)]), // row 9
        // HCZ-1 zone:    E-hi(33–37)  F-hi(44–48)  FG-hi(52–57)
        // HCZ-1 portal:  G-hi(62–66) — exit from FG-hi (2-col gap, same height)
        //                              OR entry from G-lo (jump +3 rows from row7)
        //                falls from G-hi(62–63) → H(row4,63–68) ✓
        // Corridor:      H-hi(69–72)  IJ-bridge(74–77)
        //                IJ-bridge falls → I(row7,72–77) same cols, 3 rows below ✓
        // Tower step 2:  J(79–84)
        plat(&[(33, 37), (44, 48), (52, 57), (62, 66), (69, 72), (74, 77), (79, 84)]), // row 10
        empty(), // row 11
        empty(), // row 12
        empty(), // row 13
        // K: SC3 tower penultimate step (HIGH COMMITMENT)
        plat(&[(74, 77)]), // row 14
        empty(),           // row 15
        empty(),           // row 16
        // K2: SC3 tower PEAK — multi-screen-tall summit; +3 rows RIGHT from K
        // stand_y = 133u; off-screen from ground (camera top = 120u at ground). ✓
        plat(&[(80, 83)]), // row 17
        empty(), // row 18
        empty(), // row 19
        empty(), // row 20
        empty(), // row 21
    ];

    LayerData {
        id: 0,
        tiles,
        origin_x: -1152.0,
        origin_y: -200.0,
        spawn: Vec2::new(-1107.0, -128.0),
    }
}

// ── Layer 1: Fire Escape Shaft ────────────────────────────────────────────────
//
// 128 cols × 22 rows — enclosed cave
// Rows 0–1: solid floor
// Rows 2–12: cave interior (S at col 0 and col 127, E elsewhere)
//   Row  4: platforms at (5–9), (55–59)
//   Row  7: platforms at (18–22), (75–79)
//   Row 10: platforms at (35–39), (95–99)
// Rows 13–21: solid ceiling
fn city_layer_1() -> LayerData {
    let solid = || vec![S; 128];
    let cave = |platforms: &[(usize, usize)]| {
        let mut row = vec![E; 128];
        row[0] = S;
        row[127] = S;
        for &(start, end) in platforms {
            row[start..=end].fill(P);
        }
        row
    };

    let tiles: Vec<Vec<TileType>> = vec![
        solid(),                           // row 0
        solid(),                           // row 1
        cave(&[]),                         // row 2
        cave(&[]),                         // row 3
        cave(&[(5, 9), (55, 59)]),         // row 4 — left and center-right
        cave(&[]),                         // row 5
        cave(&[]),                         // row 6
        cave(&[(18, 22), (75, 79)]),       // row 7 — offset right
        cave(&[]),                         // row 8
        cave(&[]),                         // row 9
        cave(&[(35, 39), (95, 99)]),       // row 10 — center and far right
        cave(&[]),                         // row 11
        cave(&[]),                         // row 12
        solid(),                           // row 13
        solid(),                           // row 14
        solid(),                           // row 15
        solid(),                           // row 16
        solid(),                           // row 17
        solid(),                           // row 18
        solid(),                           // row 19
        solid(),                           // row 20
        solid(),                           // row 21
    ];

    LayerData {
        id: 1,
        tiles,
        origin_x: -1152.0,
        origin_y: -200.0,
        spawn: Vec2::new(-1107.0, -155.0),
    }
}

// ── Layer 2: Rooftop ──────────────────────────────────────────────────────────
//
// 128 cols × 22 rows
// Rows 0–2: solid rooftop base
// Row  6: platforms at (8–12), (40–44), (74–78), (108–112)
// Row 10: platforms at (20–24), (62–66), (100–104)
// Row 14: platforms at (36–40), (90–94)
fn city_layer_2() -> LayerData {
    let solid = || vec![S; 128];
    let empty = || vec![E; 128];
    let plat = |platforms: &[(usize, usize)]| {
        let mut row = vec![E; 128];
        for &(start, end) in platforms {
            row[start..=end].fill(P);
        }
        row
    };

    let tiles: Vec<Vec<TileType>> = vec![
        solid(),                                              // row 0
        solid(),                                              // row 1
        solid(),                                              // row 2
        empty(),                                              // row 3
        empty(),                                              // row 4
        empty(),                                              // row 5
        plat(&[(8, 12), (40, 44), (74, 78), (108, 112)]),    // row 6
        empty(),                                              // row 7
        empty(),                                              // row 8
        empty(),                                              // row 9
        plat(&[(20, 24), (62, 66), (100, 104)]),              // row 10
        empty(),                                              // row 11
        empty(),                                              // row 12
        empty(),                                              // row 13
        plat(&[(36, 40), (90, 94)]),                          // row 14
        empty(),                                              // row 15
        empty(),                                              // row 16
        empty(),                                              // row 17
        empty(),                                              // row 18
        empty(),                                              // row 19
        empty(),                                              // row 20
        empty(),                                              // row 21
    ];

    LayerData {
        id: 2,
        tiles,
        origin_x: -1152.0,
        origin_y: -200.0,
        spawn: Vec2::new(-1107.0, -128.0),
    }
}
