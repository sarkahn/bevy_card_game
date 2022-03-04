use bevy::prelude::*;

use crate::{
    animation::{AnimationCommand, Animator},
    GameState,
};

use super::TakingATurn;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_update(GameState::Arena).with_system(do_turn));
    }
}

fn do_turn(
    mut commands: Commands,
    mut q_units: Query<(Entity, &mut Animator, &Transform), With<TakingATurn>>,
) {
    for (entity, mut driver, transform) in q_units.iter_mut() {
        //println!("TAKING A TURN");
        //commands.entity(entity).remove::<TakingATurn>();
        //driver.add_wait(3.0).add_move(-5.0, -3.0, 3.0).add_wait(3.0).add_move(3.5, 0.0, 5.0);
    }
}
