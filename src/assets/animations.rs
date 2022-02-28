use bevy::{
    asset::{AssetLoader, BoxedFuture, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
};
use serde::{Deserialize, Serialize};

pub struct GameAssetsPlugin;

impl Plugin for GameAssetsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_asset::<AnimationAsset>()
            .add_asset_loader(AnimationAssetLoader);
    }
}

#[derive(TypeUuid, Serialize, Deserialize, Debug, Default, Clone)]
#[uuid = "dc11ad52-5193-1abe-111f-13c411aaa0eb"]
pub struct AnimationAsset {
    frames: Vec<usize>,
    speed: f32,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct AnimationAssetLoader;

impl AssetLoader for AnimationAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> BoxedFuture<'a, anyhow::Result<()>> {
        Box::pin(async move {
            let asset: AnimationAsset = ron::de::from_bytes(bytes)?;

            let asset = LoadedAsset::new(asset);

            load_context.set_default_asset(asset);

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["anim"]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_file() {
        let string = "(frames:[1,2,3,4],speed:0.5)";
        let _: AnimationAsset = ron::de::from_str(string).expect("Error loading animation");
    }
}
