use bevy::prelude::*;

/// An NPC that can deliver dialogue lines when the player approaches.
#[derive(Component)]
pub struct NpcDialogue {
    pub lines: Vec<String>,
    pub current_line: usize,
    pub active: bool,
}

impl NpcDialogue {
    pub fn new(lines: Vec<&str>) -> Self {
        Self {
            lines: lines.into_iter().map(|s| s.to_owned()).collect(),
            current_line: 0,
            active: false,
        }
    }
}

/// Resource tracking the global dialogue state.
#[derive(Resource, Default)]
pub struct DialogueState {
    pub active: bool,
    pub speaker_entity: Option<Entity>,
}

/// Marker for the on-screen dialogue box UI node.
#[derive(Component)]
pub struct DialogueBox;

/// Marker for the text inside the dialogue box.
#[derive(Component)]
pub struct DialogueText;
