use std::{collections::VecDeque, time::Duration, cmp::Ordering};

use bevy::{ecs::system::EntityCommands, prelude::*, math::Vec3Swizzles};
use bevy_ascii_terminal::Point2d;
use bevy_easings::*;
use rand::{prelude::{ThreadRng, StdRng, IteratorRandom, Distribution}, thread_rng, distributions::WeightedIndex, RngCore, Rng};
use sark_pathfinding::AStar;

use crate::GameState;

use super::{
    //components::MapPosition, 
    BattleMapState, Map, MapUnits, map::CollisionMap};

pub struct UnitsPlugin;

impl Plugin for UnitsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system_set(
                SystemSet::on_update(GameState::BattleMap)
                    //.with_system(update_sprite_position)
                    .with_system(update_map_units),
            )
            .add_system_set(
                SystemSet::on_update(GameState::BattleMap).with_system(process_commands),
            );
    }
}

#[derive(Component)]
pub struct PlayerUnit;

#[derive(Component)]
pub struct PlayerBase;

#[derive(Component)]
pub struct EnemyUnit;


#[derive(Component)]
pub struct EnemyBase;


#[derive(Component, Default)]
pub struct MapUnit;

#[derive(Component)]
pub struct MapUnitSpeed(f32);
impl Default for MapUnitSpeed {
    fn default() -> Self {
        Self(0.6)
    }
}

#[derive(Bundle, Default)]
pub struct MapUnitBundle {
    map_unit: MapUnit,
    //pos: MapPosition,
    commands: UnitCommands,
    speed: MapUnitSpeed,
}


// impl MapUnitBundle {
//     pub fn new(xy: IVec2) -> Self {
//         Self {
//             //pos: MapPosition { xy },
//             ..Default::default()
//         }
//     }
// }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnitCommand {
    MoveToTile(IVec2, IVec2),
    Wait(f32),
    AiThink(),
}

#[derive(Component)]
pub struct UnitCommands {
    move_timer: Timer,
    wait_timer: Timer,
    queue: VecDeque<UnitCommand>,
    current: Option<UnitCommand>,
}

impl Default for UnitCommands {
    fn default() -> Self {
        Self { 
            move_timer: Timer::from_seconds(0.6, false), 
            wait_timer: Timer::from_seconds(0.3, false), 
            queue: Default::default(), 
            current: Default::default() 
        }
    }
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
        //println!("Next command. Size {}", self.queue.len());
        self.current = self.queue.pop_front();
        self.current.is_some()
    }

    pub fn push(&mut self, command: UnitCommand) {
        match command {
            UnitCommand::Wait(wait) => self.wait_timer.set_duration(
                Duration::from_secs_f32(wait)
            ),
            UnitCommand::MoveToTile(_,_) => {
                self.move_timer.reset();
            },
            _ => {}
        };
        self.queue.push_back(command);
        if self.queue.len() == 1 {
            self.current = Some(command);
        }
    }
    /// Does not clear current action - unit will
    /// finish what it's currently doing.
    pub fn clear(&mut self) {
        self.queue.clear();
        //self.current = None;
    }
}

fn process_commands(
    time: Res<Time>,
    mut q_set: QuerySet<(
        QueryState<(Entity, &mut UnitCommands, &mut Transform, &MapUnitSpeed)>,
        QueryState<&Transform, With<PlayerUnit>>,
    )>,
    map: Res<Map>,
    collision_map: Res<CollisionMap>,
    mut player_positions: Local<Vec<IVec2>>,
) {
    player_positions.clear();
    player_positions.extend(q_set.q1().iter().map(|t|map.to_index_2d(t.translation.xy())));
    for (_, mut unit_commands, mut transform, speed) in q_set.q0().iter_mut() {
        if let Some(command) = unit_commands.current {
            match command {
                UnitCommand::MoveToTile(a, b) => {
                    unit_commands.move_timer.tick(time.delta());
                    let t = unit_commands.move_timer.percent();
                    let (a,b) = (a.as_vec2(),b.as_vec2());
                    if t < 1.0 {
                        let p = a.lerp(b, t);
                        transform.translation = p.extend(transform.translation.z);
                    } else {
                        transform.translation = b.extend(transform.translation.z);
                        unit_commands.move_timer.reset();
                        unit_commands.next();
                    }
                    // let a = transform.translation.xy();
                    // let b = b.as_vec2();

                    // let dir = b - a;
                    // if dir.length() <= 0.1 {
                    //     println!("{} got to their desination, next command", a);
                    //     unit_commands.next();
                    // } else {
                    //     let dir = dir.normalize();
                    //     let vel = dir + speed.0 * time.delta_seconds();
                    //     println!("{} is moving to {}. Velocity {}", a, b, vel);
                    //     transform.translation += vel.extend(0.0);
                    // }
                }
                UnitCommand::Wait(_) => {
                    unit_commands.wait_timer.tick(time.delta());
                    if unit_commands.wait_timer.finished() {
                        unit_commands.wait_timer.reset();
                        unit_commands.next();
                    }
                },
                UnitCommand::AiThink() => {
                    println!("Thinking!");
                    let mut rng = thread_rng();
                    let choices = ["wait", "attack"];
                    let weights = [1_i32,5];
                    let dist = WeightedIndex::new(&weights).unwrap();
                    if choices[dist.sample(&mut rng)] == "wait" {
                        let wait:f32 = rng.gen_range(0.15..1.5);
                        //println!("Slime {:?} is gonna wait for {} seconds!", entity, wait);
                        unit_commands.push(UnitCommand::Wait(wait));
                        unit_commands.push(UnitCommand::AiThink());
                        unit_commands.next();
                        continue;
                        //continue;
                    }

                    let a = map.to_index_2d(transform.translation.xy());
                    println!("Finding nearest player: {}, {:?}", a, player_positions);
                    if let Some(b) = get_nearest_player_position(a,&player_positions) {
                        //let b = map.to_index_2d(b.as_vec2());
                        println!("Nearest player {}", b);
                        let mut astar = AStar::new(10);
                        if let Some(path) = astar.find_path(&collision_map.0, a.into(), b.into()) {
                            if let Some(next) = path.get(1) {
                                let index = IVec2::from(*next);
                                let a = map.xy_from_index_2d(a);
                                let b = map.xy_from_index_2d(index);
                                unit_commands.push(UnitCommand::MoveToTile(a.as_ivec2(), b.as_ivec2()));
                                unit_commands.push(UnitCommand::AiThink());
                                unit_commands.next();
                            }
                        } else {
                            println!("Couldn't find a player to path to!");
                        }
                    }

                }
            }
        }
    }
}

fn get_nearest_player_position(a: IVec2, positions: &Vec<IVec2>) -> Option<IVec2> {

    let res = positions.iter().map(|b|
        (a-*b).as_vec2().length());
    let res = res.enumerate()
        .min_by(|(_,a),(_,b)|
        a.partial_cmp(&b).unwrap_or(Ordering::Less)
    ).map(|(index,_)|index);
    if let Some(i) = res {
        return Some(positions[i]);
    }
    None
}

fn update_map_units(
    mut units: ResMut<MapUnits>,
    map: Res<Map>,
    q_moved_units: Query<(Entity, &Transform), (With<MapUnit>, Changed<Transform>)>,
    q_units: Query<(Entity, &Transform), (With<MapUnit>, With<PlayerUnit>)>,
) {
    if q_moved_units.is_empty() {
        return;
    }
    units.0.iter_mut().for_each(|f| *f = None);
    for (entity, pos) in q_units.iter() {
        //let xy = pos.xy() + units.size().as_ivec2() / 2;
        //println!("Map unit update pos: {}", xy);
        //units.0[xy] = Some(entity);
        //let xy = map.to_index_2d(pos.xy());
    }
}
