use bevy::{prelude::*, input::mouse::MouseWheel};
use bevy_tiled_camera::{TiledCameraBundle, TiledCameraPlugin, TiledProjection};

use crate::ResizeCamera;

pub struct GameCameraPlugin;

pub enum ZoomCamera {
    In,
    Out
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
            .add_system(zoom_test)
            .add_system(zoom)
            .add_system(resize);
    }
}

fn spawn(mut commands: Commands) {
    println!("Spawning camera");
    commands.spawn_bundle(
        TiledCameraBundle::new().with_tile_count([16, 16]), //.with_pixels_per_tile(16)
    );
    //commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn zoom_test(
    mut ev_scroll: EventReader<MouseWheel>,
    mut ev_zoom: EventWriter<ZoomCamera>,
) {
    for ev in ev_scroll.iter() {
        if ev.y > 0.0 {
            ev_zoom.send(ZoomCamera::In);
        }
        if ev.y < 0.0 {
            ev_zoom.send(ZoomCamera::Out);
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

fn zoom(
    mut ev_zoom: EventReader<ZoomCamera>,
    mut q_cam: Query<&mut TiledProjection>,
) {
    for ev in ev_zoom.iter() {  
        for mut proj in q_cam.iter_mut() {
            let zoom = proj.pixels_per_tile / 16;
            let zoom = match ev {
                ZoomCamera::In => u32::min(zoom + 1, 64),
                ZoomCamera::Out => u32::max(4, zoom - 1),
            };
            proj.pixels_per_tile = zoom * 16;
        }
    }

}
