use bevy::prelude::*;

use crate::combat::components::PlayerDamageEvent;
use crate::level::level_data::{CurrentLevel, LevelId};
use crate::particles::components::Particle;
use crate::rendering::camera::GameplayCamera;

use super::components::{
    CameraRelativeVfx, LevelNameFlash, ScreenFlash, WeatherEmitter, WeatherType,
};

// ── Weather ──────────────────────────────────────────────────────────────────

/// On level change, despawn old WeatherEmitter and spawn a new one appropriate
/// for the level's biome.
pub fn update_weather(
    mut commands: Commands,
    current_level: Res<CurrentLevel>,
    emitter_query: Query<Entity, With<WeatherEmitter>>,
) {
    if !current_level.is_changed() {
        return;
    }

    // Despawn existing emitters
    for entity in &emitter_query {
        commands.entity(entity).despawn();
    }

    // No weather effects in underground sublevels (layer 1).
    if current_level.layer_index == 1 {
        return;
    }

    let (particle_type, interval_secs) = match current_level.level_id {
        Some(LevelId::Forest) => (WeatherType::Leaves, 0.15),
        Some(LevelId::Subdivision) => (WeatherType::Rain, 0.08),
        Some(LevelId::City) => (WeatherType::Dust, 0.12),
        _ => return,
    };

    commands.spawn(WeatherEmitter {
        spawn_timer: Timer::from_seconds(interval_secs, TimerMode::Repeating),
        particle_type,
    });
}

/// Ticks the WeatherEmitter and spawns particles each interval.
pub fn emit_weather_particles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut emitter_query: Query<&mut WeatherEmitter>,
    // WHY GameplayCamera: bare With<Camera3d> is invalid per camera_role_identity_guardrail.
    // Previously used .iter().next() to dodge ambiguity — role marker makes .single() safe.
    camera_query: Query<&Transform, (With<Camera3d>, With<GameplayCamera>)>,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
) {
    use std::f32::consts::TAU;

    let cam_pos = camera_query
        .single()
        .map(|t| t.translation)
        .unwrap_or(Vec3::ZERO);

    for mut emitter in emitter_query.iter_mut() {
        emitter.spawn_timer.tick(time.delta());
        if !emitter.spawn_timer.just_finished() {
            continue;
        }

        // Deterministic "random" using time as seed — simple pseudo-random
        let t = time.elapsed_secs();
        let r1 = (t * 137.508).sin() * 0.5 + 0.5; // 0..1
        let r2 = (t * 251.317 + 1.0).sin() * 0.5 + 0.5; // 0..1
        let r3 = (t * 317.432 + 2.0).sin() * 0.5 + 0.5; // 0..1

        match emitter.particle_type {
            WeatherType::Rain => {
                // Thin blue-white rain streaks — no texture needed.
                let x = cam_pos.x + (r1 - 0.5) * 400.0;
                let spawn_y = cam_pos.y + 175.0;
                let drift_x = (r2 - 0.5) * 15.0;
                let tint = Color::srgba(0.7 + r3 * 0.2, 0.75 + r3 * 0.2, 0.9 + r3 * 0.1, 0.7);
                let mesh = meshes.add(Rectangle::new(2.0, 10.0));
                let mat = materials.add(StandardMaterial {
                    base_color: tint,
                    unlit: true,
                    alpha_mode: AlphaMode::Blend,
                    double_sided: true,
                    cull_mode: None,
                    ..default()
                });
                commands.spawn((
                    Particle {
                        velocity: Vec2::new(drift_x, -200.0),
                        lifetime: Timer::from_seconds(3.0, TimerMode::Once),
                        fade: true,
                    },
                    Mesh3d(mesh),
                    MeshMaterial3d(mat),
                    Transform::from_xyz(x, spawn_y, 20.0),
                    CameraRelativeVfx,
                ));
            }
            WeatherType::Dust => {
                // Small tan/beige dust motes — slow drift, no texture.
                let x = cam_pos.x + (r1 - 0.5) * 400.0;
                let spawn_y = cam_pos.y + 175.0;
                let drift_x = (r2 - 0.5) * 40.0;
                let tint = Color::srgba(
                    0.65 + r3 * 0.15, // warm tan
                    0.58 + r3 * 0.12,
                    0.42 + r3 * 0.13,
                    0.5,
                );
                let mesh = meshes.add(Rectangle::new(4.0, 4.0));
                let mat = materials.add(StandardMaterial {
                    base_color: tint,
                    unlit: true,
                    alpha_mode: AlphaMode::Blend,
                    double_sided: true,
                    cull_mode: None,
                    ..default()
                });
                commands.spawn((
                    Particle {
                        velocity: Vec2::new(drift_x, -50.0),
                        lifetime: Timer::from_seconds(4.0, TimerMode::Once),
                        fade: true,
                    },
                    Mesh3d(mesh),
                    MeshMaterial3d(mat),
                    Transform::from_xyz(x, spawn_y, 20.0),
                    CameraRelativeVfx,
                ));
            }
            WeatherType::Leaves => {
                // CAMERA-RELATIVE VFX EXCEPTION — jasper_camera_world_anchor_guardrail_v2 Category 4.
                // Spawn volume is computed relative to the camera to maintain stable visual density
                // around the player regardless of level width. Once spawned, each leaf moves
                // independently in world-space under its own velocity. This is not a parallax layer
                // and is not subject to world-anchor or parallax rules.
                let x = cam_pos.x + (r1 - 0.5) * 400.0;
                let spawn_y = cam_pos.y + 175.0;
                let drift_x = (r2 - 0.5) * 30.0;
                let tint = Color::srgba(0.5 + r3 * 0.4, 0.7 + r3 * 0.2, 0.1 + r3 * 0.1, 0.9);
                let leaf_texture: Handle<Image> = asset_server.load("leaf.png");
                let mesh = meshes.add(Rectangle::new(12.0, 12.0));
                let mat = materials.add(StandardMaterial {
                    base_color: tint,
                    base_color_texture: Some(leaf_texture),
                    unlit: true,
                    alpha_mode: AlphaMode::Blend,
                    double_sided: true,
                    cull_mode: None,
                    ..default()
                });
                commands.spawn((
                    Particle {
                        velocity: Vec2::new(drift_x, -70.0),
                        lifetime: Timer::from_seconds(5.0, TimerMode::Once),
                        fade: true,
                    },
                    Mesh3d(mesh),
                    MeshMaterial3d(mat),
                    Transform::from_xyz(x, spawn_y, 20.0)
                        .with_rotation(Quat::from_rotation_z(r2 * TAU)),
                    CameraRelativeVfx,
                ));
            }
        }
    }
}

// ── Screen Flash ─────────────────────────────────────────────────────────────

/// Listens to PlayerDamageEvent and spawns a red screen-flash overlay.
pub fn flash_on_damage(mut commands: Commands, mut events: MessageReader<PlayerDamageEvent>) {
    for _ in events.read() {
        commands.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            BackgroundColor(Color::srgba(1.0, 0.0, 0.0, 0.35)),
            ScreenFlash {
                timer: Timer::from_seconds(0.2, TimerMode::Once),
                color: Color::srgba(1.0, 0.0, 0.0, 0.35),
            },
            // High z-index so it renders on top of the HUD
            ZIndex(200),
        ));
    }
}

/// Ticks the ScreenFlash timer, fades alpha, and despawns when done.
pub fn tick_screen_flash(
    mut commands: Commands,
    mut query: Query<(Entity, &mut ScreenFlash, &mut BackgroundColor)>,
    time: Res<Time>,
) {
    for (entity, mut flash, mut bg) in query.iter_mut() {
        flash.timer.tick(time.delta());

        let remaining = 1.0 - flash.timer.fraction();
        let base_alpha = flash.color.alpha();
        bg.0.set_alpha(base_alpha * remaining);

        if flash.timer.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}

// ── Level Name Flash ─────────────────────────────────────────────────────────

/// On level change, spawns a centered level-name text that fades out after 2 s.
pub fn flash_level_name(
    mut commands: Commands,
    current_level: Res<CurrentLevel>,
    existing: Query<Entity, With<LevelNameFlash>>,
) {
    if !current_level.is_changed() {
        return;
    }

    // Despawn any previous flash
    for entity in &existing {
        commands.entity(entity).despawn();
    }

    let name = match current_level.level_id {
        Some(LevelId::Forest) => "Forest",
        Some(LevelId::Subdivision) => "Subdivision",
        Some(LevelId::City) => "City",
        Some(LevelId::Sanctuary) => "Sanctuary",
        None => return,
    };

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::NONE),
            LevelNameFlash {
                timer: Timer::from_seconds(2.0, TimerMode::Once),
            },
            ZIndex(190),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(name),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 1.0, 0.8, 1.0)),
            ));
        });
}

/// Ticks the LevelNameFlash timer, fades the text, and despawns when done.
pub fn tick_level_name_flash(
    mut commands: Commands,
    mut query: Query<(Entity, &mut LevelNameFlash)>,
    mut text_query: Query<&mut TextColor>,
    children_query: Query<&Children>,
    time: Res<Time>,
) {
    for (entity, mut flash) in query.iter_mut() {
        flash.timer.tick(time.delta());

        let alpha = 1.0 - flash.timer.fraction();

        // Update child text alpha
        if let Ok(children) = children_query.get(entity) {
            for child in children.iter() {
                if let Ok(mut text_color) = text_query.get_mut(child) {
                    text_color.0.set_alpha(alpha);
                }
            }
        }

        if flash.timer.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}
