use animation::AnimationPlugin;
use arena::ArenaPlugin;
//use assets::AssetsPlugin;
use bevy::{asset::LoadState, math::Vec3Swizzles, prelude::*, utils::HashMap};
use bevy_easings::EasingsPlugin;
use bevy_egui::EguiPlugin;
use camera::GameCameraPlugin;
use config::{ConfigAsset, ConfigPlugin};
use debug::DebugPlugin;
use ldtk_loader::{LdtkPlugin, LdtkMap};
use party::PartyPlugin;
use prefab::{PrefabsPlugin, DoneLoadingPrefabs, LOAD_PREFAB_SYSTEM};
use serde::{Deserialize, Serialize};
use unit::UnitsPlugin;
use unit_test::UnitTestPlugin;

use self::battle_map::BattleMapPlugin;

mod animation;
mod arena;
//mod assets;
mod battle_map;
mod camera;
mod config;
mod grid;
mod ldtk_loader;
mod party;
mod prefab;
mod unit;
mod util;
mod debug;
mod unit_test;

pub use grid::*;

pub const SETTINGS_PATH: &str = "game_settings.config";
pub const GENERATE_PARTY_SYSTEM: &str = "generate_party";

pub use util::*;
pub use party::ArenaSpriteVisibility;

#[derive(Component)]
pub struct ResizeCamera(pub IVec2);

#[derive(Default)]
pub struct LdtkHandles(Vec<Handle<LdtkMap>>);

#[derive(Component)]
pub struct BuildPrefab {
    name: String,
}

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
        .insert_resource(ClearColor(Color::rgb_u8(82, 44, 38)))
        .init_resource::<AtlasHandles>()
        .init_resource::<LdtkHandles>()
        .add_plugins(DefaultPlugins)
        .add_plugin(ConfigPlugin)
        .add_plugin(GameCameraPlugin)
        .add_plugin(EguiPlugin)
        .add_plugin(LdtkPlugin)
        .add_plugin(UnitTestPlugin)
        .add_plugin(BattleMapPlugin)
        // .add_plugin(EasingsPlugin)
        .add_plugin(ArenaPlugin)
        .add_plugin(AnimationPlugin)
        .add_plugin(PrefabsPlugin)
        .add_plugin(PartyPlugin)
        .add_plugin(UnitsPlugin)
        // .add_plugin(DebugPlugin)
        .add_state(GameState::Starting)
        .add_startup_system(load_configs)
        .add_system_set(
            SystemSet::on_update(GameState::Starting)
            .with_system(start.after(LOAD_PREFAB_SYSTEM)))
        .run();
}

fn load_configs(mut commands: Commands, asset_server: Res<AssetServer>) {
    let handle: Handle<ConfigAsset> = asset_server.load(SETTINGS_PATH);
    commands.insert_resource(handle);
}

fn start(
    mut state: ResMut<State<GameState>>, 
    configs: Res<Assets<ConfigAsset>>,
    mut ev_prefab_load: EventReader<DoneLoadingPrefabs>,
) {
    if let Some(config) = configs.get(SETTINGS_PATH) {
        for _ in ev_prefab_load.iter() {
            state.set(config.settings.begin_state).unwrap();
        }
        //println!("Loading state {:?}", config.settings.begin_state);
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
