use animation::AnimationPlugin;
use arena::ArenaPlugin;
use assets::AssetsPlugin;
use bevy::{asset::LoadState, prelude::*, utils::HashMap};
use bevy_easings::EasingsPlugin;
use bevy_egui::EguiPlugin;
use camera::GameCameraPlugin;
use config::{ConfigAsset, ConfigPlugin,};
use ldtk_loader::LdtkPlugin;
use serde::{Deserialize, Serialize};

use self::battle_map::BattleMapPlugin;

mod arena;
mod assets;
mod battle_map;
mod camera;
mod config;
mod ldtk_loader;
mod grid;
mod animation;

pub use grid::*;

pub const SETTINGS_PATH: &str = "game_settings.config";

pub use animation::{AnimationController, UnitAnimation};

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
    AssetTest,
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
        .init_resource::<AtlasHandles>()
        .add_plugins(DefaultPlugins)
        .add_plugin(GameCameraPlugin)
        .add_plugin(BattleMapPlugin)
        .add_plugin(EasingsPlugin)
        .add_plugin(ConfigPlugin)
        .add_plugin(ArenaPlugin)
        .add_plugin(LdtkPlugin)
        .add_plugin(AssetsPlugin)
        .add_plugin(EguiPlugin)
        .add_plugin(AnimationPlugin)
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

#[derive(Default)]
pub struct AtlasHandles(HashMap<String,Handle<TextureAtlas>>);

// fn load_config(
//     mut game_state: ResMut<State<GameState>>,
//     asset_server: Res<AssetServer>,
//     configs: Res<Assets<ConfigAsset2>>,
// ) {
//     let handle: Handle<ConfigAsset2> = asset_server.load("settings");
//     if asset_server.get_load_state(handle) == LoadState::Loaded {
//         println!("Settings is loaded")
//     }
//     // if let Some(config) = asset_server.load("settings") {
//     //     println!("Found settings");
//     //     if let Ok(settings) = ron::de::from_str::<GameSettings>(config.prefab_string.as_str()) {
//     //         println!("Setting state");
//     //         game_state.set(settings.begin_state).unwrap();
//     //     }
//     // }
// }
