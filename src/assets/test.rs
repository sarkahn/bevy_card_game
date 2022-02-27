use bevy::prelude::*;

use crate::GameState;

use super::assets::Prefab;

pub struct AssetTestPlugin;

impl Plugin for AssetTestPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::BLUE))
        .init_resource::<LoadingPrefabs>()
        .add_system_set(
            SystemSet::on_enter(GameState::AssetTest).with_system(setup)
        );
    }
}

#[derive(Default)]
struct LoadingPrefabs(Vec<Handle<Prefab>>);

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut loading: ResMut<LoadingPrefabs>, 
) {
    //let handle = asset_server.load("units/guy.prefab");
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    loading.0.push(asset_server.load("units/guy.prefab"));

}