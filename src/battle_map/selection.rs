use bevy::{prelude::*, math::Vec3Swizzles};
use bevy_tiled_camera::TiledProjection;
use sark_pathfinding::AStar;

use crate::{GameState, SETTINGS_PATH, config::ConfigAsset};

use super::{MapPosition, MapUnits, input::TileClickedEvent, map::CollisionMap, Map, units::{UnitCommand, UnitCommands, PlayerUnit}};

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
    map: Res<CollisionMap>,
    mut selection: ResMut<Selection>,
    mut ev_click: EventReader<TileClickedEvent>,
    configs: Res<Assets<ConfigAsset>>,
    mut q_unit_commands: Query<&mut UnitCommands>,
    q_pos: Query<(&Transform,&MapPosition)>,
    q_player: Query<&PlayerUnit>,
    q_highlight: Query<Entity,With<HighlightSprite>>,
) {
    if let Some(config) = configs.get(SETTINGS_PATH) {
        q_highlight.iter().for_each(|e|commands.entity(e).despawn());
        if let Some(selected) = selection.selected_unit {
            if let Ok((transform,_)) = q_pos.get(selected) {
                let xy = transform.translation.xy();
                //let xy = xy + map.size().as_vec2() / 2.0;
                make_sprite(
                    &mut commands, 
                    xy, 
                    Color::rgba_u8(55,155,255,150)
                );
            }
        }

        for ev in ev_click.iter() {
            if let Some(clicked_unit) = ev.unit {
                // Can only select player units
                if q_player.get(clicked_unit).is_ok() {
                    println!("Highlighting unit {:?}", clicked_unit);
                    selection.selected_unit = Some(clicked_unit);
                    selection.path = None;
                }
            }
    
            if let Some(selected) = selection.selected_unit {
                let a = q_pos.get(selected).unwrap().1.xy + map.size().as_ivec2() / 2;
                let b = ev.xy;
                println!("PAth from {} to {}?", a, b);
                if a == b {
                    return;
                }
                if let Some(path) = get_path(a,b, &map) {

                    if let Ok(mut commands) = q_unit_commands.get_mut(selected) {
                        commands.clear();
                        for p in path.iter().skip(1) {
                            commands.push(UnitCommand::MoveToTile(*p));
                            commands.push(UnitCommand::Wait(config.settings.map_move_wait));
                        }
                    }
                    selection.path = Some(path);
                    selection.selected_unit = None;
                }
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