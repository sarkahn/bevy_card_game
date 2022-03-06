use bevy::{prelude::*, math::Vec3Swizzles};
use rand::{thread_rng, prelude::SliceRandom};

use crate::{TILE_SIZE, SpawnPrefabOld, prefab::{SpawnPrefab, SpawnType}};

pub struct MapSpawnPlugin;

impl Plugin for MapSpawnPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_system(run_spawner)
        ;
    }
}

#[derive(Component)]
pub struct DespawnTimer(pub Timer);

#[derive(Component)]
pub struct Spawner {
    timer: Timer,
    entities: Vec<String>,
}

impl Spawner {
    pub fn new<'a>(timer: Timer, prefab_names: &[&str]) -> Self {
        Self {
            timer,
            entities: prefab_names.iter().map(|s|s.to_string()).collect(),
        }
    }
}

fn despawn_tick(time: Res<Time>, mut q_timers: Query<&mut DespawnTimer>) {
    for mut timer in q_timers.iter_mut() {
        timer.0.tick(time.delta());
    }
}

fn despawn_timer(mut commands: Commands, mut q_timers: Query<(Entity, &mut DespawnTimer)>) {
    for (entity, mut timer) in q_timers.iter_mut() {
        if timer.0.finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn run_spawner(
    mut commands: Commands, 
    time: Res<Time>,
    mut q_spawner: Query<(&mut Spawner, &Transform)>,
) {
    for (mut spawner, transform) in q_spawner.iter_mut() {
        spawner.timer.tick(time.delta());

        if spawner.timer.just_finished() {
            let mut rng = thread_rng();

            if let Some(to_spawn) = spawner.entities.choose(&mut rng) {
                let p = transform.translation;
                println!("Spawner pos: {}", p);
                let p = p + Vec3::new(0.0, -1.0, 0.0) * TILE_SIZE as f32;

                commands.spawn().insert(SpawnPrefab::new(to_spawn, p.xy(), 10, SpawnType::Map));
            }
        }
    }
}