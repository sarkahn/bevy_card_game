use std::time::Duration;

use bevy::{prelude::*, utils::HashMap};
use serde::{Deserialize, Serialize};

use crate::{AtlasHandles, ldtk_loader::{MapTileset, LdtkMap}};

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(animate);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnimationData {
    #[serde(default)]
    pub name: String,
    pub frames: Vec<usize>,
    pub speed: f32,
    pub tileset_path: String,
    pub ldtk_path: String,
}

#[derive(Component, Default)]
pub struct AnimationController {
    frame_index: usize,
    current: Option<String>,
    animations: HashMap<String, AnimationData>,
    paused: bool,
    timer: Timer,
    change: Option<String>,
    map: Handle<LdtkMap>,
}

impl AnimationController {
    pub fn play(&mut self, name: &str) {
        if let Some(anim) = self.animations.get(name) {
            println!("Playing animation {}. Speed {}. Path {}", name, anim.speed, &anim.tileset_path);
            self.change = Some(anim.tileset_path.to_string());
            self.current = Some(name.to_string());
            self.timer.set_duration(Duration::from_secs_f32(anim.speed));
            self.timer.set_repeating(true);
            self.timer.reset();
            self.frame_index = 0;
            self.paused = false;
        } else {
            //println!("Animation {} not found", name);
        }
    }

    pub fn add(&mut self, name: &str, anim: AnimationData) {
        let name = name.to_lowercase();
        let anim_count = self.animations.len();
        self.animations.insert(name.clone(), anim);
        if anim_count == 0 {
            self.play(&name);
        }
    }

    pub fn current_anim(&self) -> Option<&AnimationData> {
        if let Some(name) = &self.current {
            return self.animations.get(name);
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
    mut q_units: Query<(Entity, &mut AnimationController, &mut TextureAtlasSprite, &mut Handle<TextureAtlas>)>,
    ldtk: Res<Assets<LdtkMap>>,
) {
    for (entity, mut controller, mut sprite, mut atlas) in q_units.iter_mut() {
        if controller.paused || controller.current.is_none() {
            continue;
        }

        if let Some(new) = &controller.change {
            let current = controller.current_anim().unwrap();
            let handle: Handle<LdtkMap> = asset_server.load(&current.ldtk_path);
            commands.entity(entity).insert(handle.clone());
            if let Some(ldtk) = ldtk.get(handle) {
                println!("Found handle...");
                let tileset = ldtk.tileset_from_path(&current.tileset_path).unwrap_or_else(||
                    panic!("Error switching sprites, couldn't find tileset {}", new)
                );
                *atlas = get_atlas(&mut atlases, &mut atlas_handles, &tileset);
                println!("Changing to tileset {}", new);
                controller.change = None;
            } else {
                continue;
            }
        } 

        let len = controller.current_anim().unwrap().frames.len();
        controller.timer.tick(time.delta());
        if controller.timer.just_finished() {
            //println!("Should be animating?");
            controller.timer.reset();
            controller.frame_index = (controller.frame_index + 1) % len;
            let index = controller.frame_index;
            let frames = &controller.current_anim().unwrap().frames;
            sprite.index = frames[index];
        }
    }
}


fn get_atlas(
    atlases: &mut Assets<TextureAtlas>,
    atlas_handles: &mut AtlasHandles,
    tileset: &MapTileset,
) -> Handle<TextureAtlas> {
    let name = &tileset.name;
    match atlas_handles.0.get(name) {
        Some(atlas) => atlas.clone(),
        None => {
            let atlas = TextureAtlas::from_grid(
                tileset.image.clone(),
                IVec2::splat(tileset.tile_size).as_vec2(),
                tileset.tile_count.x as usize,
                tileset.tile_count.y as usize,
            );
            let handle = atlases.add(atlas);
            atlas_handles.0.insert(name.to_string(), handle.clone());
            handle
        }
    }
}