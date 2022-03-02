use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy_egui::{EguiContext, egui};
use bevy_tiled_camera::TiledProjection;

use crate::GameState;
use crate::unit::Element;
use crate::util::*;

pub struct CardsPlugin;

impl Plugin for CardsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(GameState::Arena)
            .with_system(card_ui)
        )
        ;
    }
}
#[derive(Component)]
pub struct Card;

fn card_ui(
    mut egui: ResMut<EguiContext>,
    q_cam: Query<(&Camera,&TiledProjection,&GlobalTransform)>,
    windows: Res<Windows>,
    q_cards: Query<(Entity, &Transform), With<Card>>,
    q_labels: Query<(Entity, &CardLabel, &CardLabelType, &GlobalTransform)>,
) {
    if let Ok((cam,proj,cam_transform)) = q_cam.get_single() {
        for (entity, label, ltype, ltransform) in q_labels.iter() {
            let mut p = ltransform.translation.xy();
            p.y = -p.y;
            //println!("P {}", p);
            if let Some(p) = proj.world_to_screen(cam, &windows, cam_transform, p.extend(0.0)) {
                
                egui::containers::Area::new(&label.label_name).fixed_pos(p.to_array())
                .show(egui.ctx_mut(), |ui| {
                    ui.label(&label.label_name);
                });
            }
        }
        // for (entity, label, ltype) in q_labels.iter() {
        //     //let p = (label.xy + proj.tile_count().as_vec2() / 2.0);
        //     let mut p = label.xy;
        //     //let mut p = p + proj.tile_count().as_vec2() / 2.0;
        //     p.y = proj.tile_count().as_vec2().y - p.y;
        //     // let p = p * 64.0;
        //     println!("Label {:?} pos {}", ltype, p);
        //     //println!("Running egui?");
        //     if let Some(p) = proj.world_to_screen(cam, &windows, cam_transform, p.extend(0.0)) {
        //         println!("Drawing label {:?} at {}", ltype,  p);
        //         egui::containers::Area::new(&label.label_name).fixed_pos(p.to_array())
        //         .show(egui.ctx_mut(), |ui| {
        //             ui.label(&label.label_name);
        //         });
        //     }

        // }
        // let pos = IVec2::new(2,2);
        // //let pos = IVec2::new(32,32) * scale;
        // if let Some(pos) = proj.world_to_screen(cam, &windows, transform, pos.extend(0).as_vec3()) {

        //     egui::containers::
        //     Area::new("area")
        //     .fixed_pos(pos.egui())
        //     .show(egui.ctx_mut(), |ui| {
        //         ui.label("HELLO");
        //     });
        // }
    }
}


pub enum CardEffect {
    ElementalDamage {
        element: Element,
        damage: i32,
    },
    ElementalShield {
        element: Element,
    }
}

#[derive(Component)]
pub struct CardLabel {
    label_name: String,
    label_content: String,
    area: Rect<f32>,
    xy: Vec2,
}

impl CardLabel {
    pub fn new(name: &str, area: Rect<f32>) -> Self {
        Self {
            label_name: name.to_string(),
            label_content: String::default(),
            area,
            xy: Vec2::new(area.left, area.bottom),
        }
    }

    pub fn xy(&self) -> Vec2 {
        self.xy
    }
}

#[derive(Debug, Component, Copy, Clone, Eq, PartialEq)]
pub enum CardLabelType {
    Title,
    Rarity,
    Ability(i32), // int refers to which ability it is since cards can have multiple
}


#[derive(Component)]
pub struct SpawnCard {
    pub element: Element,
}

// Elements of a card:
/*
root (sprite, area)
    title(label, area)
    rarity(label, area)
    abilities(vec of label,area)

*/
