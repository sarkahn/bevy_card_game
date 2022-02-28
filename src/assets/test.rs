use bevy::{prelude::*, utils::HashMap};
use bevy_egui::{egui, EguiContext};

use crate::{GameState, ldtk_loader::LdtkMap, AtlasHandles, AnimationController};

use super::{assets::PrefabAsset, unit::{UnitComponents, UnitAsset}};

pub struct AssetTestPlugin;

impl Plugin for AssetTestPlugin {
    fn build(&self, app: &mut App) {
        app
        .insert_resource(ClearColor(Color::rgba_u8(50,50,50,50)))
        .init_resource::<LoadingPrefabs>()
        .add_system_set(
            SystemSet::on_enter(GameState::AssetTest).with_system(setup)
        )
        // .add_system_set(
        //     SystemSet::on_update(GameState::AssetTest).with_system(load_window)
        // )
        .add_system_set(
            SystemSet::on_update(GameState::AssetTest).with_system(on_load)
        )
        ;
    }
}

#[derive(Default)]
struct LoadingPrefabs(Vec<Handle<PrefabAsset>>);

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut loading: ResMut<LoadingPrefabs>, 
) {
    //let handle = asset_server.load("units/guy.prefab");
    asset_server.watch_for_changes().unwrap();
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    loading.0.push(asset_server.load("units/guy.unit"));

}

fn on_load(
    mut commands: Commands,
    mut ev_loaded: EventReader<AssetEvent<UnitAsset>>,
    units: Res<Assets<UnitAsset>>,
    ldtk_maps: Res<Assets<LdtkMap>>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
    mut atlas_handles: ResMut<AtlasHandles>,
) {
    for ev in ev_loaded.iter() {
        match ev {
            AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
                let unit = units.get(handle).unwrap();
                let ldtk = ldtk_maps.get(unit.map_ldtk_file.clone()).unwrap();
                let tileset = ldtk.tileset_from_name(&unit.components.arena_sprite.tileset_name);
                println!("Looking for tileset {}", unit.components.arena_sprite.tileset_name);
                if let Some(tileset) = tileset {
                    let atlas = match atlas_handles.0.get(&tileset.name) {
                        Some(atlas) => atlas.clone(),
                        None => {
                            let image = ldtk.image_from_name(&tileset.name).unwrap();
                            let atlas = TextureAtlas::from_grid(
                                image.clone(), IVec2::splat(tileset.tile_size).as_vec2(), 
                                tileset.tile_count.x as usize, tileset.tile_count.y as usize
                            );
                            let handle = atlases.add(atlas);
                            atlas_handles.0.insert(tileset.name.clone(), handle.clone());
                            handle
                        },
                    };
                    let sprite = TextureAtlasSprite {
                        custom_size: Some(Vec2::ONE),
                        index: unit.components.arena_sprite.index as usize,
                        ..Default::default()
                    };
                    let sprite = SpriteSheetBundle {
                        sprite,
                        texture_atlas: atlas.clone(),
                        transform: Transform::from_xyz(5.0, 5.0, 0.0),
                        ..Default::default()
                    };
                    let mut animator = AnimationController::with_frame_time(0.2);
                    if let Some(animations) = &unit.components.arena_sprite.animations {
                        for (name, anim) in animations {
                            animator.add(name, anim.clone());
                        }

                        if let Some(name) = &unit.components.arena_sprite.default_animation {
                            animator.play(name);
                        } 
                    } 
                    commands.spawn_bundle(sprite).insert(animator);
                }
                //let map_image = images.get(unit.map_ldtk_file.clone()).unwrap();
            },
            _ => {}
        }
    }
}

// #[derive(Default)]
// struct UiState {
//     input: String,
// }

// fn load_window(
//     mut ctx: ResMut<EguiContext>,
//     mut input_state: Local<UiState>,
//     asset_server: Res<AssetServer>,
//     mut loading: ResMut<LoadingPrefabs>, 
//     prefabs: Res<Assets<PrefabAsset>>,
// ) {
//     egui::Window::new("Load Prefab").show(ctx.ctx_mut(), |ui| {
//         ui.horizontal(|ui| {
//             ui.label("Load Prefab:");
//             let response = ui.add(
//                 egui::TextEdit::singleline(&mut input_state.input)
//             );
//             if response.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
//                 if let Some(_) = prefabs.get(&input_state.input) {
//                     println!("{} is already loaded!", input_state.input);
//                     return;
//                 }
//                 println!("Attemping to load prefab at {}", input_state.input);
//                 let path = &input_state.input;
//                 let handle: Handle<PrefabAsset> = asset_server.load(path);
//                 loading.0.push(handle);
//                 input_state.input.clear();
//             }
//         });
//     });
// }

// fn on_loaded(
//     mut commands: Commands,
//     asset_server: Res<AssetServer>,
//     mut loading: ResMut<LoadingPrefabs>,
//     prefabs: Res<Assets<PrefabAsset>>,
//     mut atlases: ResMut<Assets<TextureAtlas>>,
//     mut ev_loaded: EventReader<AssetEvent<PrefabAsset>>,
//     mut q_sprites: Query<Entity, With<TextureAtlasSprite>>,
//     mut atlas_map: ResMut<AtlasMap>,
// ) {
//     for ev in ev_loaded.iter() {
//         match ev {
//             AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
                
//                 let i = loading.0.iter().position(|h|h==handle).unwrap();
//                 loading.0.remove(i);
//                 let pfb = prefabs.get(handle).unwrap();
//                 let unit: UnitComponents = ron::de::from_str(&pfb.string).unwrap();
//                 spawn_from_unit_components(&mut commands, &mut atlases, &mut atlas_map, &asset_server, &unit);
//             },
//             AssetEvent::Removed { handle: _ } => {},
//         }
//     }
// }

// #[derive(Default)]
// struct AtlasMap {
//     map: HashMap<String,Handle<TextureAtlas>>,
// }

// fn spawn_from_unit_components(
//     mut commands: &mut Commands,
//     atlases: &mut Assets<TextureAtlas>,
//     mut atlas_map: &mut AtlasMap,
//     asset_server: &AssetServer,
//     unit: &UnitComponents,
// ) {
//     // let map_image_path = &unit.map_sprite.image;
//     // let map_image: Handle<Image> = asset_server.load(map_image_path);
//     // let atlas = match atlas_map.map.get(map_image_path) {
//     //     Some(handle) => handle,
//     //     None => {
//     //         let atlas = TextureAtlas::from_grid(texture, tile_size, columns, rows)
//     //     },
//     // };
//     // let map_atlas = TextureAtlas {
//     //     texture: map_image,
//     //     size: todo!(),
//     //     textures: todo!(),
//     //     texture_handles: todo!(),
//     // };

//     // // let map_bundle = SpriteSheetBundle {
//     // //     sprite: todo!(),
//     // //     texture_atlas: todo!(),
//     // //     transform: todo!(),
//     // // };

//     // let arena_image: Handle<Image> = asset_server.load(&unit.arena_sprite.image);
//     // let arena_index = unit.arena_sprite.index;
// }