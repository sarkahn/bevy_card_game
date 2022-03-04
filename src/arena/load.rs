use bevy::prelude::*;
use rand::prelude::ThreadRng;


use crate::{
    config::ConfigAsset, ldtk_loader::*,
    AtlasHandles, GameState, SETTINGS_PATH, make_sprite_image_sized, make_sprite_atlas_sized, unit::Element, make_sprite, TILE_SIZE, LoadPrefab,
};

use super::cards::{CardLabel, CardLabelType, SpawnCard, CardsAtlas};


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

        if let Some(bg) = &ldtk.background() {
            let size = bg.size;
            let pos = size / 2;
            println!("Spawned background, Size {}",  size);
            make_sprite_image_sized(
                &mut commands, 
                pos.as_vec2(),
                0,
                bg.image.clone(),
                size,
            );
        }

        for ts in ldtk.tilesets() {
            println!("{}", ts.name);
        }
        let card_tileset = ldtk.tileset_from_name("Battle_Cards").unwrap_or_else(||
            panic!("Couldn't find 'Battle_Cards' tileset in {} file {}", "Battle_Cards", ldtk.name())
        );

        let atlas = get_atlas(&mut atlases, &mut atlas_handles, &card_tileset);

        let spawns: Vec<_> = ldtk.get_tagged("spawn_point").collect();

        let player_spawns = spawns.iter().filter(|e|e.tagged("player"));

        for spawn in player_spawns {
            let xy = spawn.xy();
            commands.spawn().insert(LoadPrefab {
                path: "units_wizard.ldtk".to_string(),
                xy: xy,
                depth: 10,
            });
        }
        
        let player_spawns = spawns.iter().filter(|e|e.tagged("enemy"));


        for spawn in player_spawns {
            let xy = spawn.xy();
            commands.spawn().insert(LoadPrefab {
                path: "units_slime.ldtk".to_string(),
                xy: xy,
                depth: 10,
            });
        }

        state.set(GameState::Arena).unwrap();
    }
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

