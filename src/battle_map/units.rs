use std::{cmp::Ordering, collections::VecDeque, time::Duration};

use bevy::{math::Vec3Swizzles, prelude::*};

use rand::{distributions::WeightedIndex, prelude::Distribution, thread_rng, Rng};
use sark_pathfinding::AStar;

use crate::{GameState, TILE_SIZE};

use super::map::CollisionMap;

pub struct UnitsPlugin;

impl Plugin for UnitsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_update(GameState::BattleMap))
            .add_system_set(
                SystemSet::on_update(GameState::BattleMap).with_system(process_commands),
            );
    }
}

#[derive(Component)]
pub struct PlayerUnit;

#[derive(Component)]
pub struct EnemyUnit;

#[derive(Component)]
pub struct EnemyBase;

#[derive(Component, Default)]
pub struct MapUnit;

#[derive(Component, Default)]
pub struct PlayerBase;

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

impl MapUnitBundle {
    pub fn with_commands(commands: &[UnitCommand]) -> Self {
        let mut me = Self {
            ..Default::default()
        };
        for c in commands {
            me.commands.push(*c);
        }

        me
    }
}

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
            current: Default::default(),
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

fn process_commands(
    time: Res<Time>,
    mut q_set: QuerySet<(
        QueryState<(Entity, &mut UnitCommands, &mut Transform, &MapUnitSpeed)>,
        QueryState<&Transform, With<PlayerUnit>>,
        QueryState<&Transform, With<PlayerBase>>,
    )>,
    //map: Res<Map>,
    map: Res<CollisionMap>,
    mut player_positions: Local<Vec<IVec2>>,
) {
    player_positions.clear();
    player_positions.extend(
        q_set
            .q1()
            .iter()
            .map(|t| t.translation.xy().as_ivec2() - map.half_offset()),
    );
    //println!("{}", q_set.q2().iter().count());
    let base_pos = q_set.q2().get_single();
    let base_pos = base_pos.map(|p| p.translation.xy());
    if base_pos.is_err() {
        //println!("Couldn't find base!");
    }
    for (entity, mut unit_commands, mut transform, speed) in q_set.q0().iter_mut() {
        //println!("{:?} Command count {}", entity, unit_commands.queue.len());
        if unit_commands.current.is_none() {
            unit_commands.next();
        }

        //println!("Reading commands");
        if let Some(command) = unit_commands.current {
            match command {
                UnitCommand::MoveToTile(a, b) => {
                    unit_commands.move_timer.tick(time.delta());
                    let t = unit_commands.move_timer.percent();
                    let a = a * TILE_SIZE;
                    let b = b * TILE_SIZE;
                    let (a, b) = (a.as_vec2(), b.as_vec2());
                    if t < 1.0 {
                        let p = a.lerp(b, t);
                        transform.translation = p.extend(transform.translation.z);
                    } else {
                        transform.translation = b.extend(transform.translation.z);
                        //println!("Done moving from {} to {} Final Pos {}",
                        //a, b, transform.translation.xy());
                        unit_commands.move_timer.reset();
                        unit_commands.next();
                    }
                }
                UnitCommand::Wait(_) => {
                    //println!("{:?} WAITING", entity);
                    unit_commands.wait_timer.tick(time.delta());
                    if unit_commands.wait_timer.finished() {
                        unit_commands.wait_timer.reset();
                        unit_commands.next();
                    }
                }
                UnitCommand::AiThink() => {
                    //println!("{:?} Thinking!", entity);
                    let mut rng = thread_rng();
                    let choices = ["wait", "attack"];
                    let weights = [1_i32, 5];
                    let dist = WeightedIndex::new(&weights).unwrap();
                    if choices[dist.sample(&mut rng)] == "wait" {
                        let wait: f32 = rng.gen_range(0.15..1.5);
                        //println!("Slime {:?} is gonna wait for {} seconds!", entity, wait);
                        unit_commands.push(UnitCommand::Wait(wait));
                        unit_commands.push(UnitCommand::AiThink());
                        unit_commands.next();
                        continue;
                    }

                    let a = transform.translation.xy().as_ivec2() / TILE_SIZE;
                    //let a -= collisiotin.axis_offset();
                    if let Ok(base_pos) = base_pos {
                        //println!("Base pos {:?}", base_pos);
                        //println!("{:?} at {}, Finding nearest player {:?}", entity, a, player_positions);
                        //let b = map.to_index_2d(b.as_vec2());
                        let b = base_pos.as_ivec2() / TILE_SIZE;
                        //println!("A to base {}, {}", a, b);
                        let mut astar = AStar::new(10);
                        if let Some(path) = astar.find_path(&map.0, a.into(), b.into()) {
                            //println!("Path {:?}", path);
                            if let Some(next) = path.get(1) {
                                let b = IVec2::from(*next);
                                //println!("Pathin {} to {} (no divide)", a, b);

                                // let a = a + map.half_offset();
                                // let b = b + map.half_offset();

                                //println!("Should see 'done moving' next");
                                unit_commands.push(UnitCommand::MoveToTile(a, b));
                                unit_commands.push(UnitCommand::AiThink());
                                //println!("{:?} pushed {} to {} to commands for move. Stack state: {:?}. Calling next",entity, a, b, unit_commands.queue);
                                unit_commands.next();
                            }
                        } else {
                            // Couldn't find a player to path to. Go for the kill!
                            //println!("Couldn't find a player!");
                        }
                    }
                }
            }
        }
    }
}

fn get_nearest_player_position(a: IVec2, positions: &Vec<IVec2>) -> Option<IVec2> {
    let res = positions.iter().map(|b| (a - *b).as_vec2().length());
    let res = res
        .enumerate()
        .min_by(|(_, a), (_, b)| a.partial_cmp(&b).unwrap_or(Ordering::Less))
        .map(|(index, _)| index);
    if let Some(i) = res {
        return Some(positions[i]);
    }
    None
}
