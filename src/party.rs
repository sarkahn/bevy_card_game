use bevy::prelude::*;

use crate::ldtk_loader::LdtkMap;



pub struct PartyPlugin;

impl Plugin for PartyPlugin {
    fn build(&self, app: &mut App) {
        
    }
}

#[derive(Component)]
pub struct GenerateParty;

fn generate_party(
    mut commands: Commands,
    ldtk: Res<Assets<LdtkMap>>,
    q_gen: Query<&GenerateParty>,
) {
}