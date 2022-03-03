use bevy::{
    asset::{AssetLoader, AssetPath, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    utils::HashMap,
};
use bevy_ascii_terminal::ldtk::LdtkAsset;
use serde::{Deserialize, Serialize};

use crate::AnimationData;

pub struct UnitAssetPlugin;

impl Plugin for UnitAssetPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_asset::<LoadUnitPrefab>()
            .add_asset_loader(UnitAssetLoader);
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct SpriteData {
    pub ldtk_file: String,
    pub tileset_name: String,
    pub index: i32,
    #[serde(default)]
    pub animations: Option<HashMap<String, AnimationData>>,
    #[serde(default)]
    pub default_animation: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Stats {
    pub max_hp: i32,
    #[serde(default)]
    pub current_hp: i32,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct UnitComponents {
    #[serde(default)]
    pub stats: Option<Stats>,
    #[serde(default)]
    pub abilities: Option<Vec<String>>,
    pub sprite_data: SpriteData,
}

#[cfg(test)]
mod test {
    use std::fs;

    use super::*;

    #[test]
    fn minimal_components() {
        let _: UnitComponents = ron::de::from_str(
            "
            #![enable(implicit_some)]
            (
                sprite_data: (
                    ldtk_file: \"poop.ldtk\",
                    tileset_name: \"Whoa\",
                    index: 0,
                ),
            )",
        )
        .unwrap();
    }

    #[test]
    fn omit_anim() {
        let _: SpriteData = ron::de::from_str(
            "
            #![enable(implicit_some)]
            (
                ldtk_file: \"bevy.ldtk\",
                tileset_name: \"hi\",
                index: 2,
            )",
        )
        .unwrap();
    }

    #[test]
    fn from_file() {
        let file = fs::read_to_string("assets/units/guy.unit").unwrap();
        let unit: UnitComponents = ron::de::from_str(&file).unwrap();

        let anims = unit.sprite_data.animations.unwrap();
        assert!(anims.contains_key("idle"));
    }
}

#[derive(TypeUuid)]
#[uuid = "da21ab52-5193-3abe-478f-10c412aaa0eb"]
pub struct LoadUnitPrefab {
    pub components: UnitComponents,
    pub ldtk_file: Handle<LdtkAsset>,
}

struct UnitAssetLoader;
impl AssetLoader for UnitAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::asset::BoxedFuture<'a, anyhow::Result<(), anyhow::Error>> {
        Box::pin(async move {
            let str = std::str::from_utf8(bytes)?;
            let components: UnitComponents = ron::de::from_str(str)?;

            let root = load_context
                .path()
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .clone();

            let ldtk_path: AssetPath = root.join(&components.sprite_data.ldtk_file).into();

            let arena_ldtk_handle: Handle<LdtkAsset> = load_context.get_handle(ldtk_path.clone());

            let asset = LoadedAsset::new(LoadUnitPrefab {
                components,
                ldtk_file: arena_ldtk_handle.clone(),
            });

            load_context.set_default_asset(asset.with_dependency(ldtk_path));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["unit"]
    }
}
