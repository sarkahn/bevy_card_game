use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub const UNITS_DEF_FILE: &str = "units.ldtk";

#[derive(Component, Serialize, Deserialize, Default)]
pub struct Stats {
    hp: i32,
    max_hp: i32,
    strength: i32,
    defense: i32,
    #[serde(default)]
    affinity: Option<Affinity>,
    #[serde(default)]
    resistance: Option<Resistance>,
}

#[derive(Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Affinity {
    element: Element,
    value: f32,
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq)]
pub struct Resistance {
    element: Element,
    value: f32,
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq)]
pub enum Element {
    Fire,
    Lightning,
    Death,
    Holy,
}
impl Element {
    pub fn get_sprite_id(&self) -> usize {
        match self {
            Element::Fire => 0,
            Element::Lightning => 1,
            Element::Death => 2,
            Element::Holy => 3,
        }
    }
}
