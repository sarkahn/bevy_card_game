use bevy::{prelude::*, utils::HashMap};

use crate::{config::ConfigAsset, GameState, SETTINGS_PATH, ldtk_loader::{PrefabEntity, LdtkMap, Tags, Fields}};

mod card;
mod arena;
mod battle_map;
mod unit;

pub const LOAD_PREFAB_SYSTEM: &str = "load_prefab";

pub const PREFAB_NAMES: &'static [&str] = &[
    "units_archer.ldtk",
    "units_slime.ldtk",
    "units_battleCard.ldtk",
];

#[derive(Default, Debug, Component, Clone)]
pub struct Prefab(Handle<LdtkMap>);

#[derive(Default, Debug)]
pub struct Prefabs(pub Vec<Handle<LdtkMap>>);

pub struct PrefabsPlugin;

impl Plugin for PrefabsPlugin {
    fn build(&self, app: &mut App) {
        app
        .init_resource::<Prefabs>()
        .add_startup_system_to_stage(
            StartupStage::PreStartup,
            build_prefabs
        )
        // .add_system_to_stage(
        //     CoreStage::PreUpdate,
        //     on_load
        // )
        ;
    }
}

fn build_prefabs(
    asset_server: Res<AssetServer>,
    mut prefabs: ResMut<Prefabs>,
) {
    for name in PREFAB_NAMES {
        let handle: Handle<LdtkMap> = asset_server.load(&name.to_string());
        prefabs.0.push(handle);
    }
}

