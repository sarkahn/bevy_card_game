use bevy::{prelude::*, utils::HashMap};

use super::{asset::LdtkAsset, LoadingMaps};

pub(crate) struct LdtkMapBuilderPlugin;

impl Plugin for LdtkMapBuilderPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system(build_from_ldtk)
            .add_event::<LdtkMapBuilt>();
    }
}

#[derive(Default)]
pub struct LdtkMapBuilt(pub Map);

fn build_from_ldtk(
    ldtk_assets: Res<Assets<LdtkAsset>>,
    mut loading_maps: ResMut<LoadingMaps>,
    mut ev_ldtk: EventReader<AssetEvent<LdtkAsset>>,
    mut ev_map_built: EventWriter<LdtkMapBuilt>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
) {
    for ev in ev_ldtk.iter() {
        match ev {
            AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
                let i = loading_maps.0.iter().position(|x| x == handle).unwrap();
                loading_maps.0.remove(i);

                if let Some(asset) = ldtk_assets.get(handle) {
                    if let Some(map) = load_from_asset(asset, &mut atlases) {
                        ev_map_built.send(LdtkMapBuilt(map));
                    }
                }
            }
            _ => {}
        }
    }
}

#[derive(Default)]
pub struct MapTile {
    pub id: i32,
    pub xy: IVec2,
}

#[derive(Default)]
pub struct MapLayer {
    pub tiles: Vec<MapTile>,
    pub tileset: Handle<Image>,
    pub atlas: Handle<TextureAtlas>,
}

#[derive(Default)]
pub struct Map {
    pub size: IVec2,
    pub layers: Vec<MapLayer>,
    pub tilesets: HashMap<i32, Handle<Image>>,
    pub tile_size: IVec2,
}

fn load_from_asset(asset: &LdtkAsset, atlases: &mut ResMut<Assets<TextureAtlas>>) -> Option<Map> {
    let p = &asset.project;

    let mut map = None;

    for level in p.levels.iter() {
        let map = map.get_or_insert(Map::default());
        let tilesets = &mut map.tilesets;
        for (id,handle) in asset.tilesets.iter() {
            tilesets.insert(*id as i32, handle.clone());
        }
        if let Some(layers) = &level.layer_instances {
            let max_width = layers.iter().map(|l| l.c_wid as i32).max().unwrap();
            let max_height = layers.iter().map(|l| l.c_hei as i32).max().unwrap();
            let max_tile_size = layers.iter().map(|l|l.grid_size as i32).max().unwrap();

            map.size = IVec2::new(max_width, max_height);
            map.tile_size = IVec2::splat(max_tile_size);
            for layer in layers.iter() {
                let mut map_layer = MapLayer::default();

                if let Some(id) = layer.tileset_def_uid {
                    if let Some(handle) = asset.tilesets.get(&id) {
                        if let Some(tsdef) = p.defs.tilesets.iter().find(|t|t.uid == id) {
                            map_layer.tileset = handle.clone();
                            let tile_size = layer.grid_size as f32;
    
                            let ts_width = tsdef.c_wid as usize;
                            let ts_height = tsdef.c_hei as usize;
                            let atlas = TextureAtlas::from_grid(
                                handle.clone(),
                                Vec2::splat(tile_size),
                                ts_width, ts_height);
                            map_layer.atlas = atlases.add(atlas);
                        }
                    }
                }

                let layer_height = layer.c_hei;
                let grid_size = layer.grid_size;
                let pixel_height = layer_height * grid_size;

                for tile in layer.grid_tiles.iter() {
                    let [x, y] = [tile.px[0], tile.px[1]];
                    let y = (pixel_height - grid_size) - y;
                    let gx = x / grid_size;
                    let gy = y / grid_size;

                    let id = tile.t as i32;
                    let xy = IVec2::new(gx as i32, gy as i32) - map.size / 2;
                    map_layer.tiles.push(MapTile { id, xy });
                }
                for tile in layer.auto_layer_tiles.iter() {
                    let [x, y] = [tile.px[0], tile.px[1]];
                    let y = (pixel_height - grid_size) - y;
                    let gx = x / grid_size;
                    let gy = y / grid_size;

                    let id = tile.t as i32;
                    let xy = IVec2::new(gx as i32, gy as i32) - map.size / 2;
                    map_layer.tiles.push(MapTile { id, xy });
                }
                map.layers.push(map_layer);
            }
        }
    }

    map
}
