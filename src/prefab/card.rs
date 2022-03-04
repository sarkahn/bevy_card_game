use bevy::prelude::*;

use crate::{
    ldtk_loader::{LdtkMap, MapEntity, MapTileset},
    AnimationController, AnimationData, AtlasHandles, LoadCardPrefab,
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
        app.add_system(load).add_system(build_prefab);
    }
}

fn load(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    q_load: Query<(Entity, &LoadCardPrefab), Without<Prefab>>,
) {
    for (entity, load) in q_load.iter() {
        //println!("Loading {}", load.path);
        let handle: Handle<LdtkMap> = asset_server.load(&load.path);
        commands.spawn().insert(Prefab(handle)).insert(load.clone());
        commands.entity(entity).despawn();
    }
}

fn build_prefab(
    mut commands: Commands,
    q_prefabs: Query<(Entity, &LoadCardPrefab)>,
    ldtk: Res<Assets<LdtkMap>>,
    asset_server: Res<AssetServer>,
) {
    for (entity, load) in q_prefabs.iter() {
        if let Some(ldtk) = ldtk.get(&load.path) {
            let handle = asset_server.load(ldtk.name());
            commands
                .entity(entity)
                .remove::<LoadCardPrefab>()
                .insert(Prefab(handle.clone()));

            let entity_id = commands.entity(entity).id();

            let sprite = get_card_sprite(
                load.atlas.clone(),
                load.xy,
                load.depth,
                load.tile_id,
                load.size,
            );

            commands.entity(entity_id).insert_bundle(sprite);
        }
    }
}

fn get_card_sprite(
    atlas: Handle<TextureAtlas>,
    xy: IVec2,
    depth: i32,
    id: i32,
    size: IVec2,
) -> SpriteSheetBundle {
    let sprite = TextureAtlasSprite {
        index: id as usize,
        custom_size: Some(size.as_vec2()),
        ..Default::default()
    };

    SpriteSheetBundle {
        sprite,
        texture_atlas: atlas,
        transform: Transform::from_translation(xy.as_vec2().extend(depth as f32)),
        ..Default::default()
    }
}
