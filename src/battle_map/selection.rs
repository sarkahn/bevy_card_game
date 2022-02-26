use bevy::prelude::*;
use bevy_ascii_terminal::{Terminal, Tile, TileWriter};
use sark_pathfinding::*;

use crate::{GameState};
use super::{components::MapPosition, render::MapOverlayTerminal, input::{TileClickedEvent, Cursor}, Map, map::{TerrainTile, CollisionMap}, units::MapUnit};

pub struct BattleMapSelectionPlugin;

impl Plugin for BattleMapSelectionPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<SelectionState>()
            .add_system_set(SystemSet::on_update(GameState::BattleMap)
            .with_system(on_tile_clicked.label("map_on_tile_clicked"))
            .with_system(path.label("path").after("map_on_tile_clicked"))
            .with_system(on_path.after("path"))
        );
    }
}

#[derive(Default)]
pub struct SelectionState {
    cursor_pos: MapPosition,
    unit: Option<Entity>,
    path: Vec<IVec2>,
}

fn on_tile_clicked(
    mut ev_tile_clicked: EventReader<TileClickedEvent>,
    mut selection: ResMut<SelectionState>,
    mut q_unit: Query<&mut MapPosition>,
) {
    for ev in ev_tile_clicked.iter() {
        if let Some(new_selected) = ev.unit {
            selection.unit = Some(new_selected);
            println!("Selected {:?}", new_selected);
            return;
        } 
        if let Some(selected) = selection.unit {
            if let Ok(mut pos) = q_unit.get_mut(selected) {
                pos.xy = ev.xy;
                //println!("Moving {:?} to {}", selected, pos.xy);
                selection.unit = None;
                selection.path.clear();
            }
        } 
    }
}

fn path(
    mut selection: ResMut<SelectionState>,
    mut q_overlay: Query<&mut Terminal, With<MapOverlayTerminal>>,
    q_cursor: Query<&MapPosition, With<Cursor>>,
    q_unit: Query<&MapPosition, With<MapUnit>>,
    mut map: ResMut<CollisionMap>,
) {
    if let Some(unit) = selection.unit {
        if let Ok(cursor_pos) = q_cursor.get_single() {
            if let Ok(unit_pos) = q_unit.get(unit) {
                // Whee, I could stand to improve the api for pathfinding!
                let a = cursor_pos.xy + map.size().as_ivec2() / 2;
                let b = unit_pos.xy + map.size().as_ivec2() / 2;
                let a_i = map.to_index(a.into());
                let b_i = map.to_index(b.into());
                //println!("xy{} i {} to xy {} i {}", a, a_i, b, b_i);
                //map.0.toggle_obstacle_index(a_i);
                //map.0.toggle_obstacle_index(b_i);
                selection.path.clear();
                let mut astar = AStar::new(10);
                if let Some(path) = astar.find_path(&map.0, a.into(), b.into()) {
                    selection.path.extend(path.iter().map(|p|IVec2::from(*p)));
                }
            }
        }
    }
}

fn on_path(
    selection: Res<SelectionState>,
    mut q_overlay: Query<&mut Terminal, With<MapOverlayTerminal>>,
) {


    if let Ok(mut term) = q_overlay.get_single_mut() {
        term.fill('a'.fg(Color::rgba_u8(0,0,0,0)));
        if !selection.is_changed() {
            return;
        }
        for p in selection.path.iter() {
            term.put_tile(*p, 'a'.fg(Color::rgba_u8(0,0,255,200)));
        }
    }
}