use std::string;

use bevy::{
    asset::{AssetLoader, BoxedFuture, LoadedAsset},
    prelude::*,
    reflect::TypeUuid, utils::HashMap,
};
use serde::{Deserialize, Serialize};
use crate::GameState;

pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_asset::<ConfigAsset>()
        .add_asset::<ConfigAsset2>()
        .add_asset_loader(ConfigAssetLoader);
    }
}

#[derive(Default, Deserialize, Serialize, Clone, Debug)]
pub struct GameSettings {
    #[serde(default)]
    pub begin_state: GameState,
    #[serde(default)]
    pub map_move_speed: f32,
    #[serde(default)]
    pub map_move_wait: f32,
    #[serde(default)]
    pub map_file: String,
}


#[derive(TypeUuid)]
#[uuid = "dc21ad42-5111-4aba-578f-11c412aaa0eb"]
pub struct ConfigAsset {
    pub settings: GameSettings,
}

#[derive(TypeUuid, Default, Deserialize, Serialize, Clone, Debug)]
#[uuid = "ac20ad32-5191-4aba-478f-10c412aaa0eb"]
pub struct ConfigAsset2 {
    pub prefab_string: String,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct ConfigAssetLoader;

impl AssetLoader for ConfigAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> BoxedFuture<'a, anyhow::Result<()>> {
        Box::pin(async move {
            // println!("Loading config");
            // let string = std::str::from_utf8(bytes)?;
            // let mut chars = string.chars();
            // if let Some(i) = chars.position(|c|c=='(') {
            //     let header = &string[0..i];
            //     let content = &string[i..];
            //     let asset = LoadedAsset::new(ConfigAsset2 {
            //         prefab_string: content.to_string(),
            //     });
            //     println!("Setting config asset for {}", header);
            //     load_context.set_labeled_asset(header, asset);
            // } else {
            //     panic!("Error loading config asset {}, couldn't find header", 
            //         load_context.path().to_str().unwrap());
            // }

            let settings: GameSettings = ron::de::from_bytes(bytes).unwrap();

            let asset = LoadedAsset::new(ConfigAsset { settings });

            load_context.set_default_asset(asset);

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["config"]
    }
}
