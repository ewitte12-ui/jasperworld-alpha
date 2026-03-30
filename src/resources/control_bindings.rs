use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum GameAction {
    MoveLeft,
    MoveRight,
    Jump,
    Interact,
    Attack,
}

impl GameAction {
    pub const ALL: [GameAction; 5] = [
        GameAction::MoveLeft,
        GameAction::MoveRight,
        GameAction::Jump,
        GameAction::Interact,
        GameAction::Attack,
    ];

    pub fn display_name(self) -> &'static str {
        match self {
            GameAction::MoveLeft => "Move Left",
            GameAction::MoveRight => "Move Right",
            GameAction::Jump => "Jump",
            GameAction::Interact => "Interact",
            GameAction::Attack => "Attack",
        }
    }
}

#[derive(Resource, Clone, Debug)]
pub struct ControlBindings {
    pub bindings: HashMap<GameAction, KeyCode>,
}

impl Default for ControlBindings {
    fn default() -> Self {
        let mut bindings = HashMap::new();
        bindings.insert(GameAction::MoveLeft, KeyCode::KeyA);
        bindings.insert(GameAction::MoveRight, KeyCode::KeyD);
        bindings.insert(GameAction::Jump, KeyCode::Space);
        bindings.insert(GameAction::Interact, KeyCode::KeyE);
        bindings.insert(GameAction::Attack, KeyCode::KeyF);
        Self { bindings }
    }
}

impl ControlBindings {
    pub fn key_for(&self, action: GameAction) -> KeyCode {
        self.bindings
            .get(&action)
            .copied()
            .unwrap_or(KeyCode::Escape)
    }

    pub fn rebind(&mut self, action: GameAction, new_key: KeyCode) {
        // If the new key is already bound to another action, swap them.
        let mut swap_action = None;
        for (&existing_action, &existing_key) in &self.bindings {
            if existing_key == new_key && existing_action != action {
                swap_action = Some(existing_action);
                break;
            }
        }
        if let Some(swap) = swap_action {
            let old_key = self.key_for(action);
            self.bindings.insert(swap, old_key);
        }
        self.bindings.insert(action, new_key);
    }
}

#[derive(Resource, Default)]
pub struct RebindingState {
    pub awaiting: Option<GameAction>,
}

pub fn keycode_display_name(key: KeyCode) -> &'static str {
    match key {
        KeyCode::KeyA => "A",
        KeyCode::KeyB => "B",
        KeyCode::KeyC => "C",
        KeyCode::KeyD => "D",
        KeyCode::KeyE => "E",
        KeyCode::KeyF => "F",
        KeyCode::KeyG => "G",
        KeyCode::KeyH => "H",
        KeyCode::KeyI => "I",
        KeyCode::KeyJ => "J",
        KeyCode::KeyK => "K",
        KeyCode::KeyL => "L",
        KeyCode::KeyM => "M",
        KeyCode::KeyN => "N",
        KeyCode::KeyO => "O",
        KeyCode::KeyP => "P",
        KeyCode::KeyQ => "Q",
        KeyCode::KeyR => "R",
        KeyCode::KeyS => "S",
        KeyCode::KeyT => "T",
        KeyCode::KeyU => "U",
        KeyCode::KeyV => "V",
        KeyCode::KeyW => "W",
        KeyCode::KeyX => "X",
        KeyCode::KeyY => "Y",
        KeyCode::KeyZ => "Z",
        KeyCode::Space => "Space",
        KeyCode::Enter => "Enter",
        KeyCode::ShiftLeft => "L-Shift",
        KeyCode::ShiftRight => "R-Shift",
        KeyCode::ControlLeft => "L-Ctrl",
        KeyCode::ControlRight => "R-Ctrl",
        KeyCode::ArrowUp => "Up",
        KeyCode::ArrowDown => "Down",
        KeyCode::ArrowLeft => "Left",
        KeyCode::ArrowRight => "Right",
        KeyCode::Tab => "Tab",
        _ => "???",
    }
}
