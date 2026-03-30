use bevy::prelude::*;

use crate::collectibles::components::CollectedEvent;
use crate::combat::components::EnemyKillEvent;
use crate::enemies::components::Enemy;
use crate::player::components::Player;

use super::components::Particle;

/// Ticks particle lifetimes, moves particles by velocity, and despawns expired ones.
pub fn tick_particles(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut Particle,
        &mut Transform,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (entity, mut particle, mut transform, mat_handle) in query.iter_mut() {
        particle.lifetime.tick(time.delta());

        if particle.lifetime.just_finished() {
            commands.entity(entity).despawn();
            continue;
        }

        // Move particle
        transform.translation.x += particle.velocity.x * dt;
        transform.translation.y += particle.velocity.y * dt;

        // Fade alpha over lifetime
        if particle.fade {
            let remaining = 1.0 - particle.lifetime.fraction();
            if let Some(mat) = materials.get_mut(&mat_handle.0) {
                mat.base_color.set_alpha(remaining);
            }
        }
    }
}

fn spawn_burst(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    position: Vec3,
    count: u32,
    color: Color,
    speed: f32,
) {
    let mesh_handle = meshes.add(Rectangle::new(4.0, 4.0));

    for i in 0..count {
        let angle = (i as f32 / count as f32) * std::f32::consts::TAU;
        let vel_x = angle.cos() * speed;
        let vel_y = angle.sin() * speed;

        let mat = materials.add(StandardMaterial {
            base_color: color,
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        });

        commands.spawn((
            Particle {
                velocity: Vec2::new(vel_x, vel_y),
                lifetime: Timer::from_seconds(0.5, TimerMode::Once),
                fade: true,
            },
            Mesh3d(mesh_handle.clone()),
            MeshMaterial3d(mat),
            Transform::from_translation(position),
        ));
    }
}

/// Listens to CollectedEvent and spawns a yellow burst at the player's position.
pub fn spawn_collect_burst(
    mut events: MessageReader<CollectedEvent>,
    player_query: Query<&Transform, With<Player>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    for _ in events.read() {
        let pos = player_transform.translation;
        spawn_burst(
            &mut commands,
            &mut meshes,
            &mut materials,
            Vec3::new(pos.x, pos.y, pos.z + 0.5),
            6,
            Color::srgb(1.0, 0.9, 0.0),
            80.0,
        );
    }
}

/// Listens to EnemyKillEvent and spawns red particles at the enemy's last known position.
pub fn spawn_kill_burst(
    mut events: MessageReader<EnemyKillEvent>,
    enemy_query: Query<&Transform, With<Enemy>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for event in events.read() {
        // Try to get the enemy's position before it is despawned.
        // process_kills (in CombatPlugin) runs after us, so the entity still exists.
        if let Ok(transform) = enemy_query.get(event.enemy) {
            let pos = transform.translation;
            spawn_burst(
                &mut commands,
                &mut meshes,
                &mut materials,
                Vec3::new(pos.x, pos.y, pos.z + 0.5),
                5,
                Color::srgb(1.0, 0.2, 0.1),
                100.0,
            );
        }
    }
}
