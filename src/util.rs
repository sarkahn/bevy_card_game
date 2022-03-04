use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_egui::egui::Pos2;
use serde_json::Value;

use crate::TILE_SIZE;


pub trait GetValue {
    fn as_f32(&self) -> Option<f32>;
    fn as_i32(&self) -> Option<i32>;
    fn as_vec2(&self) -> Option<Vec2>;
}
impl GetValue for Value {
    fn as_f32(&self) -> Option<f32> {
        self.as_f64().map(|v|v as f32)
    }

    fn as_i32(&self) -> Option<i32> {
        self.as_i64().map(|v|v as i32)
    }

    fn as_vec2(&self) -> Option<Vec2> {
        if let Some(arr) = self.as_array() {
            
        }
        None
    }
}

pub fn make_sprite<'w, 's, 'a>(
    commands: &'a mut Commands<'w, 's>,
    xy: Vec2,
    depth: i32,
    color: Color,
) -> EntityCommands<'w, 's, 'a> {
    // let xy = xy * 64.0;
    let transform = Transform::from_translation(Vec3::new(xy.x, xy.y, depth as f32));
    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            color: color,
            custom_size: Some(Vec2::splat(TILE_SIZE as f32)),
            ..Default::default()
        },
        transform,
        ..Default::default()
    })
}
pub fn make_sprite_sized<'w, 's, 'a>(
    commands: &'a mut Commands<'w, 's>,
    xy: Vec2,
    depth: i32,
    color: Color,
    size: Vec2,
) -> EntityCommands<'w, 's, 'a> {
    // let xy = xy * 64.0;
    let transform = Transform::from_translation(Vec3::new(xy.x, xy.y, depth as f32));
    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            color: color,
            //custom_size: Some(size),
            ..Default::default()
        },
        transform,
        ..Default::default()
    })
}



pub fn make_sprite_image<'w, 's, 'a>(
    commands: &'a mut Commands<'w, 's>,
    xy: Vec2,
    depth: i32,
    texture: Handle<Image>,
) -> EntityCommands<'w, 's, 'a> {
    //let xy = xy * 64.0;
    let transform = Transform::from_translation(Vec3::new(xy.x, xy.y, depth as f32));
    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            //custom_size: Some(Vec2::ONE),
            ..Default::default()
        },
        texture,
        transform,
        ..Default::default()
    })
}

pub fn make_sprite_image_sized<'w, 's, 'a>(
    commands: &'a mut Commands<'w, 's>,
    xy: Vec2,
    depth: i32,
    texture: Handle<Image>,
    size: IVec2,
) -> EntityCommands<'w, 's, 'a> {
    // let xy = xy * 64.0;
    let transform = Transform::from_translation(Vec3::new(xy.x, xy.y, depth as f32));
    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(size.as_vec2()),
            ..Default::default()
        },
        texture,
        transform,
        ..Default::default()
    })
}

pub fn make_sprite_atlas<'w, 's, 'a>(
    commands: &'a mut Commands<'w, 's>,
    xy: Vec2,
    depth: i32,
    atlas: Handle<TextureAtlas>,
    index: usize,
) -> EntityCommands<'w, 's, 'a> {
    //let xy = xy * TILE_SIZE as f32;
    let sprite = TextureAtlasSprite {
        //custom_size: Some(Vec2::ONE),
        index,
        ..Default::default()
    };
    let transform = Transform::from_translation(Vec3::new(xy.x, xy.y, depth as f32));
    let sprite = SpriteSheetBundle {
        sprite,
        texture_atlas: atlas,
        transform,
        ..Default::default()
    };
    commands.spawn_bundle(sprite)
}


pub fn make_sprite_atlas_sized<'w, 's, 'a>(
    commands: &'a mut Commands<'w, 's>,
    xy: Vec2,
    size: Vec2,
    depth: i32,
    atlas: Handle<TextureAtlas>,
    index: usize,
) -> EntityCommands<'w, 's, 'a> {
    // let xy = xy * 64.0;
    let transform = Transform::from_translation(Vec3::new(xy.x, xy.y, depth as f32));
    let sprite = TextureAtlasSprite {
        //custom_size: Some(size),
        index,
        ..Default::default()
    };
    let sprite = SpriteSheetBundle {
        sprite,
        texture_atlas: atlas,
        transform,
        ..Default::default()
    };
    commands.spawn_bundle(sprite)
}



pub trait ToEguiPos {
    fn egui(&self) -> Pos2;
}

impl ToEguiPos for IVec2 {
    fn egui(&self) -> Pos2 {
        Pos2::new(self.x as f32, self.y as f32)
    }
}

impl ToEguiPos for Vec2 {
    fn egui(&self) -> Pos2 {
        Pos2::new(self.x, self.y)
    }
}