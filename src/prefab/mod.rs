use bevy::prelude::*;

use crate::{config::ConfigAsset, GameState, SETTINGS_PATH};

mod card;
mod unit;

pub struct PrefabPlugin;

impl Plugin for PrefabPlugin {
    fn build(&self, app: &mut App) {
        app.add_state(PrefabState::Loading)
            .add_plugin(unit::PrefabPlugin)
            .add_plugin(card::PrefabPlugin)
            .add_system_set(SystemSet::on_enter(GameState::AssetTest).with_system(test));
    }
}

fn test(mut commands: Commands, config: Res<Assets<ConfigAsset>>) {
    let config = config.get(SETTINGS_PATH).unwrap();
    //println!("SPAWNING SPAWN SPAWN");
    commands
        .spawn()
        .insert(LoadUnitPrefab::from_name(&config.settings.asset_test_file));
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
enum PrefabState {
    Loading,
    Loaded,
}

#[derive(Component, Default, Clone, Debug)]
pub struct LoadUnitPrefab {
    pub path: String,
    pub xy: IVec2,
    pub depth: i32,
    pub change_sprite: Option<ChangeSprite>,
}

#[derive(Component, Default, Clone, Debug)]
pub struct LoadCardPrefab {
    pub path: String,
    pub xy: IVec2,
    pub depth: i32,
    pub atlas: Handle<TextureAtlas>,
    pub tile_id: i32,
    pub size: IVec2,
}

#[derive(Default, Clone, Debug)]
pub struct ChangeSprite {
    pub atlas: Handle<TextureAtlas>,
    pub tile_id: i32,
}

impl LoadUnitPrefab {
    pub fn from_name(name: &str) -> Self {
        Self {
            path: name.to_string(),
            ..Default::default()
        }
    }
}
