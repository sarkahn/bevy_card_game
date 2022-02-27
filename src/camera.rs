use bevy::prelude::*;
use bevy_tiled_camera::{TiledCameraBundle, TiledProjection, TiledCameraPlugin};

use crate::ResizeCamera;

pub struct GameCameraPlugin;

impl Plugin for GameCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn)
        .add_plugin(TiledCameraPlugin)
        .add_system(resize);
    }
}


fn spawn(
    mut commands: Commands,
) {
    println!("Spawning camera");
    commands.spawn_bundle(TiledCameraBundle::new()
    .with_tile_count([16,16])
    //.with_pixels_per_tile(16)
    );
    //commands.spawn_bundle(OrthographicCameraBundle::new_2d());
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