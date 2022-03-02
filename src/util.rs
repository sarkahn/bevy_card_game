use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_egui::egui::Pos2;

pub fn make_sprite<'w, 's, 'a>(
    commands: &'a mut Commands<'w, 's>,
    xy: Vec2,
    depth: i32,
    color: Color,
) -> EntityCommands<'w, 's, 'a> {
    let transform = Transform::from_translation(Vec3::new(xy.x, xy.y, depth as f32));
    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            color: color,
            custom_size: Some(Vec2::ONE),
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
    let transform = Transform::from_translation(Vec3::new(xy.x, xy.y, depth as f32));
    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            color: color,
            custom_size: Some(size),
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
    let transform = Transform::from_translation(Vec3::new(xy.x, xy.y, depth as f32));
    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::ONE),
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
    let transform = Transform::from_translation(Vec3::new(xy.x, xy.y, depth as f32));
    let sprite = TextureAtlasSprite {
        custom_size: Some(Vec2::ONE),
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


pub fn make_sprite_atlas_sized<'w, 's, 'a>(
    commands: &'a mut Commands<'w, 's>,
    xy: Vec2,
    size: Vec2,
    depth: i32,
    atlas: Handle<TextureAtlas>,
    index: usize,
) -> EntityCommands<'w, 's, 'a> {
    let transform = Transform::from_translation(Vec3::new(xy.x, xy.y, depth as f32));
    let sprite = TextureAtlasSprite {
        custom_size: Some(size),
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