use bevy::{prelude::*, utils::HashMap};

use crate::{config::ConfigAsset, GameState, SETTINGS_PATH, ldtk_loader::{PrefabEntity, LdtkMap, Tags, Fields}};

use self::unit::UnitPrefabPlugin;

mod card;
mod arena;
mod battle_map;
mod unit;

pub const LOAD_PREFAB_SYSTEM: &str = "load_prefab";

#[derive(Default, Debug, Component, Clone)]
pub struct Prefab(Handle<LdtkMap>);

#[derive(Default, Debug)]
pub struct Prefabs {
    pub map: HashMap<String, Handle<LdtkMap>>,
    pub player_units: HashMap<String, Handle<LdtkMap>>,
    pub enemy_units: HashMap<String, Handle<LdtkMap>>,
    pub cards: HashMap<String, Handle<LdtkMap>>,
}

impl Prefabs {
    pub fn iter_units(&self) -> impl Iterator<Item=(&String,&Handle<LdtkMap>)> {
        self.player_units.iter().chain(self.enemy_units.iter())
    }
}

pub struct PrefabsPlugin;

impl Plugin for PrefabsPlugin {
    fn build(&self, app: &mut App) {
        app
        .init_resource::<Prefabs>()
        .add_event::<DoneLoadingPrefabs>()
        .add_system_set(
            SystemSet::on_update(GameState::Starting)
            .with_system(load_prefabs.label(LOAD_PREFAB_SYSTEM))
        )
        .add_plugin(UnitPrefabPlugin)
        ;
    }
}

#[derive(Default, Debug)]
pub struct DoneLoadingPrefabs;

fn load_prefabs(
    asset_server: Res<AssetServer>,
    config: Res<Assets<ConfigAsset>>,
    mut ev_loaded: EventWriter<DoneLoadingPrefabs>,
    mut prefabs: ResMut<Prefabs>,
    ldtk: Res<Assets<LdtkMap>>,
) {
    if let Some(config) = config.get(SETTINGS_PATH) {
        for unit in config.settings.player_units.iter() {
            if !prefabs.player_units.contains_key(unit) {
                let handle = asset_server.load(unit);
                prefabs.player_units.insert(unit.to_owned(), handle);
            }
        }

        for unit in config.settings.enemy_units.iter() {
            if !prefabs.enemy_units.contains_key(unit) {
                let handle = asset_server.load(unit);
                prefabs.enemy_units.insert(unit.to_owned(), handle);
            }
        }

        if prefabs.player_units.iter().any(|(_,handle)| ldtk.get(handle).is_none()) {
            return;
        }
        
        if prefabs.enemy_units.iter().any(|(_,handle)| ldtk.get(handle).is_none()) {
            return;
        }

        ev_loaded.send(DoneLoadingPrefabs);
    }

}

fn check_load_state(

) {

}

// fn build_prefabs(
//     asset_server: Res<AssetServer>,
//     mut prefabs: ResMut<Prefabs>,
// ) {
//     for name in PREFAB_NAMES {
//         let handle: Handle<LdtkMap> = asset_server.load(&name.to_string());
//         prefabs.0.push(handle);
//     }
// }

