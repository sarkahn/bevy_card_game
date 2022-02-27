use bevy::prelude::*;
use bevy_ascii_terminal::{
    ldtk::{LdtkAsset, LdtkPlugin},
    renderer::uv_mapping::UvMapping,
    Terminal, TerminalBundle, TerminalMaterial, TerminalPlugin, TileWriter,
};

use crate::{ResizeCamera};

use super::map::Map;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(TerminalPlugin)
            .add_plugin(LdtkPlugin)
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

    // commands
    //     .spawn_bundle(TerminalBundle {
    //         transform: Transform::from_xyz(0.0, 0.0, 15.0),
    //         ..Default::default()
    //     })
    //     .insert(MapOverlayTerminal);
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
    mut commands: Commands,
    map: Res<Map>,
    mut q_term: Query<&mut Terminal, (With<MapTerminal>, Without<MapOverlayTerminal>)>,
    mut q_overlay: Query<&mut Terminal, (With<MapOverlayTerminal>, Without<MapTerminal>)>,
) {
    if let Ok(mut term) = q_term.get_single_mut() {
        if !map.is_changed() {
            return;
        }

        //println!("Detected map change. New Size: {}", map.size());

        if term.size() != map.size() {
            term.resize(map.size());
            term.fill(0);
            commands.spawn().insert(ResizeCamera(map.size().as_ivec2()));
            if let Ok(mut overlay) = q_overlay.get_single_mut() {
                overlay.resize(map.size());
                overlay.fill('a'.fg(Color::rgba_u8(0,0,0,0)));
            }
        }

        for (i, tile) in map.iter().enumerate() {
            if let Some(id) = map.tile_id(*tile) {
                println!("Putting tile {:?}", tile);
                let xy = term.to_xy(i);
                term.put_tile(xy, *id as u16);
            }
        }
    }
}
