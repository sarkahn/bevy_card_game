use bevy::prelude::*;
use bevy_tiled_camera::TiledProjection;

use crate::GameState;

use super::{Map, MapUnits};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::BattleMap).with_system(on_enter))
            .add_system_set(SystemSet::on_exit(GameState::BattleMap).with_system(on_exit))
            .add_event::<TileClickedEvent>()
            .add_system_set(SystemSet::on_update(GameState::BattleMap).with_system(cursor_system));
    }
}

pub struct TileClickedEvent {
    pub xy: IVec2,
    pub unit: Option<Entity>,
}

#[derive(Component, Default)]
pub struct Cursor;

fn on_enter(mut commands: Commands) {
    let col = Color::rgba(1.0, 1.0, 1.0, 0.35);
    let cursor = SpriteBundle {
        sprite: Sprite {
            color: col,
            custom_size: Some(Vec2::ONE),
            ..Default::default()
        },
        transform: Transform::from_xyz(0.0, 0.0, 2.0),
        ..Default::default()
    };
    commands.spawn_bundle(cursor).insert(Cursor);
}

fn on_exit(mut commands: Commands, q_cursor: Query<Entity, With<Cursor>>) {
    for entity in q_cursor.iter() {
        commands.entity(entity).despawn();
    }
}

fn cursor_system(
    input: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    q_camera: Query<(&Camera, &GlobalTransform, &TiledProjection)>,
    mut q_cursor: Query<(&mut Transform, &mut Visibility), With<Cursor>>,
    mut ev_tile_clicked: EventWriter<TileClickedEvent>,
    units: Res<MapUnits>,
    map: Res<Map>,
) {
    let window = windows.get_primary().unwrap();

    if let Some(pos) = window.cursor_position() {
        for (cam, cam_transform, proj) in q_camera.iter() {
            if let Some(p) = proj.screen_to_world(cam, &windows, cam_transform, pos) {
                if let Some(mut p) = proj.world_to_tile_center(cam_transform, p) {
                    p.z = 2.0;

                    let (mut cursor_transform, mut v) = q_cursor.single_mut();
                    v.is_visible = true;

                    cursor_transform.translation = p;

                    if input.just_pressed(MouseButton::Left) {
                        let xy = proj.world_to_tile(cam_transform, p).unwrap();
                        let index = map.to_index_2d(xy.as_vec2());
                        let unit = units[index];
                        //println!("Sending click event! {}: {:?}", xy, unit);
                        //ev_tile_clicked.send(TileClickedEvent { xy, unit });
                        ev_tile_clicked.send(TileClickedEvent { xy, unit });
                    }

                    return;
                }
            }
        }
    }
    let (_, mut v) = q_cursor.single_mut();
    v.is_visible = false;
}
