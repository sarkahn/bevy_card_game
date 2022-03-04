use std::slice::Iter;

use bevy::{
 math::Vec3Swizzles, prelude::*, 
};
use bevy_ascii_terminal::{Point2d};
use bevy_tiled_camera::TiledProjection;
use sark_grids::Grid;
use sark_pathfinding::PathMap2d;

use crate::{
    config::{ConfigAsset, GameSettings},
    ldtk_loader::{LdtkMap, MapEntity, MapLayer, MapTile, MapTileset, TilesLayer, EntitiesLayer},
    AnimationController, AtlasHandles, GameState, AnimationData, SETTINGS_PATH, make_sprite_atlas, GridHelper, TILE_SIZE, battle_map::units::PlayerBase,
};

use super::{ units::{MapUnit, PlayerUnit, EnemyUnit, MapUnitBundle, UnitCommand}, enemies::Spawner};


pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<MapUnits>()
            .init_resource::<CollisionMap>()
            .init_resource::<BattleMapLdtkHandle>()
            .add_system_set(
                SystemSet::on_update(GameState::LoadBattleMap).with_system(build_map),
            )
            .add_system_set(
                SystemSet::on_update(GameState::BattleMap).with_system(update_map_units),
            )
            ;
    }
}


#[derive(Default)]
pub struct BattleMapLdtkHandle(pub Handle<LdtkMap>);

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
pub struct MapUnits {
    units: Vec<Option<Entity>>,
    size: IVec2,
}
impl MapUnits {
    pub fn new(size: IVec2) -> Self {
        let len = size.x * size.y;
        Self {
            units: vec![None; len as usize],
            size,
        }
    }

    pub fn xy_to_index(&self, xy: Vec2) -> usize {
        let grid = self.xy_to_grid(xy);
        self.grid_to_index(grid)
    }

    pub fn grid_to_index(&self, grid: IVec2) -> usize {
        (grid.y * self.size.x + grid.x) as usize
    }

    pub fn xy_to_grid(&self, xy: Vec2) -> IVec2 {
        xy.floor().as_ivec2() / TILE_SIZE
    }
    pub fn grid_to_xy(&self, grid: IVec2) -> Vec2 {
        (grid * TILE_SIZE).as_vec2()
    }


    pub fn resize(&mut self, size: IVec2) {
        let len = size.x * size.y;
        self.units = vec![None; len as usize];
        self.size = size;   
    }

    pub fn len(&self) -> usize {
        (self.size.x * self.size.y) as usize 
    }

    pub fn grid_size(&self) -> IVec2 {
        self.size
    }

    pub fn clear(&mut self) {
        self.units = vec![None;self.len()];
    } 

    pub fn get_from_xy(&self, xy: Vec2) -> Option<Entity> {
        let i = self.xy_to_index(xy);
        self.units[i]
    }

    pub fn get_from_grid_xy(&self, grid_xy: IVec2) -> Option<Entity> {
        let i = self.grid_to_index(grid_xy);
        self.units[i]
    }
    pub fn get_from_index(&self, index: usize) -> Option<Entity> {
        self.units[index]
    }
    
    pub fn set_from_grid_xy(&mut self, grid_xy: IVec2, entity: Entity) {
        //println!("xy in {}", grid_xy);
        let i = self.grid_to_index(grid_xy);
        self.units[i] = Some(entity)
    }

    pub fn set_from_xy(&mut self, xy: Vec2, entity: Entity) {
        let i = self.xy_to_index(xy);
        //println!("inserting at {}", i);
        self.units[i] = Some(entity);
    }
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
        Self(PathMap2d::new([0, 0]))
    }
}

impl CollisionMap {
    pub fn set_collidable(&mut self, pos: impl Point2d) {
        let xy = pos.xy().to_array();
        if !self.0.is_obstacle(xy) {
            let i = self.0.to_index(xy.into());
            self.0.toggle_obstacle_index(i);
        }
    }
    pub fn axis_offset(&self) -> Vec2 {
        let cmp = (self.size().as_ivec2() % 2).cmpeq(IVec2::ZERO);
        Vec2::select(cmp, Vec2::new(0.5, 0.5), Vec2::ZERO)
    }

    pub fn half_offset(&self) -> IVec2 {
        let p = (self.size().as_vec2() / 2.0) + self.axis_offset();
        return -p.as_ivec2()
    }
}

pub fn axis_offset(size: IVec2) -> Vec2 {
    let cmp = (size % 2).cmpeq(IVec2::ZERO);
    Vec2::select(cmp, Vec2::new(0.5, 0.5), Vec2::ZERO)
}

fn build_map(
    mut commands: Commands,
    configs: Res<Assets<ConfigAsset>>,
    ldtk: Res<Assets<LdtkMap>>,
    mut game_state: ResMut<State<GameState>>,
    mut atlas_handles: ResMut<AtlasHandles>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
    mut map: ResMut<CollisionMap>,
    mut q_cam: Query<&mut TiledProjection>,
    mut units: ResMut<MapUnits>,
) {
    if let Some(configs) = configs.get(SETTINGS_PATH) {
        if let Some(ldtk) = ldtk.get(&configs.settings.map_file) {
            map.0 = PathMap2d::new(ldtk.size_px().as_uvec2().into());
            if let Some(tile_count) = ldtk.tile_count() {
                q_cam.single_mut().set_tile_count(tile_count.as_uvec2().into());
            }
            units.resize(map.size().as_ivec2());
            for (i,layer) in ldtk.layers().enumerate() {
                match layer {
                    MapLayer::Tiles(layer) => {
                        build_tile_layer(
                            &mut commands, 
                            ldtk, layer, 
                            &mut atlases,  
                            &mut atlas_handles, 
                            i as i32 )

                    },
                    MapLayer::Entities(layer) => {
                        build_entity_layer(
                            &mut commands, 
                            ldtk, layer, 
                            &mut atlases, 
                            &mut atlas_handles, 
                            i as i32
                        );
                    },                        
                }
                update_colliders(&mut map, &units, layer);
            }

            game_state.set(GameState::BattleMap).unwrap();
        }  
    }
}

fn build_tile_layer(
    commands: &mut Commands,
    ldtk: &LdtkMap,
    tiles: &TilesLayer,
    atlases: &mut Assets<TextureAtlas>,
    atlas_handles: &mut AtlasHandles,
    depth: i32,
) {
    let tileset = ldtk.tileset_from_id(tiles.tileset_id).unwrap();
    let atlas = get_atlas(atlases, atlas_handles, tileset);
    for tile in &tiles.tiles {
        //let offset = axis_offset(ldtk.size_px());
        //et xy = tile.grid_xy().as_vec2() + offset;//
        let xy = tile.pixel_xy().as_vec2();
        make_sprite_atlas(commands, xy, depth, atlas.clone(), tile.id() as usize);
        
    }
}

fn build_entity_layer(
    commands: &mut Commands,
    ldtk: &LdtkMap,
    layer: &EntitiesLayer,
    atlases: &mut Assets<TextureAtlas>,
    atlas_handles: &mut AtlasHandles,
    depth: i32,
) {
    for entity in layer.entities() {
        if let Some(tsid) = entity.tileset_id() {
            if let Some(tileset) = ldtk.tileset_from_id(tsid) {
                let atlas = get_atlas(atlases, atlas_handles, tileset);
                let xy = entity.pixel_xy();
                //println!("Spawning entity at {}", xy);
                let mut sprite = make_sprite_atlas(
                    commands, 
                    xy.as_vec2(), 
                    depth, 
                    atlas.clone(), 
                    entity.tile_id().unwrap_or(0) as usize
                );
                if entity.tags().contains(&"animation".to_string()) {
                    let frames = entity.get_str("frames");
                    let speed = entity.get_f32("speed");
                    let anim = AnimationData {
                        name: Default::default(),
                        frames: ron::de::from_str(frames).unwrap(),
                        speed: speed,
                        tileset_path: tileset.path.to_string(),
                        ldtk_name: ldtk.name().to_string(),
                    };
                    let controller = AnimationController::from(anim);
                    sprite.insert(controller);
                }
                if entity.tagged("player") {
                    sprite.insert(PlayerUnit).insert_bundle(MapUnitBundle::default());
                }
                if entity.tagged("monster") {
                    sprite.insert(EnemyUnit).insert_bundle(
                        MapUnitBundle::with_commands(
                            &[UnitCommand::AiThink()]
                        )
                    );
                }
                if entity.tagged("spawner") {
                    println!("SPAWNER");
                    let delay = entity.get_f32("spawn_delay");
                    sprite.insert(Spawner{
                        timer: Timer::from_seconds(delay, true),
                    }).insert(EnemyUnit);
                }

                if entity.tagged("player_base") {
                    println!("FOUND CASTLE on {:?}", entity.name());
                    sprite.insert(PlayerBase);
                }

            }
        }
    }
}

fn update_colliders(
    map: &mut CollisionMap,
    units: &MapUnits,
    layer: &MapLayer,
) {
    match layer {
        MapLayer::Tiles(layer) => {
            for tile in layer.tiles.iter() {
                if layer.has_enum(tile.id(), "collider") {
                    let xy = units.xy_to_grid(tile.pixel_xy().as_vec2());

                    //let xy = xy + map.size().as_ivec2() / 2;
                    map.set_collidable(xy);
                }
            }
        },
        MapLayer::Entities(layer) => {
            for entity in layer.entities() {
                if entity.tagged("collider") {
                    let xy = units.xy_to_grid(entity.pixel_xy().as_vec2());
                    map.set_collidable(xy);
                } 
            }
        },
    }
}



fn update_map_units(
    mut units: ResMut<MapUnits>,
    q_units: Query<(Entity,&Transform),(With<MapUnit>, With<PlayerUnit>)>, 
) {
    units.clear();

    for (entity,transform) in q_units.iter() {
     
        //println!("Inserting {:?} at {}", entity, xy);
        //let i = units.xy_to_index(transform.translation.xy());
        units.set_from_xy(transform.translation.xy(), entity);
    }
}

fn get_atlas(
    atlases: &mut Assets<TextureAtlas>,
    atlas_handles: &mut AtlasHandles,
    tileset: &MapTileset,
) -> Handle<TextureAtlas> {
    let name = &tileset.name;
    match atlas_handles.0.get(name) {
        Some(atlas) => atlas.clone(),
        None => {
            let atlas = TextureAtlas::from_grid(
                tileset.image.clone(),
                IVec2::splat(tileset.tile_size).as_vec2(),
                tileset.tile_count.x as usize,
                tileset.tile_count.y as usize,
            );
            let handle = atlases.add(atlas);
            atlas_handles.0.insert(name.to_string(), handle.clone());
            handle
        }
    }
}