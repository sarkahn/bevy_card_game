use bevy::prelude::*;

use crate::{GameState, ldtk_loader::{LdtkMap, MapEntity, MapTileset}, AtlasHandles, AnimationController, AnimationData, LoadPrefab};

#[derive(Default, Component)]
pub struct Prefab(Handle<LdtkMap>);


#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
enum PrefabState {
    Loading,
    Loaded,
}

#[derive(Default, Component)]
pub struct PrefabRefs(Vec<Entity>);



// Ref to root
#[derive(Component)]
pub struct PrefabRef(Entity);

pub struct PrefabPlugin;
impl Plugin for PrefabPlugin {
    fn build(&self, app: &mut App) {
        app.add_state(PrefabState::Loading)
        .add_system(load)
        .add_system(build_prefab)
        ;
    }
}

fn load(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    q_load: Query<(Entity,&LoadPrefab), Without<Prefab>>,
) {
    for (entity, load) in q_load.iter() {
        //println!("Loading {}", load.path);
        let handle: Handle<LdtkMap> = asset_server.load(&load.path);
        commands.spawn().insert(Prefab(handle))
        .insert(load.clone());
        commands.entity(entity).despawn();
    }
}

fn build_prefab(
    mut commands: Commands, 
    q_prefabs: Query<(Entity, &LoadPrefab), With<LoadPrefab>>,
    ldtk: Res<Assets<LdtkMap>>,
    mut atlas_handles: ResMut<AtlasHandles>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
    asset_server: Res<AssetServer>,
) {
    for (entity, load) in q_prefabs.iter() {
        if let Some(ldtk) = ldtk.get(&load.path) {

            if let Some(tile_count) = ldtk.tile_count() {
                println!("Setting tile count to {}", tile_count);
                //q_cam.single_mut().set_tile_count(tile_count.as_uvec2().into());
            } else {
               // q_cam.single_mut().set_tile_count([20,20]);
            }
            if let Some(pixels_per_tile) = ldtk.pixels_per_tile() {
                println!("Setting pixels per unit to {}", pixels_per_tile);
               // q_cam.single_mut().pixels_per_tile = pixels_per_tile as u32;
            } else {
                //q_cam.single_mut().pixels_per_tile = 128;
            }

            let handle = asset_server.load(ldtk.name());
            commands.entity(entity).remove::<LoadPrefab>().insert(Prefab(handle.clone()));
            
            let entity = commands.entity(entity).id();

            let root = get_root(ldtk, &mut commands, entity, &mut atlases, &mut atlas_handles,
            load.xy, load.depth
            );
            
            let mut refs = PrefabRefs::default();
            if let Some(root) = root {
                if let Some(anims) = get_animations(ldtk, root) {    
                    commands.entity(entity).insert(anims);
                }
                
                if let Some(spells) = make_spells(ldtk, &mut atlases, &mut atlas_handles, &mut commands) {
                    refs.0.extend(spells);
                }
            }

            if !refs.0.is_empty() {
                commands.entity(entity).insert(refs);
            }

        }
    }
    
}

fn get_root<'a>(
    ldtk: &'a LdtkMap,
    commands: &mut Commands,
    entity: Entity,
    atlases: &mut Assets<TextureAtlas>,
    atlas_handles: &mut AtlasHandles,
    xy: IVec2,
    depth: i32,
) -> Option<&'a MapEntity> {
    for layer in ldtk.layers() {
        let mut entity = commands.entity(entity);

        if let Some(entities) = layer.as_entities() {
            let unit = entities.get_tagged("root").next();
            if unit.is_none() {
                continue;
            }
            let unit = unit.unwrap();
            // println!("Pivot for {}: {}", unit.name(), unit.pivot());
       
            //let unit_name = unit.field("name").unwrap().as_str().unwrap();
            let tileset_id = unit.tileset_id().unwrap_or_else(||
                panic!("Error building prefab, has no attached tileset")
            );
    
            let tileset = ldtk.tileset_from_id(tileset_id).unwrap();
            let atlas = get_atlas(atlases, atlas_handles, tileset);
            let sprite = sprite_from_entity(unit, atlas, xy, depth);
    
            entity.insert_bundle(sprite);
            return Some(unit)
        }

    }
    None
}

fn get_animations(
    ldtk: &LdtkMap,
    unit: &MapEntity,
) -> Option<AnimationController> {
    let anims: Vec<_> = ldtk.layers().filter(|l|l.is_entities()).flat_map(
        |l|l.as_entities().unwrap().get_tagged("animation")
    ).collect();
    if anims.len() == 0 {
        return None;
    } 
    let mut all = Vec::new();
    for anim in anims {
        let anim = anim_from_entity(anim, ldtk);
        //println!("Building {} animation for {}", anim.name, unit.name());
        all.push(anim);
    }
    if !all.is_empty() {
        if let Some(mut controller) = make_animation_controller(unit, &all) {
            if let Some(initial) = unit.field("initial_animation") {
                let initial = initial.as_str().unwrap();
                //println!("Adding initial anim {} for {}", initial, unit.name());
                controller.play(initial);
            }
            return Some(controller);
        }
    }
    None
}

fn sprite_from_entity(
    entity: &MapEntity,
    atlas: Handle<TextureAtlas>,
    xy: IVec2,
    depth: i32,
) -> SpriteSheetBundle {
    let size = entity.size();
    let sprite = TextureAtlasSprite {
        index: entity.tile_id().unwrap() as usize,
        custom_size: Some(size.as_vec2()),
        ..Default::default()
    };

    //println!("Spawning prefab {} at {}", entity.name(), xy);
    SpriteSheetBundle {
        sprite,
        texture_atlas: atlas.clone(),
        transform: Transform::from_translation(xy.as_vec2().extend(depth as f32)),
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
            //println!("Adding intial anim {} for {}", anim.name, entity.name());
            controller.add(&anim.name, anim.clone());
        }
        if let Some(initial) = entity.field("initial_animation") {
            let initial = initial.as_str().unwrap();
            //println!("Running initial animation {}", initial);
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
) -> Option<Vec<Entity>> {
    let mut ids = Vec::new();
    for layer in ldtk.layers().filter(|l|l.is_entities()) {
        let entities = layer.as_entities().unwrap();
        for spell in entities.get_tagged("spell") {
            //println!("Found spell");
            let tex = spell.get_str("texture");
            let tileset = ldtk.tileset_from_path(tex).unwrap_or_else(||
                panic!("Error loading prefab {}, couldn't find tileset {}. Is it included in the ldtk file?",
                spell.name(), tex)
            );
            let atlas = get_atlas(atlases, atlas_handles, &tileset);
            let mut sprite = sprite_from_entity(spell, atlas.clone(), IVec2::ZERO, 0);
            sprite.visibility.is_visible = false;
            
            //println!("Spawning spell");

            let anim = anim_from_entity(spell, ldtk);
            let mut spell_entity = commands.spawn();
            spell_entity.insert_bundle(sprite);
            if let Some(anims) = make_animation_controller(spell, &[anim]) {
                ids.push(spell_entity.insert(anims).id());
            }
        }
    }
    
    match ids.is_empty() {
        true => Some(ids),
        false => None,
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