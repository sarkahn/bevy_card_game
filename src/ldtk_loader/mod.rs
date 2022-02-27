// This module reuses a lot of code from bevy_ecs_ldtk:
// https://github.com/Trouv/bevy_ecs_ldtk

mod asset;
mod map_builder;

use std::borrow::Cow;

use bevy::prelude::*;

use self::{asset::{LdtkAsset, LdtkAssetPlugin}, map_builder::LdtkMapBuilderPlugin};

pub use map_builder::LdtkMapBuilt;

pub struct LdtkPlugin;

impl Plugin for LdtkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LoadingMaps>()
            .add_event::<LoadLdtkMap>()
            .add_plugin(LdtkAssetPlugin)
            .add_plugin(LdtkMapBuilderPlugin)
            .add_system(load);
    }
}

#[derive(Default)]
pub(crate) struct LoadingMaps(pub(crate) Vec<Handle<LdtkAsset>>);

#[derive(Default)]
pub struct LoadLdtkMap(Cow<'static, str>);

impl LoadLdtkMap {
    pub fn from_path(string: impl Into<Cow<'static, str>>) -> Self {
        let path = string.into();
        LoadLdtkMap(path)
    }
}

fn load(
    asset_server: Res<AssetServer>,
    mut ev_reader: EventReader<LoadLdtkMap>,
    mut loading: ResMut<LoadingMaps>,
) {
    for ev in ev_reader.iter() {
        loading.0.push(asset_server.load(&*ev.0));
    }
}