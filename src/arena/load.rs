use bevy::prelude::*;


use crate::{
    config::ConfigAsset, ldtk_loader::*,
    AtlasHandles, GameState, SETTINGS_PATH, make_sprite_image_sized, make_sprite_atlas_sized, unit::Element, make_sprite,
};

use super::cards::{CardLabel, CardLabelType, SpawnCard};


pub struct ArenaLoadPlugin;

impl Plugin for ArenaLoadPlugin {
    fn build(&self, app: &mut App) {
        app
        .init_resource::<CardsAtlas>()
        .add_system_set(
            SystemSet::on_enter(GameState::LoadArena).with_system(load_data)
        )
        .add_system_set(
            SystemSet::on_update(GameState::LoadArena).with_system(setup)
        )
        .add_system_set(
            SystemSet::on_update(GameState::Arena).with_system(on_spawn)
        )
        ;
    }
}

fn load_data(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    config: Res<Assets<ConfigAsset>>,
) {
    let config = config.get(SETTINGS_PATH).unwrap();
    let handle: Handle<LdtkMap> = asset_server.load(&config.settings.arena_file);
    commands.insert_resource(handle);
    
}

fn setup(
    mut commands: Commands,
    mut state: ResMut<State<GameState>>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
    mut atlas_handles: ResMut<AtlasHandles>,
    mut card_atlas: ResMut<CardsAtlas>,
    ldtk: Res<Assets<LdtkMap>>,
    config: Res<Assets<ConfigAsset>>,
) {
    let config = config.get(SETTINGS_PATH).unwrap();
    if let Some(ldtk) = ldtk.get(&config.settings.arena_file) {

        if let Some(bg) = &ldtk.background {
            //println!("Spawned background, Size {}",  bg.size);
            let size = bg.size;
            let size = size / 64;
            make_sprite_image_sized(
                &mut commands, 
                Vec2::ZERO,
                10,
                bg.image.clone(),
                size,
            );
        }

        let card_tileset = ldtk.tileset_from_name("Battle_Cards").expect(
            "Couldn't find 'Battle_Cards' tileset in arena ldtk file!"
        );

        let atlas = get_atlas(&mut atlases, &mut atlas_handles, &card_tileset);
        card_atlas.0 = atlas;

        state.set(GameState::Arena).unwrap();

        commands.spawn().insert(SpawnCard {
            element: Element::Death,
        });
    }
}

fn on_spawn(
    mut commands: Commands, 
    ldtk: Res<Assets<LdtkMap>>,
    config: Res<Assets<ConfigAsset>>,
    atlas: Res<CardsAtlas>,
    q_spawn: Query<(Entity,&SpawnCard)>,
    asset_server: Res<AssetServer>,
) {
    if let Some(config) = config.get(SETTINGS_PATH) {
        if let Some(ldtk) = ldtk.get(&config.settings.arena_file) {
            for (entity,spawn) in q_spawn.iter() {
                commands.entity(entity).despawn();

                let data_layer = ldtk.layer_from_name("Card_Data").unwrap_or_else(||
                    panic!("Couldn't find 'Card_Data' in ldtk file {}", ldtk.name));
                
                let entities = data_layer.as_entities();
             
                let font = asset_server.load("fonts/DejaVuSerif.ttf");
                spawn_card(&mut commands, entities, atlas.0.clone(), spawn.element, font);
            }
        }
    }
}

#[derive(Default)]
pub struct CardsAtlas(Handle<TextureAtlas>);

/// Build a card entity with it's labels as child entities
fn spawn_card(
    commands: &mut Commands,
    layer: &EntitiesLayer,
    atlas: Handle<TextureAtlas>,
    element: Element,
    font: Handle<Font>,
) -> Entity {
    let root = layer.get_from_name("card")
        .expect("Couldn't find prefab 'card' in layer");
    let title = layer.get_from_name("card_title")
        .expect("Couldn't find 'card_title' in layer");
    let rarity = layer.get_from_name("card_rarity")
        .expect("Couldn't find 'card_rarity' in layer");
    let abilities = layer.get_all_from_name("card_abilities");

    let mut xy = get_xy(root);
    let mut xy = Vec2::new(3.0, 3.0);
    let size = get_size(root);
    //let root_offset = -(Vec2::new(0.0, size.y) / 2.0);
    //xy += root_offset;
    //let root_offset = Vec2::ZERO;//-root_offset * 4.0;

    println!("Spawning card at {}, size {}", xy, size);
    let depth = 11;
    let root = make_sprite_atlas_sized(
        commands,
        xy,
        size,
        depth,
        atlas,
        element.get_sprite_id(),
    ).id();

    let title_offset = title.fields.get("offset").as_ref().unwrap().as_array().unwrap();
    let title_offset = title_offset.iter().map(|v|v.as_f64().unwrap() as f32);
    let xy: Vec<f32> = title_offset.collect();
    let xy = Vec2::new(xy[0], xy[1]);
    println!("XY {:?}", xy);
    let title = get_label(title, "a");
    let title = commands.spawn().insert(title)
    .insert(CardLabelType::Title)
    .insert(Transform::from_xyz(xy.x, xy.y, 0.0))
    .insert(GlobalTransform::default())
    .id();

    //let title = get_label(title, "Title", root_offset);
    //let title = commands.spawn().insert(title)
    //.insert(CardLabelType::Title)
    // .insert_bundle(Text2dBundle {
    //     text: Text::with_section(
    //         "Hello",
    //         style.clone(),
    //         text_alignment,
    //     ),
    //     transform: ttransform,
    //     ..Default::default()
    // })
    //.id();

    // let rarity = get_label(rarity, "Rarity", root_offset);
    // let xy = rarity.xy();
    // let rarity = commands.spawn().insert(rarity)
    // .insert(Transform::from_translation(xy.extend(0.0)))
    // .insert(CardLabelType::Rarity)
    // .id();

    let mut children = Vec::new();

    children.push(title);

    //make_sprite(commands, xy, 20, Color::ORANGE);
    // children.push(rarity);

    // for (i,ability) in abilities.enumerate() {
    //     let ability = get_label(ability, &format!("Ability{}", i), root_offset);
    //     let ability = commands.spawn().insert(ability)
    //     .insert(CardLabelType::Ability(i as i32))
    //     .id();
    //     children.push(ability);
    // } 

    commands.entity(root).push_children(
        &children
    );
    root
}


fn find_entity<'a>(
    entities: &'a Vec<MapEntity>,
    name: &str 
) -> &'a MapEntity {
    entities.iter().find(|e|e.name == name).unwrap_or_else(||
        panic!("Arena build error: Couldn't find entity '{}'", name)
    )
}

fn find_all_entities<'a>(
    entities: &'a Vec<MapEntity>,
    name: &'a str
) -> impl Iterator<Item=&'a MapEntity> {
    entities.iter().filter(move |e|e.name==name)
}


fn get_label(entity: &MapEntity, name: &str) -> CardLabel {
    let area = get_area(entity);
    CardLabel::new(name, area)
}

fn get_area(entity: &MapEntity) -> Rect<f32> {
    let xy = get_xy(entity);
    let size = get_size(entity);
    let offset = Vec2::new(0.0,0.0);
    Rect {
        left: xy.x + offset.x,
        right: xy.x + size.x + offset.x,
        top: xy.y + size.y + offset.y,
        bottom: xy.y + offset.y,
    }
}

fn get_xy(entity: &MapEntity) -> Vec2 {
    entity.xy.as_vec2() / 64.0
}

fn get_size(entity: &MapEntity) -> Vec2 {
    entity.size.as_vec2() / 64.0
}

fn get_atlas(
    atlases: &mut Assets<TextureAtlas>,
    atlas_handles: &mut AtlasHandles,
    tileset: &MapTileset,
) -> Handle<TextureAtlas> {
    let name = &tileset.name;
    match atlas_handles.0.get(name) {
        Some(atlas) => atlas.clone(),
        None => {
            let atlas = TextureAtlas::from_grid(
                tileset.image.clone(),
                IVec2::splat(tileset.tile_size).as_vec2(),
                tileset.tile_count.x as usize,
                tileset.tile_count.y as usize,
            );
            let handle = atlases.add(atlas);
            atlas_handles.0.insert(name.to_string(), handle.clone());
            handle
        }
    }
}

