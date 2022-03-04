use bevy::prelude::*;

use crate::{GameState, ldtk_loader::LdtkMap};

use super::ArenaState;

pub struct ArenaUnitsPlugin;

impl Plugin for ArenaUnitsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::LoadArena)
        .with_system(spawn));
    }
}

#[derive(Component, Debug, Default)]
pub struct ArenaUnit;

pub fn spawn() {}

pub struct LoadingPrefab(Handle<LdtkMap>);
fn load_prefab(
) {

}

