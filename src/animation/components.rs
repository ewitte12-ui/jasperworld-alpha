use bevy::prelude::*;

/// Atlas grid layout for UV computation.
#[derive(Clone, Copy, Debug)]
pub struct AtlasLayout {
    pub cols: usize,
    pub rows: usize,
    pub texture_w: f32,
    pub texture_h: f32,
}

impl AtlasLayout {
    pub const RACCOON: Self = Self { cols: 4, rows: 4, texture_w: 512.0, texture_h: 512.0 };
    pub const ENEMY: Self = Self { cols: 4, rows: 2, texture_w: 512.0, texture_h: 256.0 };

    /// Returns [u_min, v_min, u_max, v_max] with half-texel inset.
    pub fn uv_for_index(&self, index: usize) -> [f32; 4] {
        let col = (index % self.cols) as f32;
        let row = (index / self.cols) as f32;
        let eps_u = 0.5 / self.texture_w;
        let eps_v = 0.5 / self.texture_h;
        let u_min = col / self.cols as f32 + eps_u;
        let v_min = row / self.rows as f32 + eps_v;
        let u_max = (col + 1.0) / self.cols as f32 - eps_u;
        let v_max = (row + 1.0) / self.rows as f32 - eps_v;
        [u_min, v_min, u_max, v_max]
    }
}

#[derive(Component)]
pub struct SpriteAnimation {
    pub frames: Vec<usize>, // atlas tile indices for each frame
    pub current_frame: usize,
    pub timer: Timer, // frame duration
    pub looping: bool,
    /// Last frame index written to the mesh UVs — skip rewrite if unchanged.
    pub last_written_frame: usize,
    /// Atlas grid layout — determines UV computation per frame index.
    pub atlas: AtlasLayout,
}

#[derive(Component, PartialEq, Eq, Clone, Copy, Debug)]
pub enum PlayerAnimState {
    Idle,
    Walking,
    Jumping,
    Hurt,
}

/// Animation state for enemies. Driven by velocity, not input.
#[derive(Component, PartialEq, Eq, Clone, Copy, Debug)]
pub enum EnemyAnimState {
    Idle,
    Walking,
}
