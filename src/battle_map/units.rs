use std::{cmp::Ordering, collections::VecDeque, time::Duration};

use bevy::{math::Vec3Swizzles, prelude::*};

use rand::{distributions::WeightedIndex, prelude::Distribution, thread_rng, Rng};
use sark_pathfinding::AStar;

use crate::{GameState, TILE_SIZE};

use super::{map::CollisionMap, PlayerBase, PlayerUnit, MapUnit, UnitCommands, UnitCommand};

pub struct UnitsPlugin;

impl Plugin for UnitsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_update(GameState::BattleMap))
            .add_system_set(
                SystemSet::on_update(GameState::BattleMap).with_system(process_commands),
            );
    }
}


fn process_commands(
    time: Res<Time>,
    mut q_set: QuerySet<(
        QueryState<(Entity, &mut UnitCommands, &mut Transform)>,
        QueryState<&Transform, With<PlayerUnit>>,
        QueryState<&Transform, With<PlayerBase>>,
    )>,
    //map: Res<Map>,
    mut map: ResMut<CollisionMap>,
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
    for (entity, mut unit_commands, mut transform) in q_set.q0().iter_mut() {
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
                    println!("{:?} Thinking!", entity);
                    let mut rng = thread_rng();
                    let choices = ["wait", "attack"];
                    let weights = [1_i32, 5];
                    let dist = WeightedIndex::new(&weights).unwrap();
                    if choices[dist.sample(&mut rng)] == "wait" {
                        let wait: f32 = rng.gen_range(0.15..1.5);
                        println!("Slime {:?} is gonna wait for {} seconds!", entity, wait);
                        unit_commands.push(UnitCommand::Wait(wait));
                        unit_commands.push(UnitCommand::AiThink());
                        unit_commands.next();
                        continue;
                    }

                    let a = transform.translation.xy().as_ivec2() / TILE_SIZE;
                    //let a -= collisiotin.axis_offset();
                    if let Ok(base_pos) = base_pos {
                        //println!("{:?} at {}, Finding nearest player {:?}", entity, a, player_positions);
                        //let b = map.to_index_2d(b.as_vec2());
                        let b = base_pos.as_ivec2() / TILE_SIZE;
                        println!("Slime to base {}, {}", a, b);
                        let mut astar = AStar::new(10);
                        let i = map.to_index(b.into());
                        map.0.toggle_obstacle_index(i);
                        if let Some(path) = astar.find_path(&map.0, a.into(), b.into()) {
                            println!("Path {:?}", path);
                            if let Some(next) = path.get(1) {
                                let b = IVec2::from(*next);
                                //println!("Pathin {} to {} (no divide)", a, b);

                                // let a = a + map.half_offset();
                                // let b = b + map.half_offset();

                                //println!("Should see 'done moving' next");
                                unit_commands.push(UnitCommand::MoveToTile(a, b));
                                unit_commands.push(UnitCommand::AiThink());
                                println!("{:?} pushed {} to {} to commands for move. Stack state: {:?}. Calling next",entity, a, b, unit_commands.queue);
                                unit_commands.next();
                            }
                        } else {
                            println!("Couldn't find path!");
                            // Couldn't find a player to path to. Go for the kill!
                            //println!("Couldn't find a player!");
                        }
                        map.0.toggle_obstacle_index(i);
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
