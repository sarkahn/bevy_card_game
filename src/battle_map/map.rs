use bevy::{prelude::*, utils::HashMap};
use bevy_ascii_terminal::{ldtk::LdtkAsset, Point2d, Size2d};
use sark_pathfinding::{PathMap2d, PathingMap, AStar};

use crate::{GameState, config::GameSettings};

use super::render::LdtkRebuild;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Map>()
            .init_resource::<MapUnits>()
            .init_resource::<CollisionMap>()
            .add_system_set(SystemSet::on_enter(GameState::LoadBattleMap).with_system(setup))
            .add_system_set(SystemSet::on_update(GameState::BattleMap).with_system(update_collision_map))
            .add_event::<LdtkRebuild>()
            .add_system(build_from_ldtk);
    }
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum TerrainTile {
    Dirt,
    Grass,
    Mountain,
    Mud,
    Water,
}

impl Default for TerrainTile {
    fn default() -> Self {
        TerrainTile::Dirt
    }
}

#[derive(Default)]
pub struct Map {
    tiles: Vec<TerrainTile>,
    size: UVec2,
    id_to_tile: HashMap<i32,TerrainTile>,
    tile_to_id: HashMap<TerrainTile,i32>,
}

#[derive(Default)]
pub struct MapUnits {
    pub units: Vec<Option<Entity>>,
    size: UVec2,
}

pub struct CollisionMap(pub PathMap2d);
impl std::ops::Deref for CollisionMap {
    type Target = PathMap2d;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Default for CollisionMap {
    fn default() -> Self {
        Self(PathMap2d::new([0,0]))
    }
}

impl MapUnits {
    pub fn get(&self, xy: impl Point2d) -> Option<Entity> {
        let i = self.world_to_index(&xy);
        //println!("Trying to get unit at {}, i {}", xy.xy(), i );
        self.units[i]
    }
    pub fn to_xy(&self, i: usize) -> IVec2 {
        let x = i as i32 % self.size.x as i32;
        let y = i as i32 / self.size.x as i32;
        IVec2::new(x, y)
    }

    pub fn to_index(&self, xy: impl Point2d) -> usize {
        (xy.y() * self.size.x as i32 + xy.x()) as usize
    }

    pub fn world_to_index(&self, xy: &impl Point2d) -> usize {
        let xy = xy.xy() + self.size.as_ivec2() / 2;
        self.to_index(xy)
    }

    pub fn resize(&mut self, size: impl Size2d) {
        self.size = size.size();
        self.units = vec![Default::default(); size.len()];
    }
    pub fn clear(&mut self) {
        self.units.fill(None);
    }
    pub fn set(&mut self, xy: impl Point2d, unit: Option<Entity>) {
        let i = self.world_to_index(&xy);
        //println!("Putting unit at {}, I {}", xy.xy(), i);
        //self.units[i] = unit;

        //println!("Unit in vec: {:?}", self.units[i]);
    }
    pub fn empty(&self) -> bool {
        self.units.is_empty()
    }
}

impl Map {
    pub fn iter(&self) -> impl Iterator<Item = &TerrainTile> {
        self.tiles.iter()
    }

    pub fn resize(&mut self, size: impl Size2d) {
        self.size = size.size();
        self.tiles = vec![Default::default(); size.len()];
    }

    pub fn size(&self) -> UVec2 {
        self.size
    }

    pub fn to_xy(&self, i: usize) -> IVec2 {
        let x = i as i32 % self.size.x as i32;
        let y = i as i32 / self.size.x as i32;
        IVec2::new(x, y)
    }

    pub fn to_index(&self, xy: impl Point2d) -> usize {
        (xy.y() * self.size.x as i32 + xy.x()) as usize
    }

    fn map_tile(&mut self, id: i32, tile: TerrainTile) {
        self.id_to_tile.insert(id, tile);
        self.tile_to_id.insert(tile,id);
    }

    pub fn tile_id(&self, tile: TerrainTile) -> Option<&i32> {
        self.tile_to_id.get(&tile)
    }
}

fn setup(
    settings: Res<GameSettings>,
    mut commands: Commands, 
    asset_server: Res<AssetServer>
) {
    let ldtk_handle: Handle<LdtkAsset> = asset_server.load(&settings.map_file);

    commands.spawn().insert(ldtk_handle).insert(BuildFromLdtk);
}

#[derive(Component)]
struct BuildFromLdtk;

fn build_from_ldtk(
    mut map: ResMut<Map>,
    mut units: ResMut<MapUnits>,
    mut commands: Commands,
    ldtk_assets: Res<Assets<LdtkAsset>>,
    mut ev_ldtk: EventReader<AssetEvent<LdtkAsset>>,
    q_builder: Query<(Entity, &Handle<LdtkAsset>), With<BuildFromLdtk>>,
    mut ev_ldtk_writer: EventWriter<LdtkRebuild>,
    mut game_state: ResMut<State<GameState>>,
) {
    for ev in ev_ldtk.iter() {
        match ev {
            AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
                println!("Asset loaded...");
                for (entity, old_handle) in q_builder.iter() {
                    if old_handle != handle {
                        continue;
                    }
                    commands.entity(entity).remove::<BuildFromLdtk>();

                    if let Some(new_map) = ldtk_assets.get(handle) {
                        let p = &new_map.project;
                        if !new_map.tilesets.is_empty() {
                            let tex_handle = new_map.tilesets.iter().next().unwrap().1.clone();

                            let tileset = &new_map.project.defs.tilesets[0];
                            let data = &tileset.custom_data;
                            for data in data.iter() {
                                let id = data.get("tileId").unwrap().as_ref().unwrap();
                                let id = id.as_i64().unwrap() as i32;
                                let tile = data.get("data").unwrap().as_ref().unwrap();
                                let tile = tile.as_str().unwrap();
                                println!("Mapping tile {} to {}", tile, id);
                                map.map_tile(id, name_to_tile(tile));
                            };
                        
                            //println!("Size of tiles vec {}", tiles.len());
                            
                            let w = tileset.c_wid as u32;
                            let h = tileset.c_hei as u32;
                            ev_ldtk_writer.send(LdtkRebuild {
                                map_size: map.size(),
                                tileset_size: UVec2::new(w, h),
                                tex: tex_handle.clone(),
                            });
                            for level in p.levels.iter() {
                                if let Some(layers) = &level.layer_instances {
                                    let w = layers.iter().map(|l| l.c_wid).max().unwrap() as u32;
                                    let h = layers.iter().map(|l| l.c_hei).max().unwrap() as u32;
                                    map.resize([w, h]);
                                    units.resize([w,h]);
    
                                    println!("Populating map. Size: {}", map.size());
                                    for layer in layers.iter().rev() {
                                        let height_offset = layer.c_hei as i32 - 1;
                                        for tile in layer.grid_tiles.iter() {
                                            let xy = IVec2::new(tile.px[0] as i32, tile.px[1] as i32);
                                            let xy = xy / layer.grid_size as i32;
                                            let xy = IVec2::new(xy.x, height_offset - xy.y);
                                            let i = xy.y as usize * h as usize + xy.x as usize;
    
                                            let id = tile.t as i32;
                                            if let Some(tile) = map.id_to_tile.get(&id) {
                                                map.tiles[i] = *tile;
                                            }
                                        }
                                        for tile in layer.auto_layer_tiles.iter() {
                                            let xy = IVec2::new(tile.px[0] as i32, tile.px[1] as i32);
                                            let xy = xy / layer.grid_size as i32;
                                            let xy = IVec2::new(xy.x, height_offset - xy.y);
                                            let i = xy.y as usize * h as usize + xy.x as usize;
    
                                            let id = tile.t as i32;
                                            if let Some(tile) = map.id_to_tile.get(&id) {
                                                map.tiles[i] = *tile;
                                            }
                                        }
                                    }
    
                                    game_state.set(GameState::BattleMap).unwrap();
                                }
                            }
                        }
                    }
                        
                }
            }
            _ => {}
        }
    }
}

fn name_to_tile(name: &str) -> TerrainTile {
    match name {
        "Dirt" => TerrainTile::Dirt,
        "Grass" => TerrainTile::Grass,
        "Mountain" => TerrainTile::Mountain,
        "Mud" => TerrainTile::Mud,
        "Water" => TerrainTile::Water,
        _ => TerrainTile::Dirt,
    }
}

fn update_collision_map(
    mut collision_map: ResMut<CollisionMap>,
    map: Res<Map>,
) {
    if !map.is_changed() {
        return;
    }
    if map.size() != collision_map.size() {
        collision_map.0 = PathMap2d::new(map.size().into());
    }
    println!("Updating collision map. Size {}", map.size());
    for (coll,tile) in collision_map.0.iter_mut().zip(map.iter()) {
        *coll = match tile {
            TerrainTile::Mountain => true,
            TerrainTile::Water => true,
            _ => false,
        };
    }
}