use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{party::PartyUnit, ArenaSpriteVisibility};

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


#[derive(Component, Debug, Default)]
pub struct ArenaUnit;


#[derive(Component)]
pub struct Player;


#[derive(Component)]
pub struct Enemy;


#[derive(Component)]
pub struct SetPosition(pub Vec3);

pub struct UnitsPlugin;

impl Plugin for UnitsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(arena_sprite_visibility)
        .add_system(set_position)
        ;
    }
}

fn arena_sprite_visibility(
    mut commands: Commands,
    q_unit: Query<(Entity,&PartyUnit, &ArenaSpriteVisibility)>,
    mut q_vis: Query<&mut Visibility>,
) {
    for (entity, unit, visibility) in q_unit.iter() {
        println!("Found show arena sprite unit!");
        if let Ok(mut vis) = q_vis.get_mut(unit.arena_sprite) {
            println!("Setting visibility to {:?}", visibility.0);
            vis.is_visible = visibility.0;
            commands.entity(entity).remove::<ArenaSpriteVisibility>();
        }
    }
}


fn set_position(
    mut commands: Commands,
    mut q_pos: Query<(Entity, &SetPosition, &mut Transform)>,
) {
    for (entity, set,mut t) in q_pos.iter_mut() {
        t.translation = set.0;
        commands.entity(entity).remove::<SetPosition>();
    }
}