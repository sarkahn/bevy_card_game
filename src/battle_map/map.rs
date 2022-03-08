use std::slice::Iter;

use bevy::{math::Vec3Swizzles, prelude::*};
use bevy_ascii_terminal::Point2d;
use bevy_tiled_camera::TiledProjection;
use sark_grids::Grid;
use sark_pathfinding::PathMap2d;

use crate::{
    config::ConfigAsset,
    ldtk_loader::{EntitiesLayer, LdtkMap, TilesLayer, Tags, MapLayer, MapTileset},
    make_sprite_atlas, AtlasHandles, GameState,
    SETTINGS_PATH, TILE_SIZE, animation::{Animator, Animation},
};

use super::{ EnemyUnit, MapUnit, PlayerUnit, MapLoaded, BattleMapEntity,
};

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapUnits>()
            .init_resource::<CollisionMap>()
            .init_resource::<BattleMapLdtkHandle>()
            .add_system_set(SystemSet::on_update(GameState::LoadBattleMap)
                .with_system(build_map)
                .label(BUILD_MAP_SYSTEM))
            .add_system_set(
                SystemSet::on_update(GameState::BattleMap)
                    .with_system(update_map_units),
            )
            ;
    }
}

pub const BUILD_MAP_SYSTEM: &str = "build_map_system";

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

    #[inline]
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
    #[inline]
    pub fn grid_to_xy(&self, grid: IVec2) -> Vec2 {
        (grid * TILE_SIZE).as_vec2()
    }

    pub fn resize(&mut self, size: IVec2) {
        let len = size.x * size.y;
        self.units = vec![None; len as usize];
        self.size = size;
    }

    #[inline]
    pub fn len(&self) -> usize {
        (self.size.x * self.size.y) as usize
    }

    #[inline]
    pub fn grid_size(&self) -> IVec2 {
        self.size
    }

    #[inline]
    pub fn clear(&mut self) {
        self.units.iter_mut().for_each(|u| *u = None);
    }

    #[inline]
    pub fn get_from_xy(&self, xy: Vec2) -> Option<Entity> {
        let i = self.xy_to_index(xy);
        self.units[i]
    }

    #[inline]
    pub fn get_from_grid_xy(&self, grid_xy: IVec2) -> Option<Entity> {
        if grid_xy.cmplt(IVec2::ZERO).any() || grid_xy.cmpge(self.size).any() {
            return None;
        }
        let i = self.grid_to_index(grid_xy);
        self.units[i]
    }
    #[inline]
    pub fn get_from_index(&self, index: usize) -> Option<Entity> {
        self.units[index]
    }

    #[inline]
    pub fn set_from_grid_xy(&mut self, grid_xy: IVec2, entity: Entity) {
        //println!("xy in {}", grid_xy);
        let i = self.grid_to_index(grid_xy);
        self.units[i] = Some(entity)
    }

    #[inline]
    pub fn set_from_xy(&mut self, xy: Vec2, entity: Entity) {
        let i = self.xy_to_index(xy);
        //println!("inserting at {}", i);
        self.units[i] = Some(entity);
    }

    pub fn size(&self) -> IVec2 {
        self.size
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
        return -p.as_ivec2();
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
    mut atlas_handles: ResMut<AtlasHandles>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
    mut map: ResMut<CollisionMap>,
    //mut q_cam: Query<&mut TiledProjection>,
    mut units: ResMut<MapUnits>,
    q_loaded: Query<&MapLoaded>,
    mut state: ResMut<State<GameState>>,
    mut q_cam: Query<&mut Transform, With<Camera>>,
) {
    if !q_loaded.is_empty() {
        return;
    }
    if let Some(configs) = configs.get(SETTINGS_PATH) {
        if let Some(ldtk) = ldtk.get(&configs.settings.map_file) {


            map.0 = PathMap2d::new(ldtk.size_px().as_uvec2().into());

            if let Ok(mut cam_transform) = q_cam.get_single_mut() {
                let xy = ldtk.size_px().as_vec2() / 2.0;
                let xyz = xy.extend(cam_transform.translation.z);
                cam_transform.translation = xyz;
            }

            units.resize(map.size().as_ivec2());
            for (i, layer) in ldtk.layers().enumerate() {
                match layer {
                    MapLayer::Tiles(layer) => build_tile_layer(
                        &mut commands,
                        ldtk,
                        layer,
                        &mut atlases,
                        &mut atlas_handles,
                        i as i32,
                    ),
                    MapLayer::Entities(layer) => {
                        build_entity_layer(
                            &mut commands,
                            ldtk,
                            layer,
                            &mut atlases,
                            &mut atlas_handles,
                            i as i32,
                        );
                    }
                }
                update_colliders(&mut map, &units, layer);

                commands.spawn().insert(MapLoaded);
            }

            state.set(GameState::BattleMap).unwrap();
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
                    entity.tile_id().unwrap_or(0) as usize,
                );
                
                if !entity.tags().none() || !entity.fields().none() {
                    let tags = Tags::new(entity.tags().iter());
                    sprite.insert(entity.fields().clone());
                    sprite.insert(tags);
                } 

                sprite.insert(BattleMapEntity);

                // if entity.tags().has_all(&["player","spawner"]) {
                //     println!("Added player spawner tags to entity {:?}", sprite.id());
                // }

                if entity.tags().has("animation") {
                    let frames = entity.fields().get_str("frames");
                    let speed = entity.fields().get_f32("speed");
                    let frames: Vec<usize> = ron::de::from_str(frames).unwrap_or_else(|_|{
                        panic!("Error creating animation for {} during battle map phase, invalid frames {}",
                        entity.name(), frames);
                    });
                    let mut animator = Animator::new();

                    animator.add_animation(Animation {
                            name: entity.name().to_string(),
                            frames,
                            speed,
                        });
                    animator.play(entity.name());
                        
                    sprite.insert(animator);
                }
                if entity.tags().has("monster") {
                    sprite
                        .insert(EnemyUnit)
                        //.insert_bundle(MapUnitBundle::with_commands(&[UnitCommand::AiThink()]))
                        ;
                }

                sprite.insert(Name::new(entity.name().to_owned()));
            }
        }
    }
}

fn update_colliders(map: &mut CollisionMap, units: &MapUnits, layer: &MapLayer) {
    match layer {
        MapLayer::Tiles(layer) => {
            for tile in layer.tiles.iter() {
                if layer.has_enum(tile.id(), "collider") {
                    let xy = units.xy_to_grid(tile.pixel_xy().as_vec2());

                    //let xy = xy + map.size().as_ivec2() / 2;
                    map.set_collidable(xy);
                }
            }
        }
        MapLayer::Entities(layer) => {
            for entity in layer.entities() {
                if entity.tags().has("collider") {
                    let xy = units.xy_to_grid(entity.pixel_xy().as_vec2());
                    map.set_collidable(xy);
                }
            }
        }
    }
}

fn update_map_units(
    mut units: ResMut<MapUnits>,
    q_units: Query<(Entity, &Transform), (With<MapUnit>, With<PlayerUnit>)>,
) {
    units.clear();

    // //println!("Count: {}", q_units.iter().count());
    for (entity, transform) in q_units.iter() {
        //println!("Inserting {:?} at {}", entity, transform.translation.xy());
        ///let i = units.xy_to_index(transform.translation.xy());
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
