use bevy::prelude::*;
use bevy_tiled_camera::TiledProjection;
use serde::{Deserialize, Serialize};

use crate::{
    config::{ConfigAsset, GameSettings},
    grid::*,
    GameState, SETTINGS_PATH,
};

use self::{
    animated::AnimScenePlugin, input::InputPlugin, map::MapPlugin, states::BattleMapStatesPlugin,
    units::UnitsPlugin, spawn::MapSpawnPlugin, enemies::EnemiesPlugin, selection::BattleMapSelectionPlugin,
};

mod animated;
mod components;
mod input;
mod map;
mod states;
mod units;
mod enemies;
mod spawn;
mod selection;

pub use components::*;
pub use map::{Map, MapUnits};

pub struct BattleMapPlugin;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum BattleMapState {
    Inactive,
    BuildingMap,
    EnemyTurn,
    SelectUnit,
    ChooseTarget,
    UnitMoving,
}

impl Plugin for BattleMapPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_state(BattleMapState::Inactive)
            .add_plugin(UnitsPlugin)
            .add_plugin(BattleMapStatesPlugin)
            .add_plugin(InputPlugin)
            .add_plugin(MapPlugin)
            .add_plugin(AnimScenePlugin)
            .add_plugin(MapSpawnPlugin)
            .add_plugin(EnemiesPlugin)
            .add_plugin(BattleMapSelectionPlugin)
            ;
    }
}
