use bevy::{prelude::*, utils::HashMap, ecs::system::EntityCommands};

use crate::{ldtk_loader::{LdtkMap, MapTileset}, LDTK_ARCHER_PATH, AtlasHandles};

pub struct UnitPrefabPlugin;

impl Plugin for UnitPrefabPlugin {
    fn build(&self, app: &mut App) {
        app
        //.init_resource::<Prefabs>()
        .add_startup_system(setup)
        .add_system(load_prefab)
        .add_system(build_from_file)
        .add_system(spawn)
        ;
    }
}


#[derive(Component)]
pub struct Unit;

#[derive(Component)]
pub struct MapSprite(pub Entity);

#[derive(Component)]
pub struct ArenaSprite(pub Entity);

#[derive(Component)]
pub struct BuildPrefab {
    pub name: String,
}



pub enum SpawnType {
    Map,
    Arena,
}

fn setup(
    mut commands: Commands,
) {
    commands.spawn().insert(BuildPrefab {
        name: LDTK_ARCHER_PATH.to_string(),
    });
}

pub fn load_prefab(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    q_build: Query<(Entity, &BuildPrefab), Added<BuildPrefab>>,
) {
    for (entity, build) in q_build.iter() {
        let handle: Handle<LdtkMap> = asset_server.load(&build.name);
        commands.entity(entity).insert(handle);
    }
}

pub fn build_from_file(
    mut commands: Commands,
    ldtk: Res<Assets<LdtkMap>>,
    q_build: Query<(Entity, &BuildPrefab, &Handle<LdtkMap>)>,
    //mut prefabs: ResMut<Prefabs>,
    mut atlas: ResMut<Assets<TextureAtlas>>,
    mut atlas_handles: ResMut<AtlasHandles>,
) {
    // for (entity, build, handle) in q_build.iter() {
    //     if let Some(ldtk) = ldtk.get(handle) {
    //         if let Some(pfb) = ldtk.get_tagged("map_sprite").next() {
    //             let tsid = pfb.tileset_id().expect("Could't get tileset id for map sprite");
    //             let tileset = ldtk.tileset_from_id(tsid).expect("Couldn't get tileset from ldtk");


    //             let sprite = TextureAtlasSprite {
    //                 index: pfb.tile_id().unwrap() as usize,
    //                 custom_size: Some(pfb.size().as_vec2()),
    //                 ..Default::default()
    //             };

    //             let name = pfb.name().to_string();

    //             info!("Loading {} prefab: {:?}",name, sprite );
    //             let atlas = get_atlas(&mut atlas, &mut atlas_handles, tileset);
    //             //let atlas = &tileset.atlas;
    //             prefabs.map.insert(
    //                 name,
    //                 UnitPrefab {
    //                 map_sprite: sprite,
    //                 atlas: atlas.clone(),
    //             });

    //             commands.entity(entity).remove::<BuildPrefab>();
    //             let tags = pfb.tags();
    //             let fields = pfb.fields();
    //         } else {
    //             warn!("Error loading prefab {}, couldn't find 'map_sprite' tag", build.name);
    //         }
    //     }
    // }
}

#[derive(Component)]
pub struct SpawnPrefab {
    name: String,
    pos: Vec2,
    depth: u32,
    spawn_type: SpawnType,
}
impl SpawnPrefab {
    pub fn new(name: &str, pos: Vec2, depth: u32, spawn_type: SpawnType) -> Self {
        Self {
            name: name.to_string(),
            pos,
            depth,
            spawn_type
        }
    }
}

fn spawn(
    mut commands: Commands,
    //prefabs: Res<Prefabs>,
    q_spawn: Query<(Entity, &SpawnPrefab)>,
    mut atlas: Res<Assets<TextureAtlas>>,
    mut atlas_handles: ResMut<AtlasHandles>,
) {
    // for (entity, spawn) in q_spawn.iter() {
    //     commands.entity(entity).remove::<SpawnPrefab>();
    //     let mut entity = commands.entity(entity);
    //     entity.remove::<SpawnPrefab>();

    //     let pfb = prefabs.get_unit_prefab(&spawn.name).unwrap_or_else(||
    //     panic!("Error spawning prefab - {} not found. Has it been loaded yet?", spawn.name));

    //     println!("Spawning {}", spawn.name, );

    //     let sprite = match spawn.spawn_type {
    //         SpawnType::Map => pfb.map_sprite.clone(),
    //         SpawnType::Arena => todo!(),
    //     };
    //     let texture_atlas = pfb.atlas.clone();

    //     //println!("Atlas: {:?}", atlas);

    //     let xyz = spawn.pos.extend(spawn.depth as f32);
    //     let bundle = SpriteSheetBundle {
    //         sprite,
    //         texture_atlas,
    //         transform: Transform::from_translation(xyz),
    //         ..Default::default()
    //     };
    //     entity.insert_bundle(bundle);

    // }
}

#[derive(Debug)]
pub struct UnitPrefab {
    map_sprite: TextureAtlasSprite,
    atlas: Handle<TextureAtlas>,
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