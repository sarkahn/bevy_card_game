use bevy::prelude::*;

use crate::{GameState, config::ConfigAsset, SETTINGS_PATH};

mod prefab;


#[derive(Component, Default, Clone, Debug)]
pub struct LoadPrefab {
    pub path: String,
    pub xy: IVec2,
    pub depth: i32,
    pub texture: Option<Handle<Image>>,
}


impl LoadPrefab {
    pub fn from_name(name: &str) -> Self {
        Self {
            path: name.to_string(),
            ..Default::default()
        }
    }
}

pub struct PrefabPlugin;

impl Plugin for PrefabPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(prefab::PrefabPlugin)
            .add_system_set(
                SystemSet::on_enter(GameState::AssetTest)
                .with_system(test)
            )
            ;
    }
}

fn test(
    mut commands: Commands,
    config: Res<Assets<ConfigAsset>>,
) {
    let config = config.get(SETTINGS_PATH).unwrap();
    //println!("SPAWNING SPAWN SPAWN");
    commands.spawn().insert(LoadPrefab::from_name(&config.settings.asset_test_file));
}
