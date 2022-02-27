use bevy::{
    asset::{AssetLoader, AssetPath, BoxedFuture, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    utils::HashMap,
};
use ldtk_rust::Project;

pub(crate) struct LdtkAssetPlugin;

impl Plugin for LdtkAssetPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset_loader(LdtkAssetLoader)
            .add_asset::<LdtkAsset>();
    }
}

#[derive(TypeUuid)]
#[uuid = "ac23ab52-5393-4bbe-178f-16c414aaa0eb"]
pub struct LdtkAsset {
    pub project: ldtk_rust::Project,
    pub tilesets: HashMap<i64, Handle<Image>>,
}

#[derive(Copy, Clone, Debug, Default)]
struct LdtkAssetLoader;

impl AssetLoader for LdtkAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let project: Project = serde_json::from_slice(bytes)?;
            let tilesets: Vec<(i64, AssetPath)> = project
                .defs
                .tilesets
                .iter()
                .map(|tileset| {
                    (
                        tileset.uid,
                        load_context
                            .path()
                            .parent()
                            .unwrap()
                            .join(tileset.rel_path.clone())
                            .into(),
                    )
                })
                .collect();

            let loaded_asset = LoadedAsset::new(LdtkAsset {
                project,
                tilesets: tilesets
                    .iter()
                    .map(|dep| (dep.0, load_context.get_handle(dep.1.clone())))
                    .collect(),
            });
            load_context.set_default_asset(
                loaded_asset.with_dependencies(tilesets.iter().map(|x| x.1.clone()).collect()),
            );
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["ldtk"]
    }
}
