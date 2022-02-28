use bevy::prelude::Plugin;

mod assets;
mod test;
mod abilities;
mod unit;

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(test::AssetTestPlugin)
            .add_plugin(assets::GameAssetsPlugin)
            .add_plugin(unit::UnitAssetPlugin)
            ;
    }
}