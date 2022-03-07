use bevy::{prelude::*, ecs::system::EntityCommands, math::Vec3Swizzles};
use rand::{thread_rng, prelude::IteratorRandom, Rng};

use crate::{ldtk_loader::{LdtkMap, Tags, Fields}, GameState, AtlasHandles, animation::Animator, SETTINGS_PATH, config::ConfigAsset, 
//SpawnPrefabOld, 
prefab::Prefabs, TILE_SIZE, battle_map::{PlayerUnit, UnitCommands}, party::{GenerateParty, Party, PartyUnit}, 

GENERATE_PARTY_SYSTEM, LdtkHandles, LDTK_PLAYER_UNITS};

use super::{map::{BUILD_MAP_SYSTEM, CollisionMap}, spawn::Spawner, MapUnit, BattleMapEntity, MapUnits, get_valid_spawn_points, PlayerBase};

//use super::Spawner;

pub struct BattleMapPlayerPlugin;

impl Plugin for BattleMapPlayerPlugin {
    fn build(&self, app: &mut App) {
        app//.add_system_set(
        //    SystemSet::on_update(GameState::LoadBattleMap)
        //    .with_system(setup.before(BUILD_MAP_SYSTEM))
        //)
        .add_system_set(
            SystemSet::on_enter(GameState::BattleMap)
            .with_system(setup)
        )
        .add_system_set(
            SystemSet::on_update(GameState::BattleMap)
            .with_system(spawn.before(GENERATE_PARTY_SYSTEM))
        )
        .add_system_to_stage(CoreStage::PreUpdate, on_spawn)
        ;
    }
}

#[derive(Component)]
pub struct Player;


fn setup(
    asset_server: Res<AssetServer>,
    ldtk: Res<Assets<LdtkMap>>,
    mut commands: Commands,
    q_tags: Query<(Entity, &Tags, &Fields, &Name), (Added<Tags>, With<BattleMapEntity>)>,
    mut ldtk_handles: ResMut<LdtkHandles>,
) {
    for (entity, tags, fields, name) in q_tags.iter() {
        if tags.has_all(&["player", "spawner"]) {
            if let Some(min) = fields.try_get_f32("spawn_delay_min") {
                if let Some(max) = fields.try_get_f32("spawn_delay_max") {
                    let mut rng = thread_rng();
                    let delay: f32 = rng.gen_range(min..max);
                    //println!("Spawning player spawner!");
                    commands.entity(entity).insert(
                        Spawner(Timer::from_seconds(delay, true))
                    ).insert(Player)
                    ;
                }
            }
        }
        if tags.has("player_base") {
            commands.entity(entity).insert(PlayerBase);
        }
    }

    for name in LDTK_PLAYER_UNITS {
        if ldtk.get(*name).is_none() {
            ldtk_handles.0.push(asset_server.load(*name));
        }
    }
}

fn spawn(
    mut commands: Commands,
    time: Res<Time>,
    mut q_spawner: Query<(&Transform, &mut Spawner), With<Player>>,
    map_units: Res<MapUnits>,
    colliders: Res<CollisionMap>,
    prefabs: Res<Prefabs>,
    ldtk: Res<Assets<LdtkMap>>,
    configs: Res<Assets<ConfigAsset>>,
) {

    for (transform, mut spawner) in q_spawner.iter_mut() {

        // Ensure all our prefabs are loaded
        for name in LDTK_PLAYER_UNITS {
            if ldtk.get(&**name).is_none() {
                return;
            }
        }

        spawner.tick(time.delta());

        if spawner.just_finished() {

            let p = transform.translation;
            let curr = map_units.xy_to_grid(p.xy());
            let spawn_points = get_valid_spawn_points(curr, &map_units, &colliders);

            if let Some(spawn_points) = spawn_points {
                let mut rng = thread_rng(); 
                let spawn = spawn_points.choose(&mut rng).unwrap();
                let spawn = map_units.grid_to_xy(spawn);
                
                let pos = Vec3::new(spawn.x, spawn.y, 10.0);

                let names = LDTK_PLAYER_UNITS.iter().map(|s|s.to_string()).collect();
                //println!("Spawning gen...");
                commands.spawn().insert(
                    GenerateParty::new(4, names, pos),
                ).insert(Player);

            } else {
                info!("No valid spot found to spawn!");
            }
        }
    }
}

fn on_spawn(
    mut commands: Commands,
    mut q_spawn: Query<(Entity,&Transform, &Children), (Added<Party>, With<Player>)>,
    configs: Res<Assets<ConfigAsset>>,
) {
    if let Some(configs) = configs.get(SETTINGS_PATH) {
        for (party, party_transform, units) in q_spawn.iter() {
            //println!("Atlas {:?}", atlas.get(unit_atlas).unwrap());
            commands.entity(party)
            .insert(UnitCommands::new(configs.settings.map_move_speed, configs.settings.map_move_wait))
            .insert(PlayerUnit)
            .insert(MapUnit);
        }
    }
}
