use bevy::prelude::*;
use bevy_ascii_terminal::{
    ldtk::{LdtkAsset, LdtkPlugin},
    renderer::uv_mapping::UvMapping,
    Terminal, TerminalBundle, TerminalMaterial, TerminalPlugin, TileWriter,
};
use bevy_tiled_camera::{TiledCameraBundle, TiledCameraPlugin, TiledProjection};

use crate::battle_map::map::tile_to_id;

use super::map::Map;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(TerminalPlugin)
            .add_plugin(LdtkPlugin)
            .add_plugin(TiledCameraPlugin)
            .add_startup_system(setup)
            .add_system(setup_from_ldtk.label("term_ldtk_setup"))
            .add_system(render.after("term_ldtk_setup"));
    }
}

#[derive(Component)]
pub struct MapTerminal;

#[derive(Component)]
pub struct MapOverlayTerminal;

fn setup(mut commands: Commands) {
    commands
        .spawn_bundle(TerminalBundle::new())
        .insert(MapTerminal);

    commands
        .spawn_bundle(TerminalBundle {
            transform: Transform::from_xyz(0.0, 0.0, 15.0),
            ..Default::default()
        })
        .insert(MapOverlayTerminal);
    commands.spawn_bundle(TiledCameraBundle::new());
}

pub struct LdtkRebuild {
    pub map_size: UVec2,
    pub tileset_size: UVec2,
    pub tex: Handle<Image>,
}

fn setup_from_ldtk(
    mut q_term: Query<(&Handle<TerminalMaterial>, &mut UvMapping), With<MapTerminal>>,
    mut materials: ResMut<Assets<TerminalMaterial>>,
    mut ev_ldtk: EventReader<LdtkRebuild>,
) {
    for ev in ev_ldtk.iter() {
        println!("SETUP FROM LDTK");
        for (mat, mut mapping) in q_term.iter_mut() {
            if let Some(mut mat) = materials.get_mut(mat) {
                mat.texture = Some(ev.tex.clone());
            }
            *mapping = UvMapping::from_grid(ev.tileset_size);
        }
    }
}

fn render(
    map: Res<Map>,
    mut q_term: Query<&mut Terminal, (With<MapTerminal>, Without<MapOverlayTerminal>)>,
    mut q_cam: Query<&mut TiledProjection>,
    mut q_overlay: Query<&mut Terminal, (With<MapOverlayTerminal>, Without<MapTerminal>)>,
) {
    if let Ok(mut term) = q_term.get_single_mut() {
        if !map.is_changed() {
            return;
        }

        //println!("Detected map change. New Size: {}", map.size());

        if term.size() != map.size() {
            term.resize(map.size());
            if let Ok(mut proj) = q_cam.get_single_mut() {
                proj.set_tile_count(map.size().into());
            }
            if let Ok(mut overlay) = q_overlay.get_single_mut() {
                overlay.resize(map.size());
                overlay.fill('a'.fg(Color::rgba_u8(0,0,0,0)));
            }
        }

        for (i, tile) in map.iter().enumerate() {
            let id = tile_to_id(tile);
            let xy = term.to_xy(i);
            term.put_tile(xy, id);
        }
    }
}
