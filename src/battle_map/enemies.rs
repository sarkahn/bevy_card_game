use bevy::{prelude::*, math::Vec3Swizzles};
use rand::{thread_rng, prelude::IteratorRandom, Rng};

use crate::{ldtk_loader::{LdtkMap, Tags, Fields}, GameState, SETTINGS_PATH, config::ConfigAsset, 
LDTK_ARCHER_PATH, prefab::Prefabs,  battle_map::{PlayerUnit, UnitCommands, MapUnitBundle, UnitCommand}, LDTK_SLIME_PATH};

use super::{map::{ CollisionMap}, spawn::Spawner, MapUnit, BattleMapEntity, MapUnits, get_valid_spawn_points, EnemyUnit};

pub struct BattleMapEnemiesPlugin;

impl Plugin for BattleMapEnemiesPlugin {
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
            .with_system(spawn)
        )
        ;
    }
}

#[derive(Component)]
pub struct BattleMapPlayerLoaded;

fn setup(
    mut commands: Commands,
    q_tags: Query<(Entity, &Tags, &Fields, &Name), (Added<Tags>, With<BattleMapEntity>)>,
) {
    for (entity, tags, fields, name) in q_tags.iter() {
        if tags.has_all(&["enemy", "spawner"]) {
            if let Some(min) = fields.try_get_f32("spawn_delay_min") {
                if let Some(max) = fields.try_get_f32("spawn_delay_max") {
                    let mut rng = thread_rng();
                    let delay: f32 = rng.gen_range(min..max);
                    println!("Spawning enemy spawner!");
                    commands.entity(entity).insert(
                        Spawner(Timer::from_seconds(delay, true))
                    ).insert(EnemyUnit);
                }
            }

        }
    }
}

fn spawn(
    mut commands: Commands,
    time: Res<Time>,
    mut q_spawner: Query<(&Transform, &mut Spawner), With<EnemyUnit>>,
    map_units: Res<MapUnits>,
    colliders: Res<CollisionMap>,
    prefabs: Res<Prefabs>,
    ldtk: Res<Assets<LdtkMap>>,
    configs: Res<Assets<ConfigAsset>>,
) {

    for (transform, mut spawner) in q_spawner.iter_mut() {
        spawner.tick(time.delta());

        if spawner.just_finished() {
            let unit_name = LDTK_SLIME_PATH;
            if let Some(ldtk) = ldtk.get(unit_name) {
                let map_pfb = ldtk.get_tagged("map_sprite").next().expect("Error loading prefab");

                let tileset = ldtk.tileset_from_id(map_pfb.tileset_id().expect("No tileset attached to prefab entity"))
                    .expect("Couldn't find tileset");
                let p = transform.translation;
                // let mut p = p + Vec3::new(0.0, -1.0, 0.0) * TILE_SIZE as f32;
                // p.z = 10.0;

                let curr = map_units.xy_to_grid(p.xy());

                let spawn_points = get_valid_spawn_points(curr, &map_units, &colliders);

                if let Some(spawn_points) = spawn_points {
                    let mut rng = thread_rng(); 
                    let spawn = spawn_points.choose(&mut rng).unwrap();
                    let spawn = map_units.grid_to_xy(spawn);
                    
                    let spawn = Vec3::new(spawn.x, spawn.y, 10.0);

                    //println!("Spawning at {}", spawn);

                    let sprite = TextureAtlasSprite {
                        index: map_pfb.tile_id().unwrap() as usize,
                        ..Default::default()
                    };
                    let sprite = SpriteSheetBundle {
                        sprite,
                        texture_atlas: tileset.atlas().clone(),
                        transform: Transform::from_translation(spawn),
                        ..Default::default()
                    };
                    
                    if let Some(configs) = configs.get(SETTINGS_PATH) {
                        let move_delay = configs.settings.map_move_speed;
                        let wait_delay = configs.settings.map_move_wait;
                        commands.spawn_bundle(sprite)
                        .insert(EnemyUnit)
                        .insert_bundle(
                            MapUnitBundle::with_commands(&[UnitCommand::AiThink()], move_delay, wait_delay ),
                        );

                    } else {
                        warn!("Attempted to spawn from player spawner, but config settings for map movement
                        were not found");
                    }
                } else {
                    info!("No valid spot found to spawn!");
                }

    
            }
        }
    }
}

