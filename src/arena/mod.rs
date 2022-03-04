pub mod cards;
mod combat;
mod load;
mod units;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::GameState;

use self::{cards::CardsPlugin, combat::CombatPlugin, load::ArenaLoadPlugin};
#[derive(Component)]
pub struct TakingATurn;

pub struct ArenaPlugin;

impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        // Game State is paused, to return to game do GameState.Pop()
        app.add_state(ArenaState::Inactive)
            //.add_system_set(SystemSet::on_enter(ArenaState::Loading).with_system(on_enter))
            .add_plugin(CombatPlugin)
            .add_plugin(ArenaLoadPlugin)
            .add_plugin(CardsPlugin);
    }
}

pub struct ArenaCombat {
    pub player_party: Entity,
    pub enemy_party: Entity,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum ArenaState {
    Inactive,
    Loading,
    ChooseCard,
    SelectTarget,
    EnemyTurn,
}
