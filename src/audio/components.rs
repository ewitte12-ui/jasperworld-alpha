use bevy::prelude::*;

#[derive(Resource)]
pub struct AudioHandles {
    pub jump: Handle<AudioSource>,
    pub pickup: Handle<AudioSource>,
    pub enemy_hit: Handle<AudioSource>,
    pub player_hurt: Handle<AudioSource>,
    pub bgm: [Handle<AudioSource>; 4], // indexed by level: Forest=0, Subdivision=1, City=2, Sanctuary=3
}

#[derive(Component)]
pub struct BackgroundMusic;
