/// Given which of the 4 cardinal neighbors are solid, return the atlas tile index.
/// Bit mask: bit 0 = up, bit 1 = right, bit 2 = down, bit 3 = left
///
/// Kenney Pixel Platformer tilemap_packed.png layout (20 cols × 9 rows, 18px tiles):
/// Row 0 (indices 0–19): grass/surface tiles
///   Index 4  = grass top standalone (no horizontal neighbors, has tile below)
///   Index 5  = grass top, left edge (right neighbor present)
///   Index 6  = grass top, middle (both left and right neighbors)
///   Index 7  = grass top, right edge (left neighbor present)
/// Row 1 (indices 20–39): dirt/fill tiles
///   Index 23 = dirt left edge
///   Index 24 = dirt fill (fully surrounded)
///   Index 25 = dirt right edge
pub fn autotile_index(up: bool, right: bool, down: bool, left: bool) -> usize {
    match (up, right, down, left) {
        // Surface tiles: no tile above (exposed top)
        (false, false, false, false) => 4, // isolated — grass top standalone
        (false, false, true, false) => 4,  // top only — grass top standalone
        (false, true, true, false) => 5,   // top-left corner — right neighbor, grass left edge
        (false, true, true, true) => 6,    // top middle — grass top middle
        (false, false, true, true) => 7,   // top-right corner — grass right edge
        (false, true, false, false) => 5,  // right only (thin slab)
        (false, false, false, true) => 7,  // left only (thin slab)
        (false, true, false, true) => 6,   // left+right, no vertical (thin slab middle)

        // Fill tiles: tile above (buried)
        (true, false, false, false) => 24, // bottom standalone
        (true, false, true, false) => 24,  // left edge column interior
        (true, true, true, false) => 23,   // interior left edge
        (true, true, true, true) => 24,    // fully surrounded
        (true, false, true, true) => 25,   // interior right edge
        (true, true, false, false) => 23,  // bottom-left corner
        (true, true, false, true) => 24,   // bottom middle
        (true, false, false, true) => 25,  // bottom-right corner
    }
}
