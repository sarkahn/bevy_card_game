use bevy::{prelude::*, ecs::system::EntityCommands};

pub fn make_sprite<'w, 's, 'a>(
    commands: &'a mut Commands<'w, 's>, 
    xy: Vec2, color: Color
) -> EntityCommands<'w, 's, 'a> {
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: color,
                custom_size: Some(Vec2::ONE),
                ..Default::default()
            },
            transform: Transform::from_xyz(xy.x, xy.y, 2.0),
            ..Default::default()
        })
}