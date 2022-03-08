use bevy::{math::Vec3Swizzles, prelude::*};
use bevy_egui::{egui::{self, panel::Side}, EguiContext};
use bevy_tiled_camera::TiledProjection;

use crate::{GameState, TILE_SIZE, screen_to_world};

use super::MapUnits;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system_set(SystemSet::on_enter(GameState::BattleMap).with_system(on_enter))
            .add_system_set(SystemSet::on_exit(GameState::BattleMap).with_system(on_exit))
            .add_event::<TileClickedEvent>()
            .add_system_set(SystemSet::on_update(GameState::BattleMap).with_system(cursor_system))
            // .add_startup_system(on_enter)
            // .add_event::<TileClickedEvent>()
            // .add_system(cursor_system)
            ;
    }
}

pub struct TileClickedEvent {
    pub xy: IVec2,
    pub unit: Option<Entity>,
}

#[derive(Component, Default)]
pub struct Cursor;

fn on_enter(mut commands: Commands) {
    //let col = Color::rgba(1.0, 1.0, 1.0, 0.55);

    let sprite_pos = Vec3::ZERO + Vec3::new(0.5,0.5,0.0) * TILE_SIZE as f32;

    let sprite = SpriteBundle {
        sprite: Sprite {
            color: Color::RED,
            custom_size: Some(Vec2::splat(TILE_SIZE as f32)),
            ..Default::default()
        },
        transform: Transform::from_translation(sprite_pos),
        ..Default::default()
    };
    commands.spawn_bundle(sprite)
    .insert(Cursor);
    // let root = commands.spawn()
    // .insert(Transform::default())
    // .insert(GlobalTransform::default())
    // .insert(Cursor)
    // .insert(Visibility::default())
    //.add_child(sprite)
    ;
}

fn on_exit(mut commands: Commands, q_cursor: Query<Entity, With<Cursor>>) {
    for entity in q_cursor.iter() {
        commands.entity(entity).despawn();
    }
}

fn cursor_system(
    input: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    mut q_cursor: Query<(&mut Transform, &mut Visibility), With<Cursor>>,
    mut ev_tile_clicked: EventWriter<TileClickedEvent>,
    units: Res<MapUnits>,
) {
    let window = windows.get_primary().unwrap();

    if let Some(pos) = window.cursor_position() {
        for (cam, global) in q_camera.iter() {
            if let Some(mut p) = screen_to_world(cam, &windows, global, pos) {
                //let mut p = (p / TILE_SIZE as f32).floor();

                let (mut cursor_transform, mut v) = q_cursor.single_mut();
                v.is_visible = true;

                //let mut p = p * TILE_SIZE as f32;
                let xy = (p.xy() / TILE_SIZE as f32).floor();
                let grid_xy = xy.as_ivec2();
                let xy = xy + Vec2::new(0.5,0.5);
                let xy = xy * TILE_SIZE as f32;
                let mut p = xy.extend(30.0);
                cursor_transform.translation = p;

                //println!("Setting cursor pos to {}", p);

                if input.just_pressed(MouseButton::Left) {

                    let unit = units.get_from_grid_xy(grid_xy);
                    println!("Clicked {}. Unit {:?}", grid_xy, unit);
                    ev_tile_clicked.send(TileClickedEvent {
                        xy: grid_xy,
                        unit,
                    });

                }

                return;
            }
        }
    }
    //let (_, mut v) = q_cursor.single_mut();
    //v.is_visible = false;
}


// fn gui(
//     mut egui: ResMut<EguiContext>,
// ) {
//     let ctx = egui.ctx_mut();
//     egui::SidePanel::new(Side::Left, "sidepanel").show(ctx, |ui| {
//         ui.label("hi");
//     });
// }