use bevy::prelude::*;

use crate::{ldtk_loader::{LdtkMap, MapEntityDef, MapTileset, MapEntity}, make_sprite_atlas_sized, AnimationData, AtlasHandles, make_sprite_atlas, AnimationController};

pub const PREFAB_ASSET_PATH: &str = "units.ldtk";

pub struct PrefabsPlugin;
impl Plugin for PrefabsPlugin {
    fn build(&self, app: &mut App) {
        app.add_state(PrefabState::Loading)
        .add_system_set(
            SystemSet::on_enter(PrefabState::Loading)
            .with_system(setup)
        )
        .add_system_set(
            SystemSet::on_update(PrefabState::Loading)
            .with_system(load.label("prefab_load"))
        )
        .add_system_set(
            SystemSet::on_update(PrefabState::Loading)
            .with_system(build.after("prefab_load"))
        )
        ;
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
enum PrefabState {
    Loading,
    Loaded,
}

fn setup(
    mut commands: Commands,
) {
    println!("SPAWNING SPAWN SPAWN");
    commands.spawn().insert(LoadPrefab("units.ldtk".to_string()));
}


#[derive(Component, Default, Clone, Debug)]
pub struct LoadPrefab(String);

#[derive(Component)]
pub struct Prefab(Handle<LdtkMap>);

fn load(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    q_load: Query<(Entity,&LoadPrefab), Without<Prefab>>,
) {
    for (entity, load) in q_load.iter() {
        //println!("Spawning prefab");
        let handle: Handle<LdtkMap> = asset_server.load(&load.0);
        commands.spawn().insert(Prefab(handle))
        .insert(load.clone());
        commands.entity(entity).despawn();
    }
}

fn build(
    mut commands: Commands, 
    q_prefabs: Query<(Entity, &LoadPrefab), With<LoadPrefab>>,
    ldtk: Res<Assets<LdtkMap>>,
    mut atlas_handles: ResMut<AtlasHandles>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
) {
    for (entity, load) in q_prefabs.iter() {
        if let Some(ldtk) = ldtk.get(&load.0) {
            let layer = ldtk.layer_from_name("unit").unwrap_or_else(||
                panic!("Error building prefab, no 'Unit' layer found")
            );

            let entities = layer.as_entities();
       
            let unit = entities.get_tagged("root").next().unwrap();

            let unit_name = unit.field("name").unwrap().as_str().unwrap();
            let tileset_id = unit.tileset_id().unwrap_or_else(||
                panic!("Error building prefab, {} has no attached tileset", unit_name));


            let tileset = ldtk.tileset_from_id(tileset_id).unwrap();
            let atlas = get_atlas(&mut atlases, &mut atlas_handles, tileset);

            
            // let defs = ldtk.entity_defs();
            // //println!("DEFS {:?}", defs);
            // let unit = defs.def_from_name("Unit")
            //     .unwrap_or_else(||panic!("Error building prefab - couldn't find 'Unit' definition in {}", ldtk.name()));


            let mut entity = commands.entity(entity);

            let sprite = sprite_from_entity(unit, atlas, Vec2::ZERO, 0);

            entity.insert_bundle(sprite);
            
            // let defs = ldtk.entity_defs();

            // let anims = defs.get_tagged("animation");
            
            let layer = ldtk.layer_from_name("animation").unwrap_or_else(||
                panic!("Error building prefab, no 'animation' layer found")
            );
            
            let anims = layer.as_entities().get_tagged("animation");
            
            let anims: Vec<AnimationData> = anims.map(|a|anim_from_entity(a, &ldtk)).collect();
            

            if !anims.is_empty() {
                let mut controller = AnimationController::default();
                for anim in anims.iter() {
                    println!("Adding anim {}", anim.name);
                    controller.add(&anim.name, anim.clone());
                }
                entity.insert(controller);

            }


            // println!("REMOVING loadprefab??");
            entity.remove::<LoadPrefab>();
        }
    }
}

fn sprite_from_entity(
    def: &MapEntity,
    atlas: Handle<TextureAtlas>,
    pos: Vec2,
    layer: i32,
) -> SpriteSheetBundle {
    let size = def.size().as_vec2() / 64.0;
    let sprite = TextureAtlasSprite {
        index: def.tile_id().unwrap() as usize,
        custom_size: Some(size),
        ..Default::default()
    };

    SpriteSheetBundle {
        sprite,
        texture_atlas: atlas.clone(),
        transform: Transform::from_translation(pos.extend(layer as f32)),
        ..Default::default()
    }
}

// fn anim_from_def(
//     def: &MapEntityDef,
// ) -> AnimationData {


//     let name = def.get_str("name");
//     let frames = def.get_str("frames");
//     let speed = def.get_f32("speed");

//     let frames: Vec<usize> = ron::de::from_str(frames)
//         .expect("Error parsing animation frames: {} should be array of indices");
//     AnimationData {
//         name: name.to_lowercase(),
//         frames,
//         speed,
//     }
// }
fn anim_from_entity(
    def: &MapEntity,
    ldtk: &LdtkMap,
) -> AnimationData {
    
    let name = def.get_str("name");
    let frames = def.get_str("frames");
    let speed = def.get_f32("speed");
    let path = def.get_str("texture");

    let frames: Vec<usize> = ron::de::from_str(frames)
        .expect("Error parsing animation frames: {} should be array of indices");
    AnimationData {
        name: name.to_lowercase(),
        frames,
        speed,
        tileset_path: path.to_string(),
        ldtk_path: ldtk.name().to_string(),
    }
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