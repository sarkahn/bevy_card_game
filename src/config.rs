use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::GameState;

pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {}
}

#[derive(Default, Deserialize, Serialize)]
pub struct GameSettings {
    #[serde(default)]
    pub begin_state: GameState,
}
