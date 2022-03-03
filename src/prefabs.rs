use bevy::prelude::*;

use crate::{ldtk_loader::{LdtkMap, MapEntityDef, MapTileset, MapEntity, MapLayer}, make_sprite_atlas_sized, AnimationData, AtlasHandles, make_sprite_atlas, AnimationController};

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
    //println!("SPAWNING SPAWN SPAWN");
    //commands.spawn().insert(LoadPrefab("units.ldtk".to_string()));
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
            commands.entity(entity).remove::<LoadPrefab>();
            let entity = commands.entity(entity).id();

            let root = get_root(ldtk, &mut commands, entity, &mut atlases, &mut atlas_handles);
            
            if let Some(root) = root {
                if let Some(anims) = get_animations(ldtk, root) {    
                    commands.entity(entity).insert(anims);
                }
            }

            make_spells(ldtk, &mut atlases, &mut atlas_handles, &mut commands);
        }
    }
    
}

fn get_root<'a>(
    ldtk: &'a LdtkMap,
    commands: &mut Commands,
    entity: Entity,
    atlases: &mut Assets<TextureAtlas>,
    atlas_handles: &mut AtlasHandles,
) -> Option<&'a MapEntity> {
    for layer in ldtk.layers() {
        let mut entity = commands.entity(entity);
        let entities = layer.as_entities();

        let unit = entities.get_tagged("root").next();
        if unit.is_none() {
            continue;
        }
        let unit = unit.unwrap();
   
        let unit_name = unit.field("name").unwrap().as_str().unwrap();
        let tileset_id = unit.tileset_id().unwrap_or_else(||
            panic!("Error building prefab, {} has no attached tileset", unit_name)
        );

        let tileset = ldtk.tileset_from_id(tileset_id).unwrap();
        let atlas = get_atlas(atlases, atlas_handles, tileset);
        let sprite = sprite_from_entity(unit, atlas, 0);

        entity.insert_bundle(sprite);
        return Some(unit)
    }
    None
}

fn get_animations(
    ldtk: &LdtkMap,
    unit: &MapEntity,
) -> Option<AnimationController> {
    for layer in ldtk.layers() {
        let anims = layer.as_entities().get_tagged("animation");

        let mut all = Vec::new();
        for anim in anims {
            let anim = anim_from_entity(anim, ldtk);
            all.push(anim);
        }
        if !all.is_empty() {
            if let Some(mut controller) = make_animation_controller(unit, &all) {
                if let Some(initial) = unit.field("initial_animation") {
                    controller.play(initial.as_str().unwrap());
                }
                return Some(controller);
            }
        }
    }
    None
}

fn sprite_from_entity(
    def: &MapEntity,
    atlas: Handle<TextureAtlas>,
    layer: i32,
) -> SpriteSheetBundle {
    let size = def.size().as_vec2() / 64.0;
    let sprite = TextureAtlasSprite {
        index: def.tile_id().unwrap() as usize,
        custom_size: Some(size),
        ..Default::default()
    };

    println!("{} pos {} size: {}", def.name(), def.xy(), def.size());

    let pos = def.xy().as_vec2().extend(layer as f32) / 64.0;
    //let pos = Vec3::ZERO;

    SpriteSheetBundle {
        sprite,
        texture_atlas: atlas.clone(),
        transform: Transform::from_translation(pos),
        ..Default::default()
    }
}

fn make_animation_controller(
    entity: &MapEntity,
    animations: &[AnimationData],
) -> Option<AnimationController> {
    if !animations.is_empty() {
        let mut controller = AnimationController::default();
        // Initial_animation
        for anim in animations.iter() {
            println!("Adding anim {}", anim.name);
            controller.add(&anim.name, anim.clone());
        }
        if let Some(initial) = entity.field("initial_animation") {
            let initial = initial.as_str().unwrap();
            controller.play(initial);
        } else {
            controller.play_any();
        }
        return Some(controller);
    }
    None
}

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
        ldtk_name: ldtk.name().to_string(),
    }
}

fn make_spells(
    ldtk: &LdtkMap,
    atlases: &mut Assets<TextureAtlas>,
    atlas_handles: &mut AtlasHandles,
    commands: &mut Commands,
) {
    for layer in ldtk.layers() {
        let entities = layer.as_entities();
        for spell in entities.get_tagged("spell") {
            println!("Found spell");
            let tex = spell.get_str("texture");
            let tileset = ldtk.tileset_from_name(tex).unwrap_or_else(||
                panic!("Error loading prefab {}, couldn't find tileset {}. Is it included in the ldtk file?",
                spell.name(), tex)
            );
            let atlas = get_atlas(atlases, atlas_handles, &tileset);
            let sprite = sprite_from_entity(spell, atlas.clone(), 1);
            
            println!("Spawning spell");

            let anim = anim_from_entity(spell, ldtk);
            let mut spell_entity = commands.spawn();
            spell_entity.insert_bundle(sprite);
            if let Some(anims) = make_animation_controller(spell, &[anim]) {
                spell_entity.insert(anims);
            }
        }
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