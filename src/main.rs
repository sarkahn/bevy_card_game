use animation::AnimationPlugin;
use arena::ArenaPlugin;
use assets::AssetsPlugin;
use bevy::{asset::LoadState, prelude::*, utils::HashMap, math::Vec3Swizzles};
use bevy_easings::EasingsPlugin;
use bevy_egui::EguiPlugin;
use camera::GameCameraPlugin;
use config::{ConfigAsset, ConfigPlugin};
use ldtk_loader::LdtkPlugin;
use prefabs::PrefabsPlugin;
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
mod party;
mod unit;
mod prefabs;

pub use grid::*;

pub const SETTINGS_PATH: &str = "game_settings.config";

pub use animation::{AnimationController, AnimationData};
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
    AssetTestLoad,
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
        .add_plugin(PrefabsPlugin)
        .add_state(GameState::Starting)
        .add_startup_system(load_configs)
        .add_system_set(
            SystemSet::on_update(GameState::Starting)
            .with_system(start)
        )
        .run();
}

fn load_configs(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let handle: Handle<ConfigAsset> = asset_server.load(SETTINGS_PATH);
    commands.insert_resource(handle);
}

fn start(
    mut state: ResMut<State<GameState>>,
    configs: Res<Assets<ConfigAsset>>,
) {
    if let Some(config) = configs.get(SETTINGS_PATH) {
        println!("Loading state {:?}", config.settings.begin_state);
        state.set(config.settings.begin_state).unwrap();
    }
}



#[derive(Default)]
pub struct AtlasHandles(HashMap<String, Handle<TextureAtlas>>);

pub trait GridHelper {
    fn to_grid(&self) -> IVec2;

    fn grid_to_xy(&self, grid: IVec2) -> Vec2 {
        grid.as_vec2() * TILE_SIZE as f32
    }

    fn xy_to_grid(&self, xy: Vec2) -> IVec2 {
        xy.as_ivec2() / TILE_SIZE
    }
}


pub const TILE_SIZE: i32 = 64;
impl GridHelper for Transform {
    fn to_grid(&self) -> IVec2 {
        self.xy_to_grid(self.translation.xy()) / TILE_SIZE
    }
}
