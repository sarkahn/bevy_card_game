use bevy::{
    asset::{AssetLoader, AssetPath, BoxedFuture, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    utils::HashMap,
};
use ldtk_rust::{Project, TilesetDefinition};

//use super::map_builder::{MapTileset, MapLayer, MapTile};

pub(crate) struct LdtkAssetPlugin;

impl Plugin for LdtkAssetPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset_loader(LdtkAssetLoader)
            .add_asset::<LdtkMap>();
    }
}



#[derive(TypeUuid, Default, Debug)]
#[uuid = "ac23ab52-5393-4bbe-178f-16c414aaa0eb"]
pub struct LdtkMap {
    pub size: IVec2,
    pub layers: Vec<MapLayer>,
    // Maps tileset id to it's image handle
    pub images: HashMap<i32, Handle<Image>>,
    // Maps tileset id to data
    pub tilesets: HashMap<i32, MapTileset>,
    // Maps tileset name to it's id
    id_map: HashMap<String, i32>,
    //pub atlases: HashMap<i32, Handle<TextureAtlas>>,
    pub tile_size: IVec2,
}

impl LdtkMap {
    pub fn image(&self, layer: &MapLayer) -> &Handle<Image> {
        self.images.get(&layer.tileset_id).unwrap()
    }
    
    pub fn tileset(&self, layer: &MapLayer) -> &MapTileset {
        self.tilesets.get(&layer.tileset_id).unwrap()
    }

    pub fn tileset_from_name(&self, name: &str) -> Option<&MapTileset> {
        if let Some(id) = self.id_map.get(&name.to_lowercase()) {
            return self.tilesets.get(&id)
        }
        None
    }
    pub fn image_from_name(&self, name: &str) -> Option<&Handle<Image>> {
        if let Some(id) = self.id_map.get(&name.to_lowercase()) {
            return self.images.get(&id)
        }
        None
    }

}


#[derive(Copy, Clone, Debug, Default)]
struct LdtkAssetLoader;

impl AssetLoader for LdtkAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let project: Project = serde_json::from_slice(bytes)?;

            let mut tilesets = HashMap::default();
            let mut images = HashMap::default();
            let mut atlases = HashMap::default();
            let mut id_map = HashMap::default();

            let path = load_context.path().parent().unwrap();
            let mut dep_paths = Vec::new(); 

            for def in project.defs.tilesets.iter() {
                let path:AssetPath = path.join(&def.rel_path).into();
                dep_paths.push(path.clone());
                let image: Handle<Image> = load_context.get_handle(path);
                let ts = build_tileset(def);
                let tile_size = Vec2::splat(def.tile_grid_size as f32);
                let atlas = TextureAtlas::from_grid(
                    image.clone(), tile_size, def.c_wid as usize, def.c_hei as usize
                );

                let id = def.uid as i32;

                id_map.insert(def.identifier.to_lowercase(), id);
                tilesets.insert(id, ts);
                images.insert(id, image.clone());
                atlases.insert(id, atlas);
            }

            let mut map_layers = Vec::new();
            for level in project.levels.iter() {
                if let Some(layers) = &level.layer_instances {
                    for layer in layers {
                        let ts_id = layer.tileset_def_uid.unwrap() as i32;

                        let layer_height = layer.c_hei;
                        let layer_width = layer.c_wid;
                        let tile_size = layer.grid_size;
                        let pixel_height = layer_height * tile_size;
                        let y_flip = pixel_height - tile_size;
                        let center_offset = IVec2::new(layer_width as i32, layer_height as i32) / 2;

                        let mut map_tiles = Vec::new();
                        for tile in layer.grid_tiles.iter() {

                            let [x, y] = [tile.px[0], tile.px[1]];
                            let y = y_flip - y;
                            let gx = x / tile_size;
                            let gy = y / tile_size;
                    
                            let id = tile.t as i32;
                            let xy = IVec2::new(gx as i32, gy as i32) - center_offset;

                            map_tiles.push(MapTile { id, xy, });
                        }
                        map_layers.push(MapLayer {
                            tiles: map_tiles,
                            tileset_id: ts_id,
                            name: layer.identifier.clone(),
                        });
                    }
                }
            }
            let layers = project.levels[0].layer_instances.as_ref().unwrap();
            let max_width = layers.iter().map(|l| l.c_wid as i32).max().unwrap();
            let max_height = layers.iter().map(|l| l.c_hei as i32).max().unwrap();
            let max_tile_size = layers.iter().map(|l|l.grid_size as i32).max().unwrap();

            let map = LdtkMap {
                size: IVec2::new(max_width, max_height),
                layers: map_layers,
                images,
                tilesets,
                id_map,
                tile_size: IVec2::splat(max_tile_size),
            };

            let asset = LoadedAsset::new(map);
            load_context.set_default_asset(
                asset.with_dependencies(dep_paths)
            );
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["ldtk"]
    }
}

fn build_tileset(def: &TilesetDefinition) -> MapTileset {
    let mut tile_data = HashMap::default();
    for data in def.custom_data.iter() {
        let id = data["tileId"].as_ref().unwrap().as_i64().unwrap() as i32;
        let data = data["data"].as_ref().unwrap().as_str().unwrap().to_string();
        tile_data.insert(id,data);
    }

    MapTileset {
        tile_count: IVec2::new(def.c_wid as i32, def.c_hei as i32),
        tile_size: def.tile_grid_size as i32,
        tile_data,
        name: def.identifier.clone()
    }
}


#[derive(Debug, Default)]
pub struct MapTile {
    pub id: i32,
    pub xy: IVec2,
}

#[derive(Debug, Default)]
pub struct MapTileset {
    // Maps tile ids to their custom data
    pub tile_data: HashMap<i32, String>,
    pub tile_size: i32,
    pub tile_count: IVec2,
    pub name: String,
}

#[derive(Debug, Default)]
pub struct MapLayer {
    pub tiles: Vec<MapTile>,
    pub tileset_id: i32,
    pub name: String,
}
// impl LdtkMap {
//     pub fn get_tileset(&self, name: &str) -> Option<&MapTileset> {
//         if let Some(id) = self.tileset_name_map.get(name) {
//             return self.tilesets.get(id);
//         }  
//         None
//     }
// }