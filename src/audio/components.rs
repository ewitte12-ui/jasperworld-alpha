use bevy::prelude::*;

#[derive(Resource)]
pub struct AudioHandles {
    pub jump: Handle<AudioSource>,
    pub pickup: Handle<AudioSource>,
    pub enemy_hit: Handle<AudioSource>,
    pub player_hurt: Handle<AudioSource>,
    pub bgm: Handle<AudioSource>,
}

#[derive(Component)]
pub struct BackgroundMusic;
