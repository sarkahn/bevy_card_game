use bevy::{ecs::system::EntityCommands, math::Vec3Swizzles, prelude::*, utils::HashMap};

use crate::{
    battle_map::{
        enemies::Spawner,
        units::{EnemyUnit, MapUnitBundle, PlayerUnit},
    },
    config::ConfigAsset,
    ldtk_loader::{LdtkMap, MapTileset},
    AnimationController, AtlasHandles, GameState, AnimationData, SETTINGS_PATH,
};

use super::{
    map::BattleMapLdtkHandle,
    units::{EnemyBase, PlayerBase, UnitCommand, UnitCommands},
};

pub const SPAWN_SYSTEM: &str = "BATTLEMAP_SPAWN_SYSTEM";

pub struct MapSpawnPlugin;

impl Plugin for MapSpawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(GameState::BattleMap).with_system(spawn_from_event),
        )
        .add_system(spawn_from_entity.label(SPAWN_SYSTEM))
        .add_event::<SpawnUnit>()
        .add_system_to_stage(CoreStage::PreUpdate, despawn_tick)
        .add_system_to_stage(CoreStage::PostUpdate, despawn_timer);
    }
}

pub enum Team {
    Player,
    Enemy,
}

#[derive(Default)]
pub struct SpawnUnit {
    pub atlas: Handle<TextureAtlas>,
    pub sprite_index: i32,
    pub position: IVec3, // X, Y, Depth
    //pub animations: Option<HashMap<String, AnimationData>>,
    pub enums: Option<Vec<String>>,
}

fn spawn_from_event(mut commands: Commands, mut ev_spawn: EventReader<SpawnUnit>) {
    for spawn in ev_spawn.iter() {
        //println!("Encountered spawn event");
        let (xy, depth) = (spawn.position.xy(), spawn.position.z);
        let transform = Transform::from_xyz(xy.x as f32, xy.y as f32, depth as f32);

        let sprite = TextureAtlasSprite {
            custom_size: Some(Vec2::ONE),
            index: spawn.sprite_index as usize,
            ..Default::default()
        };
        let sprite = SpriteSheetBundle {
            sprite,
            texture_atlas: spawn.atlas.clone(),
            transform,
            ..Default::default()
        };

        let mut new = commands.spawn_bundle(sprite);
        new.insert_bundle(MapUnitBundle::default());

        // if let Some(anims) = &spawn.animations {
        //     let mut controller = AnimationController::default();
        //     for (name, anim) in anims {
        //         println!("Adding {} animation", name);
        //         controller.add(&name, anim.clone());
        //     }
        //     controller.play("idle");
        //     new.insert(controller);
        // }

        if let Some(enums) = &spawn.enums {
            if enums.iter().any(|s| s == "player") {
                new.insert(PlayerUnit);
            }
            if enums.iter().any(|s| s == "enemy") {
                new.insert(EnemyUnit);
            }
            if enums.iter().any(|s| s == "spawner") {
                new.insert(Spawner {
                    timer: Timer::from_seconds(1.5, true),
                });
            }
            if enums.iter().any(|s| s == "playerbase") {
                new.insert(PlayerBase);
            }

            if enums.iter().any(|s| s == "enemybase") {
                new.insert(EnemyBase);
            }
        }
    }
}

#[derive(Component)]
pub struct SpawnEntity {
    pub ldtk: Handle<LdtkMap>,
    pub name: String,
    pub pos: Vec3,
}

#[derive(Component)]
pub struct DespawnTimer(pub Timer);

fn spawn_from_entity(
    mut commands: Commands,
    q_spawns: Query<(Entity, &SpawnEntity)>,
    ldtk_assets: Res<Assets<LdtkMap>>,
    mut atlas_handles: ResMut<AtlasHandles>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
    configs: Res<Assets<ConfigAsset>>,
) {
    for (entity, spawn) in q_spawns.iter() {
        if let (Some(ldtk),Some(configs)) = 
               (ldtk_assets.get(spawn.ldtk.clone()), configs.get(SETTINGS_PATH)) {
            let name = spawn.name.to_lowercase();
            let defs = ldtk.entity_defs();

            //println!("Spawn entity running");
            //println!("DEFS {:?}", defs);
            //println!("Slime? :{:?}", defs.def_from_name("slime"));
            if let Some(def) = defs.def_from_name(&name) {
                if let (Some(tileset_id), Some(tile_id)) = (def.tileset_id(), def.tile_id()) {
                    if let Some(tileset) = ldtk.tileset_from_id(tileset_id) {
                        let pos = spawn.pos;
                        let atlas = get_atlas(&mut atlases, &mut atlas_handles, &tileset);
                        let comps = build_unit(pos, atlas, tile_id);
                        //println!("Spawning {} at {}", spawn.name, pos);

                        let mut new = commands.entity(entity);
                        new.remove::<SpawnEntity>();
                        new.insert(comps.0)
                            .insert_bundle(comps.1)
                            .insert_bundle(comps.2);

                        // if let Some(animations) = def.animations() {
                        //     println!("Loading animations for {}", spawn.name);
                        //     let mut controller = AnimationController::default();
                        //     for (name, anim) in animations.iter() {
                        //         controller.add(&name, anim.clone());
                        //     }
                        //     controller.play("idle");
                        //     new.insert(controller);
                        // }
                        if tileset.tile_id_has_enum(tile_id, "enemy") {
                              //println!("Adding AI!");
                              new.insert(EnemyUnit);
                              let settings = &configs.settings;
                              let mut unit_commands = UnitCommands::new(
                                  settings.map_move_speed,
                                  settings.map_move_speed,
                              );
                              unit_commands.push(UnitCommand::AiThink());
                              new.insert(unit_commands);
                        }
                    }
                }
            } else {
                panic!("Attempting to spawn entity {}, but no definition was found in the ldtk file {}",
                name, ldtk.name(),
                );
            }
        }
    }
}

fn build_unit(
    position: Vec3,
    atlas: Handle<TextureAtlas>,
    sprite_index: i32,
) -> (Transform, SpriteSheetBundle, MapUnitBundle) {
    let (xy, depth) = (position.xy(), position.z);
    let transform = Transform::from_xyz(xy.x as f32, xy.y as f32, depth as f32);

    let sprite = TextureAtlasSprite {
        custom_size: Some(Vec2::ONE),
        index: sprite_index as usize,
        ..Default::default()
    };
    let sprite = SpriteSheetBundle {
        sprite,
        texture_atlas: atlas.clone(),
        transform,
        ..Default::default()
    };
    let unit = MapUnitBundle::default();
    (transform, sprite, unit)
}

fn get_atlas(
    atlases: &mut Assets<TextureAtlas>,
    atlas_handles: &mut AtlasHandles,
    tileset: &MapTileset,
) -> Handle<TextureAtlas> {
    let name = &tileset.name;
    match atlas_handles.0.get(name) {
        Some(atlas) => atlas.clone(),
        None => {
            let atlas = TextureAtlas::from_grid(
                tileset.image.clone(),
                IVec2::splat(tileset.tile_size).as_vec2(),
                tileset.tile_count.x as usize,
                tileset.tile_count.y as usize,
            );
            let handle = atlases.add(atlas);
            atlas_handles.0.insert(name.to_string(), handle.clone());
            handle
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
