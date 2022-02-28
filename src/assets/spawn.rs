use bevy::prelude::*;

use crate::{ldtk_loader::LdtkMap, AnimationController, AtlasHandles};

use super::unit::LoadUnitPrefab;

pub struct PrefabSpawnPlugin;

impl Plugin for PrefabSpawnPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::rgba_u8(50, 50, 50, 50)))
            .init_resource::<LoadingHandles>()
            .add_event::<SpawnUnitFromPrefab>()
            .add_system(spawn_unit)
            .add_system(on_load);
    }
}

#[derive(Default)]
pub struct SpawnUnitFromPrefab(String);

#[derive(Default)]
struct LoadingHandles(Vec<HandleUntyped>);

fn spawn_unit(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut handles: ResMut<LoadingHandles>,
    mut ev_load: EventReader<SpawnUnitFromPrefab>,
) {
    for ev in ev_load.iter() {
        asset_server.watch_for_changes().unwrap();
        commands.spawn_bundle(OrthographicCameraBundle::new_2d());
        let asset: Handle<LoadUnitPrefab> = asset_server.load(ev.0.as_str());
        handles.0.push(asset.clone_untyped());
    }
}

fn on_load(
    mut commands: Commands,
    mut loading_handles: ResMut<LoadingHandles>,
    mut ev_loaded: EventReader<AssetEvent<LoadUnitPrefab>>,
    mut spawns: ResMut<Assets<LoadUnitPrefab>>,
    ldtk_maps: Res<Assets<LdtkMap>>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
    mut atlas_handles: ResMut<AtlasHandles>,
) {
    // for ev in ev_loaded.iter() {
    //     match ev {
    //         AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
    //             if let Some(i) = loading_handles
    //                 .0
    //                 .iter()
    //                 .position(|h| h.clone().typed::<LoadUnitPrefab>() == handle.clone())
    //             {
    //                 loading_handles.0.remove(i);
    //             }
    //             let unit = spawns.get(handle).unwrap();
    //             let ldtk = ldtk_maps.get(unit.ldtk_file.clone()).unwrap();
    //             let tileset = ldtk.tileset_from_name(&unit.components.sprite_data.tileset_name);
    //             //println!("Looking for tileset {}", unit.components.arena_sprite.tileset_name);
    //             if let Some(tileset) = tileset {
    //                 let atlas = match atlas_handles.0.get(&tileset.name) {
    //                     Some(atlas) => atlas.clone(),
    //                     None => {
    //                         let image = ldtk.image_from_name(&tileset.name).unwrap();
    //                         let atlas = TextureAtlas::from_grid(
    //                             image.clone(),
    //                             IVec2::splat(tileset.tile_size).as_vec2(),
    //                             tileset.tile_count.x as usize,
    //                             tileset.tile_count.y as usize,
    //                         );
    //                         let handle = atlases.add(atlas);
    //                         atlas_handles.0.insert(tileset.name.clone(), handle.clone());
    //                         handle
    //                     }
    //                 };
    //                 let sprite = TextureAtlasSprite {
    //                     custom_size: Some(Vec2::ONE),
    //                     index: unit.components.sprite_data.index as usize,
    //                     ..Default::default()
    //                 };
    //                 let sprite = SpriteSheetBundle {
    //                     sprite,
    //                     texture_atlas: atlas.clone(),
    //                     transform: Transform::from_xyz(5.0, 5.0, 0.0),
    //                     ..Default::default()
    //                 };
    //                 let mut animator = AnimationController::default();
    //                 if let Some(animations) = &unit.components.sprite_data.animations {
    //                     for (name, anim) in animations {
    //                         animator.add(name, anim.clone());
    //                     }

    //                     if let Some(name) = &unit.components.sprite_data.default_animation {
    //                         animator.play(name);
    //                     }
    //                 }
    //                 commands.spawn_bundle(sprite).insert(animator);
    //                 spawns.remove(handle);
    //             }
    //         }
    //         _ => {}
    //     }
    // }
}
