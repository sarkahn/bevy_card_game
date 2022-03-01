use bevy::{
    asset::{AssetLoader, AssetPath, BoxedFuture, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    utils::{HashMap, HashSet},
};
use ldtk_rust::{Project, TilesetDefinition, LayerInstance, EntityDefinition};
use serde_json::Value;

use crate::UnitAnimation;

pub struct LdtkAssetPlugin;

impl Plugin for LdtkAssetPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset_loader(LdtkAssetLoader).add_asset::<LdtkMap>();
    }
}

#[derive(TypeUuid, Default, Debug)]
#[uuid = "ac23ab52-5393-4bbe-178f-16c414aaa0eb"]
pub struct LdtkMap {
    pub name: String,
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
    pub entity_defs: MapEntityDefinitions,
}

impl LdtkMap {
    pub fn image(&self, id: i32) -> &Handle<Image> {
        self.images.get(&id).unwrap()
    }

    pub fn tileset(&self, id: i32) -> &MapTileset {
        self.tilesets.get(&id).unwrap()
    }

    pub fn tileset_from_name(&self, name: &str) -> Option<&MapTileset> {
        if let Some(id) = self.id_map.get(&name.to_lowercase()) {
            return self.tilesets.get(&id);
        }
        None
    }

    pub fn image_from_name(&self, name: &str) -> Option<&Handle<Image>> {
        if let Some(id) = self.id_map.get(&name.to_lowercase()) {
            return self.images.get(&id);
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
                let path: AssetPath = path.join(&def.rel_path).into();
                dep_paths.push(path.clone());
                let image: Handle<Image> = load_context.get_handle(path);
                let ts = build_tileset(def, image.clone());
                let tile_size = Vec2::splat(def.tile_grid_size as f32);
                let atlas = TextureAtlas::from_grid(
                    image.clone(),
                    tile_size,
                    def.c_wid as usize,
                    def.c_hei as usize,
                );

                let id = def.uid as i32;

                id_map.insert(def.identifier.to_lowercase(), id);
                tilesets.insert(id, ts);
                images.insert(id, image.clone());
                atlases.insert(id, atlas);
            }

            let mut entity_defs = HashMap::default();
            for def in project.defs.entities.iter() {
                entity_defs.insert(def.uid, def);
            }

            let mut map_layers = Vec::new();
            for level in project.levels.iter() {
                if let Some(layers) = &level.layer_instances {
                    for layer in layers {
                        let name = layer.identifier.clone();
                        match layer.layer_instance_type.as_str() {
                            "IntGrid" => {

                            },
                            "Entities" => {
                                let entities = build_entities(layer, &entity_defs);
                                map_layers.push(MapLayer::Entities(entities));
                            },
                            "Tiles" => {
                                let tiles = build_tiles(layer);
                                map_layers.push(MapLayer::Tiles(tiles));
                            },
                            "AutoLayer" => {
                                let tiles = build_tiles(layer);
                                map_layers.push(MapLayer::Tiles(tiles));
                            },
                            _ => {}
                        }
                    }
                }
            }
            let layers = project.levels[0].layer_instances.as_ref().unwrap();
            let max_width = layers.iter().map(|l| l.c_wid as i32).max().unwrap();
            let max_height = layers.iter().map(|l| l.c_hei as i32).max().unwrap();
            let max_tile_size = layers.iter().map(|l| l.grid_size as i32).max().unwrap();

            let map = LdtkMap {
                name: load_context.path().to_string_lossy().to_string(),
                size: IVec2::new(max_width, max_height),
                layers: map_layers,
                images,
                tilesets,
                id_map,
                tile_size: IVec2::splat(max_tile_size),
                entity_defs: MapEntityDefinitions::from_defs(&entity_defs)
            };

            let asset = LoadedAsset::new(map);
            load_context.set_default_asset(asset.with_dependencies(dep_paths));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["ldtk"]
    }
}

fn build_tiles(layer: &LayerInstance) -> TilesLayer {
    let ts_id = layer.tileset_def_uid.expect("Error loading tile layer, no tileset id");
    let mut map_tiles = Vec::new();

    let layer_height = layer.c_hei;
    let layer_width = layer.c_wid;
    let tile_size = layer.grid_size;
    let pixel_height = layer_height * tile_size;
    let y_flip = pixel_height - tile_size;
    let center_offset = IVec2::new(layer_width as i32, layer_height as i32) / 2;

    for tile in layer.grid_tiles.iter() {
        let [x, y] = [tile.px[0], tile.px[1]];
        let y = y_flip - y;
        let gx = x / tile_size;
        let gy = y / tile_size;

        let id = tile.t as i32;
        let xy = IVec2::new(gx as i32, gy as i32) - center_offset;

        map_tiles.push(MapTile { id, xy });
    }
    TilesLayer {
        tiles: map_tiles,
        tileset_id: ts_id as i32,
        name: layer.identifier.clone(),
    }
}

fn build_tileset(def: &TilesetDefinition, image:Handle<Image>) -> MapTileset {
    let mut tile_data = HashMap::default();
    for data in def.custom_data.iter() {
        let id = data["tileId"].as_ref().unwrap().as_i64().unwrap() as i32;
        let data = data["data"].as_ref().unwrap().as_str().unwrap().to_string();
        tile_data.insert(id, data);
    }

    let mut enums: HashMap<i32, Vec<String>> = HashMap::default();
    for map in def.enum_tags.iter() {
        let enum_name = map.get("enumValueId").unwrap().as_ref().unwrap();
        let enum_name = enum_name.as_str().unwrap();
        let ids = map.get("tileIds").unwrap().as_ref().unwrap();
        let ids = ids.as_array().unwrap();
        for id in ids {
            let entry = enums.entry(id.as_i64().unwrap() as i32);
            let vec = entry.or_insert(Vec::new());
            vec.push(enum_name.to_lowercase());
        }
    }
    //println!("Player ids for tileset {}: {:?}", def.identifier, enums);

    MapTileset {
        tile_count: IVec2::new(def.c_wid as i32, def.c_hei as i32),
        tile_size: def.tile_grid_size as i32,
        tile_data,
        name: def.identifier.clone(),
        image: image,
        enums,
    }
}

fn build_entities(layer: &LayerInstance, defs: &HashMap<i64, &EntityDefinition>) -> EntitiesLayer {
    let layer_height = layer.c_hei;
    let layer_width = layer.c_wid;
    let layer_size = IVec2::new(layer_width as i32, layer_height as i32);

    let mut entity_def_ids = HashSet::default();

    let mut entities = Vec::new();
    for entity in layer.entity_instances.iter() {
        entity_def_ids.insert(entity.def_uid);
        let mut tileset_id = None;
        let mut tile_id = None;
        if let Some(def) = defs.get(&entity.def_uid) {
            if let Some(tid) = def.tile_id {
                tile_id = Some(tid as i32);
            }
            if let Some(tsid) = def.tileset_id {
                tileset_id = Some(tsid as i32);
            }
        }
        let mut fields = HashMap::default();

        for field in entity.field_instances.iter() {
            if let Some(value) = &field.value {
                let name = field.identifier.clone();
                fields.insert(name, value.clone());
            }
        }

        let [x,y] = [entity.grid[0], entity.grid[1]];
        let y = layer_height - 1 - y;
        let xy = IVec2::new(x as i32, y as i32);
        let xy = xy - layer_size / 2;

        entities.push(MapEntity {
            name: entity.identifier.to_string(),
            fields,
            xy,
            def_id: entity.def_uid as i32,
            tile_id,
            tileset_id,
        });
    }

    let mut animations = HashMap::default();
    for(id,def) in defs.iter() {
        animations.insert(*id as i32, anims_from_def(&def));
    }

    EntitiesLayer {
        entities,
        name: layer.identifier.to_string(),
        animations,
    }
}

fn anims_from_def(
    def: &EntityDefinition,
) -> HashMap<String, UnitAnimation> {
    let mut animations = HashMap::default();

    for field in def.field_defs.iter() {
        if field.identifier.to_lowercase() != "animations" {
            continue;
        }
        //println!("Attempting to load animations for {}", def.identifier);
        if let Some(content) = &field.default_override {
            match content {
                Value::Object(o) => {
                    let content = o.get("params").expect("Error loading animations, unexpected format");
                    match content {
                        Value::Array(arr) => {
                            let value = arr[0].as_str().unwrap();
                            //println!("Value {}", value);
                            animations = ron::de::from_str(value).unwrap();

                        },
                        _ => { panic!("Error loading animations array for {}, unexpected format: {:#?}", def.uid, content) }
                    }
                },
                _ => { panic!("Error loading animations for {}, unexpected format {:#?}", def.uid, content) }
            }
        }
    }
    animations
}

#[derive(Debug, Default)]
pub struct MapTile {
    pub id: i32,
    pub xy: IVec2,
}

#[derive(Debug,Default)]
pub struct MapEntityDefinitions {
    defs: HashMap<i32, MapEntityDef>,
    name_map: HashMap<String,i32>,
}
impl MapEntityDefinitions {
    pub fn def_from_id(&self, id: i32) -> Option<&MapEntityDef> {
        self.defs.get(&id)
    }
    pub fn def_from_name(&self, name: &str) -> Option<&MapEntityDef> {
        if let Some(id) = self.name_map.get(name) {
            return self.def_from_id(*id);
        }
        None
    }
    pub fn from_defs(ldtk_defs: &HashMap<i64,&EntityDefinition>) -> Self {
        let mut defs = HashMap::default();
        let mut name_map = HashMap::default();

        for (id, def) in ldtk_defs {
            name_map.insert(def.identifier.to_lowercase(), *id as i32);
            let def = MapEntityDef::from_ldtk_def(def);
            defs.insert(*id as i32,def);
        }
        
        Self {
            defs,
            name_map,
        }
    }
}

#[derive(Debug, Default)]
pub struct MapEntityDef {
    name: String,
    pub tile_id: Option<i32>,
    pub tileset_id: Option<i32>,
    pub animations: HashMap<String,UnitAnimation>,
}
impl MapEntityDef {
    pub fn from_ldtk_def(def: &EntityDefinition) -> Self {
        let animations = anims_from_def(def);
        Self {
            name: def.identifier.to_lowercase(),
            tile_id: def.tile_id.map(|id|id as i32),
            tileset_id: def.tileset_id.map(|id|id as i32),
            animations,
        }
    }
}

#[derive(Debug, Default)]
pub struct MapTileset {
    // Maps tile ids to their custom data
    pub tile_data: HashMap<i32, String>,
    pub tile_size: i32,
    pub tile_count: IVec2,
    pub name: String,
    pub image: Handle<Image>,
    pub enums: HashMap<i32, Vec<String>>,
}

#[derive(Debug)]
pub enum MapLayer {
    Tiles(TilesLayer),
    Entities(EntitiesLayer,)
}

#[derive(Debug, Default)]
pub struct TilesLayer {
    pub tiles: Vec<MapTile>,
    pub tileset_id: i32,
    pub name: String,
}


#[derive(Default, Debug)]
pub struct MapEntity {
    pub name: String,
    pub fields: HashMap<String,Value>,
    pub xy: IVec2,
    pub def_id: i32,
    pub tile_id: Option<i32>,
    pub tileset_id: Option<i32>,
}

#[derive(Default,Debug)]
pub struct EntitiesLayer {
    pub entities: Vec<MapEntity>,
    pub name: String,
    /// Map of entity uid to animations mapping. Each entity type
    /// has it's own set of animations.
    pub animations: HashMap<i32, HashMap<String,UnitAnimation>>,
}
