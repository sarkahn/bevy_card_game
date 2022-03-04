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
        app.add_system(animate).add_system(process_commands);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnimationData {
    #[serde(default)]
    pub name: String,
    pub frames: Vec<usize>,
    pub speed: f32,
    pub tileset_path: String,
    pub ldtk_name: String,
}

#[derive(Component, Default)]
pub struct AnimationController {
    frame_index: usize,
    current: Option<String>,
    animations: HashMap<String, AnimationData>,
    paused: bool,
    timer: Timer,
    change: Option<String>,
}

impl From<AnimationData> for AnimationController {
    fn from(d: AnimationData) -> Self {
        let speed = d.speed;
        let mut animations = HashMap::default();
        let name = d.name.to_string();
        animations.insert(d.name.to_lowercase(), d);
        let mut c = Self {
            animations,
            timer: Timer::from_seconds(speed, false),
            ..Default::default()
        };
        c.play(&name);
        c
    }
}

impl AnimationController {
    pub fn play(&mut self, name: &str) {
        // let name = name.to_lowercase();
        // if let Some(anim) = self.animations.get(&name) {
        //     //println!("Playing animation {}. Speed {}. Path {}", name, anim.speed, &anim.tileset_path);
        //     self.change = Some(anim.tileset_path.to_string());
        //     self.current = Some(name.to_string());
        //     self.timer.set_duration(Duration::from_secs_f32(anim.speed));
        //     self.timer.set_repeating(true);
        //     self.timer.reset();
        //     self.frame_index = 0;
        //     self.paused = false;
        // } else {
        //     warn!(
        //         "Attemping to play anim {}, but it wasn't in controller's animations",
        //         name
        //     );
        // }
    }

    pub fn play_any(&mut self) {
        // let k = self.animations.iter().next();
        // if k.is_none() {
        //     return;
        // }
        // let k = k.unwrap().0.clone();
        // self.play(&k);
    }

    pub fn add(&mut self, name: &str, anim: AnimationData) {
        let name = name.to_lowercase();
        //let anim_count = self.animations.len();
        self.animations.insert(name, anim);
        // if anim_count == 0 {
        //     self.play(&name);
        // }
    }

    pub fn current_anim(&self) -> Option<&AnimationData> {
        if let Some(name) = &self.current {
            return self.animations.get(&name.to_lowercase());
        }
        None
    }
}

fn animate(
    time: Res<Time>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
    mut atlas_handles: ResMut<AtlasHandles>,
    mut q_units: Query<(
        Entity,
        &mut AnimationController,
        &mut TextureAtlasSprite,
        &mut Handle<TextureAtlas>,
    )>,
    ldtk: Res<Assets<LdtkMap>>,
) {
    // for (entity, mut controller, mut sprite, mut atlas) in q_units.iter_mut() {
    //     if controller.paused || controller.current.is_none() {
    //         continue;
    //     }

    //     if let Some(new) = &controller.change {
    //         let current = controller.current_anim().unwrap();
    //         let handle: Handle<LdtkMap> = asset_server.load(&current.ldtk_name);
    //         commands.entity(entity).insert(handle.clone());
    //         if let Some(ldtk) = ldtk.get(handle) {
    //             //println!("Found handle...");
    //             let tileset = ldtk
    //                 .tileset_from_path(&current.tileset_path)
    //                 .unwrap_or_else(|| {
    //                     panic!("Error switching sprites, couldn't find tileset {}", new)
    //                 });
    //             *atlas = get_atlas(&mut atlases, &mut atlas_handles, &tileset);
    //             sprite.index = 0;
    //             //println!("Changing to tileset {}", new);
    //             controller.change = None;
    //         } else {
    //             continue;
    //         }
    //     }

    //     let len = controller.current_anim().unwrap().frames.len();
    //     controller.timer.tick(time.delta());
    //     if controller.timer.just_finished() {
    //         //println!("Should be animating?");
    //         controller.timer.reset();
    //         controller.frame_index = (controller.frame_index + 1) % len;
    //         let index = controller.frame_index;
    //         let frames = &controller.current_anim().unwrap().frames;
    //         sprite.index = frames[index];
    //     }
    // }
}

// fn get_atlas(
//     atlases: &mut Assets<TextureAtlas>,
//     atlas_handles: &mut AtlasHandles,
//     tileset: &MapTileset,
// ) -> Handle<TextureAtlas> {
//     let name = &tileset.name;
//     match atlas_handles.0.get(name) {
//         Some(atlas) => atlas.clone(),
//         None => {
//             let atlas = TextureAtlas::from_grid(
//                 tileset.image.clone(),
//                 IVec2::splat(tileset.tile_size).as_vec2(),
//                 tileset.tile_count.x as usize,
//                 tileset.tile_count.y as usize,
//             );
//             let handle = atlases.add(atlas);
//             atlas_handles.0.insert(name.to_string(), handle.clone());
//             handle
//         }
//     }
// }

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
            println!("Playing animation {name}: {:?}", anim);
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
