use bevy::prelude::*;
use bevy_tiled_camera::TiledProjection;
use serde::{Deserialize, Serialize};

use crate::{
    config::{ConfigAsset, GameSettings},
    grid::*,
    ldtk_loader::LdtkMap,
    GameState, SETTINGS_PATH,
};

use self::{
    combat::MapCombatPlugin, enemies::EnemiesPlugin, input::InputPlugin, map::MapPlugin,
    selection::BattleMapSelectionPlugin, spawn::MapSpawnPlugin, units::UnitsPlugin,
};

mod combat;
mod components;
mod enemies;
mod input;
mod map;
mod selection;
mod spawn;
mod units;

pub use components::*;
pub use map::MapUnits;

pub struct BattleMapPlugin;

impl Plugin for BattleMapPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(UnitsPlugin)
            .add_plugin(InputPlugin)
            .add_plugin(MapPlugin)
            .add_plugin(MapSpawnPlugin)
            .add_plugin(EnemiesPlugin)
            .add_plugin(BattleMapSelectionPlugin)
            .add_plugin(MapCombatPlugin)
            .add_system_set(SystemSet::on_enter(GameState::LoadBattleMap).with_system(load_map));
    }
}

fn load_map(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    configs: Res<Assets<ConfigAsset>>,
) {
    let config = configs.get(SETTINGS_PATH).unwrap();
    let handle: Handle<LdtkMap> = asset_server.load(&config.settings.map_file);
    commands.insert_resource(handle);
}
