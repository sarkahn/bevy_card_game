use bevy::prelude::Plugin;

mod abilities;
mod animations;
mod assets;
mod spawn;
mod unit;

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            //.add_plugin(spawn::PrefabSpawnPlugin)
            .add_plugin(assets::GameAssetsPlugin)
            .add_plugin(unit::UnitAssetPlugin);
    }
}
