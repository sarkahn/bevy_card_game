use std::slice::Iter;

use bevy::{
    ecs::system::EntityCommands, math::Vec3Swizzles, prelude::*, reflect::TypeUuid, utils::HashMap,
};
use bevy_ascii_terminal::{ldtk::LdtkAsset, Point2d, Size2d};
use bevy_tiled_camera::TiledProjection;
use sark_grids::Grid;
use sark_pathfinding::PathMap2d;

use crate::{
    config::{ConfigAsset, GameSettings},
    ldtk_loader::{LdtkMap, MapEntity, MapLayer, MapTile, MapTileset},
    AnimationController, AtlasHandles, GameState, AnimationData, SETTINGS_PATH,
};

use super::{spawn::SpawnUnit};

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Map>()
            .init_resource::<MapUnits>()
            .init_resource::<CollisionMap>()
            .init_resource::<BattleMapLdtkHandle>()
            .add_system_set(
                SystemSet::on_update(GameState::LoadBattleMap).with_system(build_map),
            );
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
pub struct Map(pub Grid<TerrainTile>);

impl std::ops::Deref for Map {
    type Target = Grid<TerrainTile>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Map {
    /// Offset a given axis based on whether it's even or odd.
    /// Allows for a nicely centered map even with odd numbered tiles.
    pub fn axis_offset(&self) -> Vec2 {
        let cmp = (self.size().as_ivec2() % 2).cmpeq(IVec2::ZERO);
        Vec2::select(cmp, Vec2::new(0.5, 0.5), Vec2::ZERO)
    }
    pub fn to_index_2d(&self, point: Vec2) -> IVec2 {
        (point + self.size().as_vec2() / 2.0).floor().as_ivec2()
    }

    pub fn xy_from_index_2d(&self, point: IVec2) -> Vec2 {
        (point - self.size().as_ivec2() / 2).as_vec2()
    }

    pub fn to_xy(&self, point: Vec2) -> IVec2 {
        point.floor().as_ivec2()
    }

    pub fn transform_to_xy(&self, transform: &Transform) -> IVec2 {
        let xy = transform.translation.xy() + Vec2::new(0.5, 0.5);
        xy.floor().as_ivec2()
    }

    pub fn pos_to_tile_center(&self, xy: Vec2) -> Vec2 {
        xy.floor() + Vec2::new(0.5, 0.5)
    }
}

#[derive(Default)]
pub struct MapUnits(pub Grid<Option<Entity>>);

impl std::ops::Deref for MapUnits {
    type Target = Grid<Option<Entity>>;

    fn deref(&self) -> &Self::Target {
        &self.0
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
}

/// Offset a given axis based on whether it's even or odd.
/// Allows for a nicely centered map even with odd numbered tiles.
fn axis_offset(size: IVec2) -> Vec2 {
    let cmp = (size % 2).cmpeq(IVec2::ZERO);
    Vec2::select(cmp, Vec2::new(0.5, 0.5), Vec2::ZERO)
}

fn build_map(
    mut commands: Commands,
    mut q_cam: Query<&mut TiledProjection>,
    mut collision_map: ResMut<CollisionMap>,
    mut map: ResMut<Map>,
    mut units: ResMut<MapUnits>,
    mut game_state: ResMut<State<GameState>>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
    mut atlas_handles: ResMut<AtlasHandles>,
    mut ev_spawn: EventWriter<SpawnUnit>,
    configs: Res<Assets<ConfigAsset>>,
    ldtk: Res<Assets<LdtkMap>>,
) {
    let configs = configs.get(SETTINGS_PATH).unwrap();
    if let Some(ldtk) = ldtk.get(&configs.settings.map_file) {
        println!("Loading map!");
        if map.size().as_ivec2() != ldtk.size() {
            map.0 = Grid::new(TerrainTile::default(), ldtk.size().as_uvec2().into());
            collision_map.0 = PathMap2d::new(map.size().into());
            units.0 = Grid::default(collision_map.size().into());
        }
        if let Ok(mut cam) = q_cam.get_single_mut() {
            cam.pixels_per_tile = 64;
            cam.set_tile_count(ldtk.size().as_uvec2().into());
        }

        for (depth, layer) in ldtk.layers().rev().enumerate() {
            match layer {
                MapLayer::Tiles(layer) => {
                    let tileset = ldtk.tileset_from_id(layer.tileset_id).unwrap();

                    let atlas = get_atlas(&mut atlases, &mut atlas_handles, &tileset);
                    let axis_offset = axis_offset(ldtk.size());

                    for tile in layer.tiles.iter() {
                        spawn_tile(
                            &mut commands,
                            tile,
                            axis_offset,
                            depth,
                            atlas.clone(),
                            tileset,
                            &mut collision_map,
                        );
                    }
                }
                MapLayer::Entities(layer) => {
                    //let animations = Default::default();
                    for entity in layer.entities() {
                        spawn_entity(
                            ldtk,
                            &mut atlases,
                            &mut atlas_handles,
                            entity,
                            &mut units,
                            depth,
                            //animations,
                            &mut ev_spawn,
                        );
                    }
                }
            }
        }
        game_state.set(GameState::BattleMap).unwrap();
    }
    
}

fn spawn_tile(
    commands: &mut Commands,
    tile: &MapTile,
    axis_offset: Vec2,
    depth: usize,
    atlas: Handle<TextureAtlas>,
    tileset: &MapTileset,
    collision_map: &mut CollisionMap,
) {
    let xy = tile.xy.as_vec2() + axis_offset;

    let transform = Transform::from_xyz(xy.x, xy.y, depth as f32);
    let sprite = TextureAtlasSprite {
        custom_size: Some(Vec2::ONE),
        index: tile.id as usize,
        ..Default::default()
    };
    let sprite = SpriteSheetBundle {
        sprite,
        texture_atlas: atlas,
        transform,
        ..Default::default()
    };

    commands.spawn_bundle(sprite);
    if let Some(enums) = tileset.enums.get(&tile.id) {
        //println!("Found enums for tileset {}: {:?}", tileset.name, enums);
        if enums.iter().any(|s| s == "collider") {
            let xy = tile.xy + collision_map.size().as_ivec2() / 2;
            collision_map.set_collidable(xy);
        }
    }
}

fn spawn_entity(
    ldtk: &LdtkMap,
    atlases: &mut Assets<TextureAtlas>,
    atlas_handles: &mut AtlasHandles,
    entity: &MapEntity,
    units: &mut MapUnits,
    depth: usize,
    //animations: &HashMap<i32, HashMap<String, AnimationData>>,
    ev_spawn: &mut EventWriter<SpawnUnit>,
) {
    let axis_offset = axis_offset(units.size().as_ivec2());
    if let (Some(tile_id), Some(tileset_id)) = (entity.tile_id(), entity.tileset_id()) {
        let tileset = ldtk.tileset_from_id(tileset_id).unwrap();
        let atlas = get_atlas(atlases, atlas_handles, tileset);
        let xy = entity.grid_xy();
        //let position = (entity.grid_xy.as_vec2() + axis_offset)
        //    .floor()
        //    .extend(depth as f32)
        //    .as_ivec3();
        //println!("Spawn pos for entity {}: {}", entity.name, position);
        let position = xy.extend(depth as i32);
        //let animations = animations.get(&entity.def_id());

        let enums = tileset.enums.get(&tile_id);

        let data = SpawnUnit {
            atlas: atlas.clone(),
            sprite_index: tile_id,
            position,
            //animations: animations.cloned(),
            enums: enums.cloned(),
        };

        ev_spawn.send(data);
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
