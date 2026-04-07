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

/// Marker on the child entity that holds the player's visual 3D model
/// (SceneRoot). Used by facing-direction and procedural animation code
/// to transform the model independently of the physics parent entity.
#[derive(Component)]
pub struct PlayerModelVisual;

/// Marker on the skeleton's root bone entity. The walk animation has
/// root motion that drifts Y downward over loops; `pin_player_root_bone`
/// resets Y to this stored value each frame after animation evaluation.
#[derive(Component)]
pub struct PlayerRootBone {
    pub original_y: f32,
}

/// Marker on the player entity at spawn. Removed once the AnimationPlayer
/// descendant is found and the animation graph is wired up. While this
/// marker is present, `setup_player_animation` polls each frame.
#[derive(Component)]
pub struct PlayerModelPending;

/// Holds animation clip handles while they're loading asynchronously.
/// Present on the player entity between finding the AnimationPlayer
/// and confirming all clips are loaded. Removed once the graph is built.
#[derive(Component)]
pub struct PlayerClipsPending {
    pub anim_entity: Entity,
    pub clip_idle: Handle<AnimationClip>,
    pub clip_walk: Handle<AnimationClip>,
    pub clip_jump: Handle<AnimationClip>,
    pub clip_hurt: Handle<AnimationClip>,
}

/// Stores the animation graph wiring for the player's skeletal animation.
/// Placed on the player (physics parent) entity once the GLB's
/// AnimationPlayer descendant is discovered and configured.
///
/// WHAT BREAKS if indices are wrong: the wrong animation clip plays for
/// a given PlayerAnimState, causing visual mismatch with gameplay.
#[derive(Component)]
pub struct PlayerAnimGraph {
    /// The descendant entity that owns the AnimationPlayer component.
    pub anim_entity: Entity,
    /// Graph node index for the idle animation (Animation(0) in jasper.glb).
    pub idle: AnimationNodeIndex,
    /// Graph node index for the walk animation (Animation(1) in jasper.glb).
    pub walk: AnimationNodeIndex,
    /// Graph node index for the jump animation (Animation(2) in jasper.glb).
    pub jump: AnimationNodeIndex,
    /// Graph node index for the hurt animation (Animation(3) in jasper.glb).
    pub hurt: AnimationNodeIndex,
    /// The currently playing animation state — used to avoid restarting
    /// the same animation every frame.
    pub current: PlayerAnimState,
}
