use std::slice::Iter;

use bevy::{math::Vec3Swizzles, prelude::*};
use bevy_ascii_terminal::Point2d;
use bevy_tiled_camera::TiledProjection;
use sark_grids::Grid;
use sark_pathfinding::PathMap2d;

use crate::{
    battle_map::units::PlayerBase,
    config::{ConfigAsset, GameSettings},
    ldtk_loader::{EntitiesLayer, LdtkMap, PrefabEntity, MapLayer, MapTile, MapTileset, TilesLayer},
    make_sprite_atlas, AnimationController, AnimationData, AtlasHandles, GameState, GridHelper,
    SETTINGS_PATH, TILE_SIZE,
};

use super::{
    enemies::Spawner,
    units::{EnemyUnit, MapUnit, MapUnitBundle, PlayerUnit, UnitCommand},
};

pub struct BattleMapSetupPlugin;

impl Plugin for BattleMapSetupPlugin {
    fn build(&self, app: &mut App) {
        
    }
}
