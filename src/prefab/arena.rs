use bevy::prelude::*;

use crate::{
    ldtk_loader::{LdtkMap, PrefabEntity, MapTileset},
    AnimationController, AnimationData, AtlasHandles, GameState, animation::{Animation, Animator}, unit::ArenaUnit,
};

use super::{SpawnPrefabOld, PrefabState, LOAD_PREFAB_SYSTEM, sprite_from_entity};


#[derive(Default, Component)]
pub struct PrefabRefs(Vec<Entity>);


pub struct PrefabPlugin;
impl Plugin for PrefabPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(build_unit.after(LOAD_PREFAB_SYSTEM));
    }
}

fn build_unit(
    mut commands: Commands,
    mut atlas_handles: ResMut<AtlasHandles>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
    mut q_animator: Query<&mut Animator>, 
    ldtk: Res<Assets<LdtkMap>>,
    q_prefabs: Query<(Entity, &SpawnPrefabOld), With<ArenaUnit>>,
) {
    for (entity, load) in q_prefabs.iter() {
        if let Some(ldtk) = ldtk.get(&load.path) {

            commands.entity(entity).remove::<SpawnPrefabOld>();

            let entity = commands
            .entity(entity)
            .id();
        
            let root = get_root(
                ldtk,
                &mut commands,
                entity,
                &mut atlases,
                &mut atlas_handles,
                load.xy,
                load.depth,
                load,
            );
        
            let mut refs = PrefabRefs::default();
            if let Some(root) = root {
                if let Ok(mut animator) = q_animator.get_mut(entity) {
                    println!("Found animator");
                    if let Some(anims) = get_animations(ldtk, root) {
                        for anim in anims {
                            animator.add_animation(anim);
                        }
                    }
                }
        
                if let Some(name) = root.fields().try_get_str("name") {
                    commands.entity(entity).insert(Name::new(name.to_string()));
                }
        
                if let Some(spells) =
                    make_spells(ldtk, &mut atlases, &mut atlas_handles, &mut commands, load)
                {
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
    load: &SpawnPrefabOld,
) -> Option<&'a PrefabEntity> {
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
            let tileset_id = unit
                .tileset_id()
                .unwrap_or_else(|| panic!("Error building prefab, has no attached tileset"));

            let tileset = ldtk.tileset_from_id(tileset_id).unwrap();
            let atlas = get_atlas(atlases, atlas_handles, tileset);
            let sprite = sprite_from_entity(unit, atlas, xy, depth, load);

            entity.insert_bundle(sprite);
            return Some(unit);
        }
    }
    None
}

fn get_animations(ldtk: &LdtkMap, unit: &PrefabEntity) -> Option<Vec<Animation>> {

    let anim_prefabs: Vec<_> = ldtk.get_tagged("animation").collect();
    if anim_prefabs.len() == 0 {
        return None;
    }
    let mut anims = Vec::new();
    for anim in anim_prefabs {
        let anim = anim_from_entity(anim, ldtk);
        let name = match unit.fields().try_get_str("name") {
            Some(n) => n,
            None => "nameless"
        };
        //println!("Building {} animation for {}", anim.name, name);
        anims.push(anim);
    }
    if !anims.is_empty() {
        return Some(anims);
    }
    None
}


fn make_animation_controller(
    entity: &PrefabEntity,
    animations: &[Animation],
) -> Option<Animator> {
    if !animations.is_empty() {
        let mut animator = Animator::default();
        // Initial_animation
        for anim in animations.iter() {
            //println!("Adding intial anim {} for {}", anim.name, entity.name());
            animator.add_animation(anim.clone());
        }
        return Some(animator);
    }
    None
}

fn anim_from_entity(def: &PrefabEntity, ldtk: &LdtkMap) -> Animation {
    let name = def.get_str("name");
    let frames = def.get_str("frames");
    let speed = def.get_f32("speed");
    let path = def.get_str("texture");

    let frames: Vec<usize> = ron::de::from_str(frames)
        .expect("Error parsing animation frames: {} should be array of indices");
    Animation { name: name.to_lowercase(), frames, speed }
    // AnimationData {
    //     name: name.to_lowercase(),
    //     frames,
    //     speed,
    //     tileset_path: path.to_string(),
    //     ldtk_name: ldtk.name().to_string(),
    // }
}

fn make_spells(
    ldtk: &LdtkMap,
    atlases: &mut Assets<TextureAtlas>,
    atlas_handles: &mut AtlasHandles,
    commands: &mut Commands,
    load: &SpawnPrefabOld,
) -> Option<Vec<Entity>> {
    let mut ids = Vec::new();
    for layer in ldtk.layers().filter(|l| l.is_entities()) {
        let entities = layer.as_entities().unwrap();
        for spell in entities.get_tagged("spell") {
            //println!("Found spell");
            let tex = spell.get_str("texture");
            let tileset = ldtk.tileset_from_path(tex).unwrap_or_else(||
                panic!("Error loading prefab {}, couldn't find tileset {}. Is it included in the ldtk file?",
                spell.name(), tex)
            );
            let atlas = get_atlas(atlases, atlas_handles, &tileset);
            let mut sprite = sprite_from_entity(spell, atlas.clone(), IVec2::ZERO, 0, load);
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
