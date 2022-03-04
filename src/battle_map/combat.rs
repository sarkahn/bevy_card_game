use bevy::{math::Vec3Swizzles, prelude::*};

use crate::{
    arena::{ArenaState, ArenaCombat}, config::ConfigAsset, ldtk_loader::LdtkMap, make_sprite, GameState,
    SETTINGS_PATH, GridHelper, TILE_SIZE,
};

use super::{
    spawn::{DespawnTimer, SpawnEntity},
    units::EnemyUnit,
     MapUnits, map::CollisionMap,
};

pub struct MapCombatPlugin;

impl Plugin for MapCombatPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system_set(SystemSet::on_update(GameState::BattleMap).with_system(on_collision))
            .add_system_set(
                SystemSet::on_update(GameState::BeginningCombat).with_system(begin_combat),
            )
            ;
    }
}

fn on_collision(
    mut commands: Commands,
    q_enemies: Query<(Entity, &Transform), (With<EnemyUnit>, Changed<Transform>)>,
    units: Res<MapUnits>,
    mut state: ResMut<State<GameState>>,
    config: Res<Assets<ConfigAsset>>,
    asset_server: Res<AssetServer>,
    map: Res<CollisionMap>,
) {
    if let Some(config) = config.get(SETTINGS_PATH) {
        let ldtk: Handle<LdtkMap> = asset_server.load(&config.settings.map_file);
        //if let Ok(ldtk_handle) = asset_server.load::<LdtkMap>(&config.settings.map_file) {
        for (enemy, transform) in q_enemies.iter() {
            let xy = units.xy_to_grid(transform.translation.xy());
            if let Some(player) = units.get_from_grid_xy(xy) {
                let mut pos = transform.translation;
                pos += Vec3::new(0.0, 0.0, 1.0);
                commands
                    .spawn()
                    .insert(SpawnEntity {
                        ldtk: ldtk.clone(),
                        name: "BeginCombat".to_string(),
                        pos,
                    })
                    .insert(BeginCombat { player, enemy })
                    .insert(DespawnTimer(Timer::from_seconds(3.0, false)));
                    state.set(GameState::BeginningCombat).unwrap();
            }
        }
    }
}

#[derive(Component)]
pub struct BeginCombat {
    player: Entity,
    enemy: Entity,
}

fn begin_combat(
    mut commands: Commands,
    mut q_begin: Query<(Entity, &mut BeginCombat, &DespawnTimer)>,
    mut state: ResMut<State<GameState>>,
    mut arena_state: ResMut<State<ArenaState>>,
    time: Res<Time>,
) {
    for (entity, mut begin, timer) in q_begin.iter_mut() {
        if timer.0.finished() {
            //arena_units.player_party = begin.player;
            //arena_units.enemy_party = begin.enemy;
            println!("Combat is beginning!");
            state.set(GameState::LoadArena).unwrap();
        }
    }
}
