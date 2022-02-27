use arena::ArenaPlugin;
use bevy::{prelude::*};
use bevy_ascii_terminal::TerminalBundle;
use bevy_easings::EasingsPlugin;
use bevy_tiled_camera::TiledCameraBundle;
use camera::GameCameraPlugin;
use config::{ConfigPlugin, ConfigAsset, ConfigAsset2, GameSettings};
use serde::{Deserialize, Serialize};

use self::{
    battle_map::BattleMapPlugin,
};

mod assets;
mod battle_map;
mod config;
mod arena;
mod camera;

#[derive(Component)]
pub struct ResizeCamera(pub IVec2);

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
    .add_plugin(GameCameraPlugin)
    .add_plugin(BattleMapPlugin)
    .add_plugin(EasingsPlugin)
    .add_plugin(ConfigPlugin)
    .add_plugin(ArenaPlugin)
    .add_state(GameState::StartScreen)
    .add_startup_system(start)
    .add_system(load_config)
    //.add_startup_system(test_start)
    .run();
}

// fn test_start(mut commands: Commands) {
//     //commands.spawn_bundle(OrthographicCameraBundle::new_2d());
//     println!("Spawning blue sprite");
//     commands.spawn_bundle(SpriteBundle {
//         sprite: Sprite {
//             color: Color::BLUE,
//             custom_size: Some(Vec2::new(10.0,10.0)),
//             ..Default::default()
//         },
//         ..Default::default()
//     });
// }

fn start(mut commands: Commands, asset_server: Res<AssetServer>) {
    //let config: Handle<ConfigAsset2> = asset_server.load("game_settings.config");
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
                println!("Settings {:?}. Setting state", config.settings);
                game_state.set(config.settings.begin_state).unwrap();
                commands.insert_resource(config.settings.clone());
            }
            _ => {}
        }
    }
}

// fn load_config(
//     mut game_state: ResMut<State<GameState>>,
//     asset_server: Res<AssetServer>,
//     //configs: Res<Assets<ConfigAsset2>>,
// ) {
//     if let Some(config) = asset_server.load("settings") {
//         println!("Found settings");
//         if let Ok(settings) = ron::de::from_str::<GameSettings>(config.prefab_string.as_str()) {
//             println!("Setting state");
//             game_state.set(settings.begin_state).unwrap();
//         }
//     }
// }
