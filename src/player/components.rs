use bevy::prelude::*;
use bevy_tnua::TnuaScheme;
use bevy_tnua::builtins::{TnuaBuiltinJump, TnuaBuiltinWalk};

/// Marker component for the player entity.
#[derive(Component)]
pub struct Player;

/// Which horizontal direction the player is facing.
#[derive(Component, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum FacingDirection {
    Left,
    #[default]
    Right,
}

/// The Tnua control scheme for the player.
///
/// Derives [`TnuaScheme`] which auto-generates `PlayerControlSchemeConfig`,
/// `PlayerControlSchemeActionDiscriminant`, and `PlayerControlSchemeActionState`.
#[derive(TnuaScheme)]
#[scheme(basis = TnuaBuiltinWalk)]
pub enum PlayerControlScheme {
    Jump(TnuaBuiltinJump),
}
