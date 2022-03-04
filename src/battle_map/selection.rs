use bevy::{math::Vec3Swizzles, prelude::*};

use sark_pathfinding::AStar;

use crate::{config::ConfigAsset, make_sprite, GameState, SETTINGS_PATH, TILE_SIZE};

use super::{
    input::{Cursor, TileClickedEvent},
    map::CollisionMap,
    units::{PlayerUnit, UnitCommand, UnitCommands},
    MapUnits,
};

pub struct BattleMapSelectionPlugin;

impl Plugin for BattleMapSelectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Selection>().add_system_set(
            SystemSet::on_update(GameState::BattleMap)
                .with_system(on_select)
                .with_system(path_sprites),
        );
    }
}

#[derive(Default)]
struct Selection {
    selected_unit: Option<Entity>,
    path: Option<Vec<IVec2>>,
}

#[derive(Component)]
struct HighlightSprite;

fn on_select(
    mut commands: Commands,
    mut selection: ResMut<Selection>,
    mut ev_click: EventReader<TileClickedEvent>,
    mut q_unit_commands: Query<&mut UnitCommands>,
    configs: Res<Assets<ConfigAsset>>,
    map: Res<CollisionMap>,
    q_pos: Query<&Transform>,
    q_player: Query<&PlayerUnit>,
    q_highlight: Query<Entity, With<HighlightSprite>>,
    q_cursor: Query<&Transform, With<Cursor>>,
) {
    if let Some(config) = configs.get(SETTINGS_PATH) {
        q_highlight
            .iter()
            .for_each(|e| commands.entity(e).despawn());
        if let Some(selected) = selection.selected_unit {
            if let Ok(transform) = q_pos.get(selected) {
                make_sprite(
                    &mut commands,
                    transform.translation.xy(),
                    3,
                    Color::rgba_u8(55, 155, 255, 150),
                )
                .insert(PathSprite);
            }

            if let Ok(cursor_transform) = q_cursor.get_single() {
                let a = q_pos.get(selected).unwrap().translation.xy().as_ivec2() / TILE_SIZE;
                //let a = (a + center_offset).floor().as_ivec2();

                let b = cursor_transform.translation.xy().as_ivec2() / TILE_SIZE / TILE_SIZE;
                //let b = (b + center_offset).floor().as_ivec2();

                //let b = map.0.to_index_2d(cursor_transform.translation.xy());
                if a == b {
                    return;
                }
                //println!("Trying to get path from {} to {}", a, b);
                selection.path = get_path(a, b, &map);
                if selection.path.is_some() {
                    // println!("Found path: {:?}", selection.path);
                }
            }
        }

        for ev in ev_click.iter() {
            //println!("Read click");
            if let Some(clicked_unit) = ev.unit {
                //println!("Unit clicked: {:?}", clicked_unit);
                // Can only select player units
                if q_player.get(clicked_unit).is_ok() {
                    println!("Selected {:?}", clicked_unit);
                    //println!("Highlighting unit {:?}", clicked_unit);
                    selection.selected_unit = Some(clicked_unit);
                    selection.path = None;
                }
                //println!("PAth from {} to {}?", a, b);
            }

            if let (Some(path), Some(selected)) = (&selection.path, selection.selected_unit) {
                if let Ok(mut commands) = q_unit_commands.get_mut(selected) {
                    //let center_offset = map.size().as_ivec2() / 2;
                    commands.clear();
                    for window in path.as_slice().windows(2) {
                        let [a, b] = [window[0], window[1]];
                        //println!("Pathing from {} to {}", a, b);

                        commands.push(UnitCommand::MoveToTile(a, b));
                        commands.push(UnitCommand::Wait(config.settings.map_move_wait));
                    }
                }
                selection.path = None;
                selection.selected_unit = None;
            }
        }
    }
}

fn get_path(a: IVec2, b: IVec2, map: &CollisionMap) -> Option<Vec<IVec2>> {
    let mut astar = AStar::new(10);
    if let Some(path) = astar.find_path(&map.0, a.into(), b.into()) {
        //let offset = map.half_offset();
        return Some(path.iter().map(|p| IVec2::from(*p)).collect::<Vec<IVec2>>());
    }
    None
}

#[derive(Component)]
struct PathSprite;

fn path_sprites(
    mut commands: Commands,
    q_path_sprites: Query<Entity, With<PathSprite>>,
    selection: Res<Selection>,
    map: Res<MapUnits>,
) {
    q_path_sprites.for_each(|e| commands.entity(e).despawn());
    if let Some(path) = &selection.path {
        for p in path.iter() {
            let xy = IVec2::from(*p).as_vec2();
            let xy = xy * TILE_SIZE as f32;
            // println!("Trying to draw path at {}", xy);
            make_sprite(&mut commands, xy, 5, Color::rgba_u8(200, 200, 200, 200))
                .insert(PathSprite);
        }
    }
}
