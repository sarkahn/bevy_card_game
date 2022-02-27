use bevy::prelude::*;
use bevy_tiled_camera::TiledProjection;
use serde::{Deserialize, Serialize};

use crate::{GameState, ldtk_loader::{LoadLdtkMap, LdtkMapBuilt}, grid::*, config::{GameSettings, ConfigAsset}, SETTINGS_PATH};

use self::{
    input::InputPlugin, map::MapPlugin, states::BattleMapSelectionPlugin,
    units::UnitsPlugin,
};

mod components;
mod input;
mod map;
mod render;
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
            //.add_plugin(RenderPlugin)
            .add_plugin(BattleMapSelectionPlugin)
            .add_plugin(InputPlugin)
            .add_plugin(MapPlugin);
    }
}
