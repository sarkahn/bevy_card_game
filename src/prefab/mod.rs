use bevy::prelude::*;

use crate::{config::ConfigAsset, GameState, SETTINGS_PATH, ldtk_loader::{PrefabEntity, LdtkMap}};

mod card;
mod arena;
mod battle_map;
mod unit;

pub use unit::SpawnPrefab;
pub use unit::SpawnType;

pub const LOAD_PREFAB_SYSTEM: &str = "load_prefab";

pub struct PrefabPlugin;


impl Plugin for PrefabPlugin {
    fn build(&self, app: &mut App) {
        app.add_state(PrefabState::Loading)
            .add_plugin(arena::PrefabPlugin)
            .add_plugin(card::PrefabPlugin)
            .add_plugin(unit::UnitPrefabPlugin)
            .add_system_set(SystemSet::on_enter(GameState::AssetTest).with_system(test));
    }
}

fn test(mut commands: Commands, config: Res<Assets<ConfigAsset>>) {
    let config = config.get(SETTINGS_PATH).unwrap();
    //println!("SPAWNING SPAWN SPAWN");
    commands
        .spawn()
        .insert(SpawnPrefabOld::from_name(&config.settings.asset_test_file));
}


#[derive(Default, Component)]
pub struct Prefab(Handle<LdtkMap>);

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
enum PrefabState {
    Loading,
    Loaded,
}

#[derive(Component, Default, Clone, Debug)]
pub struct SpawnPrefabOld {
    pub path: String,
    pub xy: IVec2,
    pub depth: i32,
    pub change_sprite: Option<ChangeSprite>,
}

#[derive(Component, Default, Clone, Debug)]
pub struct LoadCardPrefab {
    pub path: String,
    pub xy: IVec2,
    pub depth: i32,
    pub atlas: Handle<TextureAtlas>,
    pub tile_id: i32,
    pub size: IVec2,
}

#[derive(Default, Clone, Debug)]
pub struct ChangeSprite {
    pub atlas: Handle<TextureAtlas>,
    pub tile_id: i32,
}

impl SpawnPrefabOld {
    pub fn from_name(name: &str) -> Self {
        Self {
            path: name.to_string(),
            ..Default::default()
        }
    }
}

fn load(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    q_load: Query<(Entity, &SpawnPrefabOld), Without<Prefab>>,
) {
    for (entity, load) in q_load.iter() {
        //println!("Loading {}", load.path, removing load unit prefab);
        let handle: Handle<LdtkMap> = asset_server.load(&load.path);
        commands.entity(entity).insert(Prefab(handle));
    }
}

fn sprite_from_entity(
    entity: &PrefabEntity,
    atlas: Handle<TextureAtlas>,
    xy: IVec2,
    depth: i32,
    load: &SpawnPrefabOld,
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