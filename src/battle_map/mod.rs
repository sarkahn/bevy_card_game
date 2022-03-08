use std::{collections::VecDeque, time::Duration};

use bevy::prelude::*;
use bevy_tiled_camera::TiledProjection;
use serde::{Deserialize, Serialize};

use crate::{
    config::{ConfigAsset, GameSettings},
    grid::*,
    ldtk_loader::LdtkMap,
    GameState, SETTINGS_PATH,
};

use self::{
    combat::MapCombatPlugin, enemies::BattleMapEnemyPlugin, input::InputPlugin, map::{MapPlugin, CollisionMap},
    selection::BattleMapSelectionPlugin, 
    //spawn::MapSpawnPlugin, 
    units::UnitsPlugin, player::BattleMapPlayerPlugin, spawn::MapSpawnPlugin,
};

mod combat;
mod components;
mod enemies;
mod input;
mod map;
mod selection;
mod spawn;
mod units;
mod player;
mod setup;

pub use components::*;
pub use map::MapUnits;

pub struct BattleMapPlugin;

impl Plugin for BattleMapPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .add_plugin(MapPlugin)
            .add_plugin(InputPlugin)
            .add_plugin(MapSpawnPlugin)
            .add_plugin(BattleMapSelectionPlugin)
            .add_plugin(BattleMapEnemyPlugin)
            .add_plugin(UnitsPlugin)
            .add_plugin(BattleMapPlayerPlugin)
            .add_plugin(MapCombatPlugin)
            .add_system_set(SystemSet::on_enter(GameState::LoadBattleMap).with_system(load_map));
    }
}

fn load_map(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    configs: Res<Assets<ConfigAsset>>,
) {
    let config = configs.get(SETTINGS_PATH).unwrap();
    let handle: Handle<LdtkMap> = asset_server.load(&config.settings.map_file);
    commands.insert_resource(handle);
}

#[derive(Component)]
struct PlayerUnit;

#[derive(Component)]
struct EnemyUnit;

#[derive(Component)]
struct EnemyBase;

#[derive(Component, Default)]
struct MapUnit;

#[derive(Component, Default)]
struct PlayerBase;


#[derive(Default, Component)]
struct BattleMapEntity;

#[derive(Component)]
struct MapLoaded;

#[derive(Debug, Clone, Copy, PartialEq)]
enum UnitCommand {
    MoveToTile(IVec2, IVec2),
    Wait(f32),
    AiThink(),
}

#[derive(Component)]
struct UnitCommands {
    move_timer: Timer,
    wait_timer: Timer,
    queue: VecDeque<UnitCommand>,
    current: Option<UnitCommand>,
}


impl UnitCommands {
    pub fn new(move_time: f32, wait_time: f32) -> Self {
        let cmd = Self {
            move_timer: Timer::from_seconds(move_time, false),
            wait_timer: Timer::from_seconds(wait_time, false),
            queue: VecDeque::new(),
            current: None,
        };
        cmd
    }
    fn next(&mut self) -> bool {
        self.current = self.queue.pop_front();
        if let Some(current) = self.current {
            //println!("Setting current command to {:?}", current);
        }
        self.current.is_some()
    }

    pub fn push(&mut self, command: UnitCommand) {
        match command {
            UnitCommand::Wait(wait) => self.wait_timer.set_duration(Duration::from_secs_f32(wait)),
            UnitCommand::MoveToTile(_, _) => {
                self.move_timer.reset();
            }
            _ => {}
        };
        self.queue.push_back(command);
    }
    /// Does not clear current action - unit will
    /// finish what it's currently doing.
    pub fn clear(&mut self) {
        self.queue.clear();
        //self.current = None;
    }
}

#[derive(Bundle, Default)]
struct MapUnitBundle {
    map_unit: MapUnit,
    //pos: MapPosition,
    commands: UnitCommands,
}

impl MapUnitBundle {
    pub fn with_commands(commands: &[UnitCommand], move_time: f32, wait_time: f32) -> Self {
        let mut me = Self {
            commands: UnitCommands {
                move_timer: Timer::from_seconds(move_time, false),
                wait_timer: Timer::from_seconds(wait_time, false),
                ..Default::default()
            },
            ..Default::default()
        };
        for c in commands {
            me.commands.push(*c);
        }

        me
    }
}



impl Default for UnitCommands {
    fn default() -> Self {
        Self {
            move_timer: Timer::from_seconds(0.6, false),
            wait_timer: Timer::from_seconds(0.3, false),
            queue: Default::default(),
            current: Default::default(),
        }
    }
}


const ADJACENT: &[[i32;2]] = &[
    [-1,1],
    [0,1],
    [1,1],
    [-1,0],
    [1,0],
    [-1,-1],
    [0,-1],
    [1,-1],
];

fn get_valid_spawn_points<'a>(
    xy: IVec2,
    units: &'a MapUnits,
    colliders: &'a CollisionMap,
) -> Option<impl Iterator<Item=IVec2> + 'a> {
    let valid = ADJACENT.iter().filter(move |adj| {
        let adj = xy + IVec2::from(**adj);
        return units.get_from_grid_xy(adj).is_none() && !colliders.0.is_obstacle(adj.to_array());
    }).map(move |p| xy + IVec2::from(*p));
    if valid.clone().count() > 0 {
        return Some(valid);
    }
    None
}