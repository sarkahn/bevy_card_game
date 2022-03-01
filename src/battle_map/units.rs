use std::{collections::VecDeque, time::Duration, cmp::Ordering};

use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_ascii_terminal::Point2d;
use bevy_easings::*;
use rand::{prelude::{ThreadRng, StdRng, IteratorRandom, Distribution}, thread_rng, distributions::WeightedIndex, RngCore, Rng};
use sark_pathfinding::AStar;

use crate::GameState;

use super::{components::MapPosition, BattleMapState, Map, MapUnits, map::CollisionMap};

pub struct UnitsPlugin;

impl Plugin for UnitsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system_set(
                SystemSet::on_update(GameState::BattleMap)
                    .with_system(update_sprite_position)
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

#[derive(Bundle, Default)]
pub struct MapUnitBundle {
    map_unit: MapUnit,
    pos: MapPosition,
    commands: UnitCommands,
}

impl MapUnitBundle {
    pub fn new(xy: IVec2) -> Self {
        Self {
            pos: MapPosition { xy },
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnitCommand {
    MoveToTile(IVec2),
    Wait(f32),
    AiThink(),
}

#[derive(Component)]
pub struct UnitCommands {
    pos: IVec2,
    move_timer: Timer,
    wait_timer: Timer,
    queue: VecDeque<UnitCommand>,
    current: Option<UnitCommand>,
}

impl Default for UnitCommands {
    fn default() -> Self {
        Self { 
            pos: Default::default(), 
            move_timer: Timer::from_seconds(0.6, false), 
            wait_timer: Timer::from_seconds(0.3, false), 
            queue: Default::default(), 
            current: Default::default() 
        }
    }
}

impl UnitCommands {
    pub fn new(move_time: f32, wait_time: f32, start_pos: IVec2) -> Self {
        let cmd = Self {
            move_timer: Timer::from_seconds(move_time, false),
            wait_timer: Timer::from_seconds(wait_time, false),
            pos: start_pos,
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
                Duration::from_secs_f32(wait)),
            _ => {}
        };
        self.queue.push_back(command);
        if self.queue.len() == 1 {
            self.current = Some(command);
        }
    }
    pub fn clear(&mut self) {
        self.queue.clear();
        self.current = None;
    }
}

fn process_commands(
    time: Res<Time>,
    mut q_set: QuerySet<(
        QueryState<(Entity, &mut UnitCommands, &mut MapPosition, &mut Transform)>,
        QueryState<&MapPosition, With<PlayerUnit>>,
    )>,
    map: Res<Map>,
    collision_map: Res<CollisionMap>,
    mut player_positions: Local<Vec<IVec2>>,
) {
    player_positions.clear();
    player_positions.extend(q_set.q1().iter().map(|p|p.xy));
    for (_, mut unit_commands, mut pos, mut transform) in q_set.q0().iter_mut() {
        if let Some(command) = unit_commands.current {
            match command {
                UnitCommand::MoveToTile(b) => {
                    unit_commands.move_timer.tick(time.delta());
                    let t = unit_commands.move_timer.percent();
    
                    let a = unit_commands.pos;
                    let b = b - map.size().as_ivec2() / 2;
                    let p = a.as_vec2().lerp(b.as_vec2(), t) + map.axis_offset();
                    let p = p.extend(transform.translation.z);
                    transform.translation = p;
                    if t >= 1.0 {
                        unit_commands.pos = b;
                        pos.xy = b;
                        unit_commands.move_timer.reset();
                        unit_commands.next();
                    }
                }
                UnitCommand::Wait(_) => {
                    unit_commands.wait_timer.tick(time.delta());
                    if unit_commands.wait_timer.finished() {
                        unit_commands.wait_timer.reset();
                        unit_commands.next();
                    }
                },
                UnitCommand::AiThink() => {
                    //println!("Thinking!");
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
                    let a = pos.xy;
                    let res = player_positions.iter().map(|b|
                        (a-*b).as_vec2().length());
                    let res = res.enumerate()
                        .min_by(|(_,a),(_,b)|
                        a.partial_cmp(&b).unwrap_or(Ordering::Less)
                    ).map(|(index,_)|index);
                    if let Some(min) = res {
                        let a = pos.xy + map.size().as_ivec2() / 2;

                        let b = player_positions.iter().nth(min).unwrap();
                        let b = *b + map.size().as_ivec2() / 2;
                        let mut astar = AStar::new(10);
                        if let Some(path) = astar.find_path(&collision_map.0, a.into(), b.into()) {
                            if let Some(next) = path.get(1) {
                                let xy = IVec2::from(*next);
                                //let xy = xy.as_vec2() - map.size().as_vec2() / 2.0;
                                let xy = (xy.as_vec2() + Vec2::new(0.5, 0.5).floor()).as_ivec2();
                                //println!("Slime {:?} moving to {}", entity, xy);
                                unit_commands.push(UnitCommand::MoveToTile(xy));
                                unit_commands.push(UnitCommand::AiThink());
                                unit_commands.next();
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Offset a given axis based on whether it's even or odd.
/// Allows for a nicely centered map even with odd numbered tiles.
fn axis_offset(size: IVec2) -> Vec2 {
    let cmp = (size % 2).cmpeq(IVec2::ZERO);
    Vec2::select(cmp, Vec2::new(0.5, 0.5), Vec2::ZERO)
}
fn update_sprite_position(
    map: Res<Map>,
    mut q_sprites: Query<(&mut Transform, &MapPosition), (Changed<MapPosition>, With<MapUnit>)>,
) {
    let offset = axis_offset(map.size().as_ivec2());
    for (mut t, p) in q_sprites.iter_mut() {
        //println!("Updating sprite pos");
        t.translation = p.xy.extend(5).as_vec3() + offset.extend(0.0);
    }
}

fn update_map_units(
    mut units: ResMut<MapUnits>,
    q_moved_units: Query<(Entity, &MapPosition), (With<MapUnit>, Changed<MapPosition>)>,
    q_units: Query<(Entity, &MapPosition), (With<MapUnit>, With<PlayerUnit>)>,
) {
    if q_moved_units.is_empty() {
        return;
    }
    units.0.iter_mut().for_each(|f| *f = None);
    for (entity, pos) in q_units.iter() {
        let xy = pos.xy() + units.size().as_ivec2() / 2;
        //println!("Map unit update pos: {}", xy);
        units.0[xy] = Some(entity);
    }
}
