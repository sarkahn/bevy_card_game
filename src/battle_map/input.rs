use bevy::{prelude::*, math::Vec3Swizzles};
use bevy_tiled_camera::TiledProjection;

use crate::{GameState, TILE_SIZE};

use super::{MapUnits};

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
            //custom_size: Some(Vec2::ONE),
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
    q_camera: Query<(&Camera, &GlobalTransform, &OrthographicProjection)>,
    mut q_cursor: Query<(&mut Transform, &mut Visibility), With<Cursor>>,
    mut ev_tile_clicked: EventWriter<TileClickedEvent>,
    units: Res<MapUnits>,
) {
    let window = windows.get_primary().unwrap();

    if let Some(pos) = window.cursor_position() {
        for (cam, cam_transform, proj) in q_camera.iter() {
            if let Some(mut p) = screen_to_world(cam, &windows, cam_transform, pos) {

                p.z = 2.0;

                let (mut cursor_transform, mut v) = q_cursor.single_mut();
                v.is_visible = true;

                cursor_transform.translation = p * TILE_SIZE as f32;

                if input.just_pressed(MouseButton::Left) {
                    let xy = p.xy().floor() + Vec2::new(0.5,0.5) * TILE_SIZE as f32;

                    let i = units.xy_to_index(xy);
                    println!("Clicked {}. Index {}", p, i );
                    let unit = units.get_from_index(i);
                    ev_tile_clicked.send(TileClickedEvent { xy: xy.as_ivec2(), unit });

                    // let xy = xy -  offset;
                    // //println!("Sending click event! {}: {:?}", xy, unit);
                }

                return;
            }
        }
    }
    let (_, mut v) = q_cursor.single_mut();
    v.is_visible = false;
}


    /// Converts a screen position [0..resolution] to a world position
    pub fn screen_to_world(
        camera: &Camera,
        windows: &Windows,
        camera_transform: &GlobalTransform,
        screen_pos: Vec2,
    ) -> Option<Vec3> {
        let window = windows.get(camera.window)?;
        let window_size = Vec2::new(window.width(), window.height());

        // Convert screen position [0..resolution] to ndc [-1..1]
        let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;

        let min = -Vec2::ONE;
        let max = Vec2::ONE;
        let below_min = !ndc.cmpge(min);
        let above_max = !ndc.cmplt(max);
        if below_min.any() || above_max.any() {
            return None;
        }

        let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix.inverse();

        let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));
        let world_pos = world_pos.truncate().extend(0.0);

        Some(world_pos)
    }