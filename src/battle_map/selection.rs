use bevy::prelude::*;
use bevy_ascii_terminal::{Terminal, Tile, TileWriter};

use crate::{input::TileClickedEvent, GameState};

use super::{components::MapPosition, render::MapOverlayTerminal};

pub struct BattleMapSelectionPlugin;

impl Plugin for BattleMapSelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_update(GameState::BattleMap).with_system(select_unit));
    }
}

pub struct SelectionState {
    cursor_pos: MapPosition,
    selected_unit: Option<Entity>,
}

fn select_unit(
    mut ev_selection: EventReader<TileClickedEvent>,
    mut q_overlay: Query<&mut Terminal, With<MapOverlayTerminal>>,
) {
    for ev in ev_selection.iter() {
        if let Some(unit) = ev.unit {
            if let Ok(mut overlay) = q_overlay.get_single_mut() {
                //overlay.fill('*'.fg(Color::rgba_u8(0,0,0,0)));
                //let offset = overlay.size().as_ivec2() / 2;
                //overlay.fill('*'.fg(Color::BLUE).bg(Color::BLACK));
                //overlay.put_tile(ev.xy + offset, '*'.fg(Color::ORANGE).bg(Color::YELLOW));
            }
        }
    }
}
