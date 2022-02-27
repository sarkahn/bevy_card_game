use bevy::{
    asset::{AssetLoader, BoxedFuture, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
};

#[derive(Default)]
pub struct PrefabLoaded {
    pub content: String,
}

pub struct GameAssetsPlugin;

impl Plugin for GameAssetsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_asset::<PrefabAsset>()
            .add_asset_loader(PrefabAssetLoader)
            //.add_system(load_event)
            ;
    }
}

#[derive(TypeUuid)]
#[uuid = "dc21ad52-5293-4abe-578f-12c412aaa0eb"]
pub struct PrefabAsset {
    pub string: String,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct PrefabAssetLoader;

impl AssetLoader for PrefabAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> BoxedFuture<'a, anyhow::Result<()>> {
        Box::pin(async move {
            let str = std::str::from_utf8(bytes)?;

            let asset = LoadedAsset::new(PrefabAsset {
                string: str.to_string(),
            });

            load_context.set_default_asset(asset);

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["prefab"]
    }
}

// fn load_event(
//     prefabs: Res<Assets<Prefab>>,
//     mut ev_config: EventReader<AssetEvent<Prefab>>,
//     mut ev_loaded: EventWriter<PrefabLoaded>,
// ) {
//     for ev in ev_config.iter() {
//         match ev {
//             AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
//                 // let prefab = prefabs.get(handle).unwrap();
//                 // ev_loaded.send(PrefabLoaded {
//                 //     content: prefab.prefab_string.clone(),
//                 // });
//             }
//             _ => {}
//         }
//     }
// }