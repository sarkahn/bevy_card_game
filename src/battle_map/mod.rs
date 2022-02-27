use bevy::prelude::*;
use bevy_tiled_camera::TiledProjection;
use serde::{Deserialize, Serialize};

use crate::{GameState, ldtk_loader::{LoadLdtkMap, LdtkMapBuilt}, grid::*, config::{GameSettings, ConfigAsset}, SETTINGS_PATH};

use self::{
    input::InputPlugin, map::MapPlugin, states::BattleMapSelectionPlugin,
    units::UnitsPlugin,
};

mod components;
mod input;
mod map;
mod render;
mod states;
mod units;

pub use components::*;
pub use map::{Map, MapUnits};

pub struct BattleMapPlugin;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum BattleMapState {
    Inactive,
    BuildingMap,
    EnemyTurn,
    SelectUnit,
    ChooseTarget,
    UnitMoving,
}

impl Plugin for BattleMapPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_state(BattleMapState::Inactive)
            .add_plugin(UnitsPlugin)
            //.add_plugin(RenderPlugin)
            .add_plugin(BattleMapSelectionPlugin)
            .add_plugin(InputPlugin)
            .add_plugin(MapPlugin)
            .add_system_set(
                SystemSet::on_update(GameState::LoadBattleMap)
                .with_system(setup)
            ).add_system_set(
                SystemSet::on_update(GameState::BattleMap)
                .with_system(build_map)
            );
    }
}

fn setup(
    configs: Res<Assets<ConfigAsset>>,
    mut ev_writer: EventWriter<LoadLdtkMap>,
    mut state: ResMut<State<GameState>>,
) {
    if let Some(config) = configs.get(SETTINGS_PATH) {
        state.set(GameState::BattleMap).unwrap();
        ev_writer.send(LoadLdtkMap::from_path(config.settings.map_file.to_owned()));
    }
}

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
) {
    for ev in ev_reader.iter() {
        let map = &ev.0;
        let axis_offset = axis_offset(map.size);
        if let Ok(mut cam) = q_cam.get_single_mut() {
            cam.pixels_per_tile = map.tile_size.y as u32;
            cam.set_tile_count(map.size.as_uvec2().into());
        }
        for (depth, layer) in map.layers.iter().rev().enumerate() {
            let atlas = &layer.atlas;
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
                commands.spawn_bundle(sprite);
            }
        }
    }
}
