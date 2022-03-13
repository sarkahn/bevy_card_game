use bevy::{prelude::*, utils::HashMap, ecs::system::EntityCommands};

use crate::{ldtk_loader::{LdtkMap, MapTileset}, AtlasHandles, party::{PartyUnitSprite, PartyUnit}, TILE_SIZE, BuildPrefab};

pub struct UnitPrefabPlugin;

impl Plugin for UnitPrefabPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(build_prefab);
    }
}

fn build_prefab( 
    mut commands: Commands,
    ldtk: Res<Assets<LdtkMap>>,
    q_build: Query<(Entity, &BuildPrefab)>,
) {
    for (entity, build) in q_build.iter() {
        let ldtk = ldtk.get(&build.name).unwrap_or_else(||{
            panic!("Error loading prefab {}, unable to load ldtk file", build.name);
        });
        println!("Building prefab for {:?}", entity);
        //map_sprite.transform.translation = gen.pos;
        let map_sprite = get_tagged_sprite(ldtk, "map_sprite", 64.0);

        let map_sprite = commands.spawn()
            .insert_bundle(map_sprite)
            .insert(PartyUnitSprite)
            .id();

        let arena_sprite = get_tagged_sprite(ldtk, "arena_sprite", 128.0);
        let arena_sprite = commands.spawn()
            .insert_bundle(arena_sprite)
            .insert(PartyUnitSprite)
            .id();

        let unit = PartyUnit {
            map_sprite: map_sprite.clone(),
            arena_sprite: arena_sprite.clone()
        };

         commands.entity(entity)
        .insert(unit)
        .insert(Transform::default())
        .insert(GlobalTransform::default())
        .add_child(map_sprite)
        .add_child(arena_sprite)
        .remove::<BuildPrefab>();
    }
}


fn get_tagged_sprite(
    ldtk: &LdtkMap,
    tag: &str,
    size: f32,
) -> SpriteSheetBundle {
    let map_sprite = ldtk.get_tagged(tag).next().unwrap_or_else(||
        panic!("Error spawning unit {}, missing {} tag", ldtk.name(), tag)
    );
    let tileset = map_sprite.tileset_id().unwrap_or_else(||
        panic!("Error spawning unit {} {}, missing tileset id. Is a tilemap attached
        to the entity?", ldtk.name(), tag)
    );
    let tileset = ldtk.tileset_from_id(tileset).unwrap_or_else(||
        panic!("Error spawning unit {} {}, invalid tileset id. Is a tilemap attached
        to the entity?", ldtk.name(), tag)
    );
    let tile_id = map_sprite.tile_id().unwrap_or_else(||
        panic!("Error spawning unit {} {}, invalid tile id", ldtk.name(), tag)
    );
    get_sprite(tile_id as usize, tileset.atlas().clone(), Vec2::splat(size))
}

fn get_sprite(
    index: usize,
    atlas: Handle<TextureAtlas>,
    size: Vec2,
) -> SpriteSheetBundle {
    let sprite = TextureAtlasSprite {
        index,
        custom_size: Some(size),
        ..Default::default()
    };
    let xyz = Vec3::new(0.5, 0.5, 0.0) * TILE_SIZE as f32;
    SpriteSheetBundle {
        sprite,
        texture_atlas: atlas,
        transform: Transform::from_translation(xyz),
        visibility: Visibility { is_visible: false },
        ..Default::default()
    }
}