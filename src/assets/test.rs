use bevy::{prelude::*, utils::HashMap};
use bevy_egui::{egui, EguiContext};

use crate::GameState;

use super::{assets::PrefabAsset, unit::UnitComponents};

pub struct AssetTestPlugin;

impl Plugin for AssetTestPlugin {
    fn build(&self, app: &mut App) {
        app
        .insert_resource(ClearColor(Color::rgba_u8(50,50,50,50)))
        .init_resource::<LoadingPrefabs>()
        .init_resource::<AtlasMap>()
        .add_system_set(
            SystemSet::on_enter(GameState::AssetTest).with_system(setup)
        )
        .add_system_set(
            SystemSet::on_update(GameState::AssetTest).with_system(load_window)
        )
        .add_system_set(
            SystemSet::on_update(GameState::AssetTest).with_system(on_loaded)
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

    loading.0.push(asset_server.load("units/guy.prefab"));

}

#[derive(Default)]
struct UiState {
    input: String,
}

fn load_window(
    mut ctx: ResMut<EguiContext>,
    mut input_state: Local<UiState>,
    asset_server: Res<AssetServer>,
    mut loading: ResMut<LoadingPrefabs>, 
    prefabs: Res<Assets<PrefabAsset>>,
) {
    egui::Window::new("Load Prefab").show(ctx.ctx_mut(), |ui| {
        ui.horizontal(|ui| {
            ui.label("Load Prefab:");
            let response = ui.add(
                egui::TextEdit::singleline(&mut input_state.input)
            );
            if response.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
                if let Some(_) = prefabs.get(&input_state.input) {
                    println!("{} is already loaded!", input_state.input);
                    return;
                }
                println!("Attemping to load prefab at {}", input_state.input);
                let path = &input_state.input;
                let handle: Handle<PrefabAsset> = asset_server.load(path);
                loading.0.push(handle);
                input_state.input.clear();
            }
        });
    });
}

fn on_loaded(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut loading: ResMut<LoadingPrefabs>,
    prefabs: Res<Assets<PrefabAsset>>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
    mut ev_loaded: EventReader<AssetEvent<PrefabAsset>>,
    mut q_sprites: Query<Entity, With<TextureAtlasSprite>>,
    mut atlas_map: ResMut<AtlasMap>,
) {
    for ev in ev_loaded.iter() {
        match ev {
            AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
                
                let i = loading.0.iter().position(|h|h==handle).unwrap();
                loading.0.remove(i);
                let pfb = prefabs.get(handle).unwrap();
                let unit: UnitComponents = ron::de::from_str(&pfb.string).unwrap();
                spawn_from_unit_components(&mut commands, &mut atlases, &mut atlas_map, &asset_server, &unit);
            },
            AssetEvent::Removed { handle: _ } => {},
        }
    }
}

#[derive(Default)]
struct AtlasMap {
    map: HashMap<String,Handle<TextureAtlas>>,
}

fn spawn_from_unit_components(
    mut commands: &mut Commands,
    atlases: &mut Assets<TextureAtlas>,
    mut atlas_map: &mut AtlasMap,
    asset_server: &AssetServer,
    unit: &UnitComponents,
) {
    // let map_image_path = &unit.map_sprite.image;
    // let map_image: Handle<Image> = asset_server.load(map_image_path);
    // let atlas = match atlas_map.map.get(map_image_path) {
    //     Some(handle) => handle,
    //     None => {
    //         let atlas = TextureAtlas::from_grid(texture, tile_size, columns, rows)
    //     },
    // };
    // let map_atlas = TextureAtlas {
    //     texture: map_image,
    //     size: todo!(),
    //     textures: todo!(),
    //     texture_handles: todo!(),
    // };

    // // let map_bundle = SpriteSheetBundle {
    // //     sprite: todo!(),
    // //     texture_atlas: todo!(),
    // //     transform: todo!(),
    // // };

    // let arena_image: Handle<Image> = asset_server.load(&unit.arena_sprite.image);
    // let arena_index = unit.arena_sprite.index;
}