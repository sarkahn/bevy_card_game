use bevy::prelude::*;

use crate::{
    ldtk_loader::{LdtkMap, MapEntity, MapTileset},
    AnimationController, AnimationData, AtlasHandles, GameState, LoadUnitPrefab, animation::{Animation, Animator},
};

use super::PrefabState;

#[derive(Default, Component)]
pub struct Prefab(Handle<LdtkMap>);

#[derive(Default, Component)]
pub struct PrefabRefs(Vec<Entity>);

// Ref to root
#[derive(Component)]
pub struct PrefabRef(Entity);

pub struct PrefabPlugin;
impl Plugin for PrefabPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(load.label("prefab_unit_load"))
            .add_system(build_prefab.after("prefab_unit_load"));
    }
}

fn load(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    q_load: Query<(Entity, &LoadUnitPrefab), Without<Prefab>>,
) {
    for (entity, load) in q_load.iter() {
        //println!("Loading {}", load.path, removing load unit prefab);
        let handle: Handle<LdtkMap> = asset_server.load(&load.path);
        commands.entity(entity).insert(Prefab(handle));
    }
}

fn build_prefab(
    mut commands: Commands,
    q_prefabs: Query<(Entity, &LoadUnitPrefab), With<LoadUnitPrefab>>,
    ldtk: Res<Assets<LdtkMap>>,
    mut atlas_handles: ResMut<AtlasHandles>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
    asset_server: Res<AssetServer>,
    mut q_animator: Query<&mut Animator>, 
) {
    for (entity, load) in q_prefabs.iter() {
        if let Some(ldtk) = ldtk.get(&load.path) {
            let handle = asset_server.load(ldtk.name());
            commands
                .entity(entity)
                .remove::<LoadUnitPrefab>()
                .insert(Prefab(handle.clone()));

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
    load: &LoadUnitPrefab,
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

fn get_animations(ldtk: &LdtkMap, unit: &MapEntity) -> Option<Vec<Animation>> {

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

fn sprite_from_entity(
    entity: &MapEntity,
    atlas: Handle<TextureAtlas>,
    xy: IVec2,
    depth: i32,
    load: &LoadUnitPrefab,
) -> SpriteSheetBundle {
    let size = entity.size();

    let (id, atlas) = match &load.change_sprite {
        Some(change) => (change.tile_id as usize, change.atlas.clone()),
        None => (entity.tile_id().unwrap() as usize, atlas.clone()),
    };

    let sprite = TextureAtlasSprite {
        index: id,
        custom_size: Some(size.as_vec2()),
        ..Default::default()
    };

    //println!("Spawning prefab {} at {}", entity.name(), xy);
    SpriteSheetBundle {
        sprite,
        texture_atlas: atlas,
        transform: Transform::from_translation(xy.as_vec2().extend(depth as f32)),
        ..Default::default()
    }
}

fn make_animation_controller(
    entity: &MapEntity,
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

fn anim_from_entity(def: &MapEntity, ldtk: &LdtkMap) -> Animation {
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
    load: &LoadUnitPrefab,
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
