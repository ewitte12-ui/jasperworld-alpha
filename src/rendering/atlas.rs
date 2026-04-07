use bevy::prelude::*;

/// Configuration for an atlas spritesheet.
#[derive(Clone, Debug)]
pub struct AtlasConfig {
    pub tile_size: f32,
    pub spacing: f32,
    pub columns: usize,
    pub rows: usize,
}

impl AtlasConfig {
    /// Kenney Pixel Platformer tilemap_packed.png: 18x18px tiles, 1px spacing, 20 cols x 9 rows.
    pub const TILE_TILEMAP: AtlasConfig = AtlasConfig {
        tile_size: 18.0,
        spacing: 1.0,
        columns: 20,
        rows: 9,
    };

    /// Kenney Pixel Platformer tilemap-characters_packed.png: 24x24px tiles, 1px spacing, 9 cols x 3 rows.
    pub const CHAR_TILEMAP: AtlasConfig = AtlasConfig {
        tile_size: 24.0,
        spacing: 1.0,
        columns: 9,
        rows: 3,
    };

    /// Total width of the spritesheet in pixels.
    pub fn sheet_width(&self) -> f32 {
        (self.columns as f32) * (self.tile_size + self.spacing) - self.spacing
    }

    /// Total height of the spritesheet in pixels.
    pub fn sheet_height(&self) -> f32 {
        (self.rows as f32) * (self.tile_size + self.spacing) - self.spacing
    }
}

/// Resource holding the tile atlas (environment tiles).
#[derive(Resource, Clone)]
pub struct TileAtlas {
    pub image: Handle<Image>,
    pub config: AtlasConfig,
}

impl TileAtlas {
    pub fn new(image: Handle<Image>, config: AtlasConfig) -> Self {
        Self { image, config }
    }
}

/// Resource holding the character atlas.
#[derive(Resource, Clone)]
pub struct CharAtlas {
    pub image: Handle<Image>,
    pub config: AtlasConfig,
}

impl CharAtlas {
    pub fn new(image: Handle<Image>, config: AtlasConfig) -> Self {
        Self { image, config }
    }
}

/// Computes UV coordinates for a tile at the given index in the atlas.
///
/// Returns `[u_min, v_min, u_max, v_max]`.
///
/// UV space in Bevy: (0,0) is top-left, (1,1) is bottom-right.
///
/// Per jasper_sprite_atlas_guardrail.txt Rule [1], a half-texel inset is applied
/// to prevent neighbor-texel sampling at atlas cell borders, eliminating frame
/// bleeding and seam artifacts.
pub fn uv_rect(config: &AtlasConfig, tile_index: usize) -> [f32; 4] {
    let col = tile_index % config.columns;
    let row = tile_index / config.columns;

    let sheet_width = config.sheet_width();
    let sheet_height = config.sheet_height();

    // Half-texel inset: pull each edge 0.5 px inward in UV space so the GPU
    // never samples the neighboring cell even with linear filtering.
    let col_origin = col as f32 * (config.tile_size + config.spacing);
    let row_origin = row as f32 * (config.tile_size + config.spacing);

    let u_min = (col_origin + 0.5) / sheet_width;
    let v_min = (row_origin + 0.5) / sheet_height;
    let u_max = (col_origin + config.tile_size - 0.5) / sheet_width;
    let v_max = (row_origin + config.tile_size - 0.5) / sheet_height;

    [u_min, v_min, u_max, v_max]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-5
    }

    #[test]
    fn uv_rect_first_tile() {
        let config = AtlasConfig::TILE_TILEMAP;
        let uv = uv_rect(&config, 0);

        // First tile: col=0, row=0.  Half-texel inset shifts edges by 0.5px.
        let sheet_w = config.sheet_width();
        let sheet_h = config.sheet_height();
        let expected_u_min = 0.5 / sheet_w;
        let expected_v_min = 0.5 / sheet_h;
        let expected_u_max = (config.tile_size - 0.5) / sheet_w;
        let expected_v_max = (config.tile_size - 0.5) / sheet_h;

        assert!(
            approx_eq(uv[0], expected_u_min),
            "u_min expected {}, got {}",
            expected_u_min,
            uv[0]
        );
        assert!(
            approx_eq(uv[1], expected_v_min),
            "v_min expected {}, got {}",
            expected_v_min,
            uv[1]
        );
        assert!(
            approx_eq(uv[2], expected_u_max),
            "u_max expected {}, got {}",
            expected_u_max,
            uv[2]
        );
        assert!(
            approx_eq(uv[3], expected_v_max),
            "v_max expected {}, got {}",
            expected_v_max,
            uv[3]
        );
    }

    #[test]
    fn uv_rect_second_tile() {
        let config = AtlasConfig::TILE_TILEMAP;
        let uv = uv_rect(&config, 1);

        // Second tile: col=1, row=0.  col_origin = 1*(18+1) = 19.
        let sheet_w = config.sheet_width();
        let sheet_h = config.sheet_height();
        let col_origin = config.tile_size + config.spacing; // = 19
        let expected_u_min = (col_origin + 0.5) / sheet_w;
        let expected_v_min = 0.5 / sheet_h;
        assert!(
            approx_eq(uv[0], expected_u_min),
            "u_min expected {}, got {}",
            expected_u_min,
            uv[0]
        );
        assert!(
            approx_eq(uv[1], expected_v_min),
            "v_min expected {}, got {}",
            expected_v_min,
            uv[1]
        );
    }

    #[test]
    fn uv_rect_second_row_first_tile() {
        let config = AtlasConfig::TILE_TILEMAP;
        // First tile of second row: index = columns (20).  row_origin = 1*(18+1) = 19.
        let uv = uv_rect(&config, config.columns);

        let sheet_w = config.sheet_width();
        let sheet_h = config.sheet_height();
        let row_origin = config.tile_size + config.spacing; // = 19
        let expected_u_min = 0.5 / sheet_w;
        let expected_v_min = (row_origin + 0.5) / sheet_h;
        assert!(
            approx_eq(uv[0], expected_u_min),
            "u_min expected {}, got {}",
            expected_u_min,
            uv[0]
        );
        assert!(
            approx_eq(uv[1], expected_v_min),
            "v_min expected {}, got {}",
            expected_v_min,
            uv[1]
        );
    }

    #[test]
    fn uv_rect_last_tile() {
        let config = AtlasConfig::TILE_TILEMAP;
        let last_index = config.columns * config.rows - 1;
        let uv = uv_rect(&config, last_index);

        let sheet_w = config.sheet_width();
        let sheet_h = config.sheet_height();

        let col = (config.columns - 1) as f32;
        let row = (config.rows - 1) as f32;

        // Half-texel inset: col_origin + 0.5 for u_min.
        let expected_u_min = (col * (config.tile_size + config.spacing) + 0.5) / sheet_w;
        let expected_v_min = (row * (config.tile_size + config.spacing) + 0.5) / sheet_h;

        assert!(
            approx_eq(uv[0], expected_u_min),
            "u_min expected {}, got {}",
            expected_u_min,
            uv[0]
        );
        assert!(
            approx_eq(uv[1], expected_v_min),
            "v_min expected {}, got {}",
            expected_v_min,
            uv[1]
        );

        // u_max should be <= 1.0
        assert!(uv[2] <= 1.0 + 1e-5, "u_max should be <= 1.0, got {}", uv[2]);
        assert!(uv[3] <= 1.0 + 1e-5, "v_max should be <= 1.0, got {}", uv[3]);
    }

    #[test]
    fn uv_rect_char_atlas() {
        let config = AtlasConfig::CHAR_TILEMAP;
        let uv = uv_rect(&config, 0);

        // Tile 0: col=0, row=0 with half-texel inset.
        let sheet_w = config.sheet_width();
        let sheet_h = config.sheet_height();
        assert!(approx_eq(uv[0], 0.5 / sheet_w));
        assert!(approx_eq(uv[1], 0.5 / sheet_h));
        assert!(approx_eq(uv[2], (config.tile_size - 0.5) / sheet_w));
        assert!(approx_eq(uv[3], (config.tile_size - 0.5) / sheet_h));
    }

    #[test]
    fn sheet_dimensions_tile() {
        let config = AtlasConfig::TILE_TILEMAP;
        // 20 tiles * (18 + 1) - 1 = 379
        assert!(approx_eq(config.sheet_width(), 379.0));
        // 9 tiles * (18 + 1) - 1 = 170
        assert!(approx_eq(config.sheet_height(), 170.0));
    }

    #[test]
    fn sheet_dimensions_char() {
        let config = AtlasConfig::CHAR_TILEMAP;
        // 9 tiles * (24 + 1) - 1 = 224
        assert!(approx_eq(config.sheet_width(), 224.0));
        // 3 tiles * (24 + 1) - 1 = 74
        assert!(approx_eq(config.sheet_height(), 74.0));
    }
}
