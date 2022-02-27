use std::slice::Iter;

use bevy::{prelude::*, utils::HashMap, ecs::system::EntityCommands};
use bevy_ascii_terminal::{ldtk::LdtkAsset, Point2d, Size2d};
use bevy_tiled_camera::TiledProjection;
use sark_grids::Grid;
use sark_pathfinding::{AStar, PathMap2d, PathingMap, pathing_map::ArrayVec};

use crate::{config::{GameSettings, ConfigAsset}, GameState, ldtk_loader::{LoadLdtkMap, LdtkMapBuilt}, SETTINGS_PATH};

use super::{units::{MapUnit, MapUnitBundle}, MapPosition, BattleMapState};

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Map>()
            .init_resource::<MapUnits>()
            .init_resource::<CollisionMap>()
            
            .add_system_set(
                SystemSet::on_update(GameState::LoadBattleMap)
                .with_system(setup)
            ).add_system_set(
                SystemSet::on_update(BattleMapState::BuildingMap)
                .with_system(build_map)
            );
            //.add_system_set(SystemSet::on_enter(GameState::LoadBattleMap).with_system(setup))
            // .add_system_set(
            //     SystemSet::on_update(GameState::BattleMap).with_system(update_collision_map),
            // )
            //.add_event::<LdtkRebuild>()
            ;
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
        Vec2::select(cmp, Vec2::new(0.5,0.5), Vec2::ZERO)
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

fn setup(
    configs: Res<Assets<ConfigAsset>>,
    mut ev_writer: EventWriter<LoadLdtkMap>,
    mut state: ResMut<State<BattleMapState>>,
) {
    if *state.current() == BattleMapState::BuildingMap {
        return;
    }
    if let Some(config) = configs.get(SETTINGS_PATH) {
        state.set(BattleMapState::BuildingMap).unwrap();
        ev_writer.send(LoadLdtkMap::from_path(config.settings.map_file.to_owned()));
    }
}

#[derive(Component)]
pub struct MapTile;

/// Offset a given axis based on whether it's even or odd.
/// Allows for a nicely centered map even with odd numbered tiles.
fn axis_offset(size: IVec2) -> Vec2 {
    let cmp = (size % 2).cmpeq(IVec2::ZERO);
    Vec2::select(cmp, Vec2::new(0.5,0.5), Vec2::ZERO)
}
fn build_map(
    mut commands: Commands,
    mut ev_reader: EventReader<LdtkMapBuilt>,
    mut q_cam: Query<&mut TiledProjection>,
    mut collision_map: ResMut<CollisionMap>,
    mut map: ResMut<Map>,
    mut units: ResMut<MapUnits>,
    mut battle_map_state: ResMut<State<BattleMapState>>,
    mut game_state: ResMut<State<GameState>>,
) {
    for ev in ev_reader.iter() {
        let ldtk_map = &ev.0;

        if map.size().as_ivec2() != ldtk_map.size {
            map.0 = Grid::new(TerrainTile::default(), ldtk_map.size.as_uvec2().into());
            collision_map.0 = PathMap2d::new(map.size().into());
            units.0 = Grid::default(collision_map.size().into());
        }

        let axis_offset = axis_offset(ldtk_map.size);
        if let Ok(mut cam) = q_cam.get_single_mut() {
            cam.pixels_per_tile = ldtk_map.tile_size.y as u32;
            cam.set_tile_count(ldtk_map.size.as_uvec2().into());
        }
        for (depth, layer) in ldtk_map.layers.iter().rev().enumerate() {
            let atlas = &layer.atlas;
            let layer_name = &layer.name;
            let tileset = &ldtk_map.tilesets.get(&layer.tileset_id).expect("No tileset for layer");
            for tile in layer.tiles.iter() {
                let xy = tile.xy.as_vec2() + axis_offset;

                let transform = Transform::from_xyz(xy.x, xy.y, depth as f32);
                let sprite = TextureAtlasSprite {
                    custom_size: Some(Vec2::ONE),
                    index: tile.id as usize,
                    ..Default::default()
                };
                let sprite = SpriteSheetBundle {
                    sprite,
                    texture_atlas: atlas.clone(),
                    transform,
                    ..Default::default()
                };

                let mut entity = commands.spawn_bundle(sprite);
                match layer_name.to_lowercase().as_str() {
                    "units" => {
                        entity.insert_bundle(MapUnitBundle::new(xy.round().as_ivec2()));
                    }
                    _ => {}
                }
                if let Some(data) = tileset.tile_data.get(&tile.id) {
                    if data.lines().map(|l|l.to_lowercase()).position(|s| s=="collider").is_some() {
                        let xy = tile.xy + map.size().as_ivec2() / 2;
                        collision_map.set_collidable(xy);
                    }
                } 
            }
        }
        battle_map_state.set(BattleMapState::SelectUnit).unwrap();
        game_state.set(GameState::BattleMap).unwrap();
    }
}