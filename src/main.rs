use bevy::{prelude::*};
use bevy_ascii_terminal::TerminalBundle;
use bevy_easings::EasingsPlugin;
use bevy_tiled_camera::TiledCameraBundle;
use config::{ConfigPlugin, ConfigAsset};
use serde::{Deserialize, Serialize};

use self::{
    battle_map::BattleMapPlugin,
};

mod assets;
mod battle_map;
mod config;


#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum GameState {
    Starting,
    StartScreen,
    LoadBattleMap,
    BattleMap,
    LoadArena,
    Arena,
}

impl Default for GameState {
    fn default() -> Self {
        GameState::Starting
    }
}

pub fn main() {
    App::new()
    .insert_resource(WindowDescriptor {
        title: "Bevy Card Game".to_string(),
        ..Default::default()
    })
    .add_plugins(DefaultPlugins)
    //.add_plugin(GameAssetsPlugin)
    .add_plugin(BattleMapPlugin)
    .add_plugin(EasingsPlugin)
    .add_plugin(ConfigPlugin)
    .add_state(GameState::StartScreen)
    .add_startup_system(start)
    .add_system(load_config)
    .run();
}

fn start(mut commands: Commands, asset_server: Res<AssetServer>) {
    let config: Handle<ConfigAsset> = asset_server.load("game_settings.config");
    commands.insert_resource(config);
}

fn load_config(
    mut commands: Commands,
    mut game_state: ResMut<State<GameState>>,
    configs: Res<Assets<ConfigAsset>>,
    mut ev_config: EventReader<AssetEvent<ConfigAsset>>,
) {
    for ev in ev_config.iter() {
        match ev {
            AssetEvent::Created { handle } => {
                let config = &configs.get(handle).unwrap();
                println!("Settings {:?}", config.settings);
                game_state.set(config.settings.begin_state).unwrap();
                commands.insert_resource(config.settings.clone());
            }
            _ => {}
        }
    }
}
