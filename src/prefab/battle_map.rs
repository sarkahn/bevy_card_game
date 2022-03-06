use bevy::prelude::*;

use crate::{unit::MapUnit, ldtk_loader::LdtkMap, GameState, AtlasHandles, animation::Animator, SpawnPrefabOld};

pub struct MapUnitPlugin;

impl Plugin for MapUnitPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(build_unit)
        ;
    }
}

fn build_unit(
    mut commands: Commands,
    mut atlas_handles: ResMut<AtlasHandles>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
    mut q_animator: Query<&mut Animator>, 
    ldtk: Res<Assets<LdtkMap>>,
    q_prefabs: Query<(Entity, &SpawnPrefabOld), With<MapUnit>>,
) {
    for (entity, load) in q_prefabs.iter() {
        if let Some(ldtk) = ldtk.get(&load.path) {

        }
    }
}