use bevy::{prelude::*, math::Vec3Swizzles};
use bevy_tiled_camera::TiledProjection;
use sark_pathfinding::AStar;

use crate::{GameState, SETTINGS_PATH, config::ConfigAsset};

use super::{input::{TileClickedEvent, Cursor}, map::CollisionMap, Map, units::{UnitCommand, UnitCommands, PlayerUnit}};

pub struct BattleMapSelectionPlugin;

impl Plugin for BattleMapSelectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Selection>()
        .add_system_set(
            SystemSet::on_update(GameState::BattleMap)
            .with_system(on_select)
            .with_system(path_sprites)
        )
        ;
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
    collision_map: Res<CollisionMap>,
    map: Res<Map>,
    mut selection: ResMut<Selection>,
    mut ev_click: EventReader<TileClickedEvent>,
    configs: Res<Assets<ConfigAsset>>,
    mut q_unit_commands: Query<&mut UnitCommands>,
    q_pos: Query<&Transform>,
    q_player: Query<&PlayerUnit>,
    q_highlight: Query<Entity,With<HighlightSprite>>,
    q_cursor: Query<&Transform, With<Cursor>>,
) {
    if let Some(config) = configs.get(SETTINGS_PATH) {
        q_highlight.iter().for_each(|e|commands.entity(e).despawn());
        if let Some(selected) = selection.selected_unit {
            if let Ok(transform) = q_pos.get(selected) {
                let xy = transform.translation.xy();
                //let xy = xy + map.size().as_vec2() / 2.0;
                make_sprite(
                    &mut commands, 
                    xy, 
                    Color::rgba_u8(55,155,255,150)
                );
            }

            if let Ok(cursor_transform) = q_cursor.get_single() {
                let a = q_pos.get(selected).unwrap().translation.xy();
                let a = map.to_index_2d(a);
                let b = map.to_index_2d(cursor_transform.translation.xy());
                if a == b {
                    return;
                }
                selection.path = get_path(a,b, &collision_map);
            }
        }

        for ev in ev_click.iter() {
            if let Some(clicked_unit) = ev.unit {
                // Can only select player units
                if q_player.get(clicked_unit).is_ok() {
                    //println!("Highlighting unit {:?}", clicked_unit);
                    selection.selected_unit = Some(clicked_unit);
                    selection.path = None;
                } 
                //println!("PAth from {} to {}?", a, b);
            }
            
            if let (Some(path), Some(selected)) = (&selection.path, selection.selected_unit) {
                if let Ok(mut commands) = q_unit_commands.get_mut(selected) {
                    commands.clear();
                    for window in path.as_slice().windows(2) {
                        let [a,b] = [window[0],window[1]];
                        let [a,b] = [map.xy_from_index_2d(a), map.xy_from_index_2d(b)];
                        commands.push(UnitCommand::MoveToTile(a.as_ivec2(),b.as_ivec2()));
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
    if let Some(path) = astar.find_path(&map.0,a.into(),b.into()) {
        return Some(path.iter().map(|p|IVec2::from(*p)).collect::<Vec<IVec2>>());
    }
    None    
} 

#[derive(Component)]
struct PathSprite;
fn make_sprite(commands: &mut Commands, xy: Vec2, color: Color) {
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: color,
                custom_size: Some(Vec2::ONE),
                ..Default::default()
            },
            transform: Transform::from_xyz(xy.x, xy.y, 2.0),
            ..Default::default()
        })
        .insert(PathSprite);
}

fn path_sprites(
    mut commands: Commands,
    q_path_sprites: Query<Entity, With<PathSprite>>,
    selection: Res<Selection>,
    map: Res<Map>,
) {
    q_path_sprites.for_each(|e| commands.entity(e).despawn());
    if let Some(path) = &selection.path {
        for p in path.iter() {
            let xy = IVec2::from(*p).as_vec2() - map.size().as_vec2() / 2.0;
            let xy = xy + Vec2::new(0.5, 0.5);
            make_sprite(&mut commands, xy, Color::rgba_u8(200, 200, 200, 200));
        }
    }
}