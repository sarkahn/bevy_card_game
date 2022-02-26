use bevy::{
    asset::{AssetLoader, BoxedFuture, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
};
use serde::{Deserialize, Serialize};
use crate::GameState;

pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<ConfigAsset>()
        .add_asset_loader(ConfigAssetLoader);
    }
}

#[derive(Default, Deserialize, Serialize, Copy, Clone, Debug)]
pub struct GameSettings {
    #[serde(default)]
    pub begin_state: GameState,
    #[serde(default)]
    pub map_move_speed: f32,
    #[serde(default)]
    pub map_move_wait: f32,
}


#[derive(TypeUuid)]
#[uuid = "dc21ad42-5111-4aba-578f-11c412aaa0eb"]
pub struct ConfigAsset {
    pub settings: GameSettings,
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