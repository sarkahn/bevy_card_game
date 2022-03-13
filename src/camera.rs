use bevy::{input::mouse::MouseWheel, prelude::*, render::camera::WindowOrigin};
use bevy_tiled_camera::{TiledCameraBundle, TiledCameraPlugin, TiledProjection};

use crate::{ResizeCamera, TILE_SIZE};

pub struct GameCameraPlugin;

pub enum ZoomCamera {
    In,
    Out,
}
impl Default for ZoomCamera {
    fn default() -> Self {
        ZoomCamera::In
    }
}

impl Plugin for GameCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn)
            .add_plugin(TiledCameraPlugin)
            .add_event::<ZoomCamera>()
            //.add_system(input)
            .add_system(resize);
    }
}

fn spawn(mut commands: Commands) {
    //println!("Spawning camera");
    // commands.spawn_bundle(
    //     TiledCameraBundle::new().with_tile_count([16, 16]), //.with_pixels_per_tile(16)
    // );
    let mut cam = OrthographicCameraBundle::new_2d();
    //let hor = Vec3::new(0.5, 0.0, 0.0) * TILE_SIZE as f32;
    //let ver = Vec3::new(0.0, 0.5, 0.0) * TILE_SIZE as f32;
    //cam.transform.translation -= hor + ver;
    //cam.orthographic_projection.scale = 1.0 / 64.0;
    cam.transform = Transform::from_xyz(1856.0 / 2.0, 1024.0 / 2.0, cam.transform.translation.z);
    commands.spawn_bundle(cam);
}

fn input(
    time: Res<Time>,
    mut ev_scroll: EventReader<MouseWheel>,
    keyboard: Res<Input<KeyCode>>,
    mut q_cam: Query<(&mut Transform, &mut TiledProjection)>,
) {
    if let Ok((mut transform, mut proj)) = q_cam.get_single_mut() {
        for ev in ev_scroll.iter() {
            let mut zoom = proj.pixels_per_tile / 16;
            if ev.y > 0.0 {
                zoom = u32::min(zoom + 1, 64);
            }
            if ev.y < 0.0 {
                zoom = u32::max(4, zoom - 1);
            }
            proj.pixels_per_tile = zoom * 16;
        }

        let pixel = 1.0 / 64.0;
        let speed = 64.0 * 5.0;
        let vel = speed * pixel * time.delta_seconds();
        let up = Vec3::new(0.0, vel, 0.0);
        let right = Vec3::new(vel, 0.0, 0.0);
        let mut movement = Vec3::ZERO;

        if keyboard.pressed(KeyCode::W) {
            movement += up;
        }

        if keyboard.pressed(KeyCode::S) {
            movement += -up;
        }

        if keyboard.pressed(KeyCode::A) {
            movement += -right;
        }
        if keyboard.pressed(KeyCode::D) {
            movement += right;
        }

        if movement != Vec3::ZERO {
            transform.translation += movement;
        }
    }
}

fn resize(
    mut commands: Commands,
    q: Query<(Entity, &ResizeCamera)>,
    mut q_cam: Query<&mut TiledProjection>,
) {
    for (entity, size) in q.iter() {
        if let Ok(mut proj) = q_cam.get_single_mut() {
            proj.set_tile_count(size.0.as_uvec2().into());
            commands.entity(entity).despawn();
        }
    }
}

// fn zoom(
//     mut ev_zoom: EventReader<ZoomCamera>,
//     mut q_cam: Query<&mut TiledProjection>,
// ) {
//     for ev in ev_zoom.iter() {
//         for mut proj in q_cam.iter_mut() {
//             let zoom = proj.pixels_per_tile / 16;
//             let zoom = match ev {
//                 ZoomCamera::In => u32::min(zoom + 1, 64),
//                 ZoomCamera::Out => u32::max(4, zoom - 1),
//             };
//             proj.pixels_per_tile = zoom * 16;
//         }
//     }

// }
