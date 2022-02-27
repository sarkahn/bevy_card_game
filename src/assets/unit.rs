use bevy::utils::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Default,Debug,Serialize,Deserialize)]
pub struct AnimationData {
    pub speed: f32,
    pub frames: Vec<i32>,
}

#[derive(Default,Debug,Serialize,Deserialize)]
pub struct SpriteData {
    image: String,
    index: i32,
    #[serde(default)]
    animations: Option<HashMap<String,AnimationData>>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Stats {
    max_hp: i32,
    #[serde(default)]
    current_hp: i32,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct UnitComponents {
    #[serde(default)]
    stats: Option<Stats>,
    #[serde(default)]
    abilities: Option<Vec<String>>,
    arena_sprite: SpriteData,
    map_sprite: SpriteData,
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
                arena_sprite: (
                    image: \"a.png\",
                    index: 0,
                ),
                map_sprite: (
                    image: \"a.png\",
                    index: 1,
                ),
            )"
        ).unwrap();
    }

    #[test]
    fn omit_anim() {
        let _: SpriteData = ron::de::from_str(
            "
            #![enable(implicit_some)]
            (
                image: \"sheet.png\",
                index: 2,
            )",
            
        ).unwrap();
    }

    #[test]
    fn from_file() {
        let file = fs::read_to_string("assets/units/guy.prefab").unwrap();
        let unit: UnitComponents = ron::de::from_str(&file).unwrap();

        let anims = unit.arena_sprite.animations.unwrap();
        assert!(anims.contains_key("idle"));

        let anims = unit.map_sprite.animations.unwrap();
        assert!(anims.contains_key("attacking"));
    }
}