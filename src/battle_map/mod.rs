use bevy::prelude::*;
use serde::{Serialize, Deserialize};

use crate::GameState;

use self::{map::MapPlugin, render::RenderPlugin, units::UnitsPlugin, states::BattleMapSelectionPlugin, input::InputPlugin};

mod components;
mod map;
mod render;
mod states;
mod units;
mod input;

pub use map::{Map, MapUnits};
pub use components::*;

pub struct BattleMapPlugin;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum BattleMapState {
    Inactive,
    EnemyTurn,
    SelectUnit,
    ChooseTarget,
    UnitMoving,
}

impl Plugin for BattleMapPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .add_state(BattleMapState::Inactive)
            .add_plugin(UnitsPlugin)
            .add_plugin(RenderPlugin)
            .add_plugin(BattleMapSelectionPlugin)
            .add_plugin(InputPlugin)
            .add_plugin(MapPlugin)
            .add_system_set(SystemSet::on_enter(GameState::BattleMap)
            .with_system(on_enter));
    }
}

fn on_enter(
    mut state: ResMut<State<BattleMapState>>,
) {
    state.set(BattleMapState::SelectUnit).unwrap();
}
