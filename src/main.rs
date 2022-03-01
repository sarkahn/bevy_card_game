use animation::AnimationPlugin;
use arena::ArenaPlugin;
use assets::AssetsPlugin;
use bevy::{asset::LoadState, prelude::*, utils::HashMap};
use bevy_easings::EasingsPlugin;
use bevy_egui::EguiPlugin;
use camera::GameCameraPlugin;
use config::{ConfigAsset, ConfigPlugin};
use ldtk_loader::LdtkPlugin;
use serde::{Deserialize, Serialize};

use self::battle_map::BattleMapPlugin;

mod animation;
mod arena;
mod assets;
mod battle_map;
mod camera;
mod config;
mod grid;
mod ldtk_loader;
mod util;

pub use grid::*;

pub const SETTINGS_PATH: &str = "game_settings.config";

pub use animation::{AnimationController, UnitAnimation};
pub use util::*;

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
    BeginningCombat,
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
                //println!("Settings {:?}. Setting state", config.settings);
                game_state.set(config.settings.begin_state).unwrap();
                commands.insert_resource(config.settings.clone());
            }
            _ => {}
        }
    }
}

#[derive(Default)]
pub struct AtlasHandles(HashMap<String, Handle<TextureAtlas>>);
