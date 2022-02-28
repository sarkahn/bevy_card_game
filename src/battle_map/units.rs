use std::collections::VecDeque;

use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_ascii_terminal::Point2d;
use bevy_easings::*;

use crate::GameState;

use super::{components::MapPosition, BattleMapState, Map, MapUnits};

pub struct UnitsPlugin;

impl Plugin for UnitsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::BattleMap).with_system(spawn_units))
            .add_system_set(
                SystemSet::on_update(GameState::BattleMap)
                    .with_system(update_sprite_position)
                    .with_system(update_map_units),
            )
            .add_system_set(
                SystemSet::on_update(BattleMapState::UnitMoving).with_system(process_commands),
            );
    }
}

#[derive(Component, Default)]
pub struct MapUnit;

#[derive(Component, Default)]
pub struct MapUnitMovement {}

#[derive(Bundle, Default)]
pub struct MapUnitBundle {
    map_unit: MapUnit,
    pos: MapPosition,
    path: UnitPath,
}

impl MapUnitBundle {
    pub fn new(xy: IVec2) -> Self {
        Self {
            pos: MapPosition { xy },
            ..Default::default()
        }
    }
}

#[derive(Component, Default)]
pub struct UnitPath {
    pub path: Vec<IVec2>,
    pub current: f32,
    pub prev_index: usize,
    pub curr_index: usize,
    pub next_index: Option<usize>,
    tile_changed: bool,
}

impl UnitPath {
    pub fn reset(&mut self) {
        self.path.clear();
        self.current = 0.0;
    }

    pub fn reset_tile_check(&mut self) {
        self.prev_index = self.curr_index;
        self.curr_index = self.next_index.unwrap();
        self.next_index = None;
    }

    pub fn tile_changed(&self) -> bool {
        if let Some(next) = self.next_index {
            return self.curr_index == next;
        }
        false
    }

    pub fn is_done(&self) -> bool {
        self.curr_index == self.path.len() - 1
    }
}

impl UnitPath {
    pub fn path_point(&self, t: f32) -> Option<Vec2> {
        if self.path.is_empty() {
            return None;
        }
        //[3,3], [3,4], [3,5], [3,6], [3,7],
        // let a = path.path_point(0.166).unwrap();
        // let b = path.path_point(0.2001).unwrap();
        let max = (self.path.len() - 1) as f32;
        let next = (t * max).ceil() as usize;
        let curr = (t * max).floor() as usize;

        let curr_p = self.path[curr].as_vec2();
        let next_p = self.path[next].as_vec2();

        //println!("Curri {}, nexti {}", curr, next);

        // println!("Currp {}, nextp {}", curr, next);
        let v = t * max;
        // println!("V {}", v);
        let v = v - v.floor();
        // println!("V floored {}", v);
        let p = curr_p.lerp(next_p, v);

        Some(p)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnitCommand {
    MoveToTile(IVec2),
    Wait(f32),
}

#[derive(Component)]
pub struct UnitCommands {
    pos: IVec2,
    move_timer: Timer,
    wait_timer: Timer,
    queue: VecDeque<UnitCommand>,
    current: Option<UnitCommand>,
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
        self.queue.push_back(command);
    }
}

fn process_commands(
    time: Res<Time>,
    mut commands: Commands,
    mut q_unit: Query<(Entity, &mut UnitCommands, &mut MapPosition, &mut Transform)>,
    mut state: ResMut<State<BattleMapState>>,
    map: Res<Map>,
) {
    for (entity, mut unit_commands, mut pos, mut transform) in q_unit.iter_mut() {
        if unit_commands.current.is_none() {
            unit_commands.next();
        }
        //println!("Processing commands...count {}. Current {:?}", unit_commands.queue.len(), unit_commands.current);
        while let Some(command) = &unit_commands.current {
            match command {
                &UnitCommand::MoveToTile(b) => {
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
                        continue;
                    }
                    return;
                }
                UnitCommand::Wait(_) => {
                    unit_commands.wait_timer.tick(time.delta());
                    if unit_commands.wait_timer.finished() {
                        unit_commands.wait_timer.reset();
                        unit_commands.next();
                        continue;
                    }
                    return;
                }
            }
        }

        if unit_commands.current.is_none() {
            commands.entity(entity).remove::<UnitCommands>();
            println!("Exiting commands state");
            state.set(BattleMapState::SelectUnit).unwrap();
        }
    }
}

// fn make_map_unit(pos: impl Point2d, color: Color) -> MapUnitBundle {
//     let sprite_bundle = SpriteBundle {
//         sprite: Sprite {
//             color: color,
//             custom_size: Some(Vec2::ONE),
//             ..Default::default()
//         },
//         ..Default::default()
//     };
//     MapUnitBundle {
//         sprite_bundle,
//         pos: pos.xy().into(),
//         map_unit: Default::default(),
//         path: Default::default(),
//     }
// }

fn spawn_units(mut commands: Commands) {
    //commands.spawn_bundle(make_map_unit([-5, -5], Color::RED));
    //commands.spawn_bundle(make_map_unit([5, 5], Color::BLUE));
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
    q_units: Query<(Entity, &MapPosition), With<MapUnit>>,
) {
    if q_moved_units.is_empty() {
        return;
    }
    units.0.iter_mut().for_each(|f| *f = None);
    for (entity, pos) in q_units.iter() {
        let xy = pos.xy() + units.size().as_ivec2() / 2;
        units.0[xy] = Some(entity);
    }
}

#[cfg(test)]
mod test {
    use bevy::math::IVec2;
    use bevy_easings::Lerp;

    use super::UnitPath;

    #[test]
    fn move_test() {
        let path: Vec<[i32; 2]> = vec![[0, 0], [0, 1], [1, 1], [2, 1]];
        let path = UnitPath {
            path: path.iter().map(|p| IVec2::from(*p)).collect(),
            ..Default::default()
        };

        let p = path.path_point(0.25 * 0.5).unwrap();
        assert_eq!(p.to_array(), [0.0, 0.5]);
        let p = path.path_point(0.25 * 1.5).unwrap();
        assert_eq!(p.to_array(), [0.5, 1.0]);
    }

    #[test]
    fn limits() {
        let path: Vec<[i32; 2]> = vec![[0, 0], [0, 1], [1, 1], [2, 1]];
        let path = UnitPath {
            path: path.iter().map(|p| IVec2::from(*p)).collect(),
            ..Default::default()
        };
        let p = path.path_point(1.0).unwrap();
        assert_eq!([2.0, 1.0], p.to_array());

        let p = path.path_point(0.0).unwrap();
        assert_eq!([0.0, 0.0], p.to_array());
    }

    #[test]
    fn limits2() {
        let path: Vec<[i32; 2]> = vec![
            [3, 3],
            [4, 3],
            [5, 3],
            [6, 3],
            [7, 3],
            [8, 3],
            [9, 3],
            [10, 3],
            [11, 3],
            [12, 3],
            [13, 3],
        ];
        let path = UnitPath {
            path: path.iter().map(|p| IVec2::from(*p)).collect(),
            ..Default::default()
        };

        assert_eq!([4.0, 3.0], path.path_point(0.1).unwrap().to_array());
        assert_eq!([5.0, 3.0], path.path_point(0.2).unwrap().to_array());
        assert_eq!([6.0, 3.0], path.path_point(0.3).unwrap().to_array());
        assert_eq!([7.0, 3.0], path.path_point(0.4).unwrap().to_array());
        assert_eq!([8.0, 3.0], path.path_point(0.5).unwrap().to_array());
        assert_eq!([9.0, 3.0], path.path_point(0.6).unwrap().to_array());
        assert_eq!([10.0, 3.0], path.path_point(0.7).unwrap().to_array());
        assert_eq!([11.0, 3.0], path.path_point(0.8).unwrap().to_array());
        assert_eq!([12.0, 3.0], path.path_point(0.9).unwrap().to_array());
        assert_eq!([13.0, 3.0], path.path_point(1.0).unwrap().to_array());
    }

    #[test]
    fn testa() {
        let path: Vec<IVec2> = vec![[3, 3], [3, 4], [3, 5], [3, 6], [3, 7]]
            .iter()
            .map(|p| IVec2::from(*p))
            .collect();

        let path = UnitPath {
            path: path,
            ..Default::default()
        };

        let a = path.path_point(0.166).unwrap();
        let b = path.path_point(0.2001).unwrap();

        println!("A {} b {}", a, b);
    }
}
