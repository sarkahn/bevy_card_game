
use std::ops::Range;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum AbilityEffect {
    None,
    DealsDamage(Range<i32>),
}
impl Default for AbilityEffect {
    fn default() -> Self {
        AbilityEffect::None
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Ability {
    name: String,
    //effects: Vec<AbilityEffect>,
}
