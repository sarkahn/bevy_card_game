use bevy::prelude::*;

#[derive(Component)]
pub struct Combatant;

// use crate::{GameState, ldtk_loader::LdtkMap, LoadUnitPrefab};

// use super::ArenaState;

// pub struct ArenaUnitsPlugin;

// impl Plugin for ArenaUnitsPlugin {
//     fn build(&self, app: &mut App) {
//     //     app.add_system_set(SystemSet::on_enter(GameState::LoadArena)
//     //     .with_system(setup)
//     //     )
//     //     .add_system_set(SystemSet::on_update(GameState::LoadArena)
//     //     .with_system(build)
//     // );
//     }
// }

// // #[derive(Component, Debug, Default)]
// // pub struct ArenaUnit;

// // fn setup(
// //     mut commands: Commands,
// // ) {
// //     commands.spawn().insert(LoadPrefab("units_wizard.ldtk".to_string()));
// // }

// // fn build(
// //     ldtk: Res<Assets<LdtkMap>>,
// // ) {

// // }
