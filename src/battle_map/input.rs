use bevy::prelude::*;
use bevy_tiled_camera::TiledProjection;

use crate::GameState;

use super::{MapUnits, MapPosition};

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
    commands.spawn_bundle(cursor).insert(Cursor).insert(MapPosition::default());
}

fn on_exit(
    mut commands: Commands,
    q_cursor: Query<Entity, With<Cursor>>,
) {
    for entity in q_cursor.iter() {
        commands.entity(entity).despawn();
    }
}


fn cursor_system(
    input: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    q_camera: Query<(&Camera, &GlobalTransform, &TiledProjection)>,
    mut q_cursor: Query<(&mut Transform, &mut MapPosition, &mut Visibility), With<Cursor>>,
    mut ev_tile_clicked: EventWriter<TileClickedEvent>,
    units: Res<MapUnits>,
) {
    let window = windows.get_primary().unwrap();

    if let Some(pos) = window.cursor_position() {
        for (cam, cam_transform, proj) in q_camera.iter() {
            if let Some(p) = proj.screen_to_world(cam, &windows, cam_transform, pos) {
                if let Some(mut p) = proj.world_to_tile_center(cam_transform, p) {
                    p.z = 2.0;

                    let (mut cursor_transform, mut map_pos, mut v) = q_cursor.single_mut();
                    v.is_visible = true;

                    cursor_transform.translation = p;
                    map_pos.xy = p.truncate().floor().as_ivec2();

                    if input.just_pressed(MouseButton::Left) {
                        let xy = proj.world_to_tile(cam_transform, p).unwrap();
                        let unit = units.get(xy);
                        //println!("Sending click event! {}: {:?}", xy, unit);
                        ev_tile_clicked.send(TileClickedEvent { xy, unit });
                    }

                    return;
                }
            }
        }
    }
    let (_, _, mut v) = q_cursor.single_mut();
    v.is_visible = false;
}
