use bevy::{prelude::*, utils::HashMap};
use ldtk_rust::TilesetDefinition;

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
pub struct LdtkMapBuilt(pub LdtkMap);

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
pub struct MapTileset {
    // Maps tile ids to their custom data
    pub tile_data: HashMap<i32, String>,
    pub tile_size: i32,
    pub tile_count: IVec2,
}

impl From<&TilesetDefinition> for MapTileset {
    fn from(def: &TilesetDefinition) -> Self {
        let mut tile_data = HashMap::default();
        for data in def.custom_data.iter() {
            let id = data["tileId"].as_ref().unwrap().as_i64().unwrap() as i32;
            let data = data["data"].as_ref().unwrap().as_str().unwrap().to_string();
            tile_data.insert(id,data);
        }

        Self {
            tile_count: IVec2::new(def.c_wid as i32, def.c_hei as i32),
            tile_size: def.tile_grid_size as i32,
            tile_data
        }
    }
}

#[derive(Default)]
pub struct MapLayer {
    pub tiles: Vec<MapTile>,
    pub image: Handle<Image>,
    pub atlas: Handle<TextureAtlas>,
    pub tileset_id: i32,
    pub name: String,
}

#[derive(Default)]
pub struct LdtkMap {
    pub size: IVec2,
    pub layers: Vec<MapLayer>,
    // Map of bevy textures corresponding to their tileset
    pub images: HashMap<i32, Handle<Image>>,
    // Map of tileset data
    pub tilesets: HashMap<i32, MapTileset>,
    pub tile_size: IVec2,
}

fn load_from_asset(asset: &LdtkAsset, atlases: &mut ResMut<Assets<TextureAtlas>>) -> Option<LdtkMap> {
    let p = &asset.project;

    let mut map = None;

    for level in p.levels.iter() {
        let map = map.get_or_insert(LdtkMap::default());

        for def in p.defs.tilesets.iter() {
            map.tilesets.insert(def.uid as i32, def.into());
        }

        let images = &mut map.images;
        for (id,handle) in asset.tilesets.iter() {
            images.insert(*id as i32, handle.clone());
        }
        if let Some(layers) = &level.layer_instances {
            let max_width = layers.iter().map(|l| l.c_wid as i32).max().unwrap();
            let max_height = layers.iter().map(|l| l.c_hei as i32).max().unwrap();
            let max_tile_size = layers.iter().map(|l|l.grid_size as i32).max().unwrap();

            map.size = IVec2::new(max_width, max_height);
            map.tile_size = IVec2::splat(max_tile_size);
            for layer in layers.iter() {
                let mut map_layer = MapLayer::default();
                map_layer.name = layer.identifier.clone();

                if let Some(id) = layer.tileset_def_uid {
                    map_layer.tileset_id = id as i32;

                    if let Some(image) = asset.tilesets.get(&id) {
                        if let Some(tileset) = map.tilesets.get(&(id as i32)) {
                            map_layer.image = image.clone();
                            let tile_size = layer.grid_size as f32;
                            let atlas = TextureAtlas::from_grid(
                                image.clone(),
                                Vec2::splat(tile_size),
                                tileset.tile_count.x as usize, tileset.tile_count.y as usize);
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
