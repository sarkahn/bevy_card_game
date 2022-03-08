use std::{borrow::Cow, collections::VecDeque, thread::current, time::Duration};

use bevy::{math::Vec3Swizzles, prelude::*, utils::HashMap};
use serde::{Deserialize, Serialize};

use crate::{
    ldtk_loader::{LdtkMap, MapTileset},
    AtlasHandles, TILE_SIZE,
};

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(process_commands);
    }
}


#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Animation {
    pub name: String,
    pub frames: Vec<usize>,
    pub speed: f32,
}

impl Animation {
    pub fn new_state(&self) -> AnimationState {
        AnimationState {
            timer: Timer::from_seconds(self.speed, true),
            frames: self.frames.clone(),
            frame_index: self.frames[0],
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct AnimationState {
    timer: Timer,
    frames: Vec<usize>,
    frame_index: usize,
}

#[derive(Default, Debug, Clone)]
pub struct MoveState {
    timer: Timer,
    begin: Vec2,
    end: Vec2,
}

#[derive(Default, Debug, Clone)]
pub struct WaitState {
    timer: Timer,
}

#[derive(Clone, Debug)]
pub enum DriverState {
    MoveState(MoveState),
    WaitState(WaitState),
}

impl DriverState {
    pub fn as_move_state(&mut self) -> Option<&mut MoveState> {
        if let Self::MoveState(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_wait_state(&mut self) -> Option<&mut WaitState> {
        if let Self::WaitState(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum AnimationCommand {
    Play(String),
    Wait(f32),
    MoveBy([f32; 2], f32),
    Pause(),
    // Unpause(),
}

#[derive(Component, Default)]
pub struct Animator {
    queue: VecDeque<AnimationCommand>,
    animations: HashMap<String, Animation>,
    state: Option<DriverState>,
    playing: Option<AnimationState>,
    paused: Option<AnimationState>,
}

impl Animator {
    pub fn new() -> Self {
        Animator {
            ..Default::default()
        }
    }

    pub fn add_animation(&mut self, anim: Animation) {
        self.animations.insert(anim.name.clone(), anim);
    }

    pub fn push_cmd(&mut self, cmd: AnimationCommand) {
        self.queue.push_back(cmd);
    }

    pub fn play(&mut self, name: &str) {
        if let Some(anim) = self.animations.get(name) {
            //println!("Playing animation {name}: {:?}", anim);
            self.playing = Some(anim.new_state());
        } else {
            warn!(
                "Attempting to play animation {}, but it hasn't been added to the animator",
                name
            );
        }
    }

    pub fn cmd_move(&mut self, x: f32, y: f32, speed: f32) -> &mut Self {
        self.push_cmd(AnimationCommand::MoveBy([x, y], speed));
        self
    }

    pub fn cmd_wait(&mut self, time: f32) -> &mut Self {
        self.push_cmd(AnimationCommand::Wait(time));
        self
    }

    pub fn cmd_pause(&mut self) -> &mut Self {
        self.push_cmd(AnimationCommand::Pause());
        self
    }

    pub fn pause_animation(&mut self) -> &mut Self {
        let curr = self.playing.clone();
        self.paused = curr;
        self.playing = None;
        self
    }

    pub fn push_cmds_back(
        &mut self,
        commands: impl IntoIterator<Item = AnimationCommand>,
    ) -> &mut Self {
        self.queue.extend(commands);
        self
    }

    pub fn push_cmds_front(
        &mut self,
        commands: impl IntoIterator<Item = AnimationCommand>,
    ) -> &mut Self {
        for cmd in commands {
            self.queue.push_front(cmd);
        }
        self
    }
}

fn process_commands(
    time: Res<Time>,
    mut q_anim_commands: Query<(Entity, &mut Animator, &mut TextureAtlasSprite, &mut Transform)>,
    q_name: Query<&Name>,
) {
    let dt = time.delta();

    for (entity, mut driver, mut sprite, mut transform) in q_anim_commands.iter_mut() {
        // Don't go to the next command until our current state is complete
        if driver.state.is_none() {
            //println!("dequeing {:?}", driver.queue);
            // The next command will provide our next state, if any
            if let Some(ref mut cmd) = driver.queue.pop_front() {
                //println!("Queueing command {:?}", cmd);
                let next = match cmd {
                    AnimationCommand::Play(anim) => {
                        if let Ok(name) = q_name.get(entity) {
                            println!("{} is attempting to play animation {}", name.as_str(), anim);
                        } 
                        driver.play(anim);
                        None
                    }
                    AnimationCommand::Wait(wait) => Some(DriverState::WaitState(WaitState {
                        timer: Timer::from_seconds(*wait, false),
                    })),
                    AnimationCommand::MoveBy(rel, speed) => {
                        let rel = Vec2::from(*rel);
                        let begin = transform.translation.xy();
                        let end = begin + (rel * TILE_SIZE as f32);
                        let state = DriverState::MoveState(MoveState {
                            timer: Timer::from_seconds(*speed, false),
                            begin,
                            end,
                        });
                        Some(state)
                    }
                    AnimationCommand::Pause() => {
                        driver.pause_animation();
                        None
                    }
                };
                //println!("Changing state to {:?}", next);
                driver.state = next;
            }
        }

        //println!("Animator");
        // If we currently have a state...
        if let Some(ref mut state) = driver.state {
            //println!("Animator is running {:?}!", state);
            let is_done = match state {
                DriverState::MoveState(mv) => {
                    mv.timer.tick(dt);

                    let a = mv.begin;
                    let b = mv.end;
                    let xy = a.lerp(b, mv.timer.percent());
                    transform.translation = xy.extend(transform.translation.z);

                    mv.timer.finished()
                }
                DriverState::WaitState(wait) => wait.timer.tick(dt).finished(),
            };
            if is_done {
                driver.state = None;
            }
        }

        if let Some(ref mut anim) = driver.playing {
            anim.timer.tick(dt);

            if anim.timer.just_finished() {
                anim.frame_index = (anim.frame_index + 1) % anim.frames.len();
                sprite.index = anim.frames[anim.frame_index];
            }
        }
    }
}

struct AnimCmdParse {}
impl From<&str> for AnimationCommand {
    fn from(s: &str) -> Self {
        let result: AnimationCommand = ron::de::from_str(s).unwrap_or_else(|_| {
            panic!(
                "Error deserializeing string to animation commands. Contents: {}",
                s
            )
        });
        result
    }
}
/*
Play(String),
Wait(f32),
Move{ rel: [f32;2], speed: f32 },
Pause(), */

#[test]
fn from_str() {
    let cmd = AnimationCommand::MoveBy([1.0, 1.0], 3.0);
    let str = ron::ser::to_string(&cmd).unwrap();
    println!("{}", str);
    let str = " MoveBy((1,1), 3)";
    let cmds: AnimationCommand = ron::de::from_str(str).unwrap();
}

#[test]
fn play() {
    let str = "Play(\"AnimName\")";
    let anim: AnimationCommand = ron::de::from_str(str).unwrap();
}

fn wait(dt: Duration, wait: &mut WaitState) -> bool {
    if wait.timer.tick(dt).just_finished() {
        return false;
    }
    return true;
}
