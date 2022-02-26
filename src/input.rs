use bevy::prelude::*;
use bevy_tiled_camera::TiledProjection;

use super::battle_map::MapUnits;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup)
            .add_event::<TileClickedEvent>()
            .add_system(cursor_system);
    }
}

pub struct TileClickedEvent {
    pub xy: IVec2,
    pub unit: Option<Entity>,
}

#[derive(Component)]
pub struct Cursor;

fn setup(mut commands: Commands) {
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

fn cursor_system(
    input: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    q_camera: Query<(&Camera, &GlobalTransform, &TiledProjection)>,
    mut q_cursor: Query<(&mut Transform, &mut Visibility), With<Cursor>>,
    mut ev_tile_clicked: EventWriter<TileClickedEvent>,
    units: Res<MapUnits>,
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
                        let unit = units.get(xy);
                        println!("Clicked {}. Unit?: {:?}", xy, unit);
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
