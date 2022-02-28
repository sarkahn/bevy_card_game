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
    units::UnitsPlugin,
};

mod animated;
mod components;
mod input;
mod map;
mod states;
mod units;

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
            .add_plugin(AnimScenePlugin);
    }
}
