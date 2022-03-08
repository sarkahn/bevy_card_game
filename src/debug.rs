use bevy::prelude::*;
use bevy_egui::{EguiContext, egui};

use crate::{screen_to_world, TILE_SIZE};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(debug_panel);
    }
}

fn debug_panel(
    mut egui: ResMut<EguiContext>,
    windows: Res<Windows>,
    q_camera: Query<(&Camera, &GlobalTransform, &OrthographicProjection)>,
) {
    let mut ctx = egui.ctx_mut();

    let window = windows.get_primary().unwrap();
    if let Some(pos) = window.cursor_position() {
        if let Ok((cam, global, proj)) = q_camera.get_single() {
            if let Some(pos) = screen_to_world(cam, &windows, global, pos) {
                egui::panel::SidePanel::right("RightPanel").show(ctx, |ui| {
                    let pos = (pos / TILE_SIZE as f32).floor();
                    ui.label(pos.to_string());
                });
            }
        }
    }
}

