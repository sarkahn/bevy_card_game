use bevy::prelude::Plugin;

mod abilities;
mod assets;
mod spawn;

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            //.add_plugin(spawn::PrefabSpawnPlugin)
            .add_plugin(assets::GameAssetsPlugin)
            ;
    }
}
