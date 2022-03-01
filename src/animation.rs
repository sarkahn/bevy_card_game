use std::time::Duration;

use bevy::{prelude::*, utils::HashMap};
use serde::{Deserialize, Serialize};

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(animate);
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct UnitAnimation {
    frames: Vec<usize>,
    speed: f32,
}

#[derive(Component, Default)]
pub struct AnimationController {
    frame_index: usize,
    current: Option<String>,
    animations: HashMap<String, UnitAnimation>,
    paused: bool,
    timer: Timer,
}

impl AnimationController {

    pub fn play(&mut self, name: &str) {
        if let Some(anim) = self.animations.get(name) {
            self.current = Some(name.to_string());
            self.timer.set_duration(Duration::from_secs_f32(anim.speed));
            self.timer.reset();
            self.frame_index = 0;
        } else {
            //println!("Animation {} not found", name);
        }
    }

    pub fn add(&mut self, name: &str, anim: UnitAnimation) {
        self.animations.insert(name.to_string(), anim);
    }

    pub fn current_anim(&self) -> Option<&UnitAnimation> {
        if let Some(name) = &self.current {
            return self.animations.get(name);
        }
        None
    }
}

fn animate(
    time: Res<Time>,
    mut q_units: Query<(&mut AnimationController, &mut TextureAtlasSprite)>,
) {
    for (mut controller, mut sprite) in q_units.iter_mut() {
        if controller.paused || controller.current.is_none() {
            return;
        }
        let len = controller.current_anim().unwrap().frames.len();
        controller.timer.tick(time.delta());
        if controller.timer.just_finished() {
            //println!("Should tick?");
            controller.timer.reset();
            controller.frame_index = (controller.frame_index + 1) % len;
            let index = controller.frame_index;
            let frames = &controller.current_anim().unwrap().frames;
            sprite.index = frames[index];
        }
    }
}
