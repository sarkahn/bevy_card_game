use bevy::prelude::*;

use crate::{unit::MapUnit, ldtk_loader::{LdtkMap, Tags, Fields}, GameState, AtlasHandles, animation::Animator, SETTINGS_PATH, config::ConfigAsset, SpawnPrefabOld, LDTK_ARCHER_PATH};

use super::{units::PlayerUnit, spawn::Spawner, map::BattleMapEntity};

//use super::Spawner;

pub struct BattleMapPlayerPlugin;

impl Plugin for BattleMapPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(GameState::LoadBattleMap)
            .with_system(setup)
        )
        ;
    }
}

#[derive(Component)]
pub struct BattleMapPlayerLoaded;

fn setup(
    mut commands: Commands,
    q_tags: Query<(Entity, &Tags, &Fields, &Name), (Added<Tags>, With<BattleMapEntity>)>
) {
    for (entity, tags, fields, name) in q_tags.iter() {
        if tags.has_all(&["player", "spawner"]) {
            if let Some(delay) = fields.try_get_f32("spawn_delay") {
                assert!(delay > 0.0, "Spawn delay for {} is at or below zero!", name.as_str());
                commands.entity(entity).insert(
                    Spawner::new(
                        Timer::from_seconds(delay, true),
                        &[LDTK_ARCHER_PATH],
                    )
                );
            }

        }
    }
}

// fn spawn_player_units(
//     mut q_spawner: Query<&mut Spawner, With<Spawner>>,
// ) {
//     for spawner in q_spawner.iter() {

//     }
// }