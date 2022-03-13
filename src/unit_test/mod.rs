use bevy::prelude::*;
use bevy_egui::{EguiContext, egui};

use crate::{GameState, prefab::Prefabs, BuildPrefab, ArenaSpriteVisibility, unit::SetPosition};

pub struct UnitTestPlugin;

impl Plugin for UnitTestPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(GameState::AssetTest)
            .with_system(setup)
        )
        .add_system_set(
            SystemSet::on_update(GameState::AssetTest)
            .with_system(gui)
            .with_system(load)
        )
        .init_resource::<UIState>()
        .init_resource::<LoadedUnit>()
        ;
    }
}

#[derive(Default)]
struct UIState {
    selected: usize,
    prefab_names: Vec<String>,
}

#[derive(Default)]
struct LoadedUnit(Option<Entity>);

fn setup(
    mut commands: Commands,
    mut state: ResMut<UIState>,
    prefabs: Res<Prefabs>,
) {
    state.prefab_names = prefabs.iter_units().map(|(name,_)|name).cloned().collect();
}

fn load(
    mut commands: Commands,
    mut loaded: ResMut<LoadedUnit>,
    state: Res<UIState>,
) {
    if loaded.0.is_none() {
        let name = state.prefab_names[state.selected].to_string();
        println!("Spawning BuildPrefab {}", name);
        let entity = commands.spawn().insert(BuildPrefab {
            name
        })
        .insert(ArenaSpriteVisibility(true))
        .insert(SetPosition(Vec3::new(1856.0 / 2.0, 1024.0 / 2.0,5.0)))
        .id();
        loaded.0 = Some(entity);
    }
}

fn gui(
    mut egui: ResMut<EguiContext>,
    prefabs: Res<Prefabs>,
    mut state: ResMut<UIState>,
) {
    let mut ctx = egui.ctx_mut();

    let len = state.prefab_names.len();
    egui::panel::SidePanel::left("leftpanel").show(ctx, |ui| {
        let names = state.prefab_names.to_owned();
        let response = egui::ComboBox::from_label("Load prefab")
        // .show_ui( ui, |ui| {
        //     ui.show_index()
        //     // for (name, prefab) in prefabs.iter_units() {
        //     //     let text = state.selected.to_owned();
        //     //     ui.selectable_value(&mut state.selected, text, text);
        //     //     //ui.selectable_label()
        //     // }
        // });
        .show_index(
            ui,
            &mut state.selected,
            len,
            |i| names[i].to_owned()
        );

        if response.changed() {
            println!("Changed to {}!", state.prefab_names[state.selected]);
        }
    });
}