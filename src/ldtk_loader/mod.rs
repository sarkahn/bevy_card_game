// This module reuses a lot of code from bevy_ecs_ldtk:
// https://github.com/Trouv/bevy_ecs_ldtk

mod asset;

use bevy::prelude::*;

pub use asset::*;

use self::asset::LdtkAssetPlugin;
pub struct LdtkPlugin;

impl Plugin for LdtkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(LdtkAssetPlugin);
    }
}
