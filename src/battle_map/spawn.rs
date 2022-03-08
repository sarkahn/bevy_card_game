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
        app.add_system(despawn_tick)
            .add_system_to_stage(CoreStage::PostUpdate, despawn_timer)
            ;
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
pub struct DespawnTimer(Timer);

impl std::ops::Deref for DespawnTimer {
    type Target = Timer;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for DespawnTimer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl DespawnTimer {
    pub fn new(time: f32) -> Self {
        Self(Timer::from_seconds(time, false))
    }
}


fn despawn_tick(time: Res<Time>, mut q_timers: Query<&mut DespawnTimer>) {
    for mut timer in q_timers.iter_mut() {
        timer.0.tick(time.delta());
    }
}

fn despawn_timer(mut commands: Commands, mut q_timers: Query<(Entity, &mut DespawnTimer)>) {
    for (entity, timer) in q_timers.iter_mut() {
        if timer.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}
