use bevy::prelude::*;

use crate::ldtk_loader::LdtkMap;

pub const PREFAB_ASSET_PATH: &str = "units.ldtk";

pub struct PrefabsPlugin;
impl Plugin for PrefabsPlugin {
    fn build(&self, app: &mut App) {
        app.add_state(PrefabState::Loading)
        .add_system_set(
            SystemSet::on_enter(PrefabState::Loading)
        );
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
enum PrefabState {
    Loading,
    Loaded,
}

fn load(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let handle: Handle<LdtkMap> = asset_server.load(PREFAB_ASSET_PATH);
    commands.insert_resource(handle);
}

fn setup(
    ldtk: Res<Assets<LdtkMap>>,
) {
    if let Some(ldtk) = ldtk.get(PREFAB_ASSET_PATH) {
        
    }
}