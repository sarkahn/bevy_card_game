use bevy::prelude::*;

use crate::{GameState, ldtk_loader::LdtkMap};

pub struct AnimScenePlugin;

impl Plugin for AnimScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(GameState::AssetTest)
            .with_system(load)
        ).add_system_set(
            SystemSet::on_update(GameState::AssetTest)
            .with_system(on_loaded)
        );
    }
}

fn load(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let load: Handle<LdtkMap> = asset_server.load("test_tileset.ldtk");
    commands.insert_resource(load);
}

fn on_loaded(
    mut ev_loaded: EventReader<AssetEvent<LdtkMap>>,
) {
    for ev in ev_loaded.iter() {
        println!("Map loaded");
    }

}
