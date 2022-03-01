use bevy::prelude::*;

use crate::{ldtk_loader::LdtkMap, GameState};

use super::{MapPosition, spawn::{SpawnUnit, SpawnEntity, Team, BATTLE_MAP_SPAWN_SYSTEM}, map::BattleMapLdtkHandle};

pub struct EnemiesPlugin;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(GameState::BattleMap)
            .with_system(spawn)
            .before(BATTLE_MAP_SPAWN_SYSTEM)
        );
    }
}


#[derive(Component)]
pub struct EnemySpawner {
    pub timer: Timer,
}

fn spawn(
    mut commands: Commands,
    time: Res<Time>,
    mut q_spawner: Query<(&mut EnemySpawner, &MapPosition)>,
    ldtk_handle: Res<BattleMapLdtkHandle>,
) {
        //println!("SPAAAWN");
        for (mut spawner, pos) in q_spawner.iter_mut() {
            //println!("Spawners running?");
            spawner.timer.tick(time.delta());
            
            if spawner.timer.just_finished() {
                //println!("Trying to spawn slime");
                let xyz = (pos.xy + IVec2::new(0,-1)).extend(2);
                let spawn = SpawnEntity {
                    ldtk: ldtk_handle.0.clone(),
                    name: "Slime".to_string(),
                    pos: xyz,
                };
                commands.spawn().insert(spawn);
            }
        }
}