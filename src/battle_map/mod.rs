use bevy::prelude::Plugin;

use self::{map::MapPlugin, render::RenderPlugin, units::UnitsPlugin, selection::BattleMapSelectionPlugin, input::InputPlugin};

mod components;
mod map;
mod render;
mod selection;
mod units;
mod input;

pub use map::{Map, MapUnits};
pub use components::*;

pub struct BattleMapPlugin;

impl Plugin for BattleMapPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(UnitsPlugin)
            .add_plugin(RenderPlugin)
            .add_plugin(BattleMapSelectionPlugin)
            .add_plugin(InputPlugin)
            .add_plugin(MapPlugin);
    }
}
