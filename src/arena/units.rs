use bevy::prelude::*;

use crate::GameState;

use super::ArenaState;

pub struct ArenaUnitsPlugin;

impl Plugin for ArenaUnitsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::LoadArena).with_system(spawn));
    }
}

#[derive(Component, Debug, Default)]
pub struct ArenaUnit;

pub fn spawn() {}
