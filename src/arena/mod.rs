mod units;
mod render;
mod config;

use bevy::prelude::*;
use serde::{Serialize, Deserialize};

use crate::GameState;

use self::render::RenderPlugin;

pub struct ArenaPlugin;

impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_state(ArenaState::Inactive)
        .add_system_set(
            SystemSet::on_enter(GameState::LoadArena)
            .with_system(on_enter)
        )
        .add_plugin(RenderPlugin)
        ;
    }
}

fn on_enter(
    mut game_state: ResMut<State<GameState>>,
    mut arena_state: ResMut<State<ArenaState>>,
) {
    //println!("Entering arena state");
    game_state.set(GameState::Arena).unwrap();
    arena_state.set(ArenaState::ChooseCard).unwrap();
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum ArenaState {
    Inactive,
    ChooseCard,
    SelectTarget,
    EnemyTurn,
}