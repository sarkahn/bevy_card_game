use bevy::{prelude::*, ecs::system::EntityCommands, utils::HashMap};

use crate::{GameState, AtlasHandles, ldtk_loader::*, UnitAnimation, config::ConfigAsset, SETTINGS_PATH};

use super::ArenaState;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_update(GameState::LoadArena).with_system(setup));
    }
}

fn setup(
    mut commands: Commands,
    config: Res<Assets<ConfigAsset>>,
    ldtk_maps: Res<Assets<LdtkMap>>,
    asset_server: Res<AssetServer>,
) {
    if let Some(config) = config.get(SETTINGS_PATH) {
        let ldtk_handle: Handle<LdtkMap> = asset_server.load(&config.settings.arena_file);
        if let Some(ldtk) = ldtk_maps.get(ldtk_handle.clone()) { 

        }
    }
        println!("Load arena!");
    // println!("Spawning bg");
    //let bg = asset_server.load("arena_bg.png");
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

fn spawn_tile(
    commands: &mut Commands,
    tile: &MapTile,
    axis_offset: Vec2,
    depth: usize,
    atlas: Handle<TextureAtlas>,
    tileset: &MapTileset,
) {
    let xy = tile.xy.as_vec2() + axis_offset;

    let transform = Transform::from_xyz(xy.x, xy.y, depth as f32);
    let sprite = TextureAtlasSprite {
        custom_size: Some(Vec2::ONE),
        index: tile.id as usize,
        ..Default::default()
    };
    let sprite = SpriteSheetBundle {
        sprite,
        texture_atlas: atlas,
        transform,
        ..Default::default()
    };

    commands.spawn_bundle(sprite);
    if let Some(enums) = tileset.enums.get(&tile.id) {
        //println!("Found enums for tileset {}: {:?}", tileset.name, enums);
        if enums.iter().any(|s|s=="collider") {

        }
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