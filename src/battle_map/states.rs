use std::time::Duration;

use bevy::prelude::*;
use bevy_ascii_terminal::{Terminal, Tile, TileWriter};
use sark_pathfinding::*;

use crate::{GameState, battle_map::units::{UnitCommands, UnitCommand}, config::{ConfigAsset, GameSettings}};
use super::{components::MapPosition, render::MapOverlayTerminal, input::{TileClickedEvent, Cursor}, Map, map::{TerrainTile, CollisionMap}, units::{MapUnit, UnitPath, MoveUnit, MapUnitMovement, AnimationTimer}, BattleMapState};

pub struct BattleMapSelectionPlugin;

impl Plugin for BattleMapSelectionPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<SelectionState>()
            .add_system_set(SystemSet::on_update(BattleMapState::SelectUnit)
            .with_system(select_unit.label("map_on_tile_clicked"))
            //.with_system(path.label("path").after("map_on_tile_clicked"))
            //.with_system(on_path.after("path"))
        )
        .add_system_set(SystemSet::on_update(BattleMapState::ChooseTarget)
            .with_system(choose_target))
        .add_system_set(SystemSet::on_exit(BattleMapState::ChooseTarget)
            .with_system(on_exit_choose_target))

        // .add_system_set(SystemSet::on_update(BattleMapState::UnitMoving)
        //     .with_system(unit_moving_update))
        ;
    }
}

#[derive(Default)]
pub struct SelectionState {
    selected_unit: Option<Entity>,
    target_unit: Option<Entity>,
    path: Vec<IVec2>,
}

impl SelectionState {
    pub fn clear(&mut self) {
        self.selected_unit = None;
        self.target_unit = None;
        self.path.clear();
    }
}

fn select_unit(
    mut state: ResMut<State<BattleMapState>>,
    mut ev_tile_clicked: EventReader<TileClickedEvent>,
    mut selection: ResMut<SelectionState>,
    // mut commands: Commands,
    // mut q_unit: Query<&mut MapPosition>,
    // q_moving_units: Query<&mut MoveUnit>,
    // mut q_unit_path: Query<&mut UnitPath>,
    // mut state: ResMut<State<BattleMapState>>,
) {
    for ev in ev_tile_clicked.iter() {
        if let Some(new_selected) = ev.unit {
            selection.clear();
            selection.selected_unit = Some(new_selected);
            println!("Selected {:?}", new_selected);
            state.set(BattleMapState::ChooseTarget).unwrap();
            return;
        } 
    }
    // if !q_moving_units.is_empty() {
    //     return;
    // }

    // for ev in ev_tile_clicked.iter() {
    //     if let Some(new_selected) = ev.unit {
    //         selection.unit = Some(new_selected);
    //         println!("Selected {:?}", new_selected);
    //         return;
    //     } 
    //     if let Some(selected) = selection.unit {
    //         if let Ok(mut pos) = q_unit.get_mut(selected) {
    //             if let Some(path) = selection.path
    //             if let Ok(mut path) = q_unit_path.get_mut(selected) {

    //             }
    //             //pos.xy = ev.xy;
    //             //println!("Moving {:?} to {}", selected, pos.xy);
    //             selection.unit = None;
    //             selection.path.clear();
    //         }
    //     } 
    // }
}

fn choose_target(
    mut commands: Commands,
    mut state: ResMut<State<BattleMapState>>,
    mut ev_tile_clicked: EventReader<TileClickedEvent>,
    mut selection: ResMut<SelectionState>,
    mut q_units: Query<(Entity, &MapPosition, &mut MapUnitMovement, &mut UnitPath), With<MapUnit>>,
    mut q_overlay: Query<&mut Terminal, With<MapOverlayTerminal>>,
    settings: Res<GameSettings>,
    q_cursor: Query<&MapPosition, With<Cursor>>,
    map: ResMut<CollisionMap>,
) {
    if let Some(unit) = selection.selected_unit {
        if let Ok(cursor_pos) = q_cursor.get_single() {
            if let Ok((_, unit_pos, _, _)) = q_units.get(unit) { 
                // Whee, I could stand to improve the api for pathfinding!
                let b = cursor_pos.xy + map.size().as_ivec2() / 2;
                let a = unit_pos.xy + map.size().as_ivec2() / 2;
                //let a_i = map.to_index(a.into());
                //let b_i = map.to_index(b.into());
                //println!("xy{} i {} to xy {} i {}", a, a_i, b, b_i);
                //map.0.toggle_obstacle_index(a_i);
                //map.0.toggle_obstacle_index(b_i);
                let mut astar = AStar::new(10);
                selection.path.clear();
                if let Ok(mut overlay) = q_overlay.get_single_mut() {
                    overlay.fill('a'.fg(Color::rgba_u8(0,0,0,0)));
                    
                    if let Some(path) = astar.find_path(&map.0, a.into(), b.into()) {
                        selection.path.extend(path.iter().map(|p|IVec2::from(*p)));
                        for p in path.iter() {
                            overlay.put_tile(*p, '*'.fg(Color::rgba_u8(0, 0, 255, 200)));
                        }
                    }
                }
            }
        }
    }

    for ev in ev_tile_clicked.iter() {
        if let Some(target) = ev.unit {
            // Clicked on a unit (possibly self)
            return;
        } else {
            if selection.path.is_empty() {
                println!("Invalid path");
                return;
            } else {
                if let Some(unit) = selection.selected_unit {
                    if let Ok((entity, pos,mut movement,mut path)) = q_units.get_mut(selection.selected_unit.unwrap()) {

                        commands.entity(unit).insert(MoveUnit);
                        let mut cmd = UnitCommands::new(settings.map_move_speed, settings.map_move_wait, pos.xy);
                        
                        for p in selection.path.iter().skip(1) {
                            cmd.push(UnitCommand::MoveToTile(*p));
                            cmd.push(UnitCommand::Wait(settings.map_move_wait));
                        }

                        selection.path.clear();

                        commands.entity(entity).insert(cmd);

                        state.set(BattleMapState::UnitMoving).unwrap();
                    } 
                }
            }
        }
    }
}

fn on_exit_choose_target(
    mut q_overlay: Query<&mut Terminal, With<MapOverlayTerminal>> 
) {
    if let Ok(mut overlay) = q_overlay.get_single_mut() {
        overlay.fill('a'.fg(Color::rgba_u8(0,0,0,0)));
    }
}

// fn unit_moving_update(
//     mut commands: Commands,
//     time: Res<Time>,
//     mut state: ResMut<State<BattleMapState>>,
//     map: Res<Map>,
//     mut q_unit: Query<(Entity, &mut Transform, &mut MapPosition, &mut UnitPath, &mut MapUnitMovement, &mut AnimationTimer), With<MoveUnit>>,
// ) {
//     for (entity, mut transform, mut pos, mut path, mut movement, timer) in q_unit.iter_mut() {
//         let speed = movement.map_move_speed / path.path.len() as f32;

//         if path.tile_changed() {
//             movement.wait_timer.tick(time.delta());
//             if movement.wait_timer.finished() {
//                 path.reset_tile_check();
//                 movement.wait_timer.reset();
//             }
//         }

//         path.current += f32::min(speed * time.delta().as_secs_f32(), 1.0);


//         // if path.current >= 1.0 {
//         //     pos.xy = *path.path.last().unwrap() - map.size().as_ivec2() / 2;
//         //     path.reset();
//         //     state.set(BattleMapState::SelectUnit).unwrap();
//         //     commands.entity(entity).remove::<MoveUnit>();
//         // } else {
//         //     if let Some(p) = path.path_point(path.current) { 
//         //         let p = p - map.size().as_vec2() / 2.0;
//         //         //let p = p.floor() + Vec2::new(0.5,0.5);
//         //         println!("Setting transform to {} with t {}", p, path.current);
//         //         transform.translation = p.extend(transform.translation.z) + Vec3::new(0.5,0.5,0.0);
//         //     }
//         // }
//     }
// }
