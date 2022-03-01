use bevy::{prelude::*, utils::HashMap, math::Vec3Swizzles, ecs::system::EntityCommands};

use crate::{GameState, UnitAnimation, battle_map::{units::{MapUnitBundle, PlayerUnit, EnemyUnit}, enemies::EnemySpawner}, AnimationController, ldtk_loader::{LdtkMap, MapTileset}, AtlasHandles, config::ConfigAsset, SETTINGS_PATH};

use super::{map::BattleMapLdtkHandle, units::{PlayerBase, EnemyBase, UnitCommands, UnitCommand}};

pub const BATTLE_MAP_SPAWN_SYSTEM: &str = "BATTLEMAP_SPAWN_SYSTEM";

pub struct MapSpawnPlugin;

impl Plugin for MapSpawnPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_system_set(SystemSet::on_update(GameState::BattleMap)
            .with_system(spawn_from_event)
        )
        .add_system_set(SystemSet::on_update(GameState::BattleMap)
            .with_system(spawn_from_entity)
            .label(BATTLE_MAP_SPAWN_SYSTEM)
        )
        .add_event::<SpawnUnit>()
        ;
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
    pub animations: Option<HashMap<String, UnitAnimation>>,
    pub enums: Option<Vec<String>>,
}

fn spawn_from_event(
    mut commands: Commands,
    mut ev_spawn: EventReader<SpawnUnit>,
) {
    for spawn in ev_spawn.iter() {
        //println!("SHOULD BE SPAWNING!");
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
        new.insert_bundle(MapUnitBundle::new(xy));
    
        if let Some(anims) = &spawn.animations {
            //println!("Loading animations for {}", entity.name);
            let mut controller = AnimationController::default();
            for (name, anim) in anims {
                controller.add(&name, anim.clone());
            }
            controller.play("idle");
            new.insert(controller);
        }

        if let Some(enums) = &spawn.enums {
            if enums.iter().any(|s|s=="player") {
                new.insert(PlayerUnit);
            }
            if enums.iter().any(|s|s=="enemy") {
                new.insert(EnemyUnit);
            }
            if enums.iter().any(|s|s=="spawner") {
                new.insert(EnemySpawner {
                    timer: Timer::from_seconds(0.5, false),
                });
            }
            if enums.iter().any(|s|s=="playerbase") {
                new.insert(PlayerBase);
            }

            if enums.iter().any(|s|s=="enemybase") {
                new.insert(EnemyBase);
            }

        }

        // if let Some(team) = &spawn.team {
        //     match team {
        //         Team::Player => new.insert(PlayerUnit),
        //         Team::Enemy => new.insert(EnemyUnit),
        //     };
        // }
    }
} 

#[derive(Component)]
pub struct SpawnEntity {
    pub ldtk: Handle<LdtkMap>,
    pub name: String,
    pub pos: IVec3,
}

fn spawn_from_entity(
    mut commands: Commands,
    q_spawns: Query<(Entity,&SpawnEntity)>,
    ldtk_assets: Res<Assets<LdtkMap>>,
    mut atlas_handles: ResMut<AtlasHandles>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
    configs: Res<Assets<ConfigAsset>>,
    asset_server: Res<AssetServer>,
) {
    if let Some(config) = configs.get(SETTINGS_PATH) {
        for (entity,spawn) in q_spawns.iter() {
            if let Some(ldtk) = ldtk_assets.get(spawn.ldtk.clone()) {
                commands.entity(entity).despawn();
                let defs = &ldtk.entity_defs;
                //println!("DEFS {:?}", defs);
                //println!("Slime? :{:?}", defs.def_from_name("slime"));
                if let Some(def) = defs.def_from_name(&spawn.name.to_lowercase()) {
                    if let (Some(tileset_id), Some(tile_id)) = (def.tileset_id,def.tile_id) {
                        if let Some(tileset) = ldtk.tilesets.get(&tileset_id) {
                            let pos = spawn.pos;
                            let atlas = get_atlas(&mut atlases, &mut atlas_handles, &tileset);
                            let comps = build_unit(pos, atlas, tile_id);
    
                            //println!("Spawning {} at {}", spawn.name, spawn.pos);
                            let mut new = commands.spawn();
                            new.insert(comps.0)
                            .insert_bundle(comps.1)
                            .insert_bundle(comps.2);
    
                            if !def.animations.is_empty() {
                                //println!("Loading animations for {}", entity.name);
                                let mut controller = AnimationController::default();
                                for (name, anim) in def.animations.iter() {
                                    controller.add(&name, anim.clone());
                                }
                                controller.play("idle");
                                new.insert(controller);
                            }
    
                            if let Some(enums) = tileset.enums.get(&tile_id) {
                                if enums.iter().any(|s|s=="enemy") {
                                    println!("Adding AI!");
                                    new.insert(EnemyUnit);
                                    let settings = &config.settings;
                                    let mut unit_commands = UnitCommands::new(
                                        settings.map_move_speed,
                                        settings.map_move_speed, pos.truncate());
                                    unit_commands.push(UnitCommand::AiThink());
                                    new.insert(unit_commands);
                                }
                            }
                            
                        }
                    }
                } else {
                    panic!("Attempting to spawn entity {}, but no definition was found in the ldtk file {}",
                    spawn.name, ldtk.name,
                    );
                }
            }
        }
    }
    
} 

fn build_unit(
    position: IVec3,
    atlas: Handle<TextureAtlas>,
    sprite_index: i32,
) -> (Transform,SpriteSheetBundle,MapUnitBundle) {
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
    let unit = MapUnitBundle::new(xy);
    (transform,sprite,unit)
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