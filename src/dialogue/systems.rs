use bevy::prelude::*;

use crate::player::components::Player;

use super::components::{DialogueBox, DialogueState, DialogueText, NpcDialogue};

const TRIGGER_DISTANCE: f32 = 40.0;

/// Checks whether the player is near an NPC and E is pressed; activates dialogue.
pub fn check_dialogue_trigger(
    keyboard: Res<ButtonInput<KeyCode>>,
    player_query: Query<&Transform, With<Player>>,
    mut npc_query: Query<(Entity, &Transform, &mut NpcDialogue)>,
    mut dialogue_state: ResMut<DialogueState>,
) {
    // Only trigger if dialogue is not already active.
    if dialogue_state.active {
        return;
    }

    if !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }

    let Ok(player_tf) = player_query.single() else {
        return;
    };
    let player_pos = player_tf.translation.truncate();

    for (entity, npc_tf, mut npc) in &mut npc_query {
        let npc_pos = npc_tf.translation.truncate();
        if player_pos.distance(npc_pos) <= TRIGGER_DISTANCE {
            npc.current_line = 0;
            npc.active = true;
            dialogue_state.active = true;
            dialogue_state.speaker_entity = Some(entity);
            break;
        }
    }
}

/// Advances to the next dialogue line (Space or E); deactivates when finished.
pub fn advance_dialogue(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut npc_query: Query<&mut NpcDialogue>,
    mut dialogue_state: ResMut<DialogueState>,
    mut commands: Commands,
    box_query: Query<Entity, With<DialogueBox>>,
) {
    if !dialogue_state.active {
        return;
    }

    let advance = keyboard.just_pressed(KeyCode::Space) || keyboard.just_pressed(KeyCode::KeyE);

    if !advance {
        return;
    }

    let Some(speaker) = dialogue_state.speaker_entity else {
        return;
    };

    let Ok(mut npc) = npc_query.get_mut(speaker) else {
        // Speaker entity is gone — close dialogue
        close_dialogue(&mut commands, &box_query, &mut dialogue_state);
        return;
    };

    npc.current_line += 1;

    if npc.current_line >= npc.lines.len() {
        npc.active = false;
        npc.current_line = 0;
        close_dialogue(&mut commands, &box_query, &mut dialogue_state);
    }
    // Otherwise, render_dialogue_box will pick up the new current_line.
}

fn close_dialogue(
    commands: &mut Commands,
    box_query: &Query<Entity, With<DialogueBox>>,
    dialogue_state: &mut ResMut<DialogueState>,
) {
    for entity in box_query.iter() {
        commands.entity(entity).despawn();
    }
    dialogue_state.active = false;
    dialogue_state.speaker_entity = None;
}

/// Spawns / updates / despawns the dialogue box UI panel.
pub fn render_dialogue_box(
    mut commands: Commands,
    dialogue_state: Res<DialogueState>,
    npc_query: Query<&NpcDialogue>,
    box_query: Query<Entity, With<DialogueBox>>,
    mut text_query: Query<&mut Text, With<DialogueText>>,
) {
    if !dialogue_state.active {
        // Despawn if somehow still present
        for entity in box_query.iter() {
            commands.entity(entity).despawn();
        }
        return;
    }

    let Some(speaker) = dialogue_state.speaker_entity else {
        return;
    };

    let Ok(npc) = npc_query.get(speaker) else {
        return;
    };

    let line = npc
        .lines
        .get(npc.current_line)
        .map(|s| s.as_str())
        .unwrap_or("");

    if box_query.is_empty() {
        // Spawn the dialogue box
        commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(16.0),
                    left: Val::Px(16.0),
                    right: Val::Px(16.0),
                    padding: UiRect::all(Val::Px(12.0)),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.05, 0.05, 0.15, 0.88)),
                DialogueBox,
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new(line),
                    TextFont {
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    DialogueText,
                ));

                parent.spawn((
                    Text::new("[E / Space] to continue"),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.6, 0.6, 0.6)),
                ));
            });
    } else if dialogue_state.is_changed() {
        // Update the text of an existing box
        if let Ok(mut text) = text_query.single_mut() {
            **text = line.to_owned();
        }
    }
}
