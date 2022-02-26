// use bevy::{
//     asset::{AssetLoader, BoxedFuture, LoadedAsset},
//     prelude::*,
//     reflect::TypeUuid,
// };

// use super::config::GameSettings;

// pub struct GameAssetsPlugin;

// impl Plugin for GameAssetsPlugin {
//     fn build(&self, app: &mut bevy::prelude::App) {
//         app.add_asset::<Prefab>()
//             .add_asset_loader(PrefabAssetLoader)
//     }
// }

// #[derive(TypeUuid)]
// #[uuid = "dc21ad52-5293-4abe-578f-12c412aaa0eb"]
// pub struct Prefab {
//     pub prefab_string: String,
// }

// #[derive(Copy, Clone, Debug, Default)]
// pub struct PrefabAssetLoader;

// impl AssetLoader for PrefabAssetLoader {
//     fn load<'a>(
//         &'a self,
//         bytes: &'a [u8],
//         load_context: &'a mut bevy::asset::LoadContext,
//     ) -> BoxedFuture<'a, anyhow::Result<()>> {
//         Box::pin(async move {
//             let str = std::str::from_utf8(bytes)?;

//             let asset = LoadedAsset::new(Prefab {
//                 prefab_string: str.to_string(),
//             });

//             load_context.set_default_asset(asset);

//             Ok(())
//         })
//     }

//     fn extensions(&self) -> &[&str] {
//         &["prefab"]
//     }
// }

