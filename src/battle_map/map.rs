use std::slice::Iter;

use bevy::{ecs::system::EntityCommands, prelude::*, utils::HashMap};
use bevy_ascii_terminal::{ldtk::LdtkAsset, Point2d, Size2d};
use bevy_tiled_camera::TiledProjection;
use sark_grids::Grid;
use sark_pathfinding::{pathing_map::ArrayVec, AStar, PathMap2d, PathingMap};

use crate::{
    config::{ConfigAsset, GameSettings},
    ldtk_loader::{LdtkMap, MapLayer, MapEntity, MapTileset, MapTile},
    AtlasHandles, GameState, SETTINGS_PATH, UnitAnimation, AnimationController,
};

use super::{units::{MapUnitBundle, MapUnit}, BattleMapState};

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Map>()
            .init_resource::<MapUnits>()
            .init_resource::<CollisionMap>()
            .init_resource::<LoadingMap>()
            .add_system_set(SystemSet::on_update(GameState::LoadBattleMap).with_system(setup))
            .add_system_set(
                SystemSet::on_update(BattleMapState::BuildingMap).with_system(build_map),
            );
        //.add_system_set(SystemSet::on_enter(GameState::LoadBattleMap).with_system(setup))
        // .add_system_set(
        //     SystemSet::on_update(GameState::BattleMap).with_system(update_collision_map),
        // )
        //.add_event::<LdtkRebuild>()
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

#[derive(Default)]
struct LoadingMap(Handle<LdtkMap>);
fn setup(
    configs: Res<Assets<ConfigAsset>>,
    mut state: ResMut<State<BattleMapState>>,
    asset_server: Res<AssetServer>,
    mut loading_map: ResMut<LoadingMap>,
) {
    if *state.current() == BattleMapState::BuildingMap {
        return;
    }
    if let Some(config) = configs.get(SETTINGS_PATH) {
        loading_map.0 = asset_server.load(&config.settings.map_file);
        state.set(BattleMapState::BuildingMap).unwrap();
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
    mut ev_reader: EventReader<AssetEvent<LdtkMap>>,
    ldtk_maps: Res<Assets<LdtkMap>>,
    mut q_cam: Query<&mut TiledProjection>,
    mut collision_map: ResMut<CollisionMap>,
    mut map: ResMut<Map>,
    mut units: ResMut<MapUnits>,
    mut battle_map_state: ResMut<State<BattleMapState>>,
    mut game_state: ResMut<State<GameState>>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
    mut atlas_handles: ResMut<AtlasHandles>,
) {
    for ev in ev_reader.iter() {
        match ev {
            AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
                let ldtk = ldtk_maps.get(handle).unwrap();

                if map.size().as_ivec2() != ldtk.size {
                    map.0 = Grid::new(TerrainTile::default(), ldtk.size.as_uvec2().into());
                    collision_map.0 = PathMap2d::new(map.size().into());
                    units.0 = Grid::default(collision_map.size().into());
                }
                if let Ok(mut cam) = q_cam.get_single_mut() {
                    cam.pixels_per_tile = ldtk.tile_size.y as u32;
                    cam.set_tile_count(ldtk.size.as_uvec2().into());
                }

                for (depth, layer) in ldtk.layers.iter().rev().enumerate() {
                    match layer {
                        MapLayer::Tiles(layer) => {
                            let tileset = ldtk.tileset(layer.tileset_id);
                            
                            let atlas = get_atlas(&mut atlases, &mut atlas_handles, &tileset);
                            let axis_offset = axis_offset(ldtk.size);

                            for tile in layer.tiles.iter() {
                                spawn_tile(
                                    &mut commands, 
                                    tile, 
                                    axis_offset, 
                                    depth,
                                    atlas.clone(), 
                                    tileset, 
                                    &mut collision_map
                                );
                            }
                        },
                        MapLayer::Entities(layer) => {
                            let animations = &layer.animations;
                            for entity in layer.entities.iter() {
                                spawn_entity(
                                    &mut commands,
                                    ldtk,
                                    &mut atlases,
                                    &mut atlas_handles,
                                    entity,
                                    &mut units,
                                    depth,
                                    animations,
                                );
                            }
                        },
                    }
                }
                battle_map_state.set(BattleMapState::SelectUnit).unwrap();
                game_state.set(GameState::BattleMap).unwrap();
            }
            AssetEvent::Removed { handle: _ } => {}
        }
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
    if let Some(data) = tileset.tile_data.get(&tile.id) {
        if data
            .lines()
            .map(|l| l.to_lowercase())
            .position(|s| s == "collider")
            .is_some()
        {
            let xy = tile.xy + collision_map.size().as_ivec2() / 2;
            collision_map.set_collidable(xy);
        }
    }
}

fn spawn_entity(
    commands: &mut Commands,
    ldtk: &LdtkMap,
    atlases: &mut Assets<TextureAtlas>,
    atlas_handles: &mut AtlasHandles,
    entity: &MapEntity,
    units: &mut MapUnits,
    depth: usize,
    animations: &HashMap<i32, HashMap<String, UnitAnimation>>,
) {
    let axis_offset = axis_offset(units.size().as_ivec2());
    if let (Some(id), Some(tileset_id)) = (entity.tile_id, entity.tileset_id) {
        let tileset = ldtk.tileset(tileset_id);
        let xy = entity.xy.as_vec2() + axis_offset;
        
        let transform = Transform::from_xyz(xy.x, xy.y, depth as f32);
        let sprite = TextureAtlasSprite {
            custom_size: Some(Vec2::ONE),
            index: id as usize,
            ..Default::default()
        };
        let atlas = get_atlas(atlases, atlas_handles, tileset);
        let sprite = SpriteSheetBundle {
            sprite,
            texture_atlas: atlas,
            transform,
            ..Default::default()
        };
        let mut new = commands.spawn_bundle(sprite);
        new.insert_bundle(MapUnitBundle::new(xy.as_ivec2()));

        if let Some(anims) = animations.get(&entity.def_id) {
            println!("Loading animations for {}", entity.name);
            let mut controller = AnimationController::default();
            for (name, anim) in anims {
                controller.add(name, anim.clone());
            }
            controller.play("idle");
            new.insert(controller);
        }
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