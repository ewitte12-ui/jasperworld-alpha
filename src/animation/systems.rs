use avian2d::prelude::LinearVelocity;
use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;

use crate::combat::components::Invulnerable;
use crate::enemies::components::Enemy;
use crate::player::components::{FacingDirection, Player};

use super::components::{
    EnemyAnimState, PlayerAnimGraph, PlayerAnimState, PlayerClipsPending, PlayerModelPending,
    PlayerModelVisual, PlayerRootBone, SpriteAnimation,
};

/// Determines the player's animation state based on physics velocity and status components.
///
/// The player no longer carries a `SpriteAnimation`; this system only updates
/// `PlayerAnimState`. The actual visual animation is handled by
/// `animate_player_procedural`.
pub fn update_player_anim_state(
    mut query: Query<
        (
            &LinearVelocity,
            &mut PlayerAnimState,
            Option<&Invulnerable>,
        ),
        With<Player>,
    >,
) {
    let Ok((velocity, mut anim_state, invulnerable)) = query.single_mut() else {
        return;
    };

    // Hysteresis: enter Walking at >30, leave it below 8 — prevents flickering
    // when velocity hovers near the threshold on start/stop.
    let walk_speed = velocity.x.abs();
    let enter_walk = walk_speed > 30.0;
    let leave_walk = walk_speed < 8.0;

    let new_state = if invulnerable.is_some() {
        PlayerAnimState::Hurt
    } else if velocity.y.abs() > 50.0 {
        PlayerAnimState::Jumping
    } else if enter_walk || (*anim_state == PlayerAnimState::Walking && !leave_walk) {
        PlayerAnimState::Walking
    } else {
        PlayerAnimState::Idle
    };

    if new_state != *anim_state {
        *anim_state = new_state;
    }
}

/// Determines enemy animation state from velocity and flips sprite to face
/// movement direction. Cosmetic only — does not modify AI, physics, or movement.
pub fn update_enemy_anim_state(
    mut query: Query<
        (
            &LinearVelocity,
            &mut EnemyAnimState,
            &mut SpriteAnimation,
            &mut Transform,
        ),
        With<Enemy>,
    >,
) {
    for (velocity, mut anim_state, mut anim, mut transform) in query.iter_mut() {
        // Flip sprite to face movement direction.
        // Sprites face RIGHT in the sheet; negative scale.x flips to left.
        if velocity.x > 1.0 {
            transform.scale.x = transform.scale.x.abs();
        } else if velocity.x < -1.0 {
            transform.scale.x = -transform.scale.x.abs();
        }

        let new_state = if velocity.x.abs() > 5.0 {
            EnemyAnimState::Walking
        } else {
            EnemyAnimState::Idle
        };

        if new_state != *anim_state {
            *anim_state = new_state;
            anim.current_frame = 0;

            // Enemy sheets: 4×2 grid (512×256, 128px cells)
            // Row 0 (0-3): walk/motion cycle
            // Row 1 (4-7): idle/alert variants
            anim.frames = match new_state {
                EnemyAnimState::Idle => vec![0],
                EnemyAnimState::Walking => vec![0, 1, 2, 3],
            };
        }
    }
}

/// Ticks sprite animation timers and updates mesh UVs.
/// Operates on ALL entities with SpriteAnimation + Mesh3d (enemies only now
/// that the player uses a static 3D model with procedural animation).
pub fn animate_sprites(
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<(&mut SpriteAnimation, &Mesh3d)>,
) {
    for (mut anim, mesh3d) in query.iter_mut() {
        anim.timer.tick(time.delta());

        if anim.timer.just_finished() {
            if anim.looping {
                anim.current_frame = (anim.current_frame + 1) % anim.frames.len();
            } else if anim.current_frame + 1 < anim.frames.len() {
                anim.current_frame += 1;
            }
        }

        let frame_index = anim.frames[anim.current_frame];

        // Only rewrite mesh UVs when the frame actually changes.
        if frame_index == anim.last_written_frame {
            continue;
        }
        anim.last_written_frame = frame_index;

        let [u_min, v_min, u_max, v_max] = anim.atlas.uv_for_index(frame_index);

        // Rectangle vertex order: TR(0), TL(1), BL(2), BR(3)
        let new_uvs: Vec<[f32; 2]> = vec![
            [u_max, v_min],  // 0 = TR
            [u_min, v_min],  // 1 = TL
            [u_min, v_max],  // 2 = BL
            [u_max, v_max],  // 3 = BR
        ];

        if let Some(mesh) = meshes.get_mut(&mesh3d.0) {
            mesh.insert_attribute(
                Mesh::ATTRIBUTE_UV_0,
                VertexAttributeValues::Float32x2(new_uvs),
            );
        }
    }
}

/// Polls for the player entity with `PlayerModelPending` and walks its
/// descendants to find the `AnimationPlayer` that Bevy inserts when the GLB
/// scene with a skin is loaded. Once found, starts async loading of the 4
/// animation clips and inserts `PlayerClipsPending` with the handles.
///
/// Does NOT build the graph or start animations — that is deferred to
/// `finalize_player_animation` which waits for all clips to finish loading.
/// This two-phase approach fixes the race condition where Bevy silently
/// skips unloaded clips and the repeating animation never retries.
///
/// This system runs once: it removes `PlayerModelPending` after finding the
/// AnimationPlayer, so subsequent frames skip it entirely.
pub fn setup_player_animation(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player_query: Query<(Entity, &Children), (With<Player>, With<PlayerModelPending>)>,
    children_query: Query<&Children>,
    anim_player_query: Query<&AnimationPlayer>,
) {
    let Ok((player_entity, top_children)) = player_query.single() else {
        return;
    };

    // Walk all descendants (breadth-first) to find the entity with AnimationPlayer.
    // The GLB loader places AnimationPlayer on the armature entity, which is
    // several levels deep in the scene hierarchy.
    let mut queue: Vec<Entity> = top_children.iter().collect();
    let mut anim_entity: Option<Entity> = None;
    let mut visited = 0u32;

    while let Some(entity) = queue.pop() {
        visited += 1;
        if anim_player_query.get(entity).is_ok() {
            anim_entity = Some(entity);
            break;
        }
        if let Ok(grandchildren) = children_query.get(entity) {
            queue.extend(grandchildren.iter());
        }
    }

    info!("[ANIM] setup: searched {visited} descendants, found AnimationPlayer: {}", anim_entity.is_some());

    if let Some(ae) = anim_entity {
        info!("[ANIM] setup: anim_entity={ae:?}");
    }

    let Some(anim_entity) = anim_entity else {
        // GLB hasn't finished loading yet — AnimationPlayer not spawned.
        // Will retry next frame while PlayerModelPending is still present.
        return;
    };

    // Load the 4 animation clips from jasper.glb.
    // jasper7: 4 named animations — 0="hit", 1="idle", 2="jump", 3="walk"
    let clip_idle: Handle<AnimationClip> =
        asset_server.load(GltfAssetLabel::Animation(1).from_asset("models/jasper.glb"));
    let clip_walk: Handle<AnimationClip> =
        asset_server.load(GltfAssetLabel::Animation(3).from_asset("models/jasper.glb"));
    let clip_jump: Handle<AnimationClip> =
        asset_server.load(GltfAssetLabel::Animation(2).from_asset("models/jasper.glb"));
    let clip_hurt: Handle<AnimationClip> =
        asset_server.load(GltfAssetLabel::Animation(0).from_asset("models/jasper.glb"));

    info!("[ANIM] clips queued for async load, inserting PlayerClipsPending");

    // Store handles on the player entity so finalize_player_animation can
    // poll their load status each frame.
    commands.entity(player_entity).insert(PlayerClipsPending {
        anim_entity,
        clip_idle,
        clip_walk,
        clip_jump,
        clip_hurt,
    });

    // Remove the polling marker — setup phase is done; finalize takes over.
    commands.entity(player_entity).remove::<PlayerModelPending>();
}

/// Waits for all 4 animation clips to finish loading, then builds the
/// animation graph and wires it up. Runs every frame while
/// `PlayerClipsPending` is present; returns early if any clip is not yet
/// loaded. Removes `PlayerClipsPending` once the graph is built.
///
/// This is the second phase of the two-phase animation setup. The split
/// ensures the AnimationGraph is only built with fully loaded clips,
/// preventing the race condition where Bevy silently skips unloaded clips.
pub fn finalize_player_animation(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut anim_graphs: ResMut<Assets<AnimationGraph>>,
    pending_query: Query<(Entity, &PlayerClipsPending, &Children), With<Player>>,
    visual_query: Query<Entity, With<PlayerModelVisual>>,
    children_query: Query<&Children>,
    name_query: Query<&Name>,
    transform_query: Query<&Transform>,
) {
    let Ok((player_entity, pending, player_children)) = pending_query.single() else {
        return;
    };

    // Check if all 4 clips are fully loaded (including dependencies).
    let all_loaded = asset_server.is_loaded_with_dependencies(&pending.clip_idle)
        && asset_server.is_loaded_with_dependencies(&pending.clip_walk)
        && asset_server.is_loaded_with_dependencies(&pending.clip_jump)
        && asset_server.is_loaded_with_dependencies(&pending.clip_hurt);

    if !all_loaded {
        // Not ready yet — will retry next frame.
        return;
    }

    info!("[ANIM] all clips loaded, building animation graph");

    // Build the animation graph: root → 4 clip nodes.
    // Using from_clip + add_clip (proven working approach).
    let (mut graph, idle_index) = AnimationGraph::from_clip(pending.clip_idle.clone());
    let walk_index = graph.add_clip(pending.clip_walk.clone(), 1.0, graph.root);
    let jump_index = graph.add_clip(pending.clip_jump.clone(), 1.0, graph.root);
    let hurt_index = graph.add_clip(pending.clip_hurt.clone(), 1.0, graph.root);

    let graph_handle = anim_graphs.add(graph);

    let anim_entity = pending.anim_entity;

    // Insert the AnimationGraphHandle on the anim_entity so Bevy's animation
    // system knows which graph to evaluate for this AnimationPlayer.
    commands
        .entity(anim_entity)
        .insert(AnimationGraphHandle(graph_handle));

    info!("[ANIM] graph wired: player={player_entity:?} anim={anim_entity:?} idle={idle_index:?} walk={walk_index:?} jump={jump_index:?} hurt={hurt_index:?}");

    // Store the graph wiring on the player entity for drive_player_animation.
    commands.entity(player_entity).insert(PlayerAnimGraph {
        anim_entity,
        idle: idle_index,
        walk: walk_index,
        jump: jump_index,
        hurt: hurt_index,
        current: PlayerAnimState::Idle,
    });

    // Find and tag the "Root" bone to pin its Y position (prevents walk
    // animation root motion from drifting the character underground).
    {
        let mut bone_queue: Vec<Entity> = vec![anim_entity];
        while let Some(e) = bone_queue.pop() {
            if let Ok(name) = name_query.get(e) {
                if name.as_str() == "Root" {
                    if let Ok(bone_transform) = transform_query.get(e) {
                        commands.entity(e).insert(PlayerRootBone {
                            original_y: bone_transform.translation.y,
                        });
                        info!("[ANIM] tagged Root bone {e:?} original_y={}", bone_transform.translation.y);
                    }
                    break;
                }
            }
            if let Ok(children) = children_query.get(e) {
                bone_queue.extend(children.iter());
            }
        }
    }

    // Finalization complete — remove the pending marker.
    commands.entity(player_entity).remove::<PlayerClipsPending>();

    // Remove SceneRoot from the visual child entity to stop Bevy's scene
    // spawner from re-syncing the scene hierarchy on subsequent frames.
    // Without this, scene sync overwrites our AnimationGraphHandle insertion,
    // causing the AnimationPlayer to "disappear" from queries ~200ms later.
    for child in player_children.iter() {
        if visual_query.get(child).is_ok() {
            commands.entity(child).remove::<SceneRoot>();
            info!("[ANIM] removed SceneRoot from visual child {child:?} to prevent scene re-sync");
        }
    }

    info!("[ANIM] finalize complete");
}

/// One-frame diagnostic: counts AnimationTarget components and checks if the
/// AnimationPlayer has an active graph. Runs every frame but only logs once.
pub fn debug_animation_state(
    player_query: Query<&PlayerAnimGraph, With<Player>>,
    anim_player_query: Query<(&AnimationPlayer, Option<&AnimationGraphHandle>)>,
    targets: Query<&bevy::animation::AnimationTargetId>,
    mut logged: Local<bool>,
    time: Res<Time>,
) {
    // Only log once, 2 seconds after startup
    if *logged || time.elapsed_secs() < 3.0 {
        return;
    }

    let Ok(anim_graph) = player_query.single() else {
        return;
    };

    *logged = true;

    let target_count = targets.iter().count();
    info!("[ANIM-DEBUG] Total AnimationTarget entities in world: {target_count}");

    // Check if the entity even exists
    let entity_exists = anim_player_query.get(anim_graph.anim_entity).is_ok();
    info!("[ANIM-DEBUG] anim_entity {:?} query result: exists={entity_exists}", anim_graph.anim_entity);

    if let Ok((anim_player, graph_handle)) = anim_player_query.get(anim_graph.anim_entity) {
        let has_graph = graph_handle.is_some();
        let playing = anim_player.playing_animations().count();
        let active = anim_player
            .playing_animations()
            .filter(|(_, a)| !a.is_finished())
            .count();
        info!(
            "[ANIM-DEBUG] AnimationPlayer on {:?}: has_graph={has_graph} playing={playing} active={active}",
            anim_graph.anim_entity
        );
    }

    // Also check: does drive_player_animation's query type find it?
    info!("[ANIM-DEBUG] drive query would find it: {}",
        anim_player_query.get(anim_graph.anim_entity).is_ok());
}

/// Drives skeletal animation based on the current `PlayerAnimState`.
///
/// Reads the player's `PlayerAnimState` and `PlayerAnimGraph`, then starts
/// the appropriate animation clip on the `AnimationPlayer` descendant.
///
/// Runs after `update_player_anim_state` (via chain ordering in mod.rs).
/// The chain auto-inserts `apply_deferred` between systems, so deferred
/// commands from `setup_player_animation` are guaranteed to be applied
/// before this system runs.
pub fn drive_player_animation(
    mut player_query: Query<(&PlayerAnimState, &mut PlayerAnimGraph), With<Player>>,
    mut anim_player_query: Query<&mut AnimationPlayer>,
) {
    let Ok((anim_state, mut anim_graph)) = player_query.single_mut() else {
        return;
    };

    let desired_index = match *anim_state {
        PlayerAnimState::Idle => anim_graph.idle,
        PlayerAnimState::Walking => anim_graph.walk,
        PlayerAnimState::Jumping => anim_graph.jump,
        PlayerAnimState::Hurt => anim_graph.hurt,
    };

    // Check if the state matches what's currently playing AND something is
    // actively running (not finished). If so, skip — don't restart mid-animation.
    if *anim_state == anim_graph.current {
        match anim_player_query.get(anim_graph.anim_entity) {
            Ok(anim_player) => {
                let something_playing = anim_player
                    .playing_animations()
                    .any(|(_, active)| !active.is_finished());
                if something_playing {
                    return;
                }
                // Nothing playing — fall through to restart
                info!("[ANIM] drive: state={:?} matches current but nothing playing, restarting", *anim_state);
            }
            Err(_) => {
                info!("[ANIM] drive: CANNOT FIND AnimationPlayer on {:?}!", anim_graph.anim_entity);
                return;
            }
        }
    }

    // State changed or current animation finished — switch clips.
    let Ok(mut anim_player) = anim_player_query.get_mut(anim_graph.anim_entity) else {
        return;
    };

    // Diagnostic: check playing state before we act
    let playing_count: usize = anim_player.playing_animations().count();
    let active_count: usize = anim_player
        .playing_animations()
        .filter(|(_, a)| !a.is_finished())
        .count();
    info!(
        "[ANIM] drive: before stop — playing={playing_count} active={active_count}"
    );

    anim_player.stop_all();

    info!("[ANIM] drive: starting {:?} (index {:?})", *anim_state, desired_index);

    match *anim_state {
        PlayerAnimState::Idle | PlayerAnimState::Walking => {
            // Looping animations for idle and walk.
            anim_player.start(desired_index).repeat();
        }
        PlayerAnimState::Jumping | PlayerAnimState::Hurt => {
            // One-shot animations for jump and hurt.
            anim_player.start(desired_index);
        }
    }

    anim_graph.current = *anim_state;
}

/// Resets the root bone's Y translation after Bevy's animation evaluation.
/// Prevents walk animation root motion from accumulating vertical drift.
pub fn pin_player_root_bone(
    mut query: Query<(&mut Transform, &PlayerRootBone)>,
) {
    for (mut transform, root_bone) in &mut query {
        transform.translation.y = root_bone.original_y;
    }
}

/// Procedural animation for the player's static 3D model (no bones/skeleton).
///
/// Reads `PlayerAnimState` and `FacingDirection` from the physics parent,
/// then modifies the child `PlayerModelVisual` transform to create visual
/// animation effects (bob, stretch, shake) and handle facing rotation.
///
/// When skeletal animation is active (`PlayerAnimGraph` is present on the
/// player entity), this system ONLY sets facing rotation and base
/// translation/scale — the skeleton handles the visual animation through
/// bone transforms. This preserves facing direction which the skeletal
/// system does not control.
///
/// When skeletal animation is NOT active (first few frames before GLB loads,
/// or if the model has no skeleton), full procedural animation runs as
/// fallback.
///
/// This system ALSO handles facing rotation (previously in
/// `update_player_visual_facing`) to avoid conflicting Transform writes
/// between systems. The facing rotation is composed with animation effects.
///
/// Base child transform values (from controller.rs):
/// - translation: Vec3::new(0.0, model_y_offset, 0.0) where model_y_offset = -8.0
/// - scale: Vec3::splat(35.0)
/// - rotation: set by facing direction
///
/// WHAT BREAKS if base values change: the procedural offsets will be relative
/// to wrong base positions, causing visual misalignment with the collider.
pub fn animate_player_procedural(
    time: Res<Time>,
    parent_query: Query<
        (&PlayerAnimState, &FacingDirection, &Children, Option<&PlayerAnimGraph>),
        With<Player>,
    >,
    mut visual_query: Query<&mut Transform, With<PlayerModelVisual>>,
) {
    let Ok((anim_state, facing, children, anim_graph)) = parent_query.single() else {
        return;
    };

    // Whether skeletal animation is handling the visual movement.
    let skeletal_active = anim_graph.is_some();

    let t = time.elapsed_secs();

    // Base values matching controller.rs spawn — DO NOT change independently.
    // model_y_offset = -float_height + 8.0 = -3.0
    let base_y: f32 = -3.0;
    let base_scale: f32 = 37.1;

    // Facing rotation: model faces +X (right) natively.
    // Right = -45° Y rotation for camera read angle.
    // Left = 225° Y rotation with +15° X tilt compensation.
    let facing_rotation = match *facing {
        FacingDirection::Right => Quat::from_rotation_y((-45.0_f32).to_radians()),
        FacingDirection::Left => {
            Quat::from_rotation_y((225.0_f32).to_radians())
                * Quat::from_rotation_x((15.0_f32).to_radians())
        }
    };

    for child in children.iter() {
        let Ok(mut transform) = visual_query.get_mut(child) else {
            continue;
        };

        if skeletal_active {
            // Skeletal animation is driving the visual movement via bone
            // transforms. We only set facing rotation and keep base
            // translation/scale so the model stays aligned with the collider.
            // No bob, stretch, shake, or lean — the skeleton handles that.
            transform.translation = Vec3::new(0.0, base_y, 0.0);
            transform.scale = Vec3::splat(base_scale);
            transform.rotation = facing_rotation;
            continue;
        }

        // Full procedural animation fallback (no skeleton available).
        match *anim_state {
            PlayerAnimState::Idle => {
                // Gentle breathing bob — subtle up/down oscillation.
                // sin(time * 2.0) gives ~0.3 Hz cycle, amplitude 0.5 world units.
                let bob = (t * 2.0).sin() * 0.5;
                transform.translation = Vec3::new(0.0, base_y + bob, 0.0);

                // Gentle scale pulse on Y axis for breathing feel.
                transform.scale = Vec3::new(
                    base_scale,
                    base_scale + (t * 2.0).sin() * 0.3,
                    base_scale,
                );

                // No additional rotation beyond facing.
                transform.rotation = facing_rotation;
            }
            PlayerAnimState::Walking => {
                // Faster bob — bouncy walk cycle at ~1.6 Hz.
                let bob = (t * 10.0).sin() * 1.0;
                transform.translation = Vec3::new(0.0, base_y + bob, 0.0);

                // Walking uses base uniform scale (no squash/stretch).
                transform.scale = Vec3::splat(base_scale);

                // Slight forward/back lean while walking — 3° oscillation on X axis.
                // Applied on top of facing rotation for a natural gait feel.
                let lean = Quat::from_rotation_x((t * 10.0).sin() * 3.0_f32.to_radians());
                transform.rotation = facing_rotation * lean;
            }
            PlayerAnimState::Jumping => {
                // Vertical stretch (squash and stretch principle).
                // Y stretched, X/Z compressed to maintain volume feel.
                transform.translation = Vec3::new(0.0, base_y + 1.0, 0.0);
                transform.scale = Vec3::new(base_scale * 0.825, base_scale * 0.925, base_scale * 0.825);

                // No additional rotation beyond facing.
                transform.rotation = facing_rotation;
            }
            PlayerAnimState::Hurt => {
                // Quick horizontal shake — high frequency (30 Hz-ish) for impact feel.
                let shake_x = (t * 30.0).sin() * 1.5;
                transform.translation = Vec3::new(shake_x, base_y, 0.0);

                // Rapid scale pulse for flash/impact effect.
                let pulse = (t * 20.0).sin().abs() * 1.5;
                transform.scale = Vec3::splat(base_scale + pulse);

                // No additional rotation beyond facing.
                transform.rotation = facing_rotation;
            }
        }
    }
}
