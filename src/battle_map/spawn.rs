use bevy::{prelude::*, math::Vec3Swizzles, ecs::system::EntityCommands};
use rand::{thread_rng, prelude::SliceRandom};

use crate::{
    TILE_SIZE, 
    //SpawnPrefabOld, 
    //prefab::{SpawnPrefab, SpawnType}
};

pub struct MapSpawnPlugin;

impl Plugin for MapSpawnPlugin {
    fn build(&self, app: &mut App) {
        // app
        // .add_system(run_spawner)
        // ;
    }
}

#[derive(Component)]
pub struct Spawner(pub Timer);

impl std::ops::Deref for Spawner {
    type Target = Timer;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Spawner {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Component)]
pub struct DespawnTimer(pub Timer);

// #[derive(Component)]
// pub struct Spawner {
//     timer: Timer,
//     entities: Vec<String>,
// }

// #[derive(Component)]
// pub struct SpawnCommands(pub fn(&mut EntityCommands));

// impl Spawner {
//     pub fn new<'a>(timer: Timer, prefab_names: &[&str]) -> Self {
//         Self {
//             timer,
//             entities: prefab_names.iter().map(|s|s.to_string()).collect(),
//         }
//     }
// }

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

// fn run_spawner(
//     mut commands: Commands, 
//     time: Res<Time>,
//     mut q_spawner: Query<(&mut Spawner, &Transform,&SpawnCommands)>,
// ) {
//     for (mut spawner, transform, spawn_commands) in q_spawner.iter_mut() {
//         spawner.timer.tick(time.delta());

//         if spawner.timer.just_finished() {
//             let mut rng = thread_rng();

//             if let Some(to_spawn) = spawner.entities.choose(&mut rng) {
//                 let p = transform.translation;
//                 println!("Spawner pos: {}", p);
//                 let p = p + Vec3::new(0.0, -1.0, 0.0) * TILE_SIZE as f32;

//                 let mut entity = commands.spawn();
//                 spawn_commands.0(&mut entity);

//                 // commands.spawn().insert(SpawnPrefab::new(to_spawn, p.xy(), 10, SpawnType::Map))
//                 // .insert(SpawnCommands);
//             }
//         }
//     }
// }