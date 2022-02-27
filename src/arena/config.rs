use bevy::{prelude::*, reflect::TypeUuid};
use serde::{Deserialize, Serialize};

use super::ArenaState;

pub struct ArenaConfigPlugin;

impl Plugin for ArenaConfigPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<ArenaConfig>();
    }
}

#[derive(TypeUuid)]
#[uuid = "ac11ad22-5011-2aba-578f-12c512aaa0eb"]
pub struct ArenaConfig {}

#[derive(Serialize, Deserialize)]
pub struct ArenaLayout {
    allies: Vec<Vec2>,
    enemies: Vec<Vec2>,
}

fn load_assets(asset_server: Res<AssetServer>) {}

// fn on_enter(
//     mut state: ResMut<State<ArenaState::LoadArena
// ) {

// }
