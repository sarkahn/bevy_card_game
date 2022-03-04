use core::f32;
use std::collections::BTreeMap;

use bevy::{
    asset::{AssetLoader, AssetPath, BoxedFuture, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    utils::{HashMap, HashSet},
};
use ldtk_rust::{EntityDefinition, FieldInstance, LayerInstance, Project, TilesetDefinition, FieldDefinition, EntityInstance};
use serde::{Serialize, Deserialize};
use serde_json::Value;

use crate::AnimationData;


pub struct LdtkAssetPlugin;

impl Plugin for LdtkAssetPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset_loader(LdtkAssetLoader).add_asset::<LdtkMap>();
    }
}

#[derive(TypeUuid, Default, Debug)]
#[uuid = "ac23ab52-5393-4bbe-178f-16c414aaa0eb"]
pub struct LdtkMap {
    name: String,
    size_px: IVec2,
    tile_count: Option<IVec2>,
    pixels_per_tile: Option<i32>,
    layers: BTreeMap<String,MapLayer>,
    // Maps tileset id to it's image handle
    images: HashMap<i32, Handle<Image>>,
    // Maps tileset id to data
    tilesets: HashMap<i32, MapTileset>,
    // Maps depths to backgrounds
    background: Option<MapBackground>,
    // Maps tileset name to it's id
    name_map: HashMap<String, i32>,
    // Map tileset path to it's id
    path_map: HashMap<String,i32>,
    //pub atlases: HashMap<i32, Handle<TextureAtlas>>,
    max_tile_size: IVec2,
    entity_defs: MapEntityDefinitions,
    custom_fields: Fields,
}

impl LdtkMap {
    pub fn image_from_id(&self, id: i32) -> Option<&Handle<Image>> {
        self.images.get(&id)
    }

    pub fn tileset_from_id(&self, id: i32) -> Option<&MapTileset> {
        self.tilesets.get(&id)
    }

    pub fn tileset_from_name(&self, name: &str) -> Option<&MapTileset> {
        if let Some(id) = self.name_map.get(&name.to_lowercase()) {
            return self.tilesets.get(&id);
        }
        None
    }
    pub fn tileset_from_path(&self, name: &str) -> Option<&MapTileset> {
        if let Some(id) = self.path_map.get(&name.to_lowercase()) {
            return self.tilesets.get(&id);
        }
        None
    }

    pub fn image_from_name(&self, name: &str) -> Option<&Handle<Image>> {
        if let Some(id) = self.name_map.get(&name.to_lowercase()) {
            return self.images.get(&id);
        }
        None
    }

    pub fn layers(&self) -> impl DoubleEndedIterator<Item=&MapLayer> {
        self.layers.iter().map(|(_,b)|b)
    }

    pub fn layer_from_name(&self, name: &str) -> Option<&MapLayer> {
        self.layers.get(&name.to_lowercase())
    }
    pub fn entity_defs(&self) -> &MapEntityDefinitions {
        &self.entity_defs
    }

    /// Get a reference to the ldtk map's name.
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Get the ldtk map's size.
    pub fn size_px(&self) -> IVec2 {
        self.size_px
    }
    pub fn tile_count(&self) -> Option<IVec2> {
        self.tile_count
    }

    pub fn pixels_per_tile(&self) -> Option<i32> {
        self.pixels_per_tile
    }

    /// Get a reference to the ldtk map's background.
    pub fn background(&self) -> Option<&MapBackground> {
        self.background.as_ref()
    }

    pub fn tilesets(&self) -> impl Iterator<Item=&MapTileset> {
        self.tilesets.iter().map(|(_,t)|t)
    }
    // pub fn has_enum(&self, name: &str, tileset_id: i32, tile_id: i32) -> bool {
    //     if let Some(tileset) = self.tileset_from_id(tileset_id) {
    //         if let Some(v) = tileset.enums.get(&tile_id) {
    //             return v.contains(&name.to_string());
    //         }
    //     }
    //     false
    // } 
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
            let mut path_map = HashMap::default();

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

                let path = def.rel_path.to_lowercase().replace("\"", "");
                //println!("Adding {} to path map, id {}", path, id);
                path_map.insert(path, id);
            }

            let mut bg = None;
            for level in project.levels.iter() {
                if let Some(bg_path) = &level.bg_rel_path {
                    let path: AssetPath = path.join(&bg_path).into();
                    println!("Loading {}", path.path().to_string_lossy());
                    dep_paths.push(path.clone());
                    let image: Handle<Image> = load_context.get_handle(bg_path);
                    // TODO : Should derive size of tile
                    let w = level.px_wid;
                    let h = level.px_hei;
                    let size = IVec2::new(w as i32, h as i32);
                    bg = Some(MapBackground {
                        image,
                        size
                    });
                    println!("Background = yes: {}", path.path().to_string_lossy());
                }
            }

            let mut entity_defs = HashMap::default();
            for def in project.defs.entities.iter() {
                entity_defs.insert(def.uid, def);
            }
            let mut map_layers = BTreeMap::new();
            let mut fields = Fields::default();
            for level in project.levels.iter() {
                fields = Fields::from_ldtk(&level.field_instances);

                if let Some(layers) = &level.layer_instances {
                    for layer in layers {
                        let name = layer.identifier.clone();
                        let tileset = match layer.tileset_def_uid {
                            Some(id) => tilesets.get(&(id as i32)),
                            None => None,
                        };
                        //let tiles = build_tiles(layer, tileset);
                        match layer.layer_instance_type.as_str() {
                            "IntGrid" => {}
                            "Entities" => {
                                let entities = entities_from_defs(layer, &entity_defs);
                                map_layers.insert(layer.identifier.to_lowercase(), MapLayer::Entities(entities));
                                //map_layers.push(MapLayer::Entities(entities));
                            }
                            "Tiles" => {
                                let tiles = build_tiles(layer, tileset);
                                map_layers.insert(layer.identifier.to_lowercase(), MapLayer::Tiles(tiles));
                                //map_layers.push(MapLayer::Tiles(tiles));
                            }
                            "AutoLayer" => {
                                let tiles = build_tiles(layer, tileset);
                                map_layers.insert(layer.identifier.to_lowercase(), MapLayer::Tiles(tiles));
                                //map_layers.push(MapLayer::Tiles(tiles));
                            }
                            _ => {}
                        }
                    }
                }
            }
            let layers = project.levels[0].layer_instances.as_ref().unwrap();
            let max_width = layers.iter().map(|l| l.c_wid as i32).max().unwrap();
            let max_height = layers.iter().map(|l| l.c_hei as i32).max().unwrap();
            let max_tile_size = layers.iter().map(|l| l.grid_size as i32).max().unwrap();

            let tile_count = fields.try_get_ivec2("tile_count");

            let pixels_per_tile = fields.try_get_i32("pixels_per_tile");

            let map = LdtkMap {
                name: load_context.path().to_string_lossy().to_string(),
                size_px: IVec2::new(max_width, max_height),
                tile_count,
                pixels_per_tile,
                layers: map_layers,
                images,
                tilesets,
                name_map: id_map,
                path_map,
                background: bg,
                max_tile_size: IVec2::splat(max_tile_size),
                entity_defs: MapEntityDefinitions::from_defs(&entity_defs),
                custom_fields: fields,
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


fn build_tiles(layer: &LayerInstance, tileset: Option<&MapTileset>) -> TilesLayer {
    let ts_id = layer
        .tileset_def_uid
        .expect("Error loading tile layer, no tileset id");
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

        map_tiles.push(MapTile { id, grid_xy: xy });
    }

    let enums = match tileset {
        Some(ts) => ts.enums.clone(),
        None => None,
    };

    TilesLayer {
        tiles: map_tiles,
        tileset_id: ts_id as i32,
        name: layer.identifier.clone(),
        enums,
    }
}

fn build_tileset(def: &TilesetDefinition, image: Handle<Image>) -> MapTileset {
    let mut tile_data = HashMap::default();
    for data in def.custom_data.iter() {
        let id = data["tileId"].as_ref().unwrap().as_i64().unwrap() as i32;
        let data = data["data"].as_ref().unwrap().as_str().unwrap().to_string();
        tile_data.insert(id, data);
    }

    let mut enums = HashMap::default();
    for map in def.enum_tags.iter() {
        let enum_name = map.get("enumValueId").unwrap().as_ref().unwrap();
        //println!("Enum name {}", enum_name);
        let enum_name = enum_name.as_str().unwrap();
        let ids = map.get("tileIds").unwrap().as_ref().unwrap();
        let ids = ids.as_array().unwrap();
        let ids: Vec<_> = ids.iter().map(|id|id.as_i32().unwrap()).collect();
        enums.insert(enum_name.to_lowercase(), ids);
    }
    //println!("Player ids for tileset {}: {:?}", def.identifier, enums);

    MapTileset {
        tile_count: IVec2::new(def.c_wid as i32, def.c_hei as i32),
        tile_size: def.tile_grid_size as i32,
        tile_data,
        name: def.identifier.to_lowercase(),
        image: image,
        path: def.rel_path.to_lowercase(),
        enums: match enums.is_empty() {
            true => None,
            false => Some(enums),
        },
    }
}



#[derive(Debug, Default)]
pub struct MapTile {
    pub id: i32,
    pub grid_xy: IVec2,
}

#[derive(Default, Debug)]
pub struct MapEntityFields {
    pub map: HashMap<String, Value>,
}

#[derive(Default, Debug)]
pub struct MapBackground {
    pub image: Handle<Image>,
    pub size: IVec2,
}

impl MapEntityFields {
    pub fn from_ldtk_instances(fields: &Vec<FieldInstance>) -> Self {
        let mut map = HashMap::default();
        for field in fields.iter() {
            let name = field.identifier.to_lowercase();
            if let Some(value) = &field.value {
                map.insert(name, value.clone());
            }
        }

        Self { map }
    }
    
    pub fn from_ldtk_defs(fields: &Vec<FieldDefinition>) -> Self {
        let mut map = HashMap::default();
        for field in fields.iter() {
            if let Some(value) = &field.default_override {
                map.insert(
                    field.identifier.to_lowercase(),
                    value.clone()
                );
            }
        }
        Self {map}
    }
}

#[derive(Debug, Default)]
pub struct MapEntityDefinitions {
    defs: HashMap<i32, MapEntityDef>,
    name_map: HashMap<String, i32>,
}
impl MapEntityDefinitions {
    pub fn def_from_id(&self, id: i32) -> Option<&MapEntityDef> {
        self.defs.get(&id)
    }
    pub fn def_from_name(&self, name: &str) -> Option<&MapEntityDef> {
        if let Some(id) = self.name_map.get(&name.to_lowercase()) {
            return self.def_from_id(*id);
        }
        None
    }
    fn from_defs(ldtk_defs: &HashMap<i64, &EntityDefinition>) -> Self {
        let mut defs = HashMap::default();
        let mut name_map = HashMap::default();

        for (id, def) in ldtk_defs {
            //println!("Inserting {}:{} into defs", id, def.identifier);
            name_map.insert(def.identifier.to_lowercase(), *id as i32);
            let def = MapEntityDef::from_ldtk_def(def);
            defs.insert(*id as i32, def);
        }

        Self { defs, name_map }
    }
    fn from_instances(entities: &Vec<EntityInstance>) {
        for e in entities.iter() {
            println!("{} fields:", e.identifier);
            for field in e.field_instances.iter() {
                    println!("Field {}: {:?}", field.identifier, field.value);
            }
        }
    }
    pub fn all_from_name<'a>(&'a self, name: &'a str) -> impl Iterator<Item=&'a MapEntityDef> {
        self.defs.iter().map(|(_,d)| d).filter(move |d|d.name==name)
    }

    pub fn get_tagged<'a>(&'a self, tag: &'a str) -> impl Iterator<Item=&'a MapEntityDef> {
        let tag = tag.to_lowercase();
        self.defs.iter().map(|(_,d)| d).filter(move |d|d.tags.contains(&tag))
    }
}

#[derive(Debug, Default)]
pub struct MapEntityInstances {
    defs: HashMap<i32, MapEntity>,
    name_map: HashMap<String, i32>,
}
impl MapEntityInstances {
    pub fn def_from_id(&self, id: i32) -> Option<&MapEntity> {
        self.defs.get(&id)
    }
    pub fn def_from_name(&self, name: &str) -> Option<&MapEntity> {
        if let Some(id) = self.name_map.get(&name.to_lowercase()) {
            return self.def_from_id(*id);
        }
        None
    }
    pub fn all_from_name<'a>(&'a self, name: &'a str) -> impl Iterator<Item=&'a MapEntity> {
        self.defs.iter().map(|(_,d)| d).filter(move |d|d.name==name)
    }
}

#[derive(Debug, Default)]
pub struct MapEntityDef {
    name: String,
    fields: HashMap<String, Value>,
    size: IVec2,
    def_id: i32,
    tile_id: Option<i32>,
    tileset_id: Option<i32>,
    tags: Vec<String>,
}
impl MapEntityDef {
    pub fn from_ldtk_def(def: &EntityDefinition) -> Self {
        let name = def.identifier.to_lowercase();
        let fields = fields_from_defs(&def.field_defs);
        let [width,height] = [def.width as i32, def.height as i32];
        let size = IVec2::new(width, height);
        let def_id = def.uid as i32;
        let tileset_id = def.tileset_id.map(|v|v as i32);
        let tile_id = def.tile_id.map(|v|v as i32);
        let tags = def.tags.clone();

        Self {
            name,
            fields,
            size,
            def_id,
            tile_id,
            tileset_id,
            tags
        }
    }

    /// Get a reference to the map entity def's name.
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Get a reference to the map entity def's fields.
    pub fn fields(&self) -> &HashMap<String, Value> {
        &self.fields
    }

    pub fn size(&self) -> IVec2 {
        self.size
    }

    /// Get the map entity def's def id.
    pub fn def_id(&self) -> i32 {
        self.def_id
    }

    /// Get the map entity def's tile id.
    pub fn tile_id(&self) -> Option<i32> {
        self.tile_id
    }

    /// Get the map entity def's tileset id.
    pub fn tileset_id(&self) -> Option<i32> {
        self.tileset_id
    }

}

fn fields_from_defs(fields: &Vec<FieldDefinition>) -> HashMap<String,Value> {
    let mut map = HashMap::default();
    for field in fields.iter() {
        if let Some(value) = &field.default_override {
            map.insert(
                field.identifier.to_lowercase(),
                value.clone()
            );
        }
    }
    map
}

#[derive(Debug, Default)]
pub struct MapTileset {
    // Maps tile ids to their custom data
    pub tile_data: HashMap<i32, String>,
    pub tile_size: i32,
    pub tile_count: IVec2,
    pub name: String,
    pub image: Handle<Image>,
    pub path: String,
    enums: Option<HashMap<String,Vec<i32>>>,
}
impl MapTileset {
    pub fn tile_id_has_enum(&self, tile_id: i32, name: &str) -> bool {
        if let Some(enums) = &self.enums {
            if let Some(values) = enums.get(name) {
                return values.contains(&tile_id);
            }
        }
        false
    }
}

#[derive(Debug)]
pub enum MapLayer {
    Tiles(TilesLayer),
    Entities(EntitiesLayer),
}
impl MapLayer {
    pub fn as_tiles(&self) -> Option<&TilesLayer> {
        match self {
            MapLayer::Tiles(t) => Some(t),
            MapLayer::Entities(_) => None,
        }
    }
    pub fn as_entities(&self) -> Option<&EntitiesLayer> {
        match self {
            MapLayer::Tiles(_) => None,
            MapLayer::Entities(e) => Some(e),
        }
    }

    pub fn is_entities(&self) -> bool {
        self.as_entities().is_some()
    }
    
    pub fn is_tiles(&self) -> bool {
        self.as_tiles().is_some()
    }
}

#[derive(Debug, Default)]
pub struct TilesLayer {
    pub tiles: Vec<MapTile>,
    pub tileset_id: i32,
    pub name: String,
    enums: Option<HashMap<String,Vec<i32>>>,
}
impl TilesLayer {
    pub fn has_enum(&self, tile_id: i32, enum_name: &str) -> bool {
        if let Some(enums) = &self.enums {
            if let Some(values) = enums.get(enum_name) {
                return values.contains(&tile_id);
            }
        }
        false
    }
}

#[derive(Debug, Default)]
pub struct Fields {
    fields: HashMap<String,Value>,
}
impl Fields {
    pub fn get_i32(&self, field_name: &str) -> i32 {
        if let Some(val) = self.fields.get(field_name) {
            if let Some(val) = val.as_i64() {
                return val as i32;
            }
        }
        panic!("Filed to find i32 field {}", field_name);
    }
    
    pub fn try_get_i32(&self, field_name: &str) -> Option<i32> {
        if let Some(val) = self.fields.get(field_name) {
            if let Some(val) = val.as_i64() {
                return Some(val as i32);
            }
        }
        None
    }

    pub fn get_str(&self, field_name: &str) -> &str {
        if let Some(val) = self.fields.get(field_name) {
            if let Some(val) = val.as_str() {
                return val
            }
        }
        panic!("Filed to find i32 field {}", field_name);
    }
    
    
    pub fn get_f32(&self, field_name: &str) -> f32 {
        if let Some(val) = self.fields.get(field_name) {
            //println!("VAL TYPE {:?}", val);
            if let Some(val) = val.as_object() {
                //println!("Obj");
                if let Some(val) = val.get("params") {
                    if let Some(val) = val.as_array() {
                        //println!("ARRAY {:?}", val);
                        match &val[0] {
                            Value::Number(n) => {
                                return n.as_f64().unwrap() as f32;
                            },
                            _ => {}
                        }
                    }
                }
            }
        }
        panic!("Filed to find f32 field {}", field_name);
    }

    pub fn try_get_f32(&self, field_name: &str) -> f32 {
        if let Some(val) = self.fields.get(field_name) {
            //println!("VAL TYPE {:?}", val);
            if let Some(val) = val.as_object() {
                //println!("Obj");
                if let Some(val) = val.get("params") {
                    if let Some(val) = val.as_array() {
                        //println!("ARRAY {:?}", val);
                        match &val[0] {
                            Value::Number(n) => {
                                return n.as_f64().unwrap() as f32;
                            },
                            _ => {}
                        }
                    }
                }
            }
        }
        panic!("Filed to find f32 field {}", field_name);
    }
    
    pub fn get_vec2(&self, field_name: &str) -> Vec2 {
        if let Some(val) = self.fields.get(field_name) {
            if let Some(val) = val.as_array() {
                if let Some(x) = val[0].as_f32() {
                    if let Some(y) = val[1].as_f32() {
                        return Vec2::new(x,y);
                    }
                }
            }
        }
        panic!("Filed to find vec2 field {}", field_name);
    }
    
    pub fn get_ivec2(&self, field_name: &str) -> IVec2 {
        if let Some(val) = self.fields.get(field_name) {
            if let Some(val) = val.as_array() {
                if let Some(x) = val[0].as_i32() {
                    if let Some(y) = val[1].as_i32() {
                        return IVec2::new(x,y);
                    }
                }
            }
        }
        panic!("Filed to find ivec2 field {}", field_name);
    }
    
    pub fn try_get_ivec2(&self, field_name: &str) -> Option<IVec2> {
        if let Some(val) = self.try_get_array::<i32>(field_name) {
            if val.len() == 2 {
                return Some(IVec2::new(val[0], val[1]));
            }
        }
        None
    }

    
    pub fn try_get_array<'a, T: Serialize + Deserialize<'a> + Clone>(&'a self, field_name: &str) -> Option<Vec<T>> {
        if let Some(val) = self.fields.get(field_name) {
            if let Some(val) = val.as_str() {
                if let Ok(v) =  ron::de::from_str::<Vec<T>>(val) {
                    return Some(v.clone());
                }
            }
        }
        None
    }

    pub fn get_array<'a, T: Serialize + Deserialize<'a> + Clone>(&'a self, field_name: &str) -> Vec<T> {
        if let Some(val) = self.fields.get(field_name) {
            if let Some(val) = val.as_str() {
                if let Ok(v) =  ron::de::from_str::<Vec<T>>(val) {
                    return v.clone();
                }
            }
        }
        panic!("Filed to find vec2 field {}", field_name);
    }

    pub fn field(&self, name: &str) -> Option<&Value> {
        self.fields.get(name)
    }

    pub fn from_ldtk(ldtl_fields: &Vec<FieldInstance>) -> Fields {
        let mut fields = HashMap::default();
        for field in ldtl_fields.iter() {
            if let Some(value) = &field.value {
                fields.insert(field.identifier.to_lowercase(), value.clone());
            }
        }
        Self {
            fields,
        }
    }
}

#[derive(Debug)]
pub struct MapEntity {
    name: String,
    fields: Fields,
    xy: IVec2,
    grid_xy: IVec2,
    size: IVec2,
    def_id: i32,
    tile_id: Option<i32>,
    tileset_id: Option<i32>,
    pivot: Vec2,
    tags: Vec<String>,
    pixels_per_unit: i32,
}
impl MapEntity {
    /// Get a reference to the map entity's name.
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Get a reference to the map entity's fields.
    pub fn fields(&self) -> &Fields {
        &self.fields
    }

    // Pixel position of the entity
    pub fn xy(&self) -> IVec2 {
        self.xy
    }

    /// Get the map entity's grid xy.
    pub fn grid_xy(&self) -> IVec2 {
        self.grid_xy
    }

    /// Get the map entity's size.
    pub fn size(&self) -> IVec2 {
        self.size
    }

    pub fn grid_size(&self) -> i32 {
        self.pixels_per_unit
    }

    /// Get the map entity's def id.
    pub fn def_id(&self) -> i32 {
        self.def_id
    }

    /// Get the map entity's tile id.
    pub fn tile_id(&self) -> Option<i32> {
        self.tile_id
    }

    /// Get the map entity's tileset id.
    pub fn tileset_id(&self) -> Option<i32> {
        self.tileset_id
    }

    pub fn field(&self, name: &str) -> Option<&Value> {
        self.fields.field(name)
    }

    pub fn tags(&self) -> &Vec<String> {
        self.tags.as_ref()
    }

    pub fn tagged(&self, name: &str) -> bool {
        self.tags.iter().any(|t|t==name)
    }

    pub fn pivot(&self) -> Vec2 {
        self.pivot
    }

    pub fn get_str(&self, name: &str) -> &str {
        let field = self.fields.field(name).unwrap_or_else(||
            panic!("Error loading field {}, content: {:?}", name, self)
        );
        let str = field.as_str().unwrap_or_else(||
            panic!("Error loading field {}, not a string! Content {:?}", name, self)
        ); 
        str
    }
    pub fn get_f32(&self, name: &str) -> f32 {
        self.fields.field(name).unwrap().as_f32().unwrap()
    }

    /// Get the map entity's pixels per unit.
    pub fn pixels_per_unit(&self) -> i32 {
        self.pixels_per_unit
    }
}


pub trait Values {
    fn as_f32(&self) -> Option<f32>;
    fn as_i32(&self) -> Option<i32>;
    fn as_vec<T>(&self) -> Option<Vec<T>>;
}

impl Values for Value {
    fn as_f32(&self) -> Option<f32> {
        self.as_f64().map(|v|v as f32)
    }
    fn as_i32(&self) -> Option<i32> {
        self.as_i64().map(|v|v as i32)
    }
    fn as_vec<T>(&self) -> Option<Vec<T>> {
        todo!()
    }
}


// impl MapEntity {
//     pub fn get_field(&self, field_name: &str) -> &Value
//     {
//         self.fields.get
//     }
// }

#[derive(Default, Debug)]
pub struct EntitiesLayer {
    entities: Vec<MapEntity>,
    name: String,
    /// Map of entity uid to animations mapping. Each entity type
    /// has it's own set of animations.
    animations: HashMap<i32, HashMap<String, AnimationData>>,
}
impl EntitiesLayer {
    pub fn get_from_name(&self, name: &str) -> Option<&MapEntity> {
        self.entities.iter().find(|&e|e.name==name)
    }

    pub fn get_all_from_name<'a>(&'a self, name: &'a str) -> impl Iterator<Item=&MapEntity> {
        self.entities.iter().filter(move |&e|e.name==name)
    }

    pub fn get_tagged(&self, tag: &str) -> impl Iterator<Item=&MapEntity> {
        let tag = tag.to_string();
        self.entities.iter().filter(move |e|e.tags.contains(&tag))
    }

    pub fn entities(&self) -> impl Iterator<Item=&MapEntity> {
        self.entities.iter()
    } 
}



fn entities_from_defs(layer: &LayerInstance, defs: &HashMap<i64, &EntityDefinition>) -> EntitiesLayer {
    let layer_height = layer.c_hei;
    let layer_width = layer.c_wid;
    let layer_size = IVec2::new(layer_width as i32, layer_height as i32);

    let layer_px_width = layer_height * layer.grid_size;
    let layer_px_height = layer_width * layer.grid_size;
    let layer_px_size = IVec2::new(layer_width as i32, layer_height as i32);

    let mut entity_def_ids = HashSet::default();

    let mut entities = Vec::new();
    for entity in layer.entity_instances.iter() {
        let def = defs.get(&entity.def_uid).unwrap();
        
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

        let fields = Fields::from_ldtk(&entity.field_instances);

        let [grid_x, grid_y] = [entity.grid[0], entity.grid[1]];
        let [x,y] = [entity.px[0],entity.px[1]];
        let [width,height] = [entity.width, entity.height];

        let mut xy = Vec2::new(x as f32, y as f32);
        let size = Vec2::new(width as f32, height as f32);
        let mut pivot = Vec2::new(entity.pivot[0] as f32, entity.pivot[1] as f32);

        let y_flip = (layer_size.y - y as i32) as f32;

        xy.y += y_flip;

        pivot.y = 1.0 - pivot.y;

        let xy = xy - size * pivot;
        
        let xy = xy.round().as_ivec2();
        

        let tags = def.tags.clone();
        let size = size.as_ivec2();

        let entity = MapEntity {
            name: entity.identifier.to_lowercase(),
            fields,
            xy,
            grid_xy: IVec2::ZERO,
            size: size,
            def_id: entity.def_uid as i32,
            tile_id,
            tileset_id,
            pivot,
            tags,
            pixels_per_unit: size.y,
        };
        println!("game entity pos {}, gridxy {}, size {}, pivot {}", 
            entity.xy(), 
            entity.grid_xy(), 
            entity.size(), 
            entity.pivot());
        entities.push(entity);
    }

    let mut animations = HashMap::default();
    // for (id, def) in defs.iter() {
    //     animations.insert(*id as i32, anims_from_def(&def));
    // }

    EntitiesLayer {
        entities,
        name: layer.identifier.to_lowercase(),
        animations,
    }
}