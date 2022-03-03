use bevy::prelude::*;

use crate::{ldtk_loader::LdtkMap, GameState, SETTINGS_PATH, config::ConfigAsset};

use super::{
    map::BattleMapLdtkHandle,
    //MapPosition,
    spawn::{SpawnEntity, SpawnUnit, SPAWN_SYSTEM},
};

pub struct EnemiesPlugin;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(GameState::BattleMap)
                .with_system(spawn)
                .before(SPAWN_SYSTEM),
        );
    }
}

#[derive(Component)]
pub struct Spawner {
    pub timer: Timer,
}

fn spawn(
    mut commands: Commands,
    time: Res<Time>,
    mut q_spawner: Query<(&mut Spawner, &Transform)>,
    asset: Res<AssetServer>,
    config: Res<Assets<ConfigAsset>>,
) {
    let config = config.get(SETTINGS_PATH).unwrap();
    let handle: Handle<LdtkMap> = asset.load(&config.settings.map_file);
    //println!("SPAAAWN");
    for (mut spawner, transform) in q_spawner.iter_mut() {
        //println!("Spawners running?");
        spawner.timer.tick(time.delta());

        if spawner.timer.just_finished() {
            //println!("Trying to spawn slime");
            let p = transform.translation + Vec3::new(0.0, -1.0, 0.0);
            //let xyz = (pos.xy + IVec2::new(0,-1)).extend(2);
            let spawn = SpawnEntity {
                ldtk: handle.clone(),
                name: "Slime".to_string(),
                pos: p,
            };
            commands.spawn().insert(spawn);
        }
    }
}
