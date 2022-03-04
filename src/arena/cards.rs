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




#[derive(Default)]
pub struct CardsAtlas(Handle<TextureAtlas>);


// /// Build a card entity with it's labels as child entities
// fn spawn_card(
//     commands: &mut Commands,
//     layer: &EntitiesLayer,
//     atlas: Handle<TextureAtlas>,
//     element: Element,
//     font: Handle<Font>,
// ) -> Entity {
//     let root = layer.get_from_name("card")
//         .expect("Couldn't find prefab 'card' in layer");
//     let title = layer.get_from_name("card_title")
//         .expect("Couldn't find 'card_title' in layer");
//     let rarity = layer.get_from_name("card_rarity")
//         .expect("Couldn't find 'card_rarity' in layer");
//     let abilities = layer.get_all_from_name("card_abilities");

//     let mut xy = root.pixel_xy().as_vec2();
//     let size = root.size().as_vec2();

//     println!("XY: {}", xy);
//     //let root_offset = -(Vec2::new(0.0, size.y) / 2.0);
//     //xy += root_offset;
//     //let root_offset = Vec2::ZERO;//-root_offset * 4.0;
//     println!("Spawning card at {}, size {}", xy, size);
//     let depth = 11;
//     let root = make_sprite_atlas_sized(
//         commands,
//         xy,
//         size,
//         depth,
//         atlas,
//         element.get_sprite_id(),
//     ).id();


//     let title = get_label_entity(commands, title);
//     let rarity = get_label_entity(commands, rarity);

//     let mut children = Vec::new();

//     children.push(title);

//     for (i,ability) in abilities.enumerate() {
//         let ability = get_label_entity(commands, ability);
//     }

//     commands.entity(root).push_children(
//         &children
//     );
//     root
// }


// fn find_entity<'a>(
//     entities: &'a Vec<MapEntity>,
//     name: &str 
// ) -> &'a MapEntity {
//     entities.iter().find(|e|e.name() == name).unwrap_or_else(||
//         panic!("Arena build error: Couldn't find entity '{}'", name)
//     )
// }

// fn find_all_entities<'a>(
//     entities: &'a Vec<MapEntity>,
//     name: &'a str
// ) -> impl Iterator<Item=&'a MapEntity> {
//     entities.iter().filter(move |e|e.name()==name)
// }

// fn get_label_entity(commands: &mut Commands, label: &MapEntity) -> Entity {
//     let xyz = get_label_pos(label);
//     let title = get_label(label, &label.name());
//     commands.spawn().insert(title)
//     .insert(CardLabelType::Title)
//     .insert(Transform::from_translation(xyz))
//     .insert(GlobalTransform::default())
//     .id()
// }

// fn get_label(entity: &MapEntity, name: &str) -> CardLabel {
//     let area = get_area(entity);
//     CardLabel::new(name, area)
// }

// fn get_label_pos(label: &MapEntity) -> Vec3 {
//     // let xy = label.fields.get("offset").unwrap_or_else(||
//     //     panic!("Couldn't find offset fields for entity {}", label.name)
//     // );

//     // let xy = xy.as_array().unwrap_or_else(||
//     //     panic!("Offset field was an unexpected type! Should be an Array"));
//     // let xy = xy.iter().map(|v|v.as_f64().unwrap() as f32);
//     // let xy: Vec<f32> = xy.collect();
//     // Vec2::new(xy[0], xy[1]).extend(0.0)
//     Vec3::ZERO
// }

// fn get_area(entity: &MapEntity) -> Rect<f32> {
//     let xy = entity.pixel_xy().as_vec2();
//     let size = entity.size().as_vec2();
//     //todo!()
//     // let offset = Vec2::new(0.0,0.0);
//     Rect {
//         left: xy.x ,
//         right: xy.x + size.x ,
//         top: xy.y + size.y,
//         bottom: xy.y,
//     }
// }