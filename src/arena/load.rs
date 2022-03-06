use bevy::{prelude::*, utils::label::DynEq};
use rand::{prelude::SliceRandom, thread_rng, Rng};

use crate::{
    config::ConfigAsset, ldtk_loader::*, make_sprite, make_sprite_atlas_sized,
    make_sprite_image_sized, prefab::ChangeSprite, unit::Element, AtlasHandles, GameState,
    LoadCardPrefab, SpawnPrefabOld, LDTK_CARDS_PATH, SETTINGS_PATH, TILE_SIZE, animation::{Animator, AnimationCommand}, AnimationController,
};

use super::{cards::{CardLabel, CardLabelType, CardsAtlas, SpawnCard}, TakingATurn};

pub struct ArenaLoadPlugin;

impl Plugin for ArenaLoadPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CardsAtlas>()
            .add_system_set(SystemSet::on_enter(GameState::LoadArena).with_system(load_data))
            .add_system_set(SystemSet::on_update(GameState::LoadArena).with_system(setup));
    }
}

pub struct LdtkHandles(Vec<Handle<LdtkMap>>);

fn load_data(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    config: Res<Assets<ConfigAsset>>,
) {
    let config = config.get(SETTINGS_PATH).unwrap();
    let handle: Handle<LdtkMap> = asset_server.load(&config.settings.arena_file);
    let cards: Handle<LdtkMap> = asset_server.load(LDTK_CARDS_PATH);
    let handles = vec![handle, cards];
    commands.insert_resource(LdtkHandles(handles));
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
    if let (Some(ldtk), Some(cards_pfb)) = (
        ldtk.get(&config.settings.arena_file),
        ldtk.get(LDTK_CARDS_PATH),
    ) {
        if let Some(bg) = &ldtk.background() {
            let size = bg.size;
            let pos = size / 2;
            //println!("Spawned background, Size {}",  size);
            make_sprite_image_sized(
                &mut commands,
                pos.as_vec2(),
                0,
                bg.image.clone(),
                size,
            );
        }   

        let spawns: Vec<_> = ldtk.get_tagged("spawn_point").collect();

        let player_spawns = spawns.iter().filter(|e|e.tagged("player") && !e.tagged("card"));

        let actions = player_spawns.clone().map(|e|try_get_actions(e.fields(), "attackactions"));

        let actions = actions.take_while(|a|a.is_some()).next().unwrap().unwrap();
        println!("Found {} unit actions: {:?}", actions.len(), actions);

        for (i,spawn) in player_spawns.enumerate() {

            let mut animator = Animator::default();
            if let Some(actions) = try_get_actions(spawn.fields(), "attackactions") {
                //println!("Found attack acionts for {}", spawn.name());
                //animator.add_commands(actions);
            }
            animator.push_cmds_back(actions.clone());
            animator.push_cmds_front([
                AnimationCommand::Play("idle".to_string()),
                AnimationCommand::Wait(1.5 * i as f32)]
            );
 

            let xy = spawn.xy();
            commands.spawn().insert(SpawnPrefabOld {
                path: "units_wizard.ldtk".to_string(),
                xy: xy,
                depth: 10 + i as i32,
                ..Default::default()
            })
            //.insert(TakingATurn)
            .insert(animator)
            ;
        }

        let slimes = spawns.iter().filter(|e|e.tagged("enemy"));


        for (i,spawn) in slimes.rev().enumerate() {
            let xy = spawn.xy();
            commands.spawn().insert(SpawnPrefabOld {
                path: "units_slime.ldtk".to_string(),
                xy: xy,
                depth: 10 + i as i32,
                ..Default::default()
            });
        }

        let cards: Vec<_> = cards_pfb
            .get_tagged("card")
            .filter(|card| {
                if let Some(rarity) = card.fields().try_get_str("rarity") {
                    return rarity == "common" || rarity == "uncommon";
                }
                false
            })
            .collect();

        let spawns = spawns.iter().filter(|e| e.tagged("card"));

        let mut rng = thread_rng();

        for (i,spawn) in spawns.enumerate() {
            let card = cards.choose(&mut rng).unwrap();

            let texture = card.fields().try_get_str("texture").unwrap_or_else(|| {
                panic!(
                    "Error loading card {}, couldn't parse texture field ",
                    spawn.name()
                )
            });

            let tileset = cards_pfb.tileset_from_path(texture).unwrap_or_else(|| {
                panic!(
                    "Error loading tileset, {} wasn't found in the ldtk file",
                    texture
                )
            });

            let tile_id = card
                .fields()
                .try_get_i32("sprite_index")
                .unwrap_or_else(|| panic!("Error loading 'tile_id' from {}", spawn.name()));

            let atlas = get_atlas(&mut atlases, &mut atlas_handles, tileset);

            let xy = spawn.xy();
            //println!("Spawning card {}", card.name());
            commands.spawn().insert(LoadCardPrefab {
                path: "units_BattleCardPremade.ldtk".to_string(),
                xy,
                depth: 5 + i as i32,
                atlas,
                tile_id,
                size: card.size(),
            });
        }

        state.set(GameState::Arena).unwrap();
    }
}

fn get_actions<'a>(
    fields: &'a Fields,
    name: &'a str
) -> impl Iterator<Item=AnimationCommand> + 'a {
    let vals = fields.field("AttackActions").unwrap().as_array().unwrap();
    let vals = vals.iter().map(|v|v.as_str().unwrap().to_lowercase());
    vals.map(|v|ron::de::from_str(&v).unwrap())
}

fn try_get_actions<'a>(
    fields: &'a Fields,
    name: &'a str,
) -> Option<Vec<AnimationCommand>> {
    if let Some(field) = fields.field(name) {
        //println!("Found field {:?}", field);
        if let Some(arr) = field.as_array() {
            //println!("Found arr {:?}", arr);
            let strings = arr.iter().map(|v|v.as_str().unwrap());
            let commands = strings.map(|s|ron::de::from_str::<AnimationCommand>(s));
            return Some(commands.take_while(|c|c.is_ok()).map(|c|c.unwrap()).collect());
        } 
    }
    None
} 

fn play_turn(

) {
    
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
