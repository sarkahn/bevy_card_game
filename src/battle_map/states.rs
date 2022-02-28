use std::time::Duration;

use bevy::prelude::*;
use sark_pathfinding::*;

use super::{
    components::MapPosition,
    input::{Cursor, TileClickedEvent},
    map::*,
    //render::MapOverlayTerminal,
    units::{MapUnit, MapUnitMovement, UnitPath},
    BattleMapState,
    Map,
};
use crate::{
    battle_map::units::{UnitCommand, UnitCommands},
    config::{ConfigAsset, GameSettings},
    GameState,
};

pub struct BattleMapStatesPlugin;

impl Plugin for BattleMapStatesPlugin {
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

#[derive(Component)]
struct PathSprite;
fn make_path_sprite(commands: &mut Commands, xy: Vec2) {
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgba_u8(200, 200, 200, 200),
                custom_size: Some(Vec2::ONE),
                ..Default::default()
            },
            transform: Transform::from_xyz(xy.x, xy.y, 2.0),
            ..Default::default()
        })
        .insert(PathSprite);
}
/// Offset a given axis based on whether it's even or odd.
/// Allows for a nicely centered map even with odd numbered tiles.
fn axis_offset(size: IVec2) -> Vec2 {
    let cmp = (size % 2).cmpeq(IVec2::ZERO);
    Vec2::select(cmp, Vec2::new(0.5, 0.5), Vec2::ZERO)
}
fn choose_target(
    mut commands: Commands,
    mut state: ResMut<State<BattleMapState>>,
    mut ev_tile_clicked: EventReader<TileClickedEvent>,
    mut selection: ResMut<SelectionState>,
    mut q_units: Query<(Entity, &MapPosition), With<MapUnit>>,
    //mut q_overlay: Query<&mut Terminal, With<MapOverlayTerminal>>,
    settings: Res<GameSettings>,
    q_cursor: Query<&MapPosition, With<Cursor>>,
    q_path_sprites: Query<Entity, With<PathSprite>>,
    map: ResMut<CollisionMap>,
) {
    q_path_sprites.for_each(|e| commands.entity(e).despawn());
    if let Some(unit) = selection.selected_unit {
        if let Ok(cursor_pos) = q_cursor.get_single() {
            if let Ok((_, unit_pos)) = q_units.get(unit) {
                let b = cursor_pos.xy + map.size().as_ivec2() / 2;
                let a = unit_pos.xy + map.size().as_ivec2() / 2;
                selection.path.clear();
                let mut astar = AStar::new(10);
                if let Some(path) = astar.find_path(&map.0, a.into(), b.into()) {
                    for p in path {
                        let xy = IVec2::from(*p).as_vec2() - map.size().as_vec2() / 2.0;
                        let xy = xy + Vec2::new(0.5, 0.5);
                        make_path_sprite(&mut commands, xy);
                    }
                    //println!("Path length {}", path.len());
                    selection.path.extend(path.iter().map(|p| IVec2::from(*p)));
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
                    if let Ok((entity, pos)) = q_units.get_mut(unit) {
                        let mut cmd = UnitCommands::new(
                            settings.map_move_speed,
                            settings.map_move_wait,
                            pos.xy,
                        );

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

fn on_exit_choose_target(mut commands: Commands, q_path_sprites: Query<Entity, With<PathSprite>>) {
    q_path_sprites.for_each(|e| commands.entity(e).despawn());
}
