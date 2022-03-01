use bevy::prelude::*;

use crate::GameState;

use super::ArenaState;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::Arena).with_system(setup));
    }
}

fn setup(asset_server: Res<AssetServer>, mut commands: Commands) {
    // println!("Spawning bg");
    // let bg = asset_server.load("arena_bg.png");
    // let bg = SpriteBundle {
    //     sprite: Sprite {
    //         //color: Color::BLUE,
    //         custom_size: Some(Vec2::new(16.0, 16.0)),
    //         ..Default::default()
    //     },
    //     texture: bg,
    //     ..Default::default()
    // };

    // commands.spawn_bundle(bg);
}
